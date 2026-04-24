import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, render, screen } from "@testing-library/react";
import BacktestDetailPage from "./BacktestDetailPage";
import { useGraphStore } from "../store/graphStore";
import { buildBacktestSuccessFixture } from "../test/fixtures/runtime/backtestSuccess";

vi.mock("../components/EventStreamPanel", () => ({
  default: () => <div data-testid="event-stream-panel-stub" />
}));

vi.mock("../router", () => ({
  navigateTo: vi.fn(),
  strategiesPath: () => "/strategies",
  strategyBacktestsPath: (strategyId) => `/strategies/${strategyId}/backtests`,
  strategyWorkspacePath: (strategyId) => `/strategies/${strategyId}`
}));

function buildGraph(overrides = {}) {
  return {
    metadata: {
      name: "Backtest Detail Test Graph",
      graph_id: "artifact_graph",
      ...(overrides.metadata || {})
    },
    nodes: [
      { id: "node_risk_5", name: "Risk Guard", type: "risk" },
      { id: "node_execution_7", name: "Execution Desk", type: "execution" }
    ],
    edges: [],
    validation_state: {
      is_valid: true,
      is_runnable: true,
      issue_counts: { error: 0, warning: 0, info: 0 },
      graph_issues: [],
      node_issues: {},
      edge_issues: {}
    },
    compile_summary: {},
    ...overrides
  };
}

describe("BacktestDetailPage artifact projections", () => {
  const initialState = useGraphStore.getState();
  let fixture;

  beforeEach(() => {
    fixture = buildBacktestSuccessFixture({
      graphId: "artifact_graph",
      compileId: "compile_artifact_001",
      backtestId: "backtest_artifact_001"
    });
    fixture.detailResponse.runtime_diagnostics.node_details.node_risk_5 = {
      node_id: "node_risk_5",
      latest_event: null,
      explanation_summary: "Risk clamp applied before execution.",
      latest_input_rows: [],
      latest_output_rows: [],
      explanation_rows: [
        {
          key: "explanation_summary",
          label: "解释摘要",
          value: "Risk clamp applied before execution."
        }
      ],
      risk_detail_rows: [
        {
          key: "limit_triggered",
          label: "触发限制",
          value: "max_single_weight"
        },
        {
          key: "post_risk.max_target_weight",
          label: "风控后最大目标权重",
          value: "0.4500"
        }
      ],
      order_detail_rows: [],
      latest_notice: null,
      recent_events: [],
      event_count: 1
    };
    fixture.detailResponse.runtime_diagnostics.node_details.node_execution_7 = {
      ...fixture.detailResponse.runtime_diagnostics.node_details.node_execution_7,
      explanation_summary: "Execution plan sized from portfolio target diff.",
      explanation_rows: [
        {
          key: "explanation_summary",
          label: "解释摘要",
          value: "Execution plan sized from portfolio target diff."
        }
      ],
      order_detail_rows: [
        {
          key: "sizing_source",
          label: "定量来源",
          value: "portfolio_target_diff"
        },
        {
          key: "lifecycle_stage",
          label: "生命周期",
          value: "accepted"
        }
      ]
    };

    act(() => {
      useGraphStore.setState(initialState, true);
      useGraphStore.setState({
        graph: buildGraph(),
        runtime: {
          ...useGraphStore.getState().runtime,
          status: "completed",
          runKind: "backtest",
          runId: fixture.detailResponse.backtest_id,
          selectedBacktestId: fixture.detailResponse.backtest_id,
          backtestHistory: fixture.historyResponse,
          backtestArtifacts: fixture.detailResponse.backtest_artifacts,
          events: fixture.detailResponse.backtest_artifacts.event_log.events,
          account: fixture.detailResponse.account,
          diagnostics: fixture.detailResponse.runtime_diagnostics || null
        }
      });
    });

    vi.stubGlobal(
      "fetch",
      vi.fn(async () => ({
        ok: true,
        json: async () => fixture.detailResponse
      }))
    );
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
    vi.unstubAllGlobals();
  });

  it("renders structured strategy-context detail sections and manifest data", async () => {
    let renderResult;
    await act(async () => {
      renderResult = render(<BacktestDetailPage backtestId="backtest_artifact_001" />);
      await Promise.resolve();
      await Promise.resolve();
    });

    const hero = screen.getByTestId("backtest-detail-hero");
    const coreArtifactsSection = screen.getByTestId("backtest-detail-core-artifacts");
    const replayPreviewSection = screen.getByTestId("backtest-detail-replay-preview");
    const outputArtifactsSection = screen.getByTestId("backtest-detail-output-artifacts");
    const explanationsSection = screen.getByTestId("backtest-detail-explanations");
    const contextSection = screen.getByTestId("backtest-detail-context");
    expect(hero).toBeInTheDocument();
    expect(coreArtifactsSection).toBeInTheDocument();
    expect(replayPreviewSection).toBeInTheDocument();
    expect(outputArtifactsSection).toBeInTheDocument();
    expect(explanationsSection).toBeInTheDocument();
    expect(contextSection).toBeInTheDocument();

    expect(screen.getByTestId("backtest-detail-manifest-card")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-detail-metrics-card")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-detail-equity-card")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-detail-trade-card")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-detail-output-card")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-detail-risk-card")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-detail-order-card")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-detail-context-card")).toBeInTheDocument();

    expect(screen.getByTestId("backtest-detail-manifest-strategy-artifact")).toHaveTextContent("strategy_artifact_smoke_001");
    expect(screen.getByTestId("backtest-detail-manifest-compile-artifact")).toHaveTextContent("compile_artifact_smoke_001");
    expect(screen.getByTestId("backtest-detail-manifest-core-ir-artifact")).toHaveTextContent("core_ir_artifact_smoke_001");

    expect(hero.querySelector(".analysis-summary-grid")).not.toBeNull();
    expect(hero).toHaveTextContent("+12.50%");
    expect(hero).toHaveTextContent("12050");
    expect(screen.getByTestId("backtest-detail-risk-card-entry-node_risk_5")).toHaveTextContent("Risk Guard");
    expect(screen.getByTestId("backtest-detail-risk-card-entry-node_risk_5")).toHaveTextContent("max_single_weight");
    expect(screen.getByTestId("backtest-detail-order-card-entry-node_execution_7")).toHaveTextContent("Execution Desk");
    expect(screen.getByTestId("backtest-detail-order-card-entry-node_execution_7")).toHaveTextContent("portfolio_target_diff");
    expect(screen.getByTestId("backtest-detail-metrics-event-count")).toHaveTextContent("3");
    expect(hero).not.toHaveTextContent("-25.00%");
    expect(hero).not.toHaveTextContent("7500");
  });
});
