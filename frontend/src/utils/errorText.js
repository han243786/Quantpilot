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
  try {
    const parsed = JSON.parse(raw);
    if (isString(parsed?.message)) return parsed.message;
    if (isString(parsed?.error)) return parsed.error;
  } catch {
  }
  return raw;
}

function stripHtml(raw) {
  return raw
    .replace(/<[^>]+>/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

function translateMessage(message, fallback) {
  if (!message) return fallback;
  if (DIRECT_TRANSLATIONS.has(message)) return DIRECT_TRANSLATIONS.get(message);

  const unknownNodeMatch = message.match(/^Unknown module for node (.+)$/);
  if (unknownNodeMatch) {
    return `节点 ${unknownNodeMatch[1]} 使用了未知模块。`;
  }

  const httpMatch = message.match(/^HTTP\s+(\d+)$/i);
  if (httpMatch) {
    return `后端请求失败（HTTP ${httpMatch[1]}）。`;
  }

  return message;
}

export function humanizeErrorText(errorLike, fallback = "操作失败。") {
  const raw =
    typeof errorLike === "string"
      ? errorLike
      : isString(errorLike?.message)
        ? errorLike.message
        : "";

  const trimmed = raw.trim();
  if (!trimmed) return fallback;

  const fromJson = extractMessageFromJson(trimmed);
  const normalized = translateMessage(stripHtml(fromJson), fallback);
  return sanitizeDisplayText(normalized, fallback);
}
