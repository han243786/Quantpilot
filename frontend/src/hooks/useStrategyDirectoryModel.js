import { useDeferredValue, useEffect, useMemo, useState } from "react";
import { useGraphStore } from "../store/graphStore";
import { STRATEGY_TEMPLATE_LIBRARY } from "../templates/strategyTemplates";
import { navigateTo, strategyWorkspacePath } from "../router";

const MAX_RECENT_ITEMS = 4;
const MAX_ACTIVITY_ITEMS = 6;

function pushRecent(list, item, limit = MAX_RECENT_ITEMS) {
  if (!item) return list;
  list.push(item);
  list.sort((left, right) => (right.created_at_ms || 0) - (left.created_at_ms || 0));
  return list.slice(0, limit);
}

function resolveHealth(entry) {
  if (entry.isCurrent) {
    if ((entry.issueCount || 0) > 0) {
      return { tone: "danger", label: "待修复" };
    }
    if (entry.isRunnable) {
      return { tone: "success", label: "可运行" };
    }
    if (entry.isCompilable) {
      return { tone: "warning", label: "可编译" };
    }
  }

  if ((entry.backtestCount || 0) > 0 || (entry.runCount || 0) > 0) {
    return { tone: "info", label: "已跟踪" };
  }

  return { tone: "muted", label: "草稿" };
}

function resolveActivityLabel(entry) {
  if (!entry.lastActivityAt) return "暂无活动";
  if (entry.lastBacktestAt && entry.lastBacktestAt === entry.lastActivityAt) {
    return "回测";
  }
  if (entry.lastRunAt && entry.lastRunAt === entry.lastActivityAt) {
    return "模拟";
  }
  return "已更新";
}

function buildAvailableGraphIds(graph, graphIndex) {
  return new Set((Array.isArray(graphIndex) ? graphIndex : []).map((entry) => entry.graph_id).filter(Boolean));
}

function filterVisibleRuns(runtime, availableGraphIds, currentGraphId) {
  return (runtime.history || []).filter((run) =>
    availableGraphIds.has(run.graph_id || currentGraphId)
  );
}

function filterVisibleBacktests(runtime, availableGraphIds, currentGraphId) {
  return (runtime.backtestHistory || []).filter((backtest) =>
    availableGraphIds.has(backtest.graph_id || currentGraphId)
  );
}

