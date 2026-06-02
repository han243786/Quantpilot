import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { BacktestDetailReplayOutputExplanationSections } from "./BacktestDetailReplayOutputExplanationSections";

const t = (value) => value;

describe("BacktestDetailReplayOutputExplanationSections", () => {
  it("renders replay previews, output references, and explanation entries", () => {
    render(
      <BacktestDetailReplayOutputExplanationSections
        t={t}
        curvePreview={[
          {
            ts_ms: 1_700_000_000_000,
            equity: 10_000,
            cash_balance: 9_000,
            net_notional: 1_000
          }
        ]}
        tradePreview={[
          {
            fill_id: "fill_smoke_001",
            cycle_name: "slow",
            side: "buy",
            filled_qty: 0.2,
            filled_price: 50_250,
            fee_paid: 1.5
          }
        ]}
        equityCurveArtifactId="equity_curve_artifact_001"
        tradeLedgerArtifactId="trade_ledger_artifact_001"
        outputArtifacts={[
          {
            artifact_id: "metrics_artifact_001",
            kind: "metrics",
            file_name: "metrics.json"
          }
        ]}
        riskExplanationEntries={[
          {
            nodeId: "node_risk_5",
            nodeName: "Risk Guard",
            explanationSummary: "Risk clamp applied before execution.",
            rows: [{ key: "limit", label: "limit", value: "max_single_weight" }]
          }
        ]}
        orderExplanationEntries={[
          {
            nodeId: "node_execution_7",
            nodeName: "Execution Desk",
            explanationSummary: "Execution plan sized from portfolio target diff.",
            rows: [{ key: "source", label: "source", value: "portfolio_target_diff" }]
          }
        ]}
      />
    );

    expect(screen.getByTestId("backtest-detail-replay-preview")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-detail-output-artifacts")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-detail-explanations")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-detail-equity-card")).toHaveTextContent("equity_curve_artifact_001");
    expect(screen.getByTestId("backtest-detail-trade-card")).toHaveTextContent("fill_smoke_001");
    expect(screen.getByTestId("backtest-detail-output-card")).toHaveTextContent("metrics.json");
    expect(screen.getByTestId("backtest-detail-risk-card-entry-node_risk_5")).toHaveTextContent("Risk Guard");
    expect(screen.getByTestId("backtest-detail-order-card-entry-node_execution_7")).toHaveTextContent("Execution Desk");
  });

  it("keeps empty replay, output, and explanation states visible", () => {
    render(<BacktestDetailReplayOutputExplanationSections t={t} />);

    expect(screen.getByTestId("backtest-detail-equity-card")).toHaveTextContent("这次回测没有可用的权益曲线样本。");
    expect(screen.getByTestId("backtest-detail-trade-card")).toHaveTextContent("这次回测没有记录成交。");
    expect(screen.getByTestId("backtest-detail-output-card")).toHaveTextContent("这次回测没有记录任何输出文件引用。");
    expect(screen.getByTestId("backtest-detail-risk-card")).toHaveTextContent("当前回测详情还没有可展示的风控解释。");
    expect(screen.getByTestId("backtest-detail-order-card")).toHaveTextContent("当前回测详情还没有可展示的订单解释。");
  });
});
