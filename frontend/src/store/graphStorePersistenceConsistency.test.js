import { describe, expect, it } from "vitest";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";
import { buildBacktestSuccessFixture } from "../test/fixtures/runtime/backtestSuccess";
import { defaultRegistry } from "./graphStoreHelpers";
import { projectBacktestDetailGraph } from "./graphStoreRuntimeHistoryProjection";
import { buildBacktestDetailSelectionState } from "./graphStoreRuntimeHistoryState";
import { buildBacktestCompletionState } from "./graphStoreRuntimeSessionState";

function buildState() {
  return {
    selectedNodeId: null,
    runtime: {
      runId: null,
      runKind: null,
      status: "idle",
      connectionState: "disconnected",
      account: null,
      artifactPersistenceStatus: "idle",
      backtestArtifacts: null,
      diagnostics: null,
      events: [],
      backendError: null,
      selectedHistoryRunId: null,
      selectedBacktestId: null,
      highlightedNodeIds: []
    }
  };
}

function withoutArtifactPersistenceStatus(runtime) {
  const { artifactPersistenceStatus, ...rest } = runtime;
  return rest;
}

describe("graphStore persistence consistency", () => {
  it("keeps backtest selection shape aligned between live completion and persisted reload", () => {
    const fixture = buildBacktestSuccessFixture({
      graphId: "artifact_graph",
      compileId: "compile_artifact_001",
      backtestId: "backtest_artifact_001"
    });
    const graph = buildValidatedSampleGraph(defaultRegistry, (draft) => {
      draft.metadata.graph_id = "artifact_graph";
    });
    const state = buildState();

    const liveResult = buildBacktestCompletionState(
      state,
      graph,
      fixture.detailResponse,
      fixture.detailResponse.compile_id
    );
    const reloadProjection = projectBacktestDetailGraph(graph, fixture.detailResponse);
    const reloadResult = buildBacktestDetailSelectionState(
      state,
      reloadProjection.nextGraph,
      fixture.detailResponse,
      reloadProjection.events,
      reloadProjection.highlightedNodeIds
    );

    expect(liveResult.runtime.artifactPersistenceStatus).toBe("transient");
    expect(reloadResult.runtime.artifactPersistenceStatus).toBe("saved");
    expect(withoutArtifactPersistenceStatus(reloadResult.runtime)).toEqual(
      withoutArtifactPersistenceStatus(liveResult.runtime)
    );
    expect(reloadResult.selectedNodeId).toBe(liveResult.selectedNodeId);
    expect(reloadResult.graph.metadata.runtime_binding).toEqual(
      liveResult.nextGraph.metadata.runtime_binding
    );
  });
});
