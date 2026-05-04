import { test, expect } from "@playwright/test";
import { createApiMockHarness } from "./support/apiHarness";
import { backendCompileOkFixture } from "../../src/test/fixtures/runtime/capabilityRejections";
import { buildBacktestSuccessFixture } from "../../src/test/fixtures/runtime/backtestSuccess";
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

async function mockFormalCompile(api) {
  await api.json("**/api/quantscript/formal/compile", backendCompileOkFixture);
}

test("run backtest smoke covers start, history refresh, and detail page", async ({ page }) => {
  let backtestFixture = null;
  let backtestHistory = [];
  const api = await createApiMockHarness(page);

  await mockHealthyCapabilities(api);
  await mockFormalCompile(api);
  await installWorkspaceBootstrapMocks(api);
  await api.json("**/api/runtime/compile", backendCompileOkFixture);

  await api.handle("**/api/runtime/backtests/*", async (route) => {
    if (!backtestFixture) {
      await route.fulfill({
        status: 404,
        contentType: "application/json; charset=utf-8",
        body: JSON.stringify({ error: "not_found", message: "backtest fixture missing" })
      });
      return;
    }

    await route.fulfill({
      status: 200,
      contentType: "application/json; charset=utf-8",
      body: JSON.stringify(backtestFixture.detailResponse)
    });
  });

  await api.handle("**/api/runtime/backtests/*/save", async (route) => {
    if (!backtestFixture) {
      await route.fulfill({
        status: 404,
        contentType: "application/json; charset=utf-8",
        body: JSON.stringify({ error: "not_found", message: "backtest fixture missing" })
      });
      return;
    }

    await route.fulfill({
      status: 200,
      contentType: "application/json; charset=utf-8",
      body: JSON.stringify({
        backtest_id: backtestFixture.startResponse.backtest_id,
        saved: true
      })
    });
  });

  await api.handle("**/api/runtime/backtests", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json; charset=utf-8",
      body: JSON.stringify(backtestHistory)
    });
  });

  await api.handle("**/api/runtime/backtest", async (route) => {
    const body = route.request().postDataJSON();
    expect(body.backtest_options?.replay_source).toBe("deterministic_mock");

    backtestFixture = buildBacktestSuccessFixture({
      graphId: body.runtime_config.metadata.graph_id,
      compileId: body.runtime_config.metadata.compile_id
    });
    backtestHistory = backtestFixture.historyResponse;

    await route.fulfill({
      status: 200,
      contentType: "application/json; charset=utf-8",
      body: JSON.stringify(backtestFixture.startResponse)
    });
  });
  await api.installGuard();

  await enterCurrentWorkspace(page);
  await page.getByTestId("toolbar-start-backtest-action").click();

  await openResearchMode(page);
  await page.getByTestId("research-tab-backtests").click();
  await expect(page.locator(".event-summary-grid")).toContainText("backtest_smoke_001");
  await expect(page.locator(".event-summary-grid")).toContainText("3");
  await expect(page.getByTestId("runtime-artifact-save")).toBeVisible();
  await Promise.all([
    page.waitForResponse((response) =>
      response.url().includes("/api/runtime/backtests/backtest_smoke_001/save")
    ),
    page.getByTestId("runtime-artifact-save").click()
  ]);
  await expect(page.getByTestId("backtest-history-card")).toContainText("backtest_smoke_001");
  await expect(page.getByTestId("account-summary-equity")).toContainText("12050");

  await page
    .getByTestId("backtest-history-card")
    .getByRole("button", { name: /backtest_smoke_001/ })
    .click();

  await expect(page).toHaveURL(/\/backtests\/backtest_smoke_001(\?strategy=draft_graph)?$/);
  await expect(page.locator(".detail-page")).toBeVisible();
  await expect(page.getByTestId("backtest-detail-hero")).toContainText("quantpilot/runtime-config/v1");
  await expect(page.getByTestId("backtest-detail-hero")).toContainText("smoke_backtest_config_hash");
  await expect(page.getByTestId("backtest-detail-hero")).toContainText("+12.50%");
  await expect(page.getByTestId("backtest-detail-hero")).toContainText("12050");

  api.expectNoUnexpectedApiRequests();
});
