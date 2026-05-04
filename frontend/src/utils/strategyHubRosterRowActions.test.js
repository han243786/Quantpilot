import { describe, expect, it, vi } from "vitest";
import {
  projectStrategyHubRosterRowActionGroups,
  runStrategyHubRosterRowAction
} from "./strategyHubRosterRowActions";

const { navigateTo } = vi.hoisted(() => ({
  navigateTo: vi.fn()
}));

vi.mock("../router", () => ({
  navigateTo,
  strategyWorkspacePath: (strategyId) => `/strategies/${strategyId}`,
  strategyBacktestsPath: (strategyId) => `/strategies/${strategyId}/backtests`
}));

describe("strategyHubRosterRowActions", () => {
  it("projects grouped row actions with semantic labels and file-state gating", () => {
    const groups = projectStrategyHubRosterRowActionGroups({
      graphId: "alpha_strategy",
      name: "Alpha strategy",
      hasFilePath: false
    });

    expect(groups).toEqual([
      expect.objectContaining({
        key: "build",
        label: "构建",
        items: [expect.objectContaining({ key: "open-workspace", disabled: false })]
      }),
      expect.objectContaining({
        key: "research",
        label: "研究",
        items: [expect.objectContaining({ key: "open-backtests", disabled: false })]
      }),
      expect.objectContaining({
        key: "files",
        label: "文件",
        items: [expect.objectContaining({ key: "reveal-file", disabled: true })]
      }),
      expect.objectContaining({
        key: "manage",
        label: "管理",
        items: [expect.objectContaining({ key: "delete-strategy", disabled: false })]
      })
    ]);
    expect(groups[0].items[0].ariaLabel).toBe("打开 Alpha strategy 工作区");
  });

  it("routes row actions through the extracted semantic dispatcher", async () => {
    const model = {
      revealGraphFile: vi.fn().mockResolvedValue(undefined),
      deleteStrategy: vi.fn().mockResolvedValue(true)
    };
    const row = { graphId: "alpha_strategy", name: "Alpha strategy", hasFilePath: true };

    runStrategyHubRosterRowAction(model, row, "open-workspace");
    expect(navigateTo).toHaveBeenCalledWith("/strategies/alpha_strategy");

    runStrategyHubRosterRowAction(model, row, "open-backtests");
    expect(navigateTo).toHaveBeenCalledWith("/strategies/alpha_strategy/backtests");

    await runStrategyHubRosterRowAction(model, row, "reveal-file");
    expect(model.revealGraphFile).toHaveBeenCalledWith("alpha_strategy");

    await runStrategyHubRosterRowAction(model, row, "delete-strategy");
    expect(model.deleteStrategy).toHaveBeenCalledWith("alpha_strategy", "Alpha strategy");
  });
});
