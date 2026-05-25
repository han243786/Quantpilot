import { test, expect } from "@playwright/test";
import { createApiMockHarness } from "./support/apiHarness";
import { backendCompileOkFixture } from "../../src/test/fixtures/runtime/capabilityRejections";
import { buildBacktestSuccessFixture } from "../../src/test/fixtures/runtime/backtestSuccess";
import { backendCapabilitiesFixture } from "../../src/test/fixtures/capabilities/capabilityFallbacks";
import { installWorkspaceBootstrapMocks } from "./support/workspaceBootstrapMocks";

const VALID_V4_SOURCE = `
v4_strategy strategy.v4.e2e {
  venue paper-local
  mode paper_simulated
  require capability market

  machine data.market observation priority 8000 {
    state idle initial
    state ready
    memory last_signal_at: time nullable
    on market.tick from idle to ready emit bar_closed write last_signal_at
  }

  machine risk.guard decision priority 9500 {
    state idle initial
    state ready
    memory last_signal_at: time nullable
    on bar_closed from idle to ready emit risk.approved write last_signal_at
  }

  machine execution.router execution priority 4000 {
    state idle initial
    state ready
    memory last_signal_at: time nullable
    on risk.approved from idle to ready write last_signal_at
  }

  edge data.market -> risk.guard on bar_closed
  edge risk.guard -> execution.router on risk.approved
  risk_plane risk.guard priority 9000
}
`;

async function enterCurrentWorkspace(page) {
  await page.goto("/");
  await page.getByTestId("strategy-hub-open-current-workspace").click();
  await expect(page.locator(".top-toolbar--workspace")).toBeVisible();
}

async function mockHealthyCapabilities(api) {
  await api.json("**/api/capabilities", backendCapabilitiesFixture);
}

async function mockFormalCompileUnavailable(api) {
  await api.json(
    "**/api/quantscript/formal/compile",
    { error: "not_found", message: "formal compile unavailable in E2E mock" },
    404
  );
}

test("auth capability failure enters safe fallback and locks write actions", async ({ page }) => {
  const api = await createApiMockHarness(page);
  await api.json("**/api/capabilities", { error: "unauthorized", message: "auth required" }, 401);
  await installWorkspaceBootstrapMocks(api);
  await mockFormalCompileUnavailable(api);
  await api.installGuard();

  await enterCurrentWorkspace(page);

  await expect(page.getByTestId("toolbar-capability-alert")).toBeVisible();
  await expect(page.getByTestId("workspace-tab-code")).toBeDisabled();
  await expect(page.getByTestId("workspace-tab-research")).toBeDisabled();
  await expect(page.getByTestId("toolbar-compile-action")).toBeDisabled();
  await expect(page.getByTestId("toolbar-start-runtime-action")).toBeDisabled();
  await expect(page.getByTestId("toolbar-start-backtest-action")).toBeDisabled();

  api.expectNoUnexpectedApiRequests();
});

test("v4 strategy browser contract posts PaperSimulated source to runtime endpoint", async ({
  page
}) => {
  const api = await createApiMockHarness(page);
  await mockHealthyCapabilities(api);
  await installWorkspaceBootstrapMocks(api);
  await mockFormalCompileUnavailable(api);
  await api.handle("**/api/runtime/v4/run", async (route) => {
    const body = route.request().postDataJSON();
    expect(body.source).toContain("mode paper_simulated");
    expect(body.source).toContain("require capability market");
    expect(body.graph).toBeUndefined();

    await route.fulfill({
      status: 200,
      contentType: "application/json; charset=utf-8",
      body: JSON.stringify({
        run_id: "v4_run_e2e_strategy",
        graph_id: "strategy.v4.e2e",
        event_count: 2,
        output: {
          runtime_mode: "paper_simulated",
          events: [{ sequence: 1, event_type: "market.tick", source: "runtime", ts_ms: 1 }],
          memory_snapshot: {
            graph_id: "strategy.v4.e2e",
            runtime_mode: "paper_simulated",
            event_sequence: 2,
            provider_order_submission_attached: false
          },
          provider_order_submission_attached: false
        },
        handoff: {
          schema_version: "quantpilot/qs-v4-runtime-handoff-report/v1",
          accepted_for_runtime_handoff: true,
          graph_id: "strategy.v4.e2e",
          venue_id: "paper-local",
          runtime_mode: "paper_simulated",
          paper_simulated_start_allowed: true,
          provider_order_submission_attached: false,
          runtime_attached: false,
          lowering_attached: false,
          diagnostics: []
        },
        diagnostics: []
      })
    });
  });
  await api.installGuard();

  await page.goto("/");
  const result = await page.evaluate(async (source) => {
    const response = await fetch("/api/runtime/v4/run", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ source })
    });
    return { status: response.status, body: await response.json() };
  }, VALID_V4_SOURCE);

  expect(result.status).toBe(200);
  expect(result.body.handoff.accepted_for_runtime_handoff).toBe(true);
  expect(result.body.output.provider_order_submission_attached).toBe(false);
  api.expectNoUnexpectedApiRequests();
});

test("v4 backtest browser contract keeps deterministic replay and v4 artifact surface", async ({
  page
}) => {
  const api = await createApiMockHarness(page);
  const fixture = buildBacktestSuccessFixture({
    graphId: "strategy.v4.e2e",
    compileId: "compile_v4_e2e",
    backtestId: "backtest_v4_e2e"
  });
  await mockHealthyCapabilities(api);
  await installWorkspaceBootstrapMocks(api);
  await api.json("**/api/runtime/compile", backendCompileOkFixture);
  await mockFormalCompileUnavailable(api);
  await api.handle("**/api/runtime/backtest", async (route) => {
    const body = route.request().postDataJSON();
    expect(body.runtime_kind).toBe("v4");
    expect(body.backtest_options?.replay_source).toBe("deterministic_mock");
    expect(body.runtime_config?.metadata?.graph_id).toBe("strategy.v4.e2e");

    await route.fulfill({
      status: 200,
      contentType: "application/json; charset=utf-8",
      body: JSON.stringify({
        ...fixture.startResponse,
        runtime_kind: "v4",
        v4_artifact: {
          schema_version: "quantpilot/v4-backtest-artifact/v1",
          provider_order_submission_attached: false,
          replay_source: "deterministic_mock"
        }
      })
    });
  });
  await api.installGuard();

  await page.goto("/");
  const result = await page.evaluate(async () => {
    const response = await fetch("/api/runtime/backtest", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        runtime_kind: "v4",
        runtime_config: {
          schema_version: "quantpilot/runtime-config/v1",
          metadata: {
            graph_id: "strategy.v4.e2e",
            compile_id: "compile_v4_e2e"
          }
        },
        backtest_options: {
          replay_source: "deterministic_mock"
        }
      })
    });
    return { status: response.status, body: await response.json() };
  });

  expect(result.status).toBe(200);
  expect(result.body.runtime_kind).toBe("v4");
  expect(result.body.v4_artifact.provider_order_submission_attached).toBe(false);
  api.expectNoUnexpectedApiRequests();
});
