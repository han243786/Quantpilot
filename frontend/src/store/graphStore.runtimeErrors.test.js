import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act } from "@testing-library/react";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";
import { backendCapabilitiesFixture } from "../test/fixtures/capabilities/capabilityFallbacks";

const { createRuntimeEventSource } = vi.hoisted(() => ({
  createRuntimeEventSource: vi.fn()
}));

vi.mock("./graphStoreRuntimeTransport", () => ({
  createRuntimeEventSource
}));

import { useGraphStore } from "./graphStore";

describe("graphStore passive runtime error paths", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    useGraphStore.setState(initialState, true);
    useGraphStore.setState({
      graph: buildValidatedSampleGraph(initialState.registry)
    });
    window.localStorage.clear();
    vi.unstubAllGlobals();
    createRuntimeEventSource.mockReset();
  });

  afterEach(() => {
    useGraphStore.setState(initialState, true);
    window.localStorage.clear();
    vi.useRealTimers();
    vi.unstubAllGlobals();
    createRuntimeEventSource.mockReset();
  });

  it("formats run history load failures as reason plus next action", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: false,
        status: 503,
        text: async () => "backend unavailable"
      })
    );

    await useGraphStore.getState().refreshRunHistory();

    expect(useGraphStore.getState().runtime.historyStatus).toBe("error");
    expect(useGraphStore.getState().runtime.backendError).toContain("backend unavailable");
  });

  it("formats backtest history load failures as reason plus next action", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: false,
        status: 503,
        text: async () => "backend unavailable"
      })
    );

    await useGraphStore.getState().refreshBacktestHistory();

    expect(useGraphStore.getState().runtime.backtestHistoryStatus).toBe("error");
    expect(useGraphStore.getState().runtime.backendError).toContain("backend unavailable");
  });

  it("blocks runtime start before compile when capability governance is unsafe", async () => {
    const { permission_boundary: _permissionBoundary, ...malformedCapabilities } =
      backendCapabilitiesFixture;
    const compileCurrentGraph = vi.fn();
    const fetchMock = vi.fn();
    vi.stubGlobal("fetch", fetchMock);

    act(() => {
      useGraphStore.setState({
        capabilities: malformedCapabilities,
        capabilityStatus: "ready",
        capabilitySource: "remote",
        capabilityMessage: "",
        actionLock: null,
        compileCurrentGraph
      });
    });

    await act(async () => {
      await useGraphStore.getState().startRuntime();
    });

    expect(compileCurrentGraph).not.toHaveBeenCalled();
    expect(fetchMock).not.toHaveBeenCalled();
    expect(useGraphStore.getState().runtime.status).toBe("error");
    expect(useGraphStore.getState().runtime.backendError).toContain(
      "缺少 permission_boundary"
    );
  });

  it("blocks backtest before compile when capability fetch fell back to cache", async () => {
    const compileCurrentGraph = vi.fn();
    const fetchMock = vi.fn();
    vi.stubGlobal("fetch", fetchMock);

    act(() => {
      useGraphStore.setState({
        capabilities: backendCapabilitiesFixture,
        capabilityStatus: "degraded",
        capabilitySource: "cache",
        capabilityMessage: "Capability fetch failed.",
        compileCurrentGraph
      });
    });

    await act(async () => {
      await useGraphStore.getState().startBacktest();
    });

    expect(compileCurrentGraph).not.toHaveBeenCalled();
    expect(fetchMock).not.toHaveBeenCalled();
    expect(useGraphStore.getState().runtime.runKind).toBe("backtest");
    expect(useGraphStore.getState().runtime.status).toBe("error");
    expect(useGraphStore.getState().runtime.backendError).toContain("缓存能力快照");
  });

  it("formats SSE disconnects as a structured runtime failure", async () => {
    // v1.0.0: SSE 断开后自动重连, 仅重连耗尽时设 failure 状态
    let exhaustCallback = null;
    const fakeSource = {
      listeners: new Map(),
      addEventListener(type, handler) {
        this.listeners.set(type, handler);
      },
      close: vi.fn(),
      onerror: null,
      _reconnect: () => { exhaustCallback?.(); }
    };

    let runtimeStartBody = null;
    const fetchMock = vi.fn(async (url, options = {}) => {
      if (url.endsWith("/api/runtime/test-run")) {
        runtimeStartBody = JSON.parse(String(options.body || "{}"));
        return {
          ok: true,
          json: async () => ({ run_id: "run_sse_001" })
        };
      }
      if (url.endsWith("/api/runtime/runs")) {
        return {
          ok: true,
          json: async () => []
        };
      }
      throw new Error(`Unexpected fetch: ${url}`);
    });

    vi.stubGlobal("fetch", fetchMock);
    createRuntimeEventSource.mockImplementation((runId, onExhausted) => {
      exhaustCallback = onExhausted;
      return fakeSource;
    });

    act(() => {
      useGraphStore.setState({
        compileCurrentGraph: vi.fn(async () => ({
          compile_id: "compile_sse_001",
          runtime_config: {
            metadata: { graph_id: "graph_sse_001", compile_id: "compile_sse_001" }
          },
          backend_compile: { compile_id: "compile_sse_001" },
          runtime_targets: {
            source_to_node: {},
            runtime_node_id: null,
            execution_node_id: null
          }
        }))
      });
    });

    await act(async () => {
      await useGraphStore.getState().startRuntime();
    });

    // 模拟 SSE 断开 → 触发重连 → 重连耗尽
    await act(async () => {
      await fakeSource.onerror();
    });
    // 重连耗尽回调被源调用
    expect(exhaustCallback).toBeTruthy();
    exhaustCallback();

    expect(useGraphStore.getState().runtime.status).toBe("error");
    expect(createRuntimeEventSource).toHaveBeenCalledWith("run_sse_001", expect.any(Function), expect.any(Function));
    expect(runtimeStartBody.capability_context).toEqual({
      schema_hash: backendCapabilitiesFixture.schema_hash,
      permission_boundary: backendCapabilitiesFixture.permission_boundary
    });
    expect(useGraphStore.getState().runtime.backendError).toBeTruthy();
  });

  it("keeps completion after run_completed arrives before the event batch timer", async () => {
    vi.useFakeTimers();
    const fakeSource = {
      listeners: new Map(),
      addEventListener(type, handler) {
        this.listeners.set(type, handler);
      },
      close: vi.fn(),
      onerror: null
    };
    const fetchMock = vi.fn(async (url) => {
      if (url.endsWith("/api/runtime/test-run")) {
        return {
          ok: true,
          json: async () => ({ run_id: "run_batch_001" })
        };
      }
      throw new Error(`Unexpected fetch: ${url}`);
    });

    vi.stubGlobal("fetch", fetchMock);
    createRuntimeEventSource.mockReturnValue(fakeSource);
    act(() => {
      useGraphStore.setState({
        capabilities: backendCapabilitiesFixture,
        capabilityStatus: "ready",
        capabilitySource: "remote",
        capabilityMessage: "",
        actionLock: null,
        compileCurrentGraph: vi.fn(async () => ({
          compile_id: "compile_batch_001",
          runtime_config: {
            metadata: { graph_id: "graph_batch_001", compile_id: "compile_batch_001" }
          },
          backend_compile: { compile_id: "compile_batch_001" },
          runtime_targets: {
            source_to_node: {},
            runtime_node_id: null,
            execution_node_id: null
          }
        }))
      });
    });

    await act(async () => {
      await useGraphStore.getState().startRuntime();
    });

    await act(async () => {
      fakeSource.listeners.get("runtime_event")({
        data: JSON.stringify({
          event_id: "evt_batch_001",
          event_type: "DataUpdated",
          node_id: "node_data_2",
          event_time_ms: 1_700_000_000_000,
          severity: "Info",
          summary: "batched event",
          payload: {}
        })
      });
      await fakeSource.listeners.get("run_completed")({
        data: JSON.stringify({ run_id: "run_batch_001", status: "completed" })
      });
    });

    expect(useGraphStore.getState().runtime.status).toBe("completed");
    expect(useGraphStore.getState().runtime.artifactPersistenceStatus).toBe("transient");
    expect(useGraphStore.getState().runtime.runId).toBe("run_batch_001");

    act(() => {
      vi.runOnlyPendingTimers();
    });

    expect(useGraphStore.getState().runtime.status).toBe("completed");
    expect(useGraphStore.getState().runtime.events).toHaveLength(1);
  });

  it("compiles before acquiring the runtime lock when starting simulation", async () => {
    let lockObservedDuringCompile = "unset";
    const compileCurrentGraph = vi.fn(async () => {
      lockObservedDuringCompile = useGraphStore.getState().actionLock;
      return {
        compile_id: "compile_lock_001",
        runtime_config: {
          metadata: { graph_id: "graph_lock_001", compile_id: "compile_lock_001" }
        },
        backend_compile: { compile_id: "compile_lock_001" },
        runtime_targets: {
          source_to_node: {},
          runtime_node_id: null,
          execution_node_id: null
        }
      };
    });
    const fetchMock = vi.fn(async (url) => {
      if (url.endsWith("/api/runtime/test-run")) {
        return {
          ok: false,
          status: 400,
          text: async () =>
            JSON.stringify({
              error: "capability_rejected",
              message: "runtime rejected after compile"
            })
        };
      }
      throw new Error(`Unexpected fetch: ${url}`);
    });

    vi.stubGlobal("fetch", fetchMock);
    act(() => {
      useGraphStore.setState({
        capabilities: backendCapabilitiesFixture,
        capabilityStatus: "ready",
        capabilitySource: "remote",
        capabilityMessage: "",
        actionLock: null,
        compileCurrentGraph
      });
    });

    await act(async () => {
      await useGraphStore.getState().startRuntime();
    });

    expect(lockObservedDuringCompile).toBeNull();
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining("/api/runtime/test-run"),
      expect.any(Object)
    );
    expect(useGraphStore.getState().runtime.status).toBe("error");
    expect(useGraphStore.getState().runtime.backendError).toContain(
      "runtime rejected after compile"
    );
  });

  it("compiles before acquiring the runtime lock when starting a backtest", async () => {
    let lockObservedDuringCompile = "unset";
    const compileCurrentGraph = vi.fn(async () => {
      lockObservedDuringCompile = useGraphStore.getState().actionLock;
      return {
        compile_id: "compile_backtest_lock_001",
        runtime_config: {
          metadata: {
            graph_id: "graph_backtest_lock_001",
            compile_id: "compile_backtest_lock_001"
          }
        },
        backend_compile: { compile_id: "compile_backtest_lock_001" },
        runtime_targets: {
          source_to_node: {},
          runtime_node_id: null,
          execution_node_id: null
        }
      };
    });
    const fetchMock = vi.fn(async (url) => {
      if (url.endsWith("/api/runtime/backtest")) {
        return {
          ok: false,
          status: 400,
          text: async () =>
            JSON.stringify({
              error: "capability_rejected",
              message: "backtest rejected after compile"
            })
        };
      }
      throw new Error(`Unexpected fetch: ${url}`);
    });

    vi.stubGlobal("fetch", fetchMock);
    act(() => {
      useGraphStore.setState({
        capabilities: backendCapabilitiesFixture,
        capabilityStatus: "ready",
        capabilitySource: "remote",
        capabilityMessage: "",
        actionLock: null,
        compileCurrentGraph
      });
    });

    await act(async () => {
      await useGraphStore.getState().startBacktest();
    });

    expect(lockObservedDuringCompile).toBeNull();
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining("/api/runtime/backtest"),
      expect.any(Object)
    );
    expect(useGraphStore.getState().runtime.runKind).toBe("backtest");
    expect(useGraphStore.getState().runtime.status).toBe("error");
    expect(useGraphStore.getState().runtime.backendError).toContain(
      "backtest rejected after compile"
    );
  });
});
