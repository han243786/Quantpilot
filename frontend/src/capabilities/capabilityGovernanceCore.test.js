import { describe, expect, it } from "vitest";
import {
  CAPABILITY_CLASSES,
  CAPABILITY_GOVERNANCE_SCHEMA_VERSION,
  CAPABILITY_OWNER_ROLES,
  CAPABILITY_TEXT_GATES,
  buildCapabilityGovernanceEntry
} from "./capabilityGovernanceCore";

describe("capability governance core", () => {
  it("exposes the stable governance schema and class vocabulary", () => {
    expect(CAPABILITY_GOVERNANCE_SCHEMA_VERSION).toBe("quantpilot/capability-governance/v1");
    expect(CAPABILITY_CLASSES).toEqual({
      supported: "supported",
      restricted: "restricted",
      trace_only: "trace_only",
      disallowed_claim: "disallowed_claim"
    });
  });

  it("keeps owner roles and text gates machine-readable", () => {
    expect(CAPABILITY_OWNER_ROLES.frontend_editor_owner).toBe("frontend editor owner");
    expect(CAPABILITY_TEXT_GATES.positiveClaimAudit.scopedPaths).toContain(
      "frontend/src/components/TopToolbar.jsx"
    );
    expect(CAPABILITY_TEXT_GATES.positiveClaimAudit.positiveStatementPatterns.length).toBeGreaterThan(
      0
    );
  });

  it("builds normalized governance entries", () => {
    expect(
      buildCapabilityGovernanceEntry({
        id: "runtime.mode.paper",
        family: "runtime_mode",
        value: "paper",
        className: CAPABILITY_CLASSES.supported,
        ownerRole: CAPABILITY_OWNER_ROLES.backend_runtime_owner,
        reviewResponsibility: "runtime contract",
        sourceOfTruth: "backend:/api/capabilities.runtime.supported_modes",
        notes: ["example"],
        textGate: { approvedPhrase: "paper" }
      })
    ).toEqual({
      id: "runtime.mode.paper",
      family: "runtime_mode",
      value: "paper",
      class: "supported",
      ownerRole: "backend runtime owner",
      reviewResponsibility: "runtime contract",
      sourceOfTruth: "backend:/api/capabilities.runtime.supported_modes",
      notes: ["example"],
      textGate: { approvedPhrase: "paper" }
    });
  });
});
