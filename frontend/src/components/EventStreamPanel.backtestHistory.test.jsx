import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, render, screen } from "@testing-library/react";
import EventStreamPanel from "./EventStreamPanel";
import { useGraphStore } from "../store/graphStore";

vi.mock("./AssetCandlesPanel", () => ({
  default: () => <div data-testid="asset-candles-panel-stub" />
}));

// v0.5.0: 回测历史筛选/对比由独立路由页面 (BacktestDetailPage, StrategyBacktestsPage) 承载,
// EventStreamPanel 内仅保留 BacktestSummarySection

describe("EventStreamPanel backtest history filters and compare entry", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => { useGraphStore.setState(initialState, true); });
  });
  afterEach(() => {
    act(() => { useGraphStore.setState(initialState, true); });
  });

  it("renders the event panel with sidebar summary region", () => {
    act(() => {
      useGraphStore.setState({
        runtime: {
          ...useGraphStore.getState().runtime,
          events: [],
          history: [],
          backtestHistory: [],
          account: { cash_balance: 10000, available_cash_balance: 9500, frozen_cash_balance: 500, open_orders: [], open_order_count: 0 }
        }
      });
    });
    const { container } = render(<EventStreamPanel />);
    expect(container.querySelector(".event-sidebar")).not.toBeNull();
  });
});
