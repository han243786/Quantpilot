import { create } from "zustand";
import { buildActionFailureMessage } from "../utils/actionFailure";
import { humanizeErrorText } from "../utils/errorText";
import {
  attachValidationWithRegistry,
  buildRegistryFromCapabilities,
  createSafeFallbackCapabilities,
  defaultCapabilities as DEFAULT_CAPABILITIES,
  defaultRegistry,
  fallbackRunnableGraph,
  fetchJson,
  graphExistsInIndex,
  isDeprecatedBuiltinSampleGraph,
  loadCapabilitiesFromCache,
  loadGraphFromStorage,
  normalizeGraphIndex,
  resolveLoadedGraphWithRegistry,
  resolveStrategyIrDraft,
  saveCapabilitiesToCache,
  saveGraphToStorage,
  scheduleBackgroundTask
} from "./graphStoreHelpers";
import { createGraphStoreEditorActions } from "./graphStoreEditorActions";
import { createGraphStoreRuntimeActions } from "./graphStoreRuntimeActions";

export {
  fetchJson,
  parseQuantScriptDiagnosticsFromMessage,
  resolveCompileDiagnosticTargetFromGraphArtifacts
} from "./graphStoreHelpers";

export const useGraphStore = create((set, get) => ({
  registry: defaultRegistry,
  capabilities: DEFAULT_CAPABILITIES,
  capabilityStatus: "ready",
  capabilitySource: "remote",
  capabilityMessage: "",
  graphIndex: [],
  graphIndexStatus: "idle",
  graphIndexMessage: "",
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
  graph: fallbackRunnableGraph(defaultRegistry),
  selectedNodeId: null,
  selectedEdgeId: null,
  selectedCompileDiagnosticTarget: null,
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
    history: [],
    historyStatus: "idle",
    backtestHistory: [],
    backtestHistoryStatus: "idle",
    experiments: [],
    experimentsStatus: "idle",
    backtestCompareSelection: [],
    selectedHistoryRunId: null,
    selectedBacktestId: null,
    selectedExperimentId: null,
    selectedExperiment: null,
    selectedExperimentStatus: "idle",
    highlightedNodeIds: []
  },
  compileResult: null,
  runtimeController: null,
  quantScriptDraft: "",
  formalQuantScriptDraft: null,
  formalQuantScriptOverride: null,
  strategyIrDraft: resolveStrategyIrDraft(fallbackRunnableGraph(defaultRegistry), ""),

  async refreshCapabilities() {
    set({ capabilityStatus: "loading", capabilityMessage: "" });

    try {
      const capabilities = await fetchJson("/capabilities");
      const nextRegistry = buildRegistryFromCapabilities(capabilities);
      const nextGraph = attachValidationWithRegistry(get().graph, nextRegistry);
      saveCapabilitiesToCache(capabilities);
      saveGraphToStorage(nextGraph);
      set({
        registry: nextRegistry,
        capabilities,
        capabilityStatus: "ready",
        capabilitySource: "remote",
        capabilityMessage: "",
        graph: nextGraph,
        quantScriptDraft:
          nextGraph.metadata?.artifacts?.quantscript?.graph_source || get().quantScriptDraft,
        strategyIrDraft: resolveStrategyIrDraft(nextGraph, get().strategyIrDraft)
      });
      return capabilities;
    } catch (error) {
      const message = humanizeErrorText(error, "能力加载失败。");
      const cachedCapabilities = loadCapabilitiesFromCache();

      if (cachedCapabilities) {
        const cachedRegistry = buildRegistryFromCapabilities(cachedCapabilities);
        const nextGraph = attachValidationWithRegistry(get().graph, cachedRegistry);
        saveGraphToStorage(nextGraph);
        set({
          registry: cachedRegistry,
          capabilities: cachedCapabilities,
          capabilityStatus: "degraded",
          capabilitySource: "cache",
          capabilityMessage:
            "Capability fetch failed. Using the latest cached capability snapshot. Final availability still depends on live backend validation.",
          graph: nextGraph,
          quantScriptDraft:
            nextGraph.metadata?.artifacts?.quantscript?.graph_source || get().quantScriptDraft,
          strategyIrDraft: resolveStrategyIrDraft(nextGraph, get().strategyIrDraft)
        });
        return cachedCapabilities;
      }

      const safeFallbackCapabilities = createSafeFallbackCapabilities(message);
      const fallbackRegistry = buildRegistryFromCapabilities(safeFallbackCapabilities);
      const nextGraph = attachValidationWithRegistry(get().graph, fallbackRegistry);
      saveGraphToStorage(nextGraph);
      set({
        registry: fallbackRegistry,
        capabilities: safeFallbackCapabilities,
        capabilityStatus: "error",
        capabilitySource: "safe_fallback",
          capabilityMessage:
            "Capability fetch failed. Entering safe fallback mode. To avoid exposing fake capabilities, module visibility and compile/run actions were tightened to the safest profile.",
          graph: nextGraph,
          quantScriptDraft:
          nextGraph.metadata?.artifacts?.quantscript?.graph_source || get().quantScriptDraft,
          strategyIrDraft: resolveStrategyIrDraft(nextGraph, get().strategyIrDraft)
        });
      return safeFallbackCapabilities;
    }
  },

  async initialize() {
    await get().refreshCapabilities();
    const graphIndex = await get().refreshGraphIndex();

    const registry = get().registry;
    let resolvedGraph = null;
    let latestGraph = null;
    let startupRecoveryError = null;

    try {
      latestGraph = resolveLoadedGraphWithRegistry(await fetchJson("/graphs/latest"), registry);
    } catch (error) {
      startupRecoveryError = buildActionFailureMessage(
        "startup_recovery",
        error,
        "Startup graph recovery failed."
      );
    }

    if (latestGraph?.validation_state?.is_runnable && graphExistsInIndex(latestGraph, graphIndex)) {
      resolvedGraph = latestGraph;
    }

    if (!resolvedGraph) {
      const loaded = resolveLoadedGraphWithRegistry(loadGraphFromStorage(), registry);
      if (loaded?.validation_state?.is_runnable && graphExistsInIndex(loaded, graphIndex)) {
        resolvedGraph = loaded;
      } else if (latestGraph && graphExistsInIndex(latestGraph, graphIndex)) {
        resolvedGraph = latestGraph;
      }
    }

    if (!resolvedGraph) {
      resolvedGraph = fallbackRunnableGraph(registry);
    }

    saveGraphToStorage(resolvedGraph);
    set((state) => ({
      graph: resolvedGraph,
      quantScriptDraft: resolvedGraph.metadata?.artifacts?.quantscript?.graph_source || "",
      strategyIrDraft: resolveStrategyIrDraft(resolvedGraph, ""),
      runtime: {
        ...state.runtime,
        backendError: startupRecoveryError,
        backtestCompareSelection: []
      }
    }));

    void get().refreshGraphVersions(resolvedGraph.metadata?.graph_id || "");
    void get().refreshGraphAuditHistory(resolvedGraph.metadata?.graph_id || "");

    if (!latestGraph?.validation_state?.is_runnable) {
      void get().recoverLatestRunnableGraph();
    }

    scheduleBackgroundTask(() => get().warmRuntimeSidebarData());
  },

  async refreshGraphIndex() {
    set({ graphIndexStatus: "loading", graphIndexMessage: "" });

    try {
      const graphIndex = normalizeGraphIndex(await fetchJson("/graphs"));
      set({
        graphIndex,
        graphIndexStatus: "ready",
        graphIndexMessage: ""
      });
      return graphIndex;
    } catch (error) {
      set({
        graphIndexStatus: "error",
        graphIndexMessage: humanizeErrorText(error, "加载策略列表失败。")
      });
      return [];
    }
  },
  async recoverLatestRunnableGraph() {
    let lastError = null;
    for (let attempt = 0; attempt < 10; attempt += 1) {
      try {
        const latestGraph = resolveLoadedGraphWithRegistry(
          await fetchJson("/graphs/latest"),
          get().registry
        );
        if (!latestGraph?.validation_state?.is_runnable) {
          const message = buildActionFailureMessage(
            "startup_recovery",
            "Latest saved graph is not runnable yet.",
            "Startup graph recovery failed."
          );
          set((state) => ({
            runtime: {
              ...state.runtime,
              backendError: message
            }
          }));
          return null;
        }

        const currentGraph = get().graph;
        if (
          !currentGraph?.validation_state?.is_runnable ||
          isDeprecatedBuiltinSampleGraph(currentGraph)
        ) {
          saveGraphToStorage(latestGraph);
          set((state) => ({
            graph: latestGraph,
            quantScriptDraft: latestGraph.metadata?.artifacts?.quantscript?.graph_source || "",
            strategyIrDraft: resolveStrategyIrDraft(latestGraph, ""),
            runtime: {
              ...state.runtime,
              backendError: null,
              backtestCompareSelection: []
            }
          }));
        } else {
          set((state) => ({
            runtime: {
              ...state.runtime,
              backendError: null
            }
          }));
        }

        return latestGraph;
      } catch (error) {
        lastError = error;
      }

      await new Promise((resolve) => setTimeout(resolve, 1500));
    }

    const message = buildActionFailureMessage(
      "startup_recovery",
      lastError,
      "Startup graph recovery failed."
    );
    set((state) => ({
      runtime: {
        ...state.runtime,
        backendError: message
      }
    }));
    return null;
  },
  ...createGraphStoreEditorActions(set, get),
  ...createGraphStoreRuntimeActions(set, get)
}));
