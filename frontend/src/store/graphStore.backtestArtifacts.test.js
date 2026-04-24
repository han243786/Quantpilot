import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useGraphStore } from "./graphStore";
import { buildBacktestSuccessFixture } from "../test/fixtures/runtime/backtestSuccess";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";

function buildGraph(registry, overrides = {}) {
  return buildValidatedSampleGraph(registry, (graph) => {
    graph.metadata.name = "Backtest Artifact Store Graph";
    graph.metadata.graph_id = "artifact_graph";
    Object.assign(graph.metadata, overrides.metadata || {});
    Object.assign(graph, overrides);
  });
}

describe("graphStore backtest artifact loading", () => {
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

  it("prefers event_log artifact events when loading backtest detail", async () => {
    const fixture = buildBacktestSuccessFixture({
      graphId: "artifact_graph",
      compileId: "compile_artifact_001",
      backtestId: "backtest_artifact_001"
    });
    const artifactEvents = fixture.detailResponse.backtest_artifacts.event_log.events.map(
      (event, index) =>
        index === 0
          ? {
              ...event,
              node_id: "artifact_node_1",
              summary: "Artifact event log entry"
            }
          : event
    );
    fixture.detailResponse.backtest_artifacts.event_log.events = artifactEvents;
    fixture.detailResponse.runtime_diagnostics = {
      source: "backtest_event_log",
      default_selected_node_id: "artifact_node_1",
      active_nodes: [
        {
          node_id: "artifact_node_1",
          latest_event_type: artifactEvents[0].event_type,
          latest_event_label: artifactEvents[0].event_type,
          latest_event_time_ms: artifactEvents[0].event_time_ms,
          event_count: 1
        }
      ],
      node_details: {
        artifact_node_1: {
          node_id: "artifact_node_1",
          latest_event: {
            event_id: artifactEvents[0].event_id,
            event_type: artifactEvents[0].event_type,
            label: artifactEvents[0].event_type,
            summary: artifactEvents[0].summary,
            tone: "info",
            severity: artifactEvents[0].severity,
            event_time_ms: artifactEvents[0].event_time_ms
          },
          latest_input_rows: [],
          latest_output_rows: [],
          latest_notice: null,
          recent_events: [],
          event_count: 1
        }
      }
    };
    fixture.detailResponse.events = [
      {
        event_id: "legacy_event_only",
        event_type: "RuntimeNotice",
        source_id: "legacy_source",
        node_id: "legacy_node",
        event_time_ms: 1_700_000_000_500,
        severity: "Info",
        summary: "Legacy top-level event",
        payload: {}
      }
    ];

    useGraphStore.setState({
      graph: buildGraph(initialState.registry)
    });

    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: true,
        json: async () => fixture.detailResponse
      })
    );

    const detail = await useGraphStore.getState().loadBacktestDetail("backtest_artifact_001");
    const state = useGraphStore.getState();

    expect(detail.backtest_artifacts.event_log.events[0].summary).toBe("Artifact event log entry");
    expect(state.runtime.events).toEqual(artifactEvents);
    expect(state.runtime.events[0].summary).toBe("Artifact event log entry");
    expect(state.runtime.selectedBacktestId).toBe("backtest_artifact_001");
    expect(state.runtime.highlightedNodeIds).toContain("artifact_node_1");
    expect(state.runtime.highlightedNodeIds).not.toContain("legacy_node");
    expect(state.runtime.diagnostics?.source).toBe("backtest_event_log");
    expect(state.runtime.diagnostics?.node_details?.artifact_node_1?.event_count).toBe(1);
  });
});
