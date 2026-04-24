import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useGraphStore } from "./graphStore";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";

function buildGraph(registry, graphId, name, updatedAt = 1_700_000_000_000) {
  return buildValidatedSampleGraph(registry, (graph) => {
    graph.metadata.graph_id = graphId;
    graph.metadata.name = name;
    graph.metadata.created_at = updatedAt;
    graph.metadata.updated_at = updatedAt;
    if (Array.isArray(graph.nodes) && graph.nodes[0]) {
      graph.nodes[0].config = {
        ...graph.nodes[0].config,
        window_size: updatedAt === 1_700_000_000_000 ? 20 : 55
      };
    }
  });
}

describe("graphStore version history", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    useGraphStore.setState(initialState, true);
    window.localStorage.clear();
    vi.unstubAllGlobals();
    const registry = initialState.registry;
    useGraphStore.setState({
      graph: buildGraph(registry, "alpha_strategy", "Working draft"),
      graphVersions: [],
      graphVersionsStatus: "idle",
      graphVersionPreview: null,
      graphVersionPreviewStatus: "idle",
      graphVersionCompare: null,
      graphVersionCompareStatus: "idle"
    });
  });

  afterEach(() => {
    useGraphStore.setState(initialState, true);
    window.localStorage.clear();
    vi.unstubAllGlobals();
  });

  it("loads persisted versions and previews a version without overwriting the working draft", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url, options = {}) => {
        if (url.endsWith("/api/graphs/alpha_strategy/versions")) {
          return {
            ok: true,
            json: async () => [
              {
                graph_id: "alpha_strategy",
                version_id: "1700000000001",
                name: "Persisted V2",
                updated_at: 1_700_000_000_001,
                version_label: "tuned",
                save_note: "Raised window size for signal smoothing.",
                node_count: 7,
                edge_count: 6,
                path: "storage/graphs/versions/alpha_strategy/1700000000001.json",
                quantscript_path: "storage/graphs/versions/alpha_strategy/1700000000001.qs",
                is_latest: true
              }
            ]
          };
        }

        if (url.endsWith("/api/graphs/alpha_strategy/versions/1700000000001")) {
          return {
            ok: true,
            json: async () =>
              buildGraph(initialState.registry, "alpha_strategy", "Persisted V2", 1_700_000_000_001)
          };
        }

        throw new Error(`Unhandled request: ${url} ${options.method || "GET"}`);
      })
    );

    const versions = await useGraphStore.getState().refreshGraphVersions("alpha_strategy");
    const preview = await useGraphStore
      .getState()
      .loadGraphVersionPreview("alpha_strategy", "1700000000001");

    expect(versions).toHaveLength(1);
    expect(useGraphStore.getState().graph.metadata.name).toBe("Working draft");
    expect(preview.metadata.name).toBe("Persisted V2");
    expect(useGraphStore.getState().graphVersionPreview.versionId).toBe("1700000000001");
    expect(useGraphStore.getState().graphVersionPreview.graph.metadata.name).toBe("Persisted V2");
    expect(useGraphStore.getState().graphVersions[0].version_label).toBe("tuned");
    expect(useGraphStore.getState().graphVersions[0].save_note).toContain("window size");
  });

  it("restores a persisted version through the API and then reloads the current graph", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url, options = {}) => {
        const method = options.method || "GET";

        if (url.endsWith("/api/graphs/alpha_strategy/versions/1700000000001/restore") && method === "POST") {
          return {
            ok: true,
            json: async () => ({
              graph_id: "alpha_strategy",
              version_id: "1700000000002",
              saved_at: 1_700_000_000_002,
              path: "storage/graphs/alpha_strategy.json",
              quantscript_path: "storage/graphs/alpha_strategy.qs"
            })
          };
        }

        if (url.endsWith("/api/graphs") && method === "GET") {
          return {
            ok: true,
            json: async () => [
              {
                graph_id: "alpha_strategy",
                name: "Persisted V2",
                updated_at: 1_700_000_000_002,
                path: "storage/graphs/alpha_strategy.qs"
              }
            ]
          };
        }

        if (url.endsWith("/api/graphs/alpha_strategy") && method === "GET") {
          return {
            ok: true,
            json: async () =>
              buildGraph(initialState.registry, "alpha_strategy", "Persisted V2", 1_700_000_000_002)
          };
        }

        if (url.endsWith("/api/graphs/alpha_strategy/versions") && method === "GET") {
          return {
            ok: true,
            json: async () => [
              {
                graph_id: "alpha_strategy",
                version_id: "1700000000002",
                name: "Persisted V2",
                updated_at: 1_700_000_000_002,
                version_label: "restored",
                save_note: "Restored baseline after review.",
                node_count: 7,
                edge_count: 6,
                path: "storage/graphs/versions/alpha_strategy/1700000000002.json",
                quantscript_path: "storage/graphs/versions/alpha_strategy/1700000000002.qs",
                is_latest: true
              },
              {
                graph_id: "alpha_strategy",
                version_id: "1700000000001",
                name: "Persisted V1",
                updated_at: 1_700_000_000_001,
                version_label: "baseline",
                save_note: "Initial persisted strategy snapshot.",
                node_count: 7,
                edge_count: 6,
                path: "storage/graphs/versions/alpha_strategy/1700000000001.json",
                quantscript_path: "storage/graphs/versions/alpha_strategy/1700000000001.qs",
                is_latest: false
              }
            ]
          };
        }

        throw new Error(`Unhandled request: ${url} ${method}`);
      })
    );

    useGraphStore.setState({
      graphVersionPreview: {
        versionId: "1700000000001",
        graph: buildGraph(initialState.registry, "alpha_strategy", "Persisted V1", 1_700_000_000_001)
      },
      graphVersionPreviewStatus: "ready"
    });

    await useGraphStore.getState().restoreGraphVersion("alpha_strategy", "1700000000001");

    expect(useGraphStore.getState().graph.metadata.name).toBe("Persisted V2");
    expect(useGraphStore.getState().graphVersions).toHaveLength(2);
    expect(useGraphStore.getState().graphVersions[0].version_id).toBe("1700000000002");
    expect(useGraphStore.getState().graphVersionPreview).toBeNull();
  });

  it("compares persisted versions and keeps structured diff state in the store", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url, options = {}) => {
        const method = options.method || "GET";
        if (
          url.endsWith(
            "/api/graphs/alpha_strategy/versions/compare/1700000000001/1700000000002"
          ) &&
          method === "GET"
        ) {
          return {
            ok: true,
            json: async () => ({
              graph_id: "alpha_strategy",
              left: {
                graph_id: "alpha_strategy",
                version_id: "1700000000001",
                name: "Persisted V1",
                updated_at: 1_700_000_000_001,
                version_label: "baseline",
                save_note: "Initial persisted strategy snapshot.",
                node_count: 7,
                edge_count: 6,
                is_latest: false
              },
              right: {
                graph_id: "alpha_strategy",
                version_id: "1700000000002",
                name: "Persisted V2",
                updated_at: 1_700_000_000_002,
                version_label: "tuned",
                save_note: "Raised window size and removed one edge.",
                node_count: 7,
                edge_count: 5,
                is_latest: true
              },
              metadata_rows: [
                {
                  key: "version_label",
                  label: "Version label",
                  status: "different",
                  left_value: "baseline",
                  right_value: "tuned"
                }
              ],
              node_diff: {
                left_count: 7,
                right_count: 7,
                added_ids: [],
                removed_ids: [],
                changed_ids: ["data_data_1"]
              },
              edge_diff: {
                left_count: 6,
                right_count: 5,
                added_ids: [],
                removed_ids: ["edge_data_data_1_intent_intent_2_market_data_out_data_input"],
                changed_ids: []
              },
              config_diffs: [
                {
                  node_id: "data_data_1",
                  node_name: "Persisted data source",
                  field_path: "window_size",
                  status: "different",
                  left_value: "20",
                  right_value: "55"
                }
              ],
              has_changes: true
            })
          };
        }

        throw new Error(`Unhandled request: ${url} ${method}`);
      })
    );

    const compare = await useGraphStore
      .getState()
      .compareGraphVersions("alpha_strategy", "1700000000001", "1700000000002");

    expect(compare.has_changes).toBe(true);
    expect(useGraphStore.getState().graphVersionCompare.left.version_label).toBe("baseline");
    expect(useGraphStore.getState().graphVersionCompare.right.version_label).toBe("tuned");
    expect(useGraphStore.getState().graphVersionCompare.config_diffs[0].field_path).toBe(
      "window_size"
    );
  });

  it("posts actor metadata when saving a persisted version and refreshes audit history", async () => {
    const fetchSpy = vi.fn(async (url, options = {}) => {
      const method = options.method || "GET";
      if (url.endsWith("/api/graphs/save") && method === "POST") {
        return {
          ok: true,
          json: async () => ({
            graph_id: "alpha_strategy",
            version_id: "1700000000003",
            saved_at: 1_700_000_000_003,
            version_label: "owner-save",
            save_note: "Saved by owner actor.",
            path: "storage/graphs/alpha_strategy.json",
            quantscript_path: "storage/graphs/alpha_strategy.qs",
            collaboration: {
              owner: {
                actor_id: "owner_alpha",
                display_name: "Owner Alpha"
              },
              editors: [],
              last_saved_by: {
                actor_id: "owner_alpha",
                display_name: "Owner Alpha"
              }
            }
          })
        };
      }

      if (url.endsWith("/api/graphs") && method === "GET") {
        return {
          ok: true,
          json: async () => []
        };
      }

      if (url.endsWith("/api/graphs/alpha_strategy/versions") && method === "GET") {
        return {
          ok: true,
          json: async () => []
        };
      }

      if (url.endsWith("/api/graphs/alpha_strategy/audit") && method === "GET") {
        return {
          ok: true,
          json: async () => []
        };
      }

      throw new Error(`Unhandled request: ${url} ${method}`);
    });
    vi.stubGlobal("fetch", fetchSpy);

    useGraphStore.setState({
      graph: buildValidatedSampleGraph(initialState.registry, (graph) => {
        graph.metadata.graph_id = "alpha_strategy";
        graph.metadata.name = "Working draft";
        graph.metadata.collaboration = {
          owner: {
            actor_id: "owner_alpha",
            display_name: "Owner Alpha"
          },
          editors: []
        };
      })
    });

    await useGraphStore.getState().saveGraph({
      versionLabel: "owner-save",
      saveNote: "Saved by owner actor."
    });

    const saveCall = fetchSpy.mock.calls.find(
      ([url, options = {}]) =>
        String(url).endsWith("/api/graphs/save") && (options.method || "GET") === "POST"
    );
    const requestBody = JSON.parse(saveCall[1].body);
    expect(requestBody.actor).toEqual({
      actor_id: "owner_alpha",
      display_name: "Owner Alpha"
    });
  });
});
