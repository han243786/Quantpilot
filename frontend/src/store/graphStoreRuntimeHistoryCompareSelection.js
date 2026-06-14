function sanitizeCompareSelection(backtestIds) {
  return Array.isArray(backtestIds)
    ? [...new Set(backtestIds.filter(Boolean))].slice(0, 2)
    : [];
}

export function resolveBacktestCompareStrategyId(state) {
  return state.graph?.metadata?.graph_id || "_global";
}

export function getCompareSelection(state) {
  const raw = state.runtime.backtestCompareSelection;
  if (!raw) return [];
  if (Array.isArray(raw)) return raw;
  return raw[resolveBacktestCompareStrategyId(state)] || [];
}

export function buildBacktestCompareSelectionMap(state, backtestIds) {
  const raw = state.runtime.backtestCompareSelection;
  const base = Array.isArray(raw) ? {} : { ...raw };
  return {
    ...base,
    [resolveBacktestCompareStrategyId(state)]: sanitizeCompareSelection(backtestIds)
  };
}

export function toggleBacktestCompareSelectionState(state, backtestId) {
  const existing = getCompareSelection(state);
  const next = existing.includes(backtestId)
    ? existing.filter((id) => id !== backtestId)
    : existing.length >= 2
      ? existing
      : [...existing, backtestId];
  return {
    runtime: {
      ...state.runtime,
      backtestCompareSelection: buildBacktestCompareSelectionMap(state, next)
    }
  };
}

export function clearBacktestCompareSelectionState(state) {
  return {
    runtime: {
      ...state.runtime,
      backtestCompareSelection: buildBacktestCompareSelectionMap(state, [])
    }
  };
}

export function replaceBacktestCompareSelectionState(state, backtestIds) {
  return {
    runtime: {
      ...state.runtime,
      backtestCompareSelection: buildBacktestCompareSelectionMap(state, backtestIds)
    }
  };
}
