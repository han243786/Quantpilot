import { StrategyCardNote } from "./StrategyHubSharedComponents";

export function WorkspaceMetricCard({ label, value, note, tone = "muted" }) {
  return (
    <div className={`workspace-metric-card workspace-metric-card-${tone}`}>
      <span>{label}</span>
      <strong>{value}</strong>
      <small>{note}</small>
    </div>
  );
}

export function WorkspaceActionCard({
  kicker,
  title,
  note,
  meta,
  tone = "muted",
  cta,
  onClick
}) {
  return (
    <button
      type="button"
      className={`workspace-action-card workspace-action-card-${tone}`}
      onClick={onClick}
    >
      <span className="workspace-action-card__kicker">{kicker}</span>
      <strong>{title}</strong>
      <span className="workspace-action-card__note">{note}</span>
      <span className="workspace-action-card__meta">{meta}</span>
      <span className="workspace-action-card__cta">{cta}</span>
    </button>
  );
}

export function WorkspaceSection({
  title,
  subtitle,
  actions = null,
  children,
  className = "",
  testId = null
}) {
  return (
    <section className={`workspace-section-card ${className}`.trim()} data-testid={testId}>
      <div className="workspace-section-card__header">
        <div>
          <div className="panel-title strategy-card-title-note">
            <StrategyCardNote label={title} note={subtitle} />
          </div>
        </div>
        {actions}
      </div>
      <div className="workspace-section-card__body">{children}</div>
    </section>
  );
}

export function OverviewList({
  title,
  items,
  emptyText,
  renderActions
}) {
  return (
    <div className="workspace-overview-list">
      <div className="mini-list-title">{title}</div>
      {items.length === 0 ? <div className="muted-line">{emptyText}</div> : null}
      {items.map((item) => (
        <div key={item.id} className="workspace-overview-item">
          <div>
            <strong>{item.title}</strong>
            <div className="muted-line">{item.meta}</div>
          </div>
          {renderActions ? (
            <div className="workspace-overview-item__actions">{renderActions(item.raw)}</div>
          ) : null}
        </div>
      ))}
    </div>
  );
}
