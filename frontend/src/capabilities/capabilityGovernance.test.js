import { describe, expect, it } from "vitest";
import {
  CAPABILITY_ACTION_MAP,
  DECLARED_INDICATOR_KINDS,
  SUPPORTED_EXCHANGES,
  SUPPORTED_FRONTEND_MODULE_KEYS,
  SUPPORTED_RUNTIME_EXECUTION_MODULES,
  SUPPORTED_RUNTIME_MODES,
  SUPPORTED_SYMBOLS,
  SUPPORT_MATRIX,
  WORKSPACE_SURFACE_MAP
} from "./supportMatrix";
import {
  CAPABILITY_CLASSES,
  CAPABILITY_GOVERNANCE,
  CAPABILITY_GOVERNANCE_REGISTRY,
  CAPABILITY_GOVERNANCE_SCHEMA_VERSION,
  findCapabilityGovernanceEntry
} from "./capabilityGovernance";

function valuesForFamily(family) {
  return CAPABILITY_GOVERNANCE_REGISTRY
    .filter((entry) => entry.family === family)
    .map((entry) => entry.value);
}

describe("capability governance registry", () => {
  it("exposes a machine-readable schema version and registry", () => {
    expect(CAPABILITY_GOVERNANCE.schemaVersion).toBe(CAPABILITY_GOVERNANCE_SCHEMA_VERSION);
    expect(Array.isArray(CAPABILITY_GOVERNANCE.registry)).toBe(true);
    expect(CAPABILITY_GOVERNANCE.registry.length).toBeGreaterThan(0);
  });

  it("classifies every support-matrix runtime and market entry", () => {
    expect(valuesForFamily("runtime_mode")).toEqual(SUPPORTED_RUNTIME_MODES);
    expect(valuesForFamily("execution_module")).toEqual(SUPPORTED_RUNTIME_EXECUTION_MODULES);
    expect(valuesForFamily("exchange")).toEqual(SUPPORTED_EXCHANGES);
    expect(valuesForFamily("symbol")).toEqual(SUPPORTED_SYMBOLS);
  });

  it("classifies every declared indicator, frontend module, UI action, and workspace surface", () => {
    expect(valuesForFamily("strategy_ir_indicator_kind")).toEqual(DECLARED_INDICATOR_KINDS);
    expect(valuesForFamily("frontend_module")).toEqual(SUPPORTED_FRONTEND_MODULE_KEYS);
    expect(valuesForFamily("ui_action")).toEqual(Object.keys(CAPABILITY_ACTION_MAP));
    expect(valuesForFamily("workspace_surface")).toEqual(Object.keys(WORKSPACE_SURFACE_MAP));
  });

  it("keeps claim governance aligned with the support matrix guardrails", () => {
    const allowedClaims = CAPABILITY_GOVERNANCE_REGISTRY.filter(
      (entry) =>
        entry.family === "user_facing_claim" && entry.class === CAPABILITY_CLASSES.supported
    ).map((entry) => entry.value);
    const disallowedClaims = CAPABILITY_GOVERNANCE_REGISTRY.filter(
      (entry) =>
        entry.family === "user_facing_claim" &&
        entry.class === CAPABILITY_CLASSES.disallowed_claim
    ).map((entry) => entry.value);

    expect(allowedClaims).toEqual(SUPPORT_MATRIX.userFacingGuardrails.allowedClaims);
    expect(disallowedClaims).toEqual(SUPPORT_MATRIX.userFacingGuardrails.disallowedClaims);
  });

  it("attaches approved phrases to every allowed claim", () => {
    const allowedClaims = CAPABILITY_GOVERNANCE_REGISTRY.filter(
      (entry) => entry.family === "user_facing_claim" && entry.class === CAPABILITY_CLASSES.supported
    );

    expect(allowedClaims.length).toBeGreaterThan(0);
    for (const entry of allowedClaims) {
      expect(entry.textGate).toBeTruthy();
      expect(entry.textGate.approvedPhrase).toBe(entry.value);
    }
  });

  it("attaches text-gate metadata to every disallowed claim", () => {
    const disallowedClaims = CAPABILITY_GOVERNANCE_REGISTRY.filter(
      (entry) => entry.family === "user_facing_claim" && entry.class === CAPABILITY_CLASSES.disallowed_claim
    );

    expect(disallowedClaims.length).toBeGreaterThan(0);
    for (const entry of disallowedClaims) {
      expect(entry.textGate).toBeTruthy();
      expect(typeof entry.textGate.forbiddenPattern).toBe("string");
      expect(entry.textGate.forbiddenPattern.length).toBeGreaterThan(0);
      expect(typeof entry.textGate.detail).toBe("string");
      expect(entry.textGate.detail.length).toBeGreaterThan(0);
      expect(typeof entry.textGate.allowedContextPattern).toBe("string");
      expect(entry.textGate.allowedContextPattern.length).toBeGreaterThan(0);
    }
  });

  it("defines a scoped positive-claim audit policy", () => {
    expect(Array.isArray(CAPABILITY_GOVERNANCE.textGates.positiveClaimAudit.scopedPaths)).toBe(true);
    expect(CAPABILITY_GOVERNANCE.textGates.positiveClaimAudit.scopedPaths.length).toBeGreaterThan(0);
    expect(Array.isArray(CAPABILITY_GOVERNANCE.textGates.positiveClaimAudit.positiveStatementPatterns)).toBe(
      true
    );
    expect(
      CAPABILITY_GOVERNANCE.textGates.positiveClaimAudit.positiveStatementPatterns.length
    ).toBeGreaterThan(0);
    expect(typeof CAPABILITY_GOVERNANCE.textGates.positiveClaimAudit.allowedContextPattern).toBe(
      "string"
    );
  });

  it("marks restricted and trace-only examples with the intended classes", () => {
    expect(findCapabilityGovernanceEntry("strategy_ir.indicator.custom")?.class).toBe(
      CAPABILITY_CLASSES.restricted
    );
    expect(findCapabilityGovernanceEntry("strategy_ir.indicator.spread")?.class).toBe(
      CAPABILITY_CLASSES.restricted
    );
    expect(findCapabilityGovernanceEntry("frontend.module.builtin.intent.spread_observer")?.class).toBe(
      CAPABILITY_CLASSES.restricted
    );
    expect(findCapabilityGovernanceEntry("frontend.module.builtin.agent.arbitrage")?.class).toBe(
      CAPABILITY_CLASSES.trace_only
    );
    expect(findCapabilityGovernanceEntry("workspace.surface.parameter_sweep")?.class).toBe(
      CAPABILITY_CLASSES.restricted
    );
    expect(findCapabilityGovernanceEntry("compile.strategy_ir_preflight")?.class).toBe(
      CAPABILITY_CLASSES.restricted
    );
  });

  it("attaches owner and review metadata to every governance entry", () => {
    for (const entry of CAPABILITY_GOVERNANCE_REGISTRY) {
      expect(typeof entry.id).toBe("string");
      expect(typeof entry.family).toBe("string");
      expect(typeof entry.ownerRole).toBe("string");
      expect(entry.ownerRole.length).toBeGreaterThan(0);
      expect(typeof entry.reviewResponsibility).toBe("string");
      expect(entry.reviewResponsibility.length).toBeGreaterThan(0);
      expect(typeof entry.sourceOfTruth).toBe("string");
      expect(entry.sourceOfTruth.length).toBeGreaterThan(0);
    }
  });
});
