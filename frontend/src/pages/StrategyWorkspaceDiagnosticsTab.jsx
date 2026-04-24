import { Suspense, lazy } from "react";
import {
  WorkspaceActionCard,
  WorkspaceMetricCard,
  WorkspaceSection
} from "./StrategyWorkspacePageSections";
import { WorkspaceIssueQueueCard } from "./StrategyWorkspaceIssueQueueCard";
import { diagnosticQueueSource } from "../utils/strategyWorkspaceIssueQueue";
import { WorkspacePanelFallback } from "./StrategyWorkspacePanelFallbacks";
import RuntimeDiagnosticsPanel from "../components/RuntimeDiagnosticsPanel";

const DiagnosticsPanel = lazy(() => import("../components/DiagnosticsPanel"));

function compileOutputsText(outputs) {
  if (!outputs) return "-";
  return [
    `${outputs.data_sources || 0} data`,
    `${outputs.intent_generators || 0} intent`,
    `${outputs.agents || 0} agent`,
    `${outputs.risk_controls || 0} risk`,
    `${outputs.executions || 0} execution`
  ].join(" / ");
}

export default function StrategyWorkspaceDiagnosticsTab({
  graph,
  runtime,
  selectedNodeId,
  ui,
  compileSummary,
  compileCounts,
  readiness,
  issueQueue,
  issueQueueCounts,
  issueQueueSources,
  issueQueueSourceCounts,
  diagnosticsStatusHighlights,
  canvasRecommendationState
}) {
  const diagnosticsSourceCards = issueQueueSources.map((source) => ({
    source,
    kicker: diagnosticQueueSource({ source }),
    title:
      source === "validation"
        ? "Validation blockers"
        : source === "runtime"
          ? "Runtime blockers"
          : source === "strategy_ir"
            ? "Strategy IR diagnostics"
            : "Formal QuantScript diagnostics",
    note:
      source === "validation"
        ? "Issues coming from graph validation and structural checks."
        : source === "runtime"
          ? "Compile or execution blockers found on the runtime path."
          : source === "strategy_ir"
            ? "Problems related to Strategy IR fields or generation."
            : "Problems detected inside the formal authoring and compile pipeline.",
    meta: `${issueQueueSourceCounts[source] || 0} issue(s)`,
    tone:
      source === "runtime"
        ? "danger"
        : source === "validation"
          ? "warning"
          : "info",
    cta: "Filter queue",
    onClick: () =>
      ui.handleIssueQueueFiltersChange({
        showSourceFilters: true,
        sourceFilter: source,
        nodeTypeFilter: "all"
      })
  }));

  return (
    <div className="strategy-workspace-diagnostics" data-testid="strategy-workspace-diagnostics-tab">
      <div className="strategy-workspace-diagnostics__main">
        <section className="workspace-diagnostics-hero">
          <div className="workspace-diagnostics-hero__main">
            <div className="workspace-diagnostics-hero__eyebrow">Diagnostics cockpit</div>
            <h2>Route blockers, inspect compile output, and move directly into the repair flow.</h2>
            <p>
              Keep diagnostics centered on the current repair path. Narrow the scope from the queue
              first, then open the full structured diagnostics only when the selected issue needs it.
            </p>
            <div className="workspace-diagnostics-hero__status">
              <span className={`status-pill ${readiness.tone}`}>{readiness.label}</span>
              <span className="status-pill warning">
                {`${issueQueueCounts.error} blockers / ${issueQueueCounts.warning} warnings`}
              </span>
              <span className="status-pill muted">{ui.diagnosticsQueueScope}</span>
            </div>
          </div>
          <div className="workspace-diagnostics-hero__metrics">
            {diagnosticsStatusHighlights.map((item) => (
              <div key={item.label} className="workspace-diagnostics-hero__metric">
                <span>{item.label}</span>
                <strong>{item.value}</strong>
                <small>{item.note}</small>
              </div>
            ))}
          </div>
        </section>

        <section className="workspace-diagnostics-actions" aria-label="Diagnostic source lanes">
          {diagnosticsSourceCards.length > 0 ? (
            diagnosticsSourceCards.map((item) => <WorkspaceActionCard key={item.source} {...item} />)
          ) : (
            <div className="workspace-section-card workspace-diagnostics-empty-state">
              <div className="muted-line">
                No issue source is active yet. Compile or validate the strategy to populate the lanes.
              </div>
            </div>
          )}
        </section>

        <WorkspaceSection
          title="Priority repair queue"
          subtitle="Start from the narrowed repair queue, then escalate into the full diagnostic surface only if needed."
          testId="workspace-priority-repair-queue-section"
        >
          <WorkspaceIssueQueueCard
            title="Priority fixes"
            subtitle="Jump straight to the node or route that blocks the current compile path."
            items={issueQueue}
            emptyText="There is no active repair item right now."
            onSelectItem={ui.handleSelectIssueQueueItem}
            filters={ui.issueQueueFilters}
            onFiltersChange={ui.handleIssueQueueFiltersChange}
            graph={graph}
            repairPathState={canvasRecommendationState}
          />
        </WorkspaceSection>

        <WorkspaceSection
          title="Structured diagnostics"
          subtitle="Keep the full compile view available after triage so the user can inspect the complete context without leaving the workspace."
          testId="workspace-structured-diagnostics-section"
          actions={
            <div className="strategy-inspector-actions">
              <button className="ghost-btn compact-btn" onClick={() => ui.setActiveTab("code")}>
                Open code mode
              </button>
              <button className="ghost-btn compact-btn" onClick={() => ui.setActiveTab("research")}>
                Open research
              </button>
            </div>
          }
        >
          <div className="workspace-metric-grid workspace-metric-grid--triple">
            <WorkspaceMetricCard
              label="Compilable"
              value={compileSummary.compilable ? "Yes" : "No"}
              note={compileSummary.backend_verified ? "Backend verified" : "Local summary only"}
              tone={compileSummary.compilable ? "success" : "danger"}
            />
            <WorkspaceMetricCard
              label="Protocol"
              value={compileSummary.protocol_name || "-"}
              note={compileSummary.config_hash || "No config hash recorded"}
              tone="info"
            />
            <WorkspaceMetricCard
              label="Compile outputs"
              value={compileOutputsText(compileSummary.outputs)}
              note="Materialized runtime pipeline shape."
              tone="muted"
            />
          </div>
          <Suspense fallback={<WorkspacePanelFallback title="Loading diagnostics panel" />}>
            <DiagnosticsPanel
              compileSummary={compileSummary}
              onRouteDiagnostic={ui.handleRouteDiagnostic}
              graph={graph}
              repairPathState={canvasRecommendationState}
            />
          </Suspense>
        </WorkspaceSection>

        <WorkspaceSection
          title="Runtime diagnostics"
          subtitle="Keep node-level runtime state, latest input/output snapshots, and recent warnings beside the compile queue."
          testId="workspace-runtime-diagnostics-section"
        >
          <RuntimeDiagnosticsPanel
            graph={graph}
            runtime={runtime}
            selectedNodeId={selectedNodeId}
            title="Runtime diagnostics"
            subtitle="Use the current node selection and runtime event log to inspect what the selected node most recently received, emitted, and complained about."
          />
        </WorkspaceSection>
      </div>

      <aside className="strategy-workspace-diagnostics__side">
        <WorkspaceSection
          title="Source lanes"
          subtitle="Keep the source-lane summary visible so filter changes do not require reopening the whole diagnostics surface."
        >
          <div className="workspace-diagnostics-source-list">
            {diagnosticsSourceCards.length === 0 ? (
              <div className="muted-line">No source lane is active.</div>
            ) : (
              diagnosticsSourceCards.map((item) => (
                <button
                  key={item.source}
                  type="button"
                  className={`workspace-diagnostics-source-item${
                    ui.issueQueueFilters.sourceFilter === item.source
                      ? " workspace-diagnostics-source-item--active"
                      : ""
                  }`}
                  onClick={item.onClick}
                >
                  <strong>{item.kicker}</strong>
                  <span>{item.meta}</span>
                </button>
              ))
            )}
          </div>
        </WorkspaceSection>

        <WorkspaceSection
          title="Compile context"
          subtitle="Keep the compile identity visible while fixing diagnostics."
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
              <span>Protocol</span>
              <strong>{compileSummary.protocol_name || "-"}</strong>
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
          </div>
        </WorkspaceSection>

        <WorkspaceSection
          title="Next transition"
          subtitle="Move quickly between structural repair and outcome review."
        >
          <div className="strategy-inspector-actions">
            <button className="primary-btn" onClick={() => ui.setActiveTab("code")}>
              Repair in code mode
            </button>
            <button className="ghost-btn" onClick={() => ui.setActiveTab("research")}>
              Open research
            </button>
          </div>
        </WorkspaceSection>
      </aside>
    </div>
  );
}
