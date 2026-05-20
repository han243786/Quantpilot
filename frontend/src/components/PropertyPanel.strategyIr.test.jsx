import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { act, render, screen, waitFor } from "@testing-library/react";
import PropertyPanel from "./PropertyPanel";
import { useGraphStore } from "../store/graphStore";

function buildStrategyIr() {
  return {
    ir_version: "strategy_ir/v0",
    metadata: {
      strategy_id: "restricted_custom_v1",
      name: "Restricted Custom",
      summary: "Custom signal lowered into Core IR.",
      source: {
        source_type: "manual_paper_analysis",
        paper_title: "Restricted Custom",
        paper_reference: null
      }
    },
    signals: [
      {
        signal_id: "custom_signal",
        name: "Custom Signal",
        indicator: {
          kind: "custom",
          inputs: ["price_daily"],
          params: {
            custom_expr: {
              expr_version: "custom_expr/v1",
              signal_kind: "long_entry",
              predicate: {
                type: "comparison",
                op: "gt",
                left: {
                  type: "window_agg",
                  data_id: "price_daily",
                  field: "close",
                  aggregation: "sma",
                  window_size: 5
                },
                right: {
                  type: "window_agg",
                  data_id: "price_daily",
                  field: "close",
                  aggregation: "sma",
                  window_size: 20
                }
              }
            }
          }
        }
      }
    ],
    logic: {
      entry_rules: [{ rule_id: "entry_custom", condition: "custom_signal == true", action: "open_long" }],
      exit_rules: [],
      position_sizing: { method: "fixed_ratio", value: 0.1, unit: "portfolio_ratio" },
      rebalance_rule: null
    },
    risk_rules: {
      max_position_ratio: 0.2,
      stop_loss_ratio: 0.03,
      take_profit_ratio: null,
      max_drawdown_ratio: null,
      max_trades_per_day: null,
      notes: []
    },
    data_requirements: [
      {
        data_id: "price_daily",
        venue: "binance",
        symbol: "BTCUSDT",
        data_type: "kline",
        granularity: "1d",
        lookback: 200,
        fields: ["close"]
      }
    ],
    execution: {
      venue_type: "paper",
      order_type: "market",
      time_in_force: null,
      slippage_model: "fixed_bps",
      latency_assumption_ms: null,
      capital_base: null
    },
    gap_annotations: [],
    unknowns: []
  };
}

describe("PropertyPanel Strategy IR focus", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
      useGraphStore.getState().updateStrategyIrDraft(JSON.stringify(buildStrategyIr(), null, 2));
      useGraphStore.getState().applyStrategyIrSource();
      useGraphStore.getState().focusCompileDiagnostic("custom_signal.params.custom_expr");
    });
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("renders Strategy IR editor and focuses the targeted JSON segment", async () => {
    render(<PropertyPanel />);

    const card = screen.getByTestId("strategy-ir-editor-card");
    const editor = screen.getByLabelText("策略中间表示 JSON");
    expect(card).toContainElement(editor);
    expect(screen.getByTestId("strategy-ir-focus-target")).toHaveTextContent(
      "custom_signal.params.custom_expr"
    );

    await waitFor(() => {
      expect(editor.selectionStart).toBeLessThan(editor.selectionEnd);
    });
  });
});
