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

const WORKSPACE_TABS = [
  {
    id: "overview",
    label: "Overview",
    note: "Build health, issue queue, and recent research activity."
  },
  {
    id: "code",
    label: "Code",
    note: "Graph editing, node wiring, and source-focused repair work."
  },
  {
    id: "diagnostics",
    label: "Diagnostics",
    note: "Compile health, blocking issues, and repair routing."
  },
  {
    id: "research",
    label: "Research",
    note: "Runs, backtests, and the live event stream."
  }
];

const CODE_INSPECTOR_PANELS = [
  {
    id: "params",
    label: "Config",
    note: "Node configuration, graph identity, and structure controls."
  },
  {
    id: "diagnostics",
    label: "Checks",
    note: "Compile output, blockers, and repair routing."
  },
  {
    id: "code",
    label: "Source",
    note: "Graph source, Strategy IR, and code-facing tools."
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
        <div className="strategy-workspace-loading__title">Loading strategy workspace</div>
        <div className="strategy-workspace-loading__detail">
          Resolving the requested strategy graph and preparing the workspace shell.
        </div>
      </div>
    );
  }

  if (ui.status === "error") {
    return (
      <div className="strategy-workspace-loading strategy-workspace-loading--error">
        <div className="strategy-workspace-loading__title">Strategy workspace unavailable</div>
        <div className="strategy-workspace-loading__detail">{ui.error}</div>
        <button className="ghost-btn" onClick={() => navigateTo(strategiesPath())}>
          Back to strategy hub
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
              { label: "Strategies", onClick: () => navigateTo(strategiesPath()) },
              { label: graph.metadata?.name || currentGraphId },
              { label: "Workspace", current: true }
            ]}
          />
          <div className="strategy-hub-kicker">Strategy workspace</div>
          <h1>{graph.metadata?.name || currentGraphId}</h1>
          <div className="strategy-workspace-header__meta">
            <span>{currentGraphId}</span>
            <span>Updated {formatTime(graph.metadata?.updated_at)}</span>
            <span>{runtimeStatusLabel(runtime.status)}</span>
          </div>
        </div>

        <div className="strategy-workspace-header__stats">
          <button
            className="ghost-btn compact-btn"
            onClick={() => navigateTo(strategyBacktestsPath(strategyId))}
          >
            Open backtests
          </button>
          <div className={`status-pill ${readiness.tone}`}>{readiness.label}</div>
          <div className="status-pill info">{compileSummary.protocol_name || "Protocol pending"}</div>
          <div className="status-pill muted">Compare queue {compareSelection.length}/2</div>
        </div>
      </header>

      <section className="strategy-workspace-tabbar" aria-label="Workspace modes">
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
    </div>
  );
}
