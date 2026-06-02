import { describe, expect, it } from "vitest";

import { loadExternalModuleMetadata } from "./moduleRegistryAssembly";

function createExternalEntry(moduleKey = "plugin.intent.custom_expr", pluginId = "quantpilot.intent.custom_expr") {
  return {
    manifest: {
      api_version: "quantpilot/plugin-manifest/v1",
      id: pluginId,
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

describe("loadExternalModuleMetadata", () => {
  it("assembles supported metadata into active modules, marketplace entries, and manifest lookup", () => {
    const result = loadExternalModuleMetadata(
      [createExternalEntry()],
      {
        frontend: {
          supported_module_keys: ["plugin.intent.custom_expr"],
          unsupported_module_reasons: {}
        }
      }
    );

    expect(result.validationErrors).toEqual([]);
    expect(result.activeModules.map((item) => item.module_key)).toEqual(["plugin.intent.custom_expr"]);
    expect(result.marketplaceEntries).toHaveLength(1);
    expect(result.marketplaceEntries[0].supported).toBe(true);
    expect(result.manifestsByModuleKey["plugin.intent.custom_expr"]?.id).toBe("quantpilot.intent.custom_expr");
  });

  it("keeps unsupported metadata visible in marketplace but out of active modules", () => {
    const result = loadExternalModuleMetadata(
      [createExternalEntry()],
      {
        frontend: {
          supported_module_keys: ["builtin.data.kline"],
          unsupported_module_reasons: {
            "plugin.intent.custom_expr": "plugin marketplace is not yet activated"
          }
        }
      }
    );

    expect(result.validationErrors).toEqual([]);
    expect(result.activeModules).toEqual([]);
    expect(result.marketplaceEntries[0].supported).toBe(false);
    expect(result.marketplaceEntries[0].reason).toBe("plugin marketplace is not yet activated");
  });

  it("reports duplicate plugin and module keys before activation", () => {
    const duplicatePlugin = loadExternalModuleMetadata([
      createExternalEntry(),
      createExternalEntry("plugin.intent.other", "quantpilot.intent.custom_expr")
    ]);
    const duplicateModule = loadExternalModuleMetadata([
      createExternalEntry(),
      createExternalEntry("plugin.intent.custom_expr", "quantpilot.intent.other")
    ]);

    expect(duplicatePlugin.validationErrors).toContain(
      "externalModuleMetadata[1]: duplicate plugin id `quantpilot.intent.custom_expr`"
    );
    expect(duplicateModule.validationErrors).toContain(
      "externalModuleMetadata[1]: duplicate module key `plugin.intent.custom_expr`"
    );
  });
});
