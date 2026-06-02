import { createGraphStoreEditorNodeConfigActions } from "./graphStoreEditorNodeConfigActions";
import { createGraphStoreEditorNodeCreationActions } from "./graphStoreEditorNodeCreationActions";
import { createGraphStoreEditorNodePositionActions } from "./graphStoreEditorNodePositionActions";
import { createGraphStoreEditorNodeUiActions } from "./graphStoreEditorNodeUiActions";

export function createGraphStoreEditorNodeActions(set, get) {
  return {
    ...createGraphStoreEditorNodeCreationActions(set, get),
    ...createGraphStoreEditorNodePositionActions(set, get),
    ...createGraphStoreEditorNodeConfigActions(set, get),
    ...createGraphStoreEditorNodeUiActions(set, get)
  };
}
