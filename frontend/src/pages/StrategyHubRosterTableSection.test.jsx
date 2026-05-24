import { fireEvent, render, screen, waitFor } from "@testing-library/react";
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
  it("renders rows and routes row-level actions through the extracted ownership layers", async () => {
    const model = {
      toggleStrategySelection: vi.fn(),
      setSelectedStrategyId: vi.fn(),
      revealGraphFile: vi.fn().mockResolvedValue(undefined),
      deleteStrategy: vi.fn().mockResolvedValue(true)
    };

    const { container } = render(
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

    const rosterHead = container.querySelector(".strategy-directory-table__head--roster");
    expect([...rosterHead.children].map((item) => item.textContent)).toEqual([
      "",
      "策略",
      "状态",
      "活动",
      "模拟",
      "回测",
      "最近收益",
      "主操作",
      "更多"
    ]);
    expect(container.querySelectorAll(".strategy-row__actions > button")).toHaveLength(2);

    fireEvent.click(screen.getByLabelText("选择策略 Alpha strategy（alpha_strategy）"));
    expect(model.toggleStrategySelection).toHaveBeenCalledWith("alpha_strategy");

    fireEvent.click(screen.getByRole("button", { name: /Alpha strategy alpha_strategy/i }));
    expect(model.setSelectedStrategyId).toHaveBeenCalledWith("alpha_strategy");

    fireEvent.click(screen.getByRole("button", { name: "打开策略 Alpha strategy（alpha_strategy）的工作区" }));
    expect(navigateTo).toHaveBeenCalledWith("/strategies/alpha_strategy");

    fireEvent.click(screen.getByTestId("strategy-hub-roster-action-alpha_strategy-more"));

    fireEvent.click(screen.getByRole("button", { name: "打开策略 Alpha strategy（alpha_strategy）的回测页" }));
    expect(navigateTo).toHaveBeenCalledWith("/strategies/alpha_strategy/backtests");

    fireEvent.click(screen.getByTestId("strategy-hub-roster-action-alpha_strategy-more"));

    fireEvent.click(screen.getByRole("button", { name: "打开策略 Alpha strategy（alpha_strategy）的文件位置" }));
    expect(model.revealGraphFile).toHaveBeenCalledWith("alpha_strategy");

    fireEvent.click(screen.getByTestId("strategy-hub-roster-action-alpha_strategy-more"));

    fireEvent.click(screen.getByRole("button", { name: "删除策略 Alpha strategy（alpha_strategy）" }));
    await waitFor(() => {
      expect(model.deleteStrategy).toHaveBeenCalledWith("alpha_strategy", "Alpha strategy");
    });
  });

  it("shows row-level feedback when a roster action fails", async () => {
    const model = {
      toggleStrategySelection: vi.fn(),
      setSelectedStrategyId: vi.fn(),
      revealGraphFile: vi.fn().mockResolvedValue(undefined),
      deleteStrategy: vi.fn().mockRejectedValue(new Error("DELETE /api/graphs failed"))
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

    fireEvent.click(screen.getByTestId("strategy-hub-roster-action-alpha_strategy-more"));
    fireEvent.click(screen.getByRole("button", { name: "删除策略 Alpha strategy（alpha_strategy）" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("DELETE /api/graphs failed");
  });
});
