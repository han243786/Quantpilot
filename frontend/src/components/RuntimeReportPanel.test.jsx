import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import RuntimeReportPanel from "./RuntimeReportPanel";

const fetchRuntimeReports = vi.fn();
const createRuntimeReport = vi.fn();
const fetchRuntimeReportDetail = vi.fn();

vi.mock("../store/graphStoreRuntimeHistoryApi", () => ({
  fetchRuntimeReports: (...args) => fetchRuntimeReports(...args),
  createRuntimeReport: (...args) => createRuntimeReport(...args),
  fetchRuntimeReportDetail: (...args) => fetchRuntimeReportDetail(...args),
  runtimeReportExportPath: (reportId) => `/runtime/reports/${reportId}/export`
}));

function readyReport(overrides = {}) {
  return {
    report_id: "report_run_run_001_abcdef123456",
    source_kind: "run",
    source_id: "run_001",
    graph_id: "graph_001",
    status: "ready",
    source_sequence_range: { from: 1, to: 8 },
    source_event_count: 10,
    retained_event_count: 4,
    governance: {
      capability_hash: "sha256:capability-1234567890abcdef",
      deployment_revision: "rev-20260428",
      strategy_version: "strategy-v1",
      parameter_version: "params-v1"
    },
    generation_policy: "quantpilot/report-policy/v1",
    artifacts: [
      {
        kind: "evidence_report",
        artifact_id: "report_run_run_001_abcdef123456_evidence_report",
        file_name: "report_run_run_001_abcdef123456_evidence_report.json",
        content_type: "application/json"
      }
    ],
    created_at_ms: 1_710_000_000_000,
    updated_at_ms: 1_710_000_000_000,
    ...overrides
  };
}

function evidenceSource() {
  const governance = {
    capability_hash: "sha256:capability-1234567890abcdef",
    deployment_revision: "rev-20260428",
    strategy_version: "strategy-v1",
    parameter_version: "params-v1"
  };
  const entries = [
    {
      timeline_item_version: 1,
      event_id: "evt_capability",
      event_type: "CapabilitySnapshotTaken",
      sequence_no: 1,
      occurred_at_ms: 1_710_000_000_000,
      ingested_at_ms: 1_710_000_000_000,
      stage: "system",
      retention_class: "key",
      severity: "Info",
      module_key: "runtime_governance",
      node_id: "runtime",
      summary: "Capability snapshot",
      reason_code: null,
      governance,
      payload_version: 1,
      compactability: "retain"
    },
    {
      timeline_item_version: 1,
      event_id: "evt_risk",
      event_type: "RiskDecisionProduced",
      sequence_no: 3,
      occurred_at_ms: 1_710_000_000_003,
      ingested_at_ms: 1_710_000_000_003,
      stage: "risk",
      retention_class: "key",
      severity: "Warn",
      module_key: "builtin.risk.global",
      node_id: "risk",
      summary: "Risk clamp",
      reason_code: "MAX_WEIGHT_CLAMPED",
      governance,
      payload_version: 1,
      compactability: "retain"
    }
  ];
  return {
    compact_evidence: {
      projection_version: 1,
      policy_version: "quantpilot/evidence-compaction/v1",
      source_event_count: 10,
      retained_event_count: 2,
      dropped_event_count: 8,
      dropped_by_retention: { debug: 8 },
      dropped_by_stage: { agent: 8 },
      key_event_count: 2,
      system_event_count: 1,
      governance,
      entries
    }
  };
}

describe("RuntimeReportPanel", () => {
  beforeEach(() => {
    fetchRuntimeReports.mockReset();
    createRuntimeReport.mockReset();
    fetchRuntimeReportDetail.mockReset();
  });

  it("loads existing reports and exposes detail/export actions", async () => {
    const report = readyReport();
    fetchRuntimeReports.mockResolvedValueOnce([report]);
    fetchRuntimeReportDetail.mockResolvedValueOnce(report);

    render(
      <RuntimeReportPanel sourceKind="run" sourceId="run_001" evidenceSource={evidenceSource()} />
    );

    await waitFor(() => expect(fetchRuntimeReports).toHaveBeenCalled());

    expect(screen.getByTestId("runtime-report-panel")).toHaveTextContent("已就绪");
    expect(screen.getByTestId("runtime-report-panel")).toHaveTextContent("1-8");
    expect(screen.getByTestId("runtime-report-panel")).toHaveTextContent("4/10");
    expect(screen.getByTestId("runtime-report-export")).toHaveAttribute(
      "href",
      "/api/runtime/reports/report_run_run_001_abcdef123456/export"
    );
    expect(screen.getByTestId("runtime-report-evidence-summary")).toHaveTextContent("压缩视图");
    expect(screen.getByTestId("runtime-report-evidence-summary-risk_decisions")).toHaveTextContent(
      "1"
    );

    fireEvent.click(screen.getByTestId(`runtime-report-list-item-${report.report_id}`));

    await waitFor(() =>
      expect(fetchRuntimeReportDetail).toHaveBeenCalledWith("report_run_run_001_abcdef123456")
    );
  });

  it("generates a report for the selected source", async () => {
    const report = readyReport();
    fetchRuntimeReports.mockResolvedValueOnce([]);
    createRuntimeReport.mockResolvedValueOnce(report);

    render(<RuntimeReportPanel sourceKind="run" sourceId="run_001" />);

    await waitFor(() => expect(fetchRuntimeReports).toHaveBeenCalled());
    expect(screen.getByTestId("runtime-report-panel")).toHaveTextContent("未生成");

    fireEvent.click(screen.getByTestId("runtime-report-generate"));

    await waitFor(() =>
      expect(createRuntimeReport).toHaveBeenCalledWith({
        source_kind: "run",
        source_id: "run_001"
      })
    );
    expect(screen.getByTestId("runtime-report-panel")).toHaveTextContent(
      "report_run_run_001_abcdef123456"
    );
    expect(screen.getByTestId("runtime-report-reveal")).toHaveTextContent("打开报告");
  });
});
