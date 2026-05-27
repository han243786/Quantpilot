import { describe, expect, it } from "vitest";
import {
  aiProposalEventPayloadToRecord,
  buildRuntimeAiProposalState,
  normalizeRuntimeAiProposal
} from "./runtimeAiProposal";

const hash = (suffix) => `sha256:${suffix.repeat(64).slice(0, 64)}`;

const proposal = {
  ai_proposal_id: "ai_proposal_1710000000000_abcd",
  source_kind: "run",
  source_id: "run_1710000000000",
  graph_id: "graph_runtime",
  source_evidence: {
    source_kind: "run",
    source_id: "run_1710000000000",
    graph_id: "graph_runtime",
    event_count: 3,
    evidence_hash: hash("a")
  },
  target: {
    node_id: "risk_node",
    module_key: "builtin.risk.global",
    parameter_path: "max_position"
  },
  old_value: 100000,
  new_value: 90000,
  old_parameter_version: hash("b"),
  proposed_parameter_version: hash("c"),
  status: "static_check_passed",
  denial_reason: null,
  static_check: {
    status: "static_check_passed",
    reason_code: "AI_PROPOSAL_STATIC_CHECK_PASSED",
    message: "AI proposal candidate passed static validation",
    checked_at_ms: 1_710_000_000_000,
    details: []
  },
  model: {
    provider: "openai",
    model: "analysis-model",
    model_version: "2026-04-29"
  },
  prompt_hash: hash("d"),
  evidence_hash: hash("a"),
  actor: {
    actor_id: "ai_assistant",
    display_name: "AI Assistant"
  },
  reason: "Reduce risk before volatility window.",
  governance: {
    capability_hash: hash("e"),
    deployment_revision: hash("f"),
    strategy_version: "1.0.0",
    previous_parameter_version: hash("b"),
    proposed_parameter_version: hash("c"),
    permission_boundary_model_version: "quantpilot/permission-boundary/v1",
    ai_write_policy: "proposal_only"
  },
  config_domain_binding: {
    target_domain: "state_machine",
    before_digest: hash("b"),
    after_digest: hash("c"),
    evidence_anchor_ids: ["backtest:bt1"]
  },
  lifecycle: [
    {
      status: "submitted",
      event_id: "event_ai_proposal_created",
      sequence_no: 4,
      occurred_at_ms: 1_710_000_000_000,
      reason_code: "AI_PROPOSAL_CREATED",
      message: "AI proposal candidate submitted"
    }
  ],
  created_at_ms: 1_710_000_000_000,
  updated_at_ms: 1_710_000_000_000
};

describe("runtime AI proposal contract reader", () => {
  it("normalizes a backend AI proposal without losing governance identity", () => {
    expect(normalizeRuntimeAiProposal(proposal)).toMatchObject({
      ai_proposal_id: proposal.ai_proposal_id,
      status: "static_check_passed",
      is_actionable: true,
      disabled_reason: null,
      governance: proposal.governance,
      config_domain_binding: proposal.config_domain_binding
    });
  });

  it("keeps legacy or partial proposals disabled and explicit", () => {
    const normalized = normalizeRuntimeAiProposal({
      ai_proposal_id: "legacy_ai_proposal",
      status: "unknown"
    });

    expect(normalized.source_kind).toBe("unknown");
    expect(normalized.model.model).toBe("unknown");
    expect(normalized.governance.ai_write_policy).toBe("disabled");
    expect(normalized.config_domain_binding).toBeNull();
    expect(normalized.is_actionable).toBe(false);
    expect(normalized.disabled_reason).toBe("-");
  });

  it("builds proposal counts from list and wrapped shapes", () => {
    const failed = {
      ...proposal,
      ai_proposal_id: "ai_proposal_failed",
      status: "static_check_failed",
      static_check: {
        status: "static_check_failed",
        reason_code: "AI_PROPOSAL_STATIC_CHECK_FAILED",
        message: "AI proposal candidate failed static validation",
        checked_at_ms: 1,
        details: [{ code: "noop_parameter_version", target: "new_value", message: "noop" }]
      }
    };

    const state = buildRuntimeAiProposalState({ ai_proposals: [proposal, failed] });

    expect(state.static_check_passed_count).toBe(1);
    expect(state.static_check_failed_count).toBe(1);
    expect(state.actionable_count).toBe(1);
    expect(buildRuntimeAiProposalState([proposal]).proposals[0].ai_proposal_id).toBe(
      proposal.ai_proposal_id
    );
  });

  it("projects proposal event payloads into the same reader contract", () => {
    const normalized = aiProposalEventPayloadToRecord({
      event_time_ms: proposal.created_at_ms,
      payload: proposal
    });

    expect(normalized.ai_proposal_id).toBe(proposal.ai_proposal_id);
    expect(normalized.created_at_ms).toBe(proposal.created_at_ms);
    expect(normalized.governance.ai_write_policy).toBe("proposal_only");
    expect(normalized.config_domain_binding.target_domain).toBe("state_machine");
  });
});
