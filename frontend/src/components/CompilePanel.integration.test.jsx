import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import TopToolbar from "./TopToolbar";
import PropertyPanel from "./PropertyPanel";
import { useGraphStore } from "../store/graphStore";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";

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

describe("Compile panel integration", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
      useGraphStore.setState({
        graph: buildValidatedSampleGraph(initialState.registry)
      });
      useGraphStore.getState().updateStrategyIrDraft(JSON.stringify(buildStrategyIr(), null, 2));
      useGraphStore.getState().applyStrategyIrSource();
    });
    window.localStorage.clear();
    vi.unstubAllGlobals();
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
    window.localStorage.clear();
    vi.unstubAllGlobals();
  });

  it("shows Strategy IR diagnostics in the diagnostics section after clicking Compile", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url) => {
        if (url.endsWith("/api/strategy-ir/compile")) {
          return {
            ok: false,
            status: 400,
            text: async () =>
              JSON.stringify({
                error: "strategy_ir_compile_failed",
                message: "Strategy IR lowering failed",
                details: [
                  {
                    code: "CUSTOM006",
                    target: "custom_signal.params.custom_expr",
                    message:
                      "CUSTOM006 signal `custom_signal` uses undeclared input `other_data` in custom_expr",
                    reason: "Custom only allows the restricted expression whitelist and must lower into Core IR."
                  }
                ]
              })
          };
        }
        throw new Error(`Unexpected fetch: ${url}`);
      })
    );

    render(
      <>
        <TopToolbar />
        <PropertyPanel />
      </>
    );

    fireEvent.click(screen.getAllByTestId("toolbar-compile-action")[0]);

    await waitFor(() => {
      expect(screen.getByTestId("diagnostics-row-CUSTOM006")).toBeInTheDocument();
    });

    const diagnosticsSection = screen.getByTestId("property-section-diagnostics");
    const compileSummaryCard = within(diagnosticsSection).getByTestId("compile-summary-card");
    const diagnosticsPanel = within(diagnosticsSection).getByTestId("diagnostics-panel");
    const diagnosticsRow = within(diagnosticsPanel).getByTestId("diagnostics-row-CUSTOM006");
    const diagnosticsMeta = within(diagnosticsPanel).getByTestId("diagnostics-meta-CUSTOM006");
    const diagnosticsMessage = within(diagnosticsPanel).getByTestId("diagnostics-message-CUSTOM006");

    expect(compileSummaryCard).toBeInTheDocument();
    expect(diagnosticsMeta).toHaveTextContent("CUSTOM006");
    expect(diagnosticsMeta).toHaveTextContent("策略预检");
    expect(diagnosticsMessage).toHaveTextContent(
      "CUSTOM006 signal `custom_signal` uses undeclared input `other_data` in custom_expr"
    );
    expect(diagnosticsRow).toHaveTextContent(
      "Custom only allows the restricted expression whitelist and must lower into Core IR."
    );
  });
});
