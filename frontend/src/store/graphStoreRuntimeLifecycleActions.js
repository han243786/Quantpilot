import { closeController } from "./graphStoreHelpers";
import {
  buildRuntimeResetState,
  buildRuntimeStoppedState
} from "./graphStoreRuntimeSessionState";

export function createGraphStoreRuntimeLifecycleActions(set, get) {
  return {
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
