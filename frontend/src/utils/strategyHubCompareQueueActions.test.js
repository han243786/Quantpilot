import { describe, expect, it, vi } from "vitest";
import {
  projectStrategyHubCompareQueueView,
  runStrategyHubCompareQueueAction
} from "./strategyHubCompareQueueActions";

const { navigateTo } = vi.hoisted(() => ({
  navigateTo: vi.fn()
}));

vi.mock("../router", () => ({
  navigateTo,
  backtestComparePath: (ids, strategyId = "") =>
    strategyId
      ? `/backtests/compare?ids=${ids.join(",")}&strategy=${strategyId}`
      : `/backtests/compare?ids=${ids.join(",")}`
}));

describe("strategyHubCompareQueueActions", () => {
  it("projects compare queue copy, chips, and action gating", () => {
    const emptyView = projectStrategyHubCompareQueueView({ selectedIds: [], canCompare: false });
    expect(emptyView.emptyLabel).toBe("未选择回测");
    expect(emptyView.actions[0].disabled).toBe(true);
    expect(emptyView.actions[1].disabled).toBe(true);

    const readyView = projectStrategyHubCompareQueueView({
      selectedIds: ["bt_alpha_01", "bt_beta_02"],
      canCompare: true
    });
    expect(readyView.chips).toEqual(["bt_alpha_01", "bt_beta_02"]);
    expect(readyView.actions[1].disabled).toBe(false);
  });

  it("routes compare queue actions through the extracted dispatcher", () => {
    const onClearSelection = vi.fn();
    const compareQueue = {
      selectedIds: ["bt_alpha_01", "bt_beta_02"],
      canCompare: true
    };

    runStrategyHubCompareQueueAction(
      "alpha_strategy",
      compareQueue,
      "clear-selection",
      onClearSelection
    );
    expect(onClearSelection).toHaveBeenCalledTimes(1);

    runStrategyHubCompareQueueAction(
      "alpha_strategy",
      compareQueue,
      "open-compare",
      onClearSelection
    );
    expect(navigateTo).toHaveBeenCalledWith(
      "/backtests/compare?ids=bt_alpha_01,bt_beta_02&strategy=alpha_strategy"
    );
  });
});
