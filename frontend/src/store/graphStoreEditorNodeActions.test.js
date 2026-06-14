import { beforeEach, describe, expect, it } from "vitest";
import { defaultRegistry } from "./graphStoreHelpers";
import { createGraphStoreEditorNodeActions } from "./graphStoreEditorNodeActions";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";

function createHarness(overrides = {}) {
  let state = {
    registry: defaultRegistry,
    graph: buildValidatedSampleGraph(defaultRegistry),
    selectedNodeId: null,
    selectedEdgeId: "edge_1",
    compileResult: { ok: true },
    quantScriptDraft: "",
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
    actions: createGraphStoreEditorNodeActions(set, get),
    getState: get
  };
}

describe("graphStore editor node actions", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it("creates a node and selects it through the editor action facade", () => {
    const harness = createHarness();
    const previousCount = harness.getState().graph.nodes.length;

    harness.actions.createNode("builtin.data.kline");
    const state = harness.getState();

    expect(state.graph.nodes).toHaveLength(previousCount + 1);
    expect(state.selectedNodeId).toBeTruthy();
    expect(state.selectedEdgeId).toBeNull();
    expect(state.quantScriptDraft).toContain("strategy_graph");
  });

  it("updates node position, config, name, and collapsed state", () => {
    const harness = createHarness();
    const node = harness.getState().graph.nodes[0];
    const nextPosition = { x: 123, y: 456 };
    const initialCollapsed = Boolean(node.ui_state?.collapsed);

    harness.actions.updateNodePosition(node.id, nextPosition, false);
    harness.actions.updateNodeConfig(node.id, "exchange", "okx");
    harness.actions.updateNodeName(node.id, "Renamed Node");
    harness.actions.toggleNodeCollapse(node.id);

    const updatedNode = harness.getState().graph.nodes.find((item) => item.id === node.id);
    expect(updatedNode.position).toEqual(nextPosition);
    expect(updatedNode.config.exchange).toBe("okx");
    expect(updatedNode.name).toBe("Renamed Node");
    expect(Boolean(updatedNode.ui_state.collapsed)).toBe(!initialCollapsed);
    expect(harness.getState().compileResult).toBeNull();
    expect(harness.getState().quantScriptDraft).toContain("strategy_graph");
    expect(harness.getState().graph.metadata.editor.recent_node_ids[0]).toBe(node.id);
  });

  it("updates editor viewport without changing graph nodes", () => {
    const harness = createHarness();
    const nodeCount = harness.getState().graph.nodes.length;
    const viewport = { x: 10, y: 20, zoom: 0.75 };

    harness.actions.updateEditorViewport(viewport, true);

    expect(harness.getState().graph.nodes).toHaveLength(nodeCount);
    expect(harness.getState().graph.metadata.editor.viewport).toEqual(viewport);
  });
});
