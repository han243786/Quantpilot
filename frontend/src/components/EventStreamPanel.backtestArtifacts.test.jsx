import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, render, screen } from "@testing-library/react";
import EventStreamPanel from "./EventStreamPanel";
import { useGraphStore } from "../store/graphStore";
import { buildBacktestSuccessFixture } from "../test/fixtures/runtime/backtestSuccess";

vi.mock("./AssetCandlesPanel", () => ({
  default: () => <div data-testid="asset-candles-panel-stub" />
}));

function buildGraph(overrides = {}) {
  return {
    metadata: {
      name: "Backtest Event Panel Graph",
      graph_id: "artifact_graph",
      ...(overrides.metadata || {})
    },
    nodes: [],
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

describe("EventStreamPanel backtest artifact summary", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    const fixture = buildBacktestSuccessFixture({
      graphId: "artifact_graph",
      compileId: "compile_artifact_001",
      backtestId: "backtest_artifact_001"
    });
    fixture.detailResponse.backtest_artifacts.event_log.events =
      fixture.detailResponse.backtest_artifacts.event_log.events.map((event, index) =>
        index === 0 ? { ...event, summary: "Artifact event log entry" } : event
      );
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
          backtestHistoryStatus: "ready",
          backtestArtifacts: fixture.detailResponse.backtest_artifacts,
          events: [
            {
              event_id: "legacy_event_only",
              event_type: "RuntimeNotice",
              source_id: "legacy_source",
              node_id: "legacy_node",
              event_time_ms: 1_700_000_000_500,
              severity: "Info",
              summary: "Legacy top-level event",
              payload: {}
            }
          ],
          account: fixture.detailResponse.account
        },
        refreshRunHistory: vi.fn(),
        refreshBacktestHistory: vi.fn(),
        loadRunDetail: vi.fn(),
        loadBacktestDetail: vi.fn(),
        setSelectedNode: vi.fn(),
        setRunHistoryFilter: vi.fn(),
        setRunHistoryCompileFilter: vi.fn(),
        setRunHistoryFromTime: vi.fn(),
        setRunHistoryToTime: vi.fn(),
        setRunHistoryStatusFilter: vi.fn(),
        setRunHistorySortOrder: vi.fn(),
        setRunHistoryPage: vi.fn(),
        setRunHistoryPageSize: vi.fn(),
        setBacktestHistoryFilter: vi.fn(),
        setBacktestCompileFilter: vi.fn(),
        setBacktestPage: vi.fn(),
        setBacktestPageSize: vi.fn(),
        setEventTypeFilter: vi.fn(),
        setEventSearchTerm: vi.fn()
      });
    });
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("prefers artifact metrics over legacy backtest summary data", () => {
    const { container } = render(<EventStreamPanel detailMode />);

    const intro = screen.getByTestId("event-panel-intro");
    const summaryCard = screen.getByTestId("backtest-summary-card");
    const summaryMetrics = screen.getByTestId("backtest-summary-metrics");
    const metricCards = summaryMetrics.querySelectorAll(".account-metric-card");
    const configHash = screen.getByTestId("backtest-summary-config-hash");
    const backtestId = screen.getByTestId("backtest-summary-id");
    const eventFeedSection = screen.getByTestId("event-feed-section");
    const eventRows = container.querySelectorAll("[data-testid^='event-feed-row-']");

    expect(summaryCard).toBeInTheDocument();
    expect(summaryMetrics).toBeInTheDocument();
    expect(metricCards.length).toBe(4);
    expect(eventRows.length).toBeGreaterThan(0);
    expect(backtestId).toHaveTextContent("backtest_artifact_001");
    expect(configHash).toHaveTextContent("smoke_backtest_config_hash");
    expect(metricCards[0]).toHaveTextContent("+12.50%");
    expect(metricCards[2]).toHaveTextContent("1");
    expect(metricCards[3]).toHaveTextContent("12050");
    expect(intro.textContent).toContain("3");
    expect(summaryCard.textContent).not.toContain("Legacy top-level event");
    expect(eventFeedSection).toHaveTextContent("Artifact event log entry");
  });
});
