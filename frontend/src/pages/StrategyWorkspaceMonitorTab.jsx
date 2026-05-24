import { getRuntimeStatusMeta } from "../utils/runtimeStatus";
import { useI18n } from "../i18n";

function formatNumber(value, digits = 2) {
  if (!Number.isFinite(Number(value))) return "-";
  return Number(value).toFixed(digits);
}

function formatCount(value) {
  if (!Number.isFinite(Number(value))) return "0";
  return new Intl.NumberFormat().format(Number(value));
}

function runtimeKindLabel(kind, t) {
  if (kind === "backtest") return t("回测");
  if (kind === "simulation") return t("模拟");
  if (kind === "live") return t("实盘");
  return t("未运行");
}

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

function latestEvents(runtime) {
  const timeline = Array.isArray(runtime.timeline) ? runtime.timeline : [];
  const events = Array.isArray(runtime.events) ? runtime.events : [];
  return [...(timeline.length > 0 ? timeline : events)];
}

export default function StrategyWorkspaceMonitorTab({
  graph = { nodes: [] },
  runtime = {},
  recentRuns = [],
  issueQueue = [],
  formatTime = (value) => value ?? "-"
}) {
  const { t } = useI18n();
  const statusMeta = getRuntimeStatusMeta(runtime.status);
  const account = runtime.account || {};
  const openOrders = Array.isArray(account.open_orders) ? account.open_orders : [];
  const allEvents = latestEvents(runtime);
  const recentEvents = allEvents.slice(-5).reverse();
  const riskIssueCount = issueQueue.filter((item) => item.nodeType === "risk").length;
  const executionNodes = (graph.nodes || []).filter((node) => node.type === "execution").length;
  const latestRun = recentRuns[0] || null;
  const runKind = runtimeKindLabel(runtime.runKind, t);

  return (
    <div className="strategy-workspace-monitor" data-testid="strategy-workspace-monitor-tab">
      <div className="workspace-mode-strip">
        <strong>{t("运行监控工作区")}</strong>
        <div className="workspace-mode-strip__pills">
          <span className={`status-pill ${statusMeta.tone}`}>{statusMeta.label}</span>
          <span className="status-pill muted">{runKind}</span>
          <span className="status-pill info">{formatCount(openOrders.length)} {t("挂单")}</span>
        </div>
      </div>

      <div className="workspace-monitor-grid">
        <MonitorCard title={t("运行会话")} testId="workspace-monitor-runtime-card">
          <MonitorMetric label={t("状态")} value={statusMeta.label} tone={statusMeta.tone} />
          <MonitorMetric label={t("运行 ID")} value={runtime.runId || "-"} />
          <MonitorMetric label={t("类型")} value={runKind} />
          <MonitorMetric label={t("最近运行")} value={latestRun ? formatTime(latestRun.created_at_ms) : "-"} />
        </MonitorCard>

        <MonitorCard title={t("账户")} testId="workspace-monitor-account-card">
          <MonitorMetric label={t("净值估算")} value={formatNumber(account.equity_estimate)} tone="success" />
          <MonitorMetric label={t("可用现金")} value={formatNumber(account.available_cash_balance)} />
          <MonitorMetric label={t("冻结现金")} value={formatNumber(account.frozen_cash_balance)} />
          <MonitorMetric label={t("挂单")} value={formatCount(account.open_order_count ?? openOrders.length)} />
        </MonitorCard>

        <MonitorCard title={t("风险与执行")} testId="workspace-monitor-risk-card">
          <MonitorMetric label={t("风险阻塞")} value={formatCount(riskIssueCount)} tone={riskIssueCount > 0 ? "danger" : "success"} />
          <MonitorMetric label={t("执行节点")} value={formatCount(executionNodes)} />
          <MonitorMetric label={t("诊断")} value={runtime.diagnostics ? t("已连接") : "-"} />
          <MonitorMetric label={t("事件数")} value={formatCount(allEvents.length)} />
        </MonitorCard>

        <MonitorCard title={t("最近事件")} testId="workspace-monitor-events-card">
          {recentEvents.length > 0 ? (
            <div className="workspace-monitor-event-list">
              {recentEvents.map((event, index) => (
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
