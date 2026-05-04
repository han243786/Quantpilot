import { useState } from "react";
import {
  projectStrategyHubRosterRowActionGroups,
  runStrategyHubRosterRowAction
} from "../utils/strategyHubRosterRowActions";
import { buildActionFailureMessage } from "../utils/actionFailure";

export default function StrategyHubRosterRowActions({ model, row }) {
  const [pendingActionKey, setPendingActionKey] = useState("");
  const [errorText, setErrorText] = useState("");
  const actionGroups = projectStrategyHubRosterRowActionGroups(row);
  const actionItems = actionGroups.flatMap((group) =>
    group.items.map((item) => ({
      ...item,
      groupKey: group.key,
      groupLabel: group.label
    }))
  );

  async function handleActionClick(item) {
    setErrorText("");
    setPendingActionKey(item.key);
    try {
      await runStrategyHubRosterRowAction(model, row, item.key);
    } catch (error) {
      const message = buildActionFailureMessage(
        item.key,
        error,
        `${item.label}失败。`
      );
      setErrorText(message);
    } finally {
      setPendingActionKey("");
    }
  }

  return (
    <div className="strategy-row__actions">
      {actionItems.map((item) => (
        <button
          key={item.key}
          className={`strategy-row__action-button ${
            item.buttonClassName || "ghost-btn compact-btn"
          }`.trim()}
          data-testid={`strategy-hub-roster-action-${row.graphId}-${item.key}`}
          data-action-group={item.groupKey}
          aria-label={item.ariaLabel}
          disabled={item.disabled || pendingActionKey === item.key}
          onClick={(event) => {
            event.stopPropagation();
            void handleActionClick(item);
          }}
        >
          {pendingActionKey === item.key ? "处理中" : item.label}
        </button>
      ))}
      {errorText ? (
        <div className="strategy-row__action-error" role="alert">
          {errorText}
        </div>
      ) : null}
    </div>
  );
}
