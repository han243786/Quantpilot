import { Suspense, lazy } from "react";
import { StrategyCardNote } from "./StrategyHubSharedComponents";
import { WorkspacePanelFallback } from "./StrategyWorkspacePanelFallbacks";
import { useI18n } from "../i18n";

const TASK_LANES_NOTE =
  "一次只保持一个主通道活跃，必要时再展开辅助通道。";

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
  const { t } = useI18n();

  return (
    <Suspense fallback={<WorkspacePanelFallback title="正在加载工作区构建面板" />}>
      <div className="strategy-workspace-code" data-testid="strategy-workspace-code-tab">
        <div className="workspace-mode-strip">
          <strong>{t("构建工作区")}</strong>
          <div className="workspace-mode-strip__pills">
            <span className="status-pill muted">{t("模块库")}</span>
            <span className="status-pill info">{t("策略图")}</span>
            <span className="status-pill muted">{t("检查器")}</span>
          </div>
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
                  <div className="panel-title strategy-card-title-note">
                    <StrategyCardNote label="任务通道" note={TASK_LANES_NOTE} />
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
                      恢复自动跟随
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
                        ? `画布焦点已切换到 ${ui.codeLaneNotice.focusLabel}。`
                        : `画布焦点保持在 ${ui.codeLaneNotice.focusLabel}。`}
                    </span>
                  ) : null}
                </div>
              ) : null}

              <div className="workspace-inspector-nav" aria-label="构建模式任务">
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
                        {isExpanded ? "隐藏" : "显示"} {panel.label}通道
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
