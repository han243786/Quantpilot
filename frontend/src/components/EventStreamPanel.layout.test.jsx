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

  it("marks detail mode so detail pages can use natural flow instead of editor panel rows", () => {
    const { container } = render(<EventStreamPanel detailMode />);

    expect(container.querySelector(".event-panel-detail")).not.toBeNull();
  });

  it("renders account and open order labels in clean Chinese", () => {
    act(() => {
      useGraphStore.setState({
        runtime: {
          ...useGraphStore.getState().runtime,
          account: {
            cash_balance: 10000,
            available_cash_balance: 380.4,
            frozen_cash_balance: 19619.6,
            open_order_count: 1,
            open_orders: [
              {
                order_id: "ord-risk-decision-node_agent_5-1764058384655-1764058384655-Okx",
                side: "Buy",
                remaining_qty: 0.4632,
                limit_price: 42314.44,
                reserved_cash: 19619.6,
                reserved_qty: 0
              }
            ]
          }
        }
      });
    });

    const { container } = render(<EventStreamPanel />);

    expect(screen.getAllByText("冻结现金").length).toBeGreaterThanOrEqual(2);
    expect(screen.getByText("买入")).toBeInTheDocument();
    expect(screen.getByText("剩余数量")).toBeInTheDocument();
    expect(screen.getByText("限价")).toBeInTheDocument();
    expect(screen.getByText("冻结仓位")).toBeInTheDocument();
    expect(container.textContent).not.toMatch(
      /\u6d94\u677f\u53c6|\u9357\u6827\u5687|\u934f\u2564\u7db1|\u95c4\u6119\u74b0|\u9350\u837b\u7ca8|\u6d60\u64b2\u7dbd|\u9436\u4f34\u567e/
    );
  });
});
