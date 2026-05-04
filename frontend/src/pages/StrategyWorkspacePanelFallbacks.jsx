export function EventPanelFallback() {
  return (
    <section className="workspace-section-card workspace-event-fallback" aria-hidden="true">
      <div className="workspace-section-card__header">
        <div>
          <div className="panel-title">研究事件流</div>
          <div className="strategy-card-subtitle">
            正在加载研究控制台，并保持当前工作区布局稳定。
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

export function WorkspacePanelFallback({ title = "工作区面板加载中" }) {
  return (
    <section className="workspace-section-card workspace-event-fallback" aria-hidden="true">
      <div className="workspace-section-card__header">
        <div>
          <div className="panel-title">{title}</div>
          <div className="strategy-card-subtitle">
            正在加载当前工作区模式所需组件。
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
