import { describe, expect, it } from "vitest";
import { defaultRegistry } from "./graphStoreHelpers";
import { createGraphStoreEditorSelectionActions } from "./graphStoreEditorSelectionActions";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";

function createHarness(overrides = {}) {
  let state = {
    graph: buildValidatedSampleGraph(defaultRegistry),
    selectedNodeId: null,
    selectedEdgeId: null,
    selectedCompileDiagnosticTarget: null,
    strategyIrDraft: "",
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
    actions: createGraphStoreEditorSelectionActions(set, get),
    getState: get
  };
}

describe("graphStore editor selection actions", () => {
  it("selects a node and clears edge and compile diagnostic focus", () => {
    const harness = createHarness({
      selectedEdgeId: "edge_1",
      selectedCompileDiagnosticTarget: { scope: "graph", label: "Graph" }
    });

    harness.actions.setSelectedNode("node_1");

    expect(harness.getState()).toMatchObject({
      selectedNodeId: "node_1",
      selectedEdgeId: null,
      selectedCompileDiagnosticTarget: null
    });
  });

  it("selects an edge and clears node and compile diagnostic focus", () => {
    const harness = createHarness({
      selectedNodeId: "node_1",
      selectedCompileDiagnosticTarget: { scope: "graph", label: "Graph" }
    });

    harness.actions.setSelectedEdge("edge_1");

    expect(harness.getState()).toMatchObject({
      selectedNodeId: null,
      selectedEdgeId: "edge_1",
      selectedCompileDiagnosticTarget: null
    });
  });

  it("focuses node and edge compile diagnostics as editor selections", () => {
    const nodeHarness = createHarness();
    const edgeHarness = createHarness();
    const graph = nodeHarness.getState().graph;

    nodeHarness.actions.focusCompileDiagnostic({ scope: "node", node_id: graph.nodes[0].id });
    edgeHarness.actions.focusCompileDiagnostic({ scope: "edge", edge_id: graph.edges[0].id });

    expect(nodeHarness.getState()).toMatchObject({
      selectedNodeId: graph.nodes[0].id,
      selectedEdgeId: null,
      selectedCompileDiagnosticTarget: null
    });
    expect(edgeHarness.getState()).toMatchObject({
      selectedNodeId: null,
      selectedEdgeId: graph.edges[0].id,
      selectedCompileDiagnosticTarget: null
    });
  });

  it("keeps strategy IR and graph diagnostics in compile diagnostic focus", () => {
    const strategyHarness = createHarness({ selectedNodeId: "node_1", selectedEdgeId: "edge_1" });
    const graphHarness = createHarness({ selectedNodeId: "node_1", selectedEdgeId: "edge_1" });

    strategyHarness.actions.focusCompileDiagnostic({
      scope: "strategy_ir",
      field: "custom_signal.params.custom_expr",
      label: "custom_signal.params.custom_expr"
    });
    graphHarness.actions.focusCompileDiagnostic("graph.missing_field");

    expect(strategyHarness.getState()).toMatchObject({
      selectedNodeId: null,
      selectedEdgeId: null,
      selectedCompileDiagnosticTarget: {
        scope: "strategy_ir",
        field: "custom_signal.params.custom_expr",
        label: "custom_signal.params.custom_expr"
      }
    });
    expect(graphHarness.getState()).toMatchObject({
      selectedNodeId: null,
      selectedEdgeId: null,
      selectedCompileDiagnosticTarget: {
        scope: "graph",
        field: "graph.missing_field",
        label: "graph.missing_field"
      }
    });
  });
});
