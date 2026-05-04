import { describe, expect, it } from "vitest";
import {
  buildGovernanceIdentityRows,
  governanceFromRuntime,
  normalizeRuntimeGovernanceSnapshot
} from "./runtimeGovernance";

describe("runtime governance reader", () => {
  it("normalizes complete governance snapshots without losing identity", () => {
    const governance = normalizeRuntimeGovernanceSnapshot({
      schema_version: "quantpilot/runtime-governance/v1",
      governance_source: "loaded_manifest",
      capability_hash: "sha256:abc",
      strategy_version: "1.0.0",
      parameter_version: "config:123",
      deployment_revision: "sha256:def",
      permission_boundary: {
        model_version: "quantpilot/permission-boundary/v1",
        execution_owner_module: "builtin.execution.paper",
        live_execution_allowed: false,
        ai_write_policy: "proposal_only",
        plugin_network_default: "deny",
        non_execution_order_access: "deny"
      }
    });

    expect(governance.capability_hash).toBe("sha256:abc");
    expect(governance.governance_source).toBe("loaded_manifest");
    expect(governance.permission_boundary.ai_write_policy).toBe("proposal_only");
  });

  it("uses restrictive defaults for partial or missing governance", () => {
    const governance = normalizeRuntimeGovernanceSnapshot({
      capability_hash: "sha256:abc",
      permission_boundary: {
        ai_write_policy: "auto_apply",
        plugin_network_default: "open",
        non_execution_order_access: "read_write",
        live_execution_allowed: "yes"
      }
    });

    expect(governance.governance_source).toBe("legacy_default");
    expect(governance.deployment_revision).toBe("unknown");
    expect(governance.permission_boundary.live_execution_allowed).toBe(false);
    expect(governance.permission_boundary.ai_write_policy).toBe("disabled");
    expect(governance.permission_boundary.plugin_network_default).toBe("deny");
    expect(governance.permission_boundary.non_execution_order_access).toBe("deny");

    expect(normalizeRuntimeGovernanceSnapshot(null).capability_hash).toBe("unknown");
  });

  it("resolves governance from runtime state, manifest, or event envelope", () => {
    expect(
      governanceFromRuntime({
        governance: {
          capability_hash: "sha256:runtime",
          deployment_revision: "sha256:deployment"
        }
      }).capability_hash
    ).toBe("sha256:runtime");

    expect(
      governanceFromRuntime({
        backtestArtifacts: {
          manifest: {
            governance: {
              capability_hash: "sha256:manifest",
              deployment_revision: "sha256:deployment"
            }
          }
        }
      }).capability_hash
    ).toBe("sha256:manifest");

    const fromEnvelope = governanceFromRuntime({
      events: [
        {
          envelope: {
            capability_hash: "sha256:event",
            deployment_revision: "sha256:deployment",
            strategy_version: "1.0.0",
            parameter_version: "config:abc"
          }
        }
      ]
    });
    expect(fromEnvelope.capability_hash).toBe("sha256:event");
    expect(fromEnvelope.governance_source).toBe("event_envelope");
  });

  it("builds display rows with full hash metadata preserved", () => {
    const rows = buildGovernanceIdentityRows({
      capability_hash: "sha256:1234567890abcdef1234567890abcdef",
      deployment_revision: "sha256:abcdef1234567890abcdef1234567890",
      strategy_version: "1.0.0",
      parameter_version: "config:abcdef1234567890",
      governance_source: "loaded_manifest"
    });

    expect(rows.map((row) => row.key)).toContain("capability_hash");
    expect(rows.find((row) => row.key === "capability_hash").fullValue).toBe(
      "sha256:1234567890abcdef1234567890abcdef"
    );
    expect(rows.find((row) => row.key === "governance_source").value).toBe(
      "loaded_manifest"
    );
  });
});
