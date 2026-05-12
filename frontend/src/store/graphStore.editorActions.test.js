import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useGraphStore } from "./graphStore";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";

describe("graphStore editor actions", () => {
  const initialState = useGraphStore.getState();
  const registry = initialState.registry;

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

  describe("createNode", () => {
    it("adds a node to the graph", () => {
      const graph = buildValidatedSampleGraph(registry);
      useGraphStore.setState({ graph });

      useGraphStore.getState().createNode("builtin.data.kline");

      const state = useGraphStore.getState();
      expect(state.graph.nodes.length).toBeGreaterThan(graph.nodes.length);
      expect(state.selectedNodeId).toBeTruthy();
      expect(state.compileResult).toBeNull();
    });

    it("ignores invalid module key gracefully", () => {
      const graph = buildValidatedSampleGraph(registry);
      useGraphStore.setState({ graph });
      const count = graph.nodes.length;

      useGraphStore.getState().createNode("nonexistent.module.key");

      expect(useGraphStore.getState().graph.nodes.length).toBe(count);
    });
  });

  describe("setSelectedNode / setSelectedEdge", () => {
    it("setSelectedNode clears edge selection", () => {
      useGraphStore.setState({ selectedEdgeId: "edge_1" });
      useGraphStore.getState().setSelectedNode("node_1");
      const s = useGraphStore.getState();
      expect(s.selectedNodeId).toBe("node_1");
      expect(s.selectedEdgeId).toBeNull();
    });

    it("setSelectedEdge clears node selection", () => {
      useGraphStore.setState({ selectedNodeId: "node_1" });
      useGraphStore.getState().setSelectedEdge("edge_1");
      const s = useGraphStore.getState();
      expect(s.selectedEdgeId).toBe("edge_1");
      expect(s.selectedNodeId).toBeNull();
    });
  });

  describe("updateNodeConfig", () => {
    it("updates node config and clears compileResult", () => {
      const graph = buildValidatedSampleGraph(registry);
      const nodeId = graph.nodes[0]?.id;
      useGraphStore.setState({ graph, compileResult: { old: true } });

      if (nodeId) {
        useGraphStore.getState().setSelectedNode(nodeId);
        useGraphStore.getState().updateNodeConfig({ someParam: 42 });
        expect(useGraphStore.getState().compileResult).toBeNull();
      }
    });
  });
});
