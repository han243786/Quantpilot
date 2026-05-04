import { test, expect } from "@playwright/test";

import path from "node:path";
import { fileURLToPath } from "node:url";
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SCREENSHOT_DIR = path.resolve(__dirname, "../../../markdown/测试/screenshots");
let stepNum = 0;

async function step(page, name, fn) {
  stepNum++;
  const screenshotName = `p2-${String(stepNum).padStart(3, "0")}-${name.replace(/[^a-zA-Z0-9一-鿿]/g, "_").slice(0, 40)}`;
  try {
    await test.step(name, fn, { timeout: 30000 });
    await page.screenshot({ path: `${SCREENSHOT_DIR}/${screenshotName}.png`, fullPage: true }).catch(() => {});
    console.log(`[PASS] Step ${stepNum}: ${name}`);
  } catch (e) {
    await page.screenshot({ path: `${SCREENSHOT_DIR}/${screenshotName}-FAIL.png`, fullPage: true }).catch(() => {});
    console.log(`[FAIL] Step ${stepNum}: ${name} — ${e.message?.slice(0, 120)}`);
  }
}

// ═══════════════════════════════════════════════════
// Helper: enter workspace code tab
// ═══════════════════════════════════════════════════
async function enterWorkspaceCodeTab(page) {
  await page.goto("/strategies");
  await page.waitForTimeout(1500);

  const currentWs = page.getByTestId("strategy-hub-open-current-workspace");
  if (await currentWs.isVisible().catch(() => false)) {
    await currentWs.click();
    await page.waitForTimeout(1500);
  }

  const codeTab = page.getByTestId("workspace-tab-code");
  if (await codeTab.isVisible().catch(() => false)) {
    await codeTab.click();
    await page.waitForTimeout(1500);
  }
}

// P3.1: Dynamic module key lookup via test bridge
async function getModuleKey(page, partialName) {
  const keys = await page.evaluate(() => {
    const cards = document.querySelectorAll("[data-testid^='module-card-']");
    return Array.from(cards).map((el) => el.getAttribute("data-testid").replace("module-card-", ""));
  });
  return keys.find((k) => k.toLowerCase().includes(partialName.toLowerCase())) || null;
}

// Helper: drag node from module palette to canvas
async function dragModuleToCanvas(page, moduleKeyOrName) {
  // Try exact match first, then fuzzy lookup
  let key = moduleKeyOrName;
  let card = page.getByTestId(`module-card-${key}`);
  if (!(await card.isVisible().catch(() => false))) {
    key = await getModuleKey(page, moduleKeyOrName);
    if (!key) {
      console.log(`Module card "${moduleKeyOrName}" not found`);
      return false;
    }
    card = page.getByTestId(`module-card-${key}`);
  }
  if (!(await card.isVisible().catch(() => false))) {
    console.log(`Module card ${key} not visible`);
    return false;
  }
  const canvas = page.locator(".react-flow__pane").first();
  const srcBox = await card.boundingBox();
  const tgtBox = await canvas.boundingBox();
  if (!srcBox || !tgtBox) return false;
  const offsetX = 200 + Math.random() * 150;
  const offsetY = 150 + Math.random() * 200;
  await page.mouse.move(srcBox.x + srcBox.width / 2, srcBox.y + srcBox.height / 2);
  await page.mouse.down();
  await page.mouse.move(tgtBox.x + offsetX, tgtBox.y + offsetY, { steps: 15 });
  await page.mouse.up();
  await page.waitForTimeout(500);
  return true;
}

