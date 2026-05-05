const DEFAULT_AI_PROPOSAL_GOVERNANCE = {
  capability_hash: "unknown",
  deployment_revision: "unknown",
  strategy_version: "unknown",
  previous_parameter_version: "unknown",
  proposed_parameter_version: "unknown",
  permission_boundary_model_version: "unknown",
  ai_write_policy: "disabled"
};

const DEFAULT_STATIC_CHECK = {
  status: "unknown",
  reason_code: "UNKNOWN",
  message: "-",
  checked_at_ms: 0,
  details: []
};

function nonEmptyString(value, fallback) {
  return typeof value === "string" && value.trim().length > 0 ? value.trim() : fallback;
}

function normalizeTarget(target = {}) {
  return {
    node_id: nonEmptyString(target.node_id, "runtime"),
    module_key: nonEmptyString(target.module_key, "unknown"),
    parameter_path: nonEmptyString(target.parameter_path, "unknown")
  };
}

function normalizeActor(actor = {}) {
  return {
    actor_id: nonEmptyString(actor.actor_id, "unknown"),
    display_name: nonEmptyString(actor.display_name, "Unknown")
  };
}

function normalizeModel(model = {}) {
  return {
    provider: nonEmptyString(model.provider, "unknown"),
    model: nonEmptyString(model.model, "unknown"),
    model_version: nonEmptyString(model.model_version, "unknown")
  };
}

function normalizeGovernance(governance = {}) {
  return {
    capability_hash: nonEmptyString(
      governance.capability_hash,
      DEFAULT_AI_PROPOSAL_GOVERNANCE.capability_hash
    ),
    deployment_revision: nonEmptyString(
      governance.deployment_revision,
      DEFAULT_AI_PROPOSAL_GOVERNANCE.deployment_revision
    ),
    strategy_version: nonEmptyString(
      governance.strategy_version,
      DEFAULT_AI_PROPOSAL_GOVERNANCE.strategy_version
    ),
    previous_parameter_version: nonEmptyString(
      governance.previous_parameter_version,
      DEFAULT_AI_PROPOSAL_GOVERNANCE.previous_parameter_version
    ),
    proposed_parameter_version: nonEmptyString(
      governance.proposed_parameter_version,
      DEFAULT_AI_PROPOSAL_GOVERNANCE.proposed_parameter_version
    ),
    permission_boundary_model_version: nonEmptyString(
      governance.permission_boundary_model_version,
      DEFAULT_AI_PROPOSAL_GOVERNANCE.permission_boundary_model_version
    ),
    ai_write_policy: nonEmptyString(
      governance.ai_write_policy,
      DEFAULT_AI_PROPOSAL_GOVERNANCE.ai_write_policy
    )
  };
}

function normalizeSourceEvidence(evidence = {}) {
  return {
    source_kind: nonEmptyString(evidence.source_kind, "unknown"),
    source_id: nonEmptyString(evidence.source_id, "unknown"),
    graph_id: nonEmptyString(evidence.graph_id, "unknown"),
    event_count: Number(evidence.event_count) || 0,
    evidence_hash: nonEmptyString(evidence.evidence_hash, "unknown")
  };
}

function normalizeStaticCheckDetail(detail = {}) {
  return {
    code: nonEmptyString(detail.code, "UNKNOWN"),
    target: nonEmptyString(detail.target, "unknown"),
    message: nonEmptyString(detail.message, "-")
  };
}

function normalizeStaticCheck(staticCheck = {}) {
  return {
    status: nonEmptyString(staticCheck.status, DEFAULT_STATIC_CHECK.status),
    reason_code: nonEmptyString(staticCheck.reason_code, DEFAULT_STATIC_CHECK.reason_code),
    message: nonEmptyString(staticCheck.message, DEFAULT_STATIC_CHECK.message),
    checked_at_ms: Number(staticCheck.checked_at_ms) || 0,
    details: Array.isArray(staticCheck.details)
      ? staticCheck.details.map((detail) => normalizeStaticCheckDetail(detail))
      : []
  };
}

