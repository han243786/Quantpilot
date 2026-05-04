import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  buildRuntimeHistoryFailureMessage,
  saveBacktestRecordFlow,
  saveRunRecordFlow,
  warmRuntimeSidebarDataFlow
} from "./graphStoreRuntimeHistoryFlow";

const runtimeHistoryApi = vi.hoisted(() => ({
  fetchRunHistoryList: vi.fn(),
  fetchBacktestHistoryList: vi.fn(),
  fetchRunDetail: vi.fn(),
  fetchRuntimeMutations: vi.fn(),
  fetchBacktestDetail: vi.fn(),
  saveRunRecord: vi.fn(),
  saveBacktestRecord: vi.fn()
}));

vi.mock("./graphStoreRuntimeHistoryApi", async (importOriginal) => ({
  ...(await importOriginal()),
  fetchRunHistoryList: runtimeHistoryApi.fetchRunHistoryList,
  fetchBacktestHistoryList: runtimeHistoryApi.fetchBacktestHistoryList,
  fetchRunDetail: runtimeHistoryApi.fetchRunDetail,
  fetchRuntimeMutations: runtimeHistoryApi.fetchRuntimeMutations,
  fetchBacktestDetail: runtimeHistoryApi.fetchBacktestDetail,
  saveRunRecord: runtimeHistoryApi.saveRunRecord,
  saveBacktestRecord: runtimeHistoryApi.saveBacktestRecord
}));

vi.mock("./graphStoreHelpers", async (importOriginal) => ({
  ...(await importOriginal()),
  resolveGraphForDetail: vi.fn(async (_graphId, graph) => graph),
  saveGraphToStorage: vi.fn()
}));

vi.mock("./graphStoreRuntimeHistoryProjection", () => ({
  projectRunDetailGraph: vi.fn((graph) => ({
    nextGraph: graph,
    highlightedNodeIds: []
  })),
  projectBacktestDetailGraph: vi.fn((graph, detail) => ({
    nextGraph: graph,
    events: detail.events || detail.backtest_artifacts?.event_log?.events || [],
    highlightedNodeIds: []
  }))
}));

