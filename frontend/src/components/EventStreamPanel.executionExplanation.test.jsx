import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, render, screen } from "@testing-library/react";
import EventStreamPanel from "./EventStreamPanel";
import { useGraphStore } from "../store/graphStore";

vi.mock("./AssetCandlesPanel", () => ({
  default: () => <div data-testid="asset-candles-panel-stub" />
}));

function buildGraph() {
  return {
    metadata: {
      name: "Execution Explanation Graph",
      graph_id: "execution_explanation_graph"
    },
    nodes: [],
    edges: [],
    validation_state: {
      is_valid: true,
      is_runnable: true,
      issue_counts: { error: 0, warning: 0, info: 0 },
      graph_issues: [],
      node_issues: {},
      edge_issues: {}
    },
    compile_summary: {}
  };
}

describe("EventStreamPanel execution and risk explanations", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
      useGraphStore.setState({
        graph: buildGraph(),
        runtime: {
          ...useGraphStore.getState().runtime,
          history: [],
          backtestHistory: [],
          account: {
            cash_balance: 10000,
            available_cash_balance: 9500,
            frozen_cash_balance: 500,
            open_orders: [],
            open_order_count: 0
          },
          events: [
            {
              event_id: "evt_risk_1",
              event_type: "RiskDecisionProduced",
              node_id: "risk_node",
              source_id: "risk.global",
              event_time_ms: 1_710_000_000_000,
              severity: "Warn",
              summary: "risk.global risk Clamp",
              payload: {
                status: "Clamp",
                limit_triggered: "max_single_weight",
                explanation_summary:
                  "Risk clamped sizing after triggering max_single_weight.",
                reason_text: "portfolio target clamped by max_single_weight",
                post_risk: {
                  concentration_ratio: 0.42,
                  max_symbol_net_exposure_ratio: 0.25,
                  portfolio_net_exposure_ratio: 0.6
                }
              }
            },
            {
              event_id: "evt_exec_1",
              event_type: "ExecutionPlanned",
              node_id: "execution_node",
              source_id: "risk.global",
              event_time_ms: 1_710_000_001_000,
              severity: "Info",
              summary: "execution planned with 1 orders",
              payload: {
                status: "Accepted",
                lifecycle_stage: "accepted",
                sizing_source: "portfolio_target_diff",
                order_type_decision_reason: "plan_executes_immediately_when_submitted",
                explanation_summary:
                  "Execution planned 1 order(s) from portfolio_target_diff using equity_notional_ratio sizing."
              }
            }
          ]
        },
        refreshRunHistory: vi.fn(),
        refreshBacktestHistory: vi.fn(),
        loadRunDetail: vi.fn(),
        loadBacktestDetail: vi.fn()
      });
    });
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("renders human-readable risk and execution explanations from event payloads", () => {
    render(<EventStreamPanel />);

    const riskRow = screen.getByTestId("event-feed-row-evt_risk_1");
    expect(screen.getByTestId("event-feed-explanation-evt_risk_1")).toHaveTextContent(
      "Risk clamped sizing after triggering max_single_weight."
    );
    expect(riskRow).toHaveTextContent("max_single_weight");
    expect(screen.getByTestId("event-feed-post-concentration-evt_risk_1")).toHaveTextContent(
      "+42.00%"
    );
    expect(screen.getByTestId("event-feed-post-symbol-net-evt_risk_1")).toHaveTextContent(
      "+25.00%"
    );
    expect(screen.getByTestId("event-feed-post-portfolio-net-evt_risk_1")).toHaveTextContent(
      "+60.00%"
    );

    const executionRow = screen.getByTestId("event-feed-row-evt_exec_1");
    expect(screen.getByTestId("event-feed-explanation-evt_exec_1")).toHaveTextContent(
      "Execution planned 1 order(s) from portfolio_target_diff using equity_notional_ratio sizing."
    );
    expect(executionRow).toHaveTextContent("portfolio_target_diff");
    expect(executionRow).toHaveTextContent("Accepted");
    expect(executionRow).toHaveTextContent("plan_executes_immediately_when_submitted");
  });
});
