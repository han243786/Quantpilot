import { describe, expect, it } from "vitest";
import {
  backtestComparePath,
  backtestDetailPath,
  parseRoute,
  strategiesPath,
  strategyBacktestsPath,
  strategyWorkspacePath
} from "./router";

describe("router", () => {
  it("maps root and strategies paths to the strategy hub", () => {
    expect(parseRoute("/")).toEqual({ name: "strategies" });
    expect(parseRoute(strategiesPath())).toEqual({ name: "strategies" });
  });

  it("maps strategy workspace paths with decoded ids", () => {
    expect(parseRoute(strategyWorkspacePath("btc alpha"))).toEqual({
      name: "strategy-workspace",
      strategyId: "btc alpha"
    });
  });

  it("maps strategy backtest index paths with decoded ids", () => {
    expect(parseRoute(strategyBacktestsPath("btc alpha"))).toEqual({
      name: "strategy-backtests",
      strategyId: "btc alpha"
    });
  });

  it("preserves backtest compare ids", () => {
    const route = parseRoute(
      "/backtests/compare",
      `?ids=${encodeURIComponent("bt-1")},${encodeURIComponent("bt-2")}&strategy=${encodeURIComponent("btc alpha")}`
    );

    expect(route).toEqual({
      name: "backtest-compare",
      backtestIds: ["bt-1", "bt-2"],
      strategyId: "btc alpha"
    });
    expect(backtestComparePath(["bt-2", "bt-1", "bt-2"], "btc alpha")).toBe(
      "/backtests/compare?ids=bt-2%2Cbt-1&strategy=btc+alpha"
    );
  });

  it("preserves strategy context on backtest detail paths", () => {
    expect(parseRoute("/backtests/bt-1", "?strategy=btc%20alpha")).toEqual({
      name: "backtest-detail",
      backtestId: "bt-1",
      strategyId: "btc alpha"
    });
    expect(backtestDetailPath("bt-1", "btc alpha")).toBe(
      "/backtests/bt-1?strategy=btc+alpha"
    );
  });
});
