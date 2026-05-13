import { deleteJson, fetchJson, postJson, unwrapPage } from "./graphStorePersistenceHelpers";

const list = (url) => fetchJson(url).then(unwrapPage);

export function fetchRunHistoryList() {
  return list("/runtime/runs");
}

export function fetchBacktestHistoryList() {
  return list("/runtime/backtests");
}

export function fetchRunDetail(runId) {
  return fetchJson(`/runtime/runs/${runId}`);
}

export function saveRunRecord(runId) {
  return postJson(`/runtime/runs/${runId}/save`, {});
}

export function discardRunRecord(runId) {
  return deleteJson(`/runtime/runs/${runId}`);
}

export function fetchBacktestDetail(backtestId) {
  return fetchJson(`/runtime/backtests/${backtestId}`);
}

export function saveBacktestRecord(backtestId) {
  return postJson(`/runtime/backtests/${backtestId}/save`, {});
}

export function discardBacktestRecord(backtestId) {
  return deleteJson(`/runtime/backtests/${backtestId}`);
}

export function fetchExperimentHistoryList() {
  return list("/runtime/experiments");
}

export function fetchExperimentDetail(experimentId) {
  return fetchJson(`/runtime/experiments/${experimentId}`);
}

export function saveExperimentRecord(experimentId) {
  return postJson(`/runtime/experiments/${experimentId}/save`, {});
}

export function discardExperimentRecord(experimentId) {
  return deleteJson(`/runtime/experiments/${experimentId}`);
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

export function fetchRuntimeReports() {
  return list("/runtime/reports");
}

export function createRuntimeReport(request) {
  return postJson("/runtime/reports", request);
}

export function fetchRuntimeReportDetail(reportId) {
  return fetchJson(`/runtime/reports/${reportId}`);
}

export function runtimeReportExportPath(reportId) {
  return `/runtime/reports/${reportId}/export`;
}

export function fetchRuntimeMutations(params = {}) {
  return list(withQuery("/runtime/mutations", params));
}

export function createRuntimeMutation(request) {
  return postJson("/runtime/mutations", request);
}

export function fetchRuntimeMutationDetail(proposalId) {
  return fetchJson(`/runtime/mutations/${proposalId}`);
}

export function activateRuntimeMutation(proposalId, request) {
  return postJson(`/runtime/mutations/${proposalId}/activate`, request);
}

export function rollbackRuntimeMutation(proposalId, request) {
  return postJson(`/runtime/mutations/${proposalId}/rollback`, request);
}
