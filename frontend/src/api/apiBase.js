export function resolveApiBase({
  rawBase = import.meta.env.VITE_API_BASE_URL,
  hasWindow = typeof window !== "undefined",
} = {}) {
  const raw = typeof rawBase === "string" ? rawBase.trim() : "";
  if (raw) return raw.replace(/\/+$/, "");
  if (!hasWindow) return "http://127.0.0.1:3000/api";
  return "/api";
}

export const API_BASE = resolveApiBase();

export function getAuthHeaders() {
  return {};
}
