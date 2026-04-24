import StrategyHubRosterTableRow from "./StrategyHubRosterTableRow";

export default function StrategyHubRosterTableSection({ model, rosterRows }) {
  return (
    <div
      className="strategy-directory-table"
      role="table"
      aria-label="策略清单"
      data-testid="strategy-hub-roster-table"
    >
      <div className="strategy-directory-table__head strategy-directory-table__head--roster" role="row">
        <span />
        <span>策略</span>
        <span>状态</span>
        <span>活动</span>
        <span>模拟</span>
        <span>回测</span>
        <span>最近收益</span>
      </div>

      <div className="strategy-directory-table__body" data-testid="strategy-hub-roster-table-body">
        {rosterRows.map((row) => (
          <StrategyHubRosterTableRow key={row.graphId} model={model} row={row} />
        ))}

        {rosterRows.length === 0 ? (
          <div className="strategy-directory-empty">当前筛选条件下没有匹配的策略。</div>
        ) : null}
      </div>
    </div>
  );
}
