import { test, expect } from "@playwright/test";
import { createApiMockHarness } from "./support/apiHarness";
import { installWorkspaceBootstrapMocks } from "./support/workspaceBootstrapMocks";
import { backendCapabilitiesFixture } from "../../src/test/fixtures/capabilities/capabilityFallbacks";
import { buildRunSuccessFixture } from "../../src/test/fixtures/runtime/runSuccess";

const RUN_ID = "run_mutation_walkthrough_001";

function mutation(overrides = {}) {
  return {
    proposal_id: "parameter_mutation_proposed_001",
    source_kind: "run",
    source_id: RUN_ID,
    graph_id: "draft_graph",
    target: {
      node_id: "risk_risk_1",
      module_key: "builtin.risk.global",
      parameter_path: "max_position"
    },
    old_value: 0.2,
    new_value: 0.15,
    old_parameter_version: "sha256:old-parameter-version",
    proposed_parameter_version: "sha256:new-parameter-version",
    status: "proposed",
    activation_boundary: {
      requested: "next_cycle_start",
      resolved_sequence_no: null
    },
    actor: {
      actor_id: "operator_1",
      display_name: "Operator 1"
    },
    reason: "Reduce exposure before a volatile window.",
    governance: {
      capability_hash: "sha256:capability",
      deployment_revision: "sha256:deployment",
      strategy_version: "strategy-v1",
      previous_parameter_version: "sha256:old-parameter-version",
      proposed_parameter_version: "sha256:new-parameter-version",
      permission_boundary_model_version: "quantpilot/permission-boundary/v1"
    },
    lifecycle: [],
    created_at_ms: 1_710_000_000_000,
    updated_at_ms: 1_710_000_000_000,
    ...overrides
  };
}

function mutationWalkthroughRecords() {
  return [
    mutation(),
    mutation({
      proposal_id: "parameter_mutation_safe_window_denied_001",
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
          data_freshness_ms: 100,
          portfolio_exposure_bps: 100,
          cooldown_remaining_ms: 5000
        }
      }
    }),
    mutation({
      proposal_id: "parameter_mutation_activated_001",
      status: "activated",
      activation_state: {
        requested_boundary: { requested: "next_cycle_start", resolved_sequence_no: null },
        resolved_sequence_no: 8,
        scheduled_at_ms: 1_710_000_000_100,
        activated_at_ms: 1_710_000_000_101,
        active_parameter_version: "sha256:new-parameter-version",
        failure_reason: null
      }
    }),
    mutation({
      proposal_id: "parameter_mutation_rollback_001",
      status: "rolled_back",
      proposed_parameter_version: "sha256:old-parameter-version",
      rollback_of: "parameter_mutation_activated_001",
      rollback_target_parameter_version: "sha256:old-parameter-version",
      activation_state: {
        requested_boundary: { requested: "next_cycle_start", resolved_sequence_no: null },
        resolved_sequence_no: 10,
        scheduled_at_ms: 1_710_000_000_200,
        activated_at_ms: 1_710_000_000_201,
        active_parameter_version: "sha256:old-parameter-version",
        failure_reason: null
      }
    })
  ];
}

async function enterCurrentWorkspace(page) {
  await page.goto("/");
  await page.getByTestId("strategy-hub-open-current-workspace").click();
  await expect(page.locator(".top-toolbar--workspace")).toBeVisible();
}

test("runtime mutation walkthrough shows proposal, safe window, activation, and rollback states", async ({ page }) => {
  const api = await createApiMockHarness(page);
  const runFixture = buildRunSuccessFixture({
    graphId: "draft_graph",
    compileId: "compile_mutation_walkthrough_001",
    runId: RUN_ID
  });

  await api.json("**/api/capabilities", backendCapabilitiesFixture);
  await installWorkspaceBootstrapMocks(api, {
    runHistory: runFixture.historyResponse
  });
  await api.json(`**/api/runtime/runs/${RUN_ID}`, runFixture.detailResponse);
  await api.json("**/api/runtime/mutations**", mutationWalkthroughRecords());
  await api.installGuard();

  await enterCurrentWorkspace(page);
  await page.getByTestId("workspace-tab-research").click();
  await page.getByTestId("research-tab-runs").click();
  await page
    .getByTestId("run-history-card")
    .getByRole("button", { name: new RegExp(RUN_ID) })
    .click();

  const panel = page.getByTestId("run-history-mutation-panel");
  await expect(panel).toContainText("参数变更");
  await expect(panel).toContainText("max_position");
  await expect(panel).toContainText("SAFE_WINDOW_OPEN_ORDERS");
  await expect(panel).toContainText("rolled_back");
  await expect(panel).not.toContainText("\"proposal_id\"");
  await expect(page.getByTestId("run-history-mutation-panel-rollback-parameter_mutation_activated_001")).toBeDisabled();

  api.expectNoUnexpectedApiRequests();
});
