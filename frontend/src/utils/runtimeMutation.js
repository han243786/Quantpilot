const DEFAULT_MUTATION_GOVERNANCE = {
  capability_hash: "unknown",
  deployment_revision: "unknown",
  strategy_version: "unknown",
  previous_parameter_version: "unknown",
  proposed_parameter_version: "unknown",
  permission_boundary_model_version: "unknown"
};

const DEFAULT_ACTIVATION_BOUNDARY = {
  requested: "next_cycle_start",
  resolved_sequence_no: null
};

const DEFAULT_ACTIVATION_STATE = {
  requested_boundary: DEFAULT_ACTIVATION_BOUNDARY,
  resolved_sequence_no: null,
  scheduled_at_ms: 0,
  activated_at_ms: 0,
  active_parameter_version: null,
  failure_reason: null
};

const DEFAULT_SAFE_WINDOW_STATE = {
  status: "unknown",
  policy_version: "quantpilot/mutation-safe-window/v1",
  allowed: false,
  reason_code: "UNKNOWN",
  message: "-",
  retryable: false,
  retry_after_ms: null,
  snapshot: {
    policy_version: "quantpilot/mutation-safe-window/v1",
    runtime_status: "unknown",
    open_order_count: 0,
    outstanding_risk_violation: false,
    data_freshness_ms: 0,
    portfolio_exposure_bps: 0,
    cooldown_remaining_ms: 0
  }
};

function nonEmptyString(value, fallback) {
  return typeof value === "string" && value.trim().length > 0 ? value.trim() : fallback;
}

function normalizeMutationTarget(target = {}) {
  return {
    node_id: nonEmptyString(target.node_id, "runtime"),
    module_key: nonEmptyString(target.module_key, "unknown"),
    parameter_path: nonEmptyString(target.parameter_path, "unknown")
  };
}

function normalizeMutationActor(actor = {}) {
  return {
    actor_id: nonEmptyString(actor.actor_id, "unknown"),
    display_name: nonEmptyString(actor.display_name, "Unknown")
  };
}

function normalizeMutationGovernance(governance = {}) {
  return {
    capability_hash: nonEmptyString(
      governance.capability_hash,
      DEFAULT_MUTATION_GOVERNANCE.capability_hash
    ),
    deployment_revision: nonEmptyString(
      governance.deployment_revision,
      DEFAULT_MUTATION_GOVERNANCE.deployment_revision
    ),
    strategy_version: nonEmptyString(
      governance.strategy_version,
      DEFAULT_MUTATION_GOVERNANCE.strategy_version
    ),
    previous_parameter_version: nonEmptyString(
      governance.previous_parameter_version,
      DEFAULT_MUTATION_GOVERNANCE.previous_parameter_version
    ),
    proposed_parameter_version: nonEmptyString(
      governance.proposed_parameter_version,
      DEFAULT_MUTATION_GOVERNANCE.proposed_parameter_version
    ),
    permission_boundary_model_version: nonEmptyString(
      governance.permission_boundary_model_version,
      DEFAULT_MUTATION_GOVERNANCE.permission_boundary_model_version
    )
  };
}

function normalizeActivationBoundary(boundary = {}) {
  const hasResolvedSequence =
    boundary.resolved_sequence_no !== null && boundary.resolved_sequence_no !== undefined;
  return {
    requested: nonEmptyString(boundary.requested, DEFAULT_ACTIVATION_BOUNDARY.requested),
    resolved_sequence_no: hasResolvedSequence && Number.isFinite(Number(boundary.resolved_sequence_no))
      ? Number(boundary.resolved_sequence_no)
      : null
  };
}

function normalizeActivationState(state = {}) {
  const hasResolvedSequence =
    state.resolved_sequence_no !== null && state.resolved_sequence_no !== undefined;
  return {
    requested_boundary: normalizeActivationBoundary(
      state.requested_boundary || DEFAULT_ACTIVATION_STATE.requested_boundary
    ),
    resolved_sequence_no: hasResolvedSequence && Number.isFinite(Number(state.resolved_sequence_no))
      ? Number(state.resolved_sequence_no)
      : null,
    scheduled_at_ms: Number(state.scheduled_at_ms) || 0,
    activated_at_ms: Number(state.activated_at_ms) || 0,
    active_parameter_version: state.active_parameter_version || null,
    failure_reason: state.failure_reason || null
  };
}

