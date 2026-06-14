import { describe, expect, it } from "vitest";
import {
  approvalsPath,
  backtestComparePath,
  backtestDetailPath,
  parseRoute,
  quantscriptPath,
  strategyBacktestsPath,
  strategyWorkspacePath,
} from "./routeContract";

describe("routeContract", () => {
  it("builds static and strategy paths", () => {
    expect(approvalsPath()).toBe("/approvals");
    expect(quantscriptPath()).toBe("/quantscript");
    expect(strategyWorkspacePath("btc alpha")).toBe("/strategies/btc%20alpha");
    expect(strategyBacktestsPath("btc alpha")).toBe(
      "/strategies/btc%20alpha/backtests"
    );
  });

  it("preserves backtest path query contracts", () => {
    expect(backtestDetailPath("bt-1", "btc alpha")).toBe(
      "/backtests/bt-1?strategy=btc+alpha"
    );
    expect(backtestComparePath(["bt-2", "bt-1", "bt-2"], "btc alpha")).toBe(
      "/backtests/compare?ids=bt-2%2Cbt-1&strategy=btc+alpha"
    );
  });

  it("parses shell, strategy, and backtest routes", () => {
    expect(parseRoute("/")).toEqual({ name: "strategies" });
    expect(parseRoute("/settings")).toEqual({ name: "settings" });
    expect(parseRoute("/strategies/btc%20alpha")).toEqual({
      name: "strategy-workspace",
      strategyId: "btc alpha",
    });
    expect(parseRoute("/strategies/btc%20alpha/backtests")).toEqual({
      name: "strategy-backtests",
      strategyId: "btc alpha",
    });
    expect(parseRoute("/backtests/bt-1", "?strategy=btc%20alpha")).toEqual({
      name: "backtest-detail",
      backtestId: "bt-1",
      strategyId: "btc alpha",
    });
  });

  it("parses backtest compare route query values", () => {
    expect(
      parseRoute(
        "/backtests/compare",
        `?ids=${encodeURIComponent("bt-1")},${encodeURIComponent(
          "bt-2"
        )}&strategy=${encodeURIComponent("btc alpha")}`
      )
    ).toEqual({
      name: "backtest-compare",
      backtestIds: ["bt-1", "bt-2"],
      strategyId: "btc alpha",
    });
  });
});
