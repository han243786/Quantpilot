import { test, expect } from "@playwright/test";

test.skip(true, "Legacy exploratory walkthrough superseded by deterministic smoke specs and scenario-test-v2.");

const SCREENSHOT_DIR = "D:/rust-js-pr/QuantPilot/quantpilot/markdown/测试/screenshots";
let stepNum = 0;
let failureCount = 0;

async function step(page, description, fn) {
  stepNum++;
  const name = `${String(stepNum).padStart(3, "0")}-${description.replace(/[^a-zA-Z0-9一-鿿]/g, "_").slice(0, 40)}`;
  try {
    await test.step(`${description}`, fn, { timeout: 30000 });
    await page.screenshot({ path: `${SCREENSHOT_DIR}/${name}.png`, fullPage: true }).catch(() => {});
  } catch (e) {
    failureCount++;
    await page.screenshot({ path: `${SCREENSHOT_DIR}/${name}-FAIL.png`, fullPage: true }).catch(() => {});
    console.log(`[FAIL] Step ${stepNum}: ${description} — ${e.message?.slice(0, 100)}`);
  }
}

async function enterWorkspace(page) {
  await page.goto("/strategies");
  await page.waitForTimeout(1500);

  // Try opening current workspace first
  const currentWs = page.getByTestId("strategy-hub-open-current-workspace");
  if (await currentWs.isVisible().catch(() => false)) {
    await currentWs.click();
    await page.waitForTimeout(1500);
    return;
  }

  // Try blank workspace
  const blankWs = page.getByTestId("strategy-hub-open-blank-workspace");
  if (await blankWs.isVisible().catch(() => false)) {
    await blankWs.click();
    await page.waitForTimeout(1500);
    return;
  }

  // Fallback: click first strategy in roster
  const firstLink = page.locator("[data-testid='strategy-hub-roster-table-body'] a, a[href*='/strategies/']").first();
  if (await firstLink.isVisible().catch(() => false)) {
    await firstLink.click();
    await page.waitForTimeout(1500);
  }
}

async function switchToCodeTab(page) {
  const codeTab = page.getByTestId("workspace-tab-code");
  if (await codeTab.isVisible().catch(() => false)) {
    await codeTab.click();
    await page.waitForTimeout(1000);
  }
}

async function switchToResearchTab(page) {
  const tab = page.getByTestId("workspace-tab-research");
  if (await tab.isVisible().catch(() => false)) {
    await tab.click();
    await page.waitForTimeout(1000);
  }
}

