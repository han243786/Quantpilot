import { beforeEach, describe, expect, it } from "vitest";
import { defaultRegistry } from "./graphStoreHelpers";
import { createGraphStoreEditorDraftActions } from "./graphStoreEditorDraftActions";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";

function createHarness(overrides = {}) {
  let state = {
    registry: defaultRegistry,
    graph: buildValidatedSampleGraph(defaultRegistry),
    selectedNodeId: "node_1",
    selectedEdgeId: "edge_1",
    selectedCompileDiagnosticTarget: { scope: "graph", label: "Graph" },
    compileResult: { ok: true },
    quantScriptDraft: "",
    formalQuantScriptDraft: "formal draft",
    formalQuantScriptOverride: "override",
    strategyIrDraft: "strategy draft",
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
    actions: createGraphStoreEditorDraftActions(set, get),
    getState: get
  };
}

describe("graphStore editor draft actions", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it("updates and resets graph, formal, and Strategy IR drafts", () => {
    const harness = createHarness();
    const graphSource = harness.getState().graph.metadata.artifacts.quantscript.graph_source;

    harness.actions.updateQuantScriptDraft("graph draft");
    harness.actions.updateFormalQuantScriptDraft("formal source");
    harness.actions.updateStrategyIrDraft("strategy source");

    expect(harness.getState()).toMatchObject({
      quantScriptDraft: "graph draft",
      formalQuantScriptDraft: "formal source",
      strategyIrDraft: "strategy source"
    });

    harness.actions.resetQuantScriptDraft();
    harness.actions.resetFormalQuantScriptDraft();
    harness.actions.resetStrategyIrDraft();

    expect(harness.getState()).toMatchObject({
      quantScriptDraft: graphSource,
      formalQuantScriptDraft: null,
      formalQuantScriptOverride: null,
      selectedCompileDiagnosticTarget: null,
      compileResult: null
    });
    expect(typeof harness.getState().strategyIrDraft).toBe("string");
  });

  it("applies QuantScript source and clears editor focus state", () => {
    const harness = createHarness();
    const source = harness.getState().graph.metadata.artifacts.quantscript.graph_source;

    const graph = harness.actions.applyQuantScriptSource(source);
    const state = harness.getState();

    expect(graph.metadata.artifacts.quantscript.graph_source).toContain("strategy_graph");
    expect(state.graph.nodes).toHaveLength(graph.nodes.length);
    expect(state.selectedNodeId).toBeNull();
    expect(state.selectedEdgeId).toBeNull();
    expect(state.selectedCompileDiagnosticTarget).toBeNull();
    expect(state.compileResult).toBeNull();
    expect(state.quantScriptDraft).toContain("strategy_graph");
    expect(state.runtime).toEqual({ events: ["kept"] });
  });
});
