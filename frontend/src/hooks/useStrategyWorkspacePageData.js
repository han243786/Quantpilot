import { useMemo } from "react";
import { resolveCanvasRecommendations } from "../components/strategyCanvasFocus";
import {
  diagnosticQueueSource,
  workspaceIssueQueueCounts,
  workspaceIssueQueueSourceCounts,
  workspaceIssueQueueSourceOrder
} from "../utils/strategyWorkspaceIssueQueue";

function formatTime(value) {
  return value ? new Date(value).toLocaleString() : "-";
}

function formatCount(value) {
  if (!Number.isFinite(value)) return "0";
  return new Intl.NumberFormat().format(value);
}

function formatPercent(value) {
  if (!Number.isFinite(value)) return "-";
  const sign = value > 0 ? "+" : "";
  return `${sign}${(value * 100).toFixed(2)}%`;
}

function compileOutputsText(outputs) {
  if (!outputs) return "-";
  return [
    `${outputs.data_sources || 0} data`,
    `${outputs.intent_generators || 0} intent`,
    `${outputs.agents || 0} agent`,
    `${outputs.risk_controls || 0} risk`,
    `${outputs.executions || 0} execution`
  ].join(" / ");
}

function diagnosticCounts(diagnostics = []) {
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

function readinessTone({ isRunnable, isCompilable, issueCount }) {
  if (issueCount > 0) return "danger";
  if (isRunnable) return "success";
  if (isCompilable) return "warning";
  return "muted";
}

function readinessLabel({ isRunnable, isCompilable, issueCount }) {
  if (issueCount > 0) return "Blocked";
  if (isRunnable) return "Runnable";
  if (isCompilable) return "Compilable";
  return "Needs work";
}

export function useStrategyWorkspacePageData({
  graph,
  runtime,
  strategyId,
  selectedNodeId,
  selectedEdgeId,
  issueQueue,
  activeTab,
  canvasWorkspaceLaneId,
  codeInspectorPanels,
  activeCodeInspector
}) {
  const currentGraphId = graph.metadata?.graph_id || strategyId || "draft_graph";
  const compileSummary = graph.compile_summary || {};
  const compileDiagnostics = Array.isArray(compileSummary.diagnostics)
    ? compileSummary.diagnostics
    : [];
  const compileCounts = useMemo(() => diagnosticCounts(compileDiagnostics), [compileDiagnostics]);
  const issueCount =
    (graph.validation_state?.issue_counts?.error || 0) +
    (graph.validation_state?.issue_counts?.warning || 0);
  const readiness = {
    tone: readinessTone({
      isRunnable: Boolean(graph.validation_state?.is_runnable),
      isCompilable: Boolean(compileSummary.compilable),
      issueCount
    }),
    label: readinessLabel({
      isRunnable: Boolean(graph.validation_state?.is_runnable),
      isCompilable: Boolean(compileSummary.compilable),
      issueCount
    })
  };

  const recentRuns = useMemo(
    () =>
      [...(runtime.history || [])]
        .filter((item) => item.graph_id === currentGraphId)
        .sort((left, right) => (right.created_at_ms || 0) - (left.created_at_ms || 0))
        .slice(0, 4),
    [currentGraphId, runtime.history]
  );
  const recentBacktests = useMemo(
    () =>
      [...(runtime.backtestHistory || [])]
        .filter((item) => item.graph_id === currentGraphId)
        .sort((left, right) => (right.created_at_ms || 0) - (left.created_at_ms || 0))
        .slice(0, 4),
    [currentGraphId, runtime.backtestHistory]
  );
  const lastRun = recentRuns[0] || null;
  const lastBacktest = recentBacktests[0] || null;
  const compareSelection = runtime.backtestCompareSelection || [];
  const issueQueueCountsSummary = useMemo(() => workspaceIssueQueueCounts(issueQueue), [issueQueue]);
  const issueQueueSources = useMemo(() => workspaceIssueQueueSourceOrder(issueQueue), [issueQueue]);
  const issueQueueSourceCountsSummary = useMemo(
    () => workspaceIssueQueueSourceCounts(issueQueue),
    [issueQueue]
  );
  const selectedEdge = useMemo(
    () => graph.edges.find((edge) => edge.id === selectedEdgeId) || null,
    [graph.edges, selectedEdgeId]
  );
  const recommendationLaneId = activeTab === "diagnostics" ? "diagnostics" : canvasWorkspaceLaneId;
  const configureRepairAnchorId = selectedNodeId || selectedEdge?.source_node_id || null;
  const canvasRecommendationState = useMemo(
    () => resolveCanvasRecommendations(graph, selectedNodeId, recommendationLaneId),
    [graph, recommendationLaneId, selectedNodeId]
  );
  const configureRepairPathState = useMemo(
    () => resolveCanvasRecommendations(graph, configureRepairAnchorId, "diagnostics"),
    [configureRepairAnchorId, graph]
  );

  const overviewMetrics = [
    {
      label: "Readiness",
      value: readiness.label,
      note: `${graph.nodes.length} nodes / ${graph.edges.length} edges`,
      tone: readiness.tone
    },
    {
      label: "Compile outputs",
      value: compileOutputsText(compileSummary.outputs),
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
      value: `${formatCount(recentRuns.length)} runs / ${formatCount(recentBacktests.length)} backtests`,
      note: "Keep the latest activity visible without leaving the workspace.",
      tone: recentBacktests.length > 0 || recentRuns.length > 0 ? "info" : "muted"
    }
  ];

  const runPreviewItems = recentRuns.map((item) => ({
    id: item.run_id,
    title: item.run_id,
    meta: `${formatTime(item.created_at_ms)} | ${item.compile_id || "No compile ID recorded"}`,
    raw: item
  }));

  const overviewStatusHighlights = [
    {
      label: "Latest compile ID",
      value: graph.metadata?.runtime_binding?.last_compile_id || "-",
      note: compileSummary.config_hash || "No config hash recorded"
    },
    {
      label: "Latest run",
      value: lastRun ? formatTime(lastRun.created_at_ms) : "-",
      note: lastRun?.compile_id || "No run-linked compile recorded"
    },
    {
      label: "Latest backtest",
      value: lastBacktest ? formatTime(lastBacktest.created_at_ms) : "-",
      note: lastBacktest?.backtest_id || "No backtest recorded"
    }
  ];

  const backtestPreviewItems = recentBacktests.map((item) => ({
    id: item.backtest_id,
    title: item.backtest_id,
    meta: `${formatTime(item.created_at_ms)} | total return ${formatPercent(item.summary?.total_return_ratio)}`,
    raw: item
  }));

  const diagnosticsStatusHighlights = [
    {
      label: "Actionable fixes",
      value: formatCount(issueQueueCountsSummary.actionable),
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
      value: formatCount(issueQueueSources.length),
      note:
        issueQueueSources.length > 0
          ? issueQueueSources.map((source) => diagnosticQueueSource({ source })).join(" / ")
          : "No issue source is active yet."
    }
  ];

  const activeInspectorDefinition =
    codeInspectorPanels.find((panel) => panel.id === activeCodeInspector) ||
    codeInspectorPanels[0];
  const secondaryInspectorDefinitions = codeInspectorPanels.filter(
    (panel) => panel.id !== activeInspectorDefinition.id
  );

  return {
    currentGraphId,
    compileSummary,
    compileCounts,
    readiness,
    recentRuns,
    recentBacktests,
    lastRun,
    lastBacktest,
    compareSelection,
    issueQueueCounts: issueQueueCountsSummary,
    issueQueueSources,
    issueQueueSourceCounts: issueQueueSourceCountsSummary,
    canvasRecommendationState,
    configureRepairPathState,
    overviewMetrics,
    runPreviewItems,
    overviewStatusHighlights,
    backtestPreviewItems,
    diagnosticsStatusHighlights,
    activeInspectorDefinition,
    secondaryInspectorDefinitions,
    formatTime
  };
}
