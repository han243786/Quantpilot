import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act } from "@testing-library/react";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";

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

  it("formats SSE disconnects as a structured runtime failure", async () => {
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
    expect(useGraphStore.getState().runtime.backendError).toBeTruthy();
  });
});
