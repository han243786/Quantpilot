import { StrategyTaskGroup } from "./StrategyHubSharedComponents";
import {
  projectStrategyHubRecentBacktestActionGroup,
  runStrategyHubRecentBacktestAction
} from "../utils/strategyHubRecentBacktestsActions";

export default function StrategyHubRecentBacktestsSection({
  graphId,
  items,
  onToggleCompare
}) {
  return (
    <section className="strategy-inspector-section">
      <div className="mini-list-title">近期回测</div>
      <div className="strategy-inspector-list">
        {items.map((item) => {
          const actionGroup = projectStrategyHubRecentBacktestActionGroup(item);

          return (
            <div key={item.backtestId} className="strategy-inspector-item">
              <div>
                <strong>{item.backtestId}</strong>
                <div className="muted-line">
                  {item.createdAtLabel} | {item.returnLabel}
                </div>
              </div>
              <div className="strategy-inspector-item__actions">
                <StrategyTaskGroup
                  label={actionGroup.label}
                  tone={actionGroup.tone}
                  className="strategy-task-group--inline"
                >
                  {actionGroup.items.map((action) => (
                    <button
                      key={action.key}
                      className={`ad-btn ad-btn--ghost compact-btn${action.selected ? " compact-btn--selected" : ""}`}
                      aria-label={action.ariaLabel}
                      onClick={() =>
                        runStrategyHubRecentBacktestAction(
                          graphId,
                          item,
                          action.key,
                          onToggleCompare
                        )
                      }
                    >
                      {action.label}
                    </button>
                  ))}
                </StrategyTaskGroup>
              </div>
            </div>
          );
        })}
        {items.length === 0 ? (
          <div>
            <div className="muted-line">这条策略还没有持久化回测。</div>
            <div className="muted-line" style={{ marginTop: 4, fontSize: 12 }}>请先编译策略，然后点击工具栏"运行回测"按钮。</div>
          </div>
        ) : null}
      </div>
    </section>
  );
}
