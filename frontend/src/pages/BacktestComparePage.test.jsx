import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen, within } from "@testing-library/react";
import BacktestComparePage from "./BacktestComparePage";
import { buildBacktestSuccessFixture } from "../test/fixtures/runtime/backtestSuccess";
import { navigateTo } from "../router";

vi.mock("../router", () => ({
  navigateTo: vi.fn(),
  backtestDetailPath: (backtestId, strategyId = "") =>
    strategyId ? `/backtests/${backtestId}?strategy=${strategyId}` : `/backtests/${backtestId}`,
  strategiesPath: () => "/strategies",
  strategyBacktestsPath: (strategyId) => `/strategies/${strategyId}/backtests`,
  strategyWorkspacePath: (strategyId) => `/strategies/${strategyId}`
}));

describe("BacktestComparePage", () => {
  beforeEach(() => {
    navigateTo.mockReset();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("renders strategy-context comparison sections and opens detail pages", async () => {
    const left = buildBacktestSuccessFixture({
      graphId: "graph_left",
      compileId: "compile_left",
      backtestId: "backtest_left"
    }).detailResponse;

    const rightFixture = buildBacktestSuccessFixture({
      graphId: "graph_right",
      compileId: "compile_right",
      backtestId: "backtest_right"
    });
    const right = {
      ...rightFixture.detailResponse,
      backtest_artifacts: {
        ...rightFixture.detailResponse.backtest_artifacts,
        metrics: {
          ...rightFixture.detailResponse.backtest_artifacts.metrics,
          summary: {
            ...rightFixture.detailResponse.backtest_artifacts.metrics.summary,
            total_return_ratio: 0.08,
            max_drawdown_ratio: 0.03,
            trade_count: 3,
            final_equity: 10800
          }
        }
      }
    };

    vi.stubGlobal(
      "fetch",
      vi.fn(async (input) => {
        const url = String(input);
        const payload = url.includes("backtest_left") ? left : right;
        return {
          ok: true,
          json: async () => payload
        };
      })
    );

    await act(async () => {
      render(<BacktestComparePage backtestIds={["backtest_left", "backtest_right"]} />);
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(screen.getByTestId("backtest-compare-page")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-compare-hero")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-compare-hero-actions")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-compare-card-grid")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-compare-card-backtest_left")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-compare-card-backtest_right")).toBeInTheDocument();

    const summaryCard = screen.getByTestId("backtest-compare-summary-card");
    expect(summaryCard).toBeInTheDocument();
    expect(summaryCard.textContent).toContain("backtest_left vs backtest_right");

    const leftCard = screen.getByTestId("backtest-compare-card-backtest_left");
    expect(leftCard.textContent).toContain("backtest_left");

    const detailButton = screen.getByTestId("backtest-compare-open-detail-backtest_left");
    fireEvent.click(detailButton);
    expect(navigateTo).toHaveBeenCalledWith("/backtests/backtest_left");

    const compareGrid = screen.getByTestId("backtest-compare-card-grid");
    expect(within(compareGrid).getAllByTestId(/backtest-compare-card-/)).toHaveLength(2);
  });

  it("shows an error banner when fewer than two ids are selected", async () => {
    let container;
    await act(async () => {
      ({ container } = render(<BacktestComparePage backtestIds={["only_one"]} />));
      await Promise.resolve();
    });

    expect(screen.getByTestId("backtest-compare-page")).toBeInTheDocument();
    expect(container.querySelector(".analysis-status-banner--error")).not.toBeNull();
  });
});
