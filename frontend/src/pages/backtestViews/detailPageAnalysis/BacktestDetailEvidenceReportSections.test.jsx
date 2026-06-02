import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import {
  BacktestDetailGovernedTimelineSection,
  BacktestDetailReportLifecycleSection
} from "./BacktestDetailEvidenceReportSections";

const t = (value) => value;

function buildTimelineSource() {
  const event = {
    timeline_item_version: 1,
    event_id: "evt_risk_detail",
    event_type: "RiskDecisionProduced",
    sequence_no: 2,
    occurred_at_ms: 1_710_000_000_100,
    ingested_at_ms: 1_710_000_000_100,
    stage: "risk",
    retention_class: "key",
    severity: "Warn",
    module_key: "builtin.risk.global",
    node_id: "node_risk_5",
    summary: "Risk clamp applied before execution.",
    reason_code: "MAX_WEIGHT_CLAMPED",
    governance: {
      capability_hash: "sha256:detail-capability-1234567890abcdef",
      deployment_revision: "rev-detail-20260428",
      strategy_version: "strategy-v9",
      parameter_version: "params-v6"
    },
    payload_version: 1,
    compactability: "retain"
  };

  return {
    timeline: [event],
    events: [],
    retained_key_event_index: {
      source_event_count: 1,
      retained_event_count: 1,
      entries: [event]
    },
    compact_evidence: {
      source_event_count: 1,
      retained_event_count: 1,
      entries: [event]
    }
  };
}

describe("BacktestDetailGovernedTimelineSection", () => {
  it("renders the governed timeline with the detail route test ids", () => {
    render(
      <BacktestDetailGovernedTimelineSection
        t={t}
        timelineSource={buildTimelineSource()}
      />
    );

    expect(screen.getByTestId("backtest-detail-governed-timeline")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-detail-timeline")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-detail-timeline-retained-count")).toHaveTextContent("1/1");
    expect(screen.getByTestId("backtest-detail-timeline-stage-risk")).toHaveTextContent("Risk clamp applied before execution.");
  });
});

describe("BacktestDetailReportLifecycleSection", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("renders the runtime report panel with backtest source identity", () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => ({
        ok: true,
        json: async () => []
      }))
    );

    render(
      <BacktestDetailReportLifecycleSection
        t={t}
        sourceId="backtest_artifact_001"
        timelineSource={buildTimelineSource()}
      />
    );

    expect(screen.getByTestId("backtest-detail-report-lifecycle")).toBeInTheDocument();
    expect(screen.getByTestId("runtime-report-panel")).toHaveTextContent("回测证据报告");
    expect(screen.getByTestId("runtime-report-evidence-summary")).toBeInTheDocument();
  });

  it("keeps the report panel hidden when source identity is missing", () => {
    render(
      <BacktestDetailReportLifecycleSection
        t={t}
        sourceId=""
        timelineSource={buildTimelineSource()}
      />
    );

    expect(screen.getByTestId("backtest-detail-report-lifecycle")).toBeInTheDocument();
    expect(screen.queryByTestId("runtime-report-panel")).not.toBeInTheDocument();
  });
});
