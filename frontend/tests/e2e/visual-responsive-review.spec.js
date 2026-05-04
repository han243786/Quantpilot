import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test, expect } from "@playwright/test";
import { installAnalysisReviewMocks, REVIEW_GRAPH_ID } from "./support/analysisReviewFixtures";

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

  test("capture strategy hub, workspace, and backtest pages at 1280 / 1024 / 768 / 560", async ({ browser }) => {
    fs.mkdirSync(outputDir, { recursive: true });
    const freezeMotion = (page) =>
      page.addStyleTag({
        content: `
          *,
          *::before,
          *::after {
            animation-delay: 0s !important;
            animation-duration: 0s !important;
            transition-delay: 0s !important;
            transition-duration: 0s !important;
          }
        `
      });

    const pages = [
      {
        name: "strategy-hub",
        url: "/strategies",
        ready: async (page) => {
          await expect(page.getByTestId("strategy-hub-page")).toBeVisible();
          await expect(page.locator(".strategy-hub-status-strip")).toBeVisible();
        }
      },
      {
        name: "strategy-workspace",
        url: `/strategies/${REVIEW_GRAPH_ID}`,
        ready: async (page) => {
          await expect(page.locator(".strategy-workspace-page")).toBeVisible();
          await expect(page.getByTestId("strategy-workspace-overview-tab")).toBeVisible();
        }
      },
      {
        name: "backtest-detail",
        url: "/backtests/backtest_smoke_001",
        ready: async (page) => {
          await expect(page.locator(".detail-page")).toBeVisible();
          await expect(page.locator(".analysis-summary-grid")).toBeVisible();
        }
      },
      {
        name: "backtest-compare",
        url: "/backtests/compare?ids=backtest_smoke_001,backtest_compare_002",
        ready: async (page) => {
          await expect(page.locator(".detail-page")).toBeVisible();
          await expect(page.locator(".analysis-card-grid")).toBeVisible();
        }
      }
    ];

    for (const pageConfig of pages) {
      for (const viewport of viewports) {
        const context = await browser.newContext({
          viewport: { width: viewport.width, height: viewport.height },
          reducedMotion: "reduce"
        });
        const page = await context.newPage();
        const { api } = await installAnalysisReviewMocks(page);
        await page.goto(pageConfig.url);
        await freezeMotion(page);
        await pageConfig.ready(page);
        await page.waitForLoadState("networkidle");
        await page.screenshot({
          path: path.join(outputDir, `${pageConfig.name}-${viewport.label}.png`),
          fullPage: true
        });
        api.expectNoUnexpectedApiRequests();
        await context.close();
      }
    }
  });
});
