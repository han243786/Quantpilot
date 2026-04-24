import { StrategyTaskGroup } from "./StrategyHubSharedComponents";
import {
  projectStrategyHubRosterRowActionGroups,
  runStrategyHubRosterRowAction
} from "../utils/strategyHubRosterRowActions";

export default function StrategyHubRosterRowActions({ model, row }) {
  const actionGroups = projectStrategyHubRosterRowActionGroups(row);

  return (
    <div className="strategy-row__actions">
      {actionGroups.map((group) => (
        <StrategyTaskGroup
          key={group.key}
          label={group.label}
          tone={group.tone}
          className="strategy-task-group--inline strategy-task-group--compact"
        >
          {group.items.map((item) => (
            <button
              key={item.key}
              className="ghost-btn compact-btn"
              data-testid={`strategy-hub-roster-action-${row.graphId}-${item.key}`}
              aria-label={item.ariaLabel}
              disabled={item.disabled}
              onClick={() => void runStrategyHubRosterRowAction(model, row, item.key)}
            >
              {item.label}
            </button>
          ))}
        </StrategyTaskGroup>
      ))}
    </div>
  );
}
