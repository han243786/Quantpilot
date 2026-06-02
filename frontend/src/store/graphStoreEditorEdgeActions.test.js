import { beforeEach, describe, expect, it } from "vitest";
import { defaultRegistry } from "./graphStoreHelpers";
import { createGraphStoreEditorEdgeActions } from "./graphStoreEditorEdgeActions";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";

function createHarness(overrides = {}) {
  let state = {
    registry: defaultRegistry,
    graph: buildValidatedSampleGraph(defaultRegistry),
    selectedNodeId: null,
    selectedEdgeId: null,
    compileResult: { ok: true },
    quantScriptDraft: "",
    runtime: { events: ["kept"] },
    ...overrides
  };

  const set = (next) => {
    state = {
      ...state,
      ...(typeof next === "function" ? next(state) : next)
    };
  };
  const get = () => state;

  return {
    actions: createGraphStoreEditorEdgeActions(set, get),
    getState: get
  };
}

describe("graphStore editor edge actions", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it("adds an edge and records touched node ids", () => {
    const harness = createHarness();
    const [sourceNode, targetNode] = harness.getState().graph.nodes;
    const previousEdgeCount = harness.getState().graph.edges.length;

    harness.actions.addEdge({
      source: sourceNode.id,
      sourceHandle: sourceNode.output_ports?.[0]?.key || "out",
      target: targetNode.id,
      targetHandle: targetNode.input_ports?.[0]?.key || "in"
    });

    const state = harness.getState();
    expect(state.graph.edges).toHaveLength(previousEdgeCount + 1);
    expect(state.graph.metadata.editor.recent_node_ids.slice(0, 2)).toEqual([
      sourceNode.id,
      targetNode.id
    ]);
    expect(state.compileResult).toBeNull();
    expect(state.quantScriptDraft).toContain("strategy_graph");
  });

  it("removes selected nodes with incident edges and filters recent node ids", () => {
    const graph = buildValidatedSampleGraph(defaultRegistry);
    const removedNode = graph.nodes[0];
    graph.metadata.editor.recent_node_ids = [removedNode.id, graph.nodes[1].id];
    const harness = createHarness({
      graph,
      selectedNodeId: removedNode.id,
      selectedEdgeId: null
    });

    harness.actions.removeSelected();
    const state = harness.getState();

    expect(state.graph.nodes.some((node) => node.id === removedNode.id)).toBe(false);
    expect(state.graph.edges.some((edge) => edge.source_node_id === removedNode.id || edge.target_node_id === removedNode.id)).toBe(false);
    expect(state.graph.metadata.editor.recent_node_ids.includes(removedNode.id)).toBe(false);
    expect(state.selectedNodeId).toBeNull();
    expect(state.selectedEdgeId).toBeNull();
    expect(state.compileResult).toBeNull();
    expect(state.runtime).toEqual({ events: ["kept"] });
  });

  it("removes a selected edge without deleting nodes", () => {
    const graph = buildValidatedSampleGraph(defaultRegistry);
    const edge = graph.edges[0];
    const harness = createHarness({
      graph,
      selectedNodeId: null,
      selectedEdgeId: edge.id
    });

    harness.actions.removeSelected();

    expect(harness.getState().graph.edges.some((item) => item.id === edge.id)).toBe(false);
    expect(harness.getState().graph.nodes).toHaveLength(graph.nodes.length);
    expect(harness.getState().selectedEdgeId).toBeNull();
  });
});
