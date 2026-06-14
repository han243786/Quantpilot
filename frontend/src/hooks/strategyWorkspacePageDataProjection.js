import { diagnosticQueueSource } from "../utils/strategyWorkspaceIssueQueue";

export function formatWorkspaceTime(value) {
  return value ? new Date(value).toLocaleString() : "-";
}

export function formatWorkspaceCount(value) {
  if (!Number.isFinite(value)) return "0";
  return new Intl.NumberFormat().format(value);
}

export function formatWorkspacePercent(value) {
  if (!Number.isFinite(value)) return "-";
  const sign = value > 0 ? "+" : "";
  return `${sign}${(value * 100).toFixed(2)}%`;
}

export function compileWorkspaceOutputsText(outputs) {
  if (!outputs) return "-";
  return [
    `${outputs.data_sources || 0} data`,
    `${outputs.intent_generators || 0} intent`,
    `${outputs.agents || 0} agent`,
    `${outputs.risk_controls || 0} risk`,
    `${outputs.executions || 0} execution`
  ].join(" / ");
}

export function countWorkspaceDiagnostics(diagnostics = []) {
  return diagnostics.reduce(
    (summary, diagnostic) => {
      if (diagnostic?.severity === "warning") {
        summary.warning += 1;
      } else if (diagnostic?.severity === "info") {
        summary.info += 1;
      } else {
        summary.error += 1;
      }
      return summary;
    },
    { error: 0, warning: 0, info: 0 }
  );
}

export function resolveWorkspaceReadiness({ isRunnable, isCompilable, issueCount }) {
  if (issueCount > 0) {
    return { tone: "danger", label: "Blocked" };
  }
  if (isRunnable) {
    return { tone: "success", label: "Runnable" };
  }
  if (isCompilable) {
    return { tone: "warning", label: "Compilable" };
  }
  return { tone: "muted", label: "Needs work" };
}

export function selectRecentWorkspaceActivity(items = [], currentGraphId, limit = 4) {
  return [...items]
    .filter((item) => item.graph_id === currentGraphId)
    .sort((left, right) => (right.created_at_ms || 0) - (left.created_at_ms || 0))
    .slice(0, limit);
}

export function resolveWorkspaceCompareSelection(runtime, graphId) {
  const selection = runtime?.backtestCompareSelection;
  return selection?.[graphId] || (Array.isArray(selection) ? selection : []);
}

export function buildWorkspaceOverviewMetrics({
  graph,
  readiness,
  compileSummary,
  compileCounts,
  recentRuns,
  recentBacktests
}) {
  return [
    {
      label: "Readiness",
      value: readiness.label,
      note: `${graph.nodes.length} nodes / ${graph.edges.length} edges`,
      tone: readiness.tone
    },
    {
      label: "Compile outputs",
      value: compileWorkspaceOutputsText(compileSummary.outputs),
      note: compileSummary.protocol_name || "Protocol pending",
      tone: compileSummary.compilable ? "success" : "warning"
    },
    {
      label: "Diagnostics",
      value: `${compileCounts.error} / ${compileCounts.warning} / ${compileCounts.info}`,
      note: "error / warning / info",
      tone: compileCounts.error > 0 ? "danger" : compileCounts.warning > 0 ? "warning" : "muted"
    },
    {
      label: "Runs and backtests",
      value: `${formatWorkspaceCount(recentRuns.length)} runs / ${formatWorkspaceCount(recentBacktests.length)} backtests`,
      note: "Keep the latest activity visible without leaving the workspace.",
      tone: recentBacktests.length > 0 || recentRuns.length > 0 ? "info" : "muted"
    }
  ];
}

export function buildWorkspaceRunPreviewItems(recentRuns) {
  return recentRuns.map((item) => ({
    id: item.run_id,
    title: item.run_id,
    meta: `${formatWorkspaceTime(item.created_at_ms)} | ${item.compile_id || "No compile ID recorded"}`,
    raw: item
  }));
}

export function buildWorkspaceBacktestPreviewItems(recentBacktests) {
  return recentBacktests.map((item) => ({
    id: item.backtest_id,
    title: item.backtest_id,
    meta: `${formatWorkspaceTime(item.created_at_ms)} | total return ${formatWorkspacePercent(item.summary?.total_return_ratio)}`,
    raw: item
  }));
}

export function buildWorkspaceOverviewStatusHighlights({
  graph,
  compileSummary,
  lastRun,
  lastBacktest
}) {
  return [
    {
      label: "Latest compile ID",
      value: graph.metadata?.runtime_binding?.last_compile_id || "-",
      note: compileSummary.config_hash || "No config hash recorded"
    },
    {
      label: "Latest run",
      value: lastRun ? formatWorkspaceTime(lastRun.created_at_ms) : "-",
      note: lastRun?.compile_id || "No run-linked compile recorded"
    },
    {
      label: "Latest backtest",
      value: lastBacktest ? formatWorkspaceTime(lastBacktest.created_at_ms) : "-",
      note: lastBacktest?.backtest_id || "No backtest recorded"
    }
  ];
}

export function buildWorkspaceDiagnosticsStatusHighlights({
  issueQueueCountsSummary,
  compileCounts,
  issueQueueSources
}) {
  return [
    {
      label: "Actionable fixes",
      value: formatWorkspaceCount(issueQueueCountsSummary.actionable),
      note:
        issueQueueCountsSummary.actionable > 0
          ? "Jump directly from the queue to the repair surface."
          : "No actionable node-level repair item right now."
    },
    {
      label: "Compile diagnostics",
      value: `${compileCounts.error} / ${compileCounts.warning} / ${compileCounts.info}`,
      note: "error / warning / info"
    },
    {
      label: "Source lanes",
      value: formatWorkspaceCount(issueQueueSources.length),
      note:
        issueQueueSources.length > 0
          ? issueQueueSources.map((source) => diagnosticQueueSource({ source })).join(" / ")
          : "No issue source is active yet."
    }
  ];
}
