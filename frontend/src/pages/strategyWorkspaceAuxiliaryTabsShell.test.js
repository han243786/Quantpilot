import { describe, expect, it } from "vitest";

import {
  buildSourceScenarioHttpError,
  buildSourceScenarioRunRequest,
  buildSourceScenarioStepPresentation,
  buildWorkspaceMonitorModel,
  buildWorkspaceResearchStripModel,
  extractSourceScenarioActualValue,
  formatWorkspaceMonitorCount,
  formatWorkspaceMonitorNumber,
  resolveWorkspaceRuntimeKindLabel,
  selectWorkspaceRuntimeEvents
} from "./strategyWorkspaceAuxiliaryTabsShell";

describe("strategyWorkspaceAuxiliaryTabsShell", () => {
  it("formats monitor primitives and runtime kind labels", () => {
    expect(formatWorkspaceMonitorNumber("12.345")).toBe("12.35");
    expect(formatWorkspaceMonitorNumber("nope")).toBe("-");
    expect(formatWorkspaceMonitorCount(1200)).toMatch(/1,200|1\s200|1.200/);
    expect(formatWorkspaceMonitorCount(undefined)).toBe("0");

    expect(resolveWorkspaceRuntimeKindLabel("backtest")).toBe("回测");
    expect(resolveWorkspaceRuntimeKindLabel("simulation")).toBe("模拟");
    expect(resolveWorkspaceRuntimeKindLabel("live")).toBe("实盘");
    expect(resolveWorkspaceRuntimeKindLabel("unknown")).toBe("未运行");
  });

  it("selects runtime events and projects monitor cards", () => {
    const timeline = Array.from({ length: 6 }, (_, index) => ({
      event_id: `timeline-${index}`,
      stage: `stage-${index}`
    }));
    const fallbackEvents = [{ event_id: "event-fallback" }];
    const model = buildWorkspaceMonitorModel({
      graph: { nodes: [{ type: "execution" }, { type: "risk" }] },
      runtime: {
        status: "running",
        runKind: "simulation",
        runId: "run-1",
        account: {
          equity_estimate: 1000,
          available_cash_balance: 500,
          frozen_cash_balance: 25,
          open_orders: [{ id: "order-1" }, { id: "order-2" }]
        },
        diagnostics: { ok: true },
        timeline,
        events: fallbackEvents
      },
      recentRuns: [{ created_at_ms: 123 }],
      issueQueue: [{ nodeType: "risk" }, { nodeType: "execution" }],
      formatTime: (value) => `time:${value}`
    });

    expect(selectWorkspaceRuntimeEvents({ timeline, events: fallbackEvents })).toEqual(timeline);
    expect(selectWorkspaceRuntimeEvents({ events: fallbackEvents })).toEqual(fallbackEvents);
    expect(model.statusMeta.tone).toBe("success");
    expect(model.stripPills).toMatchObject([
      { tone: "success" },
      { label: "模拟", tone: "muted" },
      { label: "2 挂单", tone: "info" }
    ]);
    expect(model.runtimeMetrics).toMatchObject([
      { label: "状态", tone: "success" },
      { label: "运行 ID", value: "run-1" },
      { label: "类型", value: "模拟" },
      { label: "最近运行", value: "time:123" }
    ]);
    expect(model.accountMetrics).toMatchObject([
      { label: "净值估算", value: "1000.00", tone: "success" },
      { label: "可用现金", value: "500.00" },
      { label: "冻结现金", value: "25.00" },
      { label: "挂单", value: "2" }
    ]);
    expect(model.riskMetrics).toMatchObject([
      { label: "风险阻塞", value: "1", tone: "danger" },
      { label: "执行节点", value: "1" },
      { label: "诊断", value: "已连接" },
      { label: "事件数", value: "6" }
    ]);
    expect(model.recentEvents.map((event) => event.event_id)).toEqual([
      "timeline-5",
      "timeline-4",
      "timeline-3",
      "timeline-2",
      "timeline-1"
    ]);
  });

  it("projects research strip and source scenario run state", () => {
    expect(buildWorkspaceResearchStripModel()).toEqual({
      title: "研究回测工作区",
      pills: [
        { label: "结果", tone: "muted" },
        { label: "时间线", tone: "info" },
        { label: "详情", tone: "muted" }
      ]
    });

    expect(buildSourceScenarioRunRequest("source text")).toEqual({
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ source: "source text" })
    });
    expect(buildSourceScenarioHttpError(500, "x".repeat(350)).error).toHaveLength(
      "Request failed (".length + String(500).length + "): ".length + 300
    );
    expect(buildSourceScenarioStepPresentation("passed")).toEqual({
      icon: "✓",
      color: "var(--ad-success)"
    });
    expect(buildSourceScenarioStepPresentation("failed")).toEqual({
      icon: "✗",
      color: "var(--ad-error)"
    });
    expect(buildSourceScenarioStepPresentation("skipped")).toEqual({
      icon: "⊘",
      color: "var(--ad-text-muted)"
    });
    expect(extractSourceScenarioActualValue("expected x actual: 42)")).toBe("42");
    expect(extractSourceScenarioActualValue("no actual")).toBeNull();
  });
});
