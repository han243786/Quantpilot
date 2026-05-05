import { useI18n } from "../i18n";
import { useGraphStore } from "../store/graphStore";
import { buildActionFailureMessage } from "../utils/actionFailure";
import { buildNotice, firstBlockingMessage } from "./workspaceActionBarShared";

function downloadText(filename, data, contentType) {
  const blob = new Blob([data], { type: contentType });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  link.click();
  URL.revokeObjectURL(url);
}

function downloadJson(filename, data) {
  downloadText(filename, JSON.stringify(data, null, 2), "application/json");
}

export function useWorkspaceActionBarActions({ onNotice } = {}) {
  const { t } = useI18n();
  const saveGraph = useGraphStore((state) => state.saveGraph);
  const loadLatestGraph = useGraphStore((state) => state.loadLatestGraph);
  const exportRuntimeConfig = useGraphStore((state) => state.exportRuntimeConfig);
  const exportQuantScript = useGraphStore((state) => state.exportQuantScript);
  const compileCurrentGraph = useGraphStore((state) => state.compileCurrentGraph);
  const startRuntime = useGraphStore((state) => state.startRuntime);
  const startBacktest = useGraphStore((state) => state.startBacktest);
  const stopRuntime = useGraphStore((state) => state.stopRuntime);
  const resetRuntime = useGraphStore((state) => state.resetRuntime);
  const resetGraph = useGraphStore((state) => state.resetGraph);
  const setSelectedNode = useGraphStore((state) => state.setSelectedNode);
  const setSelectedEdge = useGraphStore((state) => state.setSelectedEdge);

  function pushNotice(type, message, fallback = message) {
    if (typeof onNotice === "function") {
      onNotice(buildNotice(type, message, fallback));
    }
  }

  async function handleSaveGraph() {
    try {
      await saveGraph();
      pushNotice("success", t("策略图已保存。"), t("策略图已保存。"));
    } catch (error) {
      pushNotice(
        "error",
        buildActionFailureMessage("save_graph", error, t("保存策略图失败。")),
        t("保存策略图失败。")
      );
    }
  }

  async function handleLoadLatestGraph() {
    try {
      await loadLatestGraph();
      pushNotice("success", t("已加载最新保存的策略图。"), t("已加载最新保存的策略图。"));
    } catch (error) {
      pushNotice(
        "error",
        buildActionFailureMessage("load_latest", error, t("加载最新策略图失败。")),
        t("加载最新策略图失败。")
      );
    }
  }

  async function handleCompile({ capabilitySyncBlocked, capabilityMessage } = {}) {
    if (capabilitySyncBlocked) {
      pushNotice(
        "error",
        buildActionFailureMessage("compile", capabilityMessage, t("能力尚未就绪，暂时无法编译。")),
        t("编译暂时被阻止。")
      );
      return;
    }

    const result = await compileCurrentGraph();
    if (!result) {
      const message =
        useGraphStore.getState().graph.compile_summary?.errors?.[0] || t("策略图编译失败。");
      pushNotice(
        "error",
        buildActionFailureMessage("compile", message, t("策略图编译失败。")),
        t("策略图编译失败。")
      );
      return;
    }

    pushNotice("success", t("编译完成，且已通过后端校验。"), t("编译完成。"));
  }

  async function handleStartRuntime({ graph, capabilitySyncBlocked, capabilityMessage, mode } = {}) {
    if (capabilitySyncBlocked) {
      pushNotice(
        "error",
        buildActionFailureMessage("simulation", capabilityMessage, t("能力尚未就绪，暂时无法启动模拟。")),
        t("模拟运行暂时被阻止。")
      );
      return;
    }
    if (mode === "testnet") {
      pushNotice(
        "info",
        t("模拟盘交易请通过 QuantScript 编辑器使用 @run { mode: \"testnet\" } 指令启动。"),
        t("请使用 QS 编辑器启动模拟盘。")
      );
      return;
    }
    if (!graph.validation_state.is_runnable) {
      pushNotice(
        "error",
        buildActionFailureMessage("simulation", firstBlockingMessage(graph, t), t("当前策略图不可运行。")),
        t("当前策略图不可运行。")
      );
      return;
    }

    await startRuntime();

    const { runtime: nextRuntime } = useGraphStore.getState();
    if (nextRuntime.status === "error" && nextRuntime.backendError) {
      pushNotice(
        "error",
        buildActionFailureMessage("simulation", nextRuntime.backendError, t("启动模拟运行失败。")),
        t("启动模拟运行失败。")
      );
      return;
    }

    pushNotice("info", t("模拟运行已启动，正在等待运行时事件。"), t("模拟运行已启动。"));
  }

  async function handleStartBacktest({ graph, capabilitySyncBlocked, capabilityMessage } = {}) {
    if (capabilitySyncBlocked) {
      pushNotice(
        "error",
        buildActionFailureMessage("backtest", capabilityMessage, t("能力尚未就绪，暂时无法启动回测。")),
        t("回测启动暂时被阻止。")
      );
      return;
    }
    if (!graph.validation_state.is_runnable) {
      pushNotice(
        "error",
        buildActionFailureMessage("backtest", firstBlockingMessage(graph, t), t("当前策略图不可运行。")),
        t("当前策略图不可运行。")
      );
      return;
    }

    await startBacktest();

    const { runtime: nextRuntime } = useGraphStore.getState();
    if (nextRuntime.status === "error" && nextRuntime.backendError) {
      pushNotice(
        "error",
        buildActionFailureMessage("backtest", nextRuntime.backendError, t("回测执行失败。")),
        t("回测执行失败。")
      );
      return;
    }

    pushNotice("success", t("回测已完成。"), t("回测已完成。"));
  }

  async function handleExportRuntimeConfig({ capabilitySyncBlocked, capabilityMessage } = {}) {
    if (capabilitySyncBlocked) {
      pushNotice(
        "error",
        buildActionFailureMessage(
          "export_config",
          capabilityMessage,
          t("能力尚未就绪，暂时无法导出运行配置。")
        ),
        t("运行配置导出暂时被阻止。")
      );
      return;
    }

    const result = await exportRuntimeConfig();
    if (!result.compile_summary.compilable) {
      pushNotice(
        "error",
        buildActionFailureMessage(
          "export_config",
          result.compile_summary.errors?.[0],
          t("策略图编译失败，当前无法导出。")
        ),
        t("运行配置当前不可导出。")
      );
      return;
    }
    downloadJson("runtime-config.json", result.runtime_config);
    pushNotice("success", t("运行配置已导出。"), t("运行配置已导出。"));
  }

  function handleExportQuantScript({ graph } = {}) {
    const source = exportQuantScript();
    downloadText(`${graph.metadata.graph_id || "strategy"}.qs`, source, "text/plain;charset=utf-8");
    pushNotice("success", t("已导出策略图源码。"), t("已导出策略图源码。"));
  }

  function focusFinding(issue) {
    if (issue.scope === "node") {
      setSelectedEdge(null);
      setSelectedNode(issue.target_id);
      return;
    }

    if (issue.scope === "edge") {
      setSelectedNode(null);
      setSelectedEdge(issue.target_id);
      return;
    }

    setSelectedNode(null);
    setSelectedEdge(null);
  }

  return {
    handleSaveGraph,
    handleLoadLatestGraph,
    handleCompile,
    handleStartRuntime,
    handleStartBacktest,
    handleExportRuntimeConfig,
    handleExportQuantScript,
    stopRuntime,
    resetRuntime,
    resetGraph,
    focusFinding
  };
}