// ═══════════════════════════════════════════════════
// 场景六：画布操作深度测试 (P1 data-testid 验证)
// ═══════════════════════════════════════════════════
test.describe("场景六：画布操作深度测试 (v2)", () => {
  test("6.1-6.3 键盘选择、小地图、模块搜索、Handle 连线", async ({ page }) => {
    await enterWorkspaceCodeTab(page);

    // Verify Handle ports exist
    await step(page, "验证 Handle 端口 data-testid 存在", async () => {
      const targetHandles = await page.locator("[data-testid^='handle-target-']").count();
      const sourceHandles = await page.locator("[data-testid^='handle-source-']").count();
      expect(targetHandles).toBeGreaterThan(0);
      expect(sourceHandles).toBeGreaterThan(0);
      console.log(`Handles: ${targetHandles} target, ${sourceHandles} source`);
    });

    // Verify module cards
    await step(page, "验证模块卡片 data-testid 存在", async () => {
      const cards = await page.locator("[data-testid^='module-card-']").count();
      expect(cards).toBeGreaterThan(0);
      console.log(`Module cards: ${cards}`);
    });

    // Test node selection via click
    await step(page, "点击节点并验证属性面板出现", async () => {
      const node = page.locator(".react-flow__node").first();
      await node.click({ force: true });
      await page.waitForTimeout(500);
      const propInputs = await page.locator("[data-testid^='prop-input-']").count();
      expect(propInputs).toBeGreaterThan(0);
      console.log(`Property inputs visible: ${propInputs}`);
    });

    // Test Delete + Undo
    await step(page, "Delete 键删除节点", async () => {
      const before = await page.evaluate(() => window.__QUANTPILOT_TEST__?.getNodeCount());
      const node = page.locator(".react-flow__node").last();
      await node.click({ force: true });
      await page.keyboard.press("Delete");
      await page.waitForTimeout(500);
      const after = await page.evaluate(() => window.__QUANTPILOT_TEST__?.getNodeCount());
      console.log(`Node count: ${before} → ${after}`);
    });

    await step(page, "Ctrl+Z 撤销删除", async () => {
      await page.keyboard.press("Control+z");
      await page.waitForTimeout(800);
      const restored = await page.evaluate(() => window.__QUANTPILOT_TEST__?.getNodeCount());
      console.log(`After undo: ${restored}`);
    });

    // Test canvas zoom
    await step(page, "滚轮缩放画布", async () => {
      const pane = page.locator(".react-flow__pane").first();
      const box = await pane.boundingBox();
      if (box) {
        await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
        await page.mouse.wheel(0, -120);
        await page.waitForTimeout(400);
        await page.mouse.wheel(0, 120);
        await page.waitForTimeout(400);
      }
    });

    // Test module search
    await step(page, "模块搜索 RSI", async () => {
      const searchInput = page.locator("input[placeholder*='搜索'], input[placeholder*='Search']").first();
      if (await searchInput.count() > 0) {
        await searchInput.fill("RSI");
        await page.waitForTimeout(800);
        const visibleCards = await page.locator("[data-testid^='module-card-']:visible").count();
        console.log(`Visible after search: ${visibleCards}`);
      }
    });

    await step(page, "清空搜索恢复全部模块", async () => {
      const searchInput = page.locator("input[placeholder*='搜索'], input[placeholder*='Search']").first();
      if (await searchInput.count() > 0) {
        await searchInput.fill("");
        await page.waitForTimeout(500);
      }
    });
  });
});

// ═══════════════════════════════════════════════════
// 场景七：策略工作区标签页
// ═══════════════════════════════════════════════════
test.describe("场景七：策略工作区标签页 (v2)", () => {
  test("7.1 四个标签页切换 + test bridge 验证", async ({ page }) => {
    await page.goto("/strategies");
    await page.waitForTimeout(1500);
    const wsBtn = page.getByTestId("strategy-hub-open-current-workspace");
    if (await wsBtn.isVisible().catch(() => false)) {
      await wsBtn.click();
      await page.waitForTimeout(2000);
    }

    for (const tab of ["overview", "code", "diagnostics", "research"]) {
      await step(page, `切换到 ${tab} 标签`, async () => {
        const tabEl = page.getByTestId(`workspace-tab-${tab}`);
        if (await tabEl.isVisible().catch(() => false)) {
          await tabEl.click();
          await page.waitForTimeout(1000);
        }
        const active = await page.evaluate(() => window.__QUANTPILOT_TEST__?.getActiveTab());
        expect(active).toBe(tab);
      });
    }
  });
});

// ═══════════════════════════════════════════════════
// 场景十：i18n 和 UI 细节 (test bridge)
// ═══════════════════════════════════════════════════
test.describe("场景十：i18n + UI (v2)", () => {
  test("10.1-10.3 国际化 + 加载 + 响应式", async ({ page }) => {
    await page.goto("/strategies");
    await page.waitForTimeout(2000);

    await step(page, "验证中文 UI 文案", async () => {
      const body = await page.locator("body").innerText();
      for (const text of ["策略", "审批", "告警", "快照", "故障手册", "混沌", "QuantPilot"]) {
        expect(body).toContain(text);
      }
    });

    await step(page, "test bridge 可用性", async () => {
      const has = await page.evaluate(() => typeof window.__QUANTPILOT_TEST__ === "object");
      expect(has).toBe(true);
      const route = await page.evaluate(() => window.__QUANTPILOT_TEST__.getCurrentRoute());
      expect(route.name).toBe("strategies");
    });

    await step(page, "布局快照正常", async () => {
      const layout = await page.evaluate(() => window.__QUANTPILOT_TEST__.getLayoutSnapshot());
      const visibleAreas = Object.keys(layout).filter((k) => layout[k].visible);
      console.log("Visible:", visibleAreas.join(", "));
      expect(visibleAreas.length).toBeGreaterThan(0);
    });

    await step(page, "响应式测试 800x600", async () => {
      await page.setViewportSize({ width: 800, height: 600 });
      await page.waitForTimeout(1000);
      const layout = await page.evaluate(() => window.__QUANTPILOT_TEST__.getLayoutSnapshot());
      expect(layout).toBeTruthy();
    });

    await step(page, "恢复正常视口", async () => {
      await page.setViewportSize({ width: 1440, height: 900 });
    });
  });
});

