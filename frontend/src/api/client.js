/**
 * QuantPilot 统一 API 客户端
 * v1.0.5 — 消除 4 个 API_BASE 来源, 统一为单一路径解析
 *
 * 用法:
 *   import { apiClient } from "../api/client";
 *   const data = await apiClient.get("/graphs/latest");
 *   await apiClient.post("/graphs/save", { graph, actor });
 *   await apiClient.del("/graphs/abc123");
 */

const resolveApiBase = () => {
  const raw = import.meta.env.VITE_API_BASE_URL?.trim();
  if (raw) return raw.replace(/\/+$/, "");
  if (typeof window === "undefined") return "http://127.0.0.1:3000/api";
  return "/api";
};

export const API_BASE = resolveApiBase();

async function request(method, path, body, { timeoutMs = 30000, headers = {} } = {}) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);

  const options = {
    method,
    headers: { "Content-Type": "application/json", ...headers },
    signal: controller.signal,
  };
  if (body !== undefined) {
    options.body = JSON.stringify(body);
  }

  try {
    const response = await fetch(`${API_BASE}${path}`, options);
    if (!response.ok) {
      const text = await response.text();
      const error = new Error(text.slice(0, 2000) || `服务器错误 (${response.status})`);
      error.status = response.status;
      throw error;
    }
    const contentType = response.headers.get("content-type") || "";
    if (contentType.includes("application/json")) {
      return response.json();
    }
    return response.text();
  } finally {
    clearTimeout(timer);
  }
}

export const apiClient = {
  get: (path, opts) => request("GET", path, undefined, opts),
  post: (path, body, opts) => request("POST", path, body, opts),
  del: (path, opts) => request("DELETE", path, undefined, opts),
};

/** 分页辅助: 将 {limit, offset} 转为 query string */
export function withPagination(path, { limit, offset } = {}) {
  const params = new URLSearchParams();
  if (limit != null) params.set("limit", String(limit));
  if (offset != null) params.set("offset", String(offset));
  const qs = params.toString();
  return qs ? `${path}?${qs}` : path;
}

/** 扩展点: 未来接入身份验证时修改此函数即可 */
export function getAuthHeaders() {
  return {};
}
