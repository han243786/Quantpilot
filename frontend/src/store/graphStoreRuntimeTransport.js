import { API_BASE } from "./graphStorePersistenceHelpers";

export function buildRuntimeEventStreamUrl(runId) {
  return `${API_BASE}/runtime/runs/${runId}/events`;
}

/**
 * 创建带自动重连的 EventSource。
 * v1.0.0: 断开后指数退避重连 (最多 5 次, 1s→2s→4s→8s→16s)
 */
export function createRuntimeEventSource(runId, onRetryExhausted) {
  const url = buildRuntimeEventStreamUrl(runId);
  let retries = 0;
  const MAX_RETRIES = 5;
  const BASE_DELAY_MS = 1000;

  function build() {
    const es = new EventSource(url);
    es._manualClose = false;

    const originalClose = es.close.bind(es);
    es.close = () => {
      es._manualClose = true;
      originalClose();
    };

    es._reconnect = () => {
      if (es._manualClose || retries >= MAX_RETRIES) {
        if (retries >= MAX_RETRIES && onRetryExhausted) {
          onRetryExhausted();
        }
        return null;
      }
      const delay = BASE_DELAY_MS * Math.pow(2, retries);
      retries++;
      return setTimeout(() => {
        const next = build();
        // 复制事件监听器
        if (es._onMessage) next.addEventListener("runtime_event", es._onMessage);
        if (es._onComplete) next.addEventListener("run_completed", es._onComplete);
        if (es._onError) next.onerror = es._onError;
      }, delay);
    };

    return es;
  }

  return build();
}
