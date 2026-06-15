import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act } from "@testing-library/react";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";
import { backendCapabilitiesFixture } from "../test/fixtures/capabilities/capabilityFallbacks";


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
  });

  afterEach(() => {
    useGraphStore.setState(initialState, true);
    window.localStorage.clear();
    vi.unstubAllGlobals();
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
      await useGraphStore.getState().startV4Simulation();
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
