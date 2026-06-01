import { describe, expect, it } from "vitest";
import { DEFAULT_CAPABILITIES } from "./builtinCapabilitySnapshot";
import { normalizeCapabilities } from "./capabilityNormalization";
import { backendCapabilitiesFixture } from "../test/fixtures/capabilities/capabilityFallbacks";

describe("capability normalization", () => {
  it("returns the default capability snapshot for invalid input", () => {
    expect(normalizeCapabilities(null)).toBe(DEFAULT_CAPABILITIES);
  });

  it("normalizes unsafe permission boundary values to the safest behavior", () => {
    const normalized = normalizeCapabilities({
      ...backendCapabilitiesFixture,
      permission_boundary: {
        model_version: "quantpilot/permission-boundary/v1",
        execution_owner_module: "builtin.execution.paper",
        live_execution_allowed: "yes",
        ai_write_policy: "auto_apply",
        plugin_network_default: "open",
        non_execution_order_access: "read_write"
      }
    });

    expect(normalized.permission_boundary.live_execution_allowed).toBe(false);
    expect(normalized.permission_boundary.ai_write_policy).toBe("disabled");
    expect(normalized.permission_boundary.plugin_network_default).toBe("deny");
    expect(normalized.permission_boundary.non_execution_order_access).toBe("deny");
  });

  it("uses known module keys when deriving frontend module support", () => {
    const normalized = normalizeCapabilities(
      {
        frontend: {
          supported_module_keys: ["builtin.data.kline"],
          unsupported_module_reasons: {
            "builtin.intent.experimental": "not released"
          }
        }
      },
      {
        knownModuleKeys: ["builtin.data.kline", "builtin.intent.experimental"]
      }
    );

    expect(normalized.frontend.declared_module_keys).toEqual([
      "builtin.data.kline",
      "builtin.intent.experimental"
    ]);
    expect(normalized.frontend.supported_module_keys).toEqual(["builtin.data.kline"]);
    expect(normalized.frontend.module_support).toContainEqual({
      module_key: "builtin.intent.experimental",
      status: "declared_only",
      reason: "not released"
    });
  });
});
