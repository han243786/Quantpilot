import { API_BASE } from "./graphStorePersistenceHelpers";

export function buildRuntimeEventStreamUrl(runId) {
  return `${API_BASE}/runtime/runs/${runId}/events`;
}

/**
 * v3.6.0 U9: 无限重连 + 浏览器online事件联动
 * 断开后指数退避 (1s→2s→4s...→max 60s), 不限次数
 */
export function createRuntimeEventSource(runId, onRetryExhausted, onReconnect) {
  const url = buildRuntimeEventStreamUrl(runId);
  let retries = 0;
  const BASE_DELAY_MS = 1000;
  const MAX_DELAY_MS = 60_000;

  function build() {
    const es = new EventSource(url);
    es._manualClose = false;

    const originalClose = es.close.bind(es);
    es.close = () => {
      es._manualClose = true;
      originalClose();
    };

    es._reconnect = () => {
      if (es._manualClose) return null;
      const delay = Math.min(BASE_DELAY_MS * Math.pow(2, retries), MAX_DELAY_MS);
      retries++;
      const timerId = setTimeout(() => {
        const next = build();
        if (es._onMessage) next.addEventListener("runtime_event", es._onMessage);
        if (es._onAccount) next.addEventListener("account", es._onAccount);
        if (es._onComplete) next.addEventListener("run_completed", es._onComplete);
        if (es._onError) next.onerror = es._onError;
        if (onReconnect) onReconnect(next);
      }, delay);
      es._reconnectTimer = timerId;
      return timerId;
    };

    // v3.6.0: 浏览器恢复在线时立即重连
    const onlineHandler = () => { if (!es._manualClose) { es._reconnectTimer = es._reconnect(); } };
    window.addEventListener("online", onlineHandler);
    es._onlineHandler = onlineHandler;

    return es;
  }

  return build();
}