function buildStrategyDirectory(graph, visibleRuns, visibleBacktests, graphIndex) {
  const map = new Map();
  const currentGraphId = graph.metadata?.graph_id || "draft_graph";
  const existingGraphs = Array.isArray(graphIndex) ? graphIndex : [];
  const currentGraphExists = existingGraphs.some((item) => item.graph_id === currentGraphId);

  function ensure(graphId, seed = {}) {
    const resolvedId = graphId || currentGraphId;
    if (!map.has(resolvedId)) {
      map.set(resolvedId, {
        graphId: resolvedId,
        name: resolvedId,
        filePath: "",
        isCurrent: false,
        isRunnable: false,
        isCompilable: false,
        issueCount: 0,
        protocolName: "",
        lastCompileId: "",
        lastConfigHash: "",
        datasetLabels: [],
        latestReturnRatio: null,
        runCount: 0,
        backtestCount: 0,
        lastRunAt: null,
        lastBacktestAt: null,
        lastActivityAt: null,
        recentRuns: [],
        recentBacktests: [],
        ...seed
      });
    }

    return map.get(resolvedId);
  }

  for (const item of existingGraphs) {
    ensure(item.graph_id, {
      graphId: item.graph_id,
      name: item.name || item.graph_id,
      filePath: item.path || "",
      lastActivityAt: item.updated_at || null
    });
  }

  if (currentGraphExists) {
    const currentEntry = ensure(currentGraphId, {
      graphId: currentGraphId,
      name: graph.metadata?.name || currentGraphId,
      filePath: existingGraphs.find((item) => item.graph_id === currentGraphId)?.path || "",
      isCurrent: true,
      isRunnable: Boolean(graph.validation_state?.is_runnable),
      isCompilable: Boolean(graph.compile_summary?.compilable),
      issueCount:
        (graph.validation_state?.issue_counts?.error || 0) +
        (graph.validation_state?.issue_counts?.warning || 0),
      protocolName: graph.compile_summary?.protocol_name || "",
      lastCompileId: graph.metadata?.runtime_binding?.last_compile_id || "",
      lastConfigHash: graph.compile_summary?.config_hash || ""
    });
    currentEntry.lastActivityAt = graph.metadata?.updated_at || currentEntry.lastActivityAt;
  }

  for (const run of visibleRuns) {
    const entry = ensure(run.graph_id, {
      name: run.graph_id || currentGraphId
    });
    entry.runCount += 1;
    entry.lastRunAt = Math.max(entry.lastRunAt || 0, run.created_at_ms || 0) || entry.lastRunAt;
    entry.lastActivityAt =
      Math.max(entry.lastActivityAt || 0, run.created_at_ms || 0) || entry.lastActivityAt;
    entry.lastCompileId = entry.lastCompileId || run.compile_id || "";
    entry.recentRuns = pushRecent(entry.recentRuns, run);
  }

  for (const backtest of visibleBacktests) {
    const entry = ensure(backtest.graph_id, {
      name: backtest.graph_id || currentGraphId
    });
    entry.backtestCount += 1;
    entry.lastBacktestAt =
      Math.max(entry.lastBacktestAt || 0, backtest.created_at_ms || 0) || entry.lastBacktestAt;
    entry.lastActivityAt =
      Math.max(entry.lastActivityAt || 0, backtest.created_at_ms || 0) || entry.lastActivityAt;
    entry.lastCompileId = entry.lastCompileId || backtest.compile_id || "";
    entry.lastConfigHash = entry.lastConfigHash || backtest.config_hash || "";
    entry.protocolName = entry.protocolName || backtest.protocol_name || "";
    entry.latestReturnRatio =
      Number.isFinite(backtest.summary?.total_return_ratio) && entry.latestReturnRatio === null
        ? backtest.summary.total_return_ratio
        : entry.latestReturnRatio;
    entry.datasetLabels =
      entry.datasetLabels.length > 0
        ? entry.datasetLabels
        : (backtest.filters?.dataset_labels || []).slice(0, 3);
    entry.recentBacktests = pushRecent(entry.recentBacktests, backtest);
  }

  return [...map.values()]
    .map((entry) => ({
      ...entry,
      datasetLabels: [...new Set(entry.datasetLabels)].slice(0, 3),
      health: resolveHealth(entry),
      activityLabel: resolveActivityLabel(entry)
    }))
    .sort((left, right) => {
      if (left.isCurrent !== right.isCurrent) {
        return left.isCurrent ? -1 : 1;
      }
      return (right.lastActivityAt || 0) - (left.lastActivityAt || 0);
    });
}

function buildActivityTimeline(visibleRuns, visibleBacktests) {
  const events = [
    ...visibleBacktests.map((item) => ({
      kind: "backtest",
      id: item.backtest_id,
      graphId: item.graph_id,
      createdAt: item.created_at_ms || 0,
      title: item.backtest_id,
      note:
        Number.isFinite(item.summary?.total_return_ratio)
          ? `${item.summary.total_return_ratio > 0 ? "+" : ""}${(
              item.summary.total_return_ratio * 100
            ).toFixed(2)}%`
          : "-",
      detail: item.protocol_name || item.compile_id || "回测记录"
    })),
    ...visibleRuns.map((item) => ({
      kind: "run",
      id: item.run_id,
      graphId: item.graph_id,
      createdAt: item.created_at_ms || 0,
      title: item.run_id,
      note: item.compile_id || "无编译 ID",
      detail: "模拟运行"
    }))
  ];

  return events
    .sort((left, right) => right.createdAt - left.createdAt)
    .slice(0, MAX_ACTIVITY_ITEMS);
}

function buildHubSummary(strategies, activityTimeline, compareSelection) {
  const withIssues = strategies.filter((entry) => entry.health.tone === "danger");
  const runnable = strategies.filter((entry) => entry.health.tone === "success");
  const researchReady = strategies.filter((entry) => entry.backtestCount > 0);

  return {
    trackedCount: strategies.length,
    issueCount: withIssues.length,
    runnableCount: runnable.length,
    researchReadyCount: researchReady.length,
    compareCount: (compareSelection || []).length,
    latestActivityAt: Math.max(
      0,
      ...strategies.map((entry) => entry.lastActivityAt || 0),
      ...(activityTimeline || []).map((item) => item.createdAt || 0)
    )
  };
}

