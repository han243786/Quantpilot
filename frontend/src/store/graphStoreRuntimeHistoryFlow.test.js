import { describe, expect, it, vi } from "vitest";
import {
  buildRuntimeHistoryFailureMessage,
  warmRuntimeSidebarDataFlow
} from "./graphStoreRuntimeHistoryFlow";

describe("graphStoreRuntimeHistoryFlow", () => {
  it("keeps the backend reason when formatting runtime-history failures", () => {
    const message = buildRuntimeHistoryFailureMessage("run_history", {
      status: 503,
      message: "backend unavailable"
    });
    expect(message).toContain("backend unavailable");
  });

  it("uses the corrected Chinese fallback copy for backtest detail failures", () => {
    const message = buildRuntimeHistoryFailureMessage("backtest_detail", null);
    expect(message).toContain("加载回测详情失败");
  });

  it("warms only the missing runtime sidebars", async () => {
    const refreshRunHistory = vi.fn(async () => ["run-1"]);
    const refreshBacktestHistory = vi.fn(async () => ["bt-1"]);
    const refreshExperimentHistory = vi.fn(async () => ["experiment-1"]);
    const get = () => ({
      runtime: {
        historyStatus: "ready",
        history: ["run-1"],
        backtestHistoryStatus: "idle",
        backtestHistory: [],
        experimentsStatus: "idle",
        experiments: []
      },
      refreshRunHistory,
      refreshBacktestHistory,
      refreshExperimentHistory
    });

    const result = await warmRuntimeSidebarDataFlow(get);
    expect(refreshRunHistory).not.toHaveBeenCalled();
    expect(refreshBacktestHistory).toHaveBeenCalledTimes(1);
    expect(refreshExperimentHistory).toHaveBeenCalledTimes(1);
    expect(result).toEqual([["bt-1"], ["experiment-1"]]);
  });
});
