export function formatValue(value) {
  if (value === null || value === undefined || value === "") return "-";
  if (typeof value === "number") {
    if (!Number.isFinite(value)) return "-";
    return Number.isInteger(value) ? String(value) : value.toFixed(4);
  }
  return String(value);
}

export { formatTime, formatPercent } from "../utils/strategyFormatters";

export function formatRatio(value) {
  if (!Number.isFinite(value)) return "-";
  const percent = value * 100;
  const sign = percent > 0 ? "+" : "";
  return `${sign}${percent.toFixed(2)}%`;
}

export function formatSharpeRatio(value) {
  if (!Number.isFinite(value)) return "-";
  return value.toFixed(2);
}

export function formatAnnualizedReturn(value) {
  if (!Number.isFinite(value)) return "-";
  const percent = value * 100;
  const sign = percent > 0 ? "+" : "";
  return `${sign}${percent.toFixed(2)}%`;
}

export function formatDays(value) {
  if (!Number.isFinite(value) || value < 0) return "-";
  return `${Math.round(value)} 天`;
}

export function formatProfitFactor(value) {
  if (!Number.isFinite(value)) return "-";
  if (value > 999) return "∞";
  return value.toFixed(2);
}

export function sharpeColor(value) {
  if (!Number.isFinite(value)) return "var(--ad-text)";
  if (value < 0) return "var(--ad-error)";
  if (value >= 1.0) return "var(--ad-success)";
  if (value >= 0.5) return "var(--ad-warning)";
  return "var(--ad-text)";
}

export function profitFactorColor(value) {
  if (!Number.isFinite(value)) return "var(--ad-text)";
  return value >= 1.0 ? "var(--ad-success)" : "var(--ad-error)";
}

export function datasetLabelsFromDetail(detail) {
  return (
    detail.backtest_artifacts?.manifest?.backtest_spec?.run_spec?.datasets?.map((dataset) => {
      const interval = dataset.interval || "na";
      return `${dataset.exchange}:${dataset.symbol}:${interval}`;
    }) || []
  );
}

export function executionAssumptionsFromDetail(detail) {
  return (
    detail.execution_assumptions ||
    detail.backtest_artifacts?.metrics?.execution_assumptions ||
    null
  );
}

export function executionAssumptionsLabelFromDetail(detail) {
  const assumptionsModule = executionAssumptionsFromDetail(detail);
  if (!assumptionsModule) return "-";
  const summaryLabel = assumptionsModule.list_tag?.label || "-";
  const sourcesLabel = assumptionsModule.list_tag?.sources_label;
  return sourcesLabel ? `${summaryLabel} (${sourcesLabel})` : summaryLabel;
}

export function comparisonMetrics(detail) {
  return detail.backtest_artifacts?.metrics || null;
}

/** v1.1.0: 从 BacktestSummary 提取风险调整指标 */
export function riskAdjustedFromSummary(summary) {
  return summary?.risk_adjusted || {};
}

/** v1.1.0: 从 BacktestSummary 提取交易分析指标 */
export function tradeAnalysisFromSummary(summary) {
  return summary?.trade_analysis || {};
}

/** v1.1.0: 从 BacktestSummary 提取回撤分析指标 */
export function drawdownAnalysisFromSummary(summary) {
  return summary?.drawdown_analysis || {};
}

/** v1.1.0: 从 BacktestSummary 提取基准对比指标 */
export function benchmarkComparisonFromSummary(summary) {
  return summary?.benchmark_comparison || null;
}

/** v1.1.0: 获取最大回撤（兼容新旧格式） */
export function maxDrawdownFromSummary(summary) {
  return summary?.drawdown_analysis?.max_drawdown_ratio ?? summary?.max_drawdown_ratio ?? 0;
}

export function MetricPair({ label, value, testId = null, fullValue = null }) {
  return (
    <div className="kv-line" data-testid={testId || undefined}>
      <span>{label}</span>
      <strong title={fullValue || value}>{value}</strong>
    </div>
  );
}
