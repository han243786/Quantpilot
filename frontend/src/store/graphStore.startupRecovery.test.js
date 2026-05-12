import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useGraphStore } from "./graphStore";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";

function buildGraph(registry, mutate = null) {
  return buildValidatedSampleGraph(registry, mutate);
}

describe("graphStore startup recovery paths", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    useGraphStore.setState(initialState, true);
    window.localStorage.clear();
    vi.unstubAllGlobals();
  });

  afterEach(() => {
    useGraphStore.setState(initialState, true);
    window.localStorage.clear();
    vi.unstubAllGlobals();
  });

  it("formats non-runnable latest graph recovery failures as reason plus next action", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: true,
        json: async () =>
          buildGraph(useGraphStore.getState().registry, (graph) => {
            graph.metadata.name = "Broken latest graph";
            graph.metadata.graph_id = "broken_latest_graph";
            graph.nodes = [];
            graph.edges = [];
          })
      })
    );

    const result = await useGraphStore.getState().recoverLatestRunnableGraph();

    expect(result).toBeNull();
    expect(useGraphStore.getState().runtime.backendError).toContain(
      "原因：Latest saved graph is not runnable yet. 后续：检查后端可用性以及是否存在已保存的可运行策略图后，再重新加载编辑器。"
    );
  });

  it("surfaces startup recovery errors during initialize while keeping the stored runnable graph", async () => {
    const storedGraph = buildGraph(useGraphStore.getState().registry, (graph) => {
      graph.metadata.name = "Stored Runnable Graph";
      graph.metadata.graph_id = "stored_runnable_graph";
    });
    window.localStorage.setItem("quantpilot_frontend_graph", JSON.stringify(storedGraph));

    const recoverLatestRunnableGraph = vi.fn(async () => null);
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: false,
        status: 503,
        text: async () => "backend unavailable"
      })
    );

    useGraphStore.setState({
      refreshCapabilities: vi.fn(async () => useGraphStore.getState().capabilities),
      refreshGraphIndex: vi.fn(async () => [
        {
          graph_id: "stored_runnable_graph",
          name: "Stored Runnable Graph",
          updated_at: 1710000000000,
          path: "storage/graphs/stored_runnable_graph.qs"
        }
      ]),
      refreshRunHistory: vi.fn(async () => []),
      refreshBacktestHistory: vi.fn(async () => []),
      recoverLatestRunnableGraph
    });

    await useGraphStore.getState().initialize();

    expect(useGraphStore.getState().graph.metadata.name).toBe("Stored Runnable Graph");
    expect(useGraphStore.getState().runtime.backendError).toContain(
      "原因：backend unavailable 后续：检查后端可用性以及是否存在已保存的可运行策略图后，再重新加载编辑器。"
    );
    expect(recoverLatestRunnableGraph).toHaveBeenCalledTimes(1);
  });

  it("v1.0.5: accepts stored runnable graphs even when missing from backend index (recovery from refresh)", async () => {
    const storedGraph = buildGraph(useGraphStore.getState().registry, (graph) => {
      graph.metadata.name = "Ghost Runnable Graph";
      graph.metadata.graph_id = "ghost_strategy";
    });
    window.localStorage.setItem("quantpilot_frontend_graph", JSON.stringify(storedGraph));

    useGraphStore.setState({
      refreshCapabilities: vi.fn(async () => useGraphStore.getState().capabilities),
      refreshGraphIndex: vi.fn(async () => [
        {
          graph_id: "real_strategy",
          name: "Real strategy",
          updated_at: 1710000000000,
          path: "storage/graphs/real_strategy.qs"
        }
      ]),
      refreshRunHistory: vi.fn(async () => []),
      refreshBacktestHistory: vi.fn(async () => []),
      recoverLatestRunnableGraph: vi.fn(async () => null)
    });

    await useGraphStore.getState().initialize();

    // v1.0.5: localStorage graph is now accepted as third fallback even when not in backend index
    expect(useGraphStore.getState().graph.metadata.graph_id).toBe("ghost_strategy");
  });
});
