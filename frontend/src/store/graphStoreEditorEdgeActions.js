import { createGraphStoreEditorEdgeCreationActions } from "./graphStoreEditorEdgeCreationActions";
import { createGraphStoreEditorEdgeRemovalActions } from "./graphStoreEditorEdgeRemovalActions";

export function createGraphStoreEditorEdgeActions(set, get) {
  return {
    ...createGraphStoreEditorEdgeCreationActions(set, get),
    ...createGraphStoreEditorEdgeRemovalActions(set, get)
  };
}
