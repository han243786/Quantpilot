import { test, expect } from "@playwright/test";
import { createApiMockHarness } from "./support/apiHarness";
import { installWorkspaceBootstrapMocks } from "./support/workspaceBootstrapMocks";
import { backendCapabilitiesFixture } from "../../src/test/fixtures/capabilities/capabilityFallbacks";
import { buildBacktestSuccessFixture } from "../../src/test/fixtures/runtime/backtestSuccess";

const GRAPH_ID = "draft_graph";
const BACKTEST_ID = "backtest_evidence_walkthrough_001";
const REPORT_ID = "report_evidence_walkthrough_001";

function governance() {
  return {
    capability_hash: "sha256:evidence-capability-1234567890abcdef",
    deployment_revision: "sha256:evidence-deployment-abcdef1234567890",
    strategy_version: "strategy-evidence-v1",
    parameter_version: "params-evidence-v1",
    governance_source: "loaded_manifest",
    permission_boundary: {
      model_version: "quantpilot/permission-boundary/v1",
      ai_write_policy: "proposal_only"
    }
  };
}

function timelineItem(sequenceNo, eventType, stage, summary, overrides = {}) {
  const baseTime = 1_710_000_000_000;
  return {
    timeline_item_version: 1,
    event_id: `evt_evidence_${sequenceNo}`,
    event_type: eventType,
    sequence_no: sequenceNo,
    occurred_at_ms: baseTime + sequenceNo * 1000,
    ingested_at_ms: baseTime + sequenceNo * 1000,
    stage,
    retention_class: "key",
    severity: overrides.severity || "Info",
    module_key: overrides.module_key || `builtin.${stage}`,
    node_id: overrides.node_id || `node_${stage}`,
    summary,
    reason_code: overrides.reason_code || eventType,
    governance: {
      capability_hash: governance().capability_hash,
      deployment_revision: governance().deployment_revision,
      strategy_version: governance().strategy_version,
      parameter_version: governance().parameter_version
    },
    payload_version: 1,
    compactability: "retain"
  };
}

function buildEvidenceBacktestFixture() {
  const fixture = buildBacktestSuccessFixture({
    graphId: GRAPH_ID,
    compileId: "compile_evidence_walkthrough_001",
    backtestId: BACKTEST_ID
  });
  const timeline = [
    timelineItem(1, "CapabilitySnapshotTaken", "system", "Capability boundary captured.", {
      module_key: "runtime_governance",
      node_id: "runtime",
      reason_code: "CAPABILITY_SNAPSHOT"
    }),
    timelineItem(2, "DataUpdated", "data", "BTCUSDT market data healthy.", {
      module_key: "builtin.data.kline",
      node_id: "node_data_2"
    }),
    timelineItem(3, "RiskDecisionProduced", "risk", "Risk decision approved.", {
      module_key: "builtin.risk.global",
      node_id: "node_risk_5"
    }),
    timelineItem(4, "ExecutionFilled", "fill", "Execution filled 0.2 BTC.", {
      module_key: "builtin.execution.paper",
      node_id: "node_execution_7"
    }),
    timelineItem(5, "PortfolioUpdated", "fill", "Portfolio updated after fill.", {
      module_key: "builtin.runtime.control",
      node_id: "node_runtime_1"
    })
  ];

  fixture.detailResponse.governance = governance();
  fixture.detailResponse.backtest_artifacts.manifest.governance = governance();
  fixture.detailResponse.timeline = timeline;
  fixture.detailResponse.retained_key_event_index = {
    index_version: 1,
    policy_version: "quantpilot/key-event-index/v1",
    source_event_count: 8,
    retained_event_count: timeline.length,
    key_event_count: timeline.length,
    system_event_count: 1,
    entries: timeline
  };
  fixture.detailResponse.compact_evidence = {
    projection_version: 1,
    policy_version: "quantpilot/evidence-compaction/v1",
    source_event_count: 8,
    retained_event_count: timeline.length,
    dropped_event_count: 3,
    dropped_by_retention: { summary: 2, debug: 1 },
    dropped_by_stage: { agent: 1, execution: 2 },
    key_event_count: timeline.length,
    system_event_count: 1,
    governance: timeline[0].governance,
    entries: timeline
  };

  return fixture;
}

function reportRecord() {
  return {
    report_id: REPORT_ID,
    source_kind: "backtest",
    source_id: BACKTEST_ID,
    graph_id: GRAPH_ID,
    status: "ready",
    source_sequence_range: { from: 1, to: 5 },
    source_event_count: 8,
    retained_event_count: 5,
    governance: {
      capability_hash: governance().capability_hash,
      deployment_revision: governance().deployment_revision,
      strategy_version: governance().strategy_version,
      parameter_version: governance().parameter_version
    },
    generation_policy: "quantpilot/report-policy/v1",
    artifacts: [
      {
        kind: "json",
        artifact_id: "report_json_evidence_walkthrough_001",
        file_name: `${REPORT_ID}.json`,
        content_type: "application/json"
      }
    ],
    created_at_ms: 1_710_000_010_000,
    updated_at_ms: 1_710_000_010_000
  };
}

