import { describe, expect, it } from "vitest";
import { getCapabilityActionBlockReason } from "./capabilityActionBlocks";
import { createSafeFallbackCapabilities } from "../modules/builtinModules";
import { backendCapabilitiesFixture } from "../test/fixtures/capabilities/capabilityFallbacks";

describe("capability action blocks", () => {
  it("allows trusted supported actions through the action-block gate", () => {
    expect(
      getCapabilityActionBlockReason({
        actionKey: "compile",
        capabilityStatus: "ready",
        capabilitySource: "remote",
        capabilityMessage: "",
        capabilities: backendCapabilitiesFixture
      })
    ).toBe("");
  });

  it("blocks risky actions while capability state is loading or safe fallback", () => {
    expect(
      getCapabilityActionBlockReason({
        actionKey: "compile",
        capabilityStatus: "loading",
        capabilitySource: "remote",
        capabilityMessage: "",
        capabilities: backendCapabilitiesFixture
      })
    ).not.toBe("");

    expect(
      getCapabilityActionBlockReason({
        actionKey: "run_parameter_sweep",
        capabilityStatus: "error",
        capabilitySource: "safe_fallback",
        capabilityMessage: "capability failed",
        capabilities: createSafeFallbackCapabilities("capability failed")
      })
    ).toContain("capability failed");
  });

  it("blocks declared actions when backend action declarations are absent", () => {
    expect(
      getCapabilityActionBlockReason({
        actionKey: "export_quantscript",
        capabilityStatus: "ready",
        capabilitySource: "remote",
        capabilityMessage: "",
        capabilities: null
      })
    ).not.toBe("");
  });
});
