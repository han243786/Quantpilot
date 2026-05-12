import { useState } from "react";
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

export const COPY = {
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

export const HISTORY_COPY = {
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

function CompareDiffTable({ compareDiff }) {
  const { t } = useI18n();
  if (!compareDiff || typeof compareDiff !== "object") return null;
  const metrics = Object.keys(compareDiff);
  if (metrics.length === 0) return null;

  const formatNum = (v) => {
    if (v === null || v === undefined || !Number.isFinite(v)) return "-";
    const abs = Math.abs(v);
    let decimals = 2;
    if (abs > 0 && abs < 0.01) decimals = 4;
    else if (abs < 1) decimals = 4;
    else if (abs < 100) decimals = 2;
    else decimals = 2;
    return v.toFixed(decimals);
  };

  const formatDiff = (diff) => {
    if (diff === null || diff === undefined || !Number.isFinite(diff)) return "-";
    const sign = diff > 0 ? "+" : "";
    return `${sign}${formatNum(diff)}`;
  };

  return (
    <>
      <style>{`
        .compare-table {
          width: 100%;
          border-collapse: collapse;
          margin-top: 6px;
          font-size: 12px;
          font-family: inherit;
        }
        .compare-table th,
        .compare-table td {
          padding: 3px 8px;
          text-align: right;
          border: 1px solid rgba(255,255,255,0.12);
        }
        .compare-table th {
          background: rgba(255,255,255,0.06);
          font-weight: 600;
          text-align: center;
        }
        .compare-table td.compare-metric-label {
          text-align: left;
          font-weight: 500;
        }
        .compare-diff-positive {
          background: rgba(0,200,83,0.18);
          color: var(--ad-success);
        }
        .compare-diff-negative {
          background: rgba(255,68,68,0.18);
          color: var(--ad-error);
        }
      `}</style>
      <table className="compare-table" data-testid="compare-diff-table">
        <thead>
          <tr>
            <th>{t("指标")}</th>
            <th>{t("回测 #0")}</th>
            <th>{t("回测 #1")}</th>
            <th>{t("差异")}</th>
          </tr>
        </thead>
        <tbody>
          {metrics.map((key) => {
            const entry = compareDiff[key];
            if (!entry || typeof entry !== "object") return null;
            const diffVal = Number.isFinite(entry.diff) ? entry.diff : null;
            const toneClass = diffVal === null
              ? ""
              : diffVal > 0
                ? "compare-diff-positive"
                : diffVal < 0
                  ? "compare-diff-negative"
                  : "";
            return (
              <tr key={key}>
                <td className="compare-metric-label">{key}</td>
                <td>{formatNum(entry.left)}</td>
                <td>{formatNum(entry.right)}</td>
                <td className={toneClass}>{formatDiff(entry.diff)}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </>
  );
}

export function formatValue(value) {
  if (value === null || value === undefined || value === "") return "-";
  if (typeof value === "number") {
    return Number.isInteger(value) ? String(value) : value.toFixed(4);
  }
  return String(value);
}

export function formatRatio(value) {
  if (!Number.isFinite(value)) return "-";
  const percent = value * 100;
  const sign = percent > 0 ? "+" : "";
  return `${sign}${percent.toFixed(2)}%`;
}

export function backtestExecutionAssumptionsLabel(filters) {
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

export function resolveRunStatus(run, runtime) {
  if (runtime.runId === run.run_id && runtime.status && runtime.status !== "idle") {
    return runtime.status;
  }
  return run.status || "completed";
}

export function ratioTone(value) {
  if (!Number.isFinite(value) || value === 0) return "muted";
  return value > 0 ? "success" : "danger";
}

export function drawdownTone(value) {
  if (!Number.isFinite(value) || value === 0) return "muted";
  return Math.abs(value) >= 0.1 ? "danger" : "warning";
}

export function runtimeTone(status) {
  return getRuntimeStatusMeta(status).tone;
}

function eventTypeLabel(type) {
  return EVENT_TYPE_LABELS[type] || type;
}

export function EventSection({ kicker, title, summary, className = "", testId = null, children }) {
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

export function HistoryMetaGrid({ items, className = "" }) {
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

export function HistoryCardHeader({ id, timestamp }) {
  return (
    <div className="run-history-topline">
      <strong title={id}>{id}</strong>
      <span className="history-time-pill" title={timestamp}>
        {timestamp}
      </span>
    </div>
  );
}

export function HistoryNotice({ children, tone = "muted" }) {
  return <div className={`muted-line history-note history-note-${tone}`}>{children}</div>;
}

export function SectionCardHeader({ title, summary, action = null, value = null }) {
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

export function HistoryCopyBlock({ title, subtitle }) {
  return (
    <div className="history-copy-block">
      <div className="mini-list-title history-copy-title">{title}</div>
      <div className="muted-line history-copy-subtitle">{subtitle}</div>
    </div>
  );
}

export function HistoryFilterGrid({ className = "", fields }) {
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

export function HistoryMetaRow({ items }) {
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

export function HistoryPagination({ currentPage, totalPages, onPrevious, onNext }) {
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

export function HistoryControlBar({
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

export function HistoryExplanationCard({ title, summary, entries, testId }) {
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
  const [expanded, setExpanded] = useState(false);
  const canSaveCurrentArtifact =
    runtime.artifactPersistenceStatus === "transient" &&
    runtime.status === "completed" &&
    Boolean(runtime.runId);
  const runKindLabel =
    runtime.runKind === "backtest"
      ? runtime.artifactPersistenceStatus === "transient"
        ? t("回测预览")
        : t("历史回测")
      : t("模拟运行");
  return (
    <div className="event-panel-header" data-testid="event-panel-intro">
      <div className="event-panel-intro">
        <div className="panel-title">{t("运行与回测面板")}</div>
        <div className="panel-subtitle">
          {t("把运行摘要、事件流、历史记录和账户状态放在同一视图里，同时保持清晰分层。")}
        </div>
      </div>
      <div className="event-summary-grid" style={{ gridTemplateColumns: "repeat(3, 1fr)" }}>
        <SummaryPill label={t("状态")} value={runtimeStatusLabel(runtime.status)} />
        <SummaryPill label={t("事件数")} value={displayedEvents.length} />
        <SummaryPill label={t("权益")} value={formatValue(runtime.account?.equity_estimate)} />
      </div>
      {expanded ? (
        <div className="event-summary-grid" style={{ marginTop: "8px" }}>
          <SummaryPill label={t("类型")} value={runKindLabel} />
          <SummaryPill label={t("运行 ID")} value={runtime.runId || "-"} />
        </div>
      ) : null}
      <div className="event-panel-actions">
        <button
          type="button"
          className="ghost-btn compact-btn"
          onClick={() => setExpanded(!expanded)}
        >
          {expanded ? t("收起") : t("展开详情")}
        </button>
      </div>
      {canSaveCurrentArtifact ? (
        <div className="event-panel-actions">
          <button
            type="button"
            className="ghost-btn compact-btn"
            data-testid="runtime-artifact-save"
            onClick={() => handleSaveCurrentRuntimeArtifact?.()}
          >
            {t("保存本次结果")}
          </button>
          <button
            type="button"
            className="ghost-btn compact-btn"
            data-testid="runtime-artifact-discard"
            onClick={() => handleDiscardCurrentRuntimeArtifact?.()}
          >
            {t("丢弃临时结果")}
          </button>
          <span className="muted-line">{t("未保存结果仅保留在当前会话。")}</span>
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
  const { t } = useI18n();
  return (
    <EventSection
      kicker={t("事件")}
      title={t("事件流")}
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
          <div className="empty-state">{t("当前筛选条件下没有事件。")}</div>
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
            <div className="event-time">{event._timeLabel || new Date(event.event_time_ms).toLocaleTimeString()}</div>
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
              {event.payload?.data_snapshot?.compare_diff ? (
                <CompareDiffTable compareDiff={event.payload.data_snapshot.compare_diff} />
              ) : null}
              <div className="event-meta-row">
                <span>{t("状态：")}{semanticText(eventBadge(event))}</span>
                {event.payload?.limit_triggered ? (
                  <span>{t("限制：")}{formatValue(event.payload.limit_triggered)}</span>
                ) : null}
                {event.payload?.post_risk?.concentration_ratio !== undefined ? (
                  <span data-testid={`event-feed-post-concentration-${event.event_id}`}>
                    {t("集中度：")}{formatRatio(event.payload.post_risk.concentration_ratio)}
                  </span>
                ) : null}
                {event.payload?.post_risk?.max_symbol_net_exposure_ratio !== undefined ? (
                  <span data-testid={`event-feed-post-symbol-net-${event.event_id}`}>
                    {t("单标的净敞口：")}{formatRatio(
                      event.payload.post_risk.max_symbol_net_exposure_ratio
                    )}
                  </span>
                ) : null}
                {event.payload?.post_risk?.portfolio_net_exposure_ratio !== undefined ? (
                  <span data-testid={`event-feed-post-portfolio-net-${event.event_id}`}>
                    {t("组合净敞口：")}{formatRatio(
                      event.payload.post_risk.portfolio_net_exposure_ratio
                    )}
                  </span>
                ) : null}
                {event.payload?.sizing_source ? (
                  <span>{t("定量来源：")}{formatValue(event.payload.sizing_source)}</span>
                ) : null}
                {event.payload?.order_type_decision_reason ? (
                  <span>{t("下单语义：")}{formatValue(event.payload.order_type_decision_reason)}</span>
                ) : null}
                {event.payload?.lifecycle_stage ? (
                  <span>{t("生命周期：")}{formatValue(event.payload.lifecycle_stage)}</span>
                ) : null}
                {event.payload?.side ? <span>{t("方向：")}{directionText(event.payload.side)}</span> : null}
                {event.payload?.signal_direction ? (
                  <span>{t("信号方向：")}{directionText(event.payload.signal_direction)}</span>
                ) : null}
                {event.payload?.source_status ? (
                  <span>{t("数据状态：")}{semanticText(event.payload.source_status)}</span>
                ) : null}
                {event.payload?.source_health ? (
                  <span data-testid={`event-feed-source-health-${event.event_id}`}>
                    {t("来源健康：")}{semanticText(event.payload.source_health)}
                  </span>
                ) : null}
                {event.payload?.freshness_ms !== undefined ? (
                  <span data-testid={`event-feed-freshness-${event.event_id}`}>
                    {t("新鲜度：")}{formatValue(event.payload.freshness_ms)} ms
                  </span>
                ) : null}
                {event.payload?.gap_count !== undefined ? (
                  <span data-testid={`event-feed-gap-count-${event.event_id}`}>
                    {t("缺口：")}{formatValue(event.payload.gap_count)}
                  </span>
                ) : null}
                {qualityFlagsText(event.payload?.quality_flags) ? (
                  <span data-testid={`event-feed-quality-flags-${event.event_id}`}>
                    {t("质量标记：")}{qualityFlagsText(event.payload.quality_flags)}
                  </span>
                ) : null}
                {event.payload?.order_id ? <span>{t("订单：")}{event.payload.order_id}</span> : null}
                {event.payload?.remaining_qty !== undefined ? (
                  <span>{t("剩余数量：")}{formatValue(event.payload.remaining_qty)}</span>
                ) : null}
                {event.payload?.price !== undefined ? (
                  <span>{t("价格：")}{formatValue(event.payload.price)}</span>
                ) : null}
                {event.payload?.limit_price !== undefined && event.payload?.limit_price !== null ? (
                  <span>{t("限价：")}{formatValue(event.payload.limit_price)}</span>
                ) : null}
                {event.payload?.reserved_cash !== undefined ? (
                  <span>{t("冻结现金：")}{formatValue(event.payload.reserved_cash)}</span>
                ) : null}
                {event.payload?.reserved_qty !== undefined ? (
                  <span>{t("冻结仓位：")}{formatValue(event.payload.reserved_qty)}</span>
                ) : null}
                {event.payload?.fee_paid !== undefined ? (
                  <span>{t("手续费：")}{formatValue(event.payload.fee_paid)}</span>
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
  const [expanded, setExpanded] = useState(false);
  if (!runtime.backtestArtifacts) return null;

  return (
    <EventSection
      kicker={t("回测")}
      className="event-sidebar-section event-sidebar-section-summary"
      title={t("回测分析")}
      summary={t("优先显示工件驱动的回测结果摘要，而不是旧式摘要拼接。")}
    >
      <div className="backtest-summary-card" data-testid="backtest-summary-card">
        <div className="account-metric-grid" data-testid="backtest-summary-metrics" style={{gridTemplateColumns: "repeat(3, 1fr)"}}>
          <div className="account-metric-card" data-testid="backtest-summary-total-return">
            <span>{t("收益")}</span>
            <strong>{formatRatio(backtestSummary?.total_return_ratio)}</strong>
          </div>
          <div className="account-metric-card" data-testid="backtest-summary-max-drawdown">
            <span>{t("回撤")}</span>
            <strong>{formatRatio(backtestSummary?.max_drawdown_ratio)}</strong>
          </div>
          <div className="account-metric-card">
            <span>{t("成交数")}</span>
            <strong>{formatValue(backtestSummary?.trade_count)}</strong>
          </div>
        </div>

        {expanded ? (
          <div style={{ marginTop: "12px" }}>
            <SectionCardHeader
              title={t("回测结果详情")}
              summary={t("回测工件和模拟运行分开持久化，详情会恢复权益曲线与事件日志。")}
              value={`${backtestSummary?.step_count || 0} ${t("步")}`}
            />
            <div className="kv-line">
              <span>{t("回测 ID")}</span>
              <strong data-testid="backtest-summary-id">
                {runtime.selectedBacktestId || runtime.runId || "-"}
              </strong>
            </div>
            <div className="kv-line">
              <span>{t("协议")}</span>
              <strong data-testid="backtest-summary-protocol">
                {selectedBacktestSummary?.protocol_name || "-"}
              </strong>
            </div>
            <div className="kv-line">
              <span>{t("配置哈希")}</span>
              <strong data-testid="backtest-summary-config-hash">
                {selectedBacktestSummary?.config_hash || "-"}
              </strong>
            </div>
            <div className="account-metric-card" data-testid="backtest-summary-trade-count">
              <span>{t("最终权益")}</span>
              <strong>{formatValue(backtestSummary?.final_equity)}</strong>
            </div>
            <div className="kv-line">
              <span>{t("开始时间")}</span>
              <strong data-testid="backtest-summary-started-at">
                {backtestStartedAt ? new Date(backtestStartedAt).toLocaleString() : "-"}
              </strong>
            </div>
            <div className="kv-line">
              <span>{t("结束时间")}</span>
              <strong data-testid="backtest-summary-ended-at">
                {backtestEndedAt ? new Date(backtestEndedAt).toLocaleString() : "-"}
              </strong>
            </div>
          </div>
        ) : null}
        <div className="event-panel-actions">
          <button
            type="button"
            className="ghost-btn compact-btn"
            onClick={() => setExpanded(!expanded)}
          >
            {expanded ? t("收起") : t("展开详情")}
          </button>
        </div>
      </div>
    </EventSection>
  );
}

// BacktestHistorySection 已迁移至 ./BacktestHistorySection.jsx
function BacktestHistorySection({
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

// RunHistorySection 已迁移至 ./RunHistorySection.jsx
function RunHistorySection({
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
              className="ghost-btn compact-btn"
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
  const { t } = useI18n();
  const [expanded, setExpanded] = useState(false);
  return (
    <EventSection
      kicker={t("账户")}
      className="event-sidebar-section event-sidebar-section-account"
      title={t("账户与挂单")}
      summary={t("把账户摘要和当前挂单放在同一块，减少来回切换。")}
    >
      <div className="open-orders-card">
        <SectionCardHeader title={t("账户摘要")} summary={t("现金、净值、杠杆和名义价值统一展示。")} />
        <div className="account-metric-grid" style={{gridTemplateColumns: "repeat(3, 1fr)"}}>
          <div className="account-metric-card" data-testid="account-summary-equity">
            <span>{t("总权益")}</span>
            <strong>{formatValue(runtime.account?.equity_estimate ?? runtime.account?.cash_balance)}</strong>
          </div>
          <div className="account-metric-card">
            <span>{t("可用现金")}</span>
            <strong>{formatValue(runtime.account?.available_cash_balance)}</strong>
          </div>
          <div className="account-metric-card">
            <span>{t("持仓")}</span>
            <strong>{formatValue(runtime.account?.positions)}</strong>
          </div>
        </div>

        {expanded ? (
          <div style={{ marginTop: "12px" }}>
            <div className="account-metric-grid">
              <div className="account-metric-card">
                <span>{t("总现金")}</span>
                <strong>{formatValue(runtime.account?.cash_balance)}</strong>
              </div>
              <div className="account-metric-card">
                <span>{t("冻结现金")}</span>
                <strong>{formatValue(runtime.account?.frozen_cash_balance)}</strong>
              </div>
              <div className="account-metric-card">
                <span>{t("总杠杆")}</span>
                <strong>{formatValue(runtime.account?.total_leverage ?? runtime.account?.pnl)}</strong>
              </div>
              <div className="account-metric-card">
                <span>{t("持仓数")}</span>
                <strong>{formatValue(runtime.account?.positions)}</strong>
              </div>
            </div>
            <div className="kv-line">
              <span>{t("总名义价值")}</span>
              <strong>{formatValue(runtime.account?.total_gross_notional)}</strong>
            </div>
            <div className="kv-line">
              <span>{t("净名义价值")}</span>
              <strong>{formatValue(runtime.account?.total_net_notional)}</strong>
            </div>
          </div>
        ) : null}
        <div className="event-panel-actions">
          <button
            type="button"
            className="ghost-btn compact-btn"
            onClick={() => setExpanded(!expanded)}
          >
            {expanded ? t("收起") : t("展开详情")}
          </button>
        </div>
      </div>
      <div className="open-orders-card">
        <SectionCardHeader
          title={t("当前挂单")}
          summary={t("买单主要冻结现金，卖单主要冻结仓位。")}
          value={formatValue(runtime.account?.open_order_count)}
        />
        {openOrders.length === 0 ? <div className="muted-line">{t("当前没有挂单。")}</div> : null}
        {openOrders.map((order) => (
          <div key={order.order_id} className="open-order-item">
            <div className="open-order-topline">
              <span className={`side-pill ${orderSideClass(order.side)}`}>
                {order.side === "Sell" ? t("卖出") : t("买入")}
              </span>
              <strong>{order.order_id}</strong>
            </div>
            <div className="open-order-grid">
              <div>
                <span>{t("剩余数量")}</span>
                <strong>{formatValue(order.remaining_qty)}</strong>
              </div>
              <div>
                <span>{t("限价")}</span>
                <strong>{formatValue(order.limit_price)}</strong>
              </div>
              <div>
                <span>{t("冻结现金")}</span>
                <strong>{formatValue(order.reserved_cash)}</strong>
              </div>
              <div>
                <span>{t("冻结仓位")}</span>
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

        </div>
      </div>
    </section>
  );
}
