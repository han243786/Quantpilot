import { useState } from "react";
import {
  projectStrategyHubRosterRowActionGroups,
  runStrategyHubRosterRowAction
} from "../utils/strategyHubRosterRowActions";
import { buildActionFailureMessage } from "../utils/actionFailure";

export default function StrategyHubRosterRowActions({ model, row }) {
  const [pendingActionKey, setPendingActionKey] = useState("");
  const [errorText, setErrorText] = useState("");
  const [menuOpen, setMenuOpen] = useState(false);
  const actionGroups = projectStrategyHubRosterRowActionGroups(row);
  const actionItems = actionGroups.flatMap((group) =>
    group.items.map((item) => ({
      ...item,
      groupKey: group.key,
      groupLabel: group.label
    }))
  );
  const primaryItem = actionItems.find((item) => item.key === "open-workspace") || actionItems[0];
  const secondaryItems = actionItems.filter((item) => item.key !== primaryItem?.key);
  const menuId = `strategy-hub-roster-action-${row.graphId}-menu`;

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

  function renderActionButton(item) {
    return (
      <button
        key={item.key}
        className={`strategy-row__action-button ${
          item.buttonClassName || "ad-btn ad-btn--ghost compact-btn"
        }`.trim()}
        data-testid={`strategy-hub-roster-action-${row.graphId}-${item.key}`}
        data-action-group={item.groupKey}
        aria-label={item.ariaLabel}
        disabled={item.disabled || pendingActionKey === item.key}
        onClick={(event) => {
          event.stopPropagation();
          setMenuOpen(false);
          void handleActionClick(item);
        }}
      >
        {pendingActionKey === item.key ? "处理中" : item.label}
      </button>
    );
  }

  return (
    <div className="strategy-row__actions">
      {primaryItem ? renderActionButton(primaryItem) : null}
      <button
        type="button"
        className="strategy-row__action-more ad-btn ad-btn--ghost compact-btn"
        data-testid={`strategy-hub-roster-action-${row.graphId}-more`}
        aria-label={`打开策略 ${row.name}（${row.graphId}）的更多操作`}
        aria-expanded={menuOpen}
        aria-controls={menuId}
        onClick={(event) => {
          event.stopPropagation();
          setMenuOpen((value) => !value);
        }}
      >
        更多
      </button>
      {menuOpen ? (
        <div
          id={menuId}
          className="strategy-row__action-menu"
          aria-label={`策略 ${row.name}（${row.graphId}）更多操作`}
        >
          {secondaryItems.map((item) => renderActionButton(item))}
        </div>
      ) : null}
      {errorText ? (
        <div className="strategy-row__action-error" role="alert">
          {errorText}
        </div>
      ) : null}
    </div>
  );
}
