const MAX_RECENT_ITEMS = 4;
const MAX_ACTIVITY_ITEMS = 6;

function pushRecent(list, item, limit = MAX_RECENT_ITEMS) {
  if (!item) return list;
  list.push(item);
  list.sort((left, right) => (right.created_at_ms || 0) - (left.created_at_ms || 0) || (left.graph_id || "").localeCompare(right.graph_id || ""));
  return list.slice(0, limit);
}

export function resolveStrategyDirectoryHealth(entry) {
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

export function resolveStrategyDirectoryActivityLabel(entry) {
  if (!entry.lastActivityAt) return "暂无活动";
  if (entry.lastBacktestAt && entry.lastBacktestAt === entry.lastActivityAt) {
    return "回测";
  }
  if (entry.lastRunAt && entry.lastRunAt === entry.lastActivityAt) {
    return "模拟";
  }
  return "已更新";
}

export function buildAvailableGraphIds(graph, graphIndex) {
  return new Set((Array.isArray(graphIndex) ? graphIndex : []).map((entry) => entry.graph_id).filter(Boolean));
}

export function filterVisibleRuns(runtime, availableGraphIds, currentGraphId) {
  return (runtime.history || []).filter((run) =>
    availableGraphIds.has(run.graph_id || currentGraphId)
  );
}

export function filterVisibleBacktests(runtime, availableGraphIds, currentGraphId) {
  return (runtime.backtestHistory || []).filter((backtest) =>
    availableGraphIds.has(backtest.graph_id || currentGraphId)
  );
}

export function buildStrategyDirectory(graph, visibleRuns, visibleBacktests, graphIndex) {
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
    const currentEntry = ensure(currentGraphId);
    Object.assign(currentEntry, {
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
        : (backtest.filters?.dataset_labels || []);
    entry.recentBacktests = pushRecent(entry.recentBacktests, backtest);
  }

  return [...map.values()]
    .map((entry) => ({
      ...entry,
      datasetLabels: [...new Set(entry.datasetLabels)].slice(0, 3),
      health: resolveStrategyDirectoryHealth(entry),
      activityLabel: resolveStrategyDirectoryActivityLabel(entry)
    }))
    .sort((left, right) => {
      if (left.isCurrent !== right.isCurrent) {
        return left.isCurrent ? -1 : 1;
      }
      return (right.lastActivityAt || 0) - (left.lastActivityAt || 0) || (left.graph_id || "").localeCompare(right.graph_id || "");
    });
}

export function buildActivityTimeline(visibleRuns, visibleBacktests) {
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
    .sort((left, right) => right.createdAt - left.createdAt || (left.id || "").localeCompare(right.id || ""))
    .slice(0, MAX_ACTIVITY_ITEMS);
}

export function buildHubSummary(strategies, activityTimeline, compareSelection) {
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
