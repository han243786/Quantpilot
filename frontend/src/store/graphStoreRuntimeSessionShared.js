import { getCapabilityActionBlockReason } from "../capabilities/supportMatrix";
import { updateRuntimeNode } from "./graphStoreHelpers";

export function getRuntimeCapabilityBlockReason(get, actionKey) {
  return getCapabilityActionBlockReason({
    actionKey,
    capabilityStatus: get().capabilityStatus,
    capabilitySource: get().capabilitySource,
    capabilityMessage: get().capabilityMessage,
    capabilities: get().capabilities
  });
}

export function setRuntimeCapabilityBlocked(set, runKind, reason) {
  set((state) => ({
    runtime: {
      ...state.runtime,
      runKind,
      status: "error",
      backendError: reason,
      artifactPersistenceStatus: "idle",
      diagnostics: null,
      governance: null
    },
    graph: {
      ...state.graph,
      nodes: updateRuntimeNode(state.graph.nodes, "error", reason)
    }
  }));
}
