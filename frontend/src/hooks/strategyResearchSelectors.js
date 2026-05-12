import { useMemo } from "react";
import { useGraphStore } from "../store/graphStore";
import { buildRuntimeDiagnosticsProjection } from "../utils/runtimeDiagnosticsProjection";

function parseTimeInput(value) {
  if (!value) return null;
  const time = new Date(value).getTime();
  return Number.isFinite(time) ? time : null;
}

function resolveRunStatus(run, runtime) {
  if (runtime.runId === run.run_id && runtime.status && runtime.status !== "idle") {
    return runtime.status;
  }
  return run.status || "completed";
}

function backtestExecutionAssumptionsSearchText(filters) {
  return [
    filters?.execution_assumptions_tag?.label,
    filters?.execution_assumptions_tag?.sources_label
  ]
    .filter(Boolean)
    .join(" ")
    .toLowerCase();
}

function selectedRunId(runtime = {}) {
  return runtime.selectedHistoryRunId || (runtime.runKind === "simulation" ? runtime.runId : null);
}

function selectedBacktestId(runtime = {}) {
  return runtime.selectedBacktestId || (runtime.runKind === "backtest" ? runtime.runId : null);
}

function isSelectedRunRecord(run, runtime) {
  const runId = selectedRunId(runtime);
  return Boolean(runId && run?.run_id === runId);
}

function isSelectedBacktestRecord(item, runtime) {
  const backtestId = selectedBacktestId(runtime);
  return Boolean(backtestId && item?.backtest_id === backtestId);
}

export function filterRunHistoryRecords(history = [], runtime = {}, runFilters = {}) {
  const graphFilter = (runFilters.historyFilter || "").trim().toLowerCase();
  const compileFilter = (runFilters.historyCompileFilter || "").trim().toLowerCase();
  const statusFilter = runFilters.historyStatusFilter || "all";
  const sortOrder = runFilters.historySortOrder || "desc";
  const fromTime = parseTimeInput(runFilters.historyFromTime);
  const toTime = parseTimeInput(runFilters.historyToTime);

  const next = history.filter((run) => {
    if (isSelectedRunRecord(run, runtime)) return true;
    const matchesGraph = !graphFilter || (run.graph_id || "").toLowerCase().includes(graphFilter);
    const matchesCompile =
      !compileFilter || (run.compile_id || "").toLowerCase().includes(compileFilter);
    const effectiveStatus = resolveRunStatus(run, runtime);
    const matchesStatus = statusFilter === "all" ? true : effectiveStatus === statusFilter;
    const matchesFrom = fromTime === null ? true : run.created_at_ms >= fromTime;
    const matchesTo = toTime === null ? true : run.created_at_ms <= toTime;
    return matchesGraph && matchesCompile && matchesStatus && matchesFrom && matchesTo;
  });

  next.sort((left, right) => {
    const delta = left.created_at_ms - right.created_at_ms;
    return sortOrder === "asc" ? delta : -delta;
  });

  return next;
}

export function filterBacktestHistoryRecords(history = [], runtime = {}, backtestFilters = {}) {
  const graphFilter = (backtestFilters.backtestHistoryFilter || "").trim().toLowerCase();
  const compileFilter = (backtestFilters.backtestCompileFilter || "").trim().toLowerCase();
  const datasetFilter = (backtestFilters.backtestDatasetFilter || "").trim().toLowerCase();
  const parameterFilter = (backtestFilters.backtestParameterFilter || "").trim().toLowerCase();
  const fromTime = parseTimeInput(backtestFilters.backtestFromTime);
  const toTime = parseTimeInput(backtestFilters.backtestToTime);

  const next = history.filter((item) => {
    if (isSelectedBacktestRecord(item, runtime)) return true;
    const matchesGraph = !graphFilter || (item.graph_id || "").toLowerCase().includes(graphFilter);
    const matchesCompile =
      !compileFilter || (item.compile_id || "").toLowerCase().includes(compileFilter);
    const datasetText = (item.filters?.dataset_labels || []).join(" ").toLowerCase();
    const parameterText = backtestExecutionAssumptionsSearchText(item.filters);
    const startedAt = item.filters?.started_at_ms ?? item.created_at_ms;
    const endedAt = item.filters?.ended_at_ms ?? item.created_at_ms;
    const matchesDataset = !datasetFilter || datasetText.includes(datasetFilter);
    const matchesParameter = !parameterFilter || parameterText.includes(parameterFilter);
    const matchesFrom = fromTime === null ? true : endedAt >= fromTime;
    const matchesTo = toTime === null ? true : startedAt <= toTime;
    return (
      matchesGraph &&
      matchesCompile &&
      matchesDataset &&
      matchesParameter &&
      matchesFrom &&
      matchesTo
    );
  });

  next.sort((left, right) => right.created_at_ms - left.created_at_ms);
  return next;
}

