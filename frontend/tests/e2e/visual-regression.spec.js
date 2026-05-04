import { test, expect } from "@playwright/test";

test.describe("Visual Regression", () => {
  test("策略中心首页布局", async ({ page }) => {
    await page.goto("/strategies");
    await page.waitForTimeout(2000);
    await expect(page).toHaveScreenshot("strategy-hub.png", {
      maxDiffPixels: 5000,
      threshold: 0.1,
    });
  });

  test("告警页面", async ({ page }) => {
    await page.goto("/alerts");
    await page.waitForTimeout(1500);
    await expect(page).toHaveScreenshot("alerts.png", {
      maxDiffPixels: 3000,
      threshold: 0.1,
    });
  });

  test("快照页面", async ({ page }) => {
    await page.goto("/snapshots");
    await page.waitForTimeout(1500);
    await expect(page).toHaveScreenshot("snapshots.png", {
      maxDiffPixels: 3000,
      threshold: 0.1,
    });
  });

  test("故障手册页面", async ({ page }) => {
    await page.goto("/runbook");
    await page.waitForTimeout(1500);
    await expect(page).toHaveScreenshot("runbook.png", {
      maxDiffPixels: 3000,
      threshold: 0.1,
    });
  });
});
