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
    const fakeSource = {
      listeners: new Map(),
      addEventListener(type, handler) {
        this.listeners.set(type, handler);
      },
      close: vi.fn(),
      onerror: null
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
    createRuntimeEventSource.mockReturnValue(fakeSource);

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

    await act(async () => {
      await fakeSource.onerror();
    });

    expect(useGraphStore.getState().runtime.status).toBe("error");
    expect(createRuntimeEventSource).toHaveBeenCalledWith("run_sse_001");
    expect(runtimeStartBody.capability_context).toEqual({
      schema_hash: backendCapabilitiesFixture.schema_hash,
      permission_boundary: backendCapabilitiesFixture.permission_boundary
    });
    expect(useGraphStore.getState().runtime.backendError).toBeTruthy();
  });
});
