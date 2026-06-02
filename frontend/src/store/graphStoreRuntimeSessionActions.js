import { humanizeErrorText } from "../utils/errorText";
import { buildCapabilityContext } from "../capabilities/supportMatrix";
import {
  closeController,
  postJson,
  resolveGraphActor
} from "./graphStoreHelpers";
import { createGraphStoreRuntimeBacktestActions } from "./graphStoreRuntimeBacktestActions";
import { createGraphStoreRuntimeSimulationActions } from "./graphStoreRuntimeSimulationActions";
import { createGraphStoreRuntimeV4SimulationActions } from "./graphStoreRuntimeV4SimulationActions";
import {
  buildRuntimeResetState,
  buildRuntimeStoppedState,
  resolveRuntimeTargets
} from "./graphStoreRuntimeSessionState";
import { getRuntimeCapabilityBlockReason } from "./graphStoreRuntimeSessionShared";

export function createGraphStoreRuntimeSessionActions(set, get) {
  return {
    ...createGraphStoreRuntimeSimulationActions(set, get),
    ...createGraphStoreRuntimeBacktestActions(set, get),
    ...createGraphStoreRuntimeV4SimulationActions(set, get),

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
