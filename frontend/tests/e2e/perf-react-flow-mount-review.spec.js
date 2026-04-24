import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test, expect } from "@playwright/test";
import {
  buildReviewGraphFixture,
  installAnalysisReviewMocks
} from "./support/analysisReviewFixtures";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "../../..");
const outputDir = path.join(repoRoot, "markdown", "performance-review");
const reportPath = path.join(outputDir, "react-flow-mount-review.md");
const rawPath = path.join(outputDir, "react-flow-mount-review.json");

const SAMPLE_COUNT = 5;
const VIEWPORT = { width: 1280, height: 900 };
const reviewGraph = buildReviewGraphFixture();
const EXPECTED_NODE_COUNT = reviewGraph.nodes.length;
const EXPECTED_EDGE_COUNT = reviewGraph.edges.length;

const variants = [
  {
    name: "full",
    label: "当前完整节点卡片",
    url: "/?node_card_mode=full"
  },
  {
    name: "staged",
    label: "首屏精简后再升级",
    url: "/"
  }
];

function round(value) {
  return Math.round(value * 100) / 100;
}

function summarize(samples, key) {
  const values = samples.map((sample) => sample[key]);
  const total = values.reduce((sum, value) => sum + value, 0);
  return {
    avg: round(total / values.length),
    min: round(Math.min(...values)),
    max: round(Math.max(...values))
  };
}

function deltaText(current, baseline) {
  const delta = round(current - baseline);
  const sign = delta > 0 ? "+" : "";
  return `${sign}${delta} ms`;
}

function buildMarkdown(results) {
  const generatedAt = new Date().toLocaleString("zh-CN", {
    hour12: false,
    timeZone: "Asia/Shanghai"
  });
  const full = results.find((result) => result.name === "full");
  const staged = results.find((result) => result.name === "staged");
  const comparison = staged && full
    ? [
        `- 首个节点可见：${staged.summary.firstNodeVisible.avg} ms，对比完整卡片 ${deltaText(staged.summary.firstNodeVisible.avg, full.summary.firstNodeVisible.avg)}`,
        `- 首个节点卡片挂载：${staged.summary.firstNodeCardMounted.avg} ms，对比完整卡片 ${deltaText(staged.summary.firstNodeCardMounted.avg, full.summary.firstNodeCardMounted.avg)}`,
        `- 全部节点挂载：${staged.summary.allNodesMounted.avg} ms，对比完整卡片 ${deltaText(staged.summary.allNodesMounted.avg, full.summary.allNodesMounted.avg)}`,
        `- 全部边挂载：${staged.summary.allEdgesMounted.avg} ms，对比完整卡片 ${deltaText(staged.summary.allEdgesMounted.avg, full.summary.allEdgesMounted.avg)}`,
        `- 全量富节点卡片可见：${staged.summary.allFullCardsMounted.avg} ms，对比完整卡片 ${deltaText(staged.summary.allFullCardsMounted.avg, full.summary.allFullCardsMounted.avg)}`
      ].join("\n")
    : "- 未生成对照结果";

  const detailSections = results
    .map((result) => {
      const sampleRows = result.samples
        .map(
          (sample) =>
            `| ${sample.sample} | ${sample.domContentLoaded} | ${sample.load} | ${sample.firstNodeVisible} | ${sample.firstNodeCardMounted} | ${sample.allNodesMounted} | ${sample.allEdgesMounted} | ${sample.allFullCardsMounted} |`
        )
        .join("\n");

      return `## ${result.label}

- 路径：\`${result.url}\`
- DOMContentLoaded：平均 ${result.summary.domContentLoaded.avg} ms，范围 ${result.summary.domContentLoaded.min} - ${result.summary.domContentLoaded.max} ms
- load：平均 ${result.summary.load.avg} ms，范围 ${result.summary.load.min} - ${result.summary.load.max} ms
- 首个节点可见：平均 ${result.summary.firstNodeVisible.avg} ms，范围 ${result.summary.firstNodeVisible.min} - ${result.summary.firstNodeVisible.max} ms
- 首个节点卡片挂载：平均 ${result.summary.firstNodeCardMounted.avg} ms，范围 ${result.summary.firstNodeCardMounted.min} - ${result.summary.firstNodeCardMounted.max} ms
- 全部节点挂载：平均 ${result.summary.allNodesMounted.avg} ms，范围 ${result.summary.allNodesMounted.min} - ${result.summary.allNodesMounted.max} ms
- 全部边挂载：平均 ${result.summary.allEdgesMounted.avg} ms，范围 ${result.summary.allEdgesMounted.min} - ${result.summary.allEdgesMounted.max} ms
- 全量富节点卡片可见：平均 ${result.summary.allFullCardsMounted.avg} ms，范围 ${result.summary.allFullCardsMounted.min} - ${result.summary.allFullCardsMounted.max} ms

| 样本 | DOMContentLoaded (ms) | load (ms) | 首个节点可见 (ms) | 首个节点卡片挂载 (ms) | 全部节点挂载 (ms) | 全部边挂载 (ms) | 全量富卡片可见 (ms) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
${sampleRows}`;
    })
    .join("\n\n");

  return `# React Flow 首次挂载专项分析

- 生成时间：${generatedAt}
- 页面：编辑器首页
- 视口：${VIEWPORT.width} x ${VIEWPORT.height}
- 采样环境：Playwright + Edge（preview 构建）
- 样本数：每种模式 ${SAMPLE_COUNT} 次冷启动
- 图规模：${EXPECTED_NODE_COUNT} 个节点，${EXPECTED_EDGE_COUNT} 条边

## 对照结论

${comparison}

## 判断

- 如果“首个节点可见”和“全部节点挂载”差距很小，说明主要成本不在节点逐步补挂，而在 React Flow 容器初始化本身。
- 如果“全部边挂载”明显晚于“全部节点挂载”，说明边路径计算与边 DOM 挂载是主要追加成本。
- 如果“首个节点卡片挂载”已经接近“首个节点可见”，说明节点卡片本身不是第一瓶颈。
- 如果“全量富节点卡片可见”明显晚于“首个节点卡片挂载”，说明 staged 模式确实把丰富节点内容延后了。

## 明细

${detailSections}
`;
}

