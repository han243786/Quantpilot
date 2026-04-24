import { describe, expect, it } from "vitest";
import {
  buildPersistedRuntimeSelection,
  buildPersistedRuntimeSelectionState
} from "./graphStoreRuntimeSelectionState";

describe("graphStoreRuntimeSelectionState", () => {
  it("builds one shared completed selection shape for persisted runtime detail", () => {
    const runtime = {
      runId: null,
      runKind: null,
      status: "idle",
      connectionState: "disconnected",
      account: null,
      backtestArtifacts: null,
      diagnostics: null,
      events: [],
      backendError: "old error",
      selectedHistoryRunId: null,
      selectedBacktestId: null,
      highlightedNodeIds: []
    };

    expect(
      buildPersistedRuntimeSelection({
        runtime,
        runId: "backtest_001",
        runKind: "backtest",
        account: { equity_estimate: 1000 },
        backtestArtifacts: { metrics: { artifact_id: "metrics_001" } },
        diagnostics: { source: "backtest_event_log" },
        events: [{ event_id: "evt_1" }],
        selectedHistoryRunId: null,
        selectedBacktestId: "backtest_001",
        highlightedNodeIds: ["execution_node"]
      })
    ).toEqual({
      ...runtime,
      runId: "backtest_001",
      runKind: "backtest",
      status: "completed",
      connectionState: "disconnected",
      account: { equity_estimate: 1000 },
      backtestArtifacts: { metrics: { artifact_id: "metrics_001" } },
      diagnostics: { source: "backtest_event_log" },
      events: [{ event_id: "evt_1" }],
      backendError: null,
      selectedHistoryRunId: null,
      selectedBacktestId: "backtest_001",
      highlightedNodeIds: ["execution_node"]
    });
  });

  it("keeps graph selection and runtime selection on the same helper path", () => {
    const state = {
      selectedNodeId: "fallback_node",
      runtime: {
        backendError: "stale error"
      }
    };
    const nextGraph = { metadata: { graph_id: "graph_001" } };

    expect(
      buildPersistedRuntimeSelectionState(state, nextGraph, {
        runId: "run_001",
        runKind: "simulation",
        account: null,
        diagnostics: null,
        events: [],
        selectedHistoryRunId: "run_001",
        selectedBacktestId: null,
        highlightedNodeIds: ["risk_node"]
      })
    ).toEqual({
      graph: nextGraph,
      selectedNodeId: "risk_node",
      runtime: {
        backendError: null,
        runId: "run_001",
        runKind: "simulation",
        status: "completed",
        connectionState: "disconnected",
        account: null,
        backtestArtifacts: null,
        diagnostics: null,
        events: [],
        selectedHistoryRunId: "run_001",
        selectedBacktestId: null,
        highlightedNodeIds: ["risk_node"]
      }
    });
  });
});
