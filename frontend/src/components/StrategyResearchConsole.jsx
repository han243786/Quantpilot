import { useMemo, useState } from "react";
import { EventPanelIntro } from "./EventStreamPanel";
import { useStrategyResearchModel } from "../hooks/useStrategyResearchModel";
import { StrategyCardNote } from "../pages/StrategyHubSharedComponents";
import StrategyBacktestsPanel from "./StrategyBacktestsPanel";
import StrategyEventsPanel from "./StrategyEventsPanel";
import StrategyRunsPanel from "./StrategyRunsPanel";

const RESEARCH_CONSOLE_NOTE =
  "每次聚焦一个主要历史视图，同时把事件流与辅助上下文保留在侧栏中。";

const RESEARCH_MODES = [
  {
    id: "backtests",
    label: "回测",
    note: "研究历史、对比队列与回放快照。"
  },
  {
    id: "runs",
    label: "运行",
    note: "模拟历史、账户状态与执行恢复。"
  }
];

const EVENT_COLORS = {
  DataUpdated: "var(--ad-accent)",
  IntentTriggered: "var(--ad-warning)",
  AgentDecisionProduced: "var(--ad-text-secondary)",
  ExecutionPlanned: "var(--ad-success)",
  ExecutionFilled: "var(--ad-success)",
  PortfolioUpdated: "var(--ad-warning)",
  RiskDecisionProduced: "var(--ad-error)"
};

function ResearchMetricCard({ label, value, note, tone = "muted", testId }) {
  return (
    <div
      className={`research-console-metric research-console-metric-${tone}`}
      data-testid={testId || `research-metric-${label}`}
    >
      <span>{label}</span>
      <strong>{value}</strong>
      <small>{note}</small>
    </div>
  );
}

