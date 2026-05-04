export function formatValue(value) {
  if (value === null || value === undefined || value === "") return "-";
  if (typeof value === "number") {
    return Number.isInteger(value) ? String(value) : value.toFixed(4);
  }
  return String(value);
}

export function formatTime(value) {
  return value ? new Date(value).toLocaleString() : "-";
}

export function formatPercent(value) {
  if (!Number.isFinite(value)) return "-";
  const sign = value > 0 ? "+" : "";
  return `${sign}${(value * 100).toFixed(2)}%`;
}

export function formatRatio(value) {
  if (!Number.isFinite(value)) return "-";
  const percent = value * 100;
  const sign = percent > 0 ? "+" : "";
  return `${sign}${percent.toFixed(2)}%`;
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

export function MetricPair({ label, value, testId = null, fullValue = null }) {
  return (
    <div className="kv-line" data-testid={testId || undefined}>
      <span>{label}</span>
      <strong title={fullValue || value}>{value}</strong>
    </div>
  );
}
