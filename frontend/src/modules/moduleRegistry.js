import { loadExternalModuleMetadata } from "./moduleRegistryAssembly";

export {
  PLUGIN_CAPABILITY_CONTRACTS,
  PLUGIN_CAPABILITY_CONTRACT_V1_VERSION,
  PLUGIN_MANIFEST_V1_VERSION,
  validateExternalModuleMetadata
} from "./moduleRegistryContracts";
export { loadExternalModuleMetadata } from "./moduleRegistryAssembly";

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
