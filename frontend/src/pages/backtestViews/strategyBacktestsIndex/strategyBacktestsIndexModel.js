import { formatPercent, formatTime, formatValue } from "../shared";

export function buildStrategyBacktestsIndexModel({ graph = {}, selectors = {}, strategyId = "" }) {
  const graphId = graph.metadata?.graph_id || "";
  const filteredBacktests = Array.isArray(selectors.filteredBacktests)
    ? selectors.filteredBacktests
    : [];
  const compareSelection = Array.isArray(selectors.compareSelection)
    ? selectors.compareSelection
    : [];
  const selectedBacktest = filteredBacktests[0] || null;
  const datasetLabels =
    graphId === strategyId ? selectedBacktest?.filters?.dataset_labels || [] : [];

  return {
    strategyName: graphId === strategyId ? graph.metadata?.name || strategyId : strategyId,
    summaryItems: [
      { label: "回测数", value: formatValue(filteredBacktests.length) },
      { label: "对比队列", value: formatValue(compareSelection.length) },
      {
        label: "最近收益",
        value: formatPercent(selectedBacktest?.summary?.total_return_ratio)
      },
      {
        label: "最近回测",
        value: selectedBacktest ? formatTime(selectedBacktest.created_at_ms) : "-"
      }
    ],
    compareButtonDisabled: compareSelection.length !== 2,
    isGraphLoading: graphId !== strategyId,
    datasetText: datasetLabels.join(", ") || "-"
  };
}
