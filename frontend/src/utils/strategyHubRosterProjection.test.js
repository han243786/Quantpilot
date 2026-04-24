import { describe, expect, it } from "vitest";
import {
  projectStrategyHubActivityItems,
  projectStrategyHubRosterRows,
  projectStrategyHubRosterToolbar
} from "./strategyHubRosterProjection";

describe("strategyHubRosterProjection", () => {
  it("projects activity timeline into backtest and run buckets", () => {
    const projected = projectStrategyHubActivityItems([
      { id: "bt_alpha_01", kind: "backtest", createdAt: 1710000200000 },
      { id: "run_alpha_01", kind: "run", createdAt: 1710000100000 }
    ]);

    expect(projected.backtestItems).toHaveLength(1);
    expect(projected.runItems).toHaveLength(1);
    expect(projected.backtestItems[0]).toEqual(
      expect.objectContaining({ id: "bt_alpha_01", createdAtLabel: expect.any(String) })
    );
  });

  it("projects roster toolbar labels and action availability", () => {
    const toolbar = projectStrategyHubRosterToolbar({
      filteredStrategies: [{ graphId: "alpha_strategy" }, { graphId: "beta_strategy" }],
      selectedStrategyCount: 1,
      selectedForWorkspace: "alpha_strategy"
    });

    expect(toolbar.filteredCountLabel).toBe("2");
    expect(toolbar.selectedCountLabel).toBe("已选择 1 条");
    expect(toolbar.canOpenWorkspace).toBe(true);
    expect(toolbar.workspaceLabel).toBe("打开已选工作区");
  });

  it("projects roster rows with derived display labels", () => {
    const rows = projectStrategyHubRosterRows({
      filteredStrategies: [
        {
          graphId: "alpha_strategy",
          name: "Alpha strategy",
          health: { tone: "success", label: "Healthy" },
          activityLabel: "Recently compiled",
          lastActivityAt: 1710000300000,
          runCount: 2,
          backtestCount: 1,
          latestReturnRatio: 0.12,
          filePath: "storage/graphs/alpha_strategy.qs"
        }
      ],
      selectedStrategyIds: ["alpha_strategy"],
      selectedStrategy: { graphId: "alpha_strategy" }
    });

    expect(rows).toEqual([
      expect.objectContaining({
        graphId: "alpha_strategy",
        selected: true,
        active: true,
        runCountLabel: "2",
        backtestCountLabel: "1",
        latestReturnLabel: "+12.00%",
        hasFilePath: true
      })
    ]);
  });
});
