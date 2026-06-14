import { describe, expect, it } from "vitest";
import {
  buildBacktestCompareMeta,
  buildBacktestCompareSummary,
  buildBacktestCompareSummaryItems,
  normalizeCompareBacktestIds,
  resolveBacktestCompareStrategyId
} from "./backtestComparePageModel";

function buildDetail({ backtestId, graphId, totalReturn, maxDrawdown, tradeCount }) {
  return {
    backtest_id: backtestId,
    graph_id: graphId,
    backtest_artifacts: {
      metrics: {
        summary: {
          total_return_ratio: totalReturn,
          max_drawdown_ratio: maxDrawdown,
          trade_count: tradeCount
        }
      }
    }
  };
}

describe("backtestComparePageModel", () => {
  it("normalizes compare ids to two unique non-empty values", () => {
    expect(normalizeCompareBacktestIds(["bt_a", "", "bt_a", "bt_b", "bt_c"])).toEqual([
      "bt_a",
      "bt_b"
    ]);
    expect(buildBacktestCompareMeta(["bt_a", "bt_a", "bt_b"])).toBe("bt_a vs bt_b");
    expect(buildBacktestCompareMeta([])).toBe("-");
  });

  it("builds numeric and formatted comparison summaries", () => {
    const summary = buildBacktestCompareSummary([
      buildDetail({
        backtestId: "bt_left",
        graphId: "graph_shared",
        totalReturn: 0.12,
        maxDrawdown: 0.02,
        tradeCount: 5
      }),
      buildDetail({
        backtestId: "bt_right",
        graphId: "graph_shared",
        totalReturn: 0.08,
        maxDrawdown: 0.03,
        tradeCount: 3
      })
    ]);

    expect(summary).toEqual({
      returnDelta: 0.039999999999999994,
      drawdownDelta: -0.009999999999999998,
      tradeDelta: 2
    });
    expect(buildBacktestCompareSummaryItems({ t: (value) => value, summary })).toEqual([
      { label: "收益差值", value: "+4.00%" },
      { label: "回撤差值", value: "-1.00%" },
      { label: "成交差值", value: "2" }
    ]);
    expect(buildBacktestCompareSummary([])).toBeNull();
    expect(buildBacktestCompareSummaryItems()).toEqual([]);
  });

  it("resolves strategy identity from route override or matching detail graph ids", () => {
    const details = [
      buildDetail({ backtestId: "bt_a", graphId: "graph_shared" }),
      buildDetail({ backtestId: "bt_b", graphId: "graph_shared" })
    ];

    expect(resolveBacktestCompareStrategyId({ strategyId: "route_strategy", details })).toBe(
      "route_strategy"
    );
    expect(resolveBacktestCompareStrategyId({ details })).toBe("graph_shared");
    expect(
      resolveBacktestCompareStrategyId({
        details: [
          buildDetail({ backtestId: "bt_a", graphId: "graph_a" }),
          buildDetail({ backtestId: "bt_b", graphId: "graph_b" })
        ]
      })
    ).toBe("");
  });
});
