import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import AppRouteHost from "./AppRouteHost";

vi.mock("../components/ErrorBoundary", () => ({
  default: ({ children }) => <div data-testid="error-boundary">{children}</div>
}));
vi.mock("./AppShellFallback", () => ({
  default: () => <div data-testid="route-fallback" />
}));
vi.mock("../pages/StrategyHubPage", () => ({
  default: () => <div data-testid="strategy-hub-page" />
}));
vi.mock("../pages/StrategyWorkspacePage", () => ({
  default: ({ strategyId }) => <div data-testid="strategy-workspace-page">{strategyId}</div>
}));
vi.mock("../pages/BacktestComparePage", () => ({
  default: ({ backtestIds, strategyId }) => (
    <div data-testid="backtest-compare-page">{`${strategyId}:${backtestIds.join("|")}`}</div>
  )
}));
vi.mock("../pages/NotFoundPage", () => ({
  default: ({ pathname }) => <div data-testid="not-found-page">{pathname}</div>
}));

describe("AppRouteHost", () => {
  it("renders the strategy hub route by default", async () => {
    render(<AppRouteHost route={{ name: "strategies" }} />);

    expect(await screen.findByTestId("strategy-hub-page")).toBeInTheDocument();
  });

  it("passes route params to workspace pages", async () => {
    render(<AppRouteHost route={{ name: "strategy-workspace", strategyId: "alpha" }} />);

    expect(await screen.findByTestId("strategy-workspace-page")).toHaveTextContent("alpha");
  });

  it("passes compare route params", async () => {
    render(
      <AppRouteHost
        route={{ name: "backtest-compare", strategyId: "s1", backtestIds: ["bt-1", "bt-2"] }}
      />
    );

    expect(await screen.findByTestId("backtest-compare-page")).toHaveTextContent("s1:bt-1|bt-2");
  });

  it("renders not-found pathnames", async () => {
    render(<AppRouteHost route={{ name: "not-found", pathname: "/missing" }} />);

    expect(await screen.findByTestId("not-found-page")).toHaveTextContent("/missing");
  });
});
