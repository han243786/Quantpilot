import { test, expect } from "@playwright/test";
import { createApiMockHarness } from "./support/apiHarness";
import {
  backendCompileOkFixture,
  capabilityRejectionFixtures
} from "../../src/test/fixtures/runtime/capabilityRejections";
import {
  backendCapabilitiesFixture,
  capabilityFallbackFixtures
} from "../../src/test/fixtures/capabilities/capabilityFallbacks";
import { installWorkspaceBootstrapMocks } from "./support/workspaceBootstrapMocks";

async function enterCurrentWorkspace(page) {
  await page.goto("/");
  await page.getByTestId("strategy-hub-open-current-workspace").click();
  // v0.4.0: 默认 tab 为仪表盘, 切换到构建标签页获取工具栏
  await page.getByTestId("workspace-tab-code").click();
  await expect(page.locator(".top-toolbar--workspace")).toBeVisible();
}

async function openCodeMode(page) {
  await page.getByTestId("workspace-tab-code").click();
  await expect(page.locator(".module-card").first()).toBeVisible();
}

function toolbarErrorNotice(page, message) {
  return page.locator(".toolbar-notice-error").filter({ hasText: message }).first();
}

async function mockEditorBootstrap(api) {
  await installWorkspaceBootstrapMocks(api);
}

async function mockFormalCompileUnavailable(api) {
  await api.json(
    "**/api/quantscript/formal/compile",
    { error: "not_found", message: "formal compile unavailable in E2E mock" },
    404
  );
}

async function mockHealthyCapabilities(api) {
  await api.json("**/api/capabilities", backendCapabilitiesFixture);
}

test("editor boot uses backend capability fixture and stays out of fallback mode", async ({
  page
}) => {
  const api = await createApiMockHarness(page);
  await mockHealthyCapabilities(api);
  await mockEditorBootstrap(api);
  await mockFormalCompileUnavailable(api);
  await api.installGuard();

  await enterCurrentWorkspace(page);

  await expect(page.getByTestId("toolbar-capability-alert")).toHaveCount(0);
  await expect(page.getByTestId("toolbar-compile-action")).toBeEnabled();
  await expect(page.getByTestId("toolbar-start-runtime-action")).toBeEnabled();
  await expect(page.getByTestId("toolbar-start-backtest-action")).toBeEnabled();

  await openCodeMode(page);
  expect(await page.locator(".module-card").count()).toBeGreaterThanOrEqual(
    backendCapabilitiesFixture.frontend.supported_module_keys.length
  );

  api.expectNoUnexpectedApiRequests();
});

test("editor falls back to cached capabilities when capability fetch fails", async ({
  page
}) => {
  const api = await createApiMockHarness(page);

  await page.addInitScript(
    ([cacheKey, capabilities]) => {
      window.localStorage.setItem(cacheKey, JSON.stringify(capabilities));
    },
    [capabilityFallbackFixtures.cacheKey, backendCapabilitiesFixture]
  );

  await api.text(
    "**/api/capabilities",
    capabilityFallbackFixtures.serviceUnavailableHttp503.body,
    capabilityFallbackFixtures.serviceUnavailableHttp503.status,
    capabilityFallbackFixtures.serviceUnavailableHttp503.contentType
  );
  await mockEditorBootstrap(api);
  await mockFormalCompileUnavailable(api);
  await api.installGuard();

  await enterCurrentWorkspace(page);

  await expect(page.getByTestId("toolbar-capability-alert")).toContainText(
    /latest cached capability snapshot/i
  );
  await expect(page.getByTestId("toolbar-compile-action")).toBeEnabled();
  await expect(page.getByTestId("toolbar-export-runtime-config-action")).toBeEnabled();
  await expect(page.getByTestId("toolbar-start-runtime-action")).toBeEnabled();
  await expect(page.getByTestId("toolbar-start-backtest-action")).toBeEnabled();

  api.expectNoUnexpectedApiRequests();
});