function replayWindow(sequenceCursor) {
  const current = Number(sequenceCursor || 1);
  const items =
    current >= 13
      ? [
          timelineItem(13, "ExecutionFilled", "fill", "Second replay page execution."),
          timelineItem(14, "PortfolioUpdated", "fill", "Second replay page portfolio.")
        ]
      : [
          timelineItem(1, "CapabilitySnapshotTaken", "system", "Capability boundary captured."),
          timelineItem(2, "RiskDecisionProduced", "risk", "Risk decision approved.")
        ];
  return {
    kind: "backtest",
    record_id: BACKTEST_ID,
    graph_id: GRAPH_ID,
    source_event_count: 14,
    total_events: 14,
    cursor: current >= 13 ? 12 : 0,
    sequence_cursor: current,
    limit: 12,
    window_end: current >= 13 ? 14 : 12,
    fill_event_count: 1,
    account: {
      equity_estimate: 12050,
      cash_balance: 11500,
      available_cash_balance: 11200,
      frozen_cash_balance: 300,
      total_leverage: 0.15,
      total_gross_notional: 550,
      total_net_notional: 550,
      positions: 1,
      open_order_count: 0,
      open_orders: []
    },
    filters: { key_only: false },
    checkpoints: [
      { cursor: 0, sequence_cursor: 1, label: "1" },
      { cursor: 12, sequence_cursor: 13, label: "13" }
    ],
    events: items.map((item) => ({
      sequence_no: item.sequence_no,
      event: {
        event_id: item.event_id,
        event_type: item.event_type,
        node_id: item.node_id,
        event_time_ms: item.occurred_at_ms,
        severity: item.severity,
        summary: item.summary,
        payload: {}
      }
    })),
    timeline: items,
    previous_cursor: current >= 13 ? 0 : null,
    previous_sequence_cursor: current >= 13 ? 1 : null,
    next_cursor: current >= 13 ? null : 12,
    next_sequence_cursor: current >= 13 ? null : 13
  };
}

test("evidence walkthrough covers timeline, replay paging, compact mode, and report lifecycle", async ({ page }) => {
  const fixture = buildEvidenceBacktestFixture();
  const api = await createApiMockHarness(page);
  let reports = [];

  await api.json("**/api/capabilities", backendCapabilitiesFixture);
  await installWorkspaceBootstrapMocks(api, {
    backtestHistory: fixture.historyResponse
  });

  await api.handle(`**/api/runtime/backtests/${BACKTEST_ID}/replay**`, async (route) => {
    const url = new URL(route.request().url());
    await route.fulfill({
      status: 200,
      contentType: "application/json; charset=utf-8",
      body: JSON.stringify(replayWindow(url.searchParams.get("sequence_cursor")))
    });
  });
  await api.json(`**/api/runtime/backtests/${BACKTEST_ID}`, fixture.detailResponse);
  await api.handle("**/api/runtime/reports", async (route) => {
    if (route.request().method() === "POST") {
      reports = [reportRecord()];
      await route.fulfill({
        status: 200,
        contentType: "application/json; charset=utf-8",
        body: JSON.stringify(reports[0])
      });
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: "application/json; charset=utf-8",
      body: JSON.stringify(reports)
    });
  });
  await api.json(`**/api/runtime/reports/${REPORT_ID}`, reportRecord());
  await api.json(`**/api/runtime/reports/${REPORT_ID}/export`, {
    schema_version: "quantpilot/evidence-report-artifact/v1",
    ...reportRecord(),
    evidence_digest: "sha256:evidence-report-digest",
    loading_strategy: {
      primary_source: "compact_evidence",
      source_event_count: 8,
      retained_event_count: 5,
      requires_detail_window: false
    },
    sections: []
  });
  await api.installGuard();

  await page.goto(`/backtests/${BACKTEST_ID}?strategy=${GRAPH_ID}`);
  await expect(page.getByTestId("backtest-detail-governed-timeline")).toBeVisible();
  await expect(page.getByTestId("backtest-detail-timeline-stage-system")).toContainText(
    "CapabilitySnapshotTaken"
  );
  await expect(page.getByTestId("backtest-detail-timeline-stage-risk")).toContainText("风控");
  await expect(page.getByTestId("backtest-detail-governed-timeline")).not.toContainText("\"event_id\"");

  await page.getByTestId("event-replay-load").click();
  await expect(page.getByTestId("event-replay-window")).toContainText("1-12/14");
  await page.getByTestId("event-replay-next").click();
  await expect(page.getByTestId("event-replay-window")).toContainText("13-14/14");
  await expect(page.getByTestId("event-replay-row-13")).toBeVisible();

  await page.getByTestId("runtime-report-generate").click();
  await expect(page.getByTestId("runtime-report-panel")).toContainText("已就绪");
  await expect(page.getByTestId("runtime-report-evidence-summary")).toContainText("压缩视图");
  await expect(page.getByTestId("runtime-report-evidence-summary-risk_decisions")).toContainText("1");
  await expect(page.getByTestId("runtime-report-export")).toHaveAttribute(
    "href",
    /\/api\/runtime\/reports\/report_evidence_walkthrough_001\/export$/
  );

  api.expectNoUnexpectedApiRequests();
});
