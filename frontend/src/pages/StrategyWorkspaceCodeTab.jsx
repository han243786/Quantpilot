import { Suspense, lazy } from "react";
import { WorkspacePanelFallback } from "./StrategyWorkspacePanelFallbacks";

const ModuleSidebar = lazy(() => import("../components/ModuleSidebar"));
const StrategyCodePanel = lazy(() => import("../components/StrategyCodePanel"));
const StrategyDiagnosticsPanel = lazy(() => import("../components/StrategyDiagnosticsPanel"));
const StrategyParamsPanel = lazy(() => import("../components/StrategyParamsPanel"));
const StrategyCanvas = lazy(() => import("../components/StrategyCanvas"));

function renderCodeInspector(panelId, { graph, ui, canvasRecommendationState, configureRepairPathState }) {
  if (panelId === "diagnostics") {
    return (
      <StrategyDiagnosticsPanel
        onRouteDiagnostic={ui.handleRouteDiagnostic}
        graph={graph}
        repairPathState={canvasRecommendationState}
      />
    );
  }

  if (panelId === "code") {
    return <StrategyCodePanel onActivateSourceLane={ui.handleActivateSourceLane} />;
  }

  return <StrategyParamsPanel repairPathState={configureRepairPathState} />;
}

export default function StrategyWorkspaceCodeTab({
  graph,
  ui,
  codeInspectorPanels,
  canvasRecommendationState,
  configureRepairPathState,
  activeInspectorDefinition,
  secondaryInspectorDefinitions
}) {
  return (
    <Suspense fallback={<WorkspacePanelFallback title="Loading workspace code panel" />}>
      <div className="strategy-workspace-code" data-testid="strategy-workspace-code-tab">
        <div className="workspace-mode-banner">
          <strong>Advanced graph mode</strong>
          <span>
            Open this mode only for structural changes, wiring work, or source-facing repair tasks.
          </span>
        </div>
        <div className="strategy-workspace-builder strategy-workspace-builder--split">
          <ModuleSidebar workspaceContext={ui.canvasWorkspaceContext} />
          <StrategyCanvas
            focusMode={ui.canvasFocusMode}
            onFocusModeChange={ui.setCanvasFocusMode}
            workspaceContext={ui.canvasWorkspaceContext}
            recommendationStateOverride={canvasRecommendationState}
          />
          <div className="strategy-workspace-builder__rail">
            <section
              className="workspace-section-card workspace-inspector-stack"
              data-testid="workspace-task-lanes-section"
            >
              <div className="workspace-section-card__header workspace-section-card__header--stack">
                <div>
                  <div className="panel-title">Task lanes</div>
                  <div className="strategy-card-subtitle">
                    Keep one primary lane active at a time, then expand secondary lanes only when needed.
                  </div>
                </div>
                <div className="workspace-inspector-stack__controls">
                  <span
                    className={`status-pill ${
                      ui.codeLaneState.mode === "manual" ? "warning" : "muted"
                    }`}
                  >
                    {ui.canvasWorkspaceContext.laneStatus}
                  </span>
                  {ui.codeLaneState.mode === "manual" ? (
                    <button
                      className="ghost-btn compact-btn"
                      onClick={ui.resumeCodeLaneAutoFollow}
                    >
                      Resume auto-follow
                    </button>
                  ) : null}
                </div>
              </div>
              {ui.codeLaneNotice ? (
                <div
                  className={`workspace-inspector-stack__reason workspace-inspector-stack__reason--${ui.codeLaneNotice.tone}${
                    ui.isCodeLaneNoticeVisible ? "" : " workspace-inspector-stack__reason--faded"
                  }`}
                  role="status"
                  aria-live="polite"
                  onMouseEnter={ui.handleCodeLaneNoticeMouseEnter}
                  onMouseLeave={ui.handleCodeLaneNoticeMouseLeave}
                >
                  <strong>{ui.codeLaneNotice.title}</strong>
                  <span>{ui.codeLaneNotice.message}</span>
                  {ui.codeLaneNotice.focusLabel ? (
                    <span className="workspace-inspector-stack__reason-focus">
                      {ui.codeLaneNotice.focusChanged
                        ? `Canvas focus switched to ${ui.codeLaneNotice.focusLabel}.`
                        : `Canvas focus stayed on ${ui.codeLaneNotice.focusLabel}.`}
                    </span>
                  ) : null}
                </div>
              ) : null}

              <div className="workspace-inspector-nav" aria-label="Code mode tasks">
                {codeInspectorPanels.map((panel) => (
                  <button
                    key={panel.id}
                    className={`workspace-inspector-nav__tab${
                      activeInspectorDefinition.id === panel.id
                        ? " workspace-inspector-nav__tab--active"
                        : ""
                    }`}
                    onClick={() => ui.activateCodeInspector(panel.id, { pin: true })}
                  >
                    <strong>{panel.label}</strong>
                    <span>{panel.note}</span>
                  </button>
                ))}
              </div>

              <div className="workspace-inspector-stack__primary">
                {renderCodeInspector(activeInspectorDefinition.id, {
                  graph,
                  ui,
                  canvasRecommendationState,
                  configureRepairPathState
                })}
              </div>

              <div className="workspace-inspector-stack__secondary">
                {secondaryInspectorDefinitions.map((panel) => {
                  const isExpanded = ui.expandedCodeInspectors.includes(panel.id);
                  return (
                    <div key={panel.id} className="workspace-inspector-disclosure">
                      <button
                        className={`ghost-btn compact-btn workspace-inspector-disclosure__toggle${
                          isExpanded
                            ? " workspace-inspector-disclosure__toggle--active"
                            : ""
                        }`}
                        onClick={() => ui.toggleExpandedInspector(panel.id)}
                      >
                        {isExpanded ? "Hide" : "Show"} {panel.label} lane
                      </button>
                      {isExpanded ? (
                        <div className="workspace-inspector-disclosure__panel">
                          {renderCodeInspector(panel.id, {
                            graph,
                            ui,
                            canvasRecommendationState,
                            configureRepairPathState
                          })}
                        </div>
                      ) : null}
                    </div>
                  );
                })}
              </div>
            </section>
          </div>
        </div>
      </div>
    </Suspense>
  );
}
