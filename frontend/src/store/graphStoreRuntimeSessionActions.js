import { buildActionFailureMessage } from "../utils/actionFailure";
import { humanizeErrorText } from "../utils/errorText";
import {
  closeController,
  postJson,
  resolveGraphActor,
  saveGraphToStorage,
  updateRuntimeNode
} from "./graphStoreHelpers";
import { createRuntimeEventSource } from "./graphStoreRuntimeTransport";
import {
  applyRuntimeStreamState,
  buildBacktestCompletionState,
  buildRuntimeAccountState,
  buildRuntimeBindingGraph,
  buildRuntimeCompletionState,
  buildRuntimeConnectingState,
  buildRuntimeFailureState,
  buildRuntimeResetState,
  buildRuntimeStoppedState,
  resolveRuntimeTargets
} from "./graphStoreRuntimeSessionState";

export function createGraphStoreRuntimeSessionActions(set, get) {
  return {
    async startRuntime() {
      const graph = get().graph;
      if (!graph.validation_state.is_runnable) return;

      const result = await get().compileCurrentGraph();
      if (!result) return;

      get().stopRuntime();

      set((state) =>
        buildRuntimeConnectingState(
          state,
          "simulation",
          "Simulation started. Waiting for runtime events."
        )
      );

      try {
        const start = await postJson("/runtime/test-run", {
          actor: resolveGraphActor(graph),
          runtime_config: result.runtime_config,
          runtime_targets: resolveRuntimeTargets(result)
        });
        const source = createRuntimeEventSource(start.run_id);

        set((state) => ({
          graph: buildRuntimeBindingGraph(
            state.graph,
            start.run_id,
            result.backend_compile?.compile_id || result.compile_id
          )
        }));

        source.addEventListener("runtime_event", (message) => {
          const event = JSON.parse(message.data);
          set((state) => applyRuntimeStreamState(state, start.run_id, event));
        });

        source.addEventListener("account", (message) => {
          const account = JSON.parse(message.data);
          set((state) => ({
            runtime: buildRuntimeAccountState(state.runtime, account)
          }));
        });

        source.addEventListener("run_completed", async () => {
          source.close();
          set((state) => ({
            runtimeController: null,
            runtime: buildRuntimeCompletionState(state.runtime)
          }));
          await get().refreshRunHistory();
        });

        source.onerror = async () => {
          source.close();
          const message = buildActionFailureMessage(
            "sse_disconnect",
            "\u4e8b\u4ef6\u6d41\u8fde\u63a5\u5df2\u5173\u95ed\u3002",
            "\u4e8b\u4ef6\u6d41\u8fde\u63a5\u5df2\u5173\u95ed\u3002"
          );
          set((state) => ({
            runtimeController: null,
            runtime: buildRuntimeFailureState(state.runtime, message)
          }));
          await get().refreshRunHistory();
        };

        set((state) => ({
          runtimeController: { close: () => source.close() },
          runtime: {
            ...state.runtime,
            runId: start.run_id,
            runKind: "simulation",
            status: "running",
            connectionState: "connected",
            backendError: null,
            backtestArtifacts: null,
            diagnostics: null,
            selectedHistoryRunId: start.run_id,
            selectedBacktestId: null,
            highlightedNodeIds: []
          }
        }));

        await get().refreshRunHistory();
      } catch (error) {
        const message = humanizeErrorText(error, "Failed to start simulation.");
        set((state) => ({
          runtimeController: null,
          runtime: {
            ...state.runtime,
            status: "error",
            backendError: message
          },
          graph: {
            ...state.graph,
            nodes: updateRuntimeNode(state.graph.nodes, "error", message)
          }
        }));
      }
    },

    async startBacktest() {
      const graph = get().graph;
      if (!graph.validation_state.is_runnable) return;

      const result = await get().compileCurrentGraph();
      if (!result) return;

      get().stopRuntime();

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
          runtime_config: result.runtime_config,
          runtime_targets: resolveRuntimeTargets(result),
          backtest_options: {
            replay_source: "deterministic_mock"
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
        await get().refreshBacktestHistory();
      } catch (error) {
        const message = humanizeErrorText(error, "Backtest request failed.");
        set((state) => ({
          runtime: {
            ...state.runtime,
            runKind: "backtest",
            status: "error",
            backendError: message,
            backtestArtifacts: null,
            diagnostics: null,
            selectedBacktestId: null
          },
          graph: {
            ...state.graph,
            nodes: updateRuntimeNode(state.graph.nodes, "error", message)
          }
        }));
      }
    },

    async startBacktestExperiment(options = {}) {
      const graph = get().graph;
      if (!graph.validation_state.is_runnable) return null;

      const result = await get().compileCurrentGraph();
      if (!result) return null;

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
          runtime_config: result.runtime_config,
          runtime_targets: resolveRuntimeTargets(result),
          backtest_options: {
            replay_source: "deterministic_mock"
          },
          parameter_grid: parameterGrid
        });
        await get().refreshBacktestHistory();
        await get().refreshExperimentHistory();
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
