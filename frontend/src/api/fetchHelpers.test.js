import { afterEach, describe, expect, it, vi } from "vitest";
import { fetchWithTimeout } from "./fetchHelpers";

describe("fetchHelpers", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.useRealTimers();
  });

  it("forwards request options and attaches an abort signal", async () => {
    const response = new Response("ok");
    const fetchImpl = vi.fn(async () => response);
    vi.stubGlobal("fetch", fetchImpl);

    await expect(
      fetchWithTimeout("/api/compile", {
        method: "POST",
        headers: { "X-Trace": "trace-1" },
      })
    ).resolves.toBe(response);

    expect(fetchImpl).toHaveBeenCalledWith(
      "/api/compile",
      expect.objectContaining({
        headers: { "X-Trace": "trace-1" },
        method: "POST",
        signal: expect.any(AbortSignal),
      })
    );
  });

  it("aborts requests when the timeout elapses", async () => {
    vi.useFakeTimers();
    const abortError = new Error("aborted");
    abortError.name = "AbortError";
    const fetchImpl = vi.fn(
      (_url, { signal }) =>
        new Promise((_resolve, reject) => {
          signal.addEventListener("abort", () => reject(abortError));
        })
    );
    vi.stubGlobal("fetch", fetchImpl);

    const request = expect(
      fetchWithTimeout("/api/slow", {}, 50)
    ).rejects.toMatchObject({ name: "AbortError" });
    await vi.advanceTimersByTimeAsync(50);

    await request;
  });
});
