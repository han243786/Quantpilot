import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import EventReplaySection from "./EventReplaySection";

const fetchRunReplay = vi.fn();
const fetchBacktestReplay = vi.fn();

vi.mock("../store/graphStoreRuntimeHistoryApi", () => ({
  fetchRunReplay: (...args) => fetchRunReplay(...args),
  fetchBacktestReplay: (...args) => fetchBacktestReplay(...args)
}));

describe("EventReplaySection", () => {
  beforeEach(() => {
    fetchRunReplay.mockReset();
    fetchBacktestReplay.mockReset();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it("loads backtest replay pages on demand and renders checkpoints", async () => {
    fetchBacktestReplay.mockResolvedValueOnce({
      kind: "backtest",
      record_id: "backtest_001",
      graph_id: "graph_001",
      source_event_count: 3,
      total_events: 3,
      cursor: 0,
      sequence_cursor: 1,
      limit: 12,
      window_end: 3,
      fill_event_count: 1,
      account: {
        cash_balance: 10_000,
        equity_estimate: 10_120
      },
      checkpoints: [
        {
          cursor: 0,
          sequence_cursor: 1,
          label: "1-3",
          event_id: "evt_001",
          event_time_ms: 1_710_000_000_000
        }
      ],
      events: [
        {
          sequence_no: 1,
          event: {
            event_type: "ExecutionPlanned",
            summary: "Execution planned",
            payload: {
              explanation_summary: "Plan derived from rebalance target."
            }
          }
        },
        {
          sequence_no: 2,
          event: {
            event_type: "ExecutionFilled",
            summary: "Order filled",
            payload: {
              explanation_summary: "Filled immediately."
            }
          }
        },
        {
          sequence_no: 3,
          event: {
            event_type: "PortfolioUpdated",
            summary: "Portfolio updated",
            payload: {}
          }
        }
      ],
      previous_cursor: null,
      next_cursor: null,
      previous_sequence_cursor: null,
      next_sequence_cursor: null
    });

    render(<EventReplaySection runtime={{ selectedBacktestId: "backtest_001" }} />);

    fireEvent.click(screen.getByTestId("event-replay-load"));

    await waitFor(() =>
      expect(fetchBacktestReplay).toHaveBeenCalledWith("backtest_001", {
        sequence_cursor: 1,
        limit: 12
      })
    );

    expect(screen.getByTestId("event-replay-window")).toHaveTextContent("1-3/3");
    expect(screen.getByTestId("event-replay-row-1")).toHaveTextContent("ExecutionPlanned");
    expect(screen.getByTestId("event-replay-row-2")).toHaveTextContent("Filled immediately.");
    expect(screen.getByTestId("event-replay-checkpoints")).toHaveTextContent("1-3");
  });

  it("supports paging through persisted run replay windows", async () => {
    fetchRunReplay
      .mockResolvedValueOnce({
        kind: "run",
        record_id: "run_001",
        graph_id: "graph_001",
        source_event_count: 8,
        total_events: 8,
        cursor: 0,
        sequence_cursor: 1,
        limit: 6,
        window_end: 6,
        fill_event_count: 0,
        account: {
          cash_balance: 5_000,
          equity_estimate: 5_010
        },
        checkpoints: [
          {
            cursor: 0,
            sequence_cursor: 1,
            label: "1-6",
            event_id: "evt_001",
            event_time_ms: 1_710_000_000_000
          },
          {
            cursor: 6,
            sequence_cursor: 7,
            label: "7-8",
            event_id: "evt_007",
            event_time_ms: 1_710_000_006_000
          }
        ],
        events: [
          {
            sequence_no: 1,
            event: { event_type: "RuntimeNotice", summary: "Started", payload: {} }
          },
          {
            sequence_no: 2,
            event: { event_type: "DataUpdated", summary: "First data", payload: {} }
          },
          {
            sequence_no: 3,
            event: { event_type: "IntentTriggered", summary: "Intent", payload: {} }
          },
          {
            sequence_no: 4,
            event: { event_type: "AgentDecisionProduced", summary: "Agent", payload: {} }
          },
          {
            sequence_no: 5,
            event: { event_type: "RiskDecisionProduced", summary: "Risk", payload: {} }
          },
          {
            sequence_no: 6,
            event: { event_type: "ExecutionPlanned", summary: "Planned", payload: {} }
          }
        ],
        previous_cursor: null,
        next_cursor: 6,
        previous_sequence_cursor: null,
        next_sequence_cursor: 7
      })
      .mockResolvedValueOnce({
        kind: "run",
        record_id: "run_001",
        graph_id: "graph_001",
        source_event_count: 8,
        total_events: 8,
        cursor: 6,
        sequence_cursor: 7,
        limit: 6,
        window_end: 8,
        fill_event_count: 1,
        account: {
          cash_balance: 5_050,
          equity_estimate: 5_080
        },
        checkpoints: [
          {
            cursor: 0,
            sequence_cursor: 1,
            label: "1-6",
            event_id: "evt_001",
            event_time_ms: 1_710_000_000_000
          },
          {
            cursor: 6,
            sequence_cursor: 7,
            label: "7-8",
            event_id: "evt_007",
            event_time_ms: 1_710_000_006_000
          }
        ],
        events: [
          {
            sequence_no: 7,
            event: { event_type: "ExecutionFilled", summary: "Filled", payload: {} }
          },
          {
            sequence_no: 8,
            event: { event_type: "PortfolioUpdated", summary: "Done", payload: {} }
          }
        ],
        previous_cursor: 0,
        next_cursor: null,
        previous_sequence_cursor: 1,
        next_sequence_cursor: null
      })
      .mockResolvedValueOnce({
        kind: "run",
        record_id: "run_001",
        graph_id: "graph_001",
        source_event_count: 8,
        total_events: 8,
        cursor: 0,
        sequence_cursor: 1,
        limit: 6,
        window_end: 6,
        fill_event_count: 0,
        account: {
          cash_balance: 5_000,
          equity_estimate: 5_010
        },
        checkpoints: [
          {
            cursor: 0,
            sequence_cursor: 1,
            label: "1-6",
            event_id: "evt_001",
            event_time_ms: 1_710_000_000_000
          },
          {
            cursor: 6,
            sequence_cursor: 7,
            label: "7-8",
            event_id: "evt_007",
            event_time_ms: 1_710_000_006_000
          }
        ],
        events: [
          {
            sequence_no: 1,
            event: { event_type: "RuntimeNotice", summary: "Started", payload: {} }
          }
        ],
        previous_cursor: null,
        next_cursor: 6,
        previous_sequence_cursor: null,
        next_sequence_cursor: 7
      });

    render(<EventReplaySection runtime={{ selectedHistoryRunId: "run_001" }} />);

    fireEvent.change(screen.getByTestId("event-replay-page-size"), {
      target: { value: "6" }
    });
    fireEvent.click(screen.getByTestId("event-replay-load"));

    await waitFor(() =>
      expect(fetchRunReplay).toHaveBeenNthCalledWith(1, "run_001", {
        sequence_cursor: 1,
        limit: 6
      })
    );

    fireEvent.click(screen.getByTestId("event-replay-next"));

    await waitFor(() =>
      expect(fetchRunReplay).toHaveBeenNthCalledWith(2, "run_001", {
        sequence_cursor: 7,
        limit: 6
      })
    );

    expect(screen.getByTestId("event-replay-window")).toHaveTextContent("7-8/8");
    expect(screen.getByTestId("event-replay-row-7")).toHaveTextContent("ExecutionFilled");

    fireEvent.click(screen.getByTestId("event-replay-prev"));

    await waitFor(() =>
      expect(fetchRunReplay).toHaveBeenNthCalledWith(3, "run_001", {
        sequence_cursor: 1,
        limit: 6
      })
    );

    expect(screen.getByTestId("event-replay-window")).toHaveTextContent("1-6/8");
  });
});
