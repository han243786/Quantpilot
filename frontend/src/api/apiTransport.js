import { API_BASE } from "./apiBase";

export async function request(
  method,
  path,
  body,
  {
    apiBase = API_BASE,
    fetchImpl = fetch,
    headers = {},
    timeoutMs = 30000,
  } = {}
) {
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
    const response = await fetchImpl(`${apiBase}${path}`, options);
    if (!response.ok) {
      const text = await response.text();
      const error = new Error(
        text.slice(0, 2000) || `服务器错误 (${response.status})`
      );
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

export function createApiClient(options = {}) {
  return {
    get: (path, opts) =>
      request("GET", path, undefined, { ...options, ...opts }),
    post: (path, body, opts) =>
      request("POST", path, body, { ...options, ...opts }),
    del: (path, opts) =>
      request("DELETE", path, undefined, { ...options, ...opts }),
  };
}
