import { test, expect } from "@playwright/test";
import { createApiMockHarness } from "./support/apiHarness";
import { backendCompileOkFixture } from "../../src/test/fixtures/runtime/capabilityRejections";
import { buildRunSuccessFixture } from "../../src/test/fixtures/runtime/runSuccess";
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

async function openRunsPrimaryMode(page) {
  await page.getByTestId("research-tab-runs").click();
}

async function mockHealthyCapabilities(api) {
  await api.json("**/api/capabilities", backendCapabilitiesFixture);
}

async function mockFormalCompile(api) {
  await api.json("**/api/quantscript/formal/compile", backendCompileOkFixture);
}

test("run simulation smoke covers start, SSE, and history refresh", async ({ page }) => {
  let runFixture = null;
  let runHistory = [];
  const api = await createApiMockHarness(page);

  await mockHealthyCapabilities(api);
  await mockFormalCompile(api);
  await installWorkspaceBootstrapMocks(api);
  await api.json("**/api/runtime/compile", backendCompileOkFixture);

  await api.handle("**/api/runtime/runs/*/events", async (route) => {
    if (!runFixture) {
      await route.fulfill({
        status: 404,
        contentType: "text/plain; charset=utf-8",
        body: "missing run fixture"
      });
      return;
    }

    await route.fulfill({
      status: 200,
      contentType: "text/event-stream; charset=utf-8",
      body: runFixture.sseBody
    });
  });

  await api.handle("**/api/runtime/runs/*", async (route) => {
    if (!runFixture) {
      await route.fulfill({
        status: 404,
        contentType: "application/json; charset=utf-8",
        body: JSON.stringify({ error: "not_found", message: "run fixture missing" })
      });
      return;
    }

    await route.fulfill({
      status: 200,
      contentType: "application/json; charset=utf-8",
      body: JSON.stringify(runFixture.detailResponse)
    });
  });

  await api.handle("**/api/runtime/runs", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json; charset=utf-8",
      body: JSON.stringify(runHistory)
    });
  });

  await api.handle("**/api/runtime/test-run", async (route) => {
    const body = route.request().postDataJSON();
    runFixture = buildRunSuccessFixture({
      graphId: body.runtime_config.metadata.graph_id,
      compileId: body.runtime_config.metadata.compile_id
    });
    runHistory = runFixture.historyResponse;

    await route.fulfill({
      status: 200,
      contentType: "application/json; charset=utf-8",
      body: JSON.stringify(runFixture.startResponse)
    });
  });
  await api.installGuard();

  await enterCurrentWorkspace(page);
  await page.getByTestId("toolbar-start-runtime-action").click();

  await openResearchMode(page);
  await openRunsPrimaryMode(page);
  await expect(page.locator(".event-summary-grid")).toContainText("run_smoke_001");
  await expect(page.locator(".event-summary-grid")).toContainText("4");
  await expect(page.getByTestId("event-feed-row-evt_run_exec_1")).toBeVisible();
  await expect(page.getByTestId("run-history-card")).toContainText("run_smoke_001");

  await page
    .getByTestId("run-history-card")
    .getByRole("button", { name: /run_smoke_001/ })
    .click();

  await expect(page.getByTestId("asset-candles-panel")).toContainText("10,250.00");
  await expect(page.locator(".event-summary-grid")).toContainText("4");

  api.expectNoUnexpectedApiRequests();
});
