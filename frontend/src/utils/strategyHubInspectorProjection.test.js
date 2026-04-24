import { describe, expect, it } from "vitest";
import {
  getStrategyInspectorNextMove,
  projectInspectorBacktests,
  projectInspectorCompareQueue,
  projectInspectorRuns,
  projectStrategyHubInspectorOverview
} from "./strategyHubInspectorProjection";

describe("strategyHubInspectorProjection", () => {
  it("projects the next move based on strategy health and research depth", () => {
    expect(getStrategyInspectorNextMove(null).title).toBe("选择一条策略查看驾驶舱");

    expect(
      getStrategyInspectorNextMove({
        health: { tone: "danger" },
        backtestCount: 0
      }).title
    ).toBe("进入诊断并修复问题");

    expect(
      getStrategyInspectorNextMove({
        health: { tone: "info" },
        backtestCount: 2
      }).title
    ).toBe("查看近期研究结果");
  });

  it("projects inspector overview header, summary, and metrics", () => {
    const overview = projectStrategyHubInspectorOverview({
      name: "Alpha strategy",
      graphId: "alpha_strategy",
      health: { tone: "info", label: "可研究" },
      issueCount: 0,
      backtestCount: 2,
      runCount: 3,
      lastCompileId: "compile_alpha_001",
      protocolName: "quantpilot/runtime-config/v1",
      lastConfigHash: "cfg_alpha_001",
      datasetLabels: ["BTC-1h", "ETH-4h"]
    });

    expect(overview.routeItems).toEqual([
      { label: "策略", current: false },
      { label: "Alpha strategy", current: true }
    ]);
    expect(overview.summaryItems).toEqual([
      expect.objectContaining({ label: "当前压力", value: "已清空" }),
      expect.objectContaining({ label: "研究深度", value: "2 条回测" }),
      expect.objectContaining({ label: "模拟轨迹", value: "3 次运行" })
    ]);
    expect(overview.metrics).toEqual([
      expect.objectContaining({ label: "最近编译", value: "compile_alpha_001" }),
      expect.objectContaining({ label: "协议", value: "quantpilot/runtime-config/v1" }),
      expect.objectContaining({ label: "配置哈希", value: "cfg_alpha_001" }),
      expect.objectContaining({ label: "数据集", value: "BTC-1h, ETH-4h" })
    ]);
  });

  it("projects recent backtests and compare queue state", () => {
    const backtests = projectInspectorBacktests(
      {
        recentBacktests: [
          {
            backtest_id: "bt_alpha_01",
            created_at_ms: 1710000200000,
            summary: { total_return_ratio: 0.12 }
          }
        ]
      },
      ["bt_alpha_01", "bt_beta_02"]
    );

    expect(backtests).toEqual([
      expect.objectContaining({
        backtestId: "bt_alpha_01",
        returnLabel: "+12.00%",
        checked: true
      })
    ]);

    expect(projectInspectorCompareQueue(["bt_alpha_01"]).canCompare).toBe(false);
    expect(projectInspectorCompareQueue(["bt_alpha_01", "bt_beta_02"]).canCompare).toBe(true);
  });

  it("projects recent runs with compile labels", () => {
    const runs = projectInspectorRuns({
      recentRuns: [
        {
          run_id: "run_alpha_01",
          created_at_ms: 1710000100000,
          compile_id: null
        }
      ]
    });

    expect(runs).toEqual([
      expect.objectContaining({
        runId: "run_alpha_01",
        compileIdLabel: "无编译 ID"
      })
    ]);
  });
});
