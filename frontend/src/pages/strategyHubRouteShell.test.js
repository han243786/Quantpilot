import { describe, expect, it } from "vitest";

import {
  STRATEGY_HUB_ROUTE_HEADING,
  STRATEGY_HUB_VISUALLY_HIDDEN_HEADING_STYLE,
  buildStrategyHubFallbackProps,
  buildStrategyHubPageShellProps,
  getStrategyHubSectionDef
} from "./strategyHubRouteShell";

describe("strategyHubRouteShell", () => {
  it("builds the stable route shell props used by StrategyHubPage", () => {
    expect(buildStrategyHubPageShellProps()).toEqual({
      className: "strategy-hub-page",
      "data-testid": "strategy-hub-page"
    });
    expect(STRATEGY_HUB_ROUTE_HEADING).toBe("策略中心");
    expect(STRATEGY_HUB_VISUALLY_HIDDEN_HEADING_STYLE).toEqual({
      position: "absolute",
      width: "1px",
      height: "1px",
      overflow: "hidden",
      clip: "rect(0,0,0,0)",
      whiteSpace: "nowrap"
    });
  });

  it("resolves lazy-section fallback props from the route shell definition", () => {
    expect(getStrategyHubSectionDef("hero")).toEqual({
      id: "hero",
      fallbackTitle: "策略中心总览"
    });
    expect(buildStrategyHubFallbackProps("body")).toEqual({
      title: "策略中心工作区"
    });
    expect(buildStrategyHubFallbackProps("unknown")).toEqual({
      title: "策略中心面板加载中"
    });
  });
});
