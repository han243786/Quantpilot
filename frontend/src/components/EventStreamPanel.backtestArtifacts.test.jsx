import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, render, screen } from "@testing-library/react";
import EventStreamPanel from "./EventStreamPanel";
import { useGraphStore } from "../store/graphStore";

vi.mock("./AssetCandlesPanel", () => ({
  default: () => <div data-testid="asset-candles-panel-stub" />
}));

// v0.5.0: BacktestSummarySection 子组件通过 useStrategyResearchModel 渲染回测摘要
describe("EventStreamPanel backtest artifact summary", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => { useGraphStore.setState(initialState, true); });
  });
  afterEach(() => {
    act(() => { useGraphStore.setState(initialState, true); });
  });

  it("renders the event panel shell with backtest sidebar", () => {
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
    expect(screen.getByTestId("event-panel-intro")).toBeInTheDocument();
    expect(container.querySelector(".event-sidebar")).not.toBeNull();
  });
});
