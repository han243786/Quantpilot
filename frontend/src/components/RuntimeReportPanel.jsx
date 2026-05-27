import { useEffect, useMemo, useState } from "react";
import {
  createRuntimeReport,
  fetchRuntimeReportDetail,
  fetchRuntimeReports,
  runtimeReportExportPath
} from "../store/graphStoreRuntimeHistoryApi";
import { API_BASE } from "../store/graphStorePersistenceHelpers";
import { buildActionFailureMessage } from "../utils/actionFailure";
import EvidenceSummaryCards from "./EvidenceSummaryCards";

const STATUS_LABELS = {
  requested: "已请求",
  generating: "生成中",
  ready: "已就绪",
  failed: "失败",
  expired: "已过期",
  source_changed: "源已变化"
};

function reportStatusLabel(status) {
  return STATUS_LABELS[status] || status || "未生成";
}

function reportStatusTone(status) {
  if (status === "ready") return "success";
  if (status === "failed" || status === "expired" || status === "source_changed") {
    return "danger";
  }
  if (status === "generating" || status === "requested") return "warning";
  return "neutral";
}

function compactIdentity(value) {
  if (!value) return "-";
  if (value.length <= 22) return value;
  return `${value.slice(0, 12)}...${value.slice(-6)}`;
}

function sourceKindLabel(sourceKind) {
  return sourceKind === "backtest" ? "回测" : "运行";
}

function reportExportUrl(reportId) {
  return `${API_BASE}${runtimeReportExportPath(reportId)}`;
}

function matchesSource(report, sourceKind, sourceId) {
  return report?.source_kind === sourceKind && report?.source_id === sourceId;
}

