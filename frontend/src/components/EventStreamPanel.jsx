import { useI18n } from "../i18n";
import { backtestComparePath, navigateTo } from "../router";
import {
  buildDiagnosticsExplanationEntries,
  getEventExplanationSummary
} from "../utils/runtimeExplanation";
import { getRuntimeStatusMeta, runtimeStatusLabel } from "../utils/runtimeStatus";
import AssetCandlesPanel from "./AssetCandlesPanel";
import EventReplaySection from "./EventReplaySection";
import RuntimeMutationPanel from "./RuntimeMutationPanel";
import { useStrategyResearchModel } from "../hooks/useStrategyResearchModel";

const COPY = {
  backtestListTitle: "\u56de\u6d4b\u8bb0\u5f55",
  backtestListSubtitle:
    "\u6309\u56fe\u3001\u7f16\u8bd1\u3001\u6570\u636e\u96c6\u3001\u53c2\u6570\u548c\u65f6\u95f4\u7a97\u53e3\u7b5b\u9009\u3002",
  runListTitle: "\u8fd0\u884c\u8bb0\u5f55",
  runListSubtitle: "\u67e5\u770b\u5df2\u4fdd\u5b58\u7684\u8fd0\u884c\u7ed3\u679c\uff0c\u5e76\u6062\u590d\u6307\u5b9a\u8fd0\u884c\u8bb0\u5f55\u3002",
  filterGraph: "\u6309\u56fe ID \u7b5b\u9009",
  filterCompile: "\u6309\u7f16\u8bd1 ID \u7b5b\u9009",
  filterDataset: "\u6309\u6570\u636e\u96c6\u7b5b\u9009",
  filterParameter: "\u6309\u53c2\u6570\u7b5b\u9009",
  eventFeedSummary: "\u5b9e\u65f6\u4e8b\u4ef6\u3001\u56de\u6d4b\u8bb0\u5f55\u548c\u8282\u70b9\u8df3\u8f6c\u4fdd\u6301\u5728\u540c\u4e00\u6761\u626b\u63cf\u8def\u5f84\u4e0a\u3002",
  eventNodeAll: "\u5168\u90e8\u8282\u70b9",
  eventNodeScopeAll: "\u5f53\u524d\u663e\u793a\u5168\u90e8\u8282\u70b9\u7684\u4e8b\u4ef6\u3002",
  eventNodeScopeSelected: (nodeName) =>
    `\u5f53\u524d\u6309\u8282\u70b9 ${nodeName} \u805a\u7126\u4e8b\u4ef6\u3002`,
  eventTypeAll: "\u5168\u90e8\u4e8b\u4ef6\u7c7b\u578b",
  eventSearchPlaceholder: "\u641c\u7d22\u4e8b\u4ef6\u6458\u8981\u3001\u8282\u70b9\u3001\u8be6\u60c5"
};

const EVENT_TYPE_LABELS = {
  DataUpdated: "\u6570\u636e\u66f4\u65b0",
  IntentTriggered: "\u610f\u56fe\u89e6\u53d1",
  IntentEvaluated: "\u610f\u56fe\u8bc4\u4f30",
  AgentDecisionProduced: "\u4ee3\u7406\u51b3\u7b56",
  RiskDecisionProduced: "\u98ce\u63a7\u51b3\u7b56",
  ExecutionFilled: "\u6210\u4ea4\u56de\u62a5",
  ExecutionPlanned: "\u6267\u884c\u8ba1\u5212",
  PortfolioUpdated: "\u7ec4\u5408\u66f4\u65b0",
  RuntimeNotice: "\u8fd0\u884c\u63d0\u793a",
  RuntimeWarning: "\u6570\u636e\u544a\u8b66",
  RuntimeError: "\u6570\u636e\u9519\u8bef"
};

const SEMANTIC_LABELS = {
  Info: "\u4fe1\u606f",
  Warn: "\u544a\u8b66",
  Warning: "\u544a\u8b66",
  Error: "\u9519\u8bef",
  error: "\u9519\u8bef",
  Open: "\u6302\u5355\u4e2d",
  open: "\u6302\u5355\u4e2d",
  Filled: "\u5df2\u6210\u4ea4",
  filled: "\u5df2\u6210\u4ea4",
  started: "\u5df2\u542f\u52a8",
  queued: "\u6392\u961f\u4e2d",
  running: "\u8fd0\u884c\u4e2d",
  completed: "\u5df2\u5b8c\u6210",
  idle: "\u7a7a\u95f2",
  Healthy: "\u6b63\u5e38",
  healthy: "\u6b63\u5e38",
  Delayed: "\u5ef6\u8fdf",
  delayed: "\u5ef6\u8fdf",
  Stale: "\u9648\u65e7",
  stale: "\u9648\u65e7",
  Missing: "\u7f3a\u5931",
  missing: "\u7f3a\u5931",
  unsupported: "\u4e0d\u652f\u6301"
};

