import { useI18n } from "../i18n";
import { buildDiagnosticsExplanationEntries } from "../utils/runtimeExplanation";
import { runtimeStatusLabel } from "../utils/runtimeStatus";
import RuntimeMutationPanel from "./RuntimeMutationPanel";
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
  formatValue,
  resolveRunStatus,
  runtimeTone,
} from "./EventStreamPanel";

export default function RunHistorySection({
  detailMode,
  graph,
  runtime,
  historyFilter,
  historyCompileFilter,
  historyFromTime,
  historyToTime,
  historyStatusFilter,
  historySortOrder,
  historyPageSize,
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
  const { t } = useI18n();
  if (detailMode) return null;
  const selectedRunRiskEntries = runtime.runId
    ? buildDiagnosticsExplanationEntries(graph, runtime.diagnostics, "risk")
    : [];
  const selectedRunOrderEntries = runtime.runId
    ? buildDiagnosticsExplanationEntries(graph, runtime.diagnostics, "order")
    : [];
  const selectedRunDataQualityEntries = runtime.runId
    ? buildDiagnosticsExplanationEntries(graph, runtime.diagnostics, "dataQuality")
    : [];

  return (
    <EventSection
      kicker={t("历史")}
      className="event-sidebar-section event-sidebar-section-run"
      title={t("运行历史")}
      summary={t("聚焦模拟运行记录，保留状态过滤、时间范围和详情恢复。")}
    >
      <div className="run-history-card" data-testid="run-history-card">
        <SectionCardHeader
          title={t("持久化运行记录")}
          summary={t("查看后端已保存的运行结果，并恢复指定运行详情。")}
          action={
            <button
              className="ad-btn ad-btn--ghost compact-btn"
              aria-label={t("刷新运行记录")}
              data-testid="run-history-refresh"
              disabled={runtime.historyStatus === "loading"}
              onClick={() => handleRefreshRunHistory()}
            >
              {HISTORY_COPY.refresh}
            </button>
          }
        />
        <HistoryCopyBlock title={COPY.runListTitle} subtitle={COPY.runListSubtitle} />
        {runtime.runId ? (
          <div className="analysis-card-grid analysis-card-grid--two">
            <HistoryExplanationCard
              title={t("已选运行风控解释")}
              summary={t("当前已加载运行 {id} 的风控说明。", { id: runtime.runId })}
              entries={selectedRunRiskEntries}
              testId="run-history-risk-explanations"
            />
            <HistoryExplanationCard
              title={t("已选运行订单解释")}
              summary={t("当前已加载运行 {id} 的订单说明。", { id: runtime.runId })}
              entries={selectedRunOrderEntries}
              testId="run-history-order-explanations"
            />
            <HistoryExplanationCard
              title={t("已选运行数据质量")}
              summary={`Data quality details for loaded run ${runtime.runId}.`}
              entries={selectedRunDataQualityEntries}
              testId="run-history-data-quality"
            />
            <RuntimeMutationPanel
              sourceKind="run"
              sourceId={runtime.runId}
              capabilityContext={runtime.governance ? { schema_hash: runtime.governance.capability_hash } : null}
              initialMutations={runtime.parameterMutations || []}
              title={t("参数变更")}
              testId="run-history-mutation-panel"
            />
          </div>
        ) : null}
        <HistoryFilterGrid
          className="history-filter-grid-run"
          fields={[
            {
              key: "graph",
              value: historyFilter ?? runtime.historyFilter ?? "",
              placeholder: t("按图 ID 过滤"),
              onChange: setRunHistoryFilter
            },
            {
              key: "compile",
              value: historyCompileFilter ?? runtime.historyCompileFilter ?? "",
              placeholder: t("按编译 ID 过滤"),
              onChange: setRunHistoryCompileFilter
            },
            {
              key: "fromTime",
              type: "datetime-local",
              className: "history-filter-input history-filter-time",
              value: historyFromTime ?? runtime.historyFromTime ?? "",
              onChange: setRunHistoryFromTime
            },
            {
              key: "toTime",
              type: "datetime-local",
              className: "history-filter-input history-filter-time",
              value: historyToTime ?? runtime.historyToTime ?? "",
              onChange: setRunHistoryToTime
            },
            {
              key: "status",
              type: "select",
              value: historyStatusFilter ?? runtime.historyStatusFilter ?? "all",
              onChange: setRunHistoryStatusFilter,
              options: [
                { value: "all", label: t("全部状态") },
                { value: "completed", label: t("已完成") },
                { value: "running", label: t("运行中") },
                { value: "connecting", label: t("连接中") },
                 { value: "error", label: t("错误") },
                { value: "stopped", label: t("已停止") }
              ]
            },
            {
              key: "sortOrder",
              type: "select",
              value: historySortOrder ?? runtime.historySortOrder ?? "desc",
              onChange: setRunHistorySortOrder,
              options: [
                 { value: "desc", label: t("时间倒序") },
                 { value: "asc", label: t("时间正序") }
              ]
            }
          ]}
        />
        <HistoryControlBar
          className="history-control-bar-run"
          refreshAriaLabel={t("刷新运行记录")}
          refreshDisabled={runtime.historyStatus === "loading"}
          onRefresh={() => handleRefreshRunHistory()}
          pageSize={historyPageSize ?? runtime.historyPageSize ?? 6}
          onPageSizeChange={setRunHistoryPageSize}
          summary={HISTORY_COPY.resultCount(filteredHistory.length)}
          actions={
            <>
              <button
                className="ad-btn ad-btn--ghost compact-btn"
                onClick={() => setRunHistoryFilter(graph.metadata?.graph_id || "")}
              >
                {HISTORY_COPY.currentGraph}
              </button>
              <button
                className="ad-btn ad-btn--ghost compact-btn"
                onClick={() => {
                  setRunHistoryFilter("");
                  setRunHistoryCompileFilter("");
                  setRunHistoryFromTime("");
                  setRunHistoryToTime("");
                  setRunHistoryStatusFilter("all");
                  setRunHistorySortOrder("desc");
                  setRunHistoryPage(1);
                }}
              >
                {HISTORY_COPY.reset}
              </button>
            </>
          }
        />
        {runtime.historyStatus === "loading" ? (
          <HistoryNotice>{t("正在加载运行记录...")}</HistoryNotice>
        ) : null}
        {filteredHistory.length === 0 && runtime.historyStatus !== "loading" ? (
          <HistoryNotice>{t("当前过滤条件下没有运行记录。")}</HistoryNotice>
        ) : null}
        <div className="run-history-list">
          {pagedHistory.map((run) => {
            const effectiveStatus = resolveRunStatus(run, runtime);
            return (
              <button
                key={run.run_id}
                className={`run-history-item ${
                  runtime.selectedHistoryRunId === run.run_id ? "active" : ""
                }`}
                onClick={() => loadRunDetail(run.run_id)}
              >
                <HistoryCardHeader
                  id={run.run_id}
                  timestamp={new Date(run.created_at_ms).toLocaleString()}
                />
                <HistoryMetaGrid
                  className="history-meta-grid-primary history-meta-grid-runtime"
                  items={[
                      { label: "图", value: run.graph_id || "-" },
                      {
                        label: "状态",
                        value: runtimeStatusLabel(effectiveStatus),
                        tone: runtimeTone(effectiveStatus)
                      },
                      { label: "编译", value: run.compile_id || "-" },
                    { label: "事件", value: formatValue(run.event_count) }
                  ]}
                />
                <HistoryMetaRow
                  items={[
                    { label: "图", value: run.graph_id || "-" },
                    { label: "状态", value: runtimeStatusLabel(effectiveStatus) },
                    { label: "编译", value: run.compile_id || "-" },
                    { label: "事件", value: formatValue(run.event_count) }
                  ]}
                />
              </button>
            );
          })}
        </div>
        <HistoryPagination
          currentPage={currentPage}
          totalPages={totalPages}
          onPrevious={() => setRunHistoryPage(currentPage - 1)}
          onNext={() => setRunHistoryPage(currentPage + 1)}
        />
      </div>
    </EventSection>
  );
}