// ═══════════════════════════════════════════════════
// 场景一：从零搭建 BTC 双均线策略并 Paper 运行
// ═══════════════════════════════════════════════════
test.describe("场景一：从零搭建 BTC 双均线策略并 Paper 运行", () => {
  test("1.1 进入策略中心", async ({ page }) => {
    await page.goto("/");
    await page.waitForTimeout(2000);

    await step(page, "页面跳转到 /strategies，显示导航栏", async () => {
      await expect(page).toHaveURL(/\/strategies/);
    });

    await step(page, "观察页面布局 — 策略花名册、模板库、近期运行/回测", async () => {
      // Check key elements exist
      const hero = page.getByTestId("strategy-hub-hero");
      const roster = page.getByTestId("strategy-hub-roster-table");
      console.log(`Hero visible: ${await hero.isVisible().catch(() => false)}`);
      console.log(`Roster visible: ${await roster.isVisible().catch(() => false)}`);
    });
  });

  test("1.2-1.6 从模板创建 + 搭建图 + 保存编译 + Paper运行", async ({ page }) => {
    await enterWorkspace(page);

    await step(page, "切换到构建标签查看编辑器布局", async () => {
      await switchToCodeTab(page);
    });

    await step(page, "观察编辑器四区域：工具栏、模块面板、画布、属性面板", async () => {
      const toolbar = page.locator(".top-toolbar--workspace");
      console.log(`Toolbar visible: ${await toolbar.isVisible().catch(() => false)}`);
      const canvas = page.locator(".react-flow");
      console.log(`Canvas visible: ${await canvas.isVisible().catch(() => false)}`);
    });

    // Drag K-line node
    await step(page, "拖入 K 线数据节点到画布", async () => {
      const moduleCard = page.locator(".module-card").first();
      const canvasPane = page.locator(".react-flow__pane").first();
      if (await moduleCard.isVisible().catch(() => false)) {
        const srcBox = await moduleCard.boundingBox();
        const tgtBox = await canvasPane.boundingBox();
        if (srcBox && tgtBox) {
          await page.mouse.move(srcBox.x + srcBox.width / 2, srcBox.y + srcBox.height / 2);
          await page.mouse.down();
          await page.mouse.move(tgtBox.x + 200, tgtBox.y + 200, { steps: 15 });
          await page.mouse.up();
          await page.waitForTimeout(800);
        }
      }
    });

    // Check node appeared on canvas
    await step(page, "确认节点出现在画布上", async () => {
      const nodeCount = await page.locator(".react-flow__node").count();
      console.log(`Nodes on canvas: ${nodeCount}`);
    });

    // Click node to see property panel
    await step(page, "点击 K 线节点查看属性面板", async () => {
      const node = page.locator(".react-flow__node").first();
      if ((await node.count()) > 0) {
        await node.click({ force: true });
        await page.waitForTimeout(800);
      }
    });

    // Drag more nodes
    const nodeNames = ["双均线", "加权代理", "全局风控", "模拟执行", "运行控制"];
    for (const name of nodeNames) {
      await step(page, `拖入 ${name} 节点`, async () => {
        const card = page.locator(".module-card").filter({ hasText: new RegExp(name) }).first();
        if (await card.isVisible().catch(() => false)) {
          const canvasPane = page.locator(".react-flow__pane").first();
          const srcBox = await card.boundingBox();
          const tgtBox = await canvasPane.boundingBox();
          if (srcBox && tgtBox) {
            const offsetX = 200 + Math.random() * 150;
            const offsetY = 100 + nodeNames.indexOf(name) * 120;
            await page.mouse.move(srcBox.x + srcBox.width / 2, srcBox.y + srcBox.height / 2);
            await page.mouse.down();
            await page.mouse.move(tgtBox.x + offsetX, tgtBox.y + offsetY, { steps: 12 });
            await page.mouse.up();
            await page.waitForTimeout(300);
          }
        } else {
          console.log(`Module card "${name}" not found`);
        }
      });
    }

    // Connect nodes
    await step(page, "连线节点（K线→双均线→加权代理→风控→执行）", async () => {
      const nodes = page.locator(".react-flow__node");
      const count = await nodes.count();
      for (let i = 0; i < Math.min(count - 1, 5); i++) {
        const src = nodes.nth(i);
        const tgt = nodes.nth(i + 1);
        const srcBox = await src.boundingBox();
        const tgtBox = await tgt.boundingBox();
        if (srcBox && tgtBox) {
          await page.mouse.move(srcBox.x + srcBox.width, srcBox.y + srcBox.height / 2);
          await page.mouse.down();
          await page.mouse.move(tgtBox.x, tgtBox.y + tgtBox.height / 2, { steps: 15 });
          await page.mouse.up();
          await page.waitForTimeout(200);
        }
      }
    });

    // Save — check button exists (may not work if graph incomplete)
    await step(page, "检查保存策略图按钮", async () => {
      const saveBtn = page.getByRole("button", { name: /保存策略图|保存.*图/i }).first();
      const found = await saveBtn.isVisible({ timeout: 3000 }).catch(() => false);
      console.log(`Save button visible: ${found}`);
      if (found) {
        await saveBtn.click();
        await page.waitForTimeout(2000);
      }
    });

    // Compile — check button state
    await step(page, "检查编译按钮状态", async () => {
      const compileBtn = page.getByTestId("toolbar-compile-action");
      const compiledEnabled = await compileBtn.isEnabled({ timeout: 3000 }).catch(() => false);
      console.log(`Compile button enabled: ${compiledEnabled}`);
      if (compiledEnabled) {
        await compileBtn.click();
        await page.waitForTimeout(4000);
      }
    });

    // Run — check button state
    await step(page, "检查启动模拟按钮状态", async () => {
      const runBtn = page.getByTestId("toolbar-start-runtime-action");
      const runEnabled = await runBtn.isEnabled({ timeout: 5000 }).catch(() => false);
      console.log(`Run button enabled: ${runEnabled}`);
      if (runEnabled) {
        await runBtn.click();
        await page.waitForTimeout(5000);
      }
    });

    // Switch to research tab to view events
    await step(page, "切换到研究标签观察事件流", async () => {
      await switchToResearchTab(page);
      await page.waitForTimeout(2000);
    });
  });
});

