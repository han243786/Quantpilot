import { beforeEach, afterEach, describe, expect, it, vi } from "vitest";
import { useGraphStore } from "./graphStore";
import { SUPPORTED_FRONTEND_MODULE_KEYS } from "../capabilities/supportMatrix";
import {
  backendCapabilitiesFixture,
  capabilityFallbackFixtures
} from "../test/fixtures/capabilities/capabilityFallbacks";

function cloneCapabilities(capabilities) {
  return JSON.parse(JSON.stringify(capabilities));
}

describe("graphStore capability fallback", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    useGraphStore.setState(initialState, true);
    window.localStorage.clear();
    vi.unstubAllGlobals();
  });

  afterEach(() => {
    useGraphStore.setState(initialState, true);
    window.localStorage.clear();
    vi.unstubAllGlobals();
  });

  it("uses remote capabilities when capability fetch succeeds", async () => {
    const remoteCapabilities = cloneCapabilities(backendCapabilitiesFixture);

    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: true,
        json: async () => remoteCapabilities
      })
    );

    const result = await useGraphStore.getState().refreshCapabilities();
    const state = useGraphStore.getState();

    expect(result).toEqual(remoteCapabilities);
    expect(state.capabilityStatus).toBe("ready");
    expect(state.capabilitySource).toBe("remote");
    expect(state.capabilityMessage).toBe("");
    expect(state.capabilities.runtime.supported_modes).toEqual(["paper"]);
    expect(state.capabilities.frontend.module_support).toEqual(remoteCapabilities.frontend.module_support);
    expect(state.capabilities.workspace.surfaces).toEqual(remoteCapabilities.workspace.surfaces);
    expect(state.capabilities.ui_actions.actions).toEqual(remoteCapabilities.ui_actions.actions);
    expect(
      JSON.parse(window.localStorage.getItem(capabilityFallbackFixtures.cacheKey))
    ).toEqual(remoteCapabilities);
  });

  it("falls back to cached capabilities when capability fetch fails", async () => {
    const cachedCapabilities = cloneCapabilities(backendCapabilitiesFixture);
    window.localStorage.setItem(
      capabilityFallbackFixtures.cacheKey,
      JSON.stringify(cachedCapabilities)
    );

    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: false,
        text: async () => "backend unavailable"
      })
    );

    const result = await useGraphStore.getState().refreshCapabilities();
    const state = useGraphStore.getState();

    expect(result).toEqual(cachedCapabilities);
    expect(state.capabilityStatus).toBe("degraded");
    expect(state.capabilitySource).toBe("cache");
    expect(state.capabilityMessage).toContain("本地缓存的能力快照");
    expect(state.capabilities.frontend.supported_module_keys).toEqual(
      cachedCapabilities.frontend.supported_module_keys
    );
  });

  it("enters safe fallback mode when capability fetch fails and no cache exists", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: false,
        text: async () => "network down"
      })
    );

    const result = await useGraphStore.getState().refreshCapabilities();
    const state = useGraphStore.getState();

    expect(result.frontend.supported_module_keys).toEqual(SUPPORTED_FRONTEND_MODULE_KEYS);
    expect(result.runtime.supported_modes).toEqual(["paper"]);
    expect(state.capabilityStatus).toBe("error");
    expect(state.capabilitySource).toBe("safe_fallback");
    expect(state.capabilityMessage).toContain("安全回退模式");
    expect(
      result.frontend.module_support.every((entry) => entry.status === "declared_only")
    ).toBe(true);
    expect(
      result.workspace.surfaces.every((entry) => entry.status === "declared_only")
    ).toBe(true);
    expect(
      result.ui_actions.actions.every((entry) => entry.status === "declared_only")
    ).toBe(true);
  });
});
