import {
  projectStrategyHubCompareQueueView,
  runStrategyHubCompareQueueAction
} from "../utils/strategyHubCompareQueueActions";

export default function StrategyHubCompareQueueSection({
  graphId,
  compareQueue,
  onClearSelection
}) {
  const view = projectStrategyHubCompareQueueView(compareQueue);

  return (
    <section className="strategy-inspector-section">
      <div className="mini-list-title">{view.title}</div>
      <div className="strategy-compare-queue">
        <div className="muted-line">{view.description}</div>
        <div className="strategy-compare-queue__chips">
          {view.chips.length === 0 ? (
            <span className="status-pill muted">{view.emptyLabel}</span>
          ) : (
            view.chips.map((backtestId) => (
              <span key={backtestId} className="status-pill info">
                {backtestId}
              </span>
            ))
          )}
        </div>
        <div className="strategy-inspector-actions">
          {view.actions.map((action) => (
            <button
              key={action.key}
              className={action.tone === "primary" ? "primary-btn" : "ghost-btn"}
              aria-label={action.ariaLabel}
              disabled={action.disabled}
              onClick={() =>
                runStrategyHubCompareQueueAction(
                  graphId,
                  compareQueue,
                  action.key,
                  onClearSelection
                )
              }
            >
              {action.label}
            </button>
          ))}
        </div>
      </div>
    </section>
  );
}
