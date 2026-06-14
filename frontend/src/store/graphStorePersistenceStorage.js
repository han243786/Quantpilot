export const STORAGE_KEY = "quantpilot_frontend_graph";
export const CAPABILITY_CACHE_KEY = "quantpilot_capabilities_cache";

const GRAPH_STORAGE_SCHEMA = 1;
const STORAGE_ESTIMATE_KEY = "__qp_storage_estimate__";
const DEFAULT_LOCAL_STORAGE_QUOTA = 5_242_880;

function safeGetItem(key) {
  if (typeof window === "undefined") return null;
  try {
    return window.localStorage.getItem(key);
  } catch (_) {
    // 隐私模式或配额限制下 getItem 可能抛出 DOMException
    return null;
  }
}

function safeSetItem(key, value) {
  if (typeof window === "undefined") return;
  let data;
  try {
    data = JSON.stringify(value);
  } catch (e) {
    console.warn("[storage] JSON.stringify 失败, 跳过 localStorage 写入", e);
    return;
  }

  if (data.length > 500_000 && typeof navigator !== "undefined" && navigator.storage?.estimate) {
    try {
      const estimate = JSON.parse(window.localStorage.getItem(STORAGE_ESTIMATE_KEY) || "{}");
      const quota = estimate.quota || DEFAULT_LOCAL_STORAGE_QUOTA;
      const usage = data.length + (estimate.usage || 0);
      if (usage > quota * 0.9) {
        console.warn("[storage] localStorage 使用量接近上限, 请清理旧策略图版本");
      }
    } catch (e) {
      console.warn("[storage] 配额检查失败", e);
    }
    navigator.storage.estimate().then((estimate) => {
      window.localStorage.setItem(STORAGE_ESTIMATE_KEY, JSON.stringify(estimate));
    }).catch(() => {});
  }

  try {
    window.localStorage.setItem(key, data);
  } catch (e) {
    if (e.name === "QuotaExceededError" || e.code === 22) {
      console.warn("[qp] localStorage 配额已满, 策略图未保存到本地缓存");
      window.dispatchEvent(new CustomEvent("qp-storage-quota-exceeded"));
    }
  }
}

export function saveGraphToStorage(graph) {
  safeSetItem(STORAGE_KEY, { _schema: GRAPH_STORAGE_SCHEMA, ...graph });
}

export function loadGraphFromStorage() {
  if (typeof window === "undefined") return null;
  const raw = safeGetItem(STORAGE_KEY);
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw);
    // v2.5.0: 使用 !== 严格匹配, schema 变更时丢弃旧缓存避免格式不兼容
    if (parsed._schema && parsed._schema !== GRAPH_STORAGE_SCHEMA) {
      console.warn(`[storage] 策略图缓存 schema 版本不兼容 (存储 ${parsed._schema}, 当前 ${GRAPH_STORAGE_SCHEMA}), 将丢弃旧数据`);
      return null;
    }
    return parsed;
  } catch (e) {
    console.warn("[storage] 数据解析失败", e);
    return null;
  }
}

export function saveCapabilitiesToCache(capabilities) {
  safeSetItem(CAPABILITY_CACHE_KEY, capabilities);
}

export function loadCapabilitiesFromCache() {
  const raw = safeGetItem(CAPABILITY_CACHE_KEY);
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw);
    return parsed && typeof parsed === "object" ? parsed : null;
  } catch (e) {
    console.warn("[storage] 数据解析失败", e);
    return null;
  }
}