function normalizeQualityFlags(value) {
  if (Array.isArray(value)) {
    return value.filter(Boolean).map((item) => String(item));
  }
  if (value === null || value === undefined || value === "") {
    return [];
  }
  return [String(value)];
}

function normalizeSourceHealth(value) {
  if (!value) return "";
  return String(value).trim().toLowerCase();
}

function formatSourceHealthLabel(value) {
  const health = normalizeSourceHealth(value);
  if (!health) return "未知";
  if (["healthy", "ok", "normal"].includes(health)) return "健康";
  if (["warning", "warn"].includes(health)) return "告警";
  if (health.includes("delay")) return "延迟";
  if (health.includes("stale")) return "过期";
  if (health.includes("missing")) return "缺失";
  if (health.includes("error")) return "错误";
  if (health.includes("degrad")) return "降级";
  return String(value);
}

function isDegradedDataMetrics(metrics = {}) {
  const health = normalizeSourceHealth(metrics.source_health || metrics.source_status);
  const qualityFlags = normalizeQualityFlags(metrics.quality_flags);
  const freshnessMs = Number(metrics.freshness_ms);
  const staleAfterMs = Number(metrics.stale_after_ms);
  const gapCount = Number(metrics.gap_count);

  return (
    qualityFlags.length > 0 ||
    (Number.isFinite(gapCount) && gapCount > 0) ||
    (Number.isFinite(freshnessMs) &&
      Number.isFinite(staleAfterMs) &&
      staleAfterMs > 0 &&
      freshnessMs > staleAfterMs) ||
    Boolean(health) &&
      !["healthy", "ok", "normal"].includes(health)
  );
}

function rowValueByKey(rows = [], key) {
  return rows.find((row) => row.key === key)?.value || null;
}

