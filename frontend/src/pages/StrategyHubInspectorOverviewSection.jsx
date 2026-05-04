import { StrategyRouteBar } from "./BacktestAnalysisLayout";
import { StrategyCardNote, StrategyTaskGroup } from "./StrategyHubSharedComponents";
import {
  projectStrategyHubInspectorOverview
} from "../utils/strategyHubInspectorProjection";
import {
  projectStrategyHubInspectorActionGroups,
  runStrategyHubInspectorAction
} from "../utils/strategyHubInspectorActions";

export default function StrategyHubInspectorOverviewSection({ model, selectedStrategy }) {
  const overview = projectStrategyHubInspectorOverview(selectedStrategy);

  if (!selectedStrategy) {
    return <div className="strategy-directory-empty">{overview.emptyText}</div>;
  }

  const actionGroups = projectStrategyHubInspectorActionGroups(selectedStrategy);

  return (
    <>
      <div className="strategy-card-header">
        <div>
          <StrategyRouteBar items={overview.routeItems} />
          <div className="panel-title strategy-card-title-note">
            <StrategyCardNote label={overview.title} note={overview.subtitle} />
          </div>
        </div>
        <div className={`status-pill ${overview.healthTone}`}>{overview.healthLabel}</div>
      </div>

      <div className="strategy-inspector-title">
        <strong>{overview.strategyName}</strong>
        <span>{overview.strategyId}</span>
      </div>

      <div className="strategy-cockpit-summary">
        {overview.summaryItems.map((item) => (
          <div key={item.label} className="strategy-cockpit-summary__item">
            <span>{item.label}</span>
            <strong>{item.value}</strong>
          </div>
        ))}
      </div>

      <div className="strategy-inspector-actions">
        {actionGroups.map((group) => (
          <StrategyTaskGroup key={group.key} label={group.label} tone={group.tone}>
            {group.items.map((item) => (
              <button
                key={item.key}
                className={group.key === "build" ? "primary-btn" : "ghost-btn"}
                aria-label={item.ariaLabel}
                onClick={() => void runStrategyHubInspectorAction(model, selectedStrategy, item.key)}
              >
                {item.label}
              </button>
            ))}
          </StrategyTaskGroup>
        ))}
      </div>

      <div className="strategy-inspector-metrics">
        {overview.metrics.map((item) => (
          <div key={item.label} className="kv-line">
            <span>{item.label}</span>
            <strong>{item.value}</strong>
          </div>
        ))}
      </div>

      <section className="strategy-inspector-section">
        <div className="mini-list-title">下一步建议</div>
        <div className="strategy-next-move">
          <strong>{overview.nextMove.title}</strong>
          <div className="muted-line">{overview.nextMove.description}</div>
        </div>
      </section>
    </>
  );
}
