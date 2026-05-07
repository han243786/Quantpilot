import { useI18n } from "../i18n";
import { backtestComparePath, navigateTo } from "../router";
import { buildDiagnosticsExplanationEntries } from "../utils/runtimeExplanation";
import {
  COPY,
  HISTORY_COPY,
  EventSection,
  SectionCardHeader,
  HistoryCopyBlock,
  HistoryExplanationCard,
  HistoryFilterGrid,
  HistoryControlBar,
  HistoryNotice,
  HistoryCardHeader,
  HistoryMetaGrid,
  HistoryMetaRow,
  HistoryPagination,
  formatRatio,
  ratioTone,
  drawdownTone,
  formatValue,
  backtestExecutionAssumptionsLabel,
} from "./EventStreamPanel";

export default function BacktestHistorySection({
  detailMode,
  graph,
  runtime,
  backtestHistoryFilter,
  backtestCompileFilter,
  backtestDatasetFilter,
  backtestParameterFilter,
  backtestFromTime,
  backtestToTime,
  backtestPageSize,
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
  onOpenBacktestDetail
}) {
  const { t } = useI18n();
  if (detailMode) return null;
  const selectedBacktestRiskEntries = runtime.selectedBacktestId
    ? buildDiagnosticsExplanationEntries(graph, runtime.diagnostics, "risk")
    : [];
  const selectedBacktestOrderEntries = runtime.selectedBacktestId
    ? buildDiagnosticsExplanationEntries(graph, runtime.diagnostics, "order")
    : [];
  const selectedBacktestDataQualityEntries = runtime.selectedBacktestId
    ? buildDiagnosticsExplanationEntries(graph, runtime.diagnostics, "dataQuality")
    : [];

  return (
    <EventSection
      kicker={t("历史")}
      className="event-sidebar-section event-sidebar-section-backtest"
      title={t("回测历史")}
      summary={t("把筛选、对比选择和详情入口统一放在同一块分析区域。")}
    >
      <div className="backtest-history-card" data-testid="backtest-history-card">
        <SectionCardHeader
          title={t("持久化回测记录")}
          summary={t("按图、编译、数据集、参数和时间窗口筛选历史结果。")}
          action={
            <button
              className="ghost-btn compact-btn"
              aria-label={t("刷新回测历史")}
              data-testid="backtest-history-refresh"
              disabled={runtime.backtestHistoryStatus === "loading"}
              onClick={() => handleRefreshBacktestHistory()}
            >
              {HISTORY_COPY.refresh}
            </button>
          }
        />
        <HistoryCopyBlock title={COPY.backtestListTitle} subtitle={COPY.backtestListSubtitle} />
        {runtime.selectedBacktestId ? (
          <div className="analysis-card-grid analysis-card-grid--two">
            <HistoryExplanationCard
              title={t("已选回测风控解释")}
              summary={t("当前已加载回测 {id} 的风控说明。", { id: runtime.selectedBacktestId })}
              entries={selectedBacktestRiskEntries}
              testId="backtest-history-risk-explanations"
            />
            <HistoryExplanationCard
              title={t("已选回测订单解释")}
              summary={t("当前已加载回测 {id} 的订单说明。", { id: runtime.selectedBacktestId })}
              entries={selectedBacktestOrderEntries}
              testId="backtest-history-order-explanations"
            />
            <HistoryExplanationCard
              title={t("已选回测数据质量")}
              summary={`Data quality details for loaded backtest ${runtime.selectedBacktestId}.`}
              entries={selectedBacktestDataQualityEntries}
              testId="backtest-history-data-quality"
            />
          </div>
        ) : null}
        <HistoryFilterGrid
          className="history-filter-grid-backtest"
          fields={[
            {
              key: "graph",
              value: backtestHistoryFilter ?? runtime.backtestHistoryFilter ?? "",
              placeholder: t("按图 ID 过滤"),
              onChange: setBacktestHistoryFilter
            },
            {
              key: "compile",
              value: backtestCompileFilter ?? runtime.backtestCompileFilter ?? "",
              placeholder: t("按编译 ID 过滤"),
              onChange: setBacktestCompileFilter
            },
            {
              key: "dataset",
              value: backtestDatasetFilter ?? runtime.backtestDatasetFilter ?? "",
              placeholder: t("按数据集过滤"),
              onChange: setBacktestDatasetFilter
            },
            {
              key: "parameter",
              value: backtestParameterFilter ?? runtime.backtestParameterFilter ?? "",
              placeholder: t("按参数过滤"),
              onChange: setBacktestParameterFilter
            },
            {
              key: "fromTime",
              type: "datetime-local",
              ariaLabel: t("回测开始时间过滤"),
              className: "history-filter-input history-filter-time",
              value: backtestFromTime ?? runtime.backtestFromTime ?? "",
              onChange: setBacktestFromTime
            },
            {
              key: "toTime",
              type: "datetime-local",
              ariaLabel: t("回测结束时间过滤"),
              className: "history-filter-input history-filter-time",
              value: backtestToTime ?? runtime.backtestToTime ?? "",
              onChange: setBacktestToTime
            }
          ]}
        />
        <HistoryControlBar
          className="history-control-bar-backtest"
          refreshAriaLabel="刷新回测历史"
          refreshDisabled={runtime.backtestHistoryStatus === "loading"}
          onRefresh={() => handleRefreshBacktestHistory()}
          pageSize={backtestPageSize ?? runtime.backtestPageSize ?? 6}
          onPageSizeChange={setBacktestPageSize}
          summary={
            compareSelection.length > 0
              ? HISTORY_COPY.compareSelection(compareSelection.length)
              : HISTORY_COPY.resultCount(filteredBacktests.length)
          }
          actions={
            <>
          <button
            className="ghost-btn compact-btn"
            data-testid="backtest-history-open-compare"
            disabled={compareSelection.length !== 2}
            onClick={() =>
              navigateTo(backtestComparePath(compareSelection, graph.metadata?.graph_id || ""))
            }
          >
            {t("打开对比")} ({compareSelection.length}/2)
          </button>
          <button className="ghost-btn compact-btn" onClick={() => clearBacktestCompareSelection()}>
            {HISTORY_COPY.clearCompare}
          </button>
          <button
            className="ghost-btn compact-btn"
            onClick={() => setBacktestHistoryFilter(graph.metadata?.graph_id || "")}
          >
            {HISTORY_COPY.currentGraph}
          </button>
          <button
            className="ghost-btn compact-btn"
            data-testid="backtest-history-reset"
            onClick={() => {
              setBacktestHistoryFilter("");
              setBacktestCompileFilter("");
              setBacktestDatasetFilter("");
              setBacktestParameterFilter("");
              setBacktestFromTime("");
              setBacktestToTime("");
              setBacktestPage(1);
              clearBacktestCompareSelection();
            }}
          >
            {HISTORY_COPY.reset}
          </button>
            </>
          }
        />
        {compareSelection.length > 0 ? (
          <HistoryNotice>{`已选对比：${compareSelection.join(", ")}`}</HistoryNotice>
        ) : null}
        {runtime.backtestHistoryStatus === "loading" ? (
          <HistoryNotice>{t("正在加载回测历史...")}</HistoryNotice>
        ) : null}
        {filteredBacktests.length === 0 && runtime.backtestHistoryStatus !== "loading" ? (
          <HistoryNotice>{t("当前过滤条件下没有回测记录。")}</HistoryNotice>
        ) : null}
        <div className="run-history-list">
          {pagedBacktests.map((item) => {
            const isCompareSelected = compareSelection.includes(item.backtest_id);
            const disableCompareToggle = !isCompareSelected && compareSelection.length >= 2;
            return (
              <div
                key={item.backtest_id}
                data-testid={`backtest-history-row-${item.backtest_id}`}
                className={`run-history-item-group ${
                  isCompareSelected ? "run-history-item-group-compare-selected" : ""
                }`.trim()}
              >
                <button
                  className={`run-history-item ${
                    runtime.selectedBacktestId === item.backtest_id ? "active" : ""
                  }`}
                  onClick={() =>
                    onOpenBacktestDetail
                      ? onOpenBacktestDetail(item.backtest_id)
                      : loadBacktestDetail(item.backtest_id)
                  }
                >
                  <HistoryCardHeader
                    id={item.backtest_id}
                    timestamp={new Date(item.created_at_ms).toLocaleString()}
                  />
                  <HistoryMetaGrid
                    className="history-meta-grid-primary"
                    items={[
                      { label: "图", value: item.graph_id || "-" },
                      { label: "编译", value: item.compile_id || "-" },
                      {
                        label: "收益",
                        value: formatRatio(item.summary?.total_return_ratio),
                        tone: ratioTone(item.summary?.total_return_ratio)
                      },
                      {
                        label: "回撤",
                        value: formatRatio(item.summary?.max_drawdown_ratio),
                        tone: drawdownTone(item.summary?.max_drawdown_ratio)
                      },
                      {
                        label: "时间窗",
                        value: `${item.filters?.started_at_ms ? new Date(item.filters.started_at_ms).toLocaleString() : "-"} ~ ${
                          item.filters?.ended_at_ms
                            ? new Date(item.filters.ended_at_ms).toLocaleString()
                            : "-"
                        }`,
                        wide: true
                      }
                    ]}
                  />
                  <HistoryMetaGrid
                    className="history-meta-grid-secondary"
                    items={[
                      { label: "成交", value: formatValue(item.summary?.trade_count) },
                      {
                        label: "数据集",
                        value: (item.filters?.dataset_labels || []).join(", ") || "-",
                        wide: true
                      },
                      {
                        label: "参数",
                        value: backtestExecutionAssumptionsLabel(item.filters),
                        wide: true
                      },
                      { label: "回放", value: item.filters?.replay_source || "-" }
                    ]}
                  />
                  <HistoryMetaRow
                    items={[
                      { label: "图", value: item.graph_id || "-" },
                      { label: "编译", value: item.compile_id || "-" },
                      { label: "收益", value: formatRatio(item.summary?.total_return_ratio) },
                      { label: "回撤", value: formatRatio(item.summary?.max_drawdown_ratio) },
                      { label: "成交", value: formatValue(item.summary?.trade_count) }
                    ]}
                  />
                  <HistoryMetaRow
                    items={[
                      {
                        label: "数据集",
                        value: (item.filters?.dataset_labels || []).join(", ") || "-"
                      },
                      { label: "参数", value: backtestExecutionAssumptionsLabel(item.filters) }
                    ]}
                  />
                  <HistoryMetaRow
                    items={[
                      { label: "回放", value: item.filters?.replay_source || "-" },
                      {
                        label: "时间窗",
                        value: `${
                          item.filters?.started_at_ms
                            ? new Date(item.filters.started_at_ms).toLocaleString()
                            : "-"
                        } ~ ${
                          item.filters?.ended_at_ms
                            ? new Date(item.filters.ended_at_ms).toLocaleString()
                            : "-"
                        }`
                      }
                    ]}
                  />
                </button>
                <div className="history-item-actions">
                  <button
                    className={`ghost-btn compact-btn history-compare-chip ${
                      isCompareSelected ? "active" : ""
                    }`.trim()}
                    data-testid={`backtest-history-compare-toggle-${item.backtest_id}`}
                    disabled={disableCompareToggle}
                    onClick={() => toggleBacktestCompareSelection(item.backtest_id)}
                  >
                    {isCompareSelected ? t("取消对比") : t("加入对比")}
                  </button>
                </div>
              </div>
            );
          })}
        </div>
        <HistoryPagination
          currentPage={backtestCurrentPage}
          totalPages={backtestTotalPages}
          onPrevious={() => setBacktestPage(backtestCurrentPage - 1)}
          onNext={() => setBacktestPage(backtestCurrentPage + 1)}
        />
      </div>
    </EventSection>
  );
}
