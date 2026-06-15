import { buildCapabilityContext } from "../capabilities/supportMatrix";
import { humanizeErrorText } from "../utils/errorText";
import {
  postJson,
  resolveGraphActor,
  saveGraphToStorage,
  updateRuntimeNode
} from "./graphStoreHelpers";
import {
  buildBacktestCompletionState,
  buildRuntimeConnectingState
} from "./graphStoreRuntimeSessionState";
import {
  getRuntimeCapabilityBlockReason,
  setRuntimeCapabilityBlocked
} from "./graphStoreRuntimeSessionShared";

export function createGraphStoreRuntimeBacktestActions(set, get) {
  return {
    async startBacktest() {
      if (get().actionLock) return;
      const graph = get().graph;
      if (!graph.validation_state.is_runnable) return;
      const capabilityBlockReason = getRuntimeCapabilityBlockReason(get, "run_backtest");
      if (capabilityBlockReason) {
        setRuntimeCapabilityBlocked(set, "backtest", capabilityBlockReason);
        return;
      }

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
    }
  };
}
