import TopToolbar from "../components/TopToolbar";
import {
  backtestComparePath,
  backtestDetailPath,
  navigateTo,
  strategyBacktestsPath
} from "../router";
import {
  OverviewList,
  WorkspaceActionCard,
  WorkspaceMetricCard,
  WorkspaceSection
} from "./StrategyWorkspacePageSections";
import { WorkspaceIssueQueueCard } from "./StrategyWorkspaceIssueQueueCard";
import StrategyWorkspaceCollaborationCard from "./StrategyWorkspaceCollaborationCard";
import StrategyWorkspaceExperimentCard from "./StrategyWorkspaceExperimentCard";
import StrategyWorkspaceVersionHistoryCard from "./StrategyWorkspaceVersionHistoryCard";

export default function StrategyWorkspaceOverviewTab({
  strategyId,
  graph,
  ui,
  compileSummary,
  compileCounts,
  readiness,
  recentRuns,
  recentBacktests,
  compareSelection,
  issueQueue,
  canvasRecommendationState,
  overviewMetrics,
  runPreviewItems,
  overviewStatusHighlights,
  backtestPreviewItems,
  lastRun,
  lastBacktest,
  formatTime
}) {
  const overviewActionCards = [
    {
      kicker: "Code",
      title: "Open the builder workspace",
      note: "Move into graph editing only when you need structural changes or source-facing work.",
      meta: `${graph.nodes.length} nodes / ${graph.edges.length} edges`,
      tone: "muted",
      cta: "Open code mode",
      onClick: () => ui.setActiveTab("code")
    },
    {
      kicker: "Diagnostics",
      title: "Review compile and validation blockers",
      note: "Start from the repair queue before diving into the full diagnostics surface.",
      meta: `${compileCounts.error} errors / ${compileCounts.warning} warnings`,
      tone: compileCounts.error > 0 ? "danger" : compileCounts.warning > 0 ? "warning" : "info",
      cta: "Open diagnostics",
      onClick: () => ui.setActiveTab("diagnostics")
    },
    {
      kicker: "Research",
      title: "Open run and backtest history",
      note: "Jump into the backtest index and compare flow from the workspace shell.",
      meta: `${recentRuns.length} runs / ${recentBacktests.length} backtests`,
      tone: recentBacktests.length > 0 || recentRuns.length > 0 ? "info" : "muted",
      cta: "Open backtests",
      onClick: () => navigateTo(strategyBacktestsPath(strategyId))
    }
  ];

  return (
    <div className="strategy-workspace-overview" data-testid="strategy-workspace-overview-tab">
      <div className="strategy-workspace-overview__main">
        <section className="workspace-overview-hero">
          <div className="workspace-overview-hero__main">
            <div className="workspace-overview-hero__eyebrow">Workspace cockpit</div>
            <h2>Drive the strategy from readiness, issue flow, and recent research context.</h2>
            <p>
              The overview stays focused on the current state and the next likely action. Move into
              code, diagnostics, or research only when the present task needs the deeper surface.
            </p>
            <div className="workspace-overview-hero__status">
              <span className={`status-pill ${readiness.tone}`}>{readiness.label}</span>
              <span className="status-pill info">
                {compileSummary.protocol_name || "Protocol pending"}
              </span>
              <span className="status-pill muted">Compare queue {compareSelection.length}/2</span>
            </div>
          </div>
          <div className="workspace-overview-hero__metrics">
            {overviewStatusHighlights.map((item) => (
              <div key={item.label} className="workspace-overview-hero__metric">
                <span>{item.label}</span>
                <strong>{item.value}</strong>
                <small>{item.note}</small>
              </div>
            ))}
          </div>
        </section>

        <section className="workspace-overview-actions">
          {overviewActionCards.map((item) => (
            <WorkspaceActionCard key={item.kicker} {...item} />
          ))}
        </section>

        <WorkspaceSection
          title="Primary controls"
          subtitle="The full toolbar remains available, but it now sits under the workspace overview instead of dominating the route shell."
          testId="workspace-primary-controls-section"
        >
          <div className="workspace-toolbar-shell">
            <TopToolbar variant="workspace" />
          </div>
        </WorkspaceSection>

        <WorkspaceSection
          title="Readiness and blockers"
          subtitle="Start from the current compile and validation queue, then open the full diagnostics workflow only if needed."
          testId="workspace-readiness-section"
          actions={
            <button className="ghost-btn compact-btn" onClick={() => ui.setActiveTab("diagnostics")}>
              Open diagnostics
            </button>
          }
        >
          <WorkspaceIssueQueueCard
            title="Repair queue"
            subtitle="Focus on the most urgent and actionable items before opening the full diagnostics surface."
            items={issueQueue}
            emptyText="There is no active blocking item in the current graph."
            actionLabel="Open diagnostics"
            onAction={() => ui.setActiveTab("diagnostics")}
            onSelectItem={ui.handleSelectIssueQueueItem}
            filters={ui.issueQueueFilters}
            onFiltersChange={ui.handleIssueQueueFiltersChange}
            graph={graph}
            repairPathState={canvasRecommendationState}
          />
        </WorkspaceSection>

        <WorkspaceSection
          title="Research snapshot"
          subtitle="Scan the latest runs and backtests here before moving into the full research surface."
          testId="workspace-research-section"
          actions={
            <button
              className="ghost-btn compact-btn"
              onClick={() => navigateTo(strategyBacktestsPath(strategyId))}
            >
              Open backtest index
            </button>
          }
        >
          <div className="workspace-overview-grid workspace-overview-grid--dual">
            <OverviewList
              title="Recent runs"
              items={runPreviewItems}
              emptyText="This strategy has no recent simulated run."
            />
            <OverviewList
              title="Recent backtests"
              items={backtestPreviewItems}
              emptyText="This strategy has no recent backtest."
              renderActions={(item) => (
                <button
                  className="ghost-btn compact-btn"
                  onClick={() => navigateTo(backtestDetailPath(item.backtest_id, strategyId))}
                >
                  Detail
                </button>
              )}
            />
          </div>
        </WorkspaceSection>
      </div>

      <aside className="strategy-workspace-overview__side">
        <div className="workspace-metric-grid">
          {overviewMetrics.map((item) => (
            <WorkspaceMetricCard key={item.label} {...item} />
          ))}
        </div>

        <WorkspaceSection
          title="Current context"
          subtitle="Keep the most repeated strategy context visible while switching between workspace modes."
        >
          <div className="strategy-inspector-metrics">
            <div className="kv-line">
              <span>Latest compile ID</span>
              <strong>{graph.metadata?.runtime_binding?.last_compile_id || "-"}</strong>
            </div>
            <div className="kv-line">
              <span>Config hash</span>
              <strong>{compileSummary.config_hash || "-"}</strong>
            </div>
            <div className="kv-line">
              <span>Strategy IR role</span>
              <strong>{compileSummary.artifact_resolution?.strategy_ir_role_label || "-"}</strong>
            </div>
            <div className="kv-line">
              <span>Runtime source</span>
              <strong>{compileSummary.artifact_resolution?.runtime_source_label || "-"}</strong>
            </div>
            <div className="kv-line">
              <span>Runnable truth</span>
              <strong>{compileSummary.artifact_resolution?.source_of_truth_label || "-"}</strong>
            </div>
            <div className="kv-line">
              <span>Latest run</span>
              <strong>{lastRun ? formatTime(lastRun.created_at_ms) : "-"}</strong>
            </div>
            <div className="kv-line">
              <span>Latest backtest</span>
              <strong>{lastBacktest ? formatTime(lastBacktest.created_at_ms) : "-"}</strong>
            </div>
          </div>
        </WorkspaceSection>

        <WorkspaceSection
          title="Owner, editors, and audit"
          subtitle="Keep permission context and recent graph actions visible while staying inside the working draft."
          testId="workspace-collaboration-section"
        >
          <StrategyWorkspaceCollaborationCard
            graphId={graph.metadata?.graph_id || strategyId || "draft_graph"}
            collaboration={graph.metadata?.collaboration}
            lastRun={lastRun}
            lastBacktest={lastBacktest}
          />
        </WorkspaceSection>

        <WorkspaceSection
          title="Experiment sweep"
          subtitle="Run a narrow execution-assumptions sweep and compare variant outcomes across persisted backtests."
          testId="workspace-experiment-section"
        >
          <StrategyWorkspaceExperimentCard strategyId={strategyId} currentGraph={graph} />
        </WorkspaceSection>

        <WorkspaceSection
          title="Persisted versions"
          subtitle="Preview persisted versions without overwriting the current working draft."
          testId="workspace-persisted-versions-section"
        >
          <StrategyWorkspaceVersionHistoryCard
            graphId={graph.metadata?.graph_id || strategyId || "draft_graph"}
            currentGraph={graph}
          />
        </WorkspaceSection>

        <WorkspaceSection
          title="Next step"
          subtitle="Surface the next likely transition directly instead of forcing the user to hunt for it."
        >
          <div className="strategy-inspector-actions">
            <button className="primary-btn" onClick={() => ui.setActiveTab("code")}>
              Open code mode
            </button>
            <button className="ghost-btn" onClick={() => ui.setActiveTab("diagnostics")}>
              Review diagnostics
            </button>
            <button className="ghost-btn" onClick={() => ui.setActiveTab("research")}>
              Open research
            </button>
          </div>
        </WorkspaceSection>

        <WorkspaceSection
          title="Compare queue"
          subtitle="Keep the compare flow reachable inside the workspace without forcing the full history view open."
        >
          <div className="strategy-compare-queue">
            <div className="strategy-compare-queue__chips">
              {compareSelection.length === 0 ? (
                <span className="status-pill muted">No backtest selected</span>
              ) : (
                compareSelection.map((backtestId) => (
                  <span key={backtestId} className="status-pill info">
                    {backtestId}
                  </span>
                ))
              )}
            </div>
            <div className="strategy-inspector-actions">
              <button
                className="primary-btn"
                disabled={compareSelection.length !== 2}
                onClick={() => navigateTo(backtestComparePath(compareSelection, strategyId))}
              >
                Open compare
              </button>
              <button
                className="ghost-btn"
                onClick={() => navigateTo(strategyBacktestsPath(strategyId))}
              >
                Open backtest index
              </button>
            </div>
          </div>
        </WorkspaceSection>
      </aside>
    </div>
  );
}