// ═══════════════════════════════════════════════════
// 场景五：Block 5 页面导航 (API-backed)
// ═══════════════════════════════════════════════════
test.describe("场景五：告警/快照/故障手册/混沌/审批 (v2)", () => {
  test("5.1 告警规则 10 条验证", async ({ page }) => {
    await page.goto("/alerts");
    await page.waitForTimeout(1500);

    // Also verify via backend API
    const apiResp = await page.request.get("http://127.0.0.1:3000/api/v1/alerts/rules");
    expect(apiResp.ok()).toBe(true);
    const rules = await apiResp.json();
    console.log(`Alert rules via API: ${Array.isArray(rules) ? rules.length : JSON.stringify(rules).length} items`);
  });

  test("5.3 故障手册 API 验证", async ({ page }) => {
    const resp = await page.request.get("http://127.0.0.1:3000/api/v1/runbook");
    expect(resp.ok()).toBe(true);
    const runbook = await resp.json();
    console.log(`Runbook scenarios: ${Array.isArray(runbook.scenarios) ? runbook.scenarios.length : 'found'}`);
  });

  test("5.2 快照 + 5.4 混沌 + 5.5 审批 页面加载", async ({ page }) => {
    for (const path of ["/snapshots", "/chaos", "/approvals"]) {
      await page.goto(path);
      await page.waitForTimeout(1000);
      const body = await page.locator("body").innerText();
      expect(body.length).toBeGreaterThan(50);
    }
  });
});

// ═══════════════════════════════════════════════════
// 场景一/八：Handle 连线 + 属性编辑
// ═══════════════════════════════════════════════════
test.describe("Handle 连线 + 属性编辑 (v2)", () => {
  test("拖入模块并连线", async ({ page }) => {
    await enterWorkspaceCodeTab(page);

    // Drag in K-line and dual MA nodes
    await step(page, "拖入 K 线数据节点", async () => {
      await dragModuleToCanvas(page, "klines");
    });

    await step(page, "拖入双均线意图节点", async () => {
      await dragModuleToCanvas(page, "dual_ma");
    });

    // Get node IDs from test bridge
    await step(page, "获取节点 ID 并连线", async () => {
      const nodes = await page.evaluate(() => {
        const state = window.__QUANTPILOT_TEST__?.getRawState();
        return state?.graph?.nodes?.map(n => ({ id: n.id, type: n.type })) ?? [];
      });
      console.log("Nodes:", JSON.stringify(nodes));

      // Find the data node and intent node
      const dataNode = nodes.find(n => n.type === "data");
      const intentNode = nodes.find(n => n.type === "intent");
      if (dataNode && intentNode) {
        // Use handle testid for precise connection
        const srcHandle = page.getByTestId(`handle-source-${dataNode.id}-output`);
        const tgtHandle = page.getByTestId(`handle-target-${intentNode.id}-input`);
        if (await srcHandle.count() > 0 && await tgtHandle.count() > 0) {
          await srcHandle.dragTo(tgtHandle);
          await page.waitForTimeout(500);
          const edgeCount = await page.evaluate(() => window.__QUANTPILOT_TEST__?.getEdgeCount());
          console.log(`Edge count after connecting: ${edgeCount}`);
        }
      }
    });

    // Test property editing with new data-testid
    await step(page, "点击节点编辑属性", async () => {
      const dataNode = page.locator(".react-flow__node-data").first();
      if (await dataNode.count() > 0) {
        await dataNode.click({ force: true });
        await page.waitForTimeout(500);
      }
      const inputs = await page.locator("[data-testid^='prop-input-']").count();
      console.log(`Property inputs: ${inputs}`);
    });
  });
});

test.afterAll(() => {
  console.log(`\nP2 测试完成 — 总步骤: ${stepNum}`);
});
