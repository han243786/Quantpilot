import { describe, expect, it } from "vitest";
import { CAPABILITY_GOVERNANCE_REGISTRY } from "./capabilityGovernanceRegistry";
import {
  CAPABILITY_ACTION_MAP,
  SUPPORTED_RUNTIME_MODES,
  WORKSPACE_SURFACE_MAP
} from "./supportMatrix";

function valuesForFamily(family) {
  return CAPABILITY_GOVERNANCE_REGISTRY
    .filter((entry) => entry.family === family)
    .map((entry) => entry.value);
}

describe("capability governance registry entries", () => {
  it("keeps generated registry entries aligned with capability truth", () => {
    expect(valuesForFamily("runtime_mode")).toEqual(SUPPORTED_RUNTIME_MODES);
    expect(valuesForFamily("ui_action")).toEqual(Object.keys(CAPABILITY_ACTION_MAP));
    expect(valuesForFamily("workspace_surface")).toEqual(Object.keys(WORKSPACE_SURFACE_MAP));
  });

  it("keeps every generated entry reviewable and traceable", () => {
    expect(CAPABILITY_GOVERNANCE_REGISTRY.length).toBeGreaterThan(0);

    for (const entry of CAPABILITY_GOVERNANCE_REGISTRY) {
      expect(entry.id).toContain(".");
      expect(entry.family.length).toBeGreaterThan(0);
      expect(entry.ownerRole.length).toBeGreaterThan(0);
      expect(entry.reviewResponsibility.length).toBeGreaterThan(0);
      expect(entry.sourceOfTruth.length).toBeGreaterThan(0);
    }
  });
});
