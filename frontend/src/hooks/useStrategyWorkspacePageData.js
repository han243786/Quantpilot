import { useMemo } from "react";
import { resolveCanvasRecommendations } from "../components/strategyCanvasFocus";
import {
  workspaceIssueQueueCounts,
  workspaceIssueQueueSourceCounts,
  workspaceIssueQueueSourceOrder
} from "../utils/strategyWorkspaceIssueQueue";
import {
  buildWorkspaceBacktestPreviewItems,
  buildWorkspaceDiagnosticsStatusHighlights,
  buildWorkspaceOverviewMetrics,
  buildWorkspaceOverviewStatusHighlights,
  buildWorkspaceRunPreviewItems,
  countWorkspaceDiagnostics,
  formatWorkspaceTime,
  resolveWorkspaceCompareSelection,
  resolveWorkspaceReadiness,
  selectRecentWorkspaceActivity
} from "./strategyWorkspacePageDataProjection";

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
  const compileCounts = useMemo(
    () => countWorkspaceDiagnostics(compileDiagnostics),
    [compileDiagnostics]
  );
  const issueCount =
    (graph.validation_state?.issue_counts?.error || 0) +
    (graph.validation_state?.issue_counts?.warning || 0);
  const readiness = resolveWorkspaceReadiness({
    isRunnable: Boolean(graph.validation_state?.is_runnable),
    isCompilable: Boolean(compileSummary.compilable),
    issueCount
  });

  const recentRuns = useMemo(
    () => selectRecentWorkspaceActivity(runtime.history || [], currentGraphId),
    [currentGraphId, runtime.history]
  );
  const recentBacktests = useMemo(
    () => selectRecentWorkspaceActivity(runtime.backtestHistory || [], currentGraphId),
    [currentGraphId, runtime.backtestHistory]
  );
  const lastRun = recentRuns[0] || null;
  const lastBacktest = recentBacktests[0] || null;
  const compareSelection = resolveWorkspaceCompareSelection(runtime, graph?.metadata?.graph_id);
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

  const overviewMetrics = buildWorkspaceOverviewMetrics({
    graph,
    readiness,
    compileSummary,
    compileCounts,
    recentRuns,
    recentBacktests
  });

  const runPreviewItems = buildWorkspaceRunPreviewItems(recentRuns);

  const overviewStatusHighlights = buildWorkspaceOverviewStatusHighlights({
    graph,
    compileSummary,
    lastRun,
    lastBacktest
  });

  const backtestPreviewItems = buildWorkspaceBacktestPreviewItems(recentBacktests);

  const diagnosticsStatusHighlights = buildWorkspaceDiagnosticsStatusHighlights({
    issueQueueCountsSummary,
    compileCounts,
    issueQueueSources
  });

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
    formatTime: formatWorkspaceTime
  };
}
