import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import {
  datasetLabelsFromDetail,
  executionAssumptionsLabelFromDetail,
  formatPercent,
  formatRatio,
  formatValue,
  MetricPair
} from "./backtestAnalysisShared";

describe("backtestAnalysisShared", () => {
  it("formats numeric values and ratios for analysis surfaces", () => {
    expect(formatValue(12.34567)).toBe("12.3457");
    expect(formatPercent(0.125)).toBe("+12.50%");
    expect(formatRatio(-0.2)).toBe("-20.00%");
  });

  it("projects dataset and execution assumption labels from detail artifacts", () => {
    const detail = {
      backtest_artifacts: {
        manifest: {
          backtest_spec: {
            run_spec: {
              datasets: [
                { exchange: "binance", symbol: "BTCUSDT", interval: "1h" },
                { exchange: "binance", symbol: "ETHUSDT" }
              ]
            }
          }
        },
        metrics: {
          execution_assumptions: {
            list_tag: {
              label: "paper",
              sources_label: "default"
            }
          }
        }
      }
    };

    expect(datasetLabelsFromDetail(detail)).toEqual([
      "binance:BTCUSDT:1h",
      "binance:ETHUSDT:na"
    ]);
    expect(executionAssumptionsLabelFromDetail(detail)).toBe("paper (default)");
  });

  it("renders metric pairs as key-value lines", () => {
    render(<MetricPair label="Return" value="+12.50%" testId="analysis-metric-return" />);
    expect(screen.getByTestId("analysis-metric-return")).toHaveTextContent("Return");
    expect(screen.getByTestId("analysis-metric-return")).toHaveTextContent("+12.50%");
  });
});
