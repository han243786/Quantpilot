import { humanizeErrorText } from "../utils/errorText";
import {
  attachValidationWithRegistry,
  buildRegistryFromCapabilities,
  createSafeFallbackCapabilities,
  loadCapabilitiesFromCache,
  resolveStrategyIrDraft,
  saveCapabilitiesToCache,
  saveGraphToStorage
} from "./graphStoreHelpers";

function buildCapabilityRefreshState({
  capabilities,
  currentState,
  capabilityStatus,
  capabilitySource,
  capabilityMessage
}) {
  const nextRegistry = buildRegistryFromCapabilities(capabilities);
  const nextGraph = attachValidationWithRegistry(currentState.graph, nextRegistry);
  saveGraphToStorage(nextGraph);

  return {
    capabilities,
    state: {
      registry: nextRegistry,
      capabilities,
      capabilityStatus,
      capabilitySource,
      capabilityMessage,
      graph: nextGraph,
      quantScriptDraft:
        nextGraph.metadata?.artifacts?.quantscript?.graph_source || currentState.quantScriptDraft,
      strategyIrDraft: resolveStrategyIrDraft(nextGraph, currentState.strategyIrDraft)
    }
  };
}

export function buildRemoteCapabilityRefreshState(capabilities, currentState) {
  saveCapabilitiesToCache(capabilities);
  return buildCapabilityRefreshState({
    capabilities,
    currentState,
    capabilityStatus: "ready",
    capabilitySource: "remote",
    capabilityMessage: ""
  });
}

export function buildCachedCapabilityRefreshState(cachedCapabilities, currentState, capabilityMessage) {
  return buildCapabilityRefreshState({
    capabilities: cachedCapabilities,
    currentState,
    capabilityStatus: "degraded",
    capabilitySource: "cache",
    capabilityMessage
  });
}

export function buildSafeFallbackCapabilityRefreshState(safeFallbackCapabilities, currentState, capabilityMessage) {
  return buildCapabilityRefreshState({
    capabilities: safeFallbackCapabilities,
    currentState,
    capabilityStatus: "error",
    capabilitySource: "safe_fallback",
    capabilityMessage
  });
}

export function buildCapabilityRefreshFailureState(error, currentState, messages) {
  const message = humanizeErrorText(error, messages.loadFailureFallback);
  const cachedCapabilities = loadCapabilitiesFromCache();

  if (cachedCapabilities) {
    return buildCachedCapabilityRefreshState(
      cachedCapabilities,
      currentState,
      messages.cacheFallbackMessage
    );
  }

  return buildSafeFallbackCapabilityRefreshState(
    createSafeFallbackCapabilities(message),
    currentState,
    messages.safeFallbackMessage
  );
}
