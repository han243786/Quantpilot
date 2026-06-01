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

import { API_BASE } from "./apiBase";

export { API_BASE, getAuthHeaders, resolveApiBase } from "./apiBase";

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
