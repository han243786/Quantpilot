import { Suspense, lazy, useEffect, useMemo, useRef, useState } from "react";
import "./strategy-workspace.css";
import {
  navigateTo,
  strategyBacktestsPath,
  strategiesPath
} from "../router";
import { useI18n } from "../i18n";
import { buildWorkspaceIssueQueue } from "../utils/strategyWorkspaceIssueQueue";
import { useStrategyWorkspaceSharedModel } from "../hooks/useStrategyWorkspaceSharedModel";
import { useStrategyWorkspaceUiState } from "../hooks/useStrategyWorkspaceUiState";
import { useStrategyWorkspacePageData } from "../hooks/useStrategyWorkspacePageData";
import { useGraphStore } from "../store/graphStore";
import { projectCapabilityView } from "../capabilities/capabilityProjection";
import TopToolbar from "../components/TopToolbar";
import { StrategyRouteBar } from "./BacktestAnalysisLayout";

const StrategyWorkspaceDashboard = lazy(() => import("./StrategyWorkspaceDashboard"));
const StrategyWorkspaceOverviewTab = lazy(() => import("./StrategyWorkspaceOverviewTab"));
const StrategyWorkspaceCodeTab = lazy(() => import("./StrategyWorkspaceCodeTab"));
const StrategyWorkspaceDiagnosticsTab = lazy(() => import("./StrategyWorkspaceDiagnosticsTab"));
const StrategyWorkspaceResearchTab = lazy(() => import("./StrategyWorkspaceResearchTab"));
const StrategyWorkspaceMonitorTab = lazy(() => import("./StrategyWorkspaceMonitorTab"));
const StrategyWorkspaceDebugTab = lazy(() => import("./StrategyWorkspaceDebugTab"));
const StrategyWorkspaceSourceTab = lazy(() => import("./StrategyWorkspaceSourceTab"));

// v1.3.3: 常量定义在组件外部，渲染时通过t()包裹
const WORKSPACE_TAB_DEFS = [
  { id: "dashboard", labelKey: "总览", kickerKey: "任务总览" },
  { id: "code", labelKey: "构建", kickerKey: "构建工作区" },
  { id: "research", labelKey: "研究回测", kickerKey: "研究回测工作区" },
  { id: "monitor", labelKey: "运行监控", kickerKey: "运行监控工作区" },
  { id: "source", labelKey: "源码", kickerKey: "源码工作区" }
];
const CODE_INSPECTOR_DEFS = [
  { id: "params", label: "配置" },
  { id: "diagnostics", label: "检查" },
  { id: "code", label: "源码" }
];

