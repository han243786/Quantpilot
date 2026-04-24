import { fetchJson } from "./graphStorePersistenceHelpers";

export function fetchRunHistoryList() {
  return fetchJson("/runtime/runs");
}

export function fetchBacktestHistoryList() {
  return fetchJson("/runtime/backtests");
}

export function fetchRunDetail(runId) {
  return fetchJson(`/runtime/runs/${runId}`);
}

export function fetchBacktestDetail(backtestId) {
  return fetchJson(`/runtime/backtests/${backtestId}`);
}

export function fetchExperimentHistoryList() {
  return fetchJson("/runtime/experiments");
}

export function fetchExperimentDetail(experimentId) {
  return fetchJson(`/runtime/experiments/${experimentId}`);
}

function withQuery(path, params = {}) {
  const search = new URLSearchParams();
  Object.entries(params).forEach(([key, value]) => {
    if (value === null || value === undefined || value === "") return;
    search.set(key, String(value));
  });
  const query = search.toString();
  return query ? `${path}?${query}` : path;
}

export function fetchRunReplay(runId, params = {}) {
  return fetchJson(withQuery(`/runtime/runs/${runId}/replay`, params));
}

export function fetchBacktestReplay(backtestId, params = {}) {
  return fetchJson(withQuery(`/runtime/backtests/${backtestId}/replay`, params));
}
