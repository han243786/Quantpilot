import { test, expect } from "@playwright/test";
import { createApiMockHarness } from "./support/apiHarness";
import { backendCapabilitiesFixture } from "../../src/test/fixtures/capabilities/capabilityFallbacks";
import { installWorkspaceBootstrapMocks } from "./support/workspaceBootstrapMocks";

async function enterCurrentWorkspace(page) {
  await page.goto("/");
  await page.getByTestId("strategy-hub-open-current-workspace").click();
  await expect(page.locator(".top-toolbar--workspace")).toBeVisible();
}

async function openResearchMode(page) {
  await page.getByTestId("workspace-tab-research").click();
  await expect(page.getByTestId("strategy-workspace-research-tab")).toBeVisible();
}

async function mockHealthyCapabilities(api) {
  await api.json("**/api/capabilities", backendCapabilitiesFixture);
}

test("run simulation smoke uses the v4 runtime endpoint", async ({ page }) => {
  const api = await createApiMockHarness(page);
  let v4RunRequested = false;

  await mockHealthyCapabilities(api);
  await installWorkspaceBootstrapMocks(api);
  await api.handle("**/api/runtime/v4/run", async (route) => {
    const body = route.request().postDataJSON();
    expect(body.source).toContain("v4_strategy");
    expect(body.source).toContain("mode paper_simulated");
    expect(body.graph).toBeUndefined();
    v4RunRequested = true;

    await route.fulfill({
      status: 200,
      contentType: "application/json; charset=utf-8",
      body: JSON.stringify({
        run_id: "v4_run_smoke_001",
        graph_id: "strategy.v4.e2e_workspace",
        event_count: 2,
        output: {
          runtime_mode: "paper_simulated",
          events: [
            {
              sequence: 1,
              event_type: "market.tick",
              source: "runtime",
              ts_ms: 1_700_000_000_000,
              payload: { price: 10250 }
            },
            {
              sequence: 2,
              event_type: "risk.approved",
              source: "risk.guard",
              ts_ms: 1_700_000_001_000,
              payload: { approved: true }
            }
          ],
          memory_snapshot: {
            graph_id: "strategy.v4.e2e_workspace",
            runtime_mode: "paper_simulated",
            event_sequence: 2,
            provider_order_submission_attached: false
          },
          provider_order_submission_attached: false
        },
        handoff: {
          schema_version: "quantpilot/qs-v4-runtime-handoff-report/v1",
          accepted_for_runtime_handoff: true,
          graph_id: "strategy.v4.e2e_workspace",
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

  await enterCurrentWorkspace(page);
  await page.getByTestId("toolbar-start-runtime-action").click();

  await openResearchMode(page);
  await expect(page.locator(".event-summary-grid").first()).toContainText("2");
  await expect(page.getByTestId("event-feed-row-v4-2")).toBeVisible();
  expect(v4RunRequested).toBe(true);

  api.expectNoUnexpectedApiRequests();
});