function buildStoreHarness() {
  let state = {
    graph: {
      metadata: { graph_id: "graph_saved" },
      nodes: [],
      edges: []
    },
    registry: {},
    runtime: {
      history: [],
      historyStatus: "idle",
      backtestHistory: [],
      backtestHistoryStatus: "idle",
      backtestCompareSelection: [],
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

  return {
    get: () => state,
    set(updater) {
      const patch = typeof updater === "function" ? updater(state) : updater;
      state = {
        ...state,
        ...patch,
        runtime: patch.runtime || state.runtime,
        graph: patch.graph || state.graph
      };
    }
  };
}

describe("graphStoreRuntimeHistoryFlow", () => {
  beforeEach(() => {
    Object.values(runtimeHistoryApi).forEach((mock) => mock.mockReset());
    runtimeHistoryApi.fetchRuntimeMutations.mockResolvedValue([]);
  });

  it("keeps the backend reason when formatting runtime-history failures", () => {
    const message = buildRuntimeHistoryFailureMessage("run_history", {
      status: 503,
      message: "backend unavailable"
    });
    expect(message).toContain("backend unavailable");
  });

  it("uses the corrected Chinese fallback copy for backtest detail failures", () => {
    const message = buildRuntimeHistoryFailureMessage("backtest_detail", null);
    expect(message).toContain("加载回测详情失败");
  });

  it("uses explicit discard fallback copy for transient artifacts", () => {
    const message = buildRuntimeHistoryFailureMessage("backtest_discard", null);
    expect(message).toContain("丢弃回测结果失败");
  });

  it("warms only the missing runtime sidebars", async () => {
    const refreshRunHistory = vi.fn(async () => ["run-1"]);
    const refreshBacktestHistory = vi.fn(async () => ["bt-1"]);
    const refreshExperimentHistory = vi.fn(async () => ["experiment-1"]);
    const get = () => ({
      runtime: {
        historyStatus: "ready",
        history: ["run-1"],
        backtestHistoryStatus: "idle",
        backtestHistory: [],
        experimentsStatus: "idle",
        experiments: []
      },
      refreshRunHistory,
      refreshBacktestHistory,
      refreshExperimentHistory
    });

    const result = await warmRuntimeSidebarDataFlow(get);
    expect(refreshRunHistory).not.toHaveBeenCalled();
    expect(refreshBacktestHistory).toHaveBeenCalledTimes(1);
    expect(refreshExperimentHistory).toHaveBeenCalledTimes(1);
    expect(result).toEqual([["bt-1"], ["experiment-1"]]);
  });

  it("saves a run before refreshing history and then reloads persisted detail", async () => {
    const order = [];
    const harness = buildStoreHarness();
    const detail = {
      run_id: "run_saved_001",
      graph_id: "graph_saved",
      compile_id: "compile_saved",
      account: { cash_balance: 10000 },
      events: [{ event_id: "evt_saved", event_type: "ExecutionFilled" }],
      runtime_diagnostics: {
        source: "runtime_events",
        active_nodes: []
      }
    };

    runtimeHistoryApi.saveRunRecord.mockImplementation(async () => {
      order.push("save");
      return { saved: true };
    });
    runtimeHistoryApi.fetchRunHistoryList.mockImplementation(async () => {
      order.push("refresh");
      return [
        {
          run_id: "run_saved_001",
          graph_id: "graph_saved",
          compile_id: "compile_saved",
          created_at_ms: 1_700_000_000_000,
          status: "completed",
          event_count: 1
        }
      ];
    });
    runtimeHistoryApi.fetchRunDetail.mockImplementation(async () => {
      order.push("detail");
      return detail;
    });

    const result = await saveRunRecordFlow(harness.set, harness.get, "run_saved_001");

    expect(result).toBe(detail);
    expect(order).toEqual(["save", "refresh", "detail"]);
    expect(harness.get().runtime.history).toHaveLength(1);
    expect(harness.get().runtime.selectedHistoryRunId).toBe("run_saved_001");
    expect(harness.get().runtime.artifactPersistenceStatus).toBe("saved");
    expect(harness.get().runtime.events).toEqual(detail.events);
  });

  it("saves a backtest before refreshing history and then reloads persisted detail", async () => {
    const order = [];
    const harness = buildStoreHarness();
    const detail = {
      backtest_id: "backtest_saved_001",
      graph_id: "graph_saved",
      compile_id: "compile_saved",
      account: { cash_balance: 12000 },
      events: [{ event_id: "evt_backtest_saved", event_type: "ExecutionFilled" }],
      backtest_artifacts: {
        event_log: {
          events: [{ event_id: "evt_backtest_saved", event_type: "ExecutionFilled" }]
        },
        metrics: {
          summary: { total_return_ratio: 0.01 }
        }
      },
      runtime_diagnostics: {
        source: "runtime_events",
        active_nodes: []
      }
    };

    runtimeHistoryApi.saveBacktestRecord.mockImplementation(async () => {
      order.push("save");
      return { saved: true };
    });
    runtimeHistoryApi.fetchBacktestHistoryList.mockImplementation(async () => {
      order.push("refresh");
      return [
        {
          backtest_id: "backtest_saved_001",
          graph_id: "graph_saved",
          compile_id: "compile_saved",
          created_at_ms: 1_700_000_000_000,
          event_count: 1,
          filters: {
            dataset_labels: ["Binance:BTCUSDT:1m"],
            started_at_ms: 1_700_000_000_000,
            ended_at_ms: 1_700_000_060_000
          }
        }
      ];
    });
    runtimeHistoryApi.fetchBacktestDetail.mockImplementation(async () => {
      order.push("detail");
      return detail;
    });

    const result = await saveBacktestRecordFlow(
      harness.set,
      harness.get,
      "backtest_saved_001"
    );

    expect(result).toBe(detail);
    expect(order).toEqual(["save", "refresh", "detail"]);
    expect(harness.get().runtime.backtestHistory).toHaveLength(1);
    expect(harness.get().runtime.selectedBacktestId).toBe("backtest_saved_001");
    expect(harness.get().runtime.artifactPersistenceStatus).toBe("saved");
    expect(harness.get().runtime.backtestArtifacts).toBe(detail.backtest_artifacts);
    expect(harness.get().runtime.events).toEqual(detail.backtest_artifacts.event_log.events);
  });
});
