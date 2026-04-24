import { describe, expect, it } from "vitest";
import { projectStrategyHubRecentRunsView } from "./strategyHubRecentRunsView";

describe("strategyHubRecentRunsView", () => {
  it("projects section copy and derived item tone", () => {
    const view = projectStrategyHubRecentRunsView([
      {
        runId: "run_alpha_01",
        createdAtLabel: "2024/3/9 12:00:00",
        compileIdLabel: "compile_alpha_001"
      }
    ]);

    expect(view.title).toBe("近期模拟");
    expect(view.emptyText).toBe("这条策略暂无近期模拟。");
    expect(view.items).toEqual([
      expect.objectContaining({
        runId: "run_alpha_01",
        statusTone: "info"
      })
    ]);
  });
});
