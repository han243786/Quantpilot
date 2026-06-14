import { describe, expect, it } from "vitest";
import {
  buildRuntimeMutationState,
  mutationEventPayloadToProposal,
  normalizeRuntimeMutationProposal
} from "./runtimeMutation";

const proposal = {
  proposal_id: "parameter_mutation_1710000000000_abcd",
  source_kind: "run",
  source_id: "run_1710000000000",
  graph_id: "graph_runtime",
  target: {
    node_id: "risk_node",
    module_key: "builtin.risk.global",
    parameter_path: "max_position"
  },
  old_value: 100000,
  new_value: 90000,
  old_parameter_version: "sha256:old",
  proposed_parameter_version: "sha256:new",
  status: "proposed",
  rejection_reason: null,
  activation_boundary: {
    requested: "next_cycle_start",
    resolved_sequence_no: null
  },
  activation_state: null,
  safe_window_state: null,
  rollback_of: null,
  rollback_target_parameter_version: null,
  actor: {
    actor_id: "trader-1",
    display_name: "Trader 1"
  },
  reason: "Reduce risk before volatility window.",
  governance: {
    capability_hash: "sha256:capability",
    deployment_revision: "sha256:deployment",
    strategy_version: "1.0.0",
    previous_parameter_version: "sha256:old",
    proposed_parameter_version: "sha256:new",
    permission_boundary_model_version: "quantpilot/permission-boundary/v1"
  },
  lifecycle: [],
  created_at_ms: 1_710_000_000_000,
  updated_at_ms: 1_710_000_000_000
};

describe("runtime mutation contract reader", () => {
  it("normalizes a backend mutation proposal without losing governance identity", () => {
    expect(normalizeRuntimeMutationProposal(proposal)).toEqual(proposal);
  });

  it("keeps missing fields restrictive and explicit for legacy callers", () => {
    const normalized = normalizeRuntimeMutationProposal({
      proposal_id: "proposal_legacy",
      status: "rejected"
    });

    expect(normalized.source_kind).toBe("unknown");
    expect(normalized.target.module_key).toBe("unknown");
    expect(normalized.actor.actor_id).toBe("unknown");
    expect(normalized.activation_boundary.requested).toBe("next_cycle_start");
    expect(normalized.governance.capability_hash).toBe("unknown");
  });

  it("builds proposal counts from list or wrapped source shapes", () => {
    const state = buildRuntimeMutationState({
      proposals: [
        proposal,
        {
          ...proposal,
          proposal_id: "parameter_mutation_rejected",
          status: "rejected"
        },
        {
          ...proposal,
          proposal_id: "parameter_mutation_active",
          status: "activated",
          activation_state: {
            requested_boundary: {
              requested: "next_cycle_start",
              resolved_sequence_no: null
            },
            resolved_sequence_no: 8,
            scheduled_at_ms: 1_710_000_000_001,
            activated_at_ms: 1_710_000_000_002,
            active_parameter_version: "sha256:new",
            failure_reason: null
          }
        }
      ]
    });

    expect(state.proposed_count).toBe(1);
    expect(state.rejected_count).toBe(1);
    expect(state.active_count).toBe(1);
    expect(state.active_parameter_version).toBe("sha256:new");
    expect(buildRuntimeMutationState([proposal]).proposals[0].proposal_id).toBe(
      proposal.proposal_id
    );
  });

  it("projects proposal event payloads into the same reader contract", () => {
    const normalized = mutationEventPayloadToProposal({
      event_time_ms: proposal.created_at_ms,
      payload: proposal
    });

    expect(normalized.proposal_id).toBe(proposal.proposal_id);
    expect(normalized.graph_id).toBe(proposal.graph_id);
    expect(normalized.old_value).toBe(proposal.old_value);
    expect(normalized.new_value).toBe(proposal.new_value);
    expect(normalized.created_at_ms).toBe(proposal.created_at_ms);
    expect(normalized.governance.proposed_parameter_version).toBe("sha256:new");
  });

  it("normalizes activation lifecycle state and entries", () => {
    const normalized = normalizeRuntimeMutationProposal({
      ...proposal,
      status: "activation_scheduled",
      activation_state: {
        requested_boundary: { requested: "manual_pause" },
        resolved_sequence_no: null,
        scheduled_at_ms: 1_710_000_000_001,
        activated_at_ms: null,
        active_parameter_version: null,
        failure_reason: null
      },
      lifecycle: [
        {
          status: "activation_scheduled",
          event_id: "event_parameter_mutation_scheduled",
          sequence_no: 9,
          occurred_at_ms: 1_710_000_000_001,
          reason_code: "PARAMETER_MUTATION_ACTIVATION_SCHEDULED",
          message: "scheduled"
        }
      ]
    });

    expect(normalized.activation_state.requested_boundary.requested).toBe("manual_pause");
    expect(normalized.activation_state.activated_at_ms).toBe(0);
    expect(normalized.lifecycle[0].sequence_no).toBe(9);
  });

  it("normalizes safe-window and rollback state", () => {
    const denied = {
      ...proposal,
      proposal_id: "parameter_mutation_denied",
      status: "safe_window_denied",
      safe_window_state: {
        status: "denied",
        policy_version: "quantpilot/mutation-safe-window/v1",
        allowed: false,
        reason_code: "SAFE_WINDOW_OPEN_ORDERS",
        message: "open orders must settle",
        retryable: true,
        retry_after_ms: 5000,
        snapshot: {
          policy_version: "quantpilot/mutation-safe-window/v1",
          runtime_status: "paused",
          open_order_count: 1,
          outstanding_risk_violation: false,
          data_freshness_ms: 10,
          portfolio_exposure_bps: 100,
          cooldown_remaining_ms: 5000
        }
      }
    };
    const rolledBack = {
      ...proposal,
      proposal_id: "parameter_rollback_1",
      status: "rolled_back",
      rollback_of: proposal.proposal_id,
      rollback_target_parameter_version: "sha256:old",
      activation_state: {
        requested_boundary: { requested: "next_cycle_start", resolved_sequence_no: null },
        resolved_sequence_no: 12,
        scheduled_at_ms: 2,
        activated_at_ms: 3,
        active_parameter_version: "sha256:old",
        failure_reason: null
      }
    };

    const state = buildRuntimeMutationState([proposal, denied, rolledBack]);

    expect(state.safe_window_denied_count).toBe(1);
    expect(state.rolled_back_count).toBe(1);
    expect(state.active_parameter_version).toBe("sha256:old");
    expect(state.proposals[1].safe_window_state.reason_code).toBe("SAFE_WINDOW_OPEN_ORDERS");
    expect(state.proposals[2].rollback_of).toBe(proposal.proposal_id);
  });
});
