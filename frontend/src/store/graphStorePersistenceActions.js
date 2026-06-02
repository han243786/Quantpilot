import {
  fetchJson,
  normalizeGraphAuditHistory,
  normalizeGraphVersionCompare,
  normalizeGraphVersions,
  postJson,
  resolveGraphActor,
  resolveLoadedGraphWithRegistry
} from "./graphStoreHelpers";
import { createGraphStoreGraphLifecycleActions } from "./graphStoreGraphLifecycleActions";

export function createGraphStorePersistenceActions(set, get) {
  return {
    ...createGraphStoreGraphLifecycleActions(set, get),

    async refreshGraphVersions(graphId = get().graph?.metadata?.graph_id || "") {
      if (!graphId || graphId === "draft_graph") {
        set({
          graphVersions: [],
          graphVersionsStatus: "idle",
          graphVersionsMessage: "",
          graphVersionPreview: null,
          graphVersionPreviewStatus: "idle",
          graphVersionPreviewMessage: "",
          graphVersionCompare: null,
          graphVersionCompareStatus: "idle",
          graphVersionCompareMessage: "",
          graphAuditHistory: [],
          graphAuditHistoryStatus: "idle",
          graphAuditHistoryMessage: ""
        });
        return [];
      }

      set({ graphVersionsStatus: "loading", graphVersionsMessage: "" });
      try {
        const graphVersions = normalizeGraphVersions(
          await fetchJson(`/graphs/${encodeURIComponent(graphId)}/versions`)
        );
        set({
          graphVersions,
          graphVersionsStatus: "ready",
          graphVersionsMessage: ""
        });
        return graphVersions;
      } catch (error) {
        set({
          graphVersions: [],
          graphVersionsStatus: "error",
          graphVersionsMessage: error.message || "加载持久化图谱版本失败。"
        });
        return [];
      }
    },

    async refreshGraphAuditHistory(graphId = get().graph?.metadata?.graph_id || "") {
      if (!graphId || graphId === "draft_graph") {
        set({
          graphAuditHistory: [],
          graphAuditHistoryStatus: "idle",
          graphAuditHistoryMessage: ""
        });
        return [];
      }

      set({ graphAuditHistoryStatus: "loading", graphAuditHistoryMessage: "" });
      try {
        const graphAuditHistory = normalizeGraphAuditHistory(
          await fetchJson(`/graphs/${encodeURIComponent(graphId)}/audit`)
        );
        set({
          graphAuditHistory,
          graphAuditHistoryStatus: "ready",
          graphAuditHistoryMessage: ""
        });
        return graphAuditHistory;
      } catch (error) {
        set({
          graphAuditHistory: [],
          graphAuditHistoryStatus: "error",
          graphAuditHistoryMessage: error.message || "加载图谱审计历史失败。"
        });
        return [];
      }
    },

    async loadGraphVersionPreview(graphId, versionId) {
      if (!graphId || !versionId) {
        throw new Error("需要同时提供图谱 ID 和版本 ID。");
      }

      set({ graphVersionPreviewStatus: "loading", graphVersionPreviewMessage: "" });
      try {
        const preview = resolveLoadedGraphWithRegistry(
          await fetchJson(
            `/graphs/${encodeURIComponent(graphId)}/versions/${encodeURIComponent(versionId)}`
          ),
          get().registry
        );
        if (!preview) {
          throw new Error("请求的图谱版本不可用。");
        }

        set({
          graphVersionPreview: {
            versionId,
            graph: preview
          },
          graphVersionPreviewStatus: "ready",
          graphVersionPreviewMessage: ""
        });
        return preview;
      } catch (error) {
        set({
          graphVersionPreview: null,
          graphVersionPreviewStatus: "error",
          graphVersionPreviewMessage: error.message || "加载所选图谱版本失败。"
        });
        throw error;
      }
    },

    clearGraphVersionPreview() {
      set({
        graphVersionPreview: null,
        graphVersionPreviewStatus: "idle",
        graphVersionPreviewMessage: ""
      });
    },

    async compareGraphVersions(graphId, leftVersionId, rightVersionId, evidence = {}) {
      if (!graphId || !leftVersionId || !rightVersionId) {
        throw new Error("需要提供图谱 ID 和两个版本 ID。");
      }

      set({
        graphVersionCompareStatus: "loading",
        graphVersionCompareMessage: ""
      });
      try {
        const query = new URLSearchParams();
        if (evidence.leftBacktestId) query.set("left_backtest_id", evidence.leftBacktestId);
        if (evidence.rightBacktestId) query.set("right_backtest_id", evidence.rightBacktestId);
        const suffix = query.toString() ? `?${query.toString()}` : "";
        const compare = normalizeGraphVersionCompare(
          await fetchJson(
            `/graphs/${encodeURIComponent(graphId)}/versions/compare/${encodeURIComponent(leftVersionId)}/${encodeURIComponent(rightVersionId)}${suffix}`
          )
        );
        set({
          graphVersionCompare: compare,
          graphVersionCompareStatus: "ready",
          graphVersionCompareMessage: ""
        });
        return compare;
      } catch (error) {
        set({
          graphVersionCompare: null,
          graphVersionCompareStatus: "error",
          graphVersionCompareMessage: error.message || "对比持久化图谱版本失败。"
        });
        throw error;
      }
    },

    clearGraphVersionCompare() {
      set({
        graphVersionCompare: null,
        graphVersionCompareStatus: "idle",
        graphVersionCompareMessage: ""
      });
    },

    async restoreGraphVersion(graphId, versionId) {
      if (!graphId || !versionId) {
        throw new Error("需要同时提供图谱 ID 和版本 ID。");
      }

      await postJson(
        `/graphs/${encodeURIComponent(graphId)}/versions/${encodeURIComponent(versionId)}/restore`,
        { actor: resolveGraphActor(get().graph) }
      );
      await get().refreshGraphIndex();
      await get().loadGraphById(graphId, { force: true });
      await get().refreshGraphVersions(graphId);
      await get().refreshGraphAuditHistory(graphId);
      get().clearGraphVersionPreview();
      get().clearGraphVersionCompare();
    }
  };
}