function buildResearchDataQualitySummary(graph, displayedEvents, diagnosticsProjection) {
  const dataNodes = (graph?.nodes || []).filter((node) => node.type === "data");
  const degradedNodes = dataNodes.filter((node) =>
    isDegradedDataMetrics(node.runtime_state?.metrics || {})
  );
  const qualityEvents = displayedEvents.filter(
    (event) =>
      event?.event_type === "RuntimeWarning" ||
      event?.event_type === "RuntimeError" ||
      (event?.event_type === "DataUpdated" && isDegradedDataMetrics(event?.payload || {}))
  );

  const rows = diagnosticsProjection?.dataQualityRows || [];
  const selectedNodeId = diagnosticsProjection?.selectedNodeId || null;
  const selectedNodeName =
    diagnosticsProjection?.activeNodes?.find((node) => node.nodeId === selectedNodeId)?.nodeName ||
    diagnosticsProjection?.selectedNode?.name ||
    selectedNodeId ||
    null;
  const sourceHealth = rowValueByKey(rows, "source_health");
  const freshnessMs = rowValueByKey(rows, "freshness_ms");
  const gapCount = rowValueByKey(rows, "gap_count");
  const qualityFlags = rowValueByKey(rows, "quality_flags");
  const hasSelectedRows = rows.length > 0;
  const degradedCount = degradedNodes.length;
  const totalCount = dataNodes.length;

  const noteParts = [];
  if (selectedNodeName) noteParts.push(selectedNodeName);
  if (sourceHealth) noteParts.push(`健康度 ${formatSourceHealthLabel(sourceHealth)}`);
  if (freshnessMs) noteParts.push(`新鲜度 ${freshnessMs}`);
  if (gapCount && gapCount !== "0") noteParts.push(`缺口 ${gapCount}`);
  if (qualityFlags) noteParts.push(`标记 ${qualityFlags}`);

  return {
    degradedNodeCount: degradedCount,
    totalDataNodeCount: totalCount,
    qualityEventCount: qualityEvents.length,
    sourceHealthLabel: formatSourceHealthLabel(sourceHealth),
    selectedNodeName,
    note:
      noteParts.join(" · ") ||
      (totalCount > 0
        ? `${degradedCount}/${totalCount} 个数据节点存在质量风险`
        : "当前图中还没有数据节点"),
    value:
      totalCount > 0 ? `${degradedCount}/${totalCount}` : String(qualityEvents.length),
    tone:
      degradedCount > 0 || qualityEvents.length > 0 || hasSelectedRows
        ? isDegradedDataMetrics({
            source_health: sourceHealth,
            freshness_ms: freshnessMs,
            gap_count: gapCount,
            quality_flags: qualityFlags
          }) || degradedCount > 0 || qualityEvents.length > 0
          ? "warning"
          : "info"
        : "muted"
  };
}

