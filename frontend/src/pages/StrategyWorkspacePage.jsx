import { Suspense, lazy, useMemo } from "react";
import "./strategy-workspace.css";
import {
  navigateTo,
  strategyBacktestsPath,
  strategiesPath
} from "../router";
import { runtimeStatusLabel } from "../utils/runtimeStatus";
import { buildWorkspaceIssueQueue } from "../utils/strategyWorkspaceIssueQueue";
import { useStrategyWorkspaceSharedModel } from "../hooks/useStrategyWorkspaceSharedModel";
import { useStrategyWorkspaceUiState } from "../hooks/useStrategyWorkspaceUiState";
import { useStrategyWorkspacePageData } from "../hooks/useStrategyWorkspacePageData";
import { StrategyRouteBar } from "./BacktestAnalysisLayout";

const StrategyWorkspaceOverviewTab = lazy(() => import("./StrategyWorkspaceOverviewTab"));
const StrategyWorkspaceCodeTab = lazy(() => import("./StrategyWorkspaceCodeTab"));
const StrategyWorkspaceDiagnosticsTab = lazy(() => import("./StrategyWorkspaceDiagnosticsTab"));
const StrategyWorkspaceResearchTab = lazy(() => import("./StrategyWorkspaceResearchTab"));
const StrategyWorkspaceDebugTab = lazy(() => import("./StrategyWorkspaceDebugTab"));
const StrategyWorkspaceSourceTab = lazy(() => import("./StrategyWorkspaceSourceTab"));

const WORKSPACE_TABS = [
  {
    id: "overview",
    label: "总览",
    note: "编译健康、问题队列与近期研究活动。"
  },
  {
    id: "code",
    label: "构建",
    note: "策略图编辑、节点连线与源码修复。"
  },
  {
    id: "diagnostics",
    label: "诊断",
    note: "编译健康、阻塞问题与修复路径。"
  },
  {
    id: "research",
    label: "研究",
    note: "模拟、回测与实时事件流。"
  },
  {
    id: "debug",
    label: "调试",
    note: "@debug 指令输出的 per-bar 变量值。"
  },
  {
    id: "source",
    label: "源码",
    note: "查看原始 QuantScript 源码并一键运行测试。"
  }
];

const CODE_INSPECTOR_PANELS = [
  {
    id: "params",
    label: "配置",
    note: "节点配置、策略图身份与结构控制。"
  },
  {
    id: "diagnostics",
    label: "检查",
    note: "编译输出、阻塞项与修复路径。"
  },
  {
    id: "code",
    label: "源码",
    note: "策略图源码、Strategy IR 与源码工具。"
  }
];

