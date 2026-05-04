import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import RuntimeDiagnosticsPanel from "./RuntimeDiagnosticsPanel";
import { useGraphStore } from "../store/graphStore";

describe("RuntimeDiagnosticsPanel", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("renders selected-node snapshots and allows switching active nodes", () => {
    const onSelectNode = vi.fn();
    const graph = {
      nodes: [
        {
          id: "data_feed",
          name: "Price Feed",
          type: "data",
          runtime_state: {
            status: "running",
            last_event_type: "DataUpdated",
            last_event_time: 1_710_000_000_000,
            last_message: "Market data updated",
            metrics: {},
            error: null
          }
        },
        {
          id: "execution",
          name: "Execution",
          type: "execution",
          runtime_state: {
            status: "error",
            last_event_type: "ExecutionFilled",
            last_event_time: 1_710_000_010_000,
            last_message: "Execution rejected",
            metrics: {},
            error: "Insufficient balance"
          }
        }
      ]
    };
    const runtime = {
      governance: {
        capability_hash: "sha256:diag-capability-1234567890abcdef",
        deployment_revision: "rev-diagnostics-20260428",
        strategy_version: "strategy-v17",
        parameter_version: "params-v4",
        governance_source: "current_runtime",
        permission_boundary: {
          model_version: "quantpilot/permission-boundary/v1",
          ai_write_policy: "proposal_only"
        }
      },
      highlightedNodeIds: ["data_feed", "execution"],
      events: [
        {
          event_id: "evt_exec_2",
          event_type: "ExecutionFilled",
          node_id: "execution",
          event_time_ms: 1_710_000_010_000,
          severity: "Error",
          summary: "Execution rejected",
          payload: {
            exec_status: "Rejected",
            order_id: "ord_002",
            qty: 0.25,
            price: 50_120
          }
        },
        {
          event_id: "evt_data_1",
          event_type: "DataUpdated",
          node_id: "data_feed",
          event_time_ms: 1_710_000_000_000,
          severity: "Warn",
          summary: "Market data updated",
          payload: {
            latest_price: 50_000,
            source_status: "Healthy",
            source_health: "Delayed",
            freshness_ms: 90_000,
            stale_after_ms: 60_000,
            source_latency_ms: 3_000,
            gap_count: 1,
            quality_flags: ["delayed_update", "gaps_detected"]
          }
        }
      ]
    };

    render(
      <RuntimeDiagnosticsPanel
        graph={graph}
        runtime={runtime}
        selectedNodeId="execution"
        onSelectNode={onSelectNode}
      />
    );

    expect(screen.getAllByText("Execution rejected").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Rejected").length).toBeGreaterThan(0);
    expect(screen.getByTestId("runtime-diagnostics-governance")).toHaveTextContent("能力边界");
    expect(screen.getByTestId("runtime-diagnostics-governance")).toHaveTextContent(
      "sha256:diag-c...abcdef"
    );
    expect(screen.getByTestId("runtime-diagnostics-governance")).toHaveTextContent(
      "proposal_only"
    );
    expect(screen.getByTestId("runtime-diagnostics-governance")).toHaveTextContent(
      "current_runtime"
    );
    fireEvent.click(screen.getByRole("button", { name: "Price Feed" }));
    expect(onSelectNode).toHaveBeenCalledWith("data_feed");
  });

  it("renders structured data quality rows from runtime diagnostics", () => {
    const graph = {
      nodes: [
        {
          id: "data_feed",
          name: "Price Feed",
          type: "data",
          runtime_state: {
            status: "warning",
            last_event_type: "RuntimeWarning",
            last_event_time: 1_710_000_010_000,
            last_message: "Data quality warning",
            metrics: {},
            error: null
          }
        }
      ]
    };
    const runtime = {
      diagnostics: {
        source: "backtest_event_log",
        default_selected_node_id: "data_feed",
        active_nodes: [
          {
            node_id: "data_feed",
            latest_event_type: "RuntimeWarning",
            latest_event_label: "数据告警",
            latest_event_time_ms: 1_710_000_010_000,
            event_count: 1
          }
        ],
        node_details: {
          data_feed: {
            node_id: "data_feed",
            latest_event: {
              event_id: "evt_data_quality_1",
              event_type: "RuntimeWarning",
              label: "数据告警",
              summary: "Data quality degraded",
              tone: "warning",
              severity: "Warn",
              event_time_ms: 1_710_000_010_000
            },
            explanation_summary: "BTCUSDT quote quality delayed with 2 missing intervals.",
            latest_input_rows: [],
            latest_output_rows: [],
            explanation_rows: [],
            data_quality_rows: [
              { key: "source_health", label: "源健康", value: "Delayed" },
              { key: "freshness_ms", label: "新鲜度(ms)", value: "120000" },
              { key: "gap_count", label: "缺口数量", value: "2" }
            ],
            risk_detail_rows: [
              {
                key: "limit_triggered",
                label: "触发限制",
                value: "max_portfolio_net_exposure_ratio"
              },
              {
                key: "post_risk.portfolio_net_exposure_ratio",
                label: "风控后组合净敞口",
                value: "0.4500"
              }
            ],
            order_detail_rows: [],
            latest_notice: null,
            recent_events: [],
            event_count: 1
          }
        }
      },
      events: []
    };

    render(<RuntimeDiagnosticsPanel graph={graph} runtime={runtime} selectedNodeId="data_feed" />);

    expect(screen.getByTestId("runtime-diagnostics-explanation-summary")).toHaveTextContent(
      "BTCUSDT quote quality delayed with 2 missing intervals."
    );
    expect(screen.getByTestId("runtime-diagnostics-data-quality")).toHaveTextContent("Delayed");
    expect(screen.getByTestId("runtime-diagnostics-data-quality")).toHaveTextContent("120000");
    expect(screen.getByTestId("runtime-diagnostics-data-quality")).toHaveTextContent("2");
  });

  it("renders backend-projected order and risk detail rows from runtime diagnostics", () => {
    const graph = {
      nodes: [
        {
          id: "execution",
          name: "Execution",
          type: "execution",
          runtime_state: {
            status: "running",
            last_event_type: "ExecutionPlanned",
            last_event_time: 1_710_000_010_000,
            last_message: "Execution planned",
            metrics: {},
            error: null
          }
        }
      ]
    };
    const runtime = {
      diagnostics: {
        source: "backtest_event_log",
        default_selected_node_id: "execution",
        active_nodes: [
          {
            node_id: "execution",
            latest_event_type: "ExecutionPlanned",
            latest_event_label: "执行计划",
            latest_event_time_ms: 1_710_000_010_000,
            event_count: 1
          }
        ],
        node_details: {
          execution: {
            node_id: "execution",
            latest_event: {
              event_id: "evt_exec_2",
              event_type: "ExecutionPlanned",
              label: "执行计划",
              summary: "Server-projected execution",
              tone: "info",
              severity: "Info",
              event_time_ms: 1_710_000_010_000
            },
            explanation_summary: "Execution plan sized from portfolio target diff.",
            latest_input_rows: [{ key: "qty", label: "数量", value: "0.2500" }],
            latest_output_rows: [
              { key: "remaining_qty", label: "剩余数量", value: "0.2500" }
            ],
            explanation_rows: [
              { key: "sizing_source", label: "定量来源", value: "portfolio_target_diff" }
            ],
            data_quality_rows: [],
            risk_detail_rows: [
              {
                key: "limit_triggered",
                label: "触发限制",
                value: "max_portfolio_net_exposure_ratio"
              },
              {
                key: "post_risk.portfolio_net_exposure_ratio",
                label: "风控后组合净敞口",
                value: "0.4500"
              }
            ],
            order_detail_rows: [
              {
                key: "order_type_decision_reason",
                label: "下单语义",
                value: "plan_executes_immediately_when_submitted"
              },
              {
                key: "preview_order_type",
                label: "首个订单类型",
                value: "Market"
              }
            ],
            latest_notice: null,
            recent_events: [],
            event_count: 1
          }
        }
      },
      events: []
    };

    render(<RuntimeDiagnosticsPanel graph={graph} runtime={runtime} selectedNodeId="execution" />);

    expect(
      screen.getByTestId("runtime-diagnostics-explanation-summary")
    ).toHaveTextContent("Execution plan sized from portfolio target diff.");
    expect(screen.getByTestId("runtime-diagnostics-order-detail")).toHaveTextContent(
      "plan_executes_immediately_when_submitted"
    );
    expect(screen.getByTestId("runtime-diagnostics-order-detail")).toHaveTextContent("Market");
    expect(screen.getByTestId("runtime-diagnostics-risk-detail")).toHaveTextContent(
      "max_portfolio_net_exposure_ratio"
    );
    expect(screen.getByTestId("runtime-diagnostics-risk-detail")).toHaveTextContent("0.4500");
  });
});
