import { describe, expect, it } from "vitest";

import {
  buildWorkspaceDashboardQuickActions,
  buildWorkspaceOverviewActionCards,
  canNavigateWorkspaceSurface,
  countWorkspaceDashboardBacktests,
  getWorkspaceSurfaceNavigationTitle,
  resolveWorkspaceDashboardRuntime
} from "./strategyWorkspaceDashboardOverviewShell";

describe("strategyWorkspaceDashboardOverviewShell", () => {
  it("projects dashboard runtime and quick-action surface state", () => {
    const storeRuntime = { status: "running", backtestHistory: [{ id: "bt-1" }] };
    const fallbackRuntime = { status: "idle", backtestHistory: [] };

    expect(resolveWorkspaceDashboardRuntime(storeRuntime, fallbackRuntime)).toBe(storeRuntime);
    expect(resolveWorkspaceDashboardRuntime(null, fallbackRuntime)).toBe(fallbackRuntime);
    expect(countWorkspaceDashboardBacktests(storeRuntime)).toBe(1);
    expect(countWorkspaceDashboardBacktests()).toBe(0);

    const surfaces = {
      code: { enabled: false, blockReason: "code disabled" },
      monitor: { enabled: false, reason: "monitor deferred" },
      research: { enabled: true }
    };

    expect(canNavigateWorkspaceSurface(surfaces, "code")).toBe(false);
    expect(canNavigateWorkspaceSurface(surfaces, "research")).toBe(true);
    expect(canNavigateWorkspaceSurface(surfaces, "source")).toBe(true);
    expect(getWorkspaceSurfaceNavigationTitle(surfaces, "code")).toBe("code disabled");
    expect(getWorkspaceSurfaceNavigationTitle(surfaces, "monitor")).toBe("monitor deferred");

    expect(buildWorkspaceDashboardQuickActions(surfaces)).toMatchObject([
      {
        surfaceKey: "code",
        label: "进入构建",
        className: "ad-btn ad-btn--primary",
        disabled: true,
        title: "code disabled"
      },
      {
        surfaceKey: "research",
        label: "研究回测",
        className: "ad-btn ad-btn--ghost",
        disabled: false
      },
      {
        surfaceKey: "monitor",
        label: "运行监控",
        className: "ad-btn ad-btn--ghost",
        disabled: true,
        title: "monitor deferred"
      },
      {
        surfaceKey: "source",
        label: "查看源码",
        className: "ad-btn ad-btn--ghost",
        disabled: false
      }
    ]);
  });

  it("projects overview action cards without binding React callbacks", () => {
    const graph = {
      nodes: [{ id: "n1" }, { id: "n2" }],
      edges: [{ id: "e1" }]
    };

    expect(
      buildWorkspaceOverviewActionCards({
        graph,
        compileCounts: { error: 1, warning: 2 },
        recentRuns: [{ run_id: "run-1" }],
        recentBacktests: []
      })
    ).toEqual([
      {
        kicker: "构建",
        title: "打开构建工作区",
        note: "只有需要结构调整、连线或源码修复时再进入。",
        meta: "2 节点 / 1 连线",
        tone: "muted",
        cta: "打开构建模式",
        targetTab: "code"
      },
      {
        kicker: "诊断",
        title: "查看编译与校验阻塞",
        note: "先从修复队列定位问题，再进入完整诊断。",
        meta: "1 错误 / 2 警告",
        tone: "danger",
        cta: "打开诊断",
        targetTab: "diagnostics"
      },
      {
        kicker: "研究",
        title: "打开模拟与回测历史",
        note: "从工作区进入回测索引和对比流程。",
        meta: "1 模拟 / 0 回测",
        tone: "info",
        cta: "打开回测",
        targetRoute: "strategyBacktests"
      }
    ]);
  });
});