function normalizeLifecycleEntry(entry = {}) {
  return {
    status: nonEmptyString(entry.status, "unknown"),
    event_id: nonEmptyString(entry.event_id, "unknown"),
    sequence_no: Number(entry.sequence_no) || 0,
    occurred_at_ms: Number(entry.occurred_at_ms) || 0,
    reason_code: nonEmptyString(entry.reason_code, "UNKNOWN"),
    message: nonEmptyString(entry.message, "-")
  };
}

export function normalizeRuntimeAiProposal(input = {}) {
  const status = nonEmptyString(input.status, "unknown");
  const staticCheck = normalizeStaticCheck(input.static_check);
  return {
    ai_proposal_id: nonEmptyString(input.ai_proposal_id, "unknown"),
    source_kind: nonEmptyString(input.source_kind, "unknown"),
    source_id: nonEmptyString(input.source_id, "unknown"),
    graph_id: nonEmptyString(input.graph_id, "unknown"),
    source_evidence: normalizeSourceEvidence(input.source_evidence),
    target: normalizeTarget(input.target),
    old_value: input.old_value ?? null,
    new_value: input.new_value ?? null,
    old_parameter_version: nonEmptyString(input.old_parameter_version, "unknown"),
    proposed_parameter_version: nonEmptyString(input.proposed_parameter_version, "unknown"),
    status,
    denial_reason: input.denial_reason || null,
    static_check: staticCheck,
    model: normalizeModel(input.model),
    prompt_hash: nonEmptyString(input.prompt_hash, "unknown"),
    evidence_hash: nonEmptyString(input.evidence_hash, "unknown"),
    actor: normalizeActor(input.actor),
    reason: nonEmptyString(input.reason, "-"),
    governance: normalizeGovernance(input.governance),
    lifecycle: Array.isArray(input.lifecycle)
      ? input.lifecycle.map((entry) => normalizeLifecycleEntry(entry))
      : [],
    created_at_ms: Number(input.created_at_ms) || 0,
    updated_at_ms: Number(input.updated_at_ms) || 0,
    is_actionable: status === "static_check_passed",
    is_denied: status === "denied" || status === "static_check_failed",
    disabled_reason:
      status === "static_check_passed"
        ? null
        : input.denial_reason || staticCheck.message || "AI 提案尚未准备好审核"
  };
}

export function buildRuntimeAiProposalState(source = {}) {
  const proposals = Array.isArray(source) ? source : source.proposals || source.ai_proposals || [];
  const normalized = proposals.map((proposal) => normalizeRuntimeAiProposal(proposal));
  return {
    proposals: normalized,
    submitted_count: normalized.filter((proposal) => proposal.status === "submitted").length,
    static_check_passed_count: normalized.filter(
      (proposal) => proposal.status === "static_check_passed"
    ).length,
    static_check_failed_count: normalized.filter(
      (proposal) => proposal.status === "static_check_failed"
    ).length,
    denied_count: normalized.filter((proposal) => proposal.status === "denied").length,
    actionable_count: normalized.filter((proposal) => proposal.is_actionable).length
  };
}

export function aiProposalEventPayloadToRecord(event = {}) {
  const payload = event.payload || event;
  return normalizeRuntimeAiProposal({
    ai_proposal_id: payload.ai_proposal_id,
    source_kind: payload.source_kind,
    source_id: payload.source_id,
    graph_id: payload.graph_id,
    source_evidence: payload.source_evidence,
    target: payload.target,
    old_parameter_version: payload.old_parameter_version,
    proposed_parameter_version: payload.proposed_parameter_version,
    status: payload.status,
    denial_reason: payload.denial_reason,
    static_check: payload.static_check,
    model: payload.model,
    prompt_hash: payload.prompt_hash,
    evidence_hash: payload.evidence_hash,
    actor: payload.actor,
    reason: payload.reason,
    governance: payload.governance,
    created_at_ms: event.event_time_ms || payload.created_at_ms,
    updated_at_ms: event.event_time_ms || payload.updated_at_ms
  });
}
