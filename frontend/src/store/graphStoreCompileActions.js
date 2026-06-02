import { createGraphStoreCompileCurrentGraphActions } from "./graphStoreCompileCurrentGraphActions";
import { createGraphStoreCompileExportActions } from "./graphStoreCompileExportActions";
import { createGraphStoreCompileSourceActions } from "./graphStoreCompileSourceActions";

export function createGraphStoreCompileActions(set, get) {
  return {
    ...createGraphStoreCompileSourceActions(set, get),
    ...createGraphStoreCompileExportActions(set, get),
    ...createGraphStoreCompileCurrentGraphActions(set, get)
  };
}