export default function RuntimeReportPanel({
  sourceKind,
  sourceId,
  evidenceSource = null,
  title = "证据报告",
  summary = "从压缩证据生成可导出的治理报告，保留来源、序列范围和治理身份。"
}) {
  const [reports, setReports] = useState([]);
  const [selectedReport, setSelectedReport] = useState(null);
  const [status, setStatus] = useState("idle");
  const [error, setError] = useState("");

  const sourceReady = Boolean(sourceKind && sourceId);
  const sourceReports = useMemo(
    () => reports.filter((report) => matchesSource(report, sourceKind, sourceId)),
    [reports, sourceId, sourceKind]
  );
  const currentReport = selectedReport || sourceReports[0] || null;

  async function refreshReports() {
    if (!sourceReady) return;
    setStatus((current) => (current === "creating" ? current : "loading"));
    setError("");
    try {
      const payload = await fetchRuntimeReports();
      setReports(Array.isArray(payload) ? payload : []);
      setStatus("ready");
    } catch (loadError) {
      setStatus("error");
      setError(buildActionFailureMessage("runtime_report", loadError, "加载报告失败。"));
    }
  }

  async function generateReport() {
    if (!sourceReady) return;
    setStatus("creating");
    setError("");
    try {
      const report = await createRuntimeReport({
        source_kind: sourceKind,
        source_id: sourceId
      });
      setSelectedReport(report);
      setReports((current) => {
        const others = current.filter((item) => item.report_id !== report.report_id);
        return [report, ...others];
      });
      setStatus("ready");
    } catch (createError) {
      setStatus("error");
      setError(buildActionFailureMessage("runtime_report", createError, "生成报告失败。"));
    }
  }

  async function openReport(reportId) {
    setStatus("loading");
    setError("");
    try {
      const detail = await fetchRuntimeReportDetail(reportId);
      setSelectedReport(detail);
      setReports((current) => {
        const others = current.filter((item) => item.report_id !== detail.report_id);
        return [detail, ...others];
      });
      setStatus("ready");
    } catch (detailError) {
      setStatus("error");
      setError(buildActionFailureMessage("runtime_report", detailError, "打开报告失败。"));
    }
  }

  useEffect(() => {
    setReports([]);
    setSelectedReport(null);
    setStatus("idle");
    setError("");
    void refreshReports();
    // source identity is the intended reload boundary.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sourceKind, sourceId]);

  if (!sourceReady) return null;

  const range = currentReport?.source_sequence_range;

  return (
    <div className="open-orders-card" data-testid="runtime-report-panel">
      <div className="open-orders-header">
        <div>
          <div className="mini-list-title">{title}</div>
          <div className="muted-line">{summary}</div>
        </div>
        <strong>
          <span className={`status-pill ${reportStatusTone(currentReport?.status)}`}>
            {status === "creating" ? "生成中" : reportStatusLabel(currentReport?.status)}
          </span>
        </strong>
      </div>

      <div className="history-filter-row history-control-bar">
        <button
          type="button"
          className="ad-btn ad-btn--ghost compact-btn"
          data-testid="runtime-report-generate"
          disabled={status === "creating" || status === "loading"}
          onClick={generateReport}
        >
          {currentReport ? "重新生成" : "生成报告"}
        </button>
        <button
          type="button"
          className="ad-btn ad-btn--ghost compact-btn"
          data-testid="runtime-report-refresh"
          disabled={status === "creating" || status === "loading"}
          onClick={refreshReports}
        >
          刷新
        </button>
        {currentReport?.status === "ready" ? (
          <>
            <a
              className="ad-btn ad-btn--ghost compact-btn"
              data-testid="runtime-report-export"
              href={reportExportUrl(currentReport.report_id)}
              target="_blank"
              rel="noreferrer"
            >
              导出 JSON
            </a>
            <a
              className="ad-btn ad-btn--ghost compact-btn"
              data-testid="runtime-report-reveal"
              href={reportExportUrl(currentReport.report_id)}
              target="_blank"
              rel="noreferrer"
            >
              打开报告
            </a>
          </>
        ) : null}
      </div>

      {error ? <div className="history-note history-note-warning">{error}</div> : null}
      {status === "loading" ? <div className="muted-line">正在加载报告...</div> : null}

      {currentReport ? (
        <div className="history-meta-grid">
          <div className="history-meta-chip history-meta-chip-wide">
            <span>报告</span>
            <strong>{currentReport.report_id}</strong>
          </div>
          <div className="history-meta-chip">
            <span>来源</span>
            <strong>{sourceKindLabel(currentReport.source_kind)}</strong>
          </div>
          <div className="history-meta-chip">
            <span>序列范围</span>
            <strong>{range ? `${range.from}-${range.to}` : "-"}</strong>
          </div>
          <div className="history-meta-chip">
            <span>保留证据</span>
            <strong>
              {currentReport.retained_event_count}/{currentReport.source_event_count}
            </strong>
          </div>
          <div className="history-meta-chip history-meta-chip-wide">
            <span>能力边界</span>
            <strong title={currentReport.governance?.capability_hash}>
              {compactIdentity(currentReport.governance?.capability_hash)}
            </strong>
          </div>
          <div className="history-meta-chip history-meta-chip-wide">
            <span>部署修订</span>
            <strong title={currentReport.governance?.deployment_revision}>
              {compactIdentity(currentReport.governance?.deployment_revision)}
            </strong>
          </div>
        </div>
      ) : (
        <div className="muted-line">
          当前{sourceKindLabel(sourceKind)}还没有报告。生成后可在这里打开、导出并查看治理身份。
        </div>
      )}

      {currentReport?.failure_reason ? (
        <div className="history-note history-note-warning">{currentReport.failure_reason}</div>
      ) : null}
      {currentReport?.failure?.reason_code ? (
        <div className="history-note history-note-info" data-testid="runtime-report-failure-meta">
          {currentReport.failure.reason_code} ·{" "}
          {currentReport.failure.retry_eligible ? "可重试" : "不可重试"}
        </div>
      ) : null}

      {evidenceSource ? (
        <EvidenceSummaryCards source={evidenceSource} testId="runtime-report-evidence-summary" />
      ) : null}

      {sourceReports.length > 0 ? (
        <div className="mini-list" data-testid="runtime-report-list">
          <div className="mini-list-title">报告列表</div>
          {sourceReports.map((report) => (
            <button
              key={report.report_id}
              type="button"
              className="open-order-item history-list-button"
              data-testid={`runtime-report-list-item-${report.report_id}`}
              onClick={() => openReport(report.report_id)}
            >
              <div className="open-order-topline">
                <strong>{report.report_id}</strong>
                <span>{reportStatusLabel(report.status)}</span>
              </div>
              <div className="muted-line">
                {report.source_sequence_range
                  ? `sequence ${report.source_sequence_range.from}-${report.source_sequence_range.to}`
                  : "no sequence range"}{" "}
                · {report.generation_policy}
              </div>
            </button>
          ))}
        </div>
      ) : null}
    </div>
  );
}
