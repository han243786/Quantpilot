export function formatTime(value) {
  return value ? new Date(value).toLocaleString() : "-";
}

export function formatCount(value) {
  if (!Number.isFinite(value)) return "0";
  return new Intl.NumberFormat().format(value);
}

export function formatPercent(value) {
  if (!Number.isFinite(value)) return "-";
  const sign = value > 0 ? "+" : "";
  return `${sign}${(value * 100).toFixed(2)}%`;
}
