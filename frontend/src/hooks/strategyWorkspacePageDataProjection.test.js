import { describe, expect, it } from "vitest";

import {
  buildWorkspaceBacktestPreviewItems,
  buildWorkspaceDiagnosticsStatusHighlights,
  buildWorkspaceOverviewMetrics,
  buildWorkspaceOverviewStatusHighlights,
  buildWorkspaceRunPreviewItems,
  compileWorkspaceOutputsText,
  countWorkspaceDiagnostics,
  formatWorkspaceCount,
  formatWorkspacePercent,
  formatWorkspaceTime,
  resolveWorkspaceCompareSelection,
  resolveWorkspaceReadiness,
  selectRecentWorkspaceActivity
} from "./strategyWorkspacePageDataProjection";

describe("strategyWorkspacePageDataProjection", () => {
  it("formats workspace primitive values with existing fallbacks", () => {
    expect(formatWorkspaceTime(null)).toBe("-");
    expect(formatWorkspaceCount(Number.NaN)).toBe("0");
    expect(formatWorkspaceCount(1200)).toMatch(/1,200|1\s200|1.200/);
    expect(formatWorkspacePercent(0.1234)).toBe("+12.34%");
    expect(formatWorkspacePercent(-0.5)).toBe("-50.00%");
    expect(formatWorkspacePercent(undefined)).toBe("-");
    expect(compileWorkspaceOutputsText()).toBe("-");
    expect(
      compileWorkspaceOutputsText({
        data_sources: 2,
        intent_generators: 1,
        agents: 3,
        risk_controls: 0,
        executions: 4
      })
    ).toBe("2 data / 1 intent / 3 agent / 0 risk / 4 execution");
  });

  it("counts diagnostics and resolves readiness without React state", () => {
    expect(
      countWorkspaceDiagnostics([
        { severity: "warning" },
        { severity: "info" },
        { severity: "error" },
        {}
      ])
    ).toEqual({ error: 2, warning: 1, info: 1 });

    expect(
      resolveWorkspaceReadiness({
        isRunnable: true,
        isCompilable: true,
        issueCount: 1
      })
    ).toEqual({ tone: "danger", label: "Blocked" });
    expect(
      resolveWorkspaceReadiness({
        isRunnable: true,
        isCompilable: true,
        issueCount: 0
      })
    ).toEqual({ tone: "success", label: "Runnable" });
    expect(
      resolveWorkspaceReadiness({
        isRunnable: false,
        isCompilable: true,
        issueCount: 0
      })
    ).toEqual({ tone: "warning", label: "Compilable" });
    expect(
      resolveWorkspaceReadiness({
        isRunnable: false,
        isCompilable: false,
        issueCount: 0
      })
    ).toEqual({ tone: "muted", label: "Needs work" });
  });

  it("selects recent graph-scoped activity and preserves preview item payloads", () => {
    const activity = [
      { run_id: "old", graph_id: "alpha", created_at_ms: 10, compile_id: "c-old" },
      { run_id: "ignored", graph_id: "beta", created_at_ms: 999, compile_id: "c-ignored" },
      { run_id: "new", graph_id: "alpha", created_at_ms: 30 },
      { run_id: "middle", graph_id: "alpha", created_at_ms: 20, compile_id: "c-middle" }
    ];

    const recent = selectRecentWorkspaceActivity(activity, "alpha", 2);

    expect(recent.map((item) => item.run_id)).toEqual(["new", "middle"]);
    expect(buildWorkspaceRunPreviewItems(recent)).toMatchObject([
      { id: "new", title: "new", raw: recent[0] },
      { id: "middle", title: "middle", raw: recent[1] }
    ]);
    expect(buildWorkspaceRunPreviewItems(recent)[0].meta).toContain("No compile ID recorded");
  });

  it("builds overview metrics, status highlights, and backtest preview contracts", () => {
    const graph = {
      nodes: [{ id: "n1" }, { id: "n2" }],
      edges: [{ id: "e1" }],
      metadata: {
        runtime_binding: {
          last_compile_id: "compile-1"
        }
      }
    };
    const compileSummary = {
      outputs: { data_sources: 1, intent_generators: 1 },
      protocol_name: "protocol",
      compilable: true,
      config_hash: "hash-1"
    };
    const backtests = [
      {
        backtest_id: "bt-1",
        created_at_ms: 100,
        summary: { total_return_ratio: 0.25 }
      }
    ];

    expect(
      buildWorkspaceOverviewMetrics({
        graph,
        readiness: { label: "Runnable", tone: "success" },
        compileSummary,
        compileCounts: { error: 0, warning: 1, info: 2 },
        recentRuns: [{ run_id: "run-1" }],
        recentBacktests: backtests
      })
    ).toMatchObject([
      { label: "Readiness", value: "Runnable", note: "2 nodes / 1 edges" },
      { label: "Compile outputs", value: "1 data / 1 intent / 0 agent / 0 risk / 0 execution" },
      { label: "Diagnostics", value: "0 / 1 / 2", tone: "warning" },
      { label: "Runs and backtests", value: "1 runs / 1 backtests", tone: "info" }
    ]);

    expect(
      buildWorkspaceOverviewStatusHighlights({
        graph,
        compileSummary,
        lastRun: { created_at_ms: 200, compile_id: "compile-run" },
        lastBacktest: backtests[0]
      })
    ).toMatchObject([
      { label: "Latest compile ID", value: "compile-1", note: "hash-1" },
      { label: "Latest run", note: "compile-run" },
      { label: "Latest backtest", note: "bt-1" }
    ]);

    expect(buildWorkspaceBacktestPreviewItems(backtests)[0]).toMatchObject({
      id: "bt-1",
      title: "bt-1",
      raw: backtests[0]
    });
    expect(buildWorkspaceBacktestPreviewItems(backtests)[0].meta).toContain("total return +25.00%");
  });

  it("builds diagnostics highlights and compare selections from runtime shape", () => {
    expect(
      buildWorkspaceDiagnosticsStatusHighlights({
        issueQueueCountsSummary: { actionable: 2 },
        compileCounts: { error: 1, warning: 2, info: 3 },
        issueQueueSources: ["runtime", "validation"]
      })
    ).toEqual([
      {
        label: "Actionable fixes",
        value: "2",
        note: "Jump directly from the queue to the repair surface."
      },
      {
        label: "Compile diagnostics",
        value: "1 / 2 / 3",
        note: "error / warning / info"
      },
      {
        label: "Source lanes",
        value: "2",
        note: expect.stringContaining(" / ")
      }
    ]);

    expect(
      resolveWorkspaceCompareSelection(
        { backtestCompareSelection: { graph_a: ["bt-1", "bt-2"] } },
        "graph_a"
      )
    ).toEqual(["bt-1", "bt-2"]);
    expect(resolveWorkspaceCompareSelection({ backtestCompareSelection: ["legacy"] })).toEqual([
      "legacy"
    ]);
    expect(resolveWorkspaceCompareSelection({}, "graph_a")).toEqual([]);
  });
});
