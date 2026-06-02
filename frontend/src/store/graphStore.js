import { create } from "zustand";
import { fetchJson } from "./graphStoreHelpers";
import { createInitialGraphStoreState } from "./graphStoreRootState";
import {
  buildCapabilityRefreshFailureState,
  buildRemoteCapabilityRefreshState
} from "./graphStoreCapabilityRefresh";
import { createGraphStoreEditorActions } from "./graphStoreEditorActions";
import { createGraphStoreRuntimeActions } from "./graphStoreRuntimeActions";
import { createGraphStoreStartupActions } from "./graphStoreStartupActions";

export { fetchJson } from "./graphStoreHelpers";

export const useGraphStore = create((set, get) => ({
  ...createInitialGraphStoreState(),

  async refreshCapabilities() {
    set({ capabilityStatus: "loading", capabilityMessage: "" });

    try {
      const capabilities = await fetchJson("/capabilities");
      const refresh = buildRemoteCapabilityRefreshState(capabilities, get());
      set(refresh.state);
      return refresh.capabilities;
    } catch (error) {
      const refresh = buildCapabilityRefreshFailureState(error, get(), {
        loadFailureFallback: "能力加载失败。",
        cacheFallbackMessage: "能力加载失败，已启用本地缓存的能力快照。最终可用性取决于后端实时验证。",
        safeFallbackMessage: "能力加载失败，已进入安全回退模式。为避免暴露虚假能力，模块可见性和编译/运行操作已收紧至最安全配置。"
      });
      set(refresh.state);
      return refresh.capabilities;
    }
  },

  ...createGraphStoreStartupActions(set, get),
  ...createGraphStoreEditorActions(set, get),
  ...createGraphStoreRuntimeActions(set, get)
}));
