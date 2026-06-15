import { test, expect } from "@playwright/test";

import { backendCapabilitiesFixture } from "../../src/test/fixtures/capabilities/capabilityFallbacks";
import { createApiMockHarness } from "./support/apiHarness";
import { installWorkspaceBootstrapMocks } from "./support/workspaceBootstrapMocks";
import { buildWorkspaceGraphFixture } from "./support/workspaceGraphFixture";

const visualAlertFixture = {
  firings: [],
  rules: [
    {
      rule_name: "runtime_error_budget",
      severity: "P1",
      enabled: true,
      description: "运行时错误预算超过阈值时触发。",
      action: "暂停自动部署并检查最近的运行报告。"
    },
    {
      rule_name: "market_data_staleness",
      severity: "P2",
      enabled: true,
      description: "市场数据延迟超过可接受窗口。",
      action: "切换数据源并刷新运行时能力快照。"
    },
    {
      rule_name: "snapshot_missing",
      severity: "P3",
      enabled: true,
      description: "部署后缺少签名快照。",
      action: "创建签名快照并记录恢复审计。"
    }
  ]
};

const visualSnapshotFixture = {
  data: [
    {
      snapshot_id: "snap-visual-001",
      deployment_revision: "visual-regression-rev",
      capability_hash: "sha256:visual-capability",
      strategy_version: "v1",
      parameter_version: "p1",
      core_ir_digest: "sha256:visual-core-ir",
      event_slice_bounds: {
        from_event_id: "evt-visual-000",
        to_event_id: "evt-visual-010",
        from_sequence: 0,
        to_sequence: 10,
        event_count: 10
      },
      created_at_ms: 1_700_000_000_000,
      signature:
        "sha256:7b2e8f630dfe9f4c7d7e79fdd6b7a9e2f7a1c5d0f88d5b6d3c6f7f9e0a4b2c1d"
    }
  ],
  total: 1,
  limit: 20,
  offset: 0
};

const visualRunbookFixture = [
  {
    scenario_id: "runtime_compile_rejected",
    name: "运行时编译被拒绝",
    severity: "P1",
    symptoms: ["编译按钮返回错误", "运行按钮保持禁用", "能力快照提示不一致"],
    diagnostic_steps: [
      {
        step_number: 1,
        description: "检查运行时编译输出和策略图校验状态。",
        api_call: "POST /api/runtime/compile"
      },
      {
        step_number: 2,
        description: "确认前端能力矩阵与后端能力快照一致。",
        api_call: "GET /api/capabilities"
      }
    ],
    recovery_steps: [
      {
        step_number: 1,
        condition: "能力快照漂移",
        action: "刷新能力快照并重新加载策略图。"
      },
      {
        step_number: 2,
        condition: "策略图不可运行",
        action: "修复阻断诊断后重新编译。"
      }
    ],
    verification: "编译通过且运行按钮恢复可用。"
  },
  {
    scenario_id: "snapshot_restore_required",
    name: "部署快照需要恢复",
    severity: "P2",
    symptoms: ["部署版本回退", "审计记录缺少恢复说明"],
    diagnostic_steps: [
      {
        step_number: 1,
        description: "列出签名快照并确认目标版本存在。",
        api_call: "GET /api/v1/snapshots"
      }
    ],
    recovery_steps: [
      {
        step_number: 1,
        condition: "签名校验通过",
        action: "恢复到目标签名快照并写入审计。"
      }
    ],
    verification: "目标部署版本、策略版本和参数版本与快照一致。"
  }
];

async function installVisualRegressionMocks(page) {
  await page.addInitScript(() => {
    window.localStorage?.setItem("qp.tutorial.seen", "1");
    window.localStorage?.setItem("quantpilot.tutorial.seen", "1");
    window.localStorage?.setItem("quantpilot.theme", "dark");
    const fixedNow = 1_700_000_000_000;
    const RealDate = Date;
    class FixedDate extends RealDate {
      constructor(...args) {
        super(...(args.length > 0 ? args : [fixedNow]));
      }

      static now() {
        return fixedNow;
      }
    }
    FixedDate.UTC = RealDate.UTC;
    FixedDate.parse = RealDate.parse;
    FixedDate.prototype = RealDate.prototype;
    window.Date = FixedDate;
  });
  const api = await createApiMockHarness(page);
  const graphFixture = buildWorkspaceGraphFixture();
  graphFixture.metadata.updated_at = 1_716_543_299_000;
  await api.json("**/api/capabilities", backendCapabilitiesFixture);
  await installWorkspaceBootstrapMocks(api, { graphFixture });
  await api.json("**/api/v1/alerts", visualAlertFixture);
  await api.json("**/api/v1/snapshots", visualSnapshotFixture);
  await api.json("**/api/v1/runbook", visualRunbookFixture);
  await api.installGuard();
  return api;
}

test.describe("Visual Regression", () => {
  test("策略中心首页布局", async ({ page }) => {
    const api = await installVisualRegressionMocks(page);
    await page.goto("/strategies");
    await page.waitForTimeout(2000);
    await expect(page).toHaveScreenshot("strategy-hub.png", {
      maxDiffPixels: 5000,
      threshold: 0.1,
    });
    api.expectNoUnexpectedApiRequests();
  });

  test("告警页面", async ({ page }) => {
    const api = await installVisualRegressionMocks(page);
    await page.goto("/alerts");
    await page.waitForTimeout(1500);
    await expect(page).toHaveScreenshot("alerts.png", {
      maxDiffPixels: 3000,
      threshold: 0.1,
    });
    api.expectNoUnexpectedApiRequests();
  });

  test("快照页面", async ({ page }) => {
    const api = await installVisualRegressionMocks(page);
    await page.goto("/snapshots");
    await page.waitForTimeout(1500);
    await expect(page).toHaveScreenshot("snapshots.png", {
      maxDiffPixels: 3000,
      threshold: 0.1,
    });
    api.expectNoUnexpectedApiRequests();
  });

  test("故障手册页面", async ({ page }) => {
    const api = await installVisualRegressionMocks(page);
    await page.goto("/runbook");
    await page.waitForTimeout(1500);
    await expect(page).toHaveScreenshot("runbook.png", {
      maxDiffPixels: 3000,
      threshold: 0.1,
    });
    api.expectNoUnexpectedApiRequests();
  });
});
