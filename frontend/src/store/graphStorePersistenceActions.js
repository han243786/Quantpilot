import { createGraphStoreGraphLifecycleActions } from "./graphStoreGraphLifecycleActions";
import { createGraphStoreVersionAuditActions } from "./graphStoreVersionAuditActions";

export function createGraphStorePersistenceActions(set, get) {
  return {
    ...createGraphStoreGraphLifecycleActions(set, get),
    ...createGraphStoreVersionAuditActions(set, get)
  };
}
