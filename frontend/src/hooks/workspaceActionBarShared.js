import { getCapabilityActionBlockReason, isCapabilitySyncBlocked } from "../capabilities/supportMatrix";
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
      text: "Formal source: override",
      tone: "warning",
      title: "Current compile uses the applied Formal QuantScript override."
    };
  }

  if (graph.metadata?.artifacts?.quantscript?.formal_source) {
    return {
      text: "Formal source: graph-generated",
      tone: "success",
      title: "Current compile uses the graph-generated Formal QuantScript source."
    };
  }

  return {
    text: "Formal source: unavailable",
    tone: "danger",
    title: "No Formal QuantScript source is currently available for compile."
  };
}

export function resolveWorkspaceActionState({
  graph,
  runtime,
  capabilityStatus,
  capabilitySource,
  capabilityMessage,
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
  const exportConfigCapabilityReason = getCapabilityActionBlockReason({
    actionKey: "export_runtime_config",
    capabilityStatus,
    capabilitySource,
    capabilityMessage
  });
  const compileCapabilityReason = getCapabilityActionBlockReason({
    actionKey: "compile",
    capabilityStatus,
    capabilitySource,
    capabilityMessage
  });
  const startSimulationCapabilityReason = getCapabilityActionBlockReason({
    actionKey: "start_simulation",
    capabilityStatus,
    capabilitySource,
    capabilityMessage
  });
  const runBacktestCapabilityReason = getCapabilityActionBlockReason({
    actionKey: "run_backtest",
    capabilityStatus,
    capabilitySource,
    capabilityMessage
  });
  const formalCompileSourceMeta = resolveFormalCompileSourceMeta({
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
    capabilitySyncBlocked,
    formalCompileSourceMeta,
    compileButtonTitle: compileButtonTitle || undefined,
    startSimulationTitle:
      startSimulationCapabilityReason ||
      (runtime.status === "running" || runtime.status === "connecting"
        ? t("运行时已经在执行中，不能重复启动。")
        : undefined),
    runBacktestTitle:
      runBacktestCapabilityReason ||
      (runtime.status === "running" || runtime.status === "connecting"
        ? t("运行时执行期间不能启动新的回测。")
        : undefined),
    exportConfigTitle: exportConfigCapabilityReason || undefined,
    canCompile: graph.validation_state.is_valid && !capabilitySyncBlocked,
    canStartRuntime:
      runtime.status !== "running" &&
      runtime.status !== "connecting" &&
      !capabilitySyncBlocked,
    canStartBacktest:
      runtime.status !== "running" &&
      runtime.status !== "connecting" &&
      !capabilitySyncBlocked,
    canStopRuntime: runtime.status === "running" || runtime.status === "connecting"
  };
}