export function useStrategyDirectoryModel() {
  const graph = useGraphStore((state) => state.graph);
  const graphIndex = useGraphStore((state) => state.graphIndex);
  const graphIndexStatus = useGraphStore((state) => state.graphIndexStatus);
  const runtime = useGraphStore((state) => state.runtime);
  const revealGraphFile = useGraphStore((state) => state.revealGraphFile);
  const deleteGraph = useGraphStore((state) => state.deleteGraph);
  const replaceBacktestCompareSelection = useGraphStore(
    (state) => state.replaceBacktestCompareSelection
  );
  const refreshGraphIndex = useGraphStore((state) => state.refreshGraphIndex);
  const refreshRunHistory = useGraphStore((state) => state.refreshRunHistory);
  const refreshBacktestHistory = useGraphStore((state) => state.refreshBacktestHistory);
  const loadLatestGraph = useGraphStore((state) => state.loadLatestGraph);
  const resetGraph = useGraphStore((state) => state.resetGraph);
  const loadStrategyTemplate = useGraphStore((state) => state.loadStrategyTemplate);
  const toggleBacktestCompareSelection = useGraphStore(
    (state) => state.toggleBacktestCompareSelection
  );
  const clearBacktestCompareSelection = useGraphStore(
    (state) => state.clearBacktestCompareSelection
  );
  const storedCompareSelection = runtime.backtestCompareSelection || [];

  const [query, setQuery] = useState("");
  const [scopeFilter, setScopeFilter] = useState("all");
  const [healthFilter, setHealthFilter] = useState("all");
  const [sortMode, setSortMode] = useState("activity");
  const [selectedStrategyId, setSelectedStrategyId] = useState("");
  const [selectedStrategyIds, setSelectedStrategyIds] = useState([]);
  const deferredQuery = useDeferredValue(query);

  useEffect(() => {
    if (graphIndexStatus === "idle") {
      void refreshGraphIndex();
    }
    if (runtime.historyStatus === "idle") {
      void refreshRunHistory();
    }
    if (runtime.backtestHistoryStatus === "idle") {
      void refreshBacktestHistory();
    }
  }, [
    graphIndexStatus,
    refreshGraphIndex,
    refreshBacktestHistory,
    refreshRunHistory,
    runtime.backtestHistoryStatus,
    runtime.historyStatus
  ]);

  const availableGraphIds = useMemo(() => buildAvailableGraphIds(graph, graphIndex), [graph, graphIndex]);
  const visibleRuns = useMemo(
    () => filterVisibleRuns(runtime, availableGraphIds, graph.metadata?.graph_id || "draft_graph"),
    [availableGraphIds, graph.metadata?.graph_id, runtime]
  );
  const visibleBacktests = useMemo(
    () =>
      filterVisibleBacktests(runtime, availableGraphIds, graph.metadata?.graph_id || "draft_graph"),
    [availableGraphIds, graph.metadata?.graph_id, runtime]
  );
  const validBacktestIds = useMemo(
    () => new Set(visibleBacktests.map((item) => item.backtest_id).filter(Boolean)),
    [visibleBacktests]
  );
  const compareSelection = useMemo(
    () => storedCompareSelection.filter((backtestId) => validBacktestIds.has(backtestId)),
    [storedCompareSelection, validBacktestIds]
  );
  const strategies = useMemo(
    () => buildStrategyDirectory(graph, visibleRuns, visibleBacktests, graphIndex),
    [graph, graphIndex, visibleBacktests, visibleRuns]
  );
  const activityTimeline = useMemo(
    () => buildActivityTimeline(visibleRuns, visibleBacktests),
    [visibleBacktests, visibleRuns]
  );
  const hubSummary = useMemo(
    () => buildHubSummary(strategies, activityTimeline, compareSelection),
    [activityTimeline, compareSelection, strategies]
  );

  useEffect(() => {
    if (compareSelection.length !== storedCompareSelection.length) {
      replaceBacktestCompareSelection(compareSelection);
    }
  }, [compareSelection, replaceBacktestCompareSelection, storedCompareSelection.length]);

  const filteredStrategies = useMemo(() => {
    const keyword = deferredQuery.trim().toLowerCase();
    const filtered = strategies.filter((entry) => {
      const matchesKeyword =
        !keyword ||
        [entry.name, entry.graphId, entry.lastCompileId, ...entry.datasetLabels]
          .filter(Boolean)
          .join(" ")
          .toLowerCase()
          .includes(keyword);

      const matchesScope =
        scopeFilter === "all"
          ? true
          : scopeFilter === "current"
            ? entry.isCurrent
            : scopeFilter === "active"
              ? entry.runCount > 0
              : entry.backtestCount > 0;

      const matchesHealth =
        healthFilter === "all"
          ? true
          : healthFilter === "runnable"
            ? entry.health.tone === "success"
            : healthFilter === "issues"
              ? entry.health.tone === "danger"
              : entry.health.tone === "info";

      return matchesKeyword && matchesScope && matchesHealth;
    });

    return filtered.sort((left, right) => {
      if (sortMode === "health") {
        const healthRank = { danger: 0, warning: 1, success: 2, info: 3, muted: 4 };
        return (
          (healthRank[left.health.tone] ?? 99) - (healthRank[right.health.tone] ?? 99) ||
          (right.lastActivityAt || 0) - (left.lastActivityAt || 0)
        );
      }

      if (sortMode === "return") {
        return (right.latestReturnRatio ?? -Infinity) - (left.latestReturnRatio ?? -Infinity);
      }

      if (sortMode === "research") {
        return (
          right.backtestCount - left.backtestCount ||
          right.runCount - left.runCount ||
          (right.lastActivityAt || 0) - (left.lastActivityAt || 0)
        );
      }

      return (right.lastActivityAt || 0) - (left.lastActivityAt || 0);
    });
  }, [deferredQuery, healthFilter, scopeFilter, sortMode, strategies]);

  useEffect(() => {
    if (filteredStrategies.length === 0) {
      setSelectedStrategyId("");
      return;
    }
    if (!filteredStrategies.some((entry) => entry.graphId === selectedStrategyId)) {
      setSelectedStrategyId(filteredStrategies[0].graphId);
    }
  }, [filteredStrategies, selectedStrategyId]);

  useEffect(() => {
    setSelectedStrategyIds((current) =>
      current.filter((graphId) => filteredStrategies.some((entry) => entry.graphId === graphId))
    );
  }, [filteredStrategies]);

  const selectedStrategy =
    filteredStrategies.find((entry) => entry.graphId === selectedStrategyId) ||
    filteredStrategies[0] ||
    null;

  const selectedStrategyCount = selectedStrategyIds.length;
  const selectedForWorkspace =
    selectedStrategyCount === 1 ? selectedStrategyIds[0] : selectedStrategy?.graphId || "";

  function toggleStrategySelection(graphId) {
    setSelectedStrategyIds((current) =>
      current.includes(graphId)
        ? current.filter((item) => item !== graphId)
        : [...current, graphId]
    );
  }

  async function applyTemplate(templateId) {
    const graph = await loadStrategyTemplate(templateId);
    navigateTo(strategyWorkspacePath(graph.metadata?.graph_id || "draft_graph"));
    return graph;
  }

  function openBlankWorkspace() {
    const graph = resetGraph();
    navigateTo(strategyWorkspacePath(graph.metadata?.graph_id || "draft_graph"));
    return graph;
  }

  async function deleteStrategy(graphId, strategyName = graphId) {
    if (!graphId) return false;
    const label = strategyName || graphId;
    const confirmed =
      typeof window === "undefined" || typeof window.confirm !== "function"
        ? true
        : window.confirm(`确认删除策略“${label}”？此操作会移除策略文件和版本记录。`);
    if (!confirmed) return false;

    await deleteGraph(graphId);
    setSelectedStrategyIds((current) => current.filter((item) => item !== graphId));
    if (selectedStrategyId === graphId) {
      setSelectedStrategyId("");
    }
    return true;
  }

  return {
    graph,
    graphIndex,
    runtime,
    revealGraphFile,
    deleteStrategy,
    refreshGraphIndex,
    compareSelection,
    refreshRunHistory,
    refreshBacktestHistory,
    loadLatestGraph,
    openBlankWorkspace,
    toggleBacktestCompareSelection,
    clearBacktestCompareSelection,
    query,
    setQuery,
    scopeFilter,
    setScopeFilter,
    healthFilter,
    setHealthFilter,
    sortMode,
    setSortMode,
    selectedStrategyId,
    setSelectedStrategyId,
    selectedStrategyIds,
    setSelectedStrategyIds,
    selectedStrategyCount,
    selectedForWorkspace,
    toggleStrategySelection,
    templateLibrary: STRATEGY_TEMPLATE_LIBRARY,
    applyTemplate,
    strategies,
    filteredStrategies,
    selectedStrategy,
    activityTimeline,
    hubSummary
  };
}
