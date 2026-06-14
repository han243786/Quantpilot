export function capabilitySet(values, fallback) {
  return new Set(Array.isArray(values) && values.length > 0 ? values : fallback);
}

export function supportMap(entries, keyField = "key") {
  return new Map(
    (Array.isArray(entries) ? entries : [])
      .filter((entry) => entry && typeof entry === "object" && entry[keyField])
      .map((entry) => [entry[keyField], entry])
  );
}

export function capabilityEntryStatus(entry, fallbackSet, key) {
  if (entry) return entry.status === "supported";
  return fallbackSet.has(key);
}

export function capabilityReason(entry, fallback = "") {
  return entry?.reason || fallback;
}

export function jsonValue(value) {
  return value === undefined ? null : value;
}

export function parseCsvStrings(value) {
  if (typeof value !== "string") {
    return [];
  }
  return value
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
}

export function parseCsvNumbers(value) {
  return parseCsvStrings(value)
    .map((item) => Number(item))
    .filter((item) => Number.isFinite(item));
}

function normalizeOptionalString(value) {
  return typeof value === "string" ? value.trim() : "";
}

export function normalizeRebalanceSchedule(value) {
  const normalized = normalizeOptionalString(value);
  if (!normalized) return null;
  if (["every_slow", "every_1d", "weekly"].includes(normalized)) {
    return normalized;
  }
  return "__invalid__";
}

export function normalizeRebalanceAllocationKind(value) {
  const normalized = normalizeOptionalString(value);
  if (!normalized) return null;
  if (["equal_weight", "score_weight", "rank_weight", "fixed_weights"].includes(normalized)) {
    return normalized;
  }
  return "__invalid__";
}

export function normalizeRebalanceRankMethod(value) {
  const normalized = normalizeOptionalString(value);
  if (!normalized) return null;
  if (["linear", "inverse_rank"].includes(normalized)) {
    return normalized;
  }
  return "__invalid__";
}

export function normalizeRebalanceScoreNormalize(value) {
  const normalized = normalizeOptionalString(value);
  if (!normalized) return null;
  if (["sum"].includes(normalized)) {
    return normalized;
  }
  return "__invalid__";
}

export function agentUsesPortfolioRebalance(config = {}) {
  return (
    parseCsvStrings(config.rebalance_symbols).length > 0 ||
    Boolean(normalizeOptionalString(config.rebalance_schedule)) ||
    Boolean(normalizeOptionalString(config.rebalance_allocation_kind)) ||
    Boolean(normalizeOptionalString(config.rebalance_rank_method)) ||
    Boolean(normalizeOptionalString(config.rebalance_score_normalize)) ||
    Boolean(normalizeOptionalString(config.rebalance_target_weights))
  );
}

function makeCompileDiagnostic(severity, message, code = null, target = null, hint = null) {
  return {
    source: "graph",
    code: code || (severity === "warning" ? "GRAPH_COMPILE_WARNING" : "GRAPH_COMPILE_ERROR"),
    severity,
    message,
    target,
    hint
  };
}

export function buildLocalCompileDiagnostics(errors, warnings) {
  return [
    ...errors.map((message) => makeCompileDiagnostic("error", message)),
    ...warnings.map((message) => makeCompileDiagnostic("warning", message))
  ];
}
