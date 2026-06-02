import { closeController } from "./graphStoreHelpers";
import { createGraphStoreRuntimeBacktestActions } from "./graphStoreRuntimeBacktestActions";
import { createGraphStoreRuntimeBacktestExperimentActions } from "./graphStoreRuntimeBacktestExperimentActions";
import { createGraphStoreRuntimeSimulationActions } from "./graphStoreRuntimeSimulationActions";
import { createGraphStoreRuntimeV4SimulationActions } from "./graphStoreRuntimeV4SimulationActions";
import {
  buildRuntimeResetState,
  buildRuntimeStoppedState
} from "./graphStoreRuntimeSessionState";

export function createGraphStoreRuntimeSessionActions(set, get) {
  return {
    ...createGraphStoreRuntimeSimulationActions(set, get),
    ...createGraphStoreRuntimeBacktestActions(set, get),
    ...createGraphStoreRuntimeV4SimulationActions(set, get),
    ...createGraphStoreRuntimeBacktestExperimentActions(set, get),

    stopRuntime() {
      const controller = get().runtimeController;
      closeController(controller);
      set((state) => buildRuntimeStoppedState(state, "Runtime stopped."));
    },

    resetRuntime() {
      const controller = get().runtimeController;
      closeController(controller);
      set((state) => buildRuntimeResetState(state));
    }
  };
}
