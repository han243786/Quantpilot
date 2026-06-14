import { describe, expect, it } from "vitest";
import { isCapabilitySyncBlocked } from "./capabilitySync";

describe("capabilitySync", () => {
  it("blocks loading and safe fallback states only", () => {
    expect(isCapabilitySyncBlocked("loading", "remote")).toBe(true);
    expect(isCapabilitySyncBlocked("error", "safe_fallback")).toBe(true);
    expect(isCapabilitySyncBlocked("degraded", "cache")).toBe(false);
    expect(isCapabilitySyncBlocked("ready", "remote")).toBe(false);
  });
});
