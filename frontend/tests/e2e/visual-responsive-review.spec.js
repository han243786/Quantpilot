import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test, expect } from "@playwright/test";
import { installAnalysisReviewMocks } from "./support/analysisReviewFixtures";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "../../..");
const outputDir = path.join(repoRoot, "markdown", "visual-review", "p2-responsive");

const viewports = [
  { width: 1280, height: 900, label: "1280" },
  { width: 1024, height: 860, label: "1024" },
  { width: 768, height: 1180, label: "768" },
  { width: 560, height: 1280, label: "560" }
];

test.describe("visual responsive review", () => {
  test.skip(
    !process.env.VISUAL_REVIEW,
    "Set VISUAL_REVIEW=1 to generate responsive review screenshots."
  );

  test("capture editor and backtest pages at 1280 / 1024 / 768 / 560", async ({ page }) => {
    fs.mkdirSync(outputDir, { recursive: true });
    const { api } = await installAnalysisReviewMocks(page);

    const pages = [
      {
        name: "editor",
        url: "/",
        ready: async () => {
          await expect(page.locator(".editor-page")).toBeVisible();
          await expect(page.locator(".main-workspace")).toBeVisible();
        }
      },
      {
        name: "backtest-detail",
        url: "/backtests/backtest_smoke_001",
        ready: async () => {
          await expect(page.locator(".detail-page")).toBeVisible();
          await expect(page.locator(".analysis-summary-grid")).toBeVisible();
        }
      },
      {
        name: "backtest-compare",
        url: "/backtests/compare?ids=backtest_smoke_001,backtest_compare_002",
        ready: async () => {
          await expect(page.locator(".detail-page")).toBeVisible();
          await expect(page.locator(".analysis-card-grid")).toBeVisible();
        }
      }
    ];

    for (const pageConfig of pages) {
      for (const viewport of viewports) {
        await page.setViewportSize({ width: viewport.width, height: viewport.height });
        await page.goto(pageConfig.url);
        await pageConfig.ready();
        await page.screenshot({
          path: path.join(outputDir, `${pageConfig.name}-${viewport.label}.png`),
          fullPage: true
        });
      }
    }

    api.expectNoUnexpectedApiRequests();
  });
});
