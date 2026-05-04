const API_BASE = import.meta.env.VITE_BACKEND_ORIGIN || "http://127.0.0.1:3000";

export async function fetchApprovals({ reviewState } = {}) {
  const params = new URLSearchParams();
  if (reviewState) params.set("review_state", reviewState);
  const url = `${API_BASE}/api/v1/ai/approvals?${params.toString()}`;
  const res = await fetch(url);
  if (!res.ok) throw new Error(`Failed to fetch approvals: ${res.status}`);
  return res.json();
}

export async function fetchApprovalDetail(approvalId) {
  const url = `${API_BASE}/api/v1/ai/approvals/${approvalId}`;
  const res = await fetch(url);
  if (!res.ok) throw new Error(`Failed to fetch approval: ${res.status}`);
  return res.json();
}

export async function approveProposal(proposalId, actorId, comment) {
  const url = `${API_BASE}/api/v1/ai/proposals/${proposalId}/approve`;
  const res = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ actor_id: actorId, comment: comment || null }),
  });
  if (!res.ok) throw new Error(`Failed to approve: ${res.status}`);
  return res.json();
}

export async function rejectProposal(proposalId, actorId, comment) {
  const url = `${API_BASE}/api/v1/ai/proposals/${proposalId}/reject`;
  const res = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ actor_id: actorId, comment: comment || null }),
  });
  if (!res.ok) throw new Error(`Failed to reject: ${res.status}`);
  return res.json();
}

export async function claimProposal(proposalId, actorId) {
  const url = `${API_BASE}/api/v1/ai/proposals/${proposalId}/claim`;
  const res = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ actor_id: actorId }),
  });
  if (!res.ok) throw new Error(`Failed to claim: ${res.status}`);
  return res.json();
}

export async function fetchSandboxReport(proposalId) {
  const url = `${API_BASE}/api/v1/ai/proposals/${proposalId}/sandbox-report`;
  const res = await fetch(url);
  if (!res.ok) throw new Error(`Failed to fetch sandbox report: ${res.status}`);
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
      return { label: "待审批", color: "#faad14" };
    case "UnderReview":
    case "under_review":
      return { label: "审核中", color: "#1890ff" };
    case "Approved":
    case "approved":
      return { label: "已通过", color: "#52c41a" };
    case "Rejected":
    case "rejected":
      return { label: "已拒绝", color: "#ff4d4f" };
    case "Expired":
    case "expired":
      return { label: "已过期", color: "#d9d9d9" };
    case "Scheduled":
    case "scheduled":
      return { label: "已排期", color: "#722ed1" };
    case "Activated":
    case "activated":
      return { label: "已激活", color: "#13c2c2" };
    case "RolledBack":
    case "rolled_back":
      return { label: "已回滚", color: "#fa8c16" };
    default:
      return { label: state || "-", color: "#999" };
  }
}

export function formatVerdict(verdict) {
  switch (verdict) {
    case "CandidateOutperformsBaseline":
    case "candidate_outperforms_baseline":
      return { label: "候选方案优于基线", color: "#52c41a" };
    case "CandidateComparable":
    case "candidate_comparable":
      return { label: "候选方案与基线相当", color: "#faad14" };
    case "CandidateUnderperforms":
    case "candidate_underperforms":
      return { label: "候选方案劣于基线", color: "#ff4d4f" };
    case "ReplayFidelityPartial":
    case "replay_fidelity_partial":
      return { label: "部分回放(参考价值有限)", color: "#faad14" };
    default:
      return { label: verdict || "-", color: "#999" };
  }
}
