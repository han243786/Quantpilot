export const WORKSPACE_DASHBOARD_QUICK_ACTIONS = [
  {
    surfaceKey: "code",
    label: "进入构建",
    variant: "primary",
    testId: "dashboard-goto-build"
  },
  {
    surfaceKey: "research",
    label: "研究回测",
    variant: "ghost",
    testId: "dashboard-goto-research"
  },
  {
    surfaceKey: "monitor",
    label: "运行监控",
    variant: "ghost",
    testId: "dashboard-goto-monitor"
  },
  {
    surfaceKey: "source",
    label: "查看源码",
    variant: "ghost",
    testId: "dashboard-goto-source"
  }
];

export function resolveWorkspaceDashboardRuntime(storeRuntime, fallbackRuntime) {
  return storeRuntime ?? fallbackRuntime;
}

export function countWorkspaceDashboardBacktests(runtime) {
  return runtime?.backtestHistory?.length || 0;
}

export function canNavigateWorkspaceSurface(workspaceSurfaces = {}, surfaceKey) {
  return workspaceSurfaces?.[surfaceKey]?.enabled !== false;
}

export function getWorkspaceSurfaceNavigationTitle(workspaceSurfaces = {}, surfaceKey) {
  return (
    workspaceSurfaces?.[surfaceKey]?.blockReason ||
    workspaceSurfaces?.[surfaceKey]?.reason ||
    undefined
  );
}

export function buildWorkspaceDashboardQuickActions(workspaceSurfaces = {}) {
  return WORKSPACE_DASHBOARD_QUICK_ACTIONS.map((action) => ({
    ...action,
    className: `ad-btn ad-btn--${action.variant}`,
    disabled: !canNavigateWorkspaceSurface(workspaceSurfaces, action.surfaceKey),
    title: getWorkspaceSurfaceNavigationTitle(workspaceSurfaces, action.surfaceKey)
  }));
}

export function buildWorkspaceOverviewActionCards({
  graph,
  compileCounts,
  recentRuns,
  recentBacktests
}) {
  return [
    {
      kicker: "构建",
      title: "打开构建工作区",
      note: "只有需要结构调整、连线或源码修复时再进入。",
      meta: `${graph.nodes.length} 节点 / ${graph.edges.length} 连线`,
      tone: "muted",
      cta: "打开构建模式",
      targetTab: "code"
    },
    {
      kicker: "诊断",
      title: "查看编译与校验阻塞",
      note: "先从修复队列定位问题，再进入完整诊断。",
      meta: `${compileCounts.error} 错误 / ${compileCounts.warning} 警告`,
      tone: compileCounts.error > 0 ? "danger" : compileCounts.warning > 0 ? "warning" : "info",
      cta: "打开诊断",
      targetTab: "diagnostics"
    },
    {
      kicker: "研究",
      title: "打开模拟与回测历史",
      note: "从工作区进入回测索引和对比流程。",
      meta: `${recentRuns.length} 模拟 / ${recentBacktests.length} 回测`,
      tone: recentBacktests.length > 0 || recentRuns.length > 0 ? "info" : "muted",
      cta: "打开回测",
      targetRoute: "strategyBacktests"
    }
  ];
}
