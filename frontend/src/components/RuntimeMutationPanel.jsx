import { useMemo } from "react";
import { buildRuntimeMutationState } from "../utils/runtimeMutation";
import { useOrderAnimation } from "../hooks/useOrderAnimation";

function compactIdentity(value) {
  if (!value) return "-";
  if (value.length <= 22) return value;
  return `${value.slice(0, 12)}...${value.slice(-6)}`;
}

function statusTone(status) {
  if (status === "activated" || status === "rolled_back") return "success";
  if (
    status === "rejected" ||
    status === "activation_failed" ||
    status === "rollback_failed" ||
    status === "safe_window_denied"
  ) {
    return "danger";
  }
  if (status === "activation_scheduled" || status === "rollback_scheduled") return "warning";
  return "neutral";
}

function boundaryLabel(proposal) {
  const state = proposal.activation_state;
  const boundary = state?.requested_boundary || proposal.activation_boundary;
  const resolved = state?.resolved_sequence_no || boundary?.resolved_sequence_no;
  return resolved ? `${boundary.requested} #${resolved}` : boundary?.requested || "-";
}

function ProposalItem({ proposal, canActivate, capabilityContext, onActivateProposal, onRollbackProposal, testId }) {
  const animClass = useOrderAnimation(proposal.status === "activated");

  return (
    <div
      className={`open-order-item${animClass ? " " + animClass : ""}`}
      data-testid={`${testId}-proposal-${proposal.proposal_id}`}
    >
      <div className="open-order-topline">
        <strong>{proposal.target.parameter_path}</strong>
        <span className={`status-pill ${statusTone(proposal.status)}`}>
          {proposal.status}
        </span>
      </div>
      <div className="muted-line">
        {proposal.target.module_key} · {boundaryLabel(proposal)}
      </div>
      {proposal.safe_window_state ? (
        <div className="muted-line" data-testid={`${testId}-safe-window-${proposal.proposal_id}`}>
          安全窗口 {proposal.safe_window_state.status} ·{" "}
          {proposal.safe_window_state.reason_code}
        </div>
      ) : null}
      {proposal.rollback_of ? (
        <div className="muted-line">
          回滚来源 {compactIdentity(proposal.rollback_of)}
        </div>
      ) : null}
      <div className="kv-line">
        <span>版本</span>
        <strong title={proposal.proposed_parameter_version}>
          {compactIdentity(proposal.proposed_parameter_version)}
        </strong>
      </div>
      <div className="inline-actions">
        <button
          type="button"
          className="ghost-btn compact-btn"
          disabled={
            !canActivate ||
            !["proposed", "safe_window_denied"].includes(proposal.status) ||
            !onActivateProposal
          }
          onClick={() =>
            onActivateProposal?.(proposal, {
              capability_context: capabilityContext,
              activation_boundary: proposal.activation_boundary
            })
          }
          data-testid={`${testId}-activate-${proposal.proposal_id}`}
        >
          激活
        </button>
        <button
          type="button"
          className="ghost-btn compact-btn"
          disabled={!canActivate || proposal.status !== "activated" || !onRollbackProposal}
          onClick={() =>
            onRollbackProposal?.(proposal, {
              capability_context: capabilityContext,
              activation_boundary: proposal.activation_boundary,
              target_parameter_version: proposal.old_parameter_version
            })
          }
          data-testid={`${testId}-rollback-${proposal.proposal_id}`}
        >
          回滚
        </button>
      </div>
    </div>
  );
}

export default function RuntimeMutationPanel({
  sourceKind,
  sourceId,
  capabilityContext = null,
  initialMutations = [],
  onActivateProposal = null,
  onRollbackProposal = null,
  title = "参数变更",
  testId = "runtime-mutation-panel"
}) {
  const sourceReady = Boolean(sourceKind && sourceId);
  const mutationState = useMemo(
    () => buildRuntimeMutationState(initialMutations),
    [initialMutations]
  );
  const canActivate = Boolean(capabilityContext);

  if (!sourceReady) return null;

  return (
    <div className="open-orders-card" data-testid={testId}>
      <div className="open-orders-header">
        <div>
          <div className="mini-list-title">{title}</div>
          <div className="muted-line">
            当前版本 {compactIdentity(mutationState.active_parameter_version)} · 待处理{" "}
            {mutationState.pending_activation_count + mutationState.pending_rollback_count}
          </div>
        </div>
        <strong>
          <span
            className={`status-pill ${statusTone(
              mutationState.active_count ? "activated" : "idle"
            )}`}
          >
            {mutationState.active_count ? "已生效" : "空闲"}
          </span>
        </strong>
      </div>

      <div className="history-meta-grid">
        <div className="history-meta-chip">
          <span>提案</span>
          <strong>{mutationState.proposed_count}</strong>
        </div>
        <div className="history-meta-chip">
          <span>待处理</span>
          <strong>{mutationState.pending_activation_count}</strong>
        </div>
        <div className="history-meta-chip">
          <span>已生效</span>
          <strong>{mutationState.active_count}</strong>
        </div>
        <div className="history-meta-chip">
          <span>已拒绝</span>
          <strong>{mutationState.safe_window_denied_count}</strong>
        </div>
        <div className="history-meta-chip">
          <span>已回滚</span>
          <strong>{mutationState.rolled_back_count}</strong>
        </div>
      </div>

      {!canActivate ? (
        <div className="history-note history-note-warning" data-testid={`${testId}-boundary-lock`}>
          激活前需要当前能力上下文。
        </div>
      ) : null}

      <div className="mini-list">
        {mutationState.proposals.map((proposal) => (
          <ProposalItem
            key={proposal.proposal_id}
            proposal={proposal}
            canActivate={canActivate}
            capabilityContext={capabilityContext}
            onActivateProposal={onActivateProposal}
            onRollbackProposal={onRollbackProposal}
            testId={testId}
          />
        ))}
        {mutationState.proposals.length === 0 ? (
          <div>
            <div className="muted-line">暂无参数变更提案。</div>
            <div className="muted-line" style={{ marginTop: 4, fontSize: 12 }}>启动模拟后AI提案自动生成，可在此调整运行参数。</div>
          </div>
        ) : null}
      </div>
    </div>
  );
}
