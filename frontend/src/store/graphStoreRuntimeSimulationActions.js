import { buildCapabilityContext } from "../capabilities/supportMatrix";
import { buildActionFailureMessage } from "../utils/actionFailure";
import { humanizeErrorText } from "../utils/errorText";
import {
  postJson,
  resolveGraphActor,
  updateRuntimeNode
} from "./graphStoreHelpers";
import { createRuntimeEventSource } from "./graphStoreRuntimeTransport";
import {
  applyRuntimeStreamState,
  buildRuntimeAccountState,
  buildRuntimeBindingGraph,
  buildRuntimeCompletionState,
  buildRuntimeConnectingState,
  buildRuntimeFailureState,
  resolveRuntimeTargets
} from "./graphStoreRuntimeSessionState";
import {
  getRuntimeCapabilityBlockReason,
  setRuntimeCapabilityBlocked
} from "./graphStoreRuntimeSessionShared";

export function createGraphStoreRuntimeSimulationActions(set, get) {
  return {
    async startRuntime() {
      if (get().actionLock) return;
      const graph = get().graph;
      if (!graph.validation_state.is_runnable) return;
      const capabilityBlockReason = getRuntimeCapabilityBlockReason(get, "start_simulation");
      if (capabilityBlockReason) {
        setRuntimeCapabilityBlocked(set, "simulation", capabilityBlockReason);
        return;
      }

      const result = await get().compileCurrentGraph();
      if (!result) return;
      const capabilityContext = buildCapabilityContext(get().capabilities);

      get().stopRuntime();
      set({ actionLock: "runtime" });

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
          capability_context: capabilityContext,
          runtime_config: result.runtime_config,
          runtime_targets: resolveRuntimeTargets(result),
          graph_json: graph
        });
        const sourceRef = { current: null };
        const source = createRuntimeEventSource(start.run_id, () => {
          // \u91cd\u8fde\u8017\u5c3d: \u8bbe\u7f6e\u5931\u8d25\u72b6\u6001
          const message = buildActionFailureMessage(
            "sse_disconnect",
            "\u4e8b\u4ef6\u6d41\u8fde\u63a5\u5df2\u5173\u95ed\u3002",
            "\u91cd\u8fde\u5c1d\u8bd5\u5df2\u8017\u5c3d\uff0c\u8bf7\u68c0\u67e5\u540e\u7aef\u662f\u5426\u4ecd\u5728\u8fd0\u884c\u3002"
          );
          set((state) => ({
            runtimeController: null,
            runtime: buildRuntimeFailureState(state.runtime, message)
          }));
        }, (newSource) => {
          // \u91cd\u8fde\u540e\u66f4\u65b0 sourceRef, \u786e\u4fdd close() \u5173\u95ed\u7684\u662f\u65b0\u8fde\u63a5
          sourceRef.current = newSource;
        });

        set((state) => ({
          graph: buildRuntimeBindingGraph(
            state.graph,
            start.run_id,
            result.backend_compile?.compile_id || result.compile_id
          )
        }));

        // v1.0.5: SSE 微批处理 — 窗口内的事件合并为一次 set()
        const SSE_BATCH_WINDOW_MS = 50;
        let batchTimer = null;
        let batchedEvents = [];
        const flushBatch = () => {
          if (batchedEvents.length === 0) return;
          const events = batchedEvents;
          batchedEvents = [];
          batchTimer = null;
          set((state) => {
            let s = state;
            for (const apply of events) s = apply(s);
            return s;
          });
        };
        // v1.1.7: 在源上存储 batchTimer，stopRuntime 时可清理

        source._onMessage = (message) => {
          try {
            const event = JSON.parse(message.data);
            event._timeLabel = new Date(event.event_time_ms).toLocaleTimeString();
            batchedEvents.push((state) => applyRuntimeStreamState(state, start.run_id, event));
            if (!batchTimer) batchTimer = setTimeout(flushBatch, SSE_BATCH_WINDOW_MS);
          } catch (e) {
            console.warn("[sse] runtime_event JSON 解析失败", e);
          }
        };
        source.addEventListener("runtime_event", source._onMessage);

        source._onAccount = (message) => {
          try {
            const account = JSON.parse(message.data);
            batchedEvents.push((state) => ({
              ...state,
              runtime: buildRuntimeAccountState(state.runtime, account)
            }));
            if (!batchTimer) batchTimer = setTimeout(flushBatch, SSE_BATCH_WINDOW_MS);
          } catch (e) {
            console.warn("[sse] account JSON 解析失败", e);
          }
        };
        source.addEventListener("account", source._onAccount);

        source._onComplete = async () => {
          source.close();
          flushBatch();
          set((state) => ({
            runtimeController: null,
            runtime: buildRuntimeCompletionState(state.runtime, start.run_id)
          }));
        };
        source.addEventListener("run_completed", source._onComplete);

        source._onError = () => {
          set((state) => ({
            runtime: { ...state.runtime, status: "reconnecting" }
          }));
          source._reconnect?.();
        };
        source.onerror = source._onError;

        // v1.0.5: 用 ref 跟踪活动源, 重连后 close 仍能关闭新连接
        sourceRef.current = source;

        set((state) => ({
          runtimeController: {
            close: () => {
              if (sourceRef.current?._reconnectTimer) clearTimeout(sourceRef.current._reconnectTimer);
              // v1.1.7: 清理挂起的批次定时器，防止陈旧事件污染
              if (batchTimer) { clearTimeout(batchTimer); batchTimer = null; }
              batchedEvents = [];
              sourceRef.current?.close();
            }
          },
          runtime: {
            ...state.runtime,
            runId: start.run_id,
            runKind: "simulation",
            status: "running",
            connectionState: "connected",
            backendError: null,
            artifactPersistenceStatus: "idle",
            backtestArtifacts: null,
            diagnostics: null,
            governance: null,
            selectedHistoryRunId: start.run_id,
            selectedBacktestId: null,
            highlightedNodeIds: []
          }
        }));
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
      } finally {
        set({ actionLock: null });
      }
    }
  };
}
