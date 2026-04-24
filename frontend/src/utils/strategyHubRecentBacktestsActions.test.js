import { describe, expect, it, vi } from "vitest";
import {
  projectStrategyHubRecentBacktestActionGroup,
  runStrategyHubRecentBacktestAction
} from "./strategyHubRecentBacktestsActions";

const { navigateTo } = vi.hoisted(() => ({
  navigateTo: vi.fn()
}));

vi.mock("../router", () => ({
  navigateTo,
  backtestDetailPath: (backtestId, strategyId = "") =>
    strategyId ? `/backtests/${backtestId}?strategy=${strategyId}` : `/backtests/${backtestId}`
}));

describe("strategyHubRecentBacktestsActions", () => {
  it("projects semantic backtest actions from item state", () => {
    const group = projectStrategyHubRecentBacktestActionGroup({
      backtestId: "bt_alpha_01",
      checked: true
    });

    expect(group).toEqual(
      expect.objectContaining({
        label: "研究",
        items: [
          expect.objectContaining({
            key: "open-detail",
            ariaLabel: "打开 bt_alpha_01 详情"
          }),
          expect.objectContaining({
            key: "toggle-compare",
            label: "已选择",
            ariaLabel: "将 bt_alpha_01 从对比中移除",
            selected: true
          })
        ]
      })
    );
  });

  it("routes detail and compare actions through the extracted dispatcher", () => {
    const onToggleCompare = vi.fn();
    const item = { backtestId: "bt_alpha_01", checked: false };

    runStrategyHubRecentBacktestAction("alpha_strategy", item, "open-detail", onToggleCompare);
    expect(navigateTo).toHaveBeenCalledWith("/backtests/bt_alpha_01?strategy=alpha_strategy");

    runStrategyHubRecentBacktestAction("alpha_strategy", item, "toggle-compare", onToggleCompare);
    expect(onToggleCompare).toHaveBeenCalledWith("bt_alpha_01");
  });
});
