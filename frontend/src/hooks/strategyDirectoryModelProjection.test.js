import { describe, expect, it } from "vitest";

import {
  buildActivityTimeline,
  buildAvailableGraphIds,
  buildHubSummary,
  buildStrategyDirectory,
  filterVisibleBacktests,
  filterVisibleRuns,
  resolveStrategyDirectoryActivityLabel,
  resolveStrategyDirectoryHealth
} from "./strategyDirectoryModelProjection";

function buildGraph(overrides = {}) {
  return {
    metadata: {
      graph_id: "alpha_strategy",
      name: "Alpha strategy",
      updated_at: 1710000000000,
      runtime_binding: {
        last_compile_id: "compile_alpha_001"
      },
      ...(overrides.metadata || {})
    },
    validation_state: {
      is_runnable: true,
      issue_counts: { error: 0, warning: 0 },
      ...(overrides.validation_state || {})
    },
    compile_summary: {
      compilable: true,
      protocol_name: "quantpilot/runtime-config/v1",
      config_hash: "cfg_alpha_001",
      ...(overrides.compile_summary || {})
    },
    ...overrides
  };
}

describe("strategyDirectoryModelProjection", () => {
  it("resolves visible runtime records from tracked graph ids only", () => {
    const availableGraphIds = buildAvailableGraphIds(buildGraph(), [
      { graph_id: "alpha_strategy" },
      { graph_id: "beta_strategy" },
      { graph_id: "" }
    ]);
    const runtime = {
      history: [
        { run_id: "run_alpha", graph_id: "alpha_strategy" },
        { run_id: "run_current_without_id" },
        { run_id: "run_unknown", graph_id: "unknown_strategy" }
      ],
      backtestHistory: [
        { backtest_id: "bt_beta", graph_id: "beta_strategy" },
        { backtest_id: "bt_unknown", graph_id: "unknown_strategy" }
      ]
    };

    expect([...availableGraphIds]).toEqual(["alpha_strategy", "beta_strategy"]);
    expect(filterVisibleRuns(runtime, availableGraphIds, "alpha_strategy").map((item) => item.run_id)).toEqual([
      "run_alpha",
      "run_current_without_id"
    ]);
    expect(filterVisibleBacktests(runtime, availableGraphIds, "alpha_strategy").map((item) => item.backtest_id)).toEqual([
      "bt_beta"
    ]);
  });

  it("builds a sorted strategy directory with health, activity labels, and recent records", () => {
    const graph = buildGraph();
    const graphIndex = [
      { graph_id: "alpha_strategy", name: "Alpha strategy", updated_at: 1710000000000, path: "alpha.qp" },
      { graph_id: "beta_strategy", name: "Beta strategy", updated_at: 1710000100000, path: "beta.qp" }
    ];
    const visibleRuns = [
      { run_id: "run_beta_1", graph_id: "beta_strategy", compile_id: "compile_beta", created_at_ms: 1710000300000 },
      { run_id: "run_alpha_1", graph_id: "alpha_strategy", compile_id: "compile_alpha_001", created_at_ms: 1710000200000 }
    ];
    const visibleBacktests = [
      {
        backtest_id: "bt_beta_1",
        graph_id: "beta_strategy",
        compile_id: "compile_beta",
        config_hash: "cfg_beta",
        protocol_name: "quantpilot/runtime-config/v1",
        created_at_ms: 1710000400000,
        summary: { total_return_ratio: -0.03 },
        filters: { dataset_labels: ["ETH-4h", "ETH-4h", "BTC-1h", "SOL-1h"] }
      }
    ];

    const directory = buildStrategyDirectory(graph, visibleRuns, visibleBacktests, graphIndex);

    expect(directory.map((entry) => entry.graphId)).toEqual(["alpha_strategy", "beta_strategy"]);
    expect(directory[0]).toMatchObject({
      graphId: "alpha_strategy",
      isCurrent: true,
      health: { tone: "success", label: "可运行" },
      activityLabel: "模拟",
      runCount: 1,
      backtestCount: 0
    });
    expect(directory[1]).toMatchObject({
      graphId: "beta_strategy",
      health: { tone: "info", label: "已跟踪" },
      activityLabel: "回测",
      runCount: 1,
      backtestCount: 1,
      latestReturnRatio: -0.03
    });
    expect(directory[1].datasetLabels).toEqual(["ETH-4h", "BTC-1h", "SOL-1h"]);
  });

  it("builds a capped, newest-first activity timeline and hub summary", () => {
    const runs = Array.from({ length: 4 }, (_, index) => ({
      run_id: `run_${index}`,
      graph_id: "alpha_strategy",
      compile_id: `compile_${index}`,
      created_at_ms: 1000 + index
    }));
    const backtests = Array.from({ length: 4 }, (_, index) => ({
      backtest_id: `bt_${index}`,
      graph_id: "alpha_strategy",
      compile_id: `compile_bt_${index}`,
      created_at_ms: 2000 + index,
      summary: { total_return_ratio: index === 0 ? -0.01 : 0.02 * index }
    }));

    const timeline = buildActivityTimeline(runs, backtests);
    const strategies = [
      { health: { tone: "danger" }, backtestCount: 0, lastActivityAt: 1000 },
      { health: { tone: "success" }, backtestCount: 1, lastActivityAt: 2003 },
      { health: { tone: "info" }, backtestCount: 2, lastActivityAt: 2001 }
    ];

    expect(timeline).toHaveLength(6);
    expect(timeline.map((item) => item.id)).toEqual(["bt_3", "bt_2", "bt_1", "bt_0", "run_3", "run_2"]);
    expect(timeline.find((item) => item.id === "bt_0")?.note).toBe("-1.00%");
    expect(timeline.find((item) => item.id === "run_3")?.detail).toBe("模拟运行");
    expect(buildHubSummary(strategies, timeline, ["bt_2", "bt_3"])).toEqual({
      trackedCount: 3,
      issueCount: 1,
      runnableCount: 1,
      researchReadyCount: 2,
      compareCount: 2,
      latestActivityAt: 2003
    });
  });

  it("keeps health and activity fallbacks explicit", () => {
    expect(resolveStrategyDirectoryHealth({ isCurrent: true, issueCount: 2 })).toEqual({
      tone: "danger",
      label: "待修复"
    });
    expect(resolveStrategyDirectoryHealth({ runCount: 0, backtestCount: 0 })).toEqual({
      tone: "muted",
      label: "草稿"
    });
    expect(resolveStrategyDirectoryActivityLabel({})).toBe("暂无活动");
    expect(resolveStrategyDirectoryActivityLabel({ lastActivityAt: 5 })).toBe("已更新");
  });
});
