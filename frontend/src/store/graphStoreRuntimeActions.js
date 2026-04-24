import { createGraphStoreRuntimeHistoryActions } from "./graphStoreRuntimeHistoryActions";
import { createGraphStoreRuntimeSessionActions } from "./graphStoreRuntimeSessionActions";

export function createGraphStoreRuntimeActions(set, get) {
  return {
    ...createGraphStoreRuntimeHistoryActions(set, get),
    ...createGraphStoreRuntimeSessionActions(set, get)
  };
}