// ═══════════════════════════════════════════════════
// 场景二：回测 + 参数对比
// ═══════════════════════════════════════════════════
test.describe("场景二：回测 + 参数对比", () => {
  test("2.1-2.4 运行回测、修改参数再回测、对比、详情", async ({ page }) => {
    await enterWorkspace(page);
    await switchToCodeTab(page);

    // Click backtest
    await step(page, "点击运行回测", async () => {
      // Try to enable backtest button by saving and compiling first
      const compileBtn = page.getByTestId("toolbar-compile-action");
      if (!(await compileBtn.isEnabled().catch(() => false))) {
        const saveBtn = page.getByRole("button", { name: /保存策略图/i }).first();
        if (await saveBtn.isVisible({ timeout: 1000 }).catch(() => false)) {
          await saveBtn.click();
          await page.waitForTimeout(2000);
        }
      }
      if (await compileBtn.isEnabled({ timeout: 3000 }).catch(() => false)) {
        await compileBtn.click();
        await page.waitForTimeout(3000);
      }
      const btBtn = page.getByTestId("toolbar-start-backtest-action");
      if (await btBtn.isEnabled({ timeout: 5000 }).catch(() => false)) {
        await btBtn.click();
        await page.waitForTimeout(3000);
      } else {
        console.log("Backtest button disabled — compile may not have completed");
      }
    });

    // Select deterministic_mock mode if available
    await step(page, "选择 deterministic_mock 模式", async () => {
      // Look for backtest options dialog
      const mockOption = page.getByRole("radio", { name: /deterministic|mock/i });
      const mockCheckbox = page.locator("label").filter({ hasText: /deterministic|mock/i });
      if (await mockOption.isVisible().catch(() => false)) {
        await mockOption.check();
        await page.waitForTimeout(500);
      } else if (await mockCheckbox.isVisible().catch(() => false)) {
        await mockCheckbox.click();
        await page.waitForTimeout(500);
      }
      const confirmBtn = page.getByRole("button", { name: /确认|启动|开始|OK|Run/i }).first();
      if (await confirmBtn.isVisible().catch(() => false)) {
        await confirmBtn.click();
      }
      await page.waitForTimeout(5000);
    });

    // Switch to research to see backtest progress
    await step(page, "观察回测进度", async () => {
      await switchToResearchTab(page);
      await page.waitForTimeout(3000);
    });

    // Modify params and run second backtest
    await step(page, "修改均线参数并再次回测", async () => {
      await switchToCodeTab(page);
      const maNode = page.locator(".react-flow__node").filter({ hasText: /双均线/i }).first();
      if ((await maNode.count()) > 0) {
        await maNode.click({ force: true });
        await page.waitForTimeout(800);
      }
    });

    // Compare
    await step(page, "查看对比页面", async () => {
      await switchToResearchTab(page);
      const compareBtn = page.getByRole("button", { name: /对比|Compare/i }).first();
      if (await compareBtn.isVisible().catch(() => false)) {
        await compareBtn.click();
        await page.waitForTimeout(2000);
      }
    });
  });
});

