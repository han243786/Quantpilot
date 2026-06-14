import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import {
  BacktestDetailCoreArtifactSections,
  BacktestDetailV4ArtifactSection
} from "./BacktestDetailCoreArtifactSections";

const t = (value) => value;

describe("BacktestDetailCoreArtifactSections", () => {
  it("renders manifest, metrics, governance, and chart sections from explicit inputs", () => {
    render(
      <BacktestDetailCoreArtifactSections
        t={t}
        selectedSummary={{ compile_id: "compile_from_history" }}
        manifest={{
          manifest_id: "manifest_bt_001",
          created_at_ms: 1_700_000_060_000,
          backtest_spec: {
            replay_source: "deterministic_mock",
            run_spec: { schema_version: "quantpilot/run-spec/v1" }
          },
          compile_artifacts: {
            strategy: { artifact_id: "strategy_artifact_001" },
            compile: { artifact_id: "compile_artifact_001" },
            core_ir: { artifact_id: "core_ir_artifact_001" }
          }
        }}
        metrics={{ event_count: 3, session_count: 1 }}
        summary={{ step_count: 12 }}
        startedAt={1_700_000_000_000}
        endedAt={1_700_000_060_000}
        governanceRows={[
          {
            key: "capability_hash",
            label: "capability_hash",
            value: "sha256:detail...abcdef",
            fullValue: "sha256:detail-capability-1234567890abcdef"
          }
        ]}
        equityCurve={[
          { ts_ms: 1_700_000_000_000, equity: 10_000 },
          { ts_ms: 1_700_000_060_000, equity: 12_000 }
        ]}
        periodReturns={[]}
        metricsArtifactId="metrics_artifact_001"
        eventsLength={9}
      />
    );

    expect(screen.getByTestId("backtest-detail-core-artifacts")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-detail-drawdown-chart")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-detail-monthly-returns")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-detail-manifest-card")).toHaveTextContent("manifest_bt_001");
    expect(screen.getByTestId("backtest-detail-metrics-card")).toHaveTextContent("metrics_artifact_001");
    expect(screen.getByTestId("backtest-detail-manifest-strategy-artifact")).toHaveTextContent("strategy_artifact_001");
    expect(screen.getByTestId("backtest-detail-governance-capability_hash")).toHaveTextContent("sha256:detail...abcdef");
    expect(screen.getByTestId("backtest-detail-metrics-step-count")).toHaveTextContent("12");
    expect(screen.getByTestId("backtest-detail-metrics-event-count")).toHaveTextContent("3");
  });
});

describe("BacktestDetailV4ArtifactSection", () => {
  it("renders v4 artifact and microstructure cards when v4 evidence exists", () => {
    render(
      <BacktestDetailV4ArtifactSection
        v4Artifact={{
          schema_version: "quantpilot/v4-backtest-artifact/v1",
          symbols: ["BTCUSDT", "ETHUSDT"],
          replay_mode: "paper_simulated",
          input_bar_count: 120,
          input_tick_count: 14,
          machine_trajectory: [{ state_id: "ready" }, { state_id: "active" }],
          risk_plane_decisions: [{ accepted: true }],
          execution_capability_sources: [{ capability: "submit_order" }]
        }}
        v4MicroMetrics={{
          submitted_order_count: 4,
          fill_rate: 0.75,
          average_slippage_bps: 2.5,
          queue_position_estimate: 0.4,
          vwap_deviation_bps: 1.25
        }}
      />
    );

    expect(screen.getByTestId("backtest-detail-v4-evidence")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-detail-v4-artifact-card")).toHaveTextContent("BTCUSDT, ETHUSDT");
    expect(screen.getByTestId("backtest-detail-v4-artifact-card")).toHaveTextContent("120");
    expect(screen.getByTestId("backtest-detail-v4-microstructure-card")).toHaveTextContent("75.00%");
  });
});