export function useStrategyResearchSelectors(uiState) {
  const graph = useGraphStore((state) => state.graph);
  const runtime = useGraphStore((state) => state.runtime);
  const selectedNodeId = useGraphStore((state) => state.selectedNodeId);
  const setSelectedNode = useGraphStore((state) => state.setSelectedNode);
  const runFilters = uiState?.runFilters || {};
  const backtestFilters = uiState?.backtestFilters || {};
  const eventFilters = uiState?.eventFilters || {};

  const filteredHistory = useMemo(() => {
    return filterRunHistoryRecords(runtime.history, runtime, runFilters);
  }, [runFilters, runtime]);

  const totalPages = Math.max(1, Math.ceil(filteredHistory.length / (runFilters.historyPageSize || 6)));
  const currentPage = Math.min(runFilters.historyPage || 1, totalPages);
  const pagedHistory = filteredHistory.slice(
    (currentPage - 1) * (runFilters.historyPageSize || 6),
    currentPage * (runFilters.historyPageSize || 6)
  );

  const filteredBacktests = useMemo(() => {
    return filterBacktestHistoryRecords(runtime.backtestHistory, runtime, backtestFilters);
  }, [
    backtestFilters.backtestDatasetFilter,
    backtestFilters.backtestCompileFilter,
    backtestFilters.backtestFromTime,
    runtime.backtestHistory,
    backtestFilters.backtestHistoryFilter,
    backtestFilters.backtestParameterFilter,
    backtestFilters.backtestToTime
  ]);

  const backtestTotalPages = Math.max(
    1,
    Math.ceil(filteredBacktests.length / (backtestFilters.backtestPageSize || 6))
  );
  const backtestCurrentPage = Math.min(backtestFilters.backtestPage || 1, backtestTotalPages);
  const pagedBacktests = filteredBacktests.slice(
    (backtestCurrentPage - 1) * (backtestFilters.backtestPageSize || 6),
    backtestCurrentPage * (backtestFilters.backtestPageSize || 6)
  );

  const displayedEvents =
    runtime.runKind === "backtest" && Array.isArray(runtime.backtestArtifacts?.event_log?.events)
      ? runtime.backtestArtifacts.event_log.events
      : runtime.events;

  const diagnosticsProjection = useMemo(
    () => buildRuntimeDiagnosticsProjection(graph, runtime, selectedNodeId),
    [graph, runtime, selectedNodeId]
  );

  const eventNodeOptions = useMemo(() => {
    if (diagnosticsProjection?.activeNodes?.length) {
      return diagnosticsProjection.activeNodes;
    }

    const nodes = Array.isArray(graph?.nodes) ? graph.nodes : [];
    const nodeMap = new Map(nodes.map((node) => [node.id, node]));
    const eventCountByNode = new Map();

    displayedEvents.forEach((event) => {
      if (!event?.node_id) return;
      eventCountByNode.set(event.node_id, (eventCountByNode.get(event.node_id) || 0) + 1);
    });

    return [...eventCountByNode.entries()].map(([nodeId, eventCount]) => {
      const node = nodeMap.get(nodeId);
      return {
        nodeId,
        nodeName: node?.name || nodeId,
        nodeType: node?.type || "node",
        status: node?.runtime_state?.status || "idle",
        latestEventLabel: null,
        latestEventTimeLabel: null,
        eventCount
      };
    });
  }, [diagnosticsProjection, displayedEvents, graph]);

  const selectedEventNodeId = useMemo(() => {
    if (eventFilters.eventNodeScope === "all") {
      return null;
    }
    const hasStructuredDefault = Boolean(runtime?.diagnostics?.default_selected_node_id);
    if (hasStructuredDefault) {
      return diagnosticsProjection?.selectedNodeId || null;
    }
    if (selectedNodeId && eventNodeOptions.some((node) => node.nodeId === selectedNodeId)) {
      return selectedNodeId;
    }
    return null;
  }, [
    diagnosticsProjection,
    eventFilters.eventNodeScope,
    eventNodeOptions,
    runtime?.diagnostics?.default_selected_node_id,
    selectedNodeId
  ]);

  const filteredEvents = useMemo(() => {
    const typeFilter = eventFilters.eventTypeFilter || "all";
    const keyword = (eventFilters.eventSearchTerm || "").trim().toLowerCase();
    return displayedEvents.filter((event) => {
      const matchesNode = !selectedEventNodeId ? true : event.node_id === selectedEventNodeId;
      const matchesType = typeFilter === "all" ? true : event.event_type === typeFilter;
      const matchesKeyword = !keyword
        ? true
        : [
            event.event_type,
            event.summary,
            event.node_id,
            event.source_id,
            JSON.stringify(event.payload || {})
          ]
            .filter(Boolean)
            .join(" ")
            .toLowerCase()
            .includes(keyword);
      return matchesNode && matchesType && matchesKeyword;
    });
  }, [displayedEvents, eventFilters.eventSearchTerm, eventFilters.eventTypeFilter, selectedEventNodeId]);

  const eventTypes = useMemo(
    () => ["all", ...new Set(displayedEvents.map((event) => event.event_type).filter(Boolean))],
    [displayedEvents]
  );

  const selectedBacktestSummary = runtime.selectedBacktestId
    ? runtime.backtestHistory.find((item) => item.backtest_id === runtime.selectedBacktestId) || null
    : null;
  const dataQualitySummary = useMemo(
    () => buildResearchDataQualitySummary(graph, displayedEvents, diagnosticsProjection),
    [diagnosticsProjection, displayedEvents, graph]
  );

  return {
    graph,
    runtime,
    openOrders: runtime.account?.open_orders || [],
    displayedEvents,
    diagnosticsProjection,
    eventTypes,
    eventNodeOptions,
    selectedEventNodeId,
    filteredEvents,
    filteredHistory,
    pagedHistory,
    currentPage,
    totalPages,
    filteredBacktests,
    pagedBacktests,
    backtestCurrentPage,
    backtestTotalPages,
    selectedBacktestSummary,
    dataQualitySummary,
    compareSelection: runtime.backtestCompareSelection?.[graph?.metadata?.graph_id] || (Array.isArray(runtime.backtestCompareSelection) ? runtime.backtestCompareSelection : []),
    backtestSummary: runtime.backtestArtifacts?.metrics?.summary || null,
    backtestStartedAt: runtime.backtestArtifacts?.metrics?.started_at_ms || null,
    backtestEndedAt: runtime.backtestArtifacts?.metrics?.ended_at_ms || null,
    setSelectedNode
  };
}
