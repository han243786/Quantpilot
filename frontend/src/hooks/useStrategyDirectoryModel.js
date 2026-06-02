import { useCallback, useEffect, useMemo, useState } from "react";
import { useGraphStore } from "../store/graphStore";
import { STRATEGY_TEMPLATE_LIBRARY } from "../templates/strategyTemplates";
import { navigateTo, strategyWorkspacePath } from "../router";
import { translateText } from "../i18n";
import {
  buildActivityTimeline,
  buildAvailableGraphIds,
  buildHubSummary,
  buildStrategyDirectory,
  filterVisibleBacktests,
  filterVisibleRuns
} from "./strategyDirectoryModelProjection";

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
  const storedCompareSelection = runtime.backtestCompareSelection?.[graph?.metadata?.graph_id] || (Array.isArray(runtime.backtestCompareSelection) ? runtime.backtestCompareSelection : []);

  const [selectedStrategyId, setSelectedStrategyId] = useState("");
  const [selectedStrategyIds, setSelectedStrategyIds] = useState([]);

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

  const filteredStrategies = strategies;

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

  const toggleStrategySelection = useCallback((graphId) => {
    setSelectedStrategyIds((current) =>
      current.includes(graphId)
        ? current.filter((item) => item !== graphId)
        : [...current, graphId]
    );
  }, []);

  async function applyTemplate(templateId) {
    const graph = await loadStrategyTemplate(templateId);
    navigateTo(strategyWorkspacePath(graph.metadata?.graph_id || "draft_graph"));
    return graph;
  }

  const openBlankWorkspace = useCallback(() => {
    const graph = resetGraph();
    navigateTo(strategyWorkspacePath(graph.metadata?.graph_id || "draft_graph"));
    return graph;
  }, []);

  async function deleteStrategy(graphId, strategyName = graphId) {
    if (!graphId) return false;
    const label = strategyName || graphId;
    const confirmed =
      typeof window === "undefined" || typeof window.confirm !== "function"
        ? true
        : window.confirm(translateText('确认删除策略”{label}”？此操作会移除策略文件和版本记录。', { label }));
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
