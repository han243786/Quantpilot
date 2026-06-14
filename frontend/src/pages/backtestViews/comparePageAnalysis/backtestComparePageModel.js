import {
  comparisonMetrics,
  formatRatio,
  formatValue,
  maxDrawdownFromSummary
} from "../shared";

export function normalizeCompareBacktestIds(backtestIds = [], limit = 2) {
  return [...new Set((backtestIds || []).filter(Boolean))].slice(0, limit);
}

export function buildBacktestCompareSummary(details = []) {
  if (!Array.isArray(details) || details.length < 2) return null;
  const [left, right] = details;
  const leftSummary = comparisonMetrics(left)?.summary || {};
  const rightSummary = comparisonMetrics(right)?.summary || {};

  return {
    returnDelta:
      (leftSummary.total_return_ratio || 0) - (rightSummary.total_return_ratio || 0),
    drawdownDelta:
      (maxDrawdownFromSummary(leftSummary) || 0) - (maxDrawdownFromSummary(rightSummary) || 0),
    tradeDelta: (leftSummary.trade_count || 0) - (rightSummary.trade_count || 0)
  };
}

export function buildBacktestCompareSummaryItems({ t = (value) => value, summary = null } = {}) {
  if (!summary) return [];

  return [
    { label: t("收益差值"), value: formatRatio(summary.returnDelta) },
    { label: t("回撤差值"), value: formatRatio(summary.drawdownDelta) },
    { label: t("成交差值"), value: formatValue(summary.tradeDelta) }
  ];
}

export function resolveBacktestCompareStrategyId({ strategyId = "", details = [] } = {}) {
  if (strategyId) return strategyId;
  if (!Array.isArray(details) || details.length !== 2) return "";

  const leftGraphId = details[0]?.graph_id || "";
  const rightGraphId = details[1]?.graph_id || "";
  return leftGraphId && leftGraphId === rightGraphId ? leftGraphId : "";
}

export function buildBacktestCompareMeta(backtestIds = []) {
  return normalizeCompareBacktestIds(backtestIds).join(" vs ") || "-";
}
