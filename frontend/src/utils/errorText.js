const DIRECT_TRANSLATIONS = new Map([
  ["QuantScript must start with strategy_graph.", "策略图源码必须以 `strategy_graph` 开头。"],
  ["Invalid strategy_graph header.", "策略图源码头部格式无效。"],
  ["strategy graph source must start with strategy_graph", "策略图源码必须以 `strategy_graph` 开头。"],
  ["invalid strategy graph source header", "策略图源码头部格式无效。"],
  ["Failed to fetch", "无法连接后端服务，请确认本地 API 已启动。"],
  [
    "NetworkError when attempting to fetch resource.",
    "无法连接后端服务，请确认本地 API 已启动。"
  ]
]);

const CORRUPTED_PATTERNS = [/锟/, /\uFFFD/, /閿/, /鐑/];

function isString(value) {
  return typeof value === "string";
}

export function looksCorruptedText(value) {
  if (!isString(value)) return false;
  const trimmed = value.trim();
  if (!trimmed) return false;
  return CORRUPTED_PATTERNS.some((pattern) => pattern.test(trimmed));
}

export function sanitizeDisplayText(value, fallback = "") {
  if (!isString(value)) return fallback;
  const trimmed = value.trim();
  if (!trimmed || looksCorruptedText(trimmed)) return fallback;
  return trimmed;
}

function extractMessageFromJson(raw) {
  const trimmed = raw.trim();
  if (!trimmed.startsWith("{") && !trimmed.startsWith("[")) return raw;
  try {
    const parsed = JSON.parse(trimmed);
    if (isString(parsed?.message)) return parsed.message;
    if (isString(parsed?.error)) return parsed.error;
  } catch {
    // 非结构化或截断响应按原文展示。这里不打印 warning，避免负向错误路径污染测试和 CI 日志。
  }
  return raw;
}

function stripHtml(raw) {
  return raw
    .replace(/<[^>]+>/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

// v3.6.0 U6: 错误码→中文映射
const ERROR_CODE_MAP = new Map([
  ["QS0001", "函数名重复"],
  ["QS0002", "未定义的标识符"],
  ["QS0403", "除零错误"],
  ["QS0501", "数据不足，需要更多K线"],
  ["QS0505", "未知交易对"],
  ["QS0605", "不支持递归调用"],
  ["QSPIPELINE", "编译通过"],
  ["ERR_BAD_REQUEST", "请求格式错误"],
  ["ERR_COMPILE_FAILED", "编译失败"],
  ["ERR_RUN_IN_PROGRESS", "已有模拟在运行中"],
  ["ERR_AUTH_UNAUTHORIZED", "认证失败"],
  ["auth_failed", "认证失败"],
  ["token_invalid", "令牌无效，请重新登录"],
  ["token_replay", "令牌已被重放，请重新登录"],
  ["refresh_token_invalid", "刷新令牌无效，请重新登录"],
  ["rate-limited", "请求过于频繁，请稍后再试"],
]);

function translateMessage(message, fallback) {
  if (!message) return fallback;
  if (DIRECT_TRANSLATIONS.has(message)) return DIRECT_TRANSLATIONS.get(message);

  // v3.6.0: 错误码替换为中文
  for (const [code, desc] of ERROR_CODE_MAP) {
    if (message.includes(code)) return desc + "。" + (message.includes(":") ? " 详情: " + message.split(":").slice(1).join(":").trim() : "");
  }

  const unknownNodeMatch = message.match(/^Unknown module for node (.+)$/);
  if (unknownNodeMatch) {
    return `节点 ${unknownNodeMatch[1]} 使用了未知模块。`;
  }

  const httpMatch = message.match(/^HTTP\s+(\d+)$/i);
  if (httpMatch) {
    return humanizeHttpError(Number(httpMatch[1]));
  }

  return message;
}

export function humanizeErrorText(errorLike, fallback = "操作失败。") {
  // fallback 支持传入操作名以提供上下文: "操作名 操作失败。"
  const contextualFallback = typeof fallback === "string" && fallback !== "操作失败。" ? fallback : "操作失败。";
  const raw =
    typeof errorLike === "string"
      ? errorLike
      : isString(errorLike?.message)
        ? errorLike.message
        : "";

  const trimmed = raw.trim();
  if (!trimmed) return fallback;

  const fromJson = extractMessageFromJson(trimmed);
  // 标准化标点: 去掉尾随句点以确保翻译键匹配
  const withoutTrailingPunct = stripHtml(fromJson).replace(/[.。！!？?]+$/g, "");
  const normalized = translateMessage(withoutTrailingPunct, fallback);
  return sanitizeDisplayText(normalized, fallback);
}

// v3.6.0 U6: HTTP 状态码→中文描述
const HTTP_CODE_MAP = {
  400: "请求格式有误",
  401: "请先登录",
  403: "无权访问",
  404: "请求的资源不存在",
  409: "资源冲突",
  422: "无法处理请求",
  423: "资源已锁定",
  429: "请求过于频繁，请稍后再试",
  500: "服务器内部错误",
  503: "服务暂时不可用",
};

export function humanizeHttpError(status) {
  const code = Number(status);
  return HTTP_CODE_MAP[code] || `后端请求失败（HTTP ${code}）`;
}
