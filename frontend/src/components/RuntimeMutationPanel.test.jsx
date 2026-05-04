import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import RuntimeMutationPanel from "./RuntimeMutationPanel";

const capabilityContext = {
  schema_hash: "sha256:capability",
  permission_boundary: {
    model_version: "quantpilot/permission-boundary/v1",
    execution_owner_module: "builtin.execution.paper",
    live_execution_allowed: false,
    ai_write_policy: "proposal_only",
    plugin_network_default: "deny",
    non_execution_order_access: "deny"
  }
};

function proposal(overrides = {}) {
  return {
    proposal_id: "parameter_mutation_1",
    source_kind: "run",
    source_id: "run_1",
    graph_id: "graph_1",
    target: {
      node_id: "risk_risk_1",
      module_key: "builtin.risk.global",
      parameter_path: "max_position"
    },
    old_value: 0.2,
    new_value: 0.15,
    old_parameter_version: "sha256:old",
    proposed_parameter_version: "sha256:new",
    status: "proposed",
    rejection_reason: null,
    activation_boundary: {
      requested: "next_cycle_start",
      resolved_sequence_no: null
    },
    activation_state: null,
    actor: {
      actor_id: "operator_1",
      display_name: "Operator 1"
    },
    reason: "Reduce max position.",
    governance: {
      capability_hash: "sha256:capability",
      deployment_revision: "sha256:deployment",
      strategy_version: "1.0.0",
      previous_parameter_version: "sha256:old",
      proposed_parameter_version: "sha256:new",
      permission_boundary_model_version: "quantpilot/permission-boundary/v1"
    },
    lifecycle: [],
    created_at_ms: 1,
    updated_at_ms: 1,
    ...overrides
  };
}

describe("RuntimeMutationPanel", () => {
  const onActivateProposal = vi.fn();
  const onRollbackProposal = vi.fn();

  beforeEach(() => {
    onActivateProposal.mockReset();
    onRollbackProposal.mockReset();
  });

  it("renders mutation proposals and emits activation requests with capability context", () => {
    render(
      <RuntimeMutationPanel
        sourceKind="run"
        sourceId="run_1"
        capabilityContext={capabilityContext}
        initialMutations={[proposal()]}
        onActivateProposal={onActivateProposal}
      />
    );

    expect(screen.getByTestId("runtime-mutation-panel")).toHaveTextContent("max_position");
    fireEvent.click(screen.getByTestId("runtime-mutation-panel-activate-parameter_mutation_1"));

    expect(onActivateProposal).toHaveBeenCalledWith(
      expect.objectContaining({ proposal_id: "parameter_mutation_1" }),
      {
        capability_context: capabilityContext,
        activation_boundary: {
          requested: "next_cycle_start",
          resolved_sequence_no: null
        }
      }
    );
  });

  it("fails closed when capability context is missing", () => {
    render(
      <RuntimeMutationPanel
        sourceKind="run"
        sourceId="run_1"
        initialMutations={[proposal()]}
        onActivateProposal={onActivateProposal}
      />
    );
    expect(screen.getByTestId("runtime-mutation-panel-boundary-lock")).toHaveTextContent(
      "能力上下文"
    );
    expect(
      screen.getByTestId("runtime-mutation-panel-activate-parameter_mutation_1")
    ).toBeDisabled();
  });

  it("shows active and pending activation state without raw json", () => {
    render(
      <RuntimeMutationPanel
        sourceKind="run"
        sourceId="run_1"
        capabilityContext={capabilityContext}
        initialMutations={[
          proposal({
            proposal_id: "parameter_mutation_pending",
            status: "activation_scheduled",
            activation_state: {
              requested_boundary: { requested: "manual_pause", resolved_sequence_no: null },
              resolved_sequence_no: null,
              scheduled_at_ms: 2,
              activated_at_ms: 0,
              active_parameter_version: null,
              failure_reason: null
            }
          }),
          proposal({
            proposal_id: "parameter_mutation_active",
            status: "activated",
            activation_state: {
              requested_boundary: { requested: "next_cycle_start", resolved_sequence_no: null },
              resolved_sequence_no: 4,
              scheduled_at_ms: 2,
              activated_at_ms: 3,
              active_parameter_version: "sha256:new",
              failure_reason: null
            }
          })
        ]}
      />
    );

    expect(screen.getByTestId("runtime-mutation-panel")).toHaveTextContent("待处理");
    expect(screen.getByTestId("runtime-mutation-panel")).toHaveTextContent("已生效");
    expect(screen.getByTestId("runtime-mutation-panel")).not.toHaveTextContent("{");
  });

  it("shows safe-window denial and emits rollback requests for activated proposals", () => {
    render(
      <RuntimeMutationPanel
        sourceKind="run"
        sourceId="run_1"
        capabilityContext={capabilityContext}
        initialMutations={[
          proposal({
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
          }),
          proposal({
            proposal_id: "parameter_mutation_active",
            status: "activated",
            activation_state: {
              requested_boundary: { requested: "next_cycle_start", resolved_sequence_no: null },
              resolved_sequence_no: 4,
              scheduled_at_ms: 2,
              activated_at_ms: 3,
              active_parameter_version: "sha256:new",
              failure_reason: null
            }
          })
        ]}
        onRollbackProposal={onRollbackProposal}
      />
    );

    expect(
      screen.getByTestId("runtime-mutation-panel-safe-window-parameter_mutation_denied")
    ).toHaveTextContent("SAFE_WINDOW_OPEN_ORDERS");
    fireEvent.click(screen.getByTestId("runtime-mutation-panel-rollback-parameter_mutation_active"));

    expect(onRollbackProposal).toHaveBeenCalledWith(
      expect.objectContaining({ proposal_id: "parameter_mutation_active" }),
      {
        capability_context: capabilityContext,
        activation_boundary: {
          requested: "next_cycle_start",
          resolved_sequence_no: null
        },
        target_parameter_version: "sha256:old"
      }
    );
  });
});
