import { useMemo, useState } from "react";
import { useI18n } from "../i18n";
import {
  buildCompactEvidenceProjection,
  buildRuntimeTimelineItemsFromDetail
} from "../utils/runtimeTimeline";

const STAGE_LABELS = {
  data: "数据",
  intent: "意图",
  agent: "代理",
  risk: "风控",
  execution: "执行",
  fill: "成交",
  system: "系统"
};

function uniqueValues(items, key) {
  return Array.from(new Set(items.map((item) => item[key]).filter(Boolean))).sort();
}

function formatGovernanceHash(value = "") {
  if (!value || value === "unknown") return "unknown";
  if (value.length <= 18) return value;
  return `${value.slice(0, 12)}...${value.slice(-6)}`;
}

function matchesFilter(value, filter) {
  return filter === "all" || value === filter;
}

export default function GovernedTimelinePanel({
  source = {},
  title = "证据时间轴",
  summary = "按治理 envelope 组织事件，优先展示关键证据并保留可审计的序号、阶段和治理身份。",
  testId = "governed-timeline-panel"
}) {
  const { t } = useI18n();
  const timeline = useMemo(() => buildRuntimeTimelineItemsFromDetail(source), [source]);
  const compactEvidence = useMemo(() => buildCompactEvidenceProjection(source), [source]);
  const [severityFilter, setSeverityFilter] = useState("all");
  const [retentionFilter, setRetentionFilter] = useState("all");
  const [moduleFilter, setModuleFilter] = useState("all");
  const [selectedEventId, setSelectedEventId] = useState(null);

  const filteredTimeline = useMemo(
    () =>
      timeline.filter(
        (item) =>
          matchesFilter(item.severity, severityFilter) &&
          matchesFilter(item.retention_class, retentionFilter) &&
          matchesFilter(item.module_key, moduleFilter)
      ),
    [moduleFilter, retentionFilter, severityFilter, timeline]
  );
  const groupedTimeline = useMemo(() => {
    const groups = new Map();
    filteredTimeline.forEach((item) => {
      const stage = item.stage || "system";
      if (!groups.has(stage)) groups.set(stage, []);
      groups.get(stage).push(item);
    });
    return Array.from(groups.entries());
  }, [filteredTimeline]);
  const selectedItem =
    filteredTimeline.find((item) => item.event_id === selectedEventId) || filteredTimeline[0] || null;

  return (
    <div className="open-orders-card governed-timeline-panel" data-testid={testId}>
      <div className="open-orders-header">
        <div>
          <div className="mini-list-title">{title}</div>
          <div className="muted-line">{summary}</div>
        </div>
        <strong data-testid={`${testId}-retained-count`}>
          {compactEvidence.retained_event_count}/{compactEvidence.source_event_count}
        </strong>
      </div>

      <div className="history-filter-grid history-filter-grid-runtime">
        <label className="field-block">
          <span>{t("严重度")}</span>
          <select
            value={severityFilter}
            onChange={(event) => setSeverityFilter(event.target.value)}
            data-testid={`${testId}-severity-filter`}
          >
            <option value="all">全部</option>
            {uniqueValues(timeline, "severity").map((value) => (
              <option key={value} value={value}>
                {value}
              </option>
            ))}
          </select>
        </label>
        <label className="field-block">
          <span>{t("保留级别")}</span>
          <select
            value={retentionFilter}
            onChange={(event) => setRetentionFilter(event.target.value)}
            data-testid={`${testId}-retention-filter`}
          >
            <option value="all">全部</option>
            {uniqueValues(timeline, "retention_class").map((value) => (
              <option key={value} value={value}>
                {value}
              </option>
            ))}
          </select>
        </label>
        <label className="field-block">
          <span>{t("模块")}</span>
          <select
            value={moduleFilter}
            onChange={(event) => setModuleFilter(event.target.value)}
            data-testid={`${testId}-module-filter`}
          >
            <option value="all">全部</option>
            {uniqueValues(timeline, "module_key").map((value) => (
              <option key={value} value={value}>
                {value}
              </option>
            ))}
          </select>
        </label>
      </div>

      {timeline.length === 0 ? (
        <div className="muted-line">当前还没有可展示的 governed timeline。</div>
      ) : null}

      {groupedTimeline.map(([stage, items]) => (
        <div key={stage} className="mini-list" data-testid={`${testId}-stage-${stage}`}>
          <div className="mini-list-title">
            {STAGE_LABELS[stage] || stage} · {items.length}
          </div>
          {items.map((item) => (
            <button
              key={item.event_id}
              type="button"
              className={`open-order-item governed-timeline-item${
                selectedItem?.event_id === item.event_id ? " is-active" : ""
              }`}
              onClick={() => setSelectedEventId(item.event_id)}
              data-testid={`${testId}-item-${item.event_id}`}
            >
              <div className="open-order-topline">
                <strong>#{item.sequence_no} · {item.event_type}</strong>
                <span>{item.retention_class}</span>
              </div>
              <div className="muted-line">{item.summary}</div>
            </button>
          ))}
        </div>
      ))}

      {selectedItem ? (
        <div className="mini-list" data-testid={`${testId}-selected-detail`}>
          <div className="mini-list-title">证据详情</div>
          <div className="kv-line">
            <span>{t("事件")}</span>
            <strong>{selectedItem.event_id}</strong>
          </div>
          <div className="kv-line">
            <span>{t("阶段")}</span>
            <strong>{STAGE_LABELS[selectedItem.stage] || selectedItem.stage}</strong>
          </div>
          <div className="kv-line">
            <span>{t("治理边界")}</span>
            <strong title={selectedItem.governance.capability_hash}>
              {formatGovernanceHash(selectedItem.governance.capability_hash)}
            </strong>
          </div>
          <div className="kv-line">
            <span>{t("原因")}</span>
            <strong>{selectedItem.reason_code || "-"}</strong>
          </div>
        </div>
      ) : null}
    </div>
  );
}
