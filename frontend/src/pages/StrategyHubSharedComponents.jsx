export function StrategyMetricCard({ label, value, note }) {
  return (
    <div className="strategy-kpi-card">
      <span>{label}</span>
      <strong>{value}</strong>
      <small>{note}</small>
    </div>
  );
}

export function StrategyOpsCard({ title, value, note, tone = "muted" }) {
  return (
    <div className={`strategy-ops-card strategy-ops-card--${tone}`}>
      <div className="strategy-ops-card__title">{title}</div>
      <strong>{value}</strong>
      <small>{note}</small>
    </div>
  );
}

export function StrategyTaskGroup({ label, tone = "muted", children, className = "" }) {
  return (
    <div className={`strategy-task-group strategy-task-group--${tone} ${className}`.trim()}>
      <span className="strategy-task-group__label">{label}</span>
      <div className="strategy-task-group__actions">{children}</div>
    </div>
  );
}

export function ActivityListCard({
  title,
  subtitle,
  items,
  emptyText,
  renderMeta,
  testId
}) {
  return (
    <section className="strategy-activity-card" data-testid={testId}>
      <div className="strategy-card-header">
        <div>
          <div className="panel-title">{title}</div>
          <div className="strategy-card-subtitle">{subtitle}</div>
        </div>
      </div>

      <div className="strategy-activity-list">
        {items.length === 0 ? <div className="strategy-directory-empty">{emptyText}</div> : null}
        {items.map((item) => (
          <div key={`${item.kind}-${item.id}`} className="strategy-activity-item">
            <div className="strategy-activity-item__copy">
              <div className="strategy-activity-item__title">
                <strong>{item.title}</strong>
                <span className={`status-pill ${item.kind === "backtest" ? "info" : "muted"}`}>
                  {item.kind === "backtest" ? "回测" : "模拟"}
                </span>
              </div>
              <div className="strategy-activity-item__meta">
                <span>{item.graphId}</span>
                <span>{item.createdAtLabel}</span>
                <span>{item.note}</span>
              </div>
              <small>{item.detail}</small>
            </div>
            {renderMeta ? (
              <div className="strategy-activity-item__actions">{renderMeta(item)}</div>
            ) : null}
          </div>
        ))}
      </div>
    </section>
  );
}
