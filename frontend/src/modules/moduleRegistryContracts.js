export const PLUGIN_MANIFEST_V1_VERSION = "quantpilot/plugin-manifest/v1";
export const PLUGIN_CAPABILITY_CONTRACT_V1_VERSION = "v1";

export const PLUGIN_CAPABILITY_CONTRACTS = Object.freeze({
  data: "quantpilot.capability.data_module_provider",
  intent: "quantpilot.capability.intent_module_provider",
  agent: "quantpilot.capability.agent_module_provider",
  risk: "quantpilot.capability.risk_checker_provider",
  execution: "quantpilot.capability.execution_module_provider"
});

const EXTENSION_POINT_BY_CATEGORY = Object.freeze({
  data: "data_module_provider",
  intent: "intent_module_provider",
  agent: "agent_module_provider",
  risk: "risk_checker_provider",
  execution: "execution_module_provider"
});

function validatePluginManifest(manifest, moduleCategory) {
  const errors = [];
  if (!manifest || typeof manifest !== "object") {
    return ["plugin manifest is required"];
  }
  if (manifest.api_version !== PLUGIN_MANIFEST_V1_VERSION) {
    errors.push(`manifest.api_version must be \`${PLUGIN_MANIFEST_V1_VERSION}\``);
  }
  if (!manifest.id || typeof manifest.id !== "string") {
    errors.push("manifest.id is required");
  }
  if (!manifest.version || typeof manifest.version !== "string") {
    errors.push("manifest.version is required");
  }
  if (!manifest.kind || typeof manifest.kind !== "string") {
    errors.push("manifest.kind is required");
  }
  if (!manifest.display || typeof manifest.display.name !== "string" || !manifest.display.name.trim()) {
    errors.push("manifest.display.name is required");
  }
  if (!Array.isArray(manifest.extension_points) || manifest.extension_points.length === 0) {
    errors.push("manifest.extension_points must contain at least one entry");
  }
  if (!Array.isArray(manifest.capability_declarations) || manifest.capability_declarations.length === 0) {
    errors.push("manifest.capability_declarations must contain at least one entry");
  }

  const expectedExtensionPoint = EXTENSION_POINT_BY_CATEGORY[moduleCategory];
  if (expectedExtensionPoint && !manifest.extension_points?.includes(expectedExtensionPoint)) {
    errors.push(`manifest.extension_points must include \`${expectedExtensionPoint}\``);
  }

  const expectedCapability = PLUGIN_CAPABILITY_CONTRACTS[moduleCategory];
  if (expectedCapability) {
    const declarations = manifest.capability_declarations || [];
    const capability = declarations.find((item) => item?.id === expectedCapability);
    if (!capability) {
      errors.push(`manifest.capability_declarations must include \`${expectedCapability}\``);
    } else if (capability.version !== PLUGIN_CAPABILITY_CONTRACT_V1_VERSION) {
      errors.push(
        `capability \`${expectedCapability}\` must use version \`${PLUGIN_CAPABILITY_CONTRACT_V1_VERSION}\``
      );
    }
  }

  return errors;
}

function validateModuleDefinition(moduleDef) {
  const errors = [];
  if (!moduleDef || typeof moduleDef !== "object") {
    return ["external module metadata requires a module definition"];
  }
  if (!moduleDef.module_key || typeof moduleDef.module_key !== "string") {
    errors.push("module.module_key is required");
  }
  if (!moduleDef.category || typeof moduleDef.category !== "string") {
    errors.push("module.category is required");
  }
  if (!moduleDef.display_name || typeof moduleDef.display_name !== "string") {
    errors.push("module.display_name is required");
  }
  if (!moduleDef.node || typeof moduleDef.node !== "object") {
    errors.push("module.node is required");
  }
  if (!moduleDef.ports || typeof moduleDef.ports !== "object") {
    errors.push("module.ports is required");
  }
  if (!moduleDef.config_schema || typeof moduleDef.config_schema !== "object") {
    errors.push("module.config_schema is required");
  }
  return errors;
}

export function validateExternalModuleMetadata(entry) {
  const manifestErrors = validatePluginManifest(entry?.manifest, entry?.module?.category);
  const moduleErrors = validateModuleDefinition(entry?.module);
  return [...manifestErrors, ...moduleErrors];
}
