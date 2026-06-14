import { describe, expect, it } from "vitest";
import {
  PLUGIN_CAPABILITY_CONTRACT_V1_VERSION,
  PLUGIN_MANIFEST_V1_VERSION,
  validateExternalModuleMetadata
} from "./moduleRegistryContracts";

function createExternalEntry() {
  return {
    manifest: {
      api_version: PLUGIN_MANIFEST_V1_VERSION,
      id: "quantpilot.intent.custom_expr",
      version: "0.1.0",
      kind: "intent",
      display: {
        name: "Custom Expr"
      },
      capability_declarations: [
        {
          id: "quantpilot.capability.intent_module_provider",
          version: PLUGIN_CAPABILITY_CONTRACT_V1_VERSION
        }
      ],
      extension_points: ["intent_module_provider"]
    },
    module: {
      module_key: "plugin.intent.custom_expr",
      category: "intent",
      display_name: "Custom Expr",
      node: {},
      ports: {},
      config_schema: {}
    }
  };
}

describe("module registry contracts", () => {
  it("accepts metadata that declares the expected plugin contract", () => {
    expect(validateExternalModuleMetadata(createExternalEntry())).toEqual([]);
  });

  it("rejects mismatched capability contract versions", () => {
    const entry = createExternalEntry();
    entry.manifest.capability_declarations[0].version = "v0";

    expect(validateExternalModuleMetadata(entry)).toContain(
      "capability `quantpilot.capability.intent_module_provider` must use version `v1`"
    );
  });
});
