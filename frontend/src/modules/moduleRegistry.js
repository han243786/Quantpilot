import { validateExternalModuleMetadata } from "./moduleRegistryContracts";

export {
  PLUGIN_CAPABILITY_CONTRACTS,
  PLUGIN_CAPABILITY_CONTRACT_V1_VERSION,
  PLUGIN_MANIFEST_V1_VERSION,
  validateExternalModuleMetadata
} from "./moduleRegistryContracts";

export function loadExternalModuleMetadata(externalModuleMetadata = [], capabilities = null) {
  const supportedModuleKeys = new Set(capabilities?.frontend?.supported_module_keys || []);
  const unsupportedModuleReasons = capabilities?.frontend?.unsupported_module_reasons || {};
  const validationErrors = [];
  const marketplaceEntries = [];
  const activeModules = [];
  const manifestsByModuleKey = {};
  const pluginIds = new Set();
  const moduleKeys = new Set();

  externalModuleMetadata.forEach((entry, index) => {
    const errors = validateExternalModuleMetadata(entry);
    if (errors.length > 0) {
      validationErrors.push(...errors.map((message) => `externalModuleMetadata[${index}]: ${message}`));
      return;
    }

    const pluginId = entry.manifest.id;
    const moduleKey = entry.module.module_key;
    if (pluginIds.has(pluginId)) {
      validationErrors.push(`externalModuleMetadata[${index}]: duplicate plugin id \`${pluginId}\``);
      return;
    }
    if (moduleKeys.has(moduleKey)) {
      validationErrors.push(`externalModuleMetadata[${index}]: duplicate module key \`${moduleKey}\``);
      return;
    }
    pluginIds.add(pluginId);
    moduleKeys.add(moduleKey);

    const supported =
      supportedModuleKeys.size === 0 ? true : supportedModuleKeys.has(moduleKey);
    const reason = supported ? null : unsupportedModuleReasons[moduleKey] || "unsupported by current backend capability boundary";
    const marketplaceEntry = {
      manifest: entry.manifest,
      module: entry.module,
      supported,
      reason
    };
    marketplaceEntries.push(marketplaceEntry);
    manifestsByModuleKey[moduleKey] = entry.manifest;
    if (supported) {
      activeModules.push(entry.module);
    }
  });

  return {
    marketplaceEntries,
    activeModules,
    manifestsByModuleKey,
    validationErrors
  };
}

export function createModuleRegistry(modules, capabilities = null, externalModuleMetadata = []) {
  const {
    marketplaceEntries,
    activeModules,
    manifestsByModuleKey,
    validationErrors
  } = loadExternalModuleMetadata(externalModuleMetadata, capabilities);
  const allModules = [...modules, ...activeModules];
  const byKey = {};

  allModules.forEach((moduleDef) => {
    byKey[moduleDef.module_key] = moduleDef;
  });

  return {
    capabilities,
    validationErrors,
    getAll() {
      return allModules;
    },
    getByKey(moduleKey) {
      return byKey[moduleKey] || null;
    },
    getByCategory(category) {
      return allModules.filter((item) => item.category === category);
    },
    getMarketplaceEntries() {
      return marketplaceEntries;
    },
    getExternalModules() {
      return marketplaceEntries.map((entry) => entry.module);
    },
    getPluginManifest(moduleKey) {
      return manifestsByModuleKey[moduleKey] || null;
    }
  };
}
