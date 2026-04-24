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
      name: "Data Quality Graph",
      graph_id: "data_quality_graph"
    },
    nodes: [
      {
        id: "data_1",
        name: "BTC Feed",
        type: "data",
        runtime_state: {
          status: "warning",
          last_event_type: "RuntimeWarning",
          last_event_time: 1_710_000_000_000,
          last_message: "Data quality degraded",
          metrics: {
            source_health: "Delayed",
            freshness_ms: 120000,
            gap_count: 2
          },
          error: null
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

describe("EventStreamPanel data quality surface", () => {
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
              event_id: "evt_data_quality_1",
              event_type: "RuntimeWarning",
              node_id: "data_1",
              source_id: "BTCUSDT.quote",
              event_time_ms: 1_710_000_000_000,
              severity: "Warn",
              summary: "Data quality degraded",
              payload: {
                source_status: "Healthy",
                source_health: "Delayed",
                freshness_ms: 120000,
                stale_after_ms: 60000,
                gap_count: 2,
                quality_flags: ["delayed_update", "gaps_detected"],
                explanation_summary:
                  "BTCUSDT quote quality delayed with 2 missing intervals."
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

  it("renders explicit data quality meta rows on runtime warning events", () => {
    render(<EventStreamPanel />);

    expect(screen.getByTestId("event-feed-explanation-evt_data_quality_1")).toHaveTextContent(
      "BTCUSDT quote quality delayed with 2 missing intervals."
    );
    expect(screen.getByTestId("event-feed-source-health-evt_data_quality_1")).toHaveTextContent(
      "延迟"
    );
    expect(screen.getByTestId("event-feed-freshness-evt_data_quality_1")).toHaveTextContent(
      "120000"
    );
    expect(screen.getByTestId("event-feed-gap-count-evt_data_quality_1")).toHaveTextContent("2");
    expect(screen.getByTestId("event-feed-quality-flags-evt_data_quality_1")).toHaveTextContent(
      "delayed_update, gaps_detected"
    );
  });
});
