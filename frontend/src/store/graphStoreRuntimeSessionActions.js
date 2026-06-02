import { createGraphStoreRuntimeBacktestActions } from "./graphStoreRuntimeBacktestActions";
import { createGraphStoreRuntimeBacktestExperimentActions } from "./graphStoreRuntimeBacktestExperimentActions";
import { createGraphStoreRuntimeLifecycleActions } from "./graphStoreRuntimeLifecycleActions";
import { createGraphStoreRuntimeSimulationActions } from "./graphStoreRuntimeSimulationActions";
import { createGraphStoreRuntimeV4SimulationActions } from "./graphStoreRuntimeV4SimulationActions";

export function createGraphStoreRuntimeSessionActions(set, get) {
  return {
    ...createGraphStoreRuntimeSimulationActions(set, get),
    ...createGraphStoreRuntimeBacktestActions(set, get),
    ...createGraphStoreRuntimeV4SimulationActions(set, get),
    ...createGraphStoreRuntimeBacktestExperimentActions(set, get),
    ...createGraphStoreRuntimeLifecycleActions(set, get)
  };
}
