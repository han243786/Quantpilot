export function StrategyHubRosterFallback() {
  return (
    <div className="strategy-hub-roster-stack" aria-hidden="true">
      <section className="strategy-directory-card">
        <div className="strategy-card-header">
          <div>
            <div className="panel-title">策略列表加载中</div>
            <div className="strategy-card-subtitle">
              保持当前布局稳定，正在加载策略清单与近期活动。
            </div>
          </div>
        </div>
        <div className="workspace-event-fallback__body">
          <div className="event-panel-loading-card" />
          <div className="event-panel-loading-card event-panel-loading-card-wide" />
        </div>
      </section>
    </div>
  );
}

export function StrategyHubInspectorFallback() {
  return (
    <aside className="strategy-inspector-card" aria-hidden="true">
      <div className="strategy-card-header">
        <div>
          <div className="panel-title">策略驾驶舱加载中</div>
          <div className="strategy-card-subtitle">
            正在整理当前选中策略的健康、研究和对比状态。
          </div>
        </div>
      </div>
      <div className="workspace-event-fallback__body">
        <div className="event-panel-loading-card" />
        <div className="event-panel-loading-card event-panel-loading-card-wide" />
      </div>
    </aside>
  );
}

export function StrategyHubInspectorSectionFallback({ title }) {
  return (
    <section className="strategy-inspector-section" aria-hidden="true">
      <div className="mini-list-title">{title}</div>
      <div className="workspace-event-fallback__body">
        <div className="event-panel-loading-card" />
      </div>
    </section>
  );
}

export function StrategyHubTemplateLibraryFallback() {
  return (
    <section className="strategy-template-library" aria-hidden="true">
      <div className="strategy-card-header">
        <div>
          <div className="panel-title">模板库加载中</div>
          <div className="strategy-card-subtitle">
            正在准备可安全加载到当前草稿的策略模板。
          </div>
        </div>
      </div>
      <div className="strategy-template-grid">
        <div className="event-panel-loading-card" />
        <div className="event-panel-loading-card" />
        <div className="event-panel-loading-card" />
      </div>
    </section>
  );
}
