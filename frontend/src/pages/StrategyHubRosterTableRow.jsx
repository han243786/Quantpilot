import StrategyHubRosterRowActions from "./StrategyHubRosterRowActions";

export default function StrategyHubRosterTableRow({ model, row }) {
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
          aria-label={`选择 ${row.name}`}
        />
      </label>
      <button
        className={`strategy-row${row.active ? " strategy-row--active" : ""}`}
        data-testid={`strategy-hub-roster-row-select-${row.graphId}`}
        onClick={() => model.setSelectedStrategyId(row.graphId)}
      >
        <span className="strategy-row__main">
          <strong>{row.name}</strong>
          <small>{row.graphId}</small>
        </span>
        <span className={`status-pill ${row.healthTone}`}>{row.healthLabel}</span>
        <span className="strategy-row__activity">
          <strong>{row.activityLabel}</strong>
          <small>{row.lastActivityLabel}</small>
        </span>
        <span>{row.runCountLabel}</span>
        <span>{row.backtestCountLabel}</span>
        <span>{row.latestReturnLabel}</span>
      </button>
      <StrategyHubRosterRowActions model={model} row={row} />
    </div>
  );
}