export default function StrategyWorkspacePage({ strategyId }) {
  const { t } = useI18n();
  const {
    graph,
    runtime,
    selectedNodeId,
    selectedEdgeId,
    selectedCompileDiagnosticTarget,
    capabilities,
    capabilityStatus,
    capabilitySource,
    capabilityMessage,
    loadGraphById
  } = useStrategyWorkspaceSharedModel();
  const stopRuntime = useGraphStore((s) => s.stopRuntime);
  const diagnosticFocusRequested = useGraphStore((s) => s.diagnosticFocusRequested);
  const clearDiagnosticFocus = () => useGraphStore.setState({ diagnosticFocusRequested: false });

  // v1.0.5: 组件卸载时清理 SSE 连接, 防止连接泄漏
  // 用 ref 跟踪最新状态以避免过期闭包
  const runtimeRef = useRef(runtime);
  runtimeRef.current = runtime;
  useEffect(() => {
    return () => {
      if (runtimeRef.current?.status === "running") {
        stopRuntime();
      }
    };
  }, []);
  const graphId = graph.metadata?.graph_id || "";
  const compileSummary = graph.compile_summary || {};
  const compileDiagnostics = Array.isArray(compileSummary.diagnostics)
    ? compileSummary.diagnostics
    : [];
  const issueQueue = useMemo(
    () => buildWorkspaceIssueQueue(graph, compileDiagnostics),
    [compileDiagnostics, graph]
  );
  const ui = useStrategyWorkspaceUiState({
    strategyId,
    graphId,
    loadGraphById,
    selectedEdgeId,
    selectedCompileDiagnosticTarget,
    issueQueue,
    codeInspectorPanels: CODE_INSPECTOR_DEFS
  });
  const pageData = useStrategyWorkspacePageData({
    graph,
    runtime,
    strategyId,
    selectedNodeId,
    selectedEdgeId,
    issueQueue,
    activeTab: ui.activeTab,
    canvasWorkspaceLaneId: ui.canvasWorkspaceContext.laneId,
    codeInspectorPanels: CODE_INSPECTOR_DEFS,
    activeCodeInspector: ui.activeCodeInspector
  });
  const [visitedTabs, setVisitedTabs] = useState(() => new Set([ui.activeTab]));
  const { currentGraphId, readiness, compareSelection, formatTime } = pageData;
  const capabilityView = useMemo(
    () =>
      projectCapabilityView({
        capabilities,
        capabilityStatus,
        capabilitySource,
        capabilityMessage
      }),
    [capabilities, capabilityMessage, capabilitySource, capabilityStatus]
  );
  const workspaceTabs = useMemo(
    () =>
      WORKSPACE_TAB_DEFS.map((tab) => ({
        ...tab,
        capability: capabilityView.workspace.surfaces[tab.id]
      })).filter((tab) => tab.capability?.visible),
    [capabilityView.workspace.surfaces]
  );
  const openBacktestsAction = capabilityView.uiActions.actions.open_backtests;
  const isWorkspaceSurfaceVisible = (surfaceKey) =>
    capabilityView.workspace.surfaces[surfaceKey]?.visible === true;
  const shouldMountTab = (surfaceKey) =>
    isWorkspaceSurfaceVisible(surfaceKey) &&
    (ui.activeTab === surfaceKey || visitedTabs.has(surfaceKey));
  const tabPanelProps = (surfaceKey) => ({
    className: "workspace-tab-panel",
    style: { display: ui.activeTab === surfaceKey ? "block" : "none" },
    "aria-hidden": ui.activeTab !== surfaceKey
  });

  useEffect(() => {
    setVisitedTabs((previous) => {
      if (previous.has(ui.activeTab)) return previous;
      const next = new Set(previous);
      next.add(ui.activeTab);
      return next;
    });
  }, [ui.activeTab]);

  useEffect(() => {
    if (ui.status !== "ready") return;
    if (isWorkspaceSurfaceVisible(ui.activeTab)) return;
    const nextTab = workspaceTabs.find((tab) => tab.capability?.visible);
    if (nextTab) {
      ui.setActiveTab(nextTab.id);
    }
  }, [capabilityView.workspace.surfaces, ui.activeTab, ui.status, ui.setActiveTab, workspaceTabs]);

  if (ui.status === "loading") {
    return (
      <div className="strategy-workspace-loading" role="status" aria-live="polite">
        <div className="strategy-workspace-loading__title">{t("正在加载策略工作区")}</div>
        <div className="strategy-workspace-loading__detail">
          {t("正在解析请求的策略图，并准备工作区界面。")}
        </div>
      </div>
    );
  }

  if (ui.status === "error") {
    return (
      <div className="strategy-workspace-loading strategy-workspace-loading--error">
        <div className="strategy-workspace-loading__title">{t("策略工作区不可用")}</div>
        <div className="strategy-workspace-loading__detail">{ui.error}</div>
        <button className="ad-btn ad-btn--ghost" onClick={() => navigateTo(strategiesPath())}>
          {t("返回策略中心")}
        </button>
        <button className="ad-btn ad-btn--ghost" onClick={() => loadGraphById(strategyId)}>
          {t("重试")}
        </button>
      </div>
    );
  }

  return (
    <main className="strategy-workspace-page">
      <h1 style={{position:"absolute",width:"1px",height:"1px",overflow:"hidden",clip:"rect(0,0,0,0)",whiteSpace:"nowrap"}}>{t("策略工作区")}</h1>
      <div className="workspace-small-screen-warning">{t("策略图编辑需要较大的屏幕空间，建议使用 ≥1180px 宽度的窗口。")}</div>
      {/* Adobe 风格紧凑页头 */}
      <header className="ad-workspace-header">
        <StrategyRouteBar
          items={[
            { label: t("策略中心"), onClick: () => navigateTo(strategiesPath()) },
            { label: graph.metadata?.name || currentGraphId },
            { label: t("工作区"), current: true }
          ]}
        />
        <div className="ad-workspace-header__info">
          <span className="ad-workspace-header__name">{graph.metadata?.name || currentGraphId}</span>
          <span className="ad-workspace-header__id">{currentGraphId}</span>
          <span className={`ad-pill ad-pill--${readiness.tone}`}>{readiness.label}</span>
          <span className="ad-pill ad-pill--muted">{compileSummary.protocol_name || t("协议待生成")}</span>
        </div>
      </header>

      <div className="workspace-toolbar-shell workspace-toolbar-shell--persistent">
        <TopToolbar variant="workspace" />
      </div>

      {/* Adobe 风格标签栏 */}
      <section className="ad-tabbar ad-tabbar--workspace-modes" aria-label="工作区模式">
        {workspaceTabs.map((tab) => (
          <button
            key={tab.id}
            data-testid={`workspace-tab-${tab.id}`}
            className={`ad-tab ad-tab--workspace-mode${ui.activeTab === tab.id ? " ad-tab--active" : ""}${tab.capability?.enabled ? "" : " ad-tab--disabled"}${diagnosticFocusRequested && tab.id === "code" && ui.activeTab !== "code" ? " ad-tab--focus-hint" : ""}`}
            disabled={!tab.capability?.enabled}
            aria-disabled={!tab.capability?.enabled}
            onClick={() => {
              if (!tab.capability?.enabled) return;
              ui.setActiveTab(tab.id);
              if (tab.id === "code" && diagnosticFocusRequested) clearDiagnosticFocus();
            }}
            title={tab.capability?.reason || t(tab.labelKey)}
          >
            <span className="ad-tab__kicker">{t(tab.kickerKey)}</span>
            <strong className="ad-tab__label">{t(tab.labelKey)}</strong>
          </button>
        ))}
        <div className="ad-tabbar__spacer" />
        <button
          className="ad-tabbar__action"
          onClick={() => navigateTo(strategyBacktestsPath(strategyId))}
          disabled={!openBacktestsAction?.enabled}
          title={openBacktestsAction?.blockReason || openBacktestsAction?.reason || undefined}
        >
          {t("打开回测")}
        </button>
      </section>

      {/* 标签页内容 */}
      {shouldMountTab("dashboard") ? (
        <section {...tabPanelProps("dashboard")}>
          <Suspense fallback={<div className="tab-skeleton">{t("加载中...")}</div>}>
            <StrategyWorkspaceDashboard
              graph={graph}
              runtime={runtime}
              compileSummary={compileSummary}
              readiness={readiness}
              onNavigate={(tabId) => ui.setActiveTab(tabId)}
              workspaceSurfaces={capabilityView.workspace.surfaces}
            />
          </Suspense>
        </section>
      ) : null}

      {shouldMountTab("overview") ? (
        <section {...tabPanelProps("overview")}>
          <Suspense fallback={<div className="tab-skeleton">{t("加载中...")}</div>}>
            <StrategyWorkspaceOverviewTab
              strategyId={strategyId} graph={graph} ui={ui}
              compileSummary={compileSummary} compileCounts={pageData.compileCounts}
              readiness={pageData.readiness} recentRuns={pageData.recentRuns}
              recentBacktests={pageData.recentBacktests} compareSelection={pageData.compareSelection}
              issueQueue={issueQueue} canvasRecommendationState={pageData.canvasRecommendationState}
              overviewMetrics={pageData.overviewMetrics} runPreviewItems={pageData.runPreviewItems}
              overviewStatusHighlights={pageData.overviewStatusHighlights}
              backtestPreviewItems={pageData.backtestPreviewItems}
              lastRun={pageData.lastRun} lastBacktest={pageData.lastBacktest}
              formatTime={pageData.formatTime}
            />
          </Suspense>
        </section>
      ) : null}

      {shouldMountTab("code") ? (
        <section {...tabPanelProps("code")}>
          <Suspense fallback={<div className="tab-skeleton">{t("加载中...")}</div>}>
            <StrategyWorkspaceCodeTab
              graph={graph} ui={ui}
              codeInspectorPanels={CODE_INSPECTOR_DEFS}
              canvasRecommendationState={pageData.canvasRecommendationState}
              configureRepairPathState={pageData.configureRepairPathState}
              activeInspectorDefinition={pageData.activeInspectorDefinition}
              secondaryInspectorDefinitions={pageData.secondaryInspectorDefinitions}
            />
          </Suspense>
        </section>
      ) : null}

      {shouldMountTab("diagnostics") ? (
        <section {...tabPanelProps("diagnostics")}>
          <Suspense fallback={<div className="tab-skeleton">{t("加载中...")}</div>}>
            <StrategyWorkspaceDiagnosticsTab
              graph={graph} runtime={runtime} selectedNodeId={selectedNodeId} ui={ui}
              compileSummary={compileSummary} compileCounts={pageData.compileCounts}
              readiness={pageData.readiness} issueQueue={issueQueue}
              issueQueueCounts={pageData.issueQueueCounts}
              issueQueueSources={pageData.issueQueueSources}
              issueQueueSourceCounts={pageData.issueQueueSourceCounts}
              diagnosticsStatusHighlights={pageData.diagnosticsStatusHighlights}
              canvasRecommendationState={pageData.canvasRecommendationState}
            />
          </Suspense>
        </section>
      ) : null}

      {shouldMountTab("research") ? (
        <section {...tabPanelProps("research")}>
          <Suspense fallback={<div className="tab-skeleton">{t("加载中...")}</div>}>
            <StrategyWorkspaceResearchTab strategyId={strategyId} />
          </Suspense>
        </section>
      ) : null}
      {shouldMountTab("monitor") ? (
        <section {...tabPanelProps("monitor")}>
          <Suspense fallback={<div className="tab-skeleton">{t("加载中...")}</div>}>
            <StrategyWorkspaceMonitorTab
              graph={graph}
              runtime={runtime}
              recentRuns={pageData.recentRuns}
              issueQueue={issueQueue}
              formatTime={pageData.formatTime}
            />
          </Suspense>
        </section>
      ) : null}
      {shouldMountTab("debug") ? (
        <section {...tabPanelProps("debug")}>
          <Suspense fallback={<div className="tab-skeleton">{t("加载中...")}</div>}>
            <StrategyWorkspaceDebugTab debugBars={pageData.debugBars || []} />
          </Suspense>
        </section>
      ) : null}
      {shouldMountTab("source") ? (
        <section {...tabPanelProps("source")}>
          <Suspense fallback={<div className="tab-skeleton">{t("加载中...")}</div>}>
            <StrategyWorkspaceSourceTab graphId={graph?.metadata?.graph_id} />
          </Suspense>
        </section>
      ) : null}
    </main>
  );
}
