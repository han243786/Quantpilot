import { humanizeErrorText } from "../utils/errorText";
import { buildCapabilityContext } from "../capabilities/supportMatrix";
import {
  closeController,
  postJson,
  resolveGraphActor,
  saveGraphToStorage,
  updateRuntimeNode
} from "./graphStoreHelpers";
import { createGraphStoreRuntimeSimulationActions } from "./graphStoreRuntimeSimulationActions";
import {
  buildBacktestCompletionState,
  buildRuntimeConnectingState,
  buildRuntimeResetState,
  buildRuntimeStoppedState,
  resolveRuntimeTargets
} from "./graphStoreRuntimeSessionState";
import {
  getRuntimeCapabilityBlockReason,
  setRuntimeCapabilityBlocked
} from "./graphStoreRuntimeSessionShared";

function resolveV4QuantScriptSource(state) {
  const override = state.formalQuantScriptOverride;
  if (typeof override === "string" && override.trim()) return override.trim();
  const formalSource = state.graph?.metadata?.artifacts?.quantscript?.formal_source;
  if (typeof formalSource === "string" && formalSource.trim()) return formalSource.trim();
  const draft = state.formalQuantScriptDraft;
  if (typeof draft === "string" && draft.trim()) return draft.trim();
  return "";
}

function isV4QuantScriptSource(source) {
  return /^\s*v4_strategy\s+\S+\s*\{/m.test(source || "");
}

function mapV4RuntimeEvents(output = {}) {
  return (output.events || []).map((event) => ({
    event_id: `v4-${event.sequence}`,
    event_time_ms: event.ts_ms,
    event_type: event.event_type,
    node_id: event.source,
    source: "runtime_v4",
    summary: `${event.event_type} <- ${event.source}`,
    input_snapshot: event.payload || {},
    output_snapshot: event.payload || {},
    raw: event
  })).reverse();
}

export function createGraphStoreRuntimeSessionActions(set, get) {
  return {
    ...createGraphStoreRuntimeSimulationActions(set, get),

    async startBacktest() {
      if (get().actionLock) return;
      const graph = get().graph;
      if (!graph.validation_state.is_runnable) return;
      const capabilityBlockReason = getRuntimeCapabilityBlockReason(get, "run_backtest");
      if (capabilityBlockReason) {
        setRuntimeCapabilityBlocked(set, "backtest", capabilityBlockReason);
        return;
      }

      // 若 graph 自上次编译后未变, 复用缓存跳过冗余编译
      const lastCompileGraphId = get().compileResult?.backend_compile?.compile_id;
      const currentGraphId = graph.metadata?.compile_summary?.compile_id;
      let result;
      if (lastCompileGraphId && lastCompileGraphId === currentGraphId) {
        result = get().compileResult;
      } else {
        result = await get().compileCurrentGraph();
      }
      if (!result) return;
      const capabilityContext = buildCapabilityContext(get().capabilities);

      get().stopRuntime();
      set({ actionLock: "runtime" });

      set((state) =>
        buildRuntimeConnectingState(
          state,
          "backtest",
          "Backtest started. Waiting for results."
        )
      );

      try {
        const response = await postJson("/runtime/backtest", {
          actor: resolveGraphActor(graph),
          capability_context: capabilityContext,
          runtime_config: result.runtime_config,
          runtime_targets: resolveRuntimeTargets(result),
          graph_json: graph,
          backtest_options: {
            replay_source: "deterministic_mock",
            replay_mode: graph.metadata?.backtest_replay_mode || graph.metadata?.replay_mode,
            volatility: 0.5,
            runtime_kind: graph.metadata?.runtime_kind || graph.metadata?.template_runtime_version,
            symbols:
              graph.metadata?.artifacts?.v4_symbols ||
              graph.metadata?.artifacts?.v4_machine_graph?.metadata?.symbols ||
              []
          }
        });
        const nextState = buildBacktestCompletionState(
          get(),
          graph,
          response,
          result.compile_id
        );
        saveGraphToStorage(nextState.nextGraph);

        set(() => ({
          graph: nextState.nextGraph,
          selectedNodeId: nextState.selectedNodeId,
          runtime: nextState.runtime
        }));
      } catch (error) {
        const message = humanizeErrorText(error, "Backtest request failed.");
        set((state) => ({
          runtime: {
            ...state.runtime,
            runKind: "backtest",
            status: "error",
            backendError: message,
            artifactPersistenceStatus: "idle",
            backtestArtifacts: null,
            diagnostics: null,
            governance: null,
            selectedBacktestId: null
          },
          graph: {
            ...state.graph,
            nodes: updateRuntimeNode(state.graph.nodes, "error", message)
          }
        }));
      } finally {
        set({ actionLock: null });
      }
    },

    async startV4Simulation() {
      if (get().actionLock) return;
      const capabilityBlockReason = getRuntimeCapabilityBlockReason(get, "start_v4_simulation");
      if (capabilityBlockReason) {
        setRuntimeCapabilityBlocked(set, "v4_simulation", capabilityBlockReason);
        return;
      }

      const source = resolveV4QuantScriptSource(get());
      if (!isV4QuantScriptSource(source)) {
        const message = "当前没有可运行的 v4 QuantScript 源码。";
        setRuntimeCapabilityBlocked(set, "v4_simulation", message);
        return;
      }

      get().stopRuntime();
      set({ actionLock: "runtime" });
      set((state) =>
        buildRuntimeConnectingState(
          state,
          "v4_simulation",
          "v4 PaperSimulated runtime started."
        )
      );

      try {
        const response = await postJson("/runtime/v4/run", { source });
        const events = mapV4RuntimeEvents(response.output);
        set((state) => ({
          runtimeController: null,
          runtime: {
            ...state.runtime,
            runId: response.run_id,
            runKind: "v4_simulation",
            status: "completed",
            connectionState: "connected",
            artifactPersistenceStatus: "transient",
            backendError: null,
            account: null,
            diagnostics: {
              source: "v4_runtime",
              diagnostics: response.diagnostics || []
            },
            governance: null,
            events,
            timeline: [],
            retainedKeyEventIndex: null,
            compactEvidence: null,
            selectedHistoryRunId: response.run_id,
            selectedBacktestId: null,
            highlightedNodeIds: [...new Set(events.map((event) => event.node_id).filter(Boolean))].slice(0, 50),
            v4_memory_snapshot: response.output?.memory_snapshot || null,
            output: response.output || null,
            v4_runtime_handoff: response.handoff || null
          }
        }));
      } catch (error) {
        const message = humanizeErrorText(error, "v4 runtime run failed.");
        set((state) => ({
          runtimeController: null,
          runtime: {
            ...state.runtime,
            runKind: "v4_simulation",
            status: "error",
            backendError: message
          },
          graph: {
            ...state.graph,
            nodes: updateRuntimeNode(state.graph.nodes, "error", message)
          }
        }));
      } finally {
        set({ actionLock: null });
      }
    },

    async startBacktestExperiment(options = {}) {
      const graph = get().graph;
      if (!graph.validation_state.is_runnable) return null;
      const capabilityBlockReason = getRuntimeCapabilityBlockReason(get, "run_parameter_sweep");
      if (capabilityBlockReason) {
        set((state) => ({
          runtime: {
            ...state.runtime,
            selectedExperimentStatus: "error",
            backendError: capabilityBlockReason
          }
        }));
        return null;
      }

      const result = await get().compileCurrentGraph();
      if (!result) return null;
      const capabilityContext = buildCapabilityContext(get().capabilities);

      const parameterGrid = {
        fee_bps: Array.isArray(options.feeBps) ? options.feeBps : [],
        slippage_bps: Array.isArray(options.slippageBps) ? options.slippageBps : [],
        latency_ms: Array.isArray(options.latencyMs) ? options.latencyMs : []
      };

      set((state) => ({
        runtime: {
          ...state.runtime,
          selectedExperimentStatus: "loading",
          backendError: null
        }
      }));

      try {
        const response = await postJson("/runtime/experiments/backtest-sweep", {
          experiment_name: options.experimentName || "",
          actor: resolveGraphActor(graph),
          capability_context: capabilityContext,
          runtime_config: result.runtime_config,
          runtime_targets: resolveRuntimeTargets(result),
          graph_json: graph,
          backtest_options: {
            replay_source: "deterministic_mock",
            replay_mode: graph.metadata?.backtest_replay_mode || graph.metadata?.replay_mode,
            volatility: 0.5,
            runtime_kind: graph.metadata?.runtime_kind || graph.metadata?.template_runtime_version,
            symbols:
              graph.metadata?.artifacts?.v4_symbols ||
              graph.metadata?.artifacts?.v4_machine_graph?.metadata?.symbols ||
              []
          },
          parameter_grid: parameterGrid
        });
        await get().loadExperimentDetail(response.experiment_id);
        return response;
      } catch (error) {
        const message = humanizeErrorText(error, "Experiment request failed.");
        set((state) => ({
          runtime: {
            ...state.runtime,
            selectedExperimentStatus: "error",
            backendError: message
          }
        }));
        return null;
      }
    },

    stopRuntime() {
      const controller = get().runtimeController;
      closeController(controller);
      set((state) => buildRuntimeStoppedState(state, "Runtime stopped."));
    },

    resetRuntime() {
      const controller = get().runtimeController;
      closeController(controller);
      set((state) => buildRuntimeResetState(state));
    }
  };
}
