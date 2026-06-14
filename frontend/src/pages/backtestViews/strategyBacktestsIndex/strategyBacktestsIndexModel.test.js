import { describe, expect, it } from "vitest";
import { buildStrategyBacktestsIndexModel } from "./strategyBacktestsIndexModel";

describe("strategyBacktestsIndexModel", () => {
  it("projects loaded strategy backtest index state", () => {
    const model = buildStrategyBacktestsIndexModel({
      strategyId: "alpha",
      graph: {
        metadata: {
          graph_id: "alpha",
          name: "Alpha"
        }
      },
      selectors: {
        filteredBacktests: [
          {
            created_at_ms: 1710000000000,
            summary: { total_return_ratio: 0.125 },
            filters: { dataset_labels: ["BTC-1h", "ETH-1h"] }
          }
        ],
        compareSelection: ["bt_1", "bt_2"]
      }
    });

    expect(model.strategyName).toBe("Alpha");
    expect(model.compareButtonDisabled).toBe(false);
    expect(model.isGraphLoading).toBe(false);
    expect(model.datasetText).toBe("BTC-1h, ETH-1h");
    expect(model.summaryItems).toEqual([
      { label: "回测数", value: "1" },
      { label: "对比队列", value: "2" },
      { label: "最近收益", value: "+12.50%" },
      { label: "最近回测", value: "2024/3/10 00:00:00" }
    ]);
  });

  it("uses route identity while strategy graph context is still loading", () => {
    const model = buildStrategyBacktestsIndexModel({
      strategyId: "alpha",
      graph: { metadata: { graph_id: "beta", name: "Beta" } },
      selectors: {
        filteredBacktests: [
          {
            filters: { dataset_labels: ["stale"] }
          }
        ],
        compareSelection: ["bt_1"]
      }
    });

    expect(model.strategyName).toBe("alpha");
    expect(model.isGraphLoading).toBe(true);
    expect(model.compareButtonDisabled).toBe(true);
    expect(model.datasetText).toBe("-");
  });
});
