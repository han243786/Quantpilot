// ── API 错误码 → 本地化文本映射 (v2.3.0) ──
// 前端使用: humanizeApiError(error_code, locale) → 显示文本

const MESSAGES = {
  "BAD_REQUEST": { zh: "请求格式错误", en: "Bad request" },
  "INTERNAL_ERROR": { zh: "内部服务器错误, 请重试", en: "Internal server error, please retry" },
  "NOT_FOUND": { zh: "请求的资源不存在", en: "Resource not found" },
  "SERVICE_UNAVAILABLE": { zh: "服务暂不可用, 请稍后重试", en: "Service unavailable, please try later" },

  "GRAPH_ID_EMPTY": { zh: "策略图 ID 不能为空", en: "Graph ID cannot be empty" },
  "GRAPH_ID_TOO_LONG": { zh: "策略图 ID 过长 (最多128字符)", en: "Graph ID too long (max 128 chars)" },
  "GRAPH_NOT_FOUND": { zh: "策略图不存在", en: "Graph not found" },
  "GRAPH_SAVE_FAILED": { zh: "策略图保存失败", en: "Graph save failed" },

  "STRATEGY_EMPTY_INTENT": { zh: "策略必须包含至少一个意图", en: "Strategy must contain at least one intent" },
  "COMPILE_CAPABILITY_GATED": { zh: "运行时配置使用了当前版本未启用的能力", en: "Runtime config uses capabilities not enabled in current version" },
  "COMPILE_FAILED": { zh: "编译失败", en: "Compilation failed" },
  "COMPILE_BUSY": { zh: "编译服务繁忙, 请稍后重试", en: "Compilation service busy, please try later" },

  "RUN_IN_PROGRESS": { zh: "已有运行在进行中, 请先停止当前运行", en: "A run is already in progress, please stop it first" },
  "RUN_NOT_FOUND": { zh: "运行记录不存在", en: "Run record not found" },

  "BACKTEST_NOT_FOUND": { zh: "回测记录不存在", en: "Backtest record not found" },
  "BACKTEST_COMPARE_TWO_IDS": { zh: "回测比较需要恰好两个 backtest_id", en: "Backtest comparison requires exactly two backtest IDs" },

  "AUTH_UNAUTHORIZED": { zh: "认证失败, 请使用有效 token", en: "Authentication failed, please use a valid token" },
  "AUTH_LOGIN_FAILED": { zh: "用户名或密码错误", en: "Invalid username or password" },
  "AUTH_TOKEN_EXPIRED": { zh: "token 已过期, 请重新登录", en: "Token expired, please login again" },
  "AUTH_RATE_LIMITED": { zh: "登录尝试过于频繁, 请稍后重试", en: "Login attempts too frequent, please try later" },

  "CREDENTIAL_VAULT_UNAVAIL": { zh: "凭证保险库未初始化", en: "Credential vault not initialized" },
  "CREDENTIAL_SAVE_FAILED": { zh: "凭证保存失败", en: "Credential save failed" },
  "CREDENTIAL_NOT_FOUND": { zh: "凭证标签不存在", en: "Credential label not found" },

  "STORAGE_FULL": { zh: "存储空间已满, 请清理过期数据后重试", en: "Storage full, please clean expired data" },

  "PLUGIN_MANIFEST_INVALID": { zh: "插件清单校验失败", en: "Plugin manifest validation failed" },
  "PLUGIN_NOT_FOUND": { zh: "插件未注册", en: "Plugin not registered" },
};

const DEFAULT_MESSAGE = { zh: "未知错误", en: "Unknown error" };

/**
 * 将 API 错误码转换为本地化文本
 * @param {string} errorCode - API 返回的 error_code
 * @param {string} locale - "zh-CN" | "en-US"
 * @param {string} fallback - 如果 error_code 未找到时的回退文本
 * @returns {string} 本地化错误文本
 */
export function humanizeApiError(errorCode, locale = "zh-CN", fallback = "") {
  const lang = locale.startsWith("en") ? "en" : "zh";
  const entry = MESSAGES[errorCode];
  if (entry && entry[lang]) {
    return entry[lang];
  }
  return fallback || (entry ? entry.zh : DEFAULT_MESSAGE[lang]);
}
