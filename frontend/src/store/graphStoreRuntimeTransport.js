import { API_BASE } from "./graphStorePersistenceHelpers";

export function buildRuntimeEventStreamUrl(runId) {
  return `${API_BASE}/runtime/runs/${runId}/events`;
}

export function createRuntimeEventSource(runId) {
  return new EventSource(buildRuntimeEventStreamUrl(runId));
}
