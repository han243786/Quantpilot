import { describe, expect, it } from "vitest";

import { createModuleRegistry } from "./moduleRegistry";

const builtinModules = [
  {
    module_key: "builtin.data.kline",
    category: "data",
    display_name: "Kline",
    node: {},
    ports: {},
    config_schema: {}
  }
];

function createExternalEntry(moduleKey = "plugin.intent.custom_expr") {
  return {
    manifest: {
      api_version: "quantpilot/plugin-manifest/v1",
      id: "quantpilot.intent.custom_expr",
      version: "0.1.0",
      kind: "intent",
      display: {
        name: "Custom Expr",
        summary: "Restricted custom expression plugin"
      },
      capability_declarations: [
        {
          id: "quantpilot.capability.intent_module_provider",
          version: "v1"
        }
      ],
      extension_points: ["intent_module_provider"],
      execution: {
        engine: "builtin",
        entrypoint: "builtin.custom_expr"
      },
      compatibility: {
        core_ir_version: "quantpilot/core-ir/v1",
        capability_api_version: "quantpilot-capabilities/v1"
      },
      security: {
        max_compute_ms: 50,
        max_memory_mb: 64,
        allow_network: false
      },
      dependencies: []
    },
    module: {
      module_key: moduleKey,
      category: "intent",
      display_name: "Custom Expr",
      description: "Restricted custom expression plugin",
      node: {},
      ports: {},
      config_schema: {}
    }
  };
}

describe("createModuleRegistry", () => {
  it("loads validated external metadata into marketplace entries", () => {
    const registry = createModuleRegistry(
      builtinModules,
      {
        frontend: {
          supported_module_keys: ["builtin.data.kline", "plugin.intent.custom_expr"],
          unsupported_module_reasons: {}
        }
      },
      [createExternalEntry()]
    );

    expect(registry.validationErrors).toEqual([]);
    expect(registry.getByKey("plugin.intent.custom_expr")).not.toBeNull();
    expect(registry.getPluginManifest("plugin.intent.custom_expr")?.id).toBe(
      "quantpilot.intent.custom_expr"
    );
    expect(registry.getMarketplaceEntries()).toHaveLength(1);
    expect(registry.getMarketplaceEntries()[0].supported).toBe(true);
  });

  it("rejects invalid plugin schema before registration", () => {
    const invalid = createExternalEntry();
    invalid.manifest.capability_declarations = [];

    const registry = createModuleRegistry(builtinModules, null, [invalid]);

    expect(registry.getByKey("plugin.intent.custom_expr")).toBeNull();
    expect(registry.validationErrors).toContain(
      "externalModuleMetadata[0]: manifest.capability_declarations must contain at least one entry"
    );
  });

  it("keeps unsupported external metadata out of active modules while preserving catalog entry", () => {
    const registry = createModuleRegistry(
      builtinModules,
      {
        frontend: {
          supported_module_keys: ["builtin.data.kline"],
          unsupported_module_reasons: {
            "plugin.intent.custom_expr": "plugin marketplace is not yet activated"
          }
        }
      },
      [createExternalEntry()]
    );

    expect(registry.getByKey("plugin.intent.custom_expr")).toBeNull();
    expect(registry.getMarketplaceEntries()).toHaveLength(1);
    expect(registry.getMarketplaceEntries()[0].supported).toBe(false);
    expect(registry.getMarketplaceEntries()[0].reason).toBe(
      "plugin marketplace is not yet activated"
    );
  });
});