export default function StrategyResearchConsole({
  detailMode = false,
  onOpenBacktestDetail = null
}) {
  const model = useStrategyResearchModel();
  const [primaryMode, setPrimaryMode] = useState("backtests");
  const activeMode = RESEARCH_MODES.find((mode) => mode.id === primaryMode) || RESEARCH_MODES[0];

  const summaryMetrics = useMemo(
    () => [
      {
        label: "回测",
        value: model.filteredBacktests.length,
        note: `已加入对比 ${model.compareSelection.length} 条。`,
        tone: model.compareSelection.length >= 2 ? "info" : "muted"
      },
      {
        label: "运行",
        value: model.filteredHistory.length,
        note: `当前账户中有 ${model.openOrders.length} 笔挂单。`,
        tone: model.openOrders.length > 0 ? "warning" : "muted"
      },
      {
        label: "事件",
        value: model.filteredEvents.length,
        note: `当前范围内共计 ${model.eventTypes.length - 1} 类事件。`,
        tone: model.filteredEvents.length > 0 ? "info" : "muted"
      },
      {
        label: "数据质量",
        value: model.dataQualitySummary?.value || "0",
        note: model.dataQualitySummary?.note || "尚未采集到数据质量明细。",
        tone: model.dataQualitySummary?.tone || "muted",
        testId: "research-data-quality-card"
      }
    ],
    [
      model.compareSelection.length,
      model.dataQualitySummary,
      model.eventTypes.length,
      model.filteredBacktests.length,
      model.filteredEvents.length,
      model.filteredHistory.length,
      model.openOrders.length
    ]
  );

  const eventTypeCounts = useMemo(() => {
    const counts = {};
    model.filteredEvents.forEach((event) => {
      const type = event.event_type || "Unknown";
      counts[type] = (counts[type] || 0) + 1;
    });
    return counts;
  }, [model.filteredEvents]);

  const maxCount = useMemo(
    () => Math.max(...Object.values(eventTypeCounts), 1),
    [eventTypeCounts]
  );

  return (
    <section
      className="event-panel event-panel--segmented research-console"
      data-testid="strategy-research-console"
    >
      <EventPanelIntro
        runtime={model.runtime}
        displayedEvents={model.displayedEvents}
        panelNotice={model.panelNotice}
        setPanelNotice={model.setPanelNotice}
        handleSaveCurrentRuntimeArtifact={model.handleSaveCurrentRuntimeArtifact}
        handleDiscardCurrentRuntimeArtifact={model.handleDiscardCurrentRuntimeArtifact}
      />

      <div className="research-console-summary" data-testid="research-summary">
        <div className="research-console-summary__main">
          <div className="panel-title strategy-card-title-note" data-testid="research-title">
            <StrategyCardNote label="研究工作区" note={RESEARCH_CONSOLE_NOTE} />
          </div>
        </div>
        <div className="research-console-summary__metrics">
          {summaryMetrics.map((metric) => (
            <ResearchMetricCard
              key={metric.label}
              label={metric.label}
              value={metric.value}
              note={metric.note}
              tone={metric.tone}
              testId={metric.testId}
            />
          ))}
        </div>
      </div>

      {Object.keys(eventTypeCounts).length > 0 ? (
        <div className="research-console-event-distribution" data-testid="research-event-distribution">
          <div className="panel-title">事件分布</div>
          {Object.entries(eventTypeCounts).map(([type, count]) => (
            <div className="event-distribution-row" key={type}>
              <span className="event-distribution-label">{type}</span>
              <div className="event-distribution-bar-container">
                <div
                  className="event-distribution-bar"
                  style={{
                    width: `${((count / maxCount) * 100).toFixed(0)}%`,
                    backgroundColor: EVENT_COLORS[type] || "var(--ad-text-muted)"
                  }}
                />
              </div>
              <span className="event-distribution-count">{count}</span>
            </div>
          ))}
        </div>
      ) : null}

      <div
        className="research-console-toolbar"
        aria-label="研究模式"
        data-testid="research-toolbar"
      >
        <div className="research-console-toolbar__tabs" data-testid="research-tabs">
          {RESEARCH_MODES.map((mode) => (
            <button
              key={mode.id}
              type="button"
              data-testid={`research-tab-${mode.id}`}
              className={`research-console-tab${
                activeMode.id === mode.id ? " research-console-tab--active" : ""
              }`}
              onClick={() => setPrimaryMode(mode.id)}
            >
              <strong>{mode.label}</strong>
              <span>{mode.note}</span>
            </button>
          ))}
        </div>
        <div className="research-console-toolbar__context" data-testid="research-context">
          <span className="status-pill info" data-testid="research-primary-mode">
            主视图：{activeMode.label}
          </span>
          <span
            className={`status-pill ${model.dataQualitySummary?.tone || "muted"}`}
            data-testid="research-data-quality-pill"
          >
            数据质量：{model.dataQualitySummary?.sourceHealthLabel || "未知"}
          </span>
          <span className="muted-line" data-testid="research-context-note">
            {primaryMode === "backtests"
              ? "在不丢失事件流的前提下，完成回测对比、查看与回放。"
              : "在保留市场与事件上下文的同时，审查模拟运行恢复情况。"}
          </span>
          <span className="muted-line" data-testid="research-data-quality-note">
            {model.dataQualitySummary?.note || "尚未采集到数据质量明细。"}
          </span>
        </div>
      </div>

      <div className="research-console-body" data-testid="research-body">
        <div className="research-console-main" data-testid="research-main-panel">
          {primaryMode === "backtests" ? (
            <StrategyBacktestsPanel
              className="research-console-panel research-console-panel-primary"
              detailMode={detailMode}
              graph={model.graph}
              runtime={model.runtime}
              selectedBacktestSummary={model.selectedBacktestSummary}
              backtestSummary={model.backtestSummary}
              backtestStartedAt={model.backtestStartedAt}
              backtestEndedAt={model.backtestEndedAt}
              backtestFilters={model.backtestFilters}
              showSummary={false}
              pagedBacktests={model.pagedBacktests}
              filteredBacktests={model.filteredBacktests}
              backtestCurrentPage={model.backtestCurrentPage}
              backtestTotalPages={model.backtestTotalPages}
              compareSelection={model.compareSelection}
              handleRefreshBacktestHistory={model.handleRefreshBacktestHistory}
              setBacktestHistoryFilter={model.setBacktestHistoryFilter}
              setBacktestCompileFilter={model.setBacktestCompileFilter}
              setBacktestDatasetFilter={model.setBacktestDatasetFilter}
              setBacktestParameterFilter={model.setBacktestParameterFilter}
              setBacktestFromTime={model.setBacktestFromTime}
              setBacktestToTime={model.setBacktestToTime}
              setBacktestPage={model.setBacktestPage}
              setBacktestPageSize={model.setBacktestPageSize}
              toggleBacktestCompareSelection={model.toggleBacktestCompareSelection}
              clearBacktestCompareSelection={model.clearBacktestCompareSelection}
              loadBacktestDetail={model.loadBacktestDetail}
              onOpenBacktestDetail={onOpenBacktestDetail}
            />
          ) : (
            <StrategyRunsPanel
              className="research-console-panel research-console-panel-primary"
              detailMode={detailMode}
              graph={model.graph}
              runtime={model.runtime}
              openOrders={model.openOrders}
              runFilters={model.runFilters}
              showAccount={false}
              pagedHistory={model.pagedHistory}
              filteredHistory={model.filteredHistory}
              currentPage={model.currentPage}
              totalPages={model.totalPages}
              handleRefreshRunHistory={model.handleRefreshRunHistory}
              setRunHistoryFilter={model.setRunHistoryFilter}
              setRunHistoryCompileFilter={model.setRunHistoryCompileFilter}
              setRunHistoryFromTime={model.setRunHistoryFromTime}
              setRunHistoryToTime={model.setRunHistoryToTime}
              setRunHistoryStatusFilter={model.setRunHistoryStatusFilter}
              setRunHistorySortOrder={model.setRunHistorySortOrder}
              setRunHistoryPage={model.setRunHistoryPage}
              setRunHistoryPageSize={model.setRunHistoryPageSize}
              loadRunDetail={model.loadRunDetail}
            />
          )}
        </div>

        <aside className="research-console-side" data-testid="research-side-panel">
          {primaryMode === "backtests" ? (
            <StrategyRunsPanel
              className="research-console-panel research-console-panel-context"
              detailMode={detailMode}
              graph={model.graph}
              runtime={model.runtime}
              openOrders={model.openOrders}
              runFilters={model.runFilters}
              showHistory={false}
              pagedHistory={model.pagedHistory}
              filteredHistory={model.filteredHistory}
              currentPage={model.currentPage}
              totalPages={model.totalPages}
              handleRefreshRunHistory={model.handleRefreshRunHistory}
              setRunHistoryFilter={model.setRunHistoryFilter}
              setRunHistoryCompileFilter={model.setRunHistoryCompileFilter}
              setRunHistoryFromTime={model.setRunHistoryFromTime}
              setRunHistoryToTime={model.setRunHistoryToTime}
              setRunHistoryStatusFilter={model.setRunHistoryStatusFilter}
              setRunHistorySortOrder={model.setRunHistorySortOrder}
              setRunHistoryPage={model.setRunHistoryPage}
              setRunHistoryPageSize={model.setRunHistoryPageSize}
              loadRunDetail={model.loadRunDetail}
            />
          ) : (
            <StrategyBacktestsPanel
              className="research-console-panel research-console-panel-context"
              detailMode={detailMode}
              graph={model.graph}
              runtime={model.runtime}
              selectedBacktestSummary={model.selectedBacktestSummary}
              backtestSummary={model.backtestSummary}
              backtestStartedAt={model.backtestStartedAt}
              backtestEndedAt={model.backtestEndedAt}
              backtestFilters={model.backtestFilters}
              showHistory={false}
              pagedBacktests={model.pagedBacktests}
              filteredBacktests={model.filteredBacktests}
              backtestCurrentPage={model.backtestCurrentPage}
              backtestTotalPages={model.backtestTotalPages}
              compareSelection={model.compareSelection}
              handleRefreshBacktestHistory={model.handleRefreshBacktestHistory}
              setBacktestHistoryFilter={model.setBacktestHistoryFilter}
              setBacktestCompileFilter={model.setBacktestCompileFilter}
              setBacktestDatasetFilter={model.setBacktestDatasetFilter}
              setBacktestParameterFilter={model.setBacktestParameterFilter}
              setBacktestFromTime={model.setBacktestFromTime}
              setBacktestToTime={model.setBacktestToTime}
              setBacktestPage={model.setBacktestPage}
              setBacktestPageSize={model.setBacktestPageSize}
              toggleBacktestCompareSelection={model.toggleBacktestCompareSelection}
              clearBacktestCompareSelection={model.clearBacktestCompareSelection}
              loadBacktestDetail={model.loadBacktestDetail}
              onOpenBacktestDetail={onOpenBacktestDetail}
            />
          )}
        </aside>

        <aside className="research-console-events-right" data-testid="research-events-sidebar">
          <StrategyEventsPanel
            className="research-console-panel research-console-panel-events"
            graph={model.graph}
            runtime={model.runtime}
            eventTypes={model.eventTypes}
            eventNodeOptions={model.eventNodeOptions}
            selectedEventNodeId={model.selectedEventNodeId}
            filteredEvents={model.filteredEvents}
            eventFilters={model.eventFilters}
            setEventNodeScope={model.setEventNodeScope}
            setEventTypeFilter={model.setEventTypeFilter}
            setEventSearchTerm={model.setEventSearchTerm}
            setSelectedNode={model.setSelectedNode}
          />
        </aside>
      </div>
    </section>
  );
}
