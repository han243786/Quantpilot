import { buildStrategyTemplateGraph } from "../templates/strategyTemplates";
import {
  attachValidationWithRegistry,
  resolveStrategyIrDraft,
  saveGraphToStorage
} from "./graphStoreHelpers";

export function createGraphStoreEditorTemplateActions(set, get) {
  return {
    loadStrategyTemplate(templateId) {
      const registry = get().registry;
      const graph = attachValidationWithRegistry(buildStrategyTemplateGraph(templateId, registry), registry);
      saveGraphToStorage(graph);
      set((state) => ({
        graph,
        selectedNodeId: null,
        selectedEdgeId: null,
        selectedCompileDiagnosticTarget: null,
        compileResult: null,
        formalQuantScriptDraft: null,
        formalQuantScriptOverride: null,
        quantScriptDraft: graph.metadata?.artifacts?.quantscript?.graph_source || "",
        strategyIrDraft: resolveStrategyIrDraft(graph, ""),
        graphVersions: [],
        graphVersionsStatus: "idle",
        graphVersionsMessage: "",
        graphVersionPreview: null,
        graphVersionPreviewStatus: "idle",
        graphVersionPreviewMessage: "",
        graphVersionCompare: null,
        graphVersionCompareStatus: "idle",
        graphVersionCompareMessage: "",
        runtime: {
          ...state.runtime,
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
        }
      }));
      return graph;
    }
  };
}
