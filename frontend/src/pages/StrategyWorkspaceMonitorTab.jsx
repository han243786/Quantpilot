import { useI18n } from "../i18n";
import { buildWorkspaceMonitorModel } from "./strategyWorkspaceAuxiliaryTabsShell";

function MonitorMetric({ label, value, tone = "muted" }) {
  return (
    <div className={`workspace-monitor-metric workspace-monitor-metric--${tone}`}>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function MonitorCard({ title, testId, children, action }) {
  return (
    <section className="workspace-monitor-card" data-testid={testId}>
      <div className="workspace-monitor-card__header">
        <strong>{title}</strong>
        {action || null}
      </div>
      <div className="workspace-monitor-card__body">{children}</div>
    </section>
  );
}

export default function StrategyWorkspaceMonitorTab({
  graph = { nodes: [] },
  runtime = {},
  recentRuns = [],
  issueQueue = [],
  formatTime = (value) => value ?? "-"
}) {
  const { t } = useI18n();
  const monitorModel = buildWorkspaceMonitorModel({
    graph,
    runtime,
    recentRuns,
    issueQueue,
    formatTime,
    t
  });

  return (
    <div className="strategy-workspace-monitor" data-testid="strategy-workspace-monitor-tab">
      <div className="workspace-mode-strip">
        <strong>{t("运行监控工作区")}</strong>
        <div className="workspace-mode-strip__pills">
          {monitorModel.stripPills.map((pill) => (
            <span key={`${pill.tone}-${pill.label}`} className={`status-pill ${pill.tone}`}>
              {pill.label}
            </span>
          ))}
        </div>
      </div>

      <div className="workspace-monitor-grid">
        <MonitorCard title={t("运行会话")} testId="workspace-monitor-runtime-card">
          {monitorModel.runtimeMetrics.map((metric) => (
            <MonitorMetric key={metric.label} {...metric} />
          ))}
        </MonitorCard>

        <MonitorCard title={t("账户")} testId="workspace-monitor-account-card">
          {monitorModel.accountMetrics.map((metric) => (
            <MonitorMetric key={metric.label} {...metric} />
          ))}
        </MonitorCard>

        <MonitorCard title={t("风险与执行")} testId="workspace-monitor-risk-card">
          {monitorModel.riskMetrics.map((metric) => (
            <MonitorMetric key={metric.label} {...metric} />
          ))}
        </MonitorCard>

        <MonitorCard title={t("最近事件")} testId="workspace-monitor-events-card">
          {monitorModel.recentEvents.length > 0 ? (
            <div className="workspace-monitor-event-list">
              {monitorModel.recentEvents.map((event, index) => (
                <div key={event.event_id || event.id || index} className="workspace-monitor-event">
                  <span>{event.stage || event.type || t("事件")}</span>
                  <strong>{event.summary || event.message || event.event_id || event.id || "-"}</strong>
                </div>
              ))}
            </div>
          ) : (
            <div className="workspace-monitor-empty">{t("暂无事件")}</div>
          )}
        </MonitorCard>
      </div>
    </div>
  );
}
