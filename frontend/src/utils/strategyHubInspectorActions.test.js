import { describe, expect, it, vi } from "vitest";
import {
  projectStrategyHubInspectorActionGroups,
  runStrategyHubInspectorAction
} from "./strategyHubInspectorActions";

const { navigateTo } = vi.hoisted(() => ({
  navigateTo: vi.fn()
}));

vi.mock("../router", () => ({
  navigateTo,
  strategyWorkspacePath: (strategyId) => `/strategies/${strategyId}`,
  strategyBacktestsPath: (strategyId) => `/strategies/${strategyId}/backtests`
}));

describe("strategyHubInspectorActions", () => {
  it("projects grouped inspector actions with semantic labels", () => {
    const groups = projectStrategyHubInspectorActionGroups({
      name: "Alpha strategy",
      graphId: "alpha_strategy"
    });

    expect(groups).toEqual([
      expect.objectContaining({
        key: "build",
        items: [expect.objectContaining({ key: "open-workspace", ariaLabel: "打开 Alpha strategy 工作区" })]
      }),
      expect.objectContaining({
        key: "research",
        items: [expect.objectContaining({ key: "open-backtests", ariaLabel: "打开 Alpha strategy 回测页" })]
      }),
      expect.objectContaining({
        key: "manage",
        items: [expect.objectContaining({ key: "refresh-strategy-data", ariaLabel: "刷新 Alpha strategy 策略数据" })]
      })
    ]);
  });

  it("routes inspector actions through the extracted semantic dispatcher", async () => {
    const model = {
      refreshRunHistory: vi.fn().mockResolvedValue(undefined),
      refreshBacktestHistory: vi.fn().mockResolvedValue(undefined)
    };
    const selectedStrategy = { name: "Alpha strategy", graphId: "alpha_strategy" };

    runStrategyHubInspectorAction(model, selectedStrategy, "open-workspace");
    expect(navigateTo).toHaveBeenCalledWith("/strategies/alpha_strategy");

    runStrategyHubInspectorAction(model, selectedStrategy, "open-backtests");
    expect(navigateTo).toHaveBeenCalledWith("/strategies/alpha_strategy/backtests");

    await runStrategyHubInspectorAction(model, selectedStrategy, "refresh-strategy-data");
    expect(model.refreshRunHistory).toHaveBeenCalledTimes(1);
    expect(model.refreshBacktestHistory).toHaveBeenCalledTimes(1);
  });
});
