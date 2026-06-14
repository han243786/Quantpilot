import { describe, expect, it } from "vitest";
import {
  CAPABILITY_ACTION_MAP,
  DECLARED_INDICATOR_KINDS,
  SUPPORTED_FRONTEND_MODULE_KEYS,
  SUPPORTED_SYMBOLS,
  WORKSPACE_SURFACE_MAP
} from "./supportMatrix";
import { DEFAULT_CAPABILITIES, createSafeFallbackCapabilities } from "./builtinCapabilitySnapshot";

describe("builtin capability snapshot", () => {
  it("keeps the default snapshot aligned with support-matrix truth", () => {
    expect(DEFAULT_CAPABILITIES.strategy_ir.declared_indicator_kinds).toEqual(
      DECLARED_INDICATOR_KINDS
    );
    expect(DEFAULT_CAPABILITIES.market_data.supported_symbols).toEqual(SUPPORTED_SYMBOLS);
    expect(DEFAULT_CAPABILITIES.frontend.supported_module_keys).toEqual(
      SUPPORTED_FRONTEND_MODULE_KEYS
    );
    expect(DEFAULT_CAPABILITIES.workspace.surfaces.map((entry) => entry.key)).toEqual(
      Object.keys(WORKSPACE_SURFACE_MAP)
    );
    expect(DEFAULT_CAPABILITIES.ui_actions.actions.map((entry) => entry.key)).toEqual(
      Object.keys(CAPABILITY_ACTION_MAP)
    );
  });

  it("creates a safe fallback snapshot that keeps risky surfaces declared-only", () => {
    const fallback = createSafeFallbackCapabilities("capability unavailable");

    expect(fallback.schema_hash).toBe("safe-fallback");
    expect(fallback.permission_boundary.ai_write_policy).toBe("disabled");
    expect(fallback.ui_actions.actions.every((entry) => entry.status === "declared_only")).toBe(
      true
    );
    expect(fallback.workspace.surfaces.every((entry) => entry.source === "safe_fallback")).toBe(
      true
    );
  });
});
