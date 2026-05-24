import { buildStrategyIdentity } from "../utils/strategyHubStrategyIdentity";
import StrategyHubRosterRowActions from "./StrategyHubRosterRowActions";

export default function StrategyHubRosterTableRow({ model, row }) {
  const strategyIdentity = buildStrategyIdentity(row);

  return (
    <div
      className={`strategy-row-shell${row.active ? " strategy-row-shell--active" : ""}`}
      data-testid={`strategy-hub-roster-row-shell-${row.graphId}`}
    >
      <label className="strategy-row__select">
        <input
          type="checkbox"
          checked={row.selected}
          onChange={() => model.toggleStrategySelection(row.graphId)}
          aria-label={`选择策略 ${strategyIdentity}`}
        />
      </label>
      <button
        className={`strategy-row${row.active ? " strategy-row--active" : ""}`}
        data-testid={`strategy-hub-roster-row-select-${row.graphId}`}
        onClick={() => model.setSelectedStrategyId(row.graphId)}
      >
        <span className="strategy-row__main" data-label="策略">
          <strong>{row.name}</strong>
          <small>{row.graphId}</small>
        </span>
        <span className="strategy-row__cell" data-label="状态">
          <span className={`status-pill ${row.healthTone}`}>{row.healthLabel}</span>
        </span>
        <span className="strategy-row__activity" data-label="活动">
          <strong>{row.activityLabel}</strong>
          <small>{row.lastActivityLabel}</small>
        </span>
        <span className="strategy-row__cell" data-label="模拟">
          {row.runCountLabel}
        </span>
        <span className="strategy-row__cell" data-label="回测">
          {row.backtestCountLabel}
        </span>
        <span className="strategy-row__cell" data-label="最近收益">
          {row.latestReturnLabel}
        </span>
      </button>
      <StrategyHubRosterRowActions model={model} row={row} />
    </div>
  );
}
