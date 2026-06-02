import { describe, expect, it } from "vitest";
import {
  defaultCapabilities,
  defaultRegistry,
  resolveStrategyIrDraft
} from "./graphStoreHelpers";
import {
  createInitialGraphStoreState,
  createInitialRuntimeState
} from "./graphStoreRootState";

describe("graphStoreRootState", () => {
  it("creates a fresh initial runtime state object", () => {
    const first = createInitialRuntimeState();
    const second = createInitialRuntimeState();

    expect(first).toMatchObject({
      status: "idle",
      connectionState: "disconnected",
      historyStatus: "idle",
      backtestHistoryStatus: "idle",
      actionLock: null,
      selectedRunStatus: "idle"
    });
    expect(first.backtestCompareSelection).toEqual({});
    expect(first).not.toBe(second);
    expect(first.events).not.toBe(second.events);
  });

  it("creates the initial graph store shell state", () => {
    const state = createInitialGraphStoreState();

    expect(state.registry).toBe(defaultRegistry);
    expect(state.capabilities).toBe(defaultCapabilities);
    expect(state.capabilityStatus).toBe("ready");
    expect(state.graphIndexStatus).toBe("idle");
    expect(state.graph).toBeTruthy();
    expect(state.strategyIrDraft).toEqual(resolveStrategyIrDraft(state.graph, ""));
    expect(state.runtime.status).toBe("idle");
  });
});
