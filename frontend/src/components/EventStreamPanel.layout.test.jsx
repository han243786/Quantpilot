import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, render, screen } from "@testing-library/react";
import EventStreamPanel from "./EventStreamPanel";
import { useGraphStore } from "../store/graphStore";

vi.mock("./AssetCandlesPanel", () => ({
  default: () => <div data-testid="asset-candles-panel-stub" />
}));

describe("EventStreamPanel information architecture", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
      useGraphStore.setState({
        runtime: {
          ...useGraphStore.getState().runtime,
          history: [],
          backtestHistory: [],
          events: [],
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

  it("groups the panel into intro, event feed, history rails, and account cards", () => {
    const { container } = render(<EventStreamPanel />);

    expect(screen.getByTestId("event-panel-intro")).toBeInTheDocument();
    expect(screen.getByTestId("asset-candles-panel-stub")).toBeInTheDocument();
    expect(screen.getByTestId("event-feed-section")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-history-card")).toBeInTheDocument();
    expect(container.querySelector(".run-history-card")).not.toBeNull();
    expect(container.querySelectorAll(".open-orders-card").length).toBeGreaterThanOrEqual(1);
  });
});
