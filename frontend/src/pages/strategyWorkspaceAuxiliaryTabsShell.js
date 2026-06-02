import { getRuntimeStatusMeta } from "../utils/runtimeStatus";

export function formatWorkspaceMonitorNumber(value, digits = 2) {
  if (!Number.isFinite(Number(value))) return "-";
  return Number(value).toFixed(digits);
}

export function formatWorkspaceMonitorCount(value) {
  if (!Number.isFinite(Number(value))) return "0";
  return new Intl.NumberFormat().format(Number(value));
}

export function resolveWorkspaceRuntimeKindLabel(kind, t = (value) => value) {
  if (kind === "backtest") return t("回测");
  if (kind === "simulation") return t("模拟");
  if (kind === "live") return t("实盘");
  return t("未运行");
}

export function selectWorkspaceRuntimeEvents(runtime = {}) {
  const timeline = Array.isArray(runtime.timeline) ? runtime.timeline : [];
  const events = Array.isArray(runtime.events) ? runtime.events : [];
  return [...(timeline.length > 0 ? timeline : events)];
}

export function buildWorkspaceMonitorModel({
  graph = { nodes: [] },
  runtime = {},
  recentRuns = [],
  issueQueue = [],
  formatTime = (value) => value ?? "-",
  t = (value) => value
}) {
  const statusMeta = getRuntimeStatusMeta(runtime.status);
  const account = runtime.account || {};
  const openOrders = Array.isArray(account.open_orders) ? account.open_orders : [];
  const allEvents = selectWorkspaceRuntimeEvents(runtime);
  const recentEvents = allEvents.slice(-5).reverse();
  const riskIssueCount = issueQueue.filter((item) => item.nodeType === "risk").length;
  const executionNodes = (graph.nodes || []).filter((node) => node.type === "execution").length;
  const latestRun = recentRuns[0] || null;
  const runKind = resolveWorkspaceRuntimeKindLabel(runtime.runKind, t);

  return {
    statusMeta,
    runKind,
    openOrders,
    allEvents,
    recentEvents,
    stripPills: [
      { label: statusMeta.label, tone: statusMeta.tone },
      { label: runKind, tone: "muted" },
      { label: `${formatWorkspaceMonitorCount(openOrders.length)} ${t("挂单")}`, tone: "info" }
    ],
    runtimeMetrics: [
      { label: t("状态"), value: statusMeta.label, tone: statusMeta.tone },
      { label: t("运行 ID"), value: runtime.runId || "-" },
      { label: t("类型"), value: runKind },
      { label: t("最近运行"), value: latestRun ? formatTime(latestRun.created_at_ms) : "-" }
    ],
    accountMetrics: [
      { label: t("净值估算"), value: formatWorkspaceMonitorNumber(account.equity_estimate), tone: "success" },
      { label: t("可用现金"), value: formatWorkspaceMonitorNumber(account.available_cash_balance) },
      { label: t("冻结现金"), value: formatWorkspaceMonitorNumber(account.frozen_cash_balance) },
      {
        label: t("挂单"),
        value: formatWorkspaceMonitorCount(account.open_order_count ?? openOrders.length)
      }
    ],
    riskMetrics: [
      {
        label: t("风险阻塞"),
        value: formatWorkspaceMonitorCount(riskIssueCount),
        tone: riskIssueCount > 0 ? "danger" : "success"
      },
      { label: t("执行节点"), value: formatWorkspaceMonitorCount(executionNodes) },
      { label: t("诊断"), value: runtime.diagnostics ? t("已连接") : "-" },
      { label: t("事件数"), value: formatWorkspaceMonitorCount(allEvents.length) }
    ]
  };
}

export function buildWorkspaceResearchStripModel(t = (value) => value) {
  return {
    title: t("研究回测工作区"),
    pills: [
      { label: t("结果"), tone: "muted" },
      { label: t("时间线"), tone: "info" },
      { label: t("详情"), tone: "muted" }
    ]
  };
}

export function buildSourceScenarioRunRequest(source) {
  return {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ source })
  };
}

export function buildSourceScenarioHttpError(status, text) {
  return { error: `HTTP ${status}: ${String(text).slice(0, 300)}` };
}

export function buildSourceScenarioStepPresentation(status) {
  if (status === "passed") {
    return { icon: "✓", color: "var(--ad-success)" };
  }
  if (status === "failed") {
    return { icon: "✗", color: "var(--ad-error)" };
  }
  return { icon: "⊘", color: "var(--ad-text-muted)" };
}

export function extractSourceScenarioActualValue(message) {
  if (typeof message !== "string" || !message.includes("actual:")) return null;
  return message.match(/actual:\s*([^)]+)/)?.[1] || "?";
}
