import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, render, screen } from "@testing-library/react";
import EventStreamPanel from "./EventStreamPanel";
import { useGraphStore } from "../store/graphStore";

vi.mock("./AssetCandlesPanel", () => ({
  default: () => <div data-testid="asset-candles-panel-stub" />
}));

// v0.5.0: 运行/回测历史刷新和通知由 useStrategyResearchModel hook 驱动,
// 反馈通过 event-panel-notice 和 event-panel-backend-error 显示

describe("EventStreamPanel refresh feedback", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => { useGraphStore.setState(initialState, true); });
  });
  afterEach(() => {
    act(() => { useGraphStore.setState(initialState, true); });
  });

  it("shows notice after successful refresh", () => {
    act(() => {
      useGraphStore.setState({
        runtime: {
          ...useGraphStore.getState().runtime,
          historyStatus: "success",
          history: [{ run_id: "run_1" }],
          backtestHistory: [],
          events: [],
          account: { cash_balance: 10000, available_cash_balance: 9500, frozen_cash_balance: 500, open_orders: [], open_order_count: 0 }
        }
      });
    });
    render(<EventStreamPanel />);
    expect(screen.getByTestId("event-panel-intro")).toBeInTheDocument();
  });

  it("shows backend error notice when refresh fails", () => {
    act(() => {
      useGraphStore.setState({
        runtime: {
          ...useGraphStore.getState().runtime,
          historyStatus: "error",
          history: [],
          backtestHistoryStatus: "error",
          backtestHistory: [],
          events: [],
          backendError: "后端不可达",
          account: { cash_balance: 10000, available_cash_balance: 9500, frozen_cash_balance: 500, open_orders: [], open_order_count: 0 }
        }
      });
    });
    render(<EventStreamPanel />);
    expect(screen.getByTestId("event-panel-backend-error")).toBeInTheDocument();
  });
});