const DIRECTION_LABELS = {
  Buy: "\u4e70\u5165",
  buy: "\u4e70\u5165",
  Sell: "\u5356\u51fa",
  sell: "\u5356\u51fa",
  Long: "\u505a\u591a",
  long: "\u505a\u591a",
  Short: "\u505a\u7a7a",
  short: "\u505a\u7a7a"
};

const HISTORY_COPY = {
  refresh: "刷新",
  clear: "清空",
  reset: "重置",
  currentGraph: "当前图",
  clearCompare: "清空对比",
  pageSize6: "每页 6 条",
  pageSize10: "每页 10 条",
  pageSize20: "每页 20 条",
  previousPage: "上一页",
  nextPage: "下一页",
  resultCount: (count) => `结果数：${count}`,
  pageLabel: (currentPage, totalPages) => `第 ${currentPage} / ${totalPages} 页`,
  compareSelection: (count) => `已选对比：${count}/2`
};

function formatValue(value) {
  if (value === null || value === undefined || value === "") return "-";
  if (typeof value === "number") {
    return Number.isInteger(value) ? String(value) : value.toFixed(4);
  }
  return String(value);
}

function formatRatio(value) {
  if (!Number.isFinite(value)) return "-";
  const percent = value * 100;
  const sign = percent > 0 ? "+" : "";
  return `${sign}${percent.toFixed(2)}%`;
}

function backtestExecutionAssumptionsLabel(filters) {
  if (filters?.execution_assumptions_tag?.label) {
    const tag = filters.execution_assumptions_tag;
    return tag.sources_label ? `${tag.label} (${tag.sources_label})` : tag.label;
  }
  return "-";
}

function eventBadge(event) {
  return event.payload?.exec_status || event.payload?.status || event.severity || "Info";
}

function orderSideClass(side) {
  return side === "Sell" ? "sell" : "buy";
}

function severityText(value) {
  const labels = {
    Info: "信息",
    Warn: "告警",
    Error: "错误",
    Open: "挂单中",
    Filled: "已成交"
  };
  return labels[value] || value;
}

function semanticText(value) {
  return SEMANTIC_LABELS[value] || severityText(value);
}

function directionText(value) {
  return DIRECTION_LABELS[value] || value;
}

function qualityFlagsText(value) {
  if (!Array.isArray(value) || value.length === 0) return null;
  return value.join(", ");
}

function resolveRunStatus(run, runtime) {
  if (runtime.runId === run.run_id && runtime.status && runtime.status !== "idle") {
    return runtime.status;
  }
  return run.status || "completed";
}

function ratioTone(value) {
  if (!Number.isFinite(value) || value === 0) return "muted";
  return value > 0 ? "success" : "danger";
}

function drawdownTone(value) {
  if (!Number.isFinite(value) || value === 0) return "muted";
  return Math.abs(value) >= 0.1 ? "danger" : "warning";
}

function runtimeTone(status) {
  return getRuntimeStatusMeta(status).tone;
}

function eventTypeLabel(type) {
  return EVENT_TYPE_LABELS[type] || type;
}

function EventSection({ kicker, title, summary, className = "", testId = null, children }) {
  return (
    <section
      className={`event-section ${className}`.trim()}
      aria-label={title}
      data-testid={testId || undefined}
    >
      <div className="event-section-header">
        {kicker ? <div className="event-section-kicker">{kicker}</div> : null}
        <div className="event-section-title">{title}</div>
        {summary ? <div className="event-section-summary">{summary}</div> : null}
      </div>
      <div className="event-section-body">{children}</div>
    </section>
  );
}

