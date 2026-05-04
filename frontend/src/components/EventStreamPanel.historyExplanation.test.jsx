import { act, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import EventStreamPanel from "./EventStreamPanel";
import { useGraphStore } from "../store/graphStore";

vi.mock("./AssetCandlesPanel", () => ({
  default: () => <div data-testid="asset-candles-panel-stub" />
}));

function buildGraph() {
  return {
    metadata: {
      name: "History Explanation Graph",
      graph_id: "history_explanation_graph"
    },
    nodes: [
      { id: "risk_node", name: "Risk guard", type: "risk", runtime_state: { status: "warning" } },
      { id: "data_node", name: "Price feed", type: "data", runtime_state: { status: "warning" } },
      {
        id: "execution_node",
        name: "Execution",
        type: "execution",
        runtime_state: { status: "running" }
      }
    ],
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

describe("EventStreamPanel history explanations", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
      useGraphStore.setState({
        graph: buildGraph(),
        runtime: {
          ...useGraphStore.getState().runtime,
          runId: "run_explain_001",
          selectedBacktestId: "backtest_explain_001",
          history: [
            {
              run_id: "run_explain_001",
              graph_id: "history_explanation_graph",
              compile_id: "compile_run_001",
              created_at_ms: 1_700_000_000_000,
              status: "completed",
              event_count: 4
            }
          ],
          historyStatus: "ready",
          backtestHistory: [
            {
              backtest_id: "backtest_explain_001",
              graph_id: "history_explanation_graph",
              compile_id: "compile_backtest_001",
              created_at_ms: 1_700_000_100_000,
              protocol_name: "quantpilot/runtime-config/v1",
              config_hash: "config_hash_001",
              event_count: 4,
              summary: {
                total_return_ratio: 0.12,
                max_drawdown_ratio: 0.03,
                trade_count: 2
              },
              filters: {
                replay_source: "deterministic_mock",
                dataset_labels: ["Binance:BTCUSDT:1m"],
                execution_assumptions_tag: {
                  label: "fee=10 slip=5 lat=0",
                  sources_label: "fee:backend slip:backend lat:backend"
                },
                started_at_ms: 1_700_000_000_000,
                ended_at_ms: 1_700_000_100_000
              }
            }
          ],
          backtestHistoryStatus: "ready",
          diagnostics: {
            source: "runtime_events",
            default_selected_node_id: "execution_node",
            active_nodes: [
              {
                node_id: "risk_node",
                latest_event_type: "RiskDecisionProduced",
                latest_event_label: "风控决策",
                latest_event_time_ms: 1_700_000_050_000,
                event_count: 1
              },
              {
                node_id: "execution_node",
                latest_event_type: "ExecutionPlanned",
                latest_event_label: "执行计划",
                latest_event_time_ms: 1_700_000_060_000,
                event_count: 1
              }
            ],
            node_details: {
              risk_node: {
                node_id: "risk_node",
                latest_event: null,
                explanation_summary: "Risk clamp applied before execution.",
                latest_input_rows: [],
                latest_output_rows: [],
                explanation_rows: [],
                data_quality_rows: [],
                risk_detail_rows: [
                  { key: "limit_triggered", label: "触发限制", value: "max_single_weight" }
                ],
                order_detail_rows: [],
                latest_notice: null,
                recent_events: [],
                event_count: 1
              },
              data_node: {
                node_id: "data_node",
                latest_event: null,
                explanation_summary: "BTCUSDT quote quality delayed with 2 missing intervals.",
                latest_input_rows: [],
                latest_output_rows: [],
                explanation_rows: [],
                data_quality_rows: [
                  { key: "source_health", label: "源健康度", value: "Delayed" },
                  { key: "gap_count", label: "缺口数量", value: "2" }
                ],
                risk_detail_rows: [],
                order_detail_rows: [],
                latest_notice: null,
                recent_events: [],
                event_count: 1
              },
              execution_node: {
                node_id: "execution_node",
                latest_event: null,
                explanation_summary: "Execution plan sized from portfolio target diff.",
                latest_input_rows: [],
                latest_output_rows: [],
                explanation_rows: [],
                data_quality_rows: [],
                risk_detail_rows: [],
                order_detail_rows: [
                  {
                    key: "order_type_decision_reason",
                    label: "下单语义",
                    value: "plan_executes_immediately_when_submitted"
                  }
                ],
                latest_notice: null,
                recent_events: [],
                event_count: 1
              }
            }
          }
        }
      });
    });
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("surfaces selected run/backtest risk and order explanations in history cards", () => {
    render(<EventStreamPanel />);

    expect(screen.getByTestId("run-history-risk-explanations")).toHaveTextContent(
      "max_single_weight"
    );
    expect(screen.getByTestId("run-history-order-explanations")).toHaveTextContent(
      "plan_executes_immediately_when_submitted"
    );
    expect(screen.getByTestId("run-history-data-quality")).toHaveTextContent("Delayed");
    expect(screen.getByTestId("backtest-history-risk-explanations")).toHaveTextContent(
      "Risk clamp applied before execution."
    );
    expect(screen.getByTestId("backtest-history-order-explanations")).toHaveTextContent(
      "Execution plan sized from portfolio target diff."
    );
    expect(screen.getByTestId("backtest-history-data-quality")).toHaveTextContent("2");
  });
});