function normalizeSafeWindowState(state = {}) {
  const snapshot = state.snapshot || {};
  const hasRetryAfter = state.retry_after_ms !== null && state.retry_after_ms !== undefined;
  return {
    status: nonEmptyString(state.status, DEFAULT_SAFE_WINDOW_STATE.status),
    policy_version: nonEmptyString(
      state.policy_version,
      DEFAULT_SAFE_WINDOW_STATE.policy_version
    ),
    allowed: Boolean(state.allowed),
    reason_code: nonEmptyString(state.reason_code, DEFAULT_SAFE_WINDOW_STATE.reason_code),
    message: nonEmptyString(state.message, DEFAULT_SAFE_WINDOW_STATE.message),
    retryable: Boolean(state.retryable),
    retry_after_ms: hasRetryAfter && Number.isFinite(Number(state.retry_after_ms))
      ? Number(state.retry_after_ms)
      : null,
    snapshot: {
      policy_version: nonEmptyString(
        snapshot.policy_version,
        DEFAULT_SAFE_WINDOW_STATE.snapshot.policy_version
      ),
      runtime_status: nonEmptyString(
        snapshot.runtime_status,
        DEFAULT_SAFE_WINDOW_STATE.snapshot.runtime_status
      ),
      open_order_count: Number(snapshot.open_order_count) || 0,
      outstanding_risk_violation: Boolean(snapshot.outstanding_risk_violation),
      data_freshness_ms: Number(snapshot.data_freshness_ms) || 0,
      portfolio_exposure_bps: Number(snapshot.portfolio_exposure_bps) || 0,
      cooldown_remaining_ms: Number(snapshot.cooldown_remaining_ms) || 0
    }
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

export function normalizeRuntimeMutationProposal(input = {}) {
  return {
    proposal_id: nonEmptyString(input.proposal_id, "unknown"),
    source_kind: nonEmptyString(input.source_kind, "unknown"),
    source_id: nonEmptyString(input.source_id, "unknown"),
    graph_id: nonEmptyString(input.graph_id, "unknown"),
    target: normalizeMutationTarget(input.target),
    old_value: input.old_value ?? null,
    new_value: input.new_value ?? null,
    old_parameter_version: nonEmptyString(input.old_parameter_version, "unknown"),
    proposed_parameter_version: nonEmptyString(input.proposed_parameter_version, "unknown"),
    status: nonEmptyString(input.status, "unknown"),
    rejection_reason: input.rejection_reason || null,
    activation_boundary: normalizeActivationBoundary(input.activation_boundary),
    activation_state: input.activation_state
      ? normalizeActivationState(input.activation_state)
      : null,
    safe_window_state: input.safe_window_state
      ? normalizeSafeWindowState(input.safe_window_state)
      : null,
    rollback_of: input.rollback_of || null,
    rollback_target_parameter_version: input.rollback_target_parameter_version || null,
    actor: normalizeMutationActor(input.actor),
    reason: nonEmptyString(input.reason, "-"),
    governance: normalizeMutationGovernance(input.governance),
    lifecycle: Array.isArray(input.lifecycle)
      ? input.lifecycle.map((entry) => normalizeLifecycleEntry(entry))
      : [],
    created_at_ms: Number(input.created_at_ms) || 0,
    updated_at_ms: Number(input.updated_at_ms) || 0
  };
}

export function buildRuntimeMutationState(source = {}) {
  const proposals = Array.isArray(source) ? source : source.proposals || source.mutations || [];
  const normalized = proposals.map((proposal) => normalizeRuntimeMutationProposal(proposal));
  return {
    proposals: normalized,
    proposed_count: normalized.filter((proposal) => proposal.status === "proposed").length,
    rejected_count: normalized.filter((proposal) => proposal.status === "rejected").length,
    pending_activation_count: normalized.filter(
      (proposal) => proposal.status === "activation_scheduled"
    ).length,
    pending_rollback_count: normalized.filter(
      (proposal) => proposal.status === "rollback_scheduled"
    ).length,
    active_count: normalized.filter((proposal) => proposal.status === "activated").length,
    rolled_back_count: normalized.filter((proposal) => proposal.status === "rolled_back").length,
    failed_activation_count: normalized.filter(
      (proposal) => proposal.status === "activation_failed"
    ).length,
    failed_rollback_count: normalized.filter(
      (proposal) => proposal.status === "rollback_failed"
    ).length,
    safe_window_denied_count: normalized.filter(
      (proposal) => proposal.status === "safe_window_denied"
    ).length,
    active_parameter_version:
      [...normalized]
        .reverse()
        .find((proposal) => ["activated", "rolled_back"].includes(proposal.status))
        ?.activation_state?.active_parameter_version || null
  };
}

export function mutationEventPayloadToProposal(event = {}) {
  const payload = event.payload || event;
  return normalizeRuntimeMutationProposal({
    proposal_id: payload.proposal_id,
    source_kind: payload.source_kind,
    source_id: payload.source_id,
    graph_id: payload.graph_id,
    target: payload.target,
    old_value: payload.old_value,
    new_value: payload.new_value,
    old_parameter_version: payload.old_parameter_version,
    proposed_parameter_version: payload.proposed_parameter_version,
    status: payload.status,
    rejection_reason: payload.rejection_reason,
    activation_boundary: payload.activation_boundary,
    activation_state: payload.activation_state,
    safe_window_state: payload.safe_window_state,
    rollback_of: payload.rollback_of,
    rollback_target_parameter_version: payload.rollback_target_parameter_version,
    actor: payload.actor,
    reason: payload.reason,
    governance: payload.governance,
    lifecycle: payload.lifecycle,
    created_at_ms: event.event_time_ms || payload.created_at_ms,
    updated_at_ms: event.event_time_ms || payload.updated_at_ms
  });
}
