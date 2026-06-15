import { getCapabilityActionBlockReason, isCapabilitySyncBlocked } from "../capabilities/supportMatrix";
import { projectUiActions } from "../capabilities/capabilityProjection";
import { humanizeErrorText } from "../utils/errorText";

export function formatScopeLabel(issue, graph, t) {
  if (issue.scope === "graph") return t("策略图");

  if (issue.scope === "node") {
    const node = graph.nodes.find((item) => item.id === issue.target_id);
    return node ? `${t("节点")}: ${node.name}` : t("节点");
  }

  if (issue.scope === "edge") {
    const edge = graph.edges.find((item) => item.id === issue.target_id);
    if (!edge) return t("连线");
    const sourceNode = graph.nodes.find((item) => item.id === edge.source_node_id);
    const targetNode = graph.nodes.find((item) => item.id === edge.target_node_id);
    const sourceName = sourceNode?.name || edge.source_node_id;
    const targetName = targetNode?.name || edge.target_node_id;
    return `${t("连线")}: ${sourceName} -> ${targetName}`;
  }

  return t("未知范围");
}

export function collectValidationFindings(graph, t) {
  const graphIssues = (graph.validation_state?.graph_issues || []).map((issue) => ({
    ...issue,
    scopeLabel: formatScopeLabel(issue, graph, t)
  }));
  const nodeIssues = Object.entries(graph.validation_state?.node_issues || {}).flatMap(
    ([, issues]) =>
      (issues || []).map((issue) => ({
        ...issue,
        scopeLabel: formatScopeLabel(issue, graph, t)
      }))
  );
  const edgeIssues = Object.entries(graph.validation_state?.edge_issues || {}).flatMap(
    ([, issues]) =>
      (issues || []).map((issue) => ({
        ...issue,
        scopeLabel: formatScopeLabel(issue, graph, t)
      }))
  );

  const levelWeight = { error: 0, warning: 1, info: 2 };
  return [...graphIssues, ...nodeIssues, ...edgeIssues].sort((left, right) => {
    const leftWeight = levelWeight[left.level] ?? 99;
    const rightWeight = levelWeight[right.level] ?? 99;
    if (leftWeight !== rightWeight) return leftWeight - rightWeight;
    return left.scopeLabel.localeCompare(right.scopeLabel, "zh-CN");
  });
}

export function firstBlockingMessage(graph, t) {
  const graphIssue = graph.validation_state?.graph_issues?.[0];
  if (graphIssue?.message) return graphIssue.message;

  const nodeIssue = Object.values(graph.validation_state?.node_issues || {})
    .flat()
    .find((issue) => issue.level === "error");
  if (nodeIssue?.message) return nodeIssue.message;

  const edgeIssue = Object.values(graph.validation_state?.edge_issues || {})
    .flat()
    .find((issue) => issue.level === "error");
  if (edgeIssue?.message) return edgeIssue.message;

  return t("当前策略图存在阻塞问题，需要先修复后再继续。");
}

export function buildNotice(type, message, fallback) {
  return {
    id: Date.now(),
    type,
    message: humanizeErrorText(message, fallback)
  };
}

export function capabilityStateLabel(status, source, t) {
  if (status === "loading") return { text: t("能力同步中"), tone: "warning" };
  if (source === "cache") return { text: t("缓存快照"), tone: "warning" };
  if (source === "safe_fallback") return { text: t("安全回退"), tone: "danger" };
  return { text: t("能力已同步"), tone: "success" };
}

export function capabilityBanner(status, source, message, t) {
  if (status === "loading") {
    return {
      type: "info",
      message: t("前端正在同步后端能力快照。同步完成前，部分高风险操作会暂时锁定。")
    };
  }
  if (!message) return null;
  if (source === "cache") return { type: "warning", message };
  if (source === "safe_fallback") return { type: "error", message };
  return null;
}

