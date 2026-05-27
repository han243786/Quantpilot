import { API_BASE } from "./api";

export async function fetchApprovals({ reviewState } = {}) {
  const params = new URLSearchParams();
  if (reviewState) params.set("review_state", reviewState);
  const url = `${API_BASE}/v1/ai/approvals?${params.toString()}`;
  const res = await fetch(url);
  if (!res.ok) throw new Error(`获取审批列表失败: ${res.status}`);
  return res.json();
}

export async function fetchApprovalDetail(approvalId) {
  const url = `${API_BASE}/v1/ai/approvals/${approvalId}`;
  const res = await fetch(url);
  if (!res.ok) throw new Error(`获取审批详情失败: ${res.status}`);
  return res.json();
}

export async function approveProposal(proposalId, actorId, comment) {
  const url = `${API_BASE}/v1/ai/proposals/${proposalId}/approve`;
  const res = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ actor_id: actorId, comment: comment || null }),
  });
  if (!res.ok) throw new Error(`审批通过失败: ${res.status}`);
  return res.json();
}

export async function rejectProposal(proposalId, actorId, comment) {
  const url = `${API_BASE}/v1/ai/proposals/${proposalId}/reject`;
  const res = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ actor_id: actorId, comment: comment || null }),
  });
  if (!res.ok) throw new Error(`审批拒绝失败: ${res.status}`);
  return res.json();
}

export async function claimProposal(proposalId, actorId) {
  const url = `${API_BASE}/v1/ai/proposals/${proposalId}/claim`;
  const res = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ actor_id: actorId }),
  });
  if (!res.ok) throw new Error(`认领审批失败: ${res.status}`);
  return res.json();
}

export async function fetchSandboxReport(proposalId) {
  const url = `${API_BASE}/v1/ai/proposals/${proposalId}/sandbox-report`;
  const res = await fetch(url);
  if (!res.ok) throw new Error(`获取沙箱报告失败: ${res.status}`);
  return res.json();
}

export async function requestSandboxVerification(proposalId) {
  const url = `${API_BASE}/v1/ai/proposals/${proposalId}/request-sandbox`;
  const res = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ proposal_id: proposalId, backtest_id: null }),
  });
  if (!res.ok) throw new Error(`请求沙箱重试失败: ${res.status}`);
  return res.json();
}

export function formatApprovalLevel(level) {
  switch (level) {
    case "L1SingleReviewer":
    case "l1_single_reviewer":
      return "L1 - 单人审批";
    case "L2DualReviewer":
    case "l2_dual_reviewer":
      return "L2 - 双人审批";
    case "L3RiskOwnerReview":
    case "l3_risk_owner_review":
      return "L3 - 风控负责人审批";
    default:
      return level || "-";
  }
}

export function formatReviewState(state) {
  switch (state) {
    case "Pending":
    case "pending":
      return { label: "待审批", color: "var(--ad-warning)" };
    case "UnderReview":
    case "under_review":
      return { label: "审核中", color: "var(--ad-accent)" };
    case "Approved":
    case "approved":
      return { label: "已通过", color: "var(--ad-success)" };
    case "Rejected":
    case "rejected":
      return { label: "已拒绝", color: "var(--ad-error)" };
    case "Expired":
    case "expired":
      return { label: "已过期", color: "var(--ad-text-secondary)" };
    case "Scheduled":
    case "scheduled":
      return { label: "已排期", color: "var(--ad-text-secondary)" };
    case "Activated":
    case "activated":
      return { label: "已激活", color: "var(--ad-success)" };
    case "RolledBack":
    case "rolled_back":
      return { label: "已回滚", color: "var(--ad-warning)" };
    default:
      return { label: state || "-", color: "var(--ad-text-muted)" };
  }
}

export function formatVerdict(verdict) {
  switch (verdict) {
    case "CandidateOutperformsBaseline":
    case "candidate_outperforms_baseline":
      return { label: "候选方案优于基线", color: "var(--ad-success)" };
    case "CandidateComparable":
    case "candidate_comparable":
      return { label: "候选方案与基线相当", color: "var(--ad-warning)" };
    case "CandidateUnderperforms":
    case "candidate_underperforms":
      return { label: "候选方案劣于基线", color: "var(--ad-error)" };
    case "ReplayFidelityPartial":
    case "replay_fidelity_partial":
      return { label: "部分回放(参考价值有限)", color: "var(--ad-warning)" };
    default:
      return { label: verdict || "-", color: "var(--ad-text-muted)" };
  }
}
