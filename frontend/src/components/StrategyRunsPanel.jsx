import {
  AccountSection,
  RunHistorySection
} from "./EventStreamPanel";

export default function StrategyRunsPanel({
  detailMode = false,
  className = "",
  graph,
  runtime,
  openOrders,
  runFilters,
  showAccount = true,
  showHistory = true,
  pagedHistory,
  filteredHistory,
  currentPage,
  totalPages,
  handleRefreshRunHistory,
  setRunHistoryFilter,
  setRunHistoryCompileFilter,
  setRunHistoryFromTime,
  setRunHistoryToTime,
  setRunHistoryStatusFilter,
  setRunHistorySortOrder,
  setRunHistoryPage,
  setRunHistoryPageSize,
  loadRunDetail
}) {
  return (
    <div className={`event-runs-panel ${className}`.trim()}>
      {showAccount ? <AccountSection runtime={runtime} openOrders={openOrders} /> : null}
      {showHistory ? (
        <RunHistorySection
          detailMode={detailMode}
          graph={graph}
          runtime={runtime}
          historyFilter={runFilters?.historyFilter}
          historyCompileFilter={runFilters?.historyCompileFilter}
          historyFromTime={runFilters?.historyFromTime}
          historyToTime={runFilters?.historyToTime}
          historyStatusFilter={runFilters?.historyStatusFilter}
          historySortOrder={runFilters?.historySortOrder}
          historyPageSize={runFilters?.historyPageSize}
          pagedHistory={pagedHistory}
          filteredHistory={filteredHistory}
          currentPage={currentPage}
          totalPages={totalPages}
          handleRefreshRunHistory={handleRefreshRunHistory}
          setRunHistoryFilter={setRunHistoryFilter}
          setRunHistoryCompileFilter={setRunHistoryCompileFilter}
          setRunHistoryFromTime={setRunHistoryFromTime}
          setRunHistoryToTime={setRunHistoryToTime}
          setRunHistoryStatusFilter={setRunHistoryStatusFilter}
          setRunHistorySortOrder={setRunHistorySortOrder}
          setRunHistoryPage={setRunHistoryPage}
          setRunHistoryPageSize={setRunHistoryPageSize}
          loadRunDetail={loadRunDetail}
        />
      ) : null}
    </div>
  );
}
