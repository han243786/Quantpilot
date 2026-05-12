// v1.0.5: API_BASE 来自 src/api/client.js — 全项目唯一来源
import { API_BASE as CLIENT_API_BASE, getAuthHeaders as _getAuthHeaders } from "../api/client";

export const API_BASE = CLIENT_API_BASE;

export function fetchWithTimeout(url, options = {}, timeoutMs = 30000) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  return fetch(url, { ...options, signal: controller.signal }).finally(() => clearTimeout(timer));
}

export const getAuthHeaders = _getAuthHeaders;