// ═══════════════════════════════════════════════════
// 场景三：策略管理 — 版本、模板、花名册
// ═══════════════════════════════════════════════════
test.describe("场景三：策略管理", () => {
  test("3.1-3.3 版本管理 + 花名册 + 对比队列", async ({ page }) => {
    await enterWorkspace(page);
    await switchToCodeTab(page);

    // Save new version
    await step(page, "保存策略图（创建版本记录）", async () => {
      const saveBtn = page.getByRole("button", { name: "保存策略图" }).first();
      if (await saveBtn.isVisible().catch(() => false)) {
        await saveBtn.click();
        await page.waitForTimeout(1500);
      }
    });

    // Go back to strategy hub
    await step(page, "返回策略中心（点击面包屑或导航）", async () => {
      const breadcrumb = page.locator("a, button, [role='button']").filter({ hasText: "策略中心" }).first();
      if (await breadcrumb.isVisible().catch(() => false)) {
        await breadcrumb.click();
      } else {
        await page.goto("/strategies");
      }
      await page.waitForTimeout(1500);
    });

    await step(page, "观察策略花名册", async () => {
      const roster = page.getByTestId("strategy-hub-roster-table");
      console.log(`Roster visible: ${await roster.isVisible().catch(() => false)}`);
    });

    await step(page, "观察近期运行和回测区域", async () => {
      // Check for recent activity sections
      const pageContent = await page.locator("body").innerText();
      console.log(`Page contains "运行": ${pageContent.includes("运行")}`);
      console.log(`Page contains "回测": ${pageContent.includes("回测")}`);
    });
  });
});

// ═══════════════════════════════════════════════════
// 场景四：故意制造错误 — 测试防御能力
// ═══════════════════════════════════════════════════
test.describe("场景四：故意制造错误", () => {
  test("4.1-4.4 不完整图编译、约束冲突、不支持的交易所", async ({ page }) => {
    await enterWorkspace(page);
    await switchToCodeTab(page);

    // Delete a node
    await step(page, "删除运行控制节点", async () => {
      const nodes = page.locator(".react-flow__node");
      const controlNode = nodes.filter({ hasText: /运行控制|runtime.?control/i });
      if ((await controlNode.count()) > 0) {
        // Use force:true to click through overlapping node cards
        await controlNode.first().click({ force: true, timeout: 5000 });
        await page.waitForTimeout(300);
        await page.keyboard.press("Delete");
        await page.waitForTimeout(500);
      } else {
        const lastNode = nodes.last();
        if (await lastNode.count() > 0) {
          await lastNode.click({ force: true, timeout: 5000 });
          await page.waitForTimeout(300);
          await page.keyboard.press("Delete");
          await page.waitForTimeout(500);
        }
      }
    });

    // Try compile - should fail
    await step(page, "编译应失败（不完整图）", async () => {
      const compileBtn = page.getByTestId("toolbar-compile-action");
      if (await compileBtn.isEnabled().catch(() => false)) {
        await compileBtn.click();
        await page.waitForTimeout(3000);
      }
    });

    // Check for error
    await step(page, "检查编译诊断信息", async () => {
      const body = await page.locator("body").innerText();
      console.log(`Diagnostics visible: ${body.includes("诊断") || body.includes("错误") || body.includes("error")}`);
      // Switch to diagnostics tab
      const diagTab = page.getByTestId("workspace-tab-diagnostics");
      if (await diagTab.isVisible().catch(() => false)) {
        await diagTab.click();
        await page.waitForTimeout(1000);
      }
    });

    // Undo
    await step(page, "Ctrl+Z 撤销删除", async () => {
      await page.keyboard.press("Control+z");
      await page.waitForTimeout(800);
    });

    // Test constraint validation
    await step(page, "测试均线参数约束（快线 > 慢线）", async () => {
      await switchToCodeTab(page);
      const maNode = page.locator(".react-flow__node").filter({ hasText: /双均线/i }).first();
      if (await maNode.isVisible().catch(() => false)) {
        await maNode.click();
        await page.waitForTimeout(500);
      }
      // Try to find and set invalid values
      const inputs = page.locator("input[type='number']");
      const count = await inputs.count();
      if (count >= 2) {
        await inputs.nth(0).fill("200");
        await inputs.nth(1).fill("50");
        await page.waitForTimeout(1000);
      }
    });

    // RSI constraint
    await step(page, "拖入 RSI 意图节点并测试约束", async () => {
      const rsiCard = page.locator(".module-card").filter({ hasText: /RSI/i }).first();
      if (await rsiCard.isVisible().catch(() => false)) {
        const canvasPane = page.locator(".react-flow__pane").first();
        const srcBox = await rsiCard.boundingBox();
        const tgtBox = await canvasPane.boundingBox();
        if (srcBox && tgtBox) {
          await page.mouse.move(srcBox.x + srcBox.width / 2, srcBox.y + srcBox.height / 2);
          await page.mouse.down();
          await page.mouse.move(tgtBox.x + 400, tgtBox.y + 300, { steps: 12 });
          await page.mouse.up();
          await page.waitForTimeout(500);
        }
      }
    });
  });
});