async function performanceNow(page) {
  return round(await page.evaluate(() => performance.now()));
}

test.describe("react flow mount performance review", () => {
  test.skip(
    !process.env.PERF_REVIEW,
    "Set PERF_REVIEW=1 to generate the React Flow mount performance report."
  );

  test("compare full node cards with staged first-screen node cards", async ({ browser, baseURL }) => {
    fs.mkdirSync(outputDir, { recursive: true });
    const results = [];

    for (const variant of variants) {
      const samples = [];

      for (let index = 0; index < SAMPLE_COUNT; index += 1) {
        const context = await browser.newContext({
          viewport: VIEWPORT,
          serviceWorkers: "block"
        });
        const page = await context.newPage();
        const { api } = await installAnalysisReviewMocks(page);

        await page.goto(new URL(variant.url, baseURL).toString(), {
          waitUntil: "domcontentloaded"
        });
        await expect(page.locator(".editor-page")).toBeVisible();
        await expect(page.locator(".main-workspace")).toBeVisible();

        await page.waitForFunction(() => document.querySelectorAll(".react-flow__node").length >= 1);
        const firstNodeVisible = await performanceNow(page);

        await page.waitForFunction(() => document.querySelectorAll(".node-card").length >= 1);
        const firstNodeCardMounted = await performanceNow(page);

        await page.waitForFunction(
          (expectedCount) => document.querySelectorAll(".react-flow__node").length >= expectedCount,
          EXPECTED_NODE_COUNT
        );
        const allNodesMounted = await performanceNow(page);

        await page.waitForFunction(
          (expectedCount) => document.querySelectorAll(".react-flow__edge").length >= expectedCount,
          EXPECTED_EDGE_COUNT
        );
        const allEdgesMounted = await performanceNow(page);

        await page.waitForFunction(
          (expectedCount) =>
            document.querySelectorAll('.node-card[data-node-card-variant="full"]').length >= expectedCount,
          EXPECTED_NODE_COUNT
        );
        const allFullCardsMounted = await performanceNow(page);

        await page.waitForLoadState("load");
        const navigation = await page.evaluate(() => {
          const entry = performance.getEntriesByType("navigation")[0];
          return {
            domContentLoaded: entry?.domContentLoadedEventEnd ?? performance.now(),
            load: entry?.loadEventEnd ?? performance.now()
          };
        });

        api.expectNoUnexpectedApiRequests();

        samples.push({
          sample: index + 1,
          domContentLoaded: round(navigation.domContentLoaded),
          load: round(navigation.load),
          firstNodeVisible,
          firstNodeCardMounted,
          allNodesMounted,
          allEdgesMounted,
          allFullCardsMounted
        });

        await context.close();
      }

      results.push({
        name: variant.name,
        label: variant.label,
        url: variant.url,
        summary: {
          domContentLoaded: summarize(samples, "domContentLoaded"),
          load: summarize(samples, "load"),
          firstNodeVisible: summarize(samples, "firstNodeVisible"),
          firstNodeCardMounted: summarize(samples, "firstNodeCardMounted"),
          allNodesMounted: summarize(samples, "allNodesMounted"),
          allEdgesMounted: summarize(samples, "allEdgesMounted"),
          allFullCardsMounted: summarize(samples, "allFullCardsMounted")
        },
        samples
      });
    }

    const output = {
      sampleCount: SAMPLE_COUNT,
      viewport: VIEWPORT,
      graph: {
        nodeCount: EXPECTED_NODE_COUNT,
        edgeCount: EXPECTED_EDGE_COUNT
      },
      results
    };

    fs.writeFileSync(rawPath, `${JSON.stringify(output, null, 2)}\n`, "utf8");
    fs.writeFileSync(reportPath, buildMarkdown(results), "utf8");
  });
});
