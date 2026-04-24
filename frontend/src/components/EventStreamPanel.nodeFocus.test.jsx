import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen } from "@testing-library/react";
import EventStreamPanel from "./EventStreamPanel";
import { useGraphStore } from "../store/graphStore";

vi.mock("./AssetCandlesPanel", () => ({
  default: () => <div data-testid="asset-candles-panel-stub" />
}));

function buildGraph() {
  return {
    metadata: {
      name: "Runtime Diagnostics Graph",
      graph_id: "runtime_diagnostics_graph"
    },
    nodes: [
      {
        id: "data_feed",
        name: "Price Feed",
        type: "data",
        runtime_state: {
          status: "running",
          last_event_type: "DataUpdated",
          last_event_time: 1_710_000_000_000,
          last_message: "Data tick applied"
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
          last_message: "Order rejected"
        }
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

describe("EventStreamPanel runtime diagnostics node focus", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
      useGraphStore.setState({
        graph: buildGraph(),
        runtime: {
          ...useGraphStore.getState().runtime,
          status: "running",
          diagnostics: {
            source: "run_detail",
            default_selected_node_id: "execution",
            active_nodes: [
              {
                node_id: "execution",
                latest_event_type: "ExecutionFilled",
                latest_event_label: "执行成交",
                latest_event_time_ms: 1_710_000_010_000,
                event_count: 1
              },
              {
                node_id: "data_feed",
                latest_event_type: "DataUpdated",
                latest_event_label: "数据更新",
                latest_event_time_ms: 1_710_000_000_000,
                event_count: 1
              }
            ],
            node_details: {
              execution: {
                node_id: "execution",
                latest_event: {
                  event_id: "evt_exec_1",
                  event_type: "ExecutionFilled",
                  label: "执行成交",
                  summary: "Execution rejected",
                  tone: "danger",
                  severity: "Error",
                  event_time_ms: 1_710_000_010_000
                },
                latest_input_rows: [],
                latest_output_rows: [],
                latest_notice: null,
                recent_events: [],
                event_count: 1
              },
              data_feed: {
                node_id: "data_feed",
                latest_event: {
                  event_id: "evt_data_1",
                  event_type: "DataUpdated",
                  label: "数据更新",
                  summary: "Data tick applied",
                  tone: "info",
                  severity: "Info",
                  event_time_ms: 1_710_000_000_000
                },
                latest_input_rows: [],
                latest_output_rows: [],
                latest_notice: null,
                recent_events: [],
                event_count: 1
              }
            }
          },
          events: [
            {
              event_id: "evt_exec_1",
              event_type: "ExecutionFilled",
              node_id: "execution",
              event_time_ms: 1_710_000_010_000,
              severity: "Error",
              summary: "Execution rejected",
              payload: {
                exec_status: "Rejected"
              }
            },
            {
              event_id: "evt_data_1",
              event_type: "DataUpdated",
              node_id: "data_feed",
              event_time_ms: 1_710_000_000_000,
              severity: "Info",
              summary: "Data tick applied",
              payload: {
                latest_price: 50000
              }
            }
          ],
          history: [],
          backtestHistory: [],
          account: {
            cash_balance: 10000,
            available_cash_balance: 9500,
            frozen_cash_balance: 500,
            open_orders: [],
            open_order_count: 0
          }
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

  it("uses the structured diagnostics node focus in the event stream and allows clearing it", () => {
    render(<EventStreamPanel />);

    expect(screen.getByTestId("event-feed-node-scope")).toHaveTextContent("Execution");
    expect(screen.getByTestId("event-feed-row-evt_exec_1")).toBeInTheDocument();
    expect(screen.queryByTestId("event-feed-row-evt_data_1")).not.toBeInTheDocument();

    fireEvent.click(screen.getByTestId("event-feed-node-chip-all"));

    expect(screen.getByTestId("event-feed-node-scope")).not.toHaveTextContent("Execution");
    expect(screen.getByTestId("event-feed-row-evt_exec_1")).toBeInTheDocument();
    expect(screen.getByTestId("event-feed-row-evt_data_1")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("event-feed-node-chip-data_feed"));

    expect(screen.getByTestId("event-feed-row-evt_data_1")).toBeInTheDocument();
    expect(screen.queryByTestId("event-feed-row-evt_exec_1")).not.toBeInTheDocument();
  });
});