function SummaryPill({ label, value, tone = "muted" }) {
  return (
    <div className={`event-summary-pill event-summary-pill-${tone}`}>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function HistoryMetaGrid({ items, className = "" }) {
  return (
    <div className={`history-meta-grid ${className}`.trim()}>
      {items.map((item) => (
        <div
          key={item.label}
          className={`history-meta-chip history-meta-chip-${item.tone || "muted"} ${
            item.wide ? "history-meta-chip-wide" : ""
          }`.trim()}
        >
          <span>{item.label}</span>
          <strong>{item.value}</strong>
        </div>
      ))}
    </div>
  );
}

function HistoryCardHeader({ id, timestamp }) {
  return (
    <div className="run-history-topline">
      <strong title={id}>{id}</strong>
      <span className="history-time-pill" title={timestamp}>
        {timestamp}
      </span>
    </div>
  );
}

function HistoryNotice({ children, tone = "muted" }) {
  return <div className={`muted-line history-note history-note-${tone}`}>{children}</div>;
}

function SectionCardHeader({ title, summary, action = null, value = null }) {
  return (
    <div className="open-orders-header">
      <div>
        <div className="mini-list-title">{title}</div>
        {summary ? <div className="muted-line">{summary}</div> : null}
      </div>
      {value !== null ? <strong>{value}</strong> : action}
    </div>
  );
}

function HistoryCopyBlock({ title, subtitle }) {
  return (
    <div className="history-copy-block">
      <div className="mini-list-title history-copy-title">{title}</div>
      <div className="muted-line history-copy-subtitle">{subtitle}</div>
    </div>
  );
}

function HistoryFilterGrid({ className = "", fields }) {
  return (
    <div className={`history-filter-grid ${className}`.trim()}>
      {fields.map((field) => {
        if (field.type === "select") {
          return (
            <select
              key={field.key}
              aria-label={field.ariaLabel}
              className={field.className || "history-filter-input history-filter-select"}
              value={field.value}
              onChange={(event) => field.onChange(event.target.value)}
            >
              {field.options.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          );
        }

        return (
          <input
            key={field.key}
            aria-label={field.ariaLabel}
            className={field.className || "history-filter-input"}
            type={field.type || "text"}
            value={field.value}
            placeholder={field.placeholder}
            onChange={(event) => field.onChange(event.target.value)}
          />
        );
      })}
    </div>
  );
}

function HistoryMetaRow({ items }) {
  const visibleItems = items.filter((item) => item && item.value !== undefined && item.value !== null);
  if (visibleItems.length === 0) return null;

  return (
    <div className="run-history-meta">
      {visibleItems.map((item) => (
        <span key={item.label}>
          {item.label}：{item.value}
        </span>
      ))}
    </div>
  );
}

function HistoryPagination({ currentPage, totalPages, onPrevious, onNext }) {
  return (
    <div className="history-pagination">
      <button className="ghost-btn compact-btn" disabled={currentPage <= 1} onClick={onPrevious}>
        {HISTORY_COPY.previousPage}
      </button>
      <span>{HISTORY_COPY.pageLabel(currentPage, totalPages)}</span>
      <button className="ghost-btn compact-btn" disabled={currentPage >= totalPages} onClick={onNext}>
        {HISTORY_COPY.nextPage}
      </button>
    </div>
  );
}

function HistoryControlBar({
  className = "",
  refreshAriaLabel,
  refreshDisabled = false,
  onRefresh,
  pageSize,
  onPageSizeChange,
  summary,
  actions = null
}) {
  return (
    <div className={`history-filter-row history-control-bar ${className}`.trim()}>
      <button
        className="ghost-btn compact-btn"
        aria-label={refreshAriaLabel}
        disabled={refreshDisabled}
        onClick={onRefresh}
      >
        {HISTORY_COPY.refresh}
      </button>
      <select
        className="history-filter-input history-filter-select history-page-size-select"
        value={String(pageSize ?? 6)}
        onChange={(event) => onPageSizeChange(Number(event.target.value))}
      >
        <option value="6">{HISTORY_COPY.pageSize6}</option>
        <option value="10">{HISTORY_COPY.pageSize10}</option>
        <option value="20">{HISTORY_COPY.pageSize20}</option>
      </select>
      <span className="history-inline-note">{summary}</span>
      {actions}
    </div>
  );
}

function HistoryExplanationCard({ title, summary, entries, testId }) {
  if (!entries || entries.length === 0) return null;

  return (
    <div className="open-orders-card" data-testid={testId}>
      <SectionCardHeader title={title} summary={summary} value={entries.length} />
      {entries.map((entry) => (
        <div key={entry.nodeId} className="open-order-item">
          <div className="open-order-topline">
            <strong>{entry.nodeName}</strong>
            <span>{entry.nodeId}</span>
          </div>
          {entry.explanationSummary ? <div className="muted-line">{entry.explanationSummary}</div> : null}
          <div className="open-order-grid">
            {entry.rows.map((row) => (
              <div key={`${entry.nodeId}_${row.key}`}>
                <span>{row.label}</span>
                <strong>{row.value}</strong>
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

export function EventPanelIntro({
  runtime,
  displayedEvents,
  panelNotice,
  setPanelNotice,
  handleSaveCurrentRuntimeArtifact,
  handleDiscardCurrentRuntimeArtifact
}) {
  const { t } = useI18n();
  const canSaveCurrentArtifact =
    runtime.artifactPersistenceStatus === "transient" &&
    runtime.status === "completed" &&
    Boolean(runtime.runId);
  const runKindLabel =
    runtime.runKind === "backtest"
      ? runtime.artifactPersistenceStatus === "transient"
        ? "回测预览"
        : "历史回测"
      : "模拟运行";
  return (
    <div className="event-panel-header" data-testid="event-panel-intro">
      <div className="event-panel-intro">
        <div className="panel-title">运行与回测面板</div>
        <div className="panel-subtitle">
          把运行摘要、事件流、历史记录和账户状态放在同一视图里，同时保持清晰分层。
        </div>
      </div>
      <div className="event-summary-grid">
        <SummaryPill label="状态" value={runtimeStatusLabel(runtime.status)} />
        <SummaryPill label="类型" value={runKindLabel} />
        <SummaryPill label="运行 ID" value={runtime.runId || "-"} />
        <SummaryPill label="事件数" value={displayedEvents.length} />
      </div>
      {canSaveCurrentArtifact ? (
        <div className="event-panel-actions">
          <button
            type="button"
            className="ghost-btn compact-btn"
            data-testid="runtime-artifact-save"
            onClick={() => handleSaveCurrentRuntimeArtifact?.()}
          >
            保存本次结果
          </button>
          <button
            type="button"
            className="ghost-btn compact-btn"
            data-testid="runtime-artifact-discard"
            onClick={() => handleDiscardCurrentRuntimeArtifact?.()}
          >
            丢弃临时结果
          </button>
          <span className="muted-line">未保存结果仅保留在当前会话。</span>
        </div>
      ) : null}
      {runtime.backendError && !(panelNotice && panelNotice.type === "error") ? (
        <div
          className="toolbar-notice panel-feedback panel-feedback-error toolbar-notice-error"
          role="alert"
          data-testid="event-panel-backend-error"
        >
          <span>{runtime.backendError}</span>
        </div>
      ) : null}
      {panelNotice ? (
        <div
          className={`toolbar-notice panel-feedback panel-feedback-${panelNotice.type} toolbar-notice-${panelNotice.type}`}
          role={panelNotice.type === "error" ? "alert" : "status"}
          data-testid="event-panel-notice"
        >
          <span>{panelNotice.message}</span>
          <button
            type="button"
            className="toolbar-notice-close"
            onClick={() => setPanelNotice(null)}
          >
            {t("关闭")}
          </button>
        </div>
      ) : null}
    </div>
  );
}

export function EventFeedSection({
  runtime,
  eventTypes,
  eventNodeOptions = [],
  selectedEventNodeId = null,
  filteredEvents,
  eventTypeFilter,
  eventSearchTerm,
  setEventNodeScope,
  setEventTypeFilter,
  setEventSearchTerm,
  setSelectedNode
}) {
  return (
    <EventSection
      kicker="事件"
      title="事件流"
      summary={COPY.eventFeedSummary}
      className="event-feed-section"
      testId="event-feed-section"
    >
      <div className="event-list">
        {eventNodeOptions.length > 0 ? (
          <div
            className="strategy-inspector-actions event-node-filter-bar"
            data-testid="event-feed-node-filter"
          >
            <button
              type="button"
              className={`ghost-btn compact-btn${selectedEventNodeId ? "" : " is-active"}`}
              data-testid="event-feed-node-chip-all"
              aria-pressed={!selectedEventNodeId}
              onClick={() => {
                setEventNodeScope("all");
                setSelectedNode(null);
              }}
            >
              {COPY.eventNodeAll}
            </button>
            {eventNodeOptions.map((node) => (
              <button
                key={node.nodeId}
                type="button"
                className={`ghost-btn compact-btn${
                  node.nodeId === selectedEventNodeId ? " is-active" : ""
                }`}
                data-testid={`event-feed-node-chip-${node.nodeId}`}
                aria-pressed={node.nodeId === selectedEventNodeId}
                onClick={() => {
                  setEventNodeScope("auto");
                  setSelectedNode(node.nodeId);
                }}
              >
                {node.nodeName}
              </button>
            ))}
          </div>
        ) : null}
        <div className="history-inline-note" data-testid="event-feed-node-scope">
          {selectedEventNodeId
            ? COPY.eventNodeScopeSelected(
                eventNodeOptions.find((node) => node.nodeId === selectedEventNodeId)?.nodeName ||
                  selectedEventNodeId
              )
            : COPY.eventNodeScopeAll}
        </div>
        <div className="event-filter-bar">
          <select
            className="history-filter-input history-filter-select history-page-size-select"
            value={eventTypeFilter ?? runtime.eventTypeFilter ?? "all"}
            onChange={(event) => setEventTypeFilter(event.target.value)}
          >
            {eventTypes.map((type) => (
              <option key={type} value={type}>
                {type === "all" ? COPY.eventTypeAll : eventTypeLabel(type)}
              </option>
            ))}
          </select>
          <input
            className="history-filter-input"
            value={eventSearchTerm ?? runtime.eventSearchTerm ?? ""}
            placeholder={COPY.eventSearchPlaceholder}
            onChange={(event) => setEventSearchTerm(event.target.value)}
          />
          <button
            className="ghost-btn compact-btn"
            onClick={() => {
              setEventTypeFilter("all");
              setEventSearchTerm("");
            }}
          >
            {HISTORY_COPY.clear}
          </button>
        </div>
        {filteredEvents.length === 0 ? (
          <div className="empty-state">当前筛选条件下没有事件。</div>
        ) : null}
        {filteredEvents.map((event) => (
          <button
            key={event.event_id}
            className="event-row"
            data-testid={`event-feed-row-${event.event_id}`}
            onClick={() => {
              if (event.node_id) {
                setEventNodeScope("auto");
                setSelectedNode(event.node_id);
              }
            }}
          >
            <div className="event-time">{new Date(event.event_time_ms).toLocaleTimeString()}</div>
            <div className="event-type">{eventTypeLabel(event.event_type)}</div>
            <div className="event-details">
              <div className="event-message">{event.summary}</div>
              {getEventExplanationSummary(event) ? (
                <div
                  className="muted-line"
                  data-testid={`event-feed-explanation-${event.event_id}`}
                >
                  {getEventExplanationSummary(event)}
                </div>
              ) : null}
              <div className="event-meta-row">
                <span>状态：{semanticText(eventBadge(event))}</span>
                {event.payload?.limit_triggered ? (
                  <span>限制：{formatValue(event.payload.limit_triggered)}</span>
                ) : null}
                {event.payload?.post_risk?.concentration_ratio !== undefined ? (
                  <span data-testid={`event-feed-post-concentration-${event.event_id}`}>
                    集中度：{formatRatio(event.payload.post_risk.concentration_ratio)}
                  </span>
                ) : null}
                {event.payload?.post_risk?.max_symbol_net_exposure_ratio !== undefined ? (
                  <span data-testid={`event-feed-post-symbol-net-${event.event_id}`}>
                    单标的净敞口：{formatRatio(
                      event.payload.post_risk.max_symbol_net_exposure_ratio
                    )}
                  </span>
                ) : null}
                {event.payload?.post_risk?.portfolio_net_exposure_ratio !== undefined ? (
                  <span data-testid={`event-feed-post-portfolio-net-${event.event_id}`}>
                    组合净敞口：{formatRatio(
                      event.payload.post_risk.portfolio_net_exposure_ratio
                    )}
                  </span>
                ) : null}
                {event.payload?.sizing_source ? (
                  <span>定量来源：{formatValue(event.payload.sizing_source)}</span>
                ) : null}
                {event.payload?.order_type_decision_reason ? (
                  <span>下单语义：{formatValue(event.payload.order_type_decision_reason)}</span>
                ) : null}
                {event.payload?.lifecycle_stage ? (
                  <span>生命周期：{formatValue(event.payload.lifecycle_stage)}</span>
                ) : null}
                {event.payload?.side ? <span>方向：{directionText(event.payload.side)}</span> : null}
                {event.payload?.signal_direction ? (
                  <span>信号方向：{directionText(event.payload.signal_direction)}</span>
                ) : null}
                {event.payload?.source_status ? (
                  <span>数据状态：{semanticText(event.payload.source_status)}</span>
                ) : null}
                {event.payload?.source_health ? (
                  <span data-testid={`event-feed-source-health-${event.event_id}`}>
                    来源健康：{semanticText(event.payload.source_health)}
                  </span>
                ) : null}
                {event.payload?.freshness_ms !== undefined ? (
                  <span data-testid={`event-feed-freshness-${event.event_id}`}>
                    新鲜度：{formatValue(event.payload.freshness_ms)} ms
                  </span>
                ) : null}
                {event.payload?.gap_count !== undefined ? (
                  <span data-testid={`event-feed-gap-count-${event.event_id}`}>
                    缺口：{formatValue(event.payload.gap_count)}
                  </span>
                ) : null}
                {qualityFlagsText(event.payload?.quality_flags) ? (
                  <span data-testid={`event-feed-quality-flags-${event.event_id}`}>
                    质量标记：{qualityFlagsText(event.payload.quality_flags)}
                  </span>
                ) : null}
                {event.payload?.order_id ? <span>订单：{event.payload.order_id}</span> : null}
                {event.payload?.remaining_qty !== undefined ? (
                  <span>剩余数量：{formatValue(event.payload.remaining_qty)}</span>
                ) : null}
                {event.payload?.price !== undefined ? (
                  <span>价格：{formatValue(event.payload.price)}</span>
                ) : null}
                {event.payload?.limit_price !== undefined && event.payload?.limit_price !== null ? (
                  <span>限价：{formatValue(event.payload.limit_price)}</span>
                ) : null}
                {event.payload?.reserved_cash !== undefined ? (
                  <span>冻结现金：{formatValue(event.payload.reserved_cash)}</span>
                ) : null}
                {event.payload?.reserved_qty !== undefined ? (
                  <span>冻结仓位：{formatValue(event.payload.reserved_qty)}</span>
                ) : null}
                {event.payload?.fee_paid !== undefined ? (
                  <span>手续费：{formatValue(event.payload.fee_paid)}</span>
                ) : null}
              </div>
            </div>
          </button>
        ))}
      </div>
    </EventSection>
  );
}

export function BacktestSummarySection({
  runtime,
  selectedBacktestSummary,
  backtestSummary,
  backtestStartedAt,
  backtestEndedAt
}) {
  const { t } = useI18n();
  if (!runtime.backtestArtifacts) return null;

  return (
    <EventSection
      kicker="回测"
      className="event-sidebar-section event-sidebar-section-summary"
      title="回测分析"
      summary={t("优先显示工件驱动的回测结果摘要，而不是旧式摘要拼接。")}
    >
      <div className="backtest-summary-card" data-testid="backtest-summary-card">
        <SectionCardHeader
          title="回测结果详情"
          summary="回测工件和模拟运行分开持久化，详情会恢复权益曲线与事件日志。"
          value={`${backtestSummary?.step_count || 0} 步`}
        />
        <div className="kv-line">
          <span>回测 ID</span>
          <strong data-testid="backtest-summary-id">
            {runtime.selectedBacktestId || runtime.runId || "-"}
          </strong>
        </div>
        <div className="kv-line">
          <span>协议</span>
          <strong data-testid="backtest-summary-protocol">
            {selectedBacktestSummary?.protocol_name || "-"}
          </strong>
        </div>
        <div className="kv-line">
          <span>配置哈希</span>
          <strong data-testid="backtest-summary-config-hash">
            {selectedBacktestSummary?.config_hash || "-"}
          </strong>
        </div>
        <div className="account-metric-grid" data-testid="backtest-summary-metrics">
          <div className="account-metric-card" data-testid="backtest-summary-total-return">
            <span>总收益率</span>
            <strong>{formatRatio(backtestSummary?.total_return_ratio)}</strong>
          </div>
          <div className="account-metric-card" data-testid="backtest-summary-max-drawdown">
            <span>最大回撤</span>
            <strong>{formatRatio(backtestSummary?.max_drawdown_ratio)}</strong>
          </div>
          <div className="account-metric-card">
            <span>成交次数</span>
            <strong>{formatValue(backtestSummary?.trade_count)}</strong>
          </div>
          <div className="account-metric-card" data-testid="backtest-summary-trade-count">
            <span>最终权益</span>
            <strong>{formatValue(backtestSummary?.final_equity)}</strong>
          </div>
        </div>
        <div className="kv-line">
          <span>开始时间</span>
          <strong data-testid="backtest-summary-started-at">
            {backtestStartedAt ? new Date(backtestStartedAt).toLocaleString() : "-"}
          </strong>
        </div>
        <div className="kv-line">
          <span>结束时间</span>
          <strong data-testid="backtest-summary-ended-at">
            {backtestEndedAt ? new Date(backtestEndedAt).toLocaleString() : "-"}
          </strong>
        </div>
      </div>
    </EventSection>
  );
}

export function BacktestHistorySection({
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
      kicker="历史"
      className="event-sidebar-section event-sidebar-section-backtest"
      title="回测历史"
      summary="把筛选、对比选择和详情入口统一放在同一块分析区域。"
    >
      <div className="backtest-history-card" data-testid="backtest-history-card">
        <SectionCardHeader
          title="持久化回测记录"
          summary="按图、编译、数据集、参数和时间窗口筛选历史结果。"
          action={
            <button
              className="ghost-btn compact-btn"
              aria-label="刷新回测历史"
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
              title="已选回测风控解释"
              summary={`当前已加载回测 ${runtime.selectedBacktestId} 的风控说明。`}
              entries={selectedBacktestRiskEntries}
              testId="backtest-history-risk-explanations"
            />
            <HistoryExplanationCard
              title="已选回测订单解释"
              summary={`当前已加载回测 ${runtime.selectedBacktestId} 的订单说明。`}
              entries={selectedBacktestOrderEntries}
              testId="backtest-history-order-explanations"
            />
            <HistoryExplanationCard
              title="已选回测数据质量"
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
              placeholder: "按图 ID 过滤",
              onChange: setBacktestHistoryFilter
            },
            {
              key: "compile",
              value: backtestCompileFilter ?? runtime.backtestCompileFilter ?? "",
              placeholder: "按编译 ID 过滤",
              onChange: setBacktestCompileFilter
            },
            {
              key: "dataset",
              value: backtestDatasetFilter ?? runtime.backtestDatasetFilter ?? "",
              placeholder: "按数据集过滤",
              onChange: setBacktestDatasetFilter
            },
            {
              key: "parameter",
              value: backtestParameterFilter ?? runtime.backtestParameterFilter ?? "",
              placeholder: "按参数过滤",
              onChange: setBacktestParameterFilter
            },
            {
              key: "fromTime",
              type: "datetime-local",
              ariaLabel: "回测开始时间过滤",
              className: "history-filter-input history-filter-time",
              value: backtestFromTime ?? runtime.backtestFromTime ?? "",
              onChange: setBacktestFromTime
            },
            {
              key: "toTime",
              type: "datetime-local",
              ariaLabel: "回测结束时间过滤",
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
            打开对比 ({compareSelection.length}/2)
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
          <HistoryNotice>正在加载回测历史...</HistoryNotice>
        ) : null}
        {filteredBacktests.length === 0 && runtime.backtestHistoryStatus !== "loading" ? (
          <HistoryNotice>当前过滤条件下没有回测记录。</HistoryNotice>
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
                      { label: "\u56fe", value: item.graph_id || "-" },
                      { label: "\u7f16\u8bd1", value: item.compile_id || "-" },
                      {
                        label: "\u6536\u76ca",
                        value: formatRatio(item.summary?.total_return_ratio),
                        tone: ratioTone(item.summary?.total_return_ratio)
                      },
                      {
                        label: "\u56de\u64a4",
                        value: formatRatio(item.summary?.max_drawdown_ratio),
                        tone: drawdownTone(item.summary?.max_drawdown_ratio)
                      },
                      {
                        label: "\u65f6\u95f4\u7a97",
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
                      { label: "\u6210\u4ea4", value: formatValue(item.summary?.trade_count) },
                      {
                        label: "\u6570\u636e\u96c6",
                        value: (item.filters?.dataset_labels || []).join(", ") || "-",
                        wide: true
                      },
                      {
                        label: "\u53c2\u6570",
                        value: backtestExecutionAssumptionsLabel(item.filters),
                        wide: true
                      },
                      { label: "\u56de\u653e", value: item.filters?.replay_source || "-" }
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
                    {isCompareSelected ? "取消对比" : "加入对比"}
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

export function RunHistorySection({
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
      kicker="历史"
      className="event-sidebar-section event-sidebar-section-run"
      title="运行历史"
      summary="聚焦模拟运行记录，保留状态过滤、时间范围和详情恢复。"
    >
      <div className="run-history-card" data-testid="run-history-card">
        <SectionCardHeader
          title="持久化运行记录"
          summary={t("查看后端已保存的运行结果，并恢复指定运行详情。")}
          action={
            <button
              className="ghost-btn compact-btn"
              aria-label="刷新运行记录"
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
              title="已选运行风控解释"
              summary={`当前已加载运行 ${runtime.runId} 的风控说明。`}
              entries={selectedRunRiskEntries}
              testId="run-history-risk-explanations"
            />
            <HistoryExplanationCard
              title="已选运行订单解释"
              summary={`当前已加载运行 ${runtime.runId} 的订单说明。`}
              entries={selectedRunOrderEntries}
              testId="run-history-order-explanations"
            />
            <HistoryExplanationCard
              title="已选运行数据质量"
              summary={`Data quality details for loaded run ${runtime.runId}.`}
              entries={selectedRunDataQualityEntries}
              testId="run-history-data-quality"
            />
            <RuntimeMutationPanel
              sourceKind="run"
              sourceId={runtime.runId}
              capabilityContext={runtime.governance ? { schema_hash: runtime.governance.capability_hash } : null}
              initialMutations={runtime.parameterMutations || []}
              title="参数变更"
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
              placeholder: "按图 ID 过滤",
              onChange: setRunHistoryFilter
            },
            {
              key: "compile",
              value: historyCompileFilter ?? runtime.historyCompileFilter ?? "",
              placeholder: "按编译 ID 过滤",
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
                { value: "all", label: "全部状态" },
                { value: "completed", label: "已完成" },
                { value: "running", label: "运行中" },
                { value: "connecting", label: "连接中" },
                 { value: "error", label: "错误" },
                { value: "stopped", label: "已停止" }
              ]
            },
            {
              key: "sortOrder",
              type: "select",
              value: historySortOrder ?? runtime.historySortOrder ?? "desc",
              onChange: setRunHistorySortOrder,
              options: [
                 { value: "desc", label: "时间倒序" },
                 { value: "asc", label: "时间正序" }
              ]
            }
          ]}
        />
        <HistoryControlBar
          className="history-control-bar-run"
          refreshAriaLabel="刷新运行记录"
          refreshDisabled={runtime.historyStatus === "loading"}
          onRefresh={() => handleRefreshRunHistory()}
          pageSize={historyPageSize ?? runtime.historyPageSize ?? 6}
          onPageSizeChange={setRunHistoryPageSize}
          summary={HISTORY_COPY.resultCount(filteredHistory.length)}
          actions={
            <>
              <button
                className="ghost-btn compact-btn"
                onClick={() => setRunHistoryFilter(graph.metadata?.graph_id || "")}
              >
                {HISTORY_COPY.currentGraph}
              </button>
              <button
                className="ghost-btn compact-btn"
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
          <HistoryNotice>正在加载运行记录...</HistoryNotice>
        ) : null}
        {filteredHistory.length === 0 && runtime.historyStatus !== "loading" ? (
          <HistoryNotice>当前过滤条件下没有运行记录。</HistoryNotice>
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
                      { label: "\u56fe", value: run.graph_id || "-" },
                      {
                        label: "\u72b6\u6001",
                        value: runtimeStatusLabel(effectiveStatus),
                        tone: runtimeTone(effectiveStatus)
                      },
                      { label: "\u7f16\u8bd1", value: run.compile_id || "-" },
                    { label: "\u4e8b\u4ef6", value: formatValue(run.event_count) }
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

export function AccountSection({ runtime, openOrders }) {
  return (
    <EventSection
      kicker="账户"
      className="event-sidebar-section event-sidebar-section-account"
      title="账户与挂单"
      summary="把账户摘要和当前挂单放在同一块，减少来回切换。"
    >
      <div className="open-orders-card">
        <SectionCardHeader title="账户摘要" summary="现金、净值、杠杆和名义价值统一展示。" />
        <div className="account-metric-grid">
          <div className="account-metric-card" data-testid="account-summary-equity">
            <span>总资产估值</span>
            <strong>{formatValue(runtime.account?.equity_estimate ?? runtime.account?.cash_balance)}</strong>
          </div>
          <div className="account-metric-card">
            <span>总现金</span>
            <strong>{formatValue(runtime.account?.cash_balance)}</strong>
          </div>
          <div className="account-metric-card">
            <span>可用现金</span>
            <strong>{formatValue(runtime.account?.available_cash_balance)}</strong>
          </div>
          <div className="account-metric-card">
            <span>冻结现金</span>
            <strong>{formatValue(runtime.account?.frozen_cash_balance)}</strong>
          </div>
          <div className="account-metric-card">
            <span>总杠杆</span>
            <strong>{formatValue(runtime.account?.total_leverage ?? runtime.account?.pnl)}</strong>
          </div>
          <div className="account-metric-card">
            <span>持仓数</span>
            <strong>{formatValue(runtime.account?.positions)}</strong>
          </div>
        </div>
        <div className="kv-line">
          <span>总名义价值</span>
          <strong>{formatValue(runtime.account?.total_gross_notional)}</strong>
        </div>
        <div className="kv-line">
          <span>净名义价值</span>
          <strong>{formatValue(runtime.account?.total_net_notional)}</strong>
        </div>
      </div>
      <div className="open-orders-card">
        <SectionCardHeader
          title="当前挂单"
          summary="买单主要冻结现金，卖单主要冻结仓位。"
          value={formatValue(runtime.account?.open_order_count)}
        />
        {openOrders.length === 0 ? <div className="muted-line">当前没有挂单。</div> : null}
        {openOrders.map((order) => (
          <div key={order.order_id} className="open-order-item">
            <div className="open-order-topline">
              <span className={`side-pill ${orderSideClass(order.side)}`}>
                {order.side === "Sell" ? "卖出" : "买入"}
              </span>
              <strong>{order.order_id}</strong>
            </div>
            <div className="open-order-grid">
              <div>
                <span>剩余数量</span>
                <strong>{formatValue(order.remaining_qty)}</strong>
              </div>
              <div>
                <span>限价</span>
                <strong>{formatValue(order.limit_price)}</strong>
              </div>
              <div>
                <span>冻结现金</span>
                <strong>{formatValue(order.reserved_cash)}</strong>
              </div>
              <div>
                <span>冻结仓位</span>
                <strong>{formatValue(order.reserved_qty)}</strong>
              </div>
            </div>
          </div>
        ))}
      </div>
    </EventSection>
  );
}


export default function EventStreamPanel({ detailMode = false, onOpenBacktestDetail = null }) {
  const model = useStrategyResearchModel();

  return (
    <section className={`event-panel${detailMode ? " event-panel-detail" : ""}`}>
      <EventPanelIntro
        runtime={model.runtime}
        displayedEvents={model.displayedEvents}
        panelNotice={model.panelNotice}
        setPanelNotice={model.setPanelNotice}
        handleSaveCurrentRuntimeArtifact={model.handleSaveCurrentRuntimeArtifact}
        handleDiscardCurrentRuntimeArtifact={model.handleDiscardCurrentRuntimeArtifact}
      />

      <div className="event-panel-body">
        <div className="event-main-column">
          <AssetCandlesPanel graph={model.graph} runtime={model.runtime} />

          <EventFeedSection
            runtime={model.runtime}
            eventTypes={model.eventTypes}
            eventNodeOptions={model.eventNodeOptions}
            selectedEventNodeId={model.selectedEventNodeId}
            filteredEvents={model.filteredEvents}
            eventTypeFilter={model.eventFilters.eventTypeFilter}
            eventSearchTerm={model.eventFilters.eventSearchTerm}
            setEventNodeScope={model.setEventNodeScope}
            setEventTypeFilter={model.setEventTypeFilter}
            setEventSearchTerm={model.setEventSearchTerm}
            setSelectedNode={model.setSelectedNode}
          />
        </div>

        <div className="event-sidebar">
          <div className="event-sidebar-overview">
            <BacktestSummarySection
              runtime={model.runtime}
              selectedBacktestSummary={model.selectedBacktestSummary}
              backtestSummary={model.backtestSummary}
              backtestStartedAt={model.backtestStartedAt}
              backtestEndedAt={model.backtestEndedAt}
            />
            <AccountSection runtime={model.runtime} openOrders={model.openOrders} />
            <EventReplaySection runtime={model.runtime} />
          </div>

          <div className="event-sidebar-history">
            <BacktestHistorySection
              detailMode={detailMode}
              graph={model.graph}
              runtime={model.runtime}
              backtestHistoryFilter={model.backtestFilters.backtestHistoryFilter}
              backtestCompileFilter={model.backtestFilters.backtestCompileFilter}
              backtestDatasetFilter={model.backtestFilters.backtestDatasetFilter}
              backtestParameterFilter={model.backtestFilters.backtestParameterFilter}
              backtestFromTime={model.backtestFilters.backtestFromTime}
              backtestToTime={model.backtestFilters.backtestToTime}
              backtestPageSize={model.backtestFilters.backtestPageSize}
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
            <RunHistorySection
              detailMode={detailMode}
              graph={model.graph}
              runtime={model.runtime}
              historyFilter={model.runFilters.historyFilter}
              historyCompileFilter={model.runFilters.historyCompileFilter}
              historyFromTime={model.runFilters.historyFromTime}
              historyToTime={model.runFilters.historyToTime}
              historyStatusFilter={model.runFilters.historyStatusFilter}
              historySortOrder={model.runFilters.historySortOrder}
              historyPageSize={model.runFilters.historyPageSize}
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
          </div>
        </div>
      </div>
    </section>
  );
}
