import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import StrategyHubRosterTableSection from "./StrategyHubRosterTableSection";

const { navigateTo } = vi.hoisted(() => ({
  navigateTo: vi.fn()
}));

vi.mock("../router", () => ({
  navigateTo,
  strategyWorkspacePath: (strategyId) => `/strategies/${strategyId}`,
  strategyBacktestsPath: (strategyId) => `/strategies/${strategyId}/backtests`
}));

describe("StrategyHubRosterTableSection", () => {
  it("renders rows and routes row-level actions through the extracted ownership layers", () => {
    const model = {
      toggleStrategySelection: vi.fn(),
      setSelectedStrategyId: vi.fn(),
      openGraphFolder: vi.fn().mockResolvedValue(undefined),
      revealGraphFile: vi.fn().mockResolvedValue(undefined)
    };

    render(
      <StrategyHubRosterTableSection
        model={model}
        rosterRows={[
          {
            graphId: "alpha_strategy",
            name: "Alpha strategy",
            healthTone: "success",
            healthLabel: "Healthy",
            activityLabel: "Recently compiled",
            lastActivityLabel: "2024/3/9 12:00:00",
            runCountLabel: "2",
            backtestCountLabel: "1",
            latestReturnLabel: "+12.00%",
            selected: true,
            active: true,
            hasFilePath: true
          }
        ]}
      />
    );

    fireEvent.click(screen.getByLabelText("选择 Alpha strategy"));
    expect(model.toggleStrategySelection).toHaveBeenCalledWith("alpha_strategy");

    fireEvent.click(screen.getByRole("button", { name: /Alpha strategy alpha_strategy/i }));
    expect(model.setSelectedStrategyId).toHaveBeenCalledWith("alpha_strategy");

    fireEvent.click(screen.getByRole("button", { name: "打开 Alpha strategy 工作区" }));
    expect(navigateTo).toHaveBeenCalledWith("/strategies/alpha_strategy");

    fireEvent.click(screen.getByRole("button", { name: "打开 Alpha strategy 回测页" }));
    expect(navigateTo).toHaveBeenCalledWith("/strategies/alpha_strategy/backtests");

    fireEvent.click(screen.getByRole("button", { name: "打开 Alpha strategy 文件夹" }));
    expect(model.openGraphFolder).toHaveBeenCalledWith("alpha_strategy");

    fireEvent.click(screen.getByRole("button", { name: "打开 Alpha strategy 文件位置" }));
    expect(model.revealGraphFile).toHaveBeenCalledWith("alpha_strategy");
  });
});
