import { describe, expect, it } from "vitest";
import { buildBacktestDetailSummaryModel } from "./backtestDetailSummaryModel";

describe("backtestDetailSummaryModel", () => {
  it("builds folded and expanded summary metrics for the detail hero", () => {
    const base = {
      summary: {
        total_return_ratio: 0.125,
        annualized_return: 0.2,
        annualized_volatility: 0.15,
        win_rate: 0.6,
        trade_count: 8,
        final_equity: 11200,
        risk_adjusted: {
          sharpe_ratio: 1.25,
          sortino_ratio: 1.8,
          calmar_ratio: 0.9
        },
        trade_analysis: {
          profit_factor: 1.4
        },
        drawdown_analysis: {
          max_drawdown_ratio: 0.08,
          max_drawdown_duration_days: 6.2
        },
        benchmark_comparison: {
          alpha: 0.03,
          beta: 0.72
        }
      },
      manifest: {
        protocol_name: "runtime-config/v1"
      },
      metrics: {
        final_account: { equity_estimate: 11100 }
      }
    };

    const folded = buildBacktestDetailSummaryModel(base);
    const expanded = buildBacktestDetailSummaryModel({ ...base, summaryExpanded: true });

    expect(folded.summaryItems.map((item) => [item.label, item.value])).toEqual([
      ["收益", "+12.50%"],
      ["年化收益", "+20.00%"],
      ["夏普", "1.25"],
      ["最大回撤", "+8.00%"],
      ["盈亏比", "1.40"]
    ]);
    expect(expanded.summaryItems.map((item) => [item.label, item.value])).toEqual([
      ["收益", "+12.50%"],
      ["年化收益", "+20.00%"],
      ["夏普", "1.25"],
      ["最大回撤", "+8.00%"],
      ["盈亏比", "1.40"],
      ["索提诺", "1.80"],
      ["卡尔玛", "0.90"],
      ["年化波动率", "+15.00%"],
      ["最大回撤持续", "6 天"],
      ["Alpha", "+3.00%"],
      ["Beta", "0.72"],
      ["胜率", "60.0%"],
      ["成交数", "8"],
      ["协议", "runtime-config/v1"],
      ["最终权益", "11200"]
    ]);
  });

  it("falls back to selected summary protocol and trade length when metrics are sparse", () => {
    const model = buildBacktestDetailSummaryModel({
      summary: {},
      selectedSummary: { protocol_name: "selected-protocol" },
      trades: [{ id: 1 }, { id: 2 }],
      summaryExpanded: true
    });

    expect(model.summaryItems.find((item) => item.label === "协议")?.value).toBe("selected-protocol");
    expect(model.summaryItems.find((item) => item.label === "成交数")?.value).toBe("2");
  });
});
