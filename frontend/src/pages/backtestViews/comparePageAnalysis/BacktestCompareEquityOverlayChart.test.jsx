import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import {
  BacktestCompareEquityOverlayChart,
  buildBacktestCompareEquityOverlayModel,
  resolveBacktestCompareEquityPoints
} from "./BacktestCompareEquityOverlayChart";

function buildDetail({ backtestId, equityCurve = [], benchmarkCurve = null }) {
  return {
    backtest_id: backtestId,
    backtest_artifacts: {
      equity_curve: equityCurve.map((equity) => ({ equity })),
      ...(benchmarkCurve
        ? { benchmark_equity_curve: benchmarkCurve.map((equity) => ({ equity })) }
        : {})
    }
  };
}

describe("BacktestCompareEquityOverlayChart", () => {
  it("normalizes artifact-wrapped equity point lists", () => {
    expect(
      resolveBacktestCompareEquityPoints({
        points: [{ equity: 100 }, { equity: 102 }]
      })
    ).toEqual([{ equity: 100 }, { equity: 102 }]);
    expect(resolveBacktestCompareEquityPoints(null)).toEqual([]);
  });

  it("builds a stable overlay model from left, right, and benchmark curves", () => {
    const model = buildBacktestCompareEquityOverlayModel([
      buildDetail({
        backtestId: "left-run-123",
        equityCurve: [100, 104],
        benchmarkCurve: [99, 101]
      }),
      buildDetail({
        backtestId: "right-run-456",
        equityCurve: [100, 102, 108]
      })
    ]);

    expect(model.leftLabel).toBe("left-run");
    expect(model.rightLabel).toBe("right-ru");
    expect(model.hasBenchmark).toBe(true);
    expect(model.rows).toEqual([
      { cycle: 0, a: 100, b: 100, benchmark: 99 },
      { cycle: 1, a: 104, b: 102, benchmark: 101 },
      { cycle: 2, a: null, b: 108, benchmark: null }
    ]);
  });

  it("renders the no-data state when no equity points are available", () => {
    render(<BacktestCompareEquityOverlayChart details={[]} />);

    expect(screen.getByText("无权益曲线数据")).toBeInTheDocument();
  });
});
