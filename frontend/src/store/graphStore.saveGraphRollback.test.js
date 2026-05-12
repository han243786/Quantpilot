import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useGraphStore } from "./graphStore";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";

describe("graphStore saveGraph rollback", () => {
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

  it("prevents concurrent save when actionLock is set", async () => {
    useGraphStore.setState({ actionLock: "saving" });
    await useGraphStore.getState().saveGraph();
    // Should return early without error
    expect(useGraphStore.getState().actionLock).toBe("saving");
  });

  it("saves graph to localStorage before POST", async () => {
    const graph = buildValidatedSampleGraph(initialState.registry, (g) => {
      g.metadata.name = "Test Save Graph";
    });

    vi.stubGlobal("fetch", vi.fn(async (url, opts) => {
      if (opts?.method === "POST" && url.includes("/graphs/save")) {
        return { ok: true, json: async () => ({}) };
      }
      return { ok: true, json: async () => [] };
    }));

    useGraphStore.setState({ graph });
    await useGraphStore.getState().saveGraph();

    const stored = JSON.parse(window.localStorage.getItem("quantpilot_frontend_graph") || "{}");
    expect(stored.metadata?.name).toBe("Test Save Graph");
  });
});
