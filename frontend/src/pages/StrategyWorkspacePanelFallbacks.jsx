export function EventPanelFallback() {
  return (
    <section className="workspace-section-card workspace-event-fallback" aria-hidden="true">
      <div className="workspace-section-card__header">
        <div>
          <div className="panel-title">Research Event Stream</div>
          <div className="strategy-card-subtitle">
            Loading the research console while keeping the current workspace layout stable.
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

export function WorkspacePanelFallback({ title = "Workspace Panel Loading" }) {
  return (
    <section className="workspace-section-card workspace-event-fallback" aria-hidden="true">
      <div className="workspace-section-card__header">
        <div>
          <div className="panel-title">{title}</div>
          <div className="strategy-card-subtitle">
            Loading the components required for the current workspace mode.
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
