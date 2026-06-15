import { createGraphStoreRuntimeBacktestActions } from "./graphStoreRuntimeBacktestActions";
import { createGraphStoreRuntimeBacktestExperimentActions } from "./graphStoreRuntimeBacktestExperimentActions";
import { createGraphStoreRuntimeLifecycleActions } from "./graphStoreRuntimeLifecycleActions";
import { createGraphStoreRuntimeV4SimulationActions } from "./graphStoreRuntimeV4SimulationActions";

export function createGraphStoreRuntimeSessionActions(set, get) {
  return {
    ...createGraphStoreRuntimeBacktestActions(set, get),
    ...createGraphStoreRuntimeV4SimulationActions(set, get),
    ...createGraphStoreRuntimeBacktestExperimentActions(set, get),
    ...createGraphStoreRuntimeLifecycleActions(set, get)
  };
}
