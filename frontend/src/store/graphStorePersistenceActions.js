import { createEmptyGraph } from "../graph/createGraph";
import { compileGraph } from "../graph/compileGraph";
import {
  attachValidationWithRegistry,
  fetchJson,
  normalizeGraphAuditHistory,
  normalizeGraphVersionCompare,
  normalizeGraphVersions,
  postJson,
  resolveGraphActor,
  resolveLoadedGraphWithRegistry,
  resolveStrategyIrDraft,
  saveGraphToStorage
} from "./graphStoreHelpers";

export function createGraphStorePersistenceActions(set, get) {
  return {
    resetGraph() {
      const registry = get().registry;
      const history = get().runtime.history;
      const historyStatus = get().runtime.historyStatus;
      const backtestHistory = get().runtime.backtestHistory;
      const backtestHistoryStatus = get().runtime.backtestHistoryStatus;
      const experiments = get().runtime.experiments;
      const experimentsStatus = get().runtime.experimentsStatus;
      const graph = attachValidationWithRegistry(createEmptyGraph(registry), registry);
      saveGraphToStorage(graph);
      set({
        graph,
        selectedNodeId: null,
        selectedEdgeId: null,
        selectedCompileDiagnosticTarget: null,
        compileResult: null,
        runtime: {
          runId: null,
          runKind: null,
          status: "idle",
          connectionState: "disconnected",
          account: null,
          backtestArtifacts: null,
          diagnostics: null,
          events: [],
          backendError: null,
          history,
          historyStatus,
          backtestHistory,
          backtestHistoryStatus,
          experiments,
          experimentsStatus,
          backtestCompareSelection: [],
          selectedHistoryRunId: null,
          selectedBacktestId: null,
          selectedExperimentId: null,
          selectedExperiment: null,
          selectedExperimentStatus: "idle",
          highlightedNodeIds: []
        },
        graphVersions: [],
        graphVersionsStatus: "idle",
        graphVersionsMessage: "",
        graphVersionPreview: null,
        graphVersionPreviewStatus: "idle",
        graphVersionPreviewMessage: "",
        graphVersionCompare: null,
        graphVersionCompareStatus: "idle",
        graphVersionCompareMessage: "",
        graphAuditHistory: [],
        graphAuditHistoryStatus: "idle",
        graphAuditHistoryMessage: "",
        quantScriptDraft: graph.metadata?.artifacts?.quantscript?.graph_source || "",
        strategyIrDraft: resolveStrategyIrDraft(graph, "")
      });
    },

    async saveGraph(options = {}) {
      const registry = get().registry;
      const currentGraph = get().graph;
      const versionLabel =
        Object.prototype.hasOwnProperty.call(options, "versionLabel") ? options.versionLabel : undefined;
      const saveNote =
        Object.prototype.hasOwnProperty.call(options, "saveNote") ? options.saveNote : undefined;
      const actor = resolveGraphActor(currentGraph);
      const graphForSave = {
        ...currentGraph,
        metadata: {
          ...currentGraph.metadata,
          collaboration: {
            ...(currentGraph.metadata?.collaboration || {}),
            ...(currentGraph.metadata?.collaboration?.owner ? {} : { owner: actor }),
            last_saved_by: actor
          },
          ...(versionLabel !== undefined
            ? {
                version_label: versionLabel?.trim?.() ? versionLabel.trim() : undefined
              }
            : {}),
          ...(saveNote !== undefined
            ? {
                save_note: saveNote?.trim?.() ? saveNote.trim() : undefined
              }
            : {})
        }
      };
      if (versionLabel !== undefined && !versionLabel?.trim?.()) {
        delete graphForSave.metadata.version_label;
      }
      if (saveNote !== undefined && !saveNote?.trim?.()) {
        delete graphForSave.metadata.save_note;
      }
      const compiled = compileGraph(graphForSave, registry);
      if (!compiled.compile_summary.compilable) {
        throw new Error(
          compiled.compile_summary.errors[0] || "Strategy graph failed compile validation."
        );
      }
      const graph = attachValidationWithRegistry(
        {
          ...compiled.graph,
          metadata: {
            ...compiled.graph.metadata,
            ...(graphForSave.metadata?.version_label
              ? { version_label: graphForSave.metadata.version_label }
              : {}),
            ...(graphForSave.metadata?.save_note
              ? { save_note: graphForSave.metadata.save_note }
              : {})
          },
          compile_summary: compiled.compile_summary
        },
        registry
      );
      saveGraphToStorage(graph);
      set({
        graph,
        compileResult: compiled,
        graphVersionPreview: null,
        graphVersionPreviewStatus: "idle",
        graphVersionPreviewMessage: "",
        graphVersionCompare: null,
        graphVersionCompareStatus: "idle",
        graphVersionCompareMessage: "",
        quantScriptDraft: graph.metadata?.artifacts?.quantscript?.graph_source || compiled.quantscript || "",
        strategyIrDraft: resolveStrategyIrDraft(graph, get().strategyIrDraft)
      });
      const request = { graph, actor };
      if (versionLabel !== undefined) {
        request.version_label = versionLabel;
      }
      if (saveNote !== undefined) {
        request.save_note = saveNote;
      }
      await postJson("/graphs/save", request);
      await get().refreshGraphIndex();
      await get().refreshGraphVersions(graph.metadata?.graph_id || "");
      await get().refreshGraphAuditHistory(graph.metadata?.graph_id || "");
    },

    async loadLatestGraph() {
      const finalGraph = resolveLoadedGraphWithRegistry(await fetchJson("/graphs/latest"), get().registry);
      if (!finalGraph) {
        throw new Error("The latest saved strategy graph is unavailable.");
      }
      saveGraphToStorage(finalGraph);
      set((state) => ({
        graph: finalGraph,
        selectedNodeId: null,
        selectedEdgeId: null,
        selectedCompileDiagnosticTarget: null,
        graphVersionPreview: null,
        graphVersionPreviewStatus: "idle",
        graphVersionPreviewMessage: "",
        graphVersionCompare: null,
        graphVersionCompareStatus: "idle",
        graphVersionCompareMessage: "",
        graphAuditHistory: [],
        graphAuditHistoryStatus: "idle",
        graphAuditHistoryMessage: "",
        quantScriptDraft: finalGraph.metadata?.artifacts?.quantscript?.graph_source || "",
        strategyIrDraft: resolveStrategyIrDraft(finalGraph, ""),
        runtime: {
          ...state.runtime
        }
      }));
      await get().refreshGraphVersions(finalGraph.metadata?.graph_id || "");
      await get().refreshGraphAuditHistory(finalGraph.metadata?.graph_id || "");
    },

    async loadGraphById(graphId, options = {}) {
      if (!graphId) {
        throw new Error("A strategy graph ID is required.");
      }

      const { force = false } = options;
      const currentGraph = get().graph;
      if (!force && currentGraph?.metadata?.graph_id === graphId) {
        return currentGraph;
      }

      const finalGraph = resolveLoadedGraphWithRegistry(
        await fetchJson(`/graphs/${encodeURIComponent(graphId)}`),
        get().registry
      );
      if (!finalGraph) {
        throw new Error("The requested strategy graph is unavailable.");
      }

      saveGraphToStorage(finalGraph);
      set((state) => ({
        graph: finalGraph,
        selectedNodeId: null,
        selectedEdgeId: null,
        selectedCompileDiagnosticTarget: null,
        graphVersionPreview: null,
        graphVersionPreviewStatus: "idle",
        graphVersionPreviewMessage: "",
        graphVersionCompare: null,
        graphVersionCompareStatus: "idle",
        graphVersionCompareMessage: "",
        graphAuditHistory: [],
        graphAuditHistoryStatus: "idle",
        graphAuditHistoryMessage: "",
        quantScriptDraft: finalGraph.metadata?.artifacts?.quantscript?.graph_source || "",
        strategyIrDraft: resolveStrategyIrDraft(finalGraph, ""),
        runtime: {
          ...state.runtime
        }
      }));
      await get().refreshGraphVersions(finalGraph.metadata?.graph_id || graphId);
      await get().refreshGraphAuditHistory(finalGraph.metadata?.graph_id || graphId);
      return finalGraph;
    },

    async refreshGraphVersions(graphId = get().graph?.metadata?.graph_id || "") {
      if (!graphId || graphId === "draft_graph") {
        set({
          graphVersions: [],
          graphVersionsStatus: "idle",
          graphVersionsMessage: "",
          graphVersionPreview: null,
          graphVersionPreviewStatus: "idle",
          graphVersionPreviewMessage: "",
          graphVersionCompare: null,
          graphVersionCompareStatus: "idle",
          graphVersionCompareMessage: "",
          graphAuditHistory: [],
          graphAuditHistoryStatus: "idle",
          graphAuditHistoryMessage: ""
        });
        return [];
      }

      set({ graphVersionsStatus: "loading", graphVersionsMessage: "" });
      try {
        const graphVersions = normalizeGraphVersions(
          await fetchJson(`/graphs/${encodeURIComponent(graphId)}/versions`)
        );
        set({
          graphVersions,
          graphVersionsStatus: "ready",
          graphVersionsMessage: ""
        });
        return graphVersions;
      } catch (error) {
        set({
          graphVersions: [],
          graphVersionsStatus: "error",
          graphVersionsMessage: error.message || "Failed to load persisted graph versions."
        });
        return [];
      }
    },

    async refreshGraphAuditHistory(graphId = get().graph?.metadata?.graph_id || "") {
      if (!graphId || graphId === "draft_graph") {
        set({
          graphAuditHistory: [],
          graphAuditHistoryStatus: "idle",
          graphAuditHistoryMessage: ""
        });
        return [];
      }

      set({ graphAuditHistoryStatus: "loading", graphAuditHistoryMessage: "" });
      try {
        const graphAuditHistory = normalizeGraphAuditHistory(
          await fetchJson(`/graphs/${encodeURIComponent(graphId)}/audit`)
        );
        set({
          graphAuditHistory,
          graphAuditHistoryStatus: "ready",
          graphAuditHistoryMessage: ""
        });
        return graphAuditHistory;
      } catch (error) {
        set({
          graphAuditHistory: [],
          graphAuditHistoryStatus: "error",
          graphAuditHistoryMessage: error.message || "Failed to load graph audit history."
        });
        return [];
      }
    },

    async loadGraphVersionPreview(graphId, versionId) {
      if (!graphId || !versionId) {
        throw new Error("Both a graph ID and version ID are required.");
      }

      set({ graphVersionPreviewStatus: "loading", graphVersionPreviewMessage: "" });
      try {
        const preview = resolveLoadedGraphWithRegistry(
          await fetchJson(
            `/graphs/${encodeURIComponent(graphId)}/versions/${encodeURIComponent(versionId)}`
          ),
          get().registry
        );
        if (!preview) {
          throw new Error("The requested graph version is unavailable.");
        }

        set({
          graphVersionPreview: {
            versionId,
            graph: preview
          },
          graphVersionPreviewStatus: "ready",
          graphVersionPreviewMessage: ""
        });
        return preview;
      } catch (error) {
        set({
          graphVersionPreview: null,
          graphVersionPreviewStatus: "error",
          graphVersionPreviewMessage: error.message || "Failed to load the selected graph version."
        });
        throw error;
      }
    },

    clearGraphVersionPreview() {
      set({
        graphVersionPreview: null,
        graphVersionPreviewStatus: "idle",
        graphVersionPreviewMessage: ""
      });
    },

    async compareGraphVersions(graphId, leftVersionId, rightVersionId) {
      if (!graphId || !leftVersionId || !rightVersionId) {
        throw new Error("Graph ID and two version IDs are required.");
      }

      set({
        graphVersionCompareStatus: "loading",
        graphVersionCompareMessage: ""
      });
      try {
        const compare = normalizeGraphVersionCompare(
          await fetchJson(
            `/graphs/${encodeURIComponent(graphId)}/versions/compare/${encodeURIComponent(leftVersionId)}/${encodeURIComponent(rightVersionId)}`
          )
        );
        set({
          graphVersionCompare: compare,
          graphVersionCompareStatus: "ready",
          graphVersionCompareMessage: ""
        });
        return compare;
      } catch (error) {
        set({
          graphVersionCompare: null,
          graphVersionCompareStatus: "error",
          graphVersionCompareMessage: error.message || "Failed to compare persisted graph versions."
        });
        throw error;
      }
    },

    clearGraphVersionCompare() {
      set({
        graphVersionCompare: null,
        graphVersionCompareStatus: "idle",
        graphVersionCompareMessage: ""
      });
    },

    async restoreGraphVersion(graphId, versionId) {
      if (!graphId || !versionId) {
        throw new Error("Both a graph ID and version ID are required.");
      }

      await postJson(
        `/graphs/${encodeURIComponent(graphId)}/versions/${encodeURIComponent(versionId)}/restore`,
        { actor: resolveGraphActor(get().graph) }
      );
      await get().refreshGraphIndex();
      await get().loadGraphById(graphId, { force: true });
      await get().refreshGraphVersions(graphId);
      await get().refreshGraphAuditHistory(graphId);
      get().clearGraphVersionPreview();
      get().clearGraphVersionCompare();
    },

    async revealGraphFile(graphId) {
      if (!graphId) {
        throw new Error("A strategy graph ID is required.");
      }

      return postJson(`/graphs/${encodeURIComponent(graphId)}/reveal`, {});
    },

    async openGraphFolder(graphId) {
      if (!graphId) {
        throw new Error("A strategy graph ID is required.");
      }

      return postJson(`/graphs/${encodeURIComponent(graphId)}/reveal-folder`, {});
    }
  };
}
