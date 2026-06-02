import { createGraphStoreCompileActions } from "./graphStoreCompileActions";
import { createGraphStoreEditorDraftActions } from "./graphStoreEditorDraftActions";
import { createGraphStoreEditorEdgeActions } from "./graphStoreEditorEdgeActions";
import { createGraphStoreEditorNodeActions } from "./graphStoreEditorNodeActions";
import { createGraphStoreEditorSelectionActions } from "./graphStoreEditorSelectionActions";
import { createGraphStoreEditorTemplateActions } from "./graphStoreEditorTemplateActions";
import { createGraphStorePersistenceActions } from "./graphStorePersistenceActions";

export function createGraphStoreEditorActions(set, get) {
  return {
    ...createGraphStoreEditorDraftActions(set, get),
    ...createGraphStoreEditorSelectionActions(set, get),
    ...createGraphStoreEditorTemplateActions(set, get),
    ...createGraphStoreEditorNodeActions(set, get),
    ...createGraphStoreEditorEdgeActions(set, get),
    ...createGraphStoreCompileActions(set, get),
    ...createGraphStorePersistenceActions(set, get)
  };
}