function resolveFormalCompileSourceMeta({ graph, formalQuantScriptOverride }) {
  if (formalQuantScriptOverride !== null) {
    return {
      text: "正式源码: 覆盖",
      tone: "warning",
      title: "当前编译使用了已应用的正式 QuantScript 覆盖。"
    };
  }

  if (graph.metadata?.artifacts?.quantscript?.formal_source) {
    return {
      text: "正式源码: 图谱生成",
      tone: "success",
      title: "当前编译使用了图谱生成的正式 QuantScript 源码。"
    };
  }

  return {
    text: "正式源码: 不可用",
    tone: "danger",
    title: "当前没有可用的正式 QuantScript 源码用于编译。"
  };
}

function hasRunnableV4QuantScriptSource({ graph, formalQuantScriptOverride }) {
  const source =
    typeof formalQuantScriptOverride === "string" && formalQuantScriptOverride.trim()
      ? formalQuantScriptOverride
      : graph.metadata?.artifacts?.quantscript?.formal_source || "";
  return /^\s*v4_strategy\s+\S+\s*\{/m.test(source);
}

export function resolveWorkspaceActionState({
  graph,
  runtime,
  capabilityStatus,
  capabilitySource,
  capabilityMessage,
  capabilities,
  formalQuantScriptOverride,
  t
}) {
  const statusLabel = !graph.validation_state.is_valid
    ? { text: t("存在阻塞问题"), tone: "danger" }
    : !graph.validation_state.is_runnable
      ? { text: t("可编辑但不可运行"), tone: "warning" }
      : { text: t("可运行"), tone: "success" };

  const validationFindings = collectValidationFindings(graph, t);
  const visibleFindings = validationFindings.slice(0, 5);
  const hiddenFindingCount = Math.max(validationFindings.length - visibleFindings.length, 0);
  const capabilitySyncBlocked = isCapabilitySyncBlocked(capabilityStatus, capabilitySource);
  const actionStates = projectUiActions({
    capabilities,
    capabilityStatus,
    capabilitySource,
    capabilityMessage
  });
  const isActionEnabled = (actionKey) => actionStates[actionKey]?.enabled !== false;
  const exportConfigCapabilityReason = getCapabilityActionBlockReason({
    actionKey: "export_runtime_config",
    capabilityStatus,
    capabilitySource,
    capabilityMessage,
    capabilities
  });
  const compileCapabilityReason = getCapabilityActionBlockReason({
    actionKey: "compile",
    capabilityStatus,
    capabilitySource,
    capabilityMessage,
    capabilities
  });
  const startSimulationCapabilityReason = getCapabilityActionBlockReason({
    actionKey: "start_v4_simulation",
    capabilityStatus,
    capabilitySource,
    capabilityMessage,
    capabilities
  });
  const startV4SimulationCapabilityReason = getCapabilityActionBlockReason({
    actionKey: "start_v4_simulation",
    capabilityStatus,
    capabilitySource,
    capabilityMessage,
    capabilities
  });
  const runBacktestCapabilityReason = getCapabilityActionBlockReason({
    actionKey: "run_backtest",
    capabilityStatus,
    capabilitySource,
    capabilityMessage,
    capabilities
  });
  const formalCompileSourceMeta = resolveFormalCompileSourceMeta({
    graph,
    formalQuantScriptOverride
  });
  const hasV4RuntimeSource = hasRunnableV4QuantScriptSource({
    graph,
    formalQuantScriptOverride
  });
  const compileButtonBlockReason =
    compileCapabilityReason ||
    (!graph.validation_state.is_valid
      ? t("策略图仍有阻塞问题，修复后才能执行编译。")
      : undefined);
  const compileButtonTitle = [compileButtonBlockReason, formalCompileSourceMeta.title]
    .filter(Boolean)
    .join(" | ");

  return {
    graph,
    runtime,
    capabilityMessage,
    statusLabel,
    capabilityLabel: capabilityStateLabel(capabilityStatus, capabilitySource, t),
    capabilityAlert: capabilityBanner(capabilityStatus, capabilitySource, capabilityMessage, t),
    validationFindings,
    visibleFindings,
    hiddenFindingCount,
    actionStates,
    capabilitySyncBlocked,
    formalCompileSourceMeta,
    compileButtonTitle: compileButtonTitle || undefined,
    startSimulationTitle:
      startSimulationCapabilityReason ||
      (runtime.status === "running" || runtime.status === "connecting"
        ? t("运行时已经在执行中，不能重复启动。")
        : undefined),
    startV4SimulationTitle:
      startV4SimulationCapabilityReason ||
      (!hasV4RuntimeSource
        ? "当前没有可运行的 v4 QuantScript 源码。"
        : runtime.status === "running" || runtime.status === "connecting"
          ? "运行时正在执行中，不能重复启动 v4 模拟。"
          : undefined),
    runBacktestTitle:
      runBacktestCapabilityReason ||
      (runtime.status === "running" || runtime.status === "connecting"
        ? t("运行时执行期间不能启动新的回测。")
        : undefined),
    exportConfigTitle: exportConfigCapabilityReason || undefined,
    saveGraphTitle: actionStates.save_graph?.blockReason || actionStates.save_graph?.reason || undefined,
    loadLatestTitle:
      actionStates.load_latest_graph?.blockReason ||
      actionStates.load_latest_graph?.reason ||
      undefined,
    resetGraphTitle: actionStates.reset_graph?.blockReason || actionStates.reset_graph?.reason || undefined,
    exportQuantScriptTitle:
      actionStates.export_quantscript?.blockReason ||
      actionStates.export_quantscript?.reason ||
      undefined,
    stopRuntimeTitle: actionStates.stop_runtime?.blockReason || actionStates.stop_runtime?.reason || undefined,
    resetRuntimeTitle:
      actionStates.reset_runtime?.blockReason || actionStates.reset_runtime?.reason || undefined,
    tutorialTitle: actionStates.open_tutorial?.blockReason || t("查看使用教程"),
    credentialsTitle:
      actionStates.manage_credentials?.blockReason ||
      actionStates.manage_credentials?.reason ||
      t("管理交易所凭证"),
    openBacktestsTitle:
      actionStates.open_backtests?.blockReason || actionStates.open_backtests?.reason || undefined,
    canCompile:
      graph.validation_state.is_valid &&
      !capabilitySyncBlocked &&
      isActionEnabled("compile"),
    canStartRuntime:
      runtime.status !== "running" &&
      runtime.status !== "connecting" &&
      !capabilitySyncBlocked &&
      isActionEnabled("start_v4_simulation") &&
      hasV4RuntimeSource,
    canStartV4Simulation:
      runtime.status !== "running" &&
      runtime.status !== "connecting" &&
      !capabilitySyncBlocked &&
      isActionEnabled("start_v4_simulation") &&
      hasV4RuntimeSource,
    canStartBacktest:
      runtime.status !== "running" &&
      runtime.status !== "connecting" &&
      !capabilitySyncBlocked &&
      isActionEnabled("run_backtest"),
    canStopRuntime:
      (runtime.status === "running" || runtime.status === "connecting") &&
      isActionEnabled("stop_runtime"),
    canSaveGraph: isActionEnabled("save_graph"),
    canLoadLatestGraph: isActionEnabled("load_latest_graph"),
    canResetGraph: isActionEnabled("reset_graph"),
    canResetRuntime: isActionEnabled("reset_runtime"),
    canExportRuntimeConfig: !capabilitySyncBlocked && isActionEnabled("export_runtime_config"),
    canExportQuantScript: isActionEnabled("export_quantscript"),
    canOpenTutorial: isActionEnabled("open_tutorial"),
    canOpenCredentials: isActionEnabled("manage_credentials"),
    canOpenBacktests: isActionEnabled("open_backtests"),
    issueSummary:
      (graph.validation_state?.issue_counts?.error || 0) > 0
        ? `${graph.validation_state.issue_counts.error}E`
        : (graph.validation_state?.issue_counts?.warning || 0) > 0
          ? `${graph.validation_state.issue_counts.warning}W`
          : ""
  };
}
