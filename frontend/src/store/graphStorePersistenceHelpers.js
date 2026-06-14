import {
  defaultRegistry,
  resolveLoadedGraphWithRegistry
} from "./graphStoreGraphShapeValidation";
import { fetchJson } from "./graphStorePersistenceTransport";

export {
  API_BASE,
  deleteJson,
  fetchJson,
  postJson,
  unwrapPage
} from "./graphStorePersistenceTransport";

export {
  CAPABILITY_CACHE_KEY,
  STORAGE_KEY,
  loadCapabilitiesFromCache,
  loadGraphFromStorage,
  saveCapabilitiesToCache,
  saveGraphToStorage
} from "./graphStorePersistenceStorage";

export {
  resolveGraphActor,
  withGraphActorMetadata
} from "./graphStoreActorCollaboration";

export {
  createSafeFallbackCapabilities,
  defaultCapabilities,
  defaultRegistry,
  fallbackRunnableGraph,
  hasUsableGraphShape,
  isDeprecatedBuiltinSampleGraph,
  normalizeGraphShape,
  recordRecentNodeIds,
  resolveLoadedGraph,
  resolveLoadedGraphWithRegistry,
  withRecentNodeIds,
  attachValidationWithRegistry,
  buildRegistryFromCapabilities
} from "./graphStoreGraphShapeValidation";

export {
  graphExistsInIndex,
  normalizeGraphAuditHistory,
  normalizeGraphIndex,
  normalizeGraphVersionCompare,
  normalizeGraphVersions
} from "./graphStoreVersionAuditNormalizers";

async function resolveGraphForDetail(graphId, fallbackGraph, registry = defaultRegistry) {
  if (!graphId || fallbackGraph?.metadata?.graph_id === graphId) {
    return fallbackGraph;
  }

  try {
    const loaded = resolveLoadedGraphWithRegistry(await fetchJson(`/graphs/${graphId}`), registry);
    return loaded || fallbackGraph;
  } catch (e) {
    console.warn("graphStorePersistenceHelpers: resolveGraphForDetail failed", e);
    return fallbackGraph;
  }
}

export {
  resolveGraphForDetail
};
