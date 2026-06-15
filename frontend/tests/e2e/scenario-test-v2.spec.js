import { test, expect } from "@playwright/test";

async function enterWorkspaceCodeTab(page) {
  await page.goto("/strategies");
  await page.getByTestId("strategy-hub-open-current-workspace").click();
  await expect(page.getByTestId("workspace-tab-code")).toBeVisible();
  await page.getByTestId("workspace-tab-code").click();
  await expect(page.locator("[data-testid^='module-card-']").first()).toBeVisible();
}

async function createModuleNode(page, moduleKey) {
  const before = await page.evaluate(() => window.__QUANTPILOT_TEST__?.getNodeCount() ?? 0);
  await page.getByTestId(`module-card-${moduleKey}`).last().click();
  await page.waitForFunction(
    (expected) => (window.__QUANTPILOT_TEST__?.getNodeCount() ?? 0) > expected,
    before
  );
}

async function getNodes(page) {
  return page.evaluate(() => {
    const state = window.__QUANTPILOT_TEST__?.getRawState();
    return state?.graph?.nodes?.map((node) => ({
      id: node.id,
      type: node.type,
      module_key: node.module_key
    })) ?? [];
  });
}

test.describe("Scenario 6: canvas interaction depth", () => {
  test("creates nodes, exposes handles, selects, deletes, recovers, zooms, and searches modules", async ({ page }) => {
    await enterWorkspaceCodeTab(page);

    await expect(page.locator("[data-testid^='module-card-']")).not.toHaveCount(0);
    await createModuleNode(page, "builtin.runtime.control");
    await createModuleNode(page, "builtin.data.kline");
    await createModuleNode(page, "builtin.intent.double_ma");
    await expect(page.locator(".react-flow__node")).toHaveCount(3);

    const targetHandles = page.locator("[data-testid^='handle-target-']");
    const sourceHandles = page.locator("[data-testid^='handle-source-']");
    await expect(targetHandles.first()).toBeVisible();
    await expect(sourceHandles.first()).toBeVisible();

    await page.locator(".react-flow__node").first().click({ force: true });
    await expect(page.locator("[data-testid^='prop-input-']").first()).toBeVisible();

    const beforeDelete = await page.evaluate(() => window.__QUANTPILOT_TEST__?.getNodeCount() ?? 0);
    await page.locator(".react-flow__node").last().click({ force: true });
    await page.getByTestId("prop-action-delete-node").click();
    await page.waitForFunction(
      (expected) => (window.__QUANTPILOT_TEST__?.getNodeCount() ?? 0) < expected,
      beforeDelete
    );
    await createModuleNode(page, "builtin.intent.rsi");
    await page.waitForFunction(
      (expected) => (window.__QUANTPILOT_TEST__?.getNodeCount() ?? 0) === expected,
      beforeDelete
    );

    const pane = page.locator(".react-flow__pane").first();
    const box = await pane.boundingBox();
    expect(box).toBeTruthy();
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.wheel(0, -120);
    await page.mouse.wheel(0, 120);

    await page.getByTestId("module-sidebar-search").fill("RSI");
    await expect(page.getByTestId("module-card-builtin.intent.rsi").first()).toBeVisible();
    await page.getByTestId("module-sidebar-search").fill("");
    await expect(page.getByTestId("module-card-builtin.data.kline").first()).toBeVisible();
  });
});

test.describe("Scenario 7: strategy workspace tabs", () => {
  test("switches all workspace tabs through the test bridge", async ({ page }) => {
    await page.goto("/strategies");
    await page.getByTestId("strategy-hub-open-current-workspace").click();

    for (const tab of ["dashboard", "code", "research", "monitor", "source"]) {
      await page.getByTestId(`workspace-tab-${tab}`).click();
      await expect
        .poll(() => page.evaluate(() => window.__QUANTPILOT_TEST__?.getActiveTab()))
        .toBe(tab);
    }
  });
});

test.describe("Scenario 10: i18n and responsive layout", () => {
  test("loads Chinese UI, test bridge, layout snapshot, and compact viewport", async ({ page }) => {
    await page.goto("/strategies");
    await expect(page.locator("body")).toContainText("QuantPilot");
    await expect(page.locator("body")).toContainText("策略");
    await expect(page.locator(".strategy-hub-hero")).toBeVisible();

    await expect
      .poll(() => page.evaluate(() => typeof window.__QUANTPILOT_TEST__ === "object"))
      .toBe(true);
    await expect
      .poll(() => page.evaluate(() => window.__QUANTPILOT_TEST__.getCurrentRoute().name))
      .toBe("strategies");

    const visibleAreas = await page.evaluate(() => {
      const layout = window.__QUANTPILOT_TEST__.getLayoutSnapshot();
      return Object.keys(layout).filter((key) => layout[key].visible);
    });
    expect(visibleAreas.length).toBeGreaterThan(0);

    await page.setViewportSize({ width: 800, height: 600 });
    await expect
      .poll(() => page.evaluate(() => Boolean(window.__QUANTPILOT_TEST__.getLayoutSnapshot())))
      .toBe(true);
    await page.setViewportSize({ width: 1440, height: 900 });
  });
});

test.describe("Scenario 5: API-backed utility pages", () => {
  test("loads alerts, runbook, snapshots, chaos, and approvals", async ({ page }) => {
    await page.goto("/alerts");
    await expect(page.locator("body")).toContainText("告警");
    const alerts = await page.request.get("http://127.0.0.1:3000/api/v1/alerts/rules");
    expect(alerts.ok()).toBe(true);

    const runbook = await page.request.get("http://127.0.0.1:3000/api/v1/runbook");
    expect(runbook.ok()).toBe(true);

    for (const route of ["/snapshots", "/chaos", "/approvals"]) {
      await page.goto(route);
      await expect(page.locator("body")).not.toHaveText("");
    }
  });
});

test.describe("Handle connection and property editing", () => {
  test("creates data and intent nodes, connects handles, and opens properties", async ({ page }) => {
    await enterWorkspaceCodeTab(page);
    await createModuleNode(page, "builtin.data.kline");
    await createModuleNode(page, "builtin.intent.double_ma");

    const nodes = await getNodes(page);
    const dataNode = nodes.find((node) => node.type === "data");
    const intentNode = nodes.find((node) => node.type === "intent");
    expect(dataNode).toBeTruthy();
    expect(intentNode).toBeTruthy();

    await page.locator(`[data-node-card-id='${dataNode.id}']`).click({ force: true });
    await page.locator(`[data-node-card-id='${intentNode.id}']`).click({ force: true });

    const srcHandle = page.locator(`[data-testid^='handle-source-${dataNode.id}-']`).first();
    const tgtHandle = page.locator(`[data-testid^='handle-target-${intentNode.id}-']`).first();
    await expect(srcHandle).toBeVisible();
    await expect(tgtHandle).toBeVisible();

    await srcHandle.dragTo(tgtHandle);
    await expect
      .poll(() => page.evaluate(() => window.__QUANTPILOT_TEST__?.getEdgeCount() ?? 0))
      .toBeGreaterThan(0);

    await page.locator(".react-flow__node-data").first().click({ force: true });
    await expect(page.locator("[data-testid^='prop-input-']").first()).toBeVisible();
  });
});
