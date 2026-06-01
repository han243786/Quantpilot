import { describe, expect, it, vi } from "vitest";
import { createApiClient, request } from "./apiTransport";

describe("apiTransport", () => {
  it("sends JSON GET requests through the configured API base", async () => {
    const fetchImpl = vi.fn(async () =>
      new Response(JSON.stringify({ ok: true }), {
        headers: { "content-type": "application/json" },
      })
    );

    await expect(
      request("GET", "/health", undefined, {
        apiBase: "http://localhost:7897/api",
        fetchImpl,
      })
    ).resolves.toEqual({ ok: true });

    expect(fetchImpl).toHaveBeenCalledWith(
      "http://localhost:7897/api/health",
      expect.objectContaining({
        headers: { "Content-Type": "application/json" },
        method: "GET",
        signal: expect.any(AbortSignal),
      })
    );
  });

  it("serializes request bodies and merges headers", async () => {
    const fetchImpl = vi.fn(async () =>
      new Response("accepted", {
        headers: { "content-type": "text/plain" },
      })
    );

    await expect(
      request("POST", "/graphs/save", { graph: "draft" }, {
        apiBase: "/api",
        fetchImpl,
        headers: { "X-Trace": "trace-1" },
      })
    ).resolves.toBe("accepted");

    expect(fetchImpl).toHaveBeenCalledWith(
      "/api/graphs/save",
      expect.objectContaining({
        body: JSON.stringify({ graph: "draft" }),
        headers: {
          "Content-Type": "application/json",
          "X-Trace": "trace-1",
        },
        method: "POST",
      })
    );
  });

  it("attaches response status to failed requests", async () => {
    const fetchImpl = vi.fn(async () =>
      new Response("backend unavailable", { status: 503 })
    );

    await expect(
      request("DELETE", "/graphs/draft", undefined, {
        apiBase: "/api",
        fetchImpl,
      })
    ).rejects.toMatchObject({
      message: "backend unavailable",
      status: 503,
    });
  });

  it("creates method-specific client helpers", async () => {
    const fetchImpl = vi.fn(async () =>
      new Response(JSON.stringify({ deleted: true }), {
        headers: { "content-type": "application/json" },
      })
    );
    const client = createApiClient({ apiBase: "/api", fetchImpl });

    await expect(client.del("/graphs/draft")).resolves.toEqual({
      deleted: true,
    });
    expect(fetchImpl).toHaveBeenCalledWith(
      "/api/graphs/draft",
      expect.objectContaining({ method: "DELETE" })
    );
  });
});
