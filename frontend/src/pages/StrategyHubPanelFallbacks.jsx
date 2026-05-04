export function StrategyHubSectionFallback({ title = "策略中心面板加载中" }) {
  return (
    <section className="strategy-directory-card" aria-hidden="true">
      <div className="strategy-card-header">
        <div>
          <div className="panel-title">{title}</div>
          <div className="strategy-card-subtitle">
            正在加载当前路由的策略中心区块，并保持布局稳定。
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
