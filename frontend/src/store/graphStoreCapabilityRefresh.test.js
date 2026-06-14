import { beforeEach, describe, expect, it } from "vitest";

import {
  buildCapabilityRefreshFailureState,
  buildRemoteCapabilityRefreshState
} from "./graphStoreCapabilityRefresh";
import {
  CAPABILITY_CACHE_KEY,
  defaultCapabilities,
  defaultRegistry,
  fallbackRunnableGraph
} from "./graphStoreHelpers";
import { backendCapabilitiesFixture } from "../test/fixtures/capabilities/capabilityFallbacks";

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function createCurrentState() {
  return {
    graph: fallbackRunnableGraph(defaultRegistry),
    quantScriptDraft: "",
    strategyIrDraft: ""
  };
}

const messages = {
  loadFailureFallback: "capability load failed",
  cacheFallbackMessage: "using cached capabilities",
  safeFallbackMessage: "using safe fallback capabilities"
};

describe("graphStoreCapabilityRefresh", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it("projects remote capabilities into ready store state and caches the snapshot", () => {
    const capabilities = clone(backendCapabilitiesFixture);
    const result = buildRemoteCapabilityRefreshState(capabilities, createCurrentState());

    expect(result.capabilities).toEqual(capabilities);
    expect(result.state.capabilityStatus).toBe("ready");
    expect(result.state.capabilitySource).toBe("remote");
    expect(result.state.capabilityMessage).toBe("");
    expect(result.state.registry.capabilities).toEqual(capabilities);
    expect(result.state.graph.validation_state).toEqual(expect.objectContaining({
      is_runnable: expect.any(Boolean)
    }));
    expect(JSON.parse(window.localStorage.getItem(CAPABILITY_CACHE_KEY))).toEqual(capabilities);
  });

  it("projects a failed refresh from cached capabilities before safe fallback", () => {
    const cachedCapabilities = clone(backendCapabilitiesFixture);
    window.localStorage.setItem(CAPABILITY_CACHE_KEY, JSON.stringify(cachedCapabilities));

    const result = buildCapabilityRefreshFailureState(
      new Error("backend unavailable"),
      createCurrentState(),
      messages
    );

    expect(result.capabilities).toEqual(cachedCapabilities);
    expect(result.state.capabilityStatus).toBe("degraded");
    expect(result.state.capabilitySource).toBe("cache");
    expect(result.state.capabilityMessage).toBe(messages.cacheFallbackMessage);
  });

  it("projects a failed refresh into safe fallback when no cache exists", () => {
    const result = buildCapabilityRefreshFailureState(
      new Error("backend unavailable"),
      createCurrentState(),
      messages
    );

    expect(result.capabilities.api_version).toBe(defaultCapabilities.api_version);
    expect(result.state.capabilityStatus).toBe("error");
    expect(result.state.capabilitySource).toBe("safe_fallback");
    expect(result.state.capabilityMessage).toBe(messages.safeFallbackMessage);
    expect(result.capabilities.frontend.module_support.every((entry) => entry.status === "declared_only")).toBe(true);
  });
});
