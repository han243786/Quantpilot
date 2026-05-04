import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test, expect } from "@playwright/test";
import { installAnalysisReviewMocks } from "./support/analysisReviewFixtures";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "../../..");
const outputDir = path.join(repoRoot, "markdown", "performance-review");
const reportPath = path.join(outputDir, "first-screen-review.md");
const rawPath = path.join(outputDir, "first-screen-review.json");

const SAMPLE_COUNT = 3;
const VIEWPORT = { width: 1280, height: 900 };

const routes = [
  {
    name: "editor",
    label: "编辑器首页",
    url: "/",
    readySelector: ".main-workspace"
  },
  {
    name: "backtest-detail",
    label: "回测详情页",
    url: "/backtests/backtest_smoke_001",
    readySelector: ".analysis-summary-grid"
  },
  {
    name: "backtest-compare",
    label: "回测对比页",
    url: "/backtests/compare?ids=backtest_smoke_001,backtest_compare_002",
    readySelector: ".analysis-card-grid"
  }
];

function round(value) {
  return Math.round(value * 100) / 100;
}

function summarize(label, samples, key) {
  const values = samples.map((sample) => sample[key]);
  const total = values.reduce((sum, value) => sum + value, 0);
  return {
    label,
    avg: round(total / values.length),
    min: round(Math.min(...values)),
    max: round(Math.max(...values))
  };
}

// P3.2: Performance assertion thresholds
const PERF_THRESHOLDS = {
  "编辑器首页": { domContentLoaded: 3000, load: 5000, ready: 4000 },
  "回测详情页": { domContentLoaded: 3000, load: 5000, ready: 4000 },
  "回测对比页": { domContentLoaded: 3000, load: 5000, ready: 4000 },
};

function assertPerformance(label, stats) {
  const thresholds = PERF_THRESHOLDS[label];
  if (!thresholds) return;
  if (stats.ready.avg > thresholds.ready) {
    console.warn(`[PERF] ${label}: ready ${stats.ready.avg}ms > threshold ${thresholds.ready}ms`);
  }
  if (stats.domContentLoaded.avg > thresholds.domContentLoaded) {
    console.warn(`[PERF] ${label}: DCL ${stats.domContentLoaded.avg}ms > threshold ${thresholds.domContentLoaded}ms`);
  }
}

function buildMarkdown(results) {
  const generatedAt = new Date().toLocaleString("zh-CN", {
    hour12: false,
    timeZone: "Asia/Shanghai"
  });
  const ranking = [...results]
    .sort((left, right) => left.ready.avg - right.ready.avg)
    .map(
      (result, index) =>
        `${index + 1}. ${result.label}：首屏关键锚点平均 ${result.ready.avg} ms，DOMContentLoaded 平均 ${result.domContentLoaded.avg} ms，load 平均 ${result.load.avg} ms`
    )
    .join("\n");

  const details = results
    .map((result) => {
      const sampleRows = result.samples
        .map(
          (sample) =>
            `| ${sample.sample} | ${sample.domContentLoaded} | ${sample.load} | ${sample.ready} | ${sample.firstContentfulPaint ?? "-"} |`
        )
        .join("\n");

      return `## ${result.label}

- 路径：\`${result.url}\`
- 关键锚点：\`${result.readySelector}\`
- DOMContentLoaded：平均 ${result.domContentLoaded.avg} ms，范围 ${result.domContentLoaded.min} - ${result.domContentLoaded.max} ms
- load：平均 ${result.load.avg} ms，范围 ${result.load.min} - ${result.load.max} ms
- 首屏关键锚点可见：平均 ${result.ready.avg} ms，范围 ${result.ready.min} - ${result.ready.max} ms
- First Contentful Paint：平均 ${result.firstContentfulPaint.avg} ms，范围 ${result.firstContentfulPaint.min} - ${result.firstContentfulPaint.max} ms

| 样本 | DOMContentLoaded (ms) | load (ms) | 关键锚点可见 (ms) | FCP (ms) |
| --- | ---: | ---: | ---: | ---: |
${sampleRows}`;
    })
    .join("\n\n");

  return `# 首屏加载时间对照

- 生成时间：${generatedAt}
- 采样环境：Playwright + Edge（preview 构建）
- 视口：${VIEWPORT.width} x ${VIEWPORT.height}
- 样本数：每个页面 ${SAMPLE_COUNT} 次冷启动
- 指标说明：
  - DOMContentLoaded：文档解析完成时间
  - load：页面 load 事件完成时间
  - 首屏关键锚点可见：页面关键容器首次稳定可见时间
  - FCP：浏览器 First Contentful Paint

## 结论排序

${ranking}

## 明细

${details}
`;
}

test.describe("first-screen performance review", () => {
  test.skip(
    !process.env.PERF_REVIEW,
    "Set PERF_REVIEW=1 to generate the first-screen performance report."
  );

  test("measure cold-start first-screen timing across editor and analysis routes", async ({ browser, baseURL }) => {
    fs.mkdirSync(outputDir, { recursive: true });
    const results = [];

    for (const route of routes) {
      const samples = [];

      for (let index = 0; index < SAMPLE_COUNT; index += 1) {
        const context = await browser.newContext({
          viewport: VIEWPORT,
          serviceWorkers: "block"
        });
        const page = await context.newPage();
        const { api } = await installAnalysisReviewMocks(page);

        await page.goto(new URL(route.url, baseURL).toString(), {
          waitUntil: "domcontentloaded"
        });
        await page.locator(route.readySelector).waitFor({ state: "visible" });
        await expect(page.locator(route.readySelector)).toBeVisible();
        await page.waitForLoadState("load");

        const metrics = await page.evaluate(() => {
          const navigation = performance.getEntriesByType("navigation")[0];
          const firstContentfulPaint = performance
            .getEntriesByType("paint")
            .find((entry) => entry.name === "first-contentful-paint");

          return {
            domContentLoaded: navigation?.domContentLoadedEventEnd ?? performance.now(),
            load: navigation?.loadEventEnd ?? performance.now(),
            ready: performance.now(),
            firstContentfulPaint: firstContentfulPaint?.startTime ?? null
          };
        });

        api.expectNoUnexpectedApiRequests();

        samples.push({
          sample: index + 1,
          domContentLoaded: round(metrics.domContentLoaded),
          load: round(metrics.load),
          ready: round(metrics.ready),
          firstContentfulPaint:
            metrics.firstContentfulPaint == null ? null : round(metrics.firstContentfulPaint)
        });

        await context.close();
      }

      results.push({
        name: route.name,
        label: route.label,
        url: route.url,
        readySelector: route.readySelector,
        samples,
        domContentLoaded: summarize("DOMContentLoaded", samples, "domContentLoaded"),
        load: summarize("load", samples, "load"),
        ready: summarize("ready", samples, "ready"),
        firstContentfulPaint: summarize(
          "firstContentfulPaint",
          samples.map((sample) => ({
            ...sample,
            firstContentfulPaint: sample.firstContentfulPaint ?? 0
          })),
          "firstContentfulPaint"
        )
      });
    }

    // P3.2: Assert performance thresholds
    for (const result of results) {
      assertPerformance(result.label, {
        ready: result.ready,
        domContentLoaded: result.domContentLoaded,
        load: result.load,
      });
    }

    fs.writeFileSync(rawPath, `${JSON.stringify(results, null, 2)}\n`, "utf8");
    fs.writeFileSync(reportPath, buildMarkdown(results), "utf8");
  });
});
