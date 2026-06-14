import { compileGraph } from "../graph/compileGraph";
import { createEmptyGraph } from "../graph/createGraph";
import {
  attachValidationWithRegistry,
  deleteJson,
  fetchJson,
  postJson,
  resolveGraphActor,
  resolveLoadedGraphWithRegistry,
  resolveStrategyIrDraft,
  saveGraphToStorage
} from "./graphStoreHelpers";
import { closeController } from "./graphStoreRuntimeHelpers";

export function createGraphStoreGraphLifecycleActions(set, get) {
  return {
    resetGraph() {
      closeController(get().runtimeController);
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
        formalQuantScriptOverride: null,
        formalQuantScriptDraft: "",
        compileResultNotice: null,
        actionLock: null,
        actionLock: null,
        runtime: {
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
          history,
          historyStatus,
          backtestHistory,
          backtestHistoryStatus,
          experiments,
          experimentsStatus,
          backtestCompareSelection: {},
          selectedHistoryRunId: null,
          selectedBacktestId: null,
          selectedExperimentId: null,
          selectedExperiment: null,
          selectedExperimentStatus: "idle",
          highlightedNodeIds: [],
          parameterMutations: []
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
      return graph;
    },

    async saveGraph(options = {}) {
      if (get().actionLock) return;
      set({ actionLock: "saving" });
      try {
        const registry = get().registry;
        const currentGraph = get().graph;
        const prevGraph = currentGraph;
        const prevCompileResult = get().compileResult;
        const prevQuantScriptDraft = get().quantScriptDraft;
        const prevStrategyIrDraft = get().strategyIrDraft;
        const prevGraphVersionPreview = get().graphVersionPreview;
        const prevGraphVersionPreviewStatus = get().graphVersionPreviewStatus;
        const prevGraphVersionCompare = get().graphVersionCompare;
        const prevGraphVersionCompareStatus = get().graphVersionCompareStatus;
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
          throw new Error(compiled.compile_summary.errors[0] || "策略图编译校验失败。");
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
        try {
          await postJson("/graphs/save", request);
        } catch (e) {
          saveGraphToStorage(prevGraph);
          set({
            graph: prevGraph,
            compileResult: prevCompileResult,
            graphVersionPreview: prevGraphVersionPreview,
            graphVersionPreviewStatus: prevGraphVersionPreviewStatus,
            graphVersionPreviewMessage: "",
            graphVersionCompare: prevGraphVersionCompare,
            graphVersionCompareStatus: prevGraphVersionCompareStatus,
            graphVersionCompareMessage: "",
            quantScriptDraft: prevQuantScriptDraft,
            strategyIrDraft: prevStrategyIrDraft
          });
          throw e;
        }
        await get().refreshGraphIndex();
        await get().refreshGraphVersions(graph.metadata?.graph_id || "");
        await get().refreshGraphAuditHistory(graph.metadata?.graph_id || "");
      } finally {
        set({ actionLock: null });
      }
    },

    async loadLatestGraph() {
      if (get().runtime?.status === "running") get().stopRuntime();
      const finalGraph = resolveLoadedGraphWithRegistry(await fetchJson("/graphs/latest"), get().registry);
      if (!finalGraph) {
        throw new Error("最新保存的策略图不可用。");
      }
      saveGraphToStorage(finalGraph);
      set((state) => ({
        graph: finalGraph,
        compileResult: null,
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

    async deleteGraph(graphId) {
      if (!graphId || graphId === "draft_graph") {
        throw new Error("需要提供已保存的策略图 ID。");
      }

      const response = await deleteJson(`/graphs/${encodeURIComponent(graphId)}`);
      const currentGraphId = get().graph?.metadata?.graph_id || "";
      await get().refreshGraphIndex();

      if (currentGraphId === graphId) {
        get().resetGraph();
      }

      return response;
    },

    async loadGraphById(graphId, options = {}) {
      if (!graphId) {
        throw new Error("需要提供策略图 ID。");
      }
      if (get().runtime?.status === "running") get().stopRuntime();

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
        throw new Error("请求的策略图不可用。");
      }

      saveGraphToStorage(finalGraph);
      set((state) => ({
        graph: finalGraph,
        compileResult: null,
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

    importStrategyPackage(packageData) {
      const rawGraph = packageData?.graph || packageData?.strategy_graph || packageData;
      const loadedGraph = resolveLoadedGraphWithRegistry(rawGraph, get().registry);
      if (!loadedGraph) {
        throw new Error("策略包中没有可用的策略图。");
      }

      const sourceGraphId = loadedGraph.metadata?.graph_id || "";
      const importedGraphId = `imported_${Date.now()}`;
      const finalGraph = attachValidationWithRegistry(
        {
          ...loadedGraph,
          metadata: {
            ...(loadedGraph.metadata || {}),
            graph_id: importedGraphId,
            name: loadedGraph.metadata?.name
              ? `${loadedGraph.metadata.name} (imported)`
              : "Imported Strategy",
            imported_from_graph_id: sourceGraphId,
            imported_at: new Date().toISOString()
          }
        },
        get().registry
      );

      saveGraphToStorage(finalGraph);
      set((state) => ({
        graph: finalGraph,
        compileResult: null,
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
          ...state.runtime,
          backtestCompareSelection: {}
        }
      }));
      return finalGraph;
    },

    async revealGraphFile(graphId) {
      if (!graphId) {
        throw new Error("需要提供策略图 ID。");
      }

      return postJson(`/graphs/${encodeURIComponent(graphId)}/reveal`, {});
    }
  };
}
