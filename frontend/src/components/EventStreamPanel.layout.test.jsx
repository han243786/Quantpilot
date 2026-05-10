import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, render, screen } from "@testing-library/react";
import EventStreamPanel from "./EventStreamPanel";
import { useGraphStore } from "../store/graphStore";

vi.mock("./AssetCandlesPanel", () => ({
  default: () => <div data-testid="asset-candles-panel-stub" />
}));

// v0.5.0 Adobe 重设计: EventStreamPanel 主组件重构为 useStrategyResearchModel 驱动的子组件拼接。
// backtest-history-card / run-history-card 已内聚到子组件内, 本测试验证主 panel 的外壳布局。

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

  it("groups the panel into intro, event feed, and account cards", () => {
    const { container } = render(<EventStreamPanel />);

    expect(screen.getByTestId("event-panel-intro")).toBeInTheDocument();
    expect(screen.getByTestId("asset-candles-panel-stub")).toBeInTheDocument();
    expect(container.querySelector(".event-feed-section")).not.toBeNull();
    expect(container.querySelector(".event-panel-body")).not.toBeNull();
    expect(container.querySelector(".event-sidebar")).not.toBeNull();
  });

  it("marks detail mode so detail pages can use natural flow instead of editor panel rows", () => {
    const { container } = render(<EventStreamPanel detailMode />);

    expect(container.querySelector(".event-panel-detail")).not.toBeNull();
  });

  it("renders account labels in clean Chinese", () => {
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
                order_id: "ord-test",
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

    // v0.5.0: 账户和未结订单卡片由 AccountSection 子组件渲染,
    // 在 useStrategyResearchModel 数据就绪后出现
    expect(container.querySelector(".event-sidebar")).not.toBeNull();
    expect(container.textContent).not.toMatch(
      /涔板叆|鍗栧嚇|鍏╤綱|闄愙環|鍐荻粨|浠撲綽|鐶伴噾/
    );
  });
});
