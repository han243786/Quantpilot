import { API_BASE } from "../api/client";
import { fetchWithTimeout } from "../utils/api";
import { humanizeErrorText } from "../utils/errorText";

export { API_BASE };

export async function fetchJson(path) {
  const response = await fetchWithTimeout(`${API_BASE}${path}`);
  if (!response.ok) {
    const text = await response.text();
    throw new Error(humanizeErrorText(text, `Request failed with status ${response.status}.`));
  }
  return response.json();
}

export function unwrapPage(json) {
  if (json && typeof json === "object" && Array.isArray(json.data) && typeof json.total === "number") {
    return json.data;
  }
  return json;
}

export async function postJson(path, body) {
  const response = await fetchWithTimeout(`${API_BASE}${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body)
  });

  if (!response.ok) {
    const text = await response.text();
    let payload = null;
    try {
      payload = JSON.parse(text);
    } catch (e) {
      console.warn("graphStorePersistenceHelpers: postJson parse failed", e);
    }

    const error = new Error(
      humanizeErrorText(
        payload?.message || text,
        `Request failed with status ${response.status}.`
      )
    );
    error.status = response.status;
    error.error = payload?.error || null;
    error.details = Array.isArray(payload?.details) ? payload.details : [];
    error.partial_artifacts = payload?.partial_artifacts || null;
    throw error;
  }

  const json = await response.json();
  return json;
}

export async function deleteJson(path) {
  const response = await fetchWithTimeout(`${API_BASE}${path}`, {
    method: "DELETE"
  });

  if (!response.ok) {
    const text = await response.text();
    let payload = null;
    try {
      payload = JSON.parse(text);
    } catch (e) {
      console.warn("graphStorePersistenceHelpers: deleteJson parse failed", e);
    }

    const error = new Error(
      humanizeErrorText(
        payload?.message || text,
        `Request failed with status ${response.status}.`
      )
    );
    error.status = response.status;
    error.error = payload?.error || null;
    error.details = Array.isArray(payload?.details) ? payload.details : [];
    throw error;
  }

  const json = await response.json();
  return json;
}
