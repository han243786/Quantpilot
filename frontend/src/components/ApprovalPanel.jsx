import React, { useState, useEffect, useCallback } from "react";
import Block5Nav from "./Block5Nav";
import { API_BASE } from "../utils/api";
import { useI18n } from "../i18n";
import {
  fetchApprovals,
  approveProposal,
  rejectProposal,
  claimProposal,
  fetchSandboxReport,
  formatApprovalLevel,
  formatReviewState,
  formatVerdict,
} from "../utils/runtimeApproval";

const ACTOR_ID =
  import.meta.env.VITE_ACTOR_ID || "local_operator";

export default function ApprovalPanel() {
  const { t } = useI18n();
  const [approvals, setApprovals] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [selected, setSelected] = useState(null);
  const [sandboxReport, setSandboxReport] = useState(null);
  const [actionMsg, setActionMsg] = useState(null);
  const [rejectingId, setRejectingId] = useState(null);
  const [rejectComment, setRejectComment] = useState("");

  const loadApprovals = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await fetchApprovals();
      setApprovals(data || []);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadApprovals();
  }, [loadApprovals]);

  const handleClaim = async (proposalId) => {
    try {
      await claimProposal(proposalId, ACTOR_ID);
      setActionMsg({ type: "success", text: t("已认领审批单") });
      loadApprovals();
    } catch (err) {
      setActionMsg({ type: "error", text: err.message });
    }
  };

  const handleApprove = async (proposalId) => {
    try {
      await approveProposal(proposalId, ACTOR_ID);
      setActionMsg({ type: "success", text: t("审批已通过") });
      loadApprovals();
    } catch (err) {
      setActionMsg({ type: "error", text: err.message });
    }
  };

  const handleReject = async (proposalId) => {
    if (!rejectingId) {
      setRejectingId(proposalId);
      setRejectComment("");
      return;
    }
    try {
      await rejectProposal(proposalId, ACTOR_ID, rejectComment || null);
      setActionMsg({ type: "success", text: t("审批已拒绝") });
      setRejectingId(null);
      loadApprovals();
    } catch (err) {
      setActionMsg({ type: "error", text: err.message });
    }
  };

  const handleViewReport = async (proposalId) => {
    try {
      const report = await fetchSandboxReport(proposalId);
      setSandboxReport(report);
    } catch (err) {
      setActionMsg({ type: "error", text: t("沙箱报告不可用: ") + err.message });
    }
  };

  return (
    <div className="qp-page">
      <Block5Nav />

      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h2 style={{ margin: 0 }}>{t("审批队列")}</h2>
        <button
          className="qp-btn qp-btn--ghost qp-btn--sm"
          onClick={loadApprovals}
          disabled={loading}
        >
          {loading ? t("加载中...") : t("刷新")}
        </button>
      </div>

      {actionMsg && (
        <div
          className={actionMsg.type === "success" ? "qp-badge qp-badge--ok" : "qp-error"}
          style={actionMsg.type === "success" ? { margin: "12px 0", padding: "8px 14px" } : {}}
          role="alert"
        >
          {actionMsg.text}
          <button
            className="qp-btn qp-btn--ghost qp-btn--sm"
            style={{ marginLeft: 12 }}
            onClick={() => setActionMsg(null)}
          >
            {t("关闭")}
          </button>
        </div>
      )}

      {error && (
        <div className="qp-error" role="alert">
          <span>{t("加载失败:")} {error}</span>
          <button className="qp-btn qp-btn--sm" onClick={loadApprovals}>{t("重试")}</button>
        </div>
      )}

      {!loading && !error && approvals.length === 0 && (
        <div className="qp-empty">{t("暂无待审批的 AI 提案")}</div>
      )}

      {approvals.map((approval) => {
        const state = formatReviewState(approval.review_state);
        const isPending =
          approval.review_state === "Pending" || approval.review_state === "pending";
        const isUnderReview =
          approval.review_state === "UnderReview" || approval.review_state === "under_review";
        const isActionable = isPending || isUnderReview;
        const isExpanded = selected === approval.approval_id;

        return (
          <div
            className="qp-card qp-fade-in"
            key={approval.approval_id}
            onClick={() => setSelected(isExpanded ? null : approval.approval_id)}
            style={{ cursor: "pointer" }}
            role="listitem"
          >
            <div className="qp-card__header">
              <div>
                <span className="qp-card__title qp-metric" style={{ fontSize: 12 }}>
                  {approval.proposal_id}
                </span>
                <span
                  className="qp-badge"
                  style={{
                    marginLeft: 10,
                    background: state.color + "22",
                    color: state.color,
                  }}
                >
                  {state.label}
                </span>
                <span style={{ marginLeft: 8, fontSize: 12, color: "var(--tv-text-muted)" }}>
                  {formatApprovalLevel(approval.approval_level)}
                </span>
              </div>
              {isActionable && (
                <div style={{ display: "flex", gap: 6 }} onClick={(e) => e.stopPropagation()}>
                  {isPending && (
                    <button className="qp-btn qp-btn--primary qp-btn--sm" onClick={() => handleClaim(approval.proposal_id)}>
                      {t("认领")}
                    </button>
                  )}
                  <button className="qp-btn qp-btn--primary qp-btn--sm" onClick={() => handleApprove(approval.proposal_id)}>
                    {t("通过")}
                  </button>
                  <button className="qp-btn qp-btn--danger qp-btn--sm" onClick={() => handleReject(approval.proposal_id)}>
                    {rejectingId === approval.proposal_id ? t("确认") : t("拒绝")}
                  </button>
                </div>
              )}
            </div>

            <div className="qp-card__meta">
              <span>
                {t("审批进度: ")}{approval.reviewers_approved?.length || 0}/{approval.reviewers_required}
              </span>
              <span>{t("到期: ")}{new Date(approval.expires_at_ms).toLocaleString()}</span>
              {approval.chain_stage_impact?.length > 0 && (
                <span>{t("影响: ")}{approval.chain_stage_impact.join(" → ")}</span>
              )}
            </div>

            {rejectingId === approval.proposal_id && (
              <div className="qp-card__body" onClick={(e) => e.stopPropagation()}>
                <input
                  className="qp-input"
                  type="text"
                  placeholder={t("拒绝原因（可选，Enter 提交）")}
                  value={rejectComment}
                  onChange={(e) => setRejectComment(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && handleReject(approval.proposal_id)}
                  autoFocus
                />
                <div style={{ marginTop: 8, display: "flex", gap: 6 }}>
                  <button className="qp-btn qp-btn--danger qp-btn--sm" onClick={() => handleReject(approval.proposal_id)}>
                    {t("确认拒绝")}
                  </button>
                  <button className="qp-btn qp-btn--ghost qp-btn--sm" onClick={() => setRejectingId(null)}>
                    {t("取消")}
                  </button>
                </div>
              </div>
            )}

            {isExpanded && (
              <div className="qp-card__body">
                {approval.sandbox_report_url && (
                  <p>
                    <button className="qp-btn qp-btn--ghost qp-btn--sm" onClick={() => handleViewReport(approval.proposal_id)}>
                      {t("查看沙箱报告")}
                    </button>
                  </p>
                )}
                <p><strong>{t("审批ID:")}</strong> {approval.approval_id}</p>
                <p><strong>{t("审批人:")}</strong> {approval.reviewers_assigned?.join(", ") || t("未指定")}</p>
                <p><strong>{t("已通过:")}</strong> {approval.reviewers_approved?.join(", ") || "-"}</p>
                {approval.rollback_plan?.method && (
                  <p><strong>{t("回滚方案:")}</strong> {approval.rollback_plan.method} ({t("预计")} {approval.rollback_plan.estimated_recovery_ms}ms)</p>
                )}

                {approval.lifecycle?.length > 0 && (
                  <>
                    <h3>{t("审批时间轴")}</h3>
                    <div className="qp-timeline">
                      {approval.lifecycle.map((entry, idx) => (
                        <div className="qp-timeline__item" key={idx}>
                          <span style={{ color: "var(--tv-text)", fontWeight: 500 }}>
                            {entry.message}
                          </span>
                          <br />
                          <span style={{ fontSize: 11 }}>
                            {new Date(entry.occurred_at_ms).toLocaleTimeString()}
                            {entry.actor_id && ` — ${entry.actor_id}`}
                          </span>
                        </div>
                      ))}
                    </div>
                  </>
                )}
              </div>
            )}
          </div>
        );
      })}

      {sandboxReport && (
        <div className="qp-card" style={{ marginTop: 16, borderColor: "var(--tv-accent)" }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 12 }}>
            <h3 style={{ margin: 0 }}>{t("沙箱验证报告")}</h3>
            <button className="qp-btn qp-btn--ghost qp-btn--sm" onClick={() => setSandboxReport(null)}>
              {t("关闭")}
            </button>
          </div>

          <div className="qp-card__meta" style={{ marginBottom: 12 }}>
            <span>
              {t("回放窗口: ")}{sandboxReport.replay_window?.from_ts} → {sandboxReport.replay_window?.to_ts}
            </span>
            <span>
              {t("判定: ")}{" "}
              <span
                className="qp-badge"
                style={{
                  background: formatVerdict(sandboxReport.verdict).color + "22",
                  color: formatVerdict(sandboxReport.verdict).color,
                }}
              >
                {formatVerdict(sandboxReport.verdict).label}
              </span>
            </span>
          </div>

          {sandboxReport.warnings?.length > 0 && (
            <div className="qp-card__body" style={{ marginBottom: 12 }}>
              {sandboxReport.warnings.map((w, i) => (
                <div key={i} style={{ color: "var(--tv-orange)", fontSize: 12 }}>
                  {w}
                </div>
              ))}
            </div>
          )}

          <table className="qp-table">
            <thead>
              <tr>
                <th>{t("指标")}</th>
                <th>{t("基线")}</th>
                <th>{t("候选")}</th>
                <th>{t("差异")}</th>
              </tr>
            </thead>
            <tbody>
              {sandboxReport.baseline_metrics &&
                Object.keys(sandboxReport.baseline_metrics).map((key) => {
                  const diff = sandboxReport.diffs?.[key] || "-";
                  const isUp = typeof diff === "string" && diff.startsWith("+");
                  const isDown = typeof diff === "string" && diff.startsWith("-");
                  return (
                    <tr key={key}>
                      <td style={{ color: "var(--tv-text-secondary)", fontFamily: "var(--tv-font)", fontSize: 12 }}>
                        {key}
                      </td>
                      <td className="qp-metric" style={{ fontSize: 12 }}>
                        {Number(sandboxReport.baseline_metrics[key]).toFixed(4)}
                      </td>
                      <td className="qp-metric" style={{ fontSize: 12 }}>
                        {Number(sandboxReport.candidate_metrics?.[key] || 0).toFixed(4)}
                      </td>
                      <td className={`qp-metric ${isUp ? "up" : isDown ? "down" : ""}`} style={{ fontSize: 12 }}>
                        {diff}
                      </td>
                    </tr>
                  );
                })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
