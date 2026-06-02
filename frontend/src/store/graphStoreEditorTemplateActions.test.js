import { beforeEach, describe, expect, it } from "vitest";
import { defaultRegistry } from "./graphStoreHelpers";
import { createGraphStoreEditorTemplateActions } from "./graphStoreEditorTemplateActions";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";

function createHarness(overrides = {}) {
  let state = {
    registry: defaultRegistry,
    graph: buildValidatedSampleGraph(defaultRegistry),
    selectedNodeId: "node_1",
    selectedEdgeId: "edge_1",
    selectedCompileDiagnosticTarget: { scope: "graph", label: "Graph" },
    compileResult: { ok: true },
    formalQuantScriptDraft: "formal draft",
    formalQuantScriptOverride: "override",
    quantScriptDraft: "graph draft",
    strategyIrDraft: "strategy draft",
    graphVersions: [{ version_id: "v1" }],
    graphVersionsStatus: "ready",
    graphVersionsMessage: "loaded",
    graphVersionPreview: { version_id: "v1" },
    graphVersionPreviewStatus: "ready",
    graphVersionPreviewMessage: "loaded",
    graphVersionCompare: { left: "v1" },
    graphVersionCompareStatus: "ready",
    graphVersionCompareMessage: "loaded",
    runtime: {
      runId: "run_1",
      runKind: "backtest",
      status: "running",
      connectionState: "connected",
      account: { id: "paper" },
      backtestArtifacts: { summary: true },
      diagnostics: { ok: false },
      governance: { locked: true },
      events: [{ type: "RuntimeStarted" }],
      timeline: [{ type: "started" }],
      retainedKeyEventIndex: 0,
      compactEvidence: { id: "ev_1" },
      backendError: "error",
      backtestCompareSelection: { _global: ["bt_1"] },
      selectedHistoryRunId: "run_1",
      selectedBacktestId: "bt_1",
      selectedExperimentId: "exp_1",
      selectedExperiment: { experiment_id: "exp_1" },
      selectedExperimentStatus: "ready",
      highlightedNodeIds: ["node_1"],
      history: [{ run_id: "run_1" }],
      backtestHistory: [{ backtest_id: "bt_1" }],
      experiments: [{ experiment_id: "exp_1" }]
    },
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
    actions: createGraphStoreEditorTemplateActions(set, get),
    getState: get
  };
}

describe("graphStore editor template actions", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it("loads a template graph and resets editor, version, and active runtime focus state", () => {
    const harness = createHarness();

    const graph = harness.actions.loadStrategyTemplate("multi_symbol_rebalance");
    const state = harness.getState();

    expect(graph.metadata.template_id).toBe("multi_symbol_rebalance");
    expect(graph.validation_state.is_runnable).toBe(true);
    expect(state).toMatchObject({
      graph,
      selectedNodeId: null,
      selectedEdgeId: null,
      selectedCompileDiagnosticTarget: null,
      compileResult: null,
      formalQuantScriptDraft: null,
      formalQuantScriptOverride: null,
      graphVersions: [],
      graphVersionsStatus: "idle",
      graphVersionsMessage: "",
      graphVersionPreview: null,
      graphVersionPreviewStatus: "idle",
      graphVersionPreviewMessage: "",
      graphVersionCompare: null,
      graphVersionCompareStatus: "idle",
      graphVersionCompareMessage: ""
    });
    expect(state.quantScriptDraft).toContain("strategy_graph");
    expect(typeof state.strategyIrDraft).toBe("string");
    expect(state.runtime.history).toHaveLength(1);
    expect(state.runtime.backtestHistory).toHaveLength(1);
    expect(state.runtime.experiments).toHaveLength(1);
    expect(state.runtime).toMatchObject({
      runId: null,
      runKind: null,
      status: "idle",
      connectionState: "disconnected",
      account: null,
      backtestArtifacts: null,
      diagnostics: null,
      governance: null,
      events: [],
      timeline: [],
      retainedKeyEventIndex: null,
      compactEvidence: null,
      backendError: null,
      backtestCompareSelection: {},
      selectedHistoryRunId: null,
      selectedBacktestId: null,
      selectedExperimentId: null,
      selectedExperiment: null,
      selectedExperimentStatus: "idle",
      highlightedNodeIds: []
    });
  });
});