export default function StrategyWorkspacePage({ strategyId }) {
  const {
    graph,
    runtime,
    selectedNodeId,
    selectedEdgeId,
    selectedCompileDiagnosticTarget,
    loadGraphById
  } = useStrategyWorkspaceSharedModel();
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
    codeInspectorPanels: CODE_INSPECTOR_PANELS
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
    codeInspectorPanels: CODE_INSPECTOR_PANELS,
    activeCodeInspector: ui.activeCodeInspector
  });
  const { currentGraphId, readiness, compareSelection, formatTime } = pageData;

  if (ui.status === "loading") {
    return (
      <div className="strategy-workspace-loading" role="status" aria-live="polite">
        <div className="strategy-workspace-loading__title">正在加载策略工作区</div>
        <div className="strategy-workspace-loading__detail">
          正在解析请求的策略图，并准备工作区界面。
        </div>
      </div>
    );
  }

  if (ui.status === "error") {
    return (
      <div className="strategy-workspace-loading strategy-workspace-loading--error">
        <div className="strategy-workspace-loading__title">策略工作区不可用</div>
        <div className="strategy-workspace-loading__detail">{ui.error}</div>
        <button className="ghost-btn" onClick={() => navigateTo(strategiesPath())}>
          返回策略中心
        </button>
      </div>
    );
  }

  return (
    <div className="strategy-workspace-page">
      <header className="strategy-workspace-header">
        <div className="strategy-workspace-header__lead">
          <StrategyRouteBar
            items={[
              { label: "策略中心", onClick: () => navigateTo(strategiesPath()) },
              { label: graph.metadata?.name || currentGraphId },
              { label: "工作区", current: true }
            ]}
          />
          <div className="strategy-hub-kicker">策略工作区</div>
          <h1>{graph.metadata?.name || currentGraphId}</h1>
          <div className="strategy-workspace-header__meta">
            <span>{currentGraphId}</span>
            <span>更新于 {formatTime(graph.metadata?.updated_at)}</span>
            <span>{runtimeStatusLabel(runtime.status)}</span>
          </div>
        </div>

        <div className="strategy-workspace-header__stats">
          <button
            className="ghost-btn compact-btn"
            onClick={() => navigateTo(strategyBacktestsPath(strategyId))}
          >
            打开回测
          </button>
          <div className={`status-pill ${readiness.tone}`}>{readiness.label}</div>
          <div className="status-pill info">{compileSummary.protocol_name || "协议待生成"}</div>
          {runtime.testnet_status ? (
            <div className={`status-pill ${runtime.testnet_status === "connecting" ? "warning" : "success"}`}>
              模拟盘: {runtime.testnet_status === "connecting" ? "连接中..." : `${runtime.testnet_exchange || "OKX"} | 余额 ${runtime.testnet_balance || "0"} USDT`}
            </div>
          ) : null}
          <div className="status-pill muted">对比队列 {compareSelection.length}/2</div>
        </div>
      </header>

      <section className="strategy-workspace-tabbar" aria-label="工作区模式">
        {WORKSPACE_TABS.map((tab) => (
          <button
            key={tab.id}
            data-testid={`workspace-tab-${tab.id}`}
            className={`workspace-tab${ui.activeTab === tab.id ? " workspace-tab--active" : ""}`}
            onClick={() => ui.setActiveTab(tab.id)}
          >
            <strong>{tab.label}</strong>
            <span>{tab.note}</span>
          </button>
        ))}
      </section>

      {ui.activeTab === "overview" ? (
        <Suspense fallback={null}>
          <StrategyWorkspaceOverviewTab
            strategyId={strategyId}
            graph={graph}
            ui={ui}
            compileSummary={compileSummary}
            compileCounts={pageData.compileCounts}
            readiness={pageData.readiness}
            recentRuns={pageData.recentRuns}
            recentBacktests={pageData.recentBacktests}
            compareSelection={pageData.compareSelection}
            issueQueue={issueQueue}
            canvasRecommendationState={pageData.canvasRecommendationState}
            overviewMetrics={pageData.overviewMetrics}
            runPreviewItems={pageData.runPreviewItems}
            overviewStatusHighlights={pageData.overviewStatusHighlights}
            backtestPreviewItems={pageData.backtestPreviewItems}
            lastRun={pageData.lastRun}
            lastBacktest={pageData.lastBacktest}
            formatTime={pageData.formatTime}
          />
        </Suspense>
      ) : null}

      {ui.activeTab === "code" ? (
        <Suspense fallback={null}>
          <StrategyWorkspaceCodeTab
            graph={graph}
            ui={ui}
            codeInspectorPanels={CODE_INSPECTOR_PANELS}
            canvasRecommendationState={pageData.canvasRecommendationState}
            configureRepairPathState={pageData.configureRepairPathState}
            activeInspectorDefinition={pageData.activeInspectorDefinition}
            secondaryInspectorDefinitions={pageData.secondaryInspectorDefinitions}
          />
        </Suspense>
      ) : null}

      {ui.activeTab === "diagnostics" ? (
        <Suspense fallback={null}>
          <StrategyWorkspaceDiagnosticsTab
            graph={graph}
            runtime={runtime}
            selectedNodeId={selectedNodeId}
            ui={ui}
            compileSummary={compileSummary}
            compileCounts={pageData.compileCounts}
            readiness={pageData.readiness}
            issueQueue={issueQueue}
            issueQueueCounts={pageData.issueQueueCounts}
            issueQueueSources={pageData.issueQueueSources}
            issueQueueSourceCounts={pageData.issueQueueSourceCounts}
            diagnosticsStatusHighlights={pageData.diagnosticsStatusHighlights}
            canvasRecommendationState={pageData.canvasRecommendationState}
          />
        </Suspense>
      ) : null}

      {ui.activeTab === "research" ? (
        <Suspense fallback={null}>
          <StrategyWorkspaceResearchTab strategyId={strategyId} />
        </Suspense>
      ) : null}
      {ui.activeTab === "debug" ? (
        <Suspense fallback={null}>
          <StrategyWorkspaceDebugTab debugBars={pageData.debugBars || []} />
        </Suspense>
      ) : null}
      {ui.activeTab === "source" ? (
        <Suspense fallback={null}>
          <StrategyWorkspaceSourceTab graphId={graph?.metadata?.graph_id} />
        </Suspense>
      ) : null}
    </div>
  );
}