test("editor enters safe fallback mode when capability fetch fails without cache", async ({
  page
}) => {
  const api = await createApiMockHarness(page);

  await api.text(
    "**/api/capabilities",
    capabilityFallbackFixtures.serviceUnavailableHttp503.body,
    capabilityFallbackFixtures.serviceUnavailableHttp503.status,
    capabilityFallbackFixtures.serviceUnavailableHttp503.contentType
  );
  await mockEditorBootstrap(api);
  await mockFormalCompileUnavailable(api);
  await api.installGuard();

  await enterCurrentWorkspace(page);

  await expect(page.getByTestId("toolbar-capability-alert")).toContainText(
    /to avoid exposing fake capabilities/i
  );
  await expect(page.getByTestId("toolbar-compile-action")).toBeDisabled();
  await expect(page.getByTestId("toolbar-export-runtime-config-action")).toBeDisabled();
  await expect(page.getByTestId("toolbar-start-runtime-action")).toBeDisabled();
  await expect(page.getByTestId("toolbar-start-backtest-action")).toBeDisabled();

  api.expectNoUnexpectedApiRequests();
});

test("compile surfaces backend structured capability rejection", async ({ page }) => {
  const api = await createApiMockHarness(page);
  await mockHealthyCapabilities(api);
  await mockEditorBootstrap(api);
  await mockFormalCompileUnavailable(api);
  await api.json(
    "**/api/runtime/compile",
    capabilityRejectionFixtures.compileExecutionModuleUnsupported,
    400
  );
  await api.installGuard();

  await enterCurrentWorkspace(page);
  await page.getByTestId("toolbar-compile-action").click();

  await expect(
    toolbarErrorNotice(
      page,
      capabilityRejectionFixtures.compileExecutionModuleUnsupported.message
    )
  ).toBeVisible();
  await expect(page.locator(".top-toolbar--workspace")).toContainText("运行时：空闲");

  api.expectNoUnexpectedApiRequests();
});

test("simulation start surfaces backend structured capability rejection", async ({ page }) => {
  const api = await createApiMockHarness(page);
  await mockHealthyCapabilities(api);
  await mockEditorBootstrap(api);
  await mockFormalCompileUnavailable(api);
  await api.json("**/api/runtime/compile", backendCompileOkFixture);
  await api.json("**/api/runtime/test-run", capabilityRejectionFixtures.runtimeModeUnsupported, 400);
  await api.installGuard();

  await enterCurrentWorkspace(page);
  await page.getByTestId("toolbar-start-runtime-action").click();

  await expect(
    toolbarErrorNotice(
      page,
      capabilityRejectionFixtures.runtimeModeUnsupported.message
    )
  ).toBeVisible();
  await expect(page.locator(".top-toolbar--workspace")).toContainText("运行时：错误");

  api.expectNoUnexpectedApiRequests();
});

test("backtest start surfaces backend structured capability rejection", async ({ page }) => {
  const api = await createApiMockHarness(page);
  await mockHealthyCapabilities(api);
  await mockEditorBootstrap(api);
  await mockFormalCompileUnavailable(api);
  await api.json("**/api/runtime/compile", backendCompileOkFixture);
  await api.handle("**/api/runtime/backtest", async (route) => {
    const body = route.request().postDataJSON();
    expect(body.backtest_options?.replay_source).toBe("deterministic_mock");

    await route.fulfill({
      status: 400,
      contentType: "application/json; charset=utf-8",
      body: JSON.stringify(capabilityRejectionFixtures.symbolUnsupported)
    });
  });
  await api.installGuard();

  await enterCurrentWorkspace(page);
  await page.getByTestId("toolbar-start-backtest-action").click();

  await expect(
    toolbarErrorNotice(
      page,
      capabilityRejectionFixtures.symbolUnsupported.message
    )
  ).toBeVisible();
  await expect(page.locator(".top-toolbar--workspace")).toContainText("运行时：错误");

  api.expectNoUnexpectedApiRequests();
});
