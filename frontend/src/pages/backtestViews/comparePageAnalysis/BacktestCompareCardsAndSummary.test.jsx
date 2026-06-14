import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { buildBacktestSuccessFixture } from "../../../test/fixtures/runtime/backtestSuccess";
import {
  BacktestCompareCardsSection,
  BacktestCompareSummarySidebar,
  buildBacktestCompareCardModel,
  buildBacktestCompareSummaryCardModel
} from "./BacktestCompareCardsAndSummary";

function buildDetail({ backtestId, graphId = "graph_compare", compileId = "compile_compare" }) {
  return buildBacktestSuccessFixture({
    backtestId,
    graphId,
    compileId
  }).detailResponse;
}

describe("BacktestCompareCardsAndSummary", () => {
  it("builds stable card and summary models", () => {
    const left = buildDetail({ backtestId: "backtest_left" });
    const right = buildDetail({ backtestId: "backtest_right" });

    expect(buildBacktestCompareCardModel(left)).toMatchObject({
      backtestId: "backtest_left",
      graphId: "graph_compare",
      compileId: "compile_compare",
      replaySource: "deterministic_mock"
    });

    expect(
      buildBacktestCompareSummaryCardModel({
        details: [left, right],
        resolvedStrategyId: "graph_compare",
        summary: {
          returnDelta: 0.1,
          drawdownDelta: -0.02,
          tradeDelta: 3
        }
      })
    ).toEqual({
      strategyId: "graph_compare",
      returnDelta: 0.1,
      drawdownDelta: -0.02,
      tradeDelta: 3,
      comparedBacktests: "backtest_left vs backtest_right"
    });
  });

  it("renders compare cards and delegates detail navigation", () => {
    const onOpenDetail = vi.fn();
    render(
      <BacktestCompareCardsSection
        details={[
          buildDetail({ backtestId: "backtest_left" }),
          buildDetail({ backtestId: "backtest_right" })
        ]}
        onOpenDetail={onOpenDetail}
      />
    );

    expect(screen.getByTestId("backtest-compare-card-grid")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-compare-card-backtest_left")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("backtest-compare-open-detail-backtest_left"));
    expect(onOpenDetail).toHaveBeenCalledWith("backtest_left");
  });

  it("renders the summary sidebar from the whitebox summary model", () => {
    render(
      <BacktestCompareSummarySidebar
        details={[
          buildDetail({ backtestId: "backtest_left" }),
          buildDetail({ backtestId: "backtest_right" })
        ]}
        resolvedStrategyId="graph_compare"
        summary={{
          returnDelta: 0.1,
          drawdownDelta: -0.02,
          tradeDelta: 3
        }}
      />
    );

    const summaryCard = screen.getByTestId("backtest-compare-summary-card");
    expect(summaryCard.textContent).toContain("graph_compare");
    expect(summaryCard.textContent).toContain("backtest_left vs backtest_right");
  });
});