// ═══════════════════════════════════════════════════
// 场景五：Block 5 — 告警、快照、故障手册、混沌、审批
// ═══════════════════════════════════════════════════
test.describe("场景五：告警、快照、故障手册、混沌、审批", () => {
  test("5.1 告警页面", async ({ page }) => {
    await page.goto("/alerts");
    await page.waitForTimeout(2000);

    await step(page, "观察告警页面", async () => {
      const body = await page.locator("body").innerText();
      console.log(`Page contains "告警": ${body.includes("告警")}`);
      console.log(`Page contains "规则": ${body.includes("规则")}`);
    });

    await step(page, "检查 10 条内置规则存在", async () => {
      const rules = [
        "data_freshness", "event_orphan", "risk_reject",
        "replay_divergence", "ai_proposal", "sandbox_verification",
        "storage_watermark", "approval_expiry", "hotswap_rollback",
        "capability_hash"
      ];
      const body = await page.locator("body").innerText();
      for (const rule of rules) {
        console.log(`Rule "${rule}": ${body.includes(rule) ? "FOUND" : "MISSING"}`);
      }
    });
  });

  test("5.2 快照页面", async ({ page }) => {
    await page.goto("/snapshots");
    await page.waitForTimeout(2000);

    await step(page, "观察快照列表", async () => {
      const body = await page.locator("body").innerText();
      console.log(`Has snapshots: ${body.includes("snap") || body.includes("快照")}`);
    });

    await step(page, "点击创建快照", async () => {
      const createBtn = page.getByRole("button", { name: /创建|新建|Create/i }).first();
      if (await createBtn.isVisible().catch(() => false)) {
        await createBtn.click();
        await page.waitForTimeout(1500);
      }
    });
  });

  test("5.3 故障手册", async ({ page }) => {
    await page.goto("/runbook");
    await page.waitForTimeout(2000);

    await step(page, "观察故障场景列表", async () => {
      const body = await page.locator("body").innerText();
      console.log(`Has symptoms: ${body.includes("symptom") || body.includes("症状") || body.includes("diagnostic") || body.includes("诊断")}`);
    });

    await step(page, "展开第一个故障场景详情", async () => {
      const expandBtn = page.getByRole("button").first();
      if (await expandBtn.isVisible().catch(() => false)) {
        await expandBtn.click();
        await page.waitForTimeout(1000);
      }
    });
  });

  test("5.4 混沌实验", async ({ page }) => {
    await page.goto("/chaos");
    await page.waitForTimeout(2000);

    await step(page, "观察混沌实验页面", async () => {
      const body = await page.locator("body").innerText();
      console.log(`Has "混沌" content: ${body.includes("混沌") || body.includes("chaos") || body.includes("实验")}`);
    });

    await step(page, "点击创建实验", async () => {
      const createBtn = page.getByRole("button", { name: /创建|新建|Create/i }).first();
      if (await createBtn.isVisible().catch(() => false)) {
        await createBtn.click();
        await page.waitForTimeout(1500);
      }
    });
  });

  test("5.5 审批页面", async ({ page }) => {
    await page.goto("/approvals");
    await page.waitForTimeout(2000);

    await step(page, "观察审批列表", async () => {
      const body = await page.locator("body").innerText();
      console.log(`Has approvals content: ${body.includes("审批") || body.includes("proposal") || body.includes("Approval")}`);
    });

    await step(page, "检查操作按钮（Claim/Approve/Reject）", async () => {
      const claimBtn = page.getByRole("button", { name: /认领|Claim/i }).first();
      const approveBtn = page.getByRole("button", { name: /批准|Approve/i }).first();
      const rejectBtn = page.getByRole("button", { name: /驳回|Reject/i }).first();
      console.log(`Claim: ${await claimBtn.isVisible().catch(() => false)}`);
      console.log(`Approve: ${await approveBtn.isVisible().catch(() => false)}`);
      console.log(`Reject: ${await rejectBtn.isVisible().catch(() => false)}`);
    });
  });
});

