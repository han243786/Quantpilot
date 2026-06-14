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

import { createApiClient, request } from "./apiTransport";

export { API_BASE, getAuthHeaders, resolveApiBase } from "./apiBase";
export { createApiClient, request } from "./apiTransport";
export { fetchWithTimeout } from "./fetchHelpers";

export const apiClient = createApiClient();

/** 分页辅助: 将 {limit, offset} 转为 query string */
export function withPagination(path, { limit, offset } = {}) {
  const params = new URLSearchParams();
  if (limit != null) params.set("limit", String(limit));
  if (offset != null) params.set("offset", String(offset));
  const qs = params.toString();
  return qs ? `${path}?${qs}` : path;
}
