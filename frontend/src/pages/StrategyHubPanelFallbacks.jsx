export function StrategyHubSectionFallback({ title = "Strategy Hub Panel Loading" }) {
  return (
    <section className="strategy-directory-card" aria-hidden="true">
      <div className="strategy-card-header">
        <div>
          <div className="panel-title">{title}</div>
          <div className="strategy-card-subtitle">
            Loading the route-owned hub sections while keeping the current layout stable.
          </div>
        </div>
      </div>
      <div className="workspace-event-fallback__body">
        <div className="event-panel-loading-card" />
        <div className="event-panel-loading-card event-panel-loading-card-wide" />
      </div>
    </section>
  );
}