// ═══════════════════════════════════════════════════
// 场景六：画布操作深度测试
// ═══════════════════════════════════════════════════
test.describe("场景六：画布操作深度测试", () => {
  test("6.1-6.3 键盘选择、小地图、模块搜索", async ({ page }) => {
    await enterWorkspace(page);
    await switchToCodeTab(page);

    await step(page, "点击节点并 Delete 删除", async () => {
      const node = page.locator(".react-flow__node").last();
      if (await node.isVisible().catch(() => false)) {
        await node.click();
        await page.keyboard.press("Delete");
        await page.waitForTimeout(500);
      }
    });

    await step(page, "Ctrl+Z 撤销", async () => {
      await page.keyboard.press("Control+z");
      await page.waitForTimeout(800);
    });

    await step(page, "鼠标滚轮缩放画布", async () => {
      const canvas = page.locator(".react-flow__pane").first();
      const box = await canvas.boundingBox();
      if (box) {
        await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
        await page.mouse.wheel(0, -100);
        await page.waitForTimeout(500);
        await page.mouse.wheel(0, 100);
        await page.waitForTimeout(500);
      }
    });

    await step(page, "框选多个节点", async () => {
      const canvas = page.locator(".react-flow__pane").first();
      const box = await canvas.boundingBox();
      if (box) {
        await page.mouse.move(box.x + 50, box.y + 50);
        await page.mouse.down();
        await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2, { steps: 10 });
        await page.mouse.up();
        await page.waitForTimeout(500);
      }
    });

    await step(page, "模块搜索：输入 RSI", async () => {
      const searchInput = page.locator("input[placeholder*='搜索'], input[placeholder*='search'], input[placeholder*='Search'], input[placeholder*='筛选']").first();
      if (await searchInput.isVisible().catch(() => false)) {
        await searchInput.fill("RSI");
        await page.waitForTimeout(1000);
      }
    });

    await step(page, "清空搜索", async () => {
      const searchInput = page.locator("input[placeholder*='搜索'], input[placeholder*='search'], input[placeholder*='Search'], input[placeholder*='筛选']").first();
      if (await searchInput.isVisible().catch(() => false)) {
        await searchInput.fill("");
        await page.waitForTimeout(500);
      }
    });
  });
});

// ═══════════════════════════════════════════════════
// 场景七：策略工作区标签页
// ═══════════════════════════════════════════════════
test.describe("场景七：策略工作区标签页", () => {
  test("7.1 四个标签页切换", async ({ page }) => {
    await enterWorkspace(page);

    const tabs = [
      { id: "overview", label: "总览" },
      { id: "code", label: "构建" },
      { id: "diagnostics", label: "诊断" },
      { id: "research", label: "研究" },
    ];

    for (const tab of tabs) {
      await step(page, `切换到 ${tab.label} 标签`, async () => {
        const tabEl = page.getByTestId(`workspace-tab-${tab.id}`);
        if (await tabEl.isVisible().catch(() => false)) {
          await tabEl.click();
          await page.waitForTimeout(1500);
        }
      });
    }
  });
});

