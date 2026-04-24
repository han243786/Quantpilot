import {
  BacktestHistorySection,
  BacktestSummarySection
} from "./EventStreamPanel";

export default function StrategyBacktestsPanel({
  detailMode = false,
  className = "",
  graph,
  runtime,
  selectedBacktestSummary,
  backtestSummary,
  backtestStartedAt,
  backtestEndedAt,
  backtestFilters,
  showHistory = true,
  showSummary = true,
  pagedBacktests,
  filteredBacktests,
  backtestCurrentPage,
  backtestTotalPages,
  compareSelection,
  handleRefreshBacktestHistory,
  setBacktestHistoryFilter,
  setBacktestCompileFilter,
  setBacktestDatasetFilter,
  setBacktestParameterFilter,
  setBacktestFromTime,
  setBacktestToTime,
  setBacktestPage,
  setBacktestPageSize,
  toggleBacktestCompareSelection,
  clearBacktestCompareSelection,
  loadBacktestDetail,
  onOpenBacktestDetail = null
}) {
  return (
    <div className={`event-backtests-panel ${className}`.trim()}>
      {showSummary ? (
        <BacktestSummarySection
          runtime={runtime}
          selectedBacktestSummary={selectedBacktestSummary}
          backtestSummary={backtestSummary}
          backtestStartedAt={backtestStartedAt}
          backtestEndedAt={backtestEndedAt}
        />
      ) : null}
      {showHistory ? (
        <BacktestHistorySection
          detailMode={detailMode}
          graph={graph}
          runtime={runtime}
          backtestHistoryFilter={backtestFilters?.backtestHistoryFilter}
          backtestCompileFilter={backtestFilters?.backtestCompileFilter}
          backtestDatasetFilter={backtestFilters?.backtestDatasetFilter}
          backtestParameterFilter={backtestFilters?.backtestParameterFilter}
          backtestFromTime={backtestFilters?.backtestFromTime}
          backtestToTime={backtestFilters?.backtestToTime}
          backtestPageSize={backtestFilters?.backtestPageSize}
          pagedBacktests={pagedBacktests}
          filteredBacktests={filteredBacktests}
          backtestCurrentPage={backtestCurrentPage}
          backtestTotalPages={backtestTotalPages}
          compareSelection={compareSelection}
          handleRefreshBacktestHistory={handleRefreshBacktestHistory}
          setBacktestHistoryFilter={setBacktestHistoryFilter}
          setBacktestCompileFilter={setBacktestCompileFilter}
          setBacktestDatasetFilter={setBacktestDatasetFilter}
          setBacktestParameterFilter={setBacktestParameterFilter}
          setBacktestFromTime={setBacktestFromTime}
          setBacktestToTime={setBacktestToTime}
          setBacktestPage={setBacktestPage}
          setBacktestPageSize={setBacktestPageSize}
          toggleBacktestCompareSelection={toggleBacktestCompareSelection}
          clearBacktestCompareSelection={clearBacktestCompareSelection}
          loadBacktestDetail={loadBacktestDetail}
          onOpenBacktestDetail={onOpenBacktestDetail}
        />
      ) : null}
    </div>
  );
}
