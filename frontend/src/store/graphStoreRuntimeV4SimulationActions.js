import { humanizeErrorText } from "../utils/errorText";
import { postJson, updateRuntimeNode } from "./graphStoreHelpers";
import { buildRuntimeConnectingState } from "./graphStoreRuntimeSessionState";
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

export function createGraphStoreRuntimeV4SimulationActions(set, get) {
  return {
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
    }
  };
}