// ═══════════════════════════════════════════════════
// 场景八：多交易对和多交易所
// ═══════════════════════════════════════════════════
test.describe("场景八：多交易对和多交易所", () => {
  test("8.1-8.2 ETHUSDT 和 OKX", async ({ page }) => {
    await enterWorkspace(page);
    await switchToCodeTab(page);

    // Click K-line node
    await step(page, "点击 K 线节点查看交易对下拉框", async () => {
      const klineNode = page.locator(".react-flow__node").filter({ hasText: /K线|K 线|klines|data/i }).first();
      if (await klineNode.isVisible().catch(() => false)) {
        await klineNode.click();
        await page.waitForTimeout(500);
      }
    });

    await step(page, "检查交易对下拉选项（BTCUSDT/ETHUSDT/SOLUSDT）", async () => {
      const selects = page.locator("select");
      const count = await selects.count();
      console.log(`Select elements found: ${count}`);
      for (let i = 0; i < count; i++) {
        const options = await selects.nth(i).locator("option").allTextContents();
        if (options.some(o => o.includes("BTC") || o.includes("ETH"))) {
          console.log(`Select ${i} options: ${options.join(", ")}`);
        }
      }
    });

    await step(page, "修改交易对为 ETHUSDT (如可选)", async () => {
      const symbolSelect = page.locator("select").first();
      if (await symbolSelect.isVisible().catch(() => false)) {
        await symbolSelect.selectOption({ label: /ETHUSDT/i }).catch(() => {});
        await page.waitForTimeout(500);
      }
    });

    await step(page, "修改交易所为 OKX (如可选)", async () => {
      const selects = page.locator("select");
      const count = await selects.count();
      for (let i = 0; i < count; i++) {
        const options = await selects.nth(i).locator("option").allTextContents();
        if (options.some(o => o.includes("Binance") || o.includes("OKX"))) {
          await selects.nth(i).selectOption({ label: /OKX/i }).catch(() => {});
          break;
        }
      }
      await page.waitForTimeout(500);
    });
  });
});

// ═══════════════════════════════════════════════════
// 场景十：i18n 和 UI 细节
// ═══════════════════════════════════════════════════
test.describe("场景十：i18n 和 UI 细节", () => {
  test("10.1-10.3 国际化 + 加载状态 + 响应式", async ({ page }) => {
    await page.goto("/strategies");
    await page.waitForTimeout(2000);

    await step(page, "检查中文 UI 文本", async () => {
      const body = await page.locator("body").innerText();
      const checks = ["策略", "审批", "告警", "快照", "故障手册", "混沌", "QuantPilot"];
      for (const text of checks) {
        console.log(`Text "${text}": ${body.includes(text) ? "FOUND" : "MISSING"}`);
      }
    });

    await step(page, "刷新页面检查加载状态", async () => {
      await page.reload();
      await page.waitForTimeout(2000);
    });

    await step(page, "缩小窗口测试响应式 (800x600)", async () => {
      await page.setViewportSize({ width: 800, height: 600 });
      await page.waitForTimeout(1500);
    });

    await step(page, "恢复正常视口", async () => {
      await page.setViewportSize({ width: 1440, height: 900 });
    });
  });
});

// ═══════════════════════════════════════════════════
// 场景十一：资源管理 — 保存、丢弃、删除
// ═══════════════════════════════════════════════════
test.describe("场景十一：资源管理", () => {
  test("11.1-11.2 运行记录管理 + 策略删除", async ({ page }) => {
    await enterWorkspace(page);
    await switchToResearchTab(page);

    await step(page, "检查运行历史", async () => {
      const body = await page.locator("body").innerText();
      console.log(`Has run history: ${body.includes("运行") || body.includes("模拟") || body.includes("run")}`);
    });

    // Go to strategy hub
    await step(page, "返回策略中心", async () => {
      await page.goto("/strategies");
      await page.waitForTimeout(2000);
    });

    await step(page, "检查花名册中的删除按钮", async () => {
      const deleteBtn = page.getByRole("button", { name: /删除|Delete/i }).first();
      console.log(`Delete button visible: ${await deleteBtn.isVisible().catch(() => false)}`);
    });
  });
});

test.afterAll(() => {
  console.log(`\n=== 测试完成 ===`);
  console.log(`总步骤数: ${stepNum}`);
  console.log(`失败步骤数: ${failureCount}`);
  console.log(`通过率: ${stepNum > 0 ? Math.round((1 - failureCount / stepNum) * 100) : 0}%`);
});
