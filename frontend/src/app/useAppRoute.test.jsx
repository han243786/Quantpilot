import { act, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { getInitialAppRoute, useAppRoute } from "./useAppRoute";

function RouteProbe() {
  const route = useAppRoute();
  return <div data-testid="route-name">{route.name}</div>;
}

describe("useAppRoute", () => {
  afterEach(() => {
    window.history.pushState({}, "", "/strategies");
  });

  it("reads the initial route from browser location", () => {
    window.history.pushState({}, "", "/strategies/alpha");

    expect(getInitialAppRoute()).toEqual({
      name: "strategy-workspace",
      strategyId: "alpha"
    });
  });

  it("updates route on popstate", () => {
    window.history.pushState({}, "", "/strategies");

    render(<RouteProbe />);
    expect(screen.getByTestId("route-name")).toHaveTextContent("strategies");

    act(() => {
      window.history.pushState({}, "", "/alerts");
      window.dispatchEvent(new PopStateEvent("popstate"));
    });

    expect(screen.getByTestId("route-name")).toHaveTextContent("alerts");
  });

  it("redirects root to strategies path", async () => {
    window.history.pushState({}, "", "/");

    render(<RouteProbe />);

    await waitFor(() => expect(window.location.pathname).toBe("/strategies"));
    expect(screen.getByTestId("route-name")).toHaveTextContent("strategies");
  });
});
