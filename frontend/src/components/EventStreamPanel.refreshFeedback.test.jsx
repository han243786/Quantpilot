import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen } from "@testing-library/react";
import EventStreamPanel from "./EventStreamPanel";
import { useGraphStore } from "../store/graphStore";

vi.mock("./AssetCandlesPanel", () => ({
  default: () => <div data-testid="asset-candles-panel-stub" />
}));

function buildGraph(overrides = {}) {
  return {
    metadata: {
      name: "Refresh Feedback Graph",
      graph_id: "refresh_feedback_graph",
      ...(overrides.metadata || {})
    },
    nodes: [],
    edges: [],
    validation_state: {
      is_valid: true,
      is_runnable: true,
      issue_counts: { error: 0, warning: 0, info: 0 },
      graph_issues: [],
      node_issues: {},
      edge_issues: {},
      ...(overrides.validation_state || {})
    },
    compile_summary: {},
    ...overrides
  };
}

describe("EventStreamPanel refresh feedback", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
      useGraphStore.setState({
        graph: buildGraph(),
        runtime: {
          ...useGraphStore.getState().runtime,
          history: [],
          historyStatus: "ready",
          backtestHistory: [],
          backtestHistoryStatus: "ready",
          backendError: null
        },
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

  it("shows a success notice after refreshing run history", async () => {
    const refreshRunHistory = vi.fn(async () => {
      useGraphStore.setState((state) => ({
        runtime: {
          ...state.runtime,
          historyStatus: "ready",
          history: [
            {
              run_id: "run_refresh_001",
              graph_id: "refresh_feedback_graph",
              compile_id: "compile_refresh_001",
              created_at_ms: 1_700_000_000_000,
              status: "completed",
              event_count: 4
            }
          ],
          backendError: null
        }
      }));
      return useGraphStore.getState().runtime.history;
    });

    act(() => {
      useGraphStore.setState({ refreshRunHistory });
    });

    render(<EventStreamPanel />);

    await act(async () => {
      fireEvent.click(screen.getByTestId("run-history-refresh"));
    });

    expect(refreshRunHistory).toHaveBeenCalledTimes(1);
    expect(screen.getByTestId("event-panel-notice")).toHaveTextContent(
      /Run history refreshed\.|运行历史已刷新。/
    );
  });

  it("shows the unified failure notice after refreshing backtest history fails", async () => {
    const failureMessage =
      "原因：backend unavailable。后续：检查后端可用性，并在运行时 API 可访问后重新刷新回测历史。";
    const refreshBacktestHistory = vi.fn(async () => {
      useGraphStore.setState((state) => ({
        runtime: {
          ...state.runtime,
          backtestHistoryStatus: "error",
          backendError: failureMessage
        }
      }));
      return [];
    });

    act(() => {
      useGraphStore.setState({ refreshBacktestHistory });
    });

    render(<EventStreamPanel />);

    await act(async () => {
      fireEvent.click(screen.getByTestId("backtest-history-refresh"));
    });

    expect(refreshBacktestHistory).toHaveBeenCalledTimes(1);
    expect(screen.getByTestId("event-panel-notice")).toHaveTextContent(failureMessage);
  });
});
