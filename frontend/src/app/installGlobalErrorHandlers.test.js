import { afterEach, describe, expect, it, vi } from "vitest";
import { installGlobalErrorHandlers } from "./installGlobalErrorHandlers";

describe("installGlobalErrorHandlers", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("logs unhandled rejection reasons", () => {
    const errorSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    const reason = new Error("bootstrap failure");
    const event = new Event("unhandledrejection");
    Object.defineProperty(event, "reason", { value: reason });

    installGlobalErrorHandlers();
    window.dispatchEvent(event);

    expect(errorSpy).toHaveBeenCalledWith("[UnhandledRejection]", reason);
  });
});
