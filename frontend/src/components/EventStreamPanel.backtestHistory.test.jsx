import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen, within } from "@testing-library/react";
import EventStreamPanel from "./EventStreamPanel";
import { useGraphStore } from "../store/graphStore";
import { buildBacktestSuccessFixture } from "../test/fixtures/runtime/backtestSuccess";

const { navigateTo } = vi.hoisted(() => ({
  navigateTo: vi.fn()
}));

vi.mock("./AssetCandlesPanel", () => ({
  default: () => <div data-testid="asset-candles-panel-stub" />
}));

vi.mock("../router", () => ({
  navigateTo,
  backtestComparePath: (backtestIds) =>
    `/backtests/compare?ids=${encodeURIComponent(backtestIds.join(","))}`
}));

function buildGraph(overrides = {}) {
  return {
    metadata: {
      name: "Backtest History Graph",
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

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function toDateTimeLocal(ms) {
  const date = new Date(ms);
  const offset = date.getTimezoneOffset() * 60_000;
  return new Date(ms - offset).toISOString().slice(0, 16);
}

function buildExecutionAssumptionsTag(label, sourcesLabel) {
  return {
    label,
    sources_label: sourcesLabel
  };
}

describe("EventStreamPanel backtest history filters and compare entry", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    const fixtureA = buildBacktestSuccessFixture({
      graphId: "artifact_graph",
      compileId: "compile_a",
      backtestId: "backtest_a"
    });
    const fixtureB = buildBacktestSuccessFixture({
      graphId: "artifact_graph",
      compileId: "compile_b",
      backtestId: "backtest_b"
    });
    const fixtureC = buildBacktestSuccessFixture({
      graphId: "artifact_graph",
      compileId: "compile_c",
      backtestId: "backtest_c"
    });

    const itemA = clone(fixtureA.historyResponse[0]);
    itemA.filters.dataset_labels = ["Binance:BTCUSDT:1m"];
    itemA.filters.execution_assumptions_tag = buildExecutionAssumptionsTag(
      "fee=10 slip=5 lat=0",
      "fee:backend slip:backend lat:backend"
    );
    itemA.filters.started_at_ms = 1_700_000_000_000;
    itemA.filters.ended_at_ms = 1_700_000_060_000;
    itemA.created_at_ms = 1_700_000_060_000;

    const itemB = clone(fixtureB.historyResponse[0]);
    itemB.filters.dataset_labels = ["Binance:ETHUSDT:5m"];
    itemB.filters.execution_assumptions_tag = buildExecutionAssumptionsTag(
      "fee=20 slip=8 lat=25",
      "fee:backend slip:backend lat:backend"
    );
    itemB.filters.started_at_ms = 1_700_086_400_000;
    itemB.filters.ended_at_ms = 1_700_086_760_000;
    itemB.created_at_ms = 1_700_086_760_000;

    const itemC = clone(fixtureC.historyResponse[0]);
    itemC.filters.dataset_labels = ["Binance:SOLUSDT:15m"];
    itemC.filters.execution_assumptions_tag = buildExecutionAssumptionsTag(
      "fee=5 slip=2 lat=0",
      "fee:backend slip:backend lat:backend"
    );
    itemC.filters.started_at_ms = 1_700_172_800_000;
    itemC.filters.ended_at_ms = 1_700_173_160_000;
    itemC.created_at_ms = 1_700_173_160_000;

    act(() => {
      useGraphStore.setState(initialState, true);
      useGraphStore.setState({
        graph: buildGraph(),
        runtime: {
          ...useGraphStore.getState().runtime,
          backtestHistory: [itemA, itemB, itemC],
          backtestHistoryStatus: "ready",
          backtestPageSize: 10
        },
        refreshBacktestHistory: vi.fn(),
        loadBacktestDetail: vi.fn()
      });
    });
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
    navigateTo.mockReset();
  });

  it("filters by dataset, parameters, time window, and opens compare route", () => {
    render(<EventStreamPanel />);
    const backtestHistoryCard = screen.getByTestId("backtest-history-card");
    const filterInputs = backtestHistoryCard.querySelectorAll(".history-filter-grid-backtest input");

    expect(filterInputs).toHaveLength(6);

    fireEvent.change(filterInputs[2], {
      target: { value: "ETHUSDT" }
    });
    expect(screen.getByTestId("backtest-history-row-backtest_b")).toBeInTheDocument();
    expect(screen.queryByTestId("backtest-history-row-backtest_a")).not.toBeInTheDocument();

    fireEvent.change(filterInputs[3], {
      target: { value: "fee=20" }
    });
    expect(screen.getByTestId("backtest-history-row-backtest_b")).toBeInTheDocument();

    fireEvent.change(filterInputs[4], {
      target: { value: toDateTimeLocal(1_700_086_400_000) }
    });
    fireEvent.change(filterInputs[5], {
      target: { value: toDateTimeLocal(1_700_086_760_000) }
    });
    expect(screen.getByTestId("backtest-history-row-backtest_b")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("backtest-history-reset"));
    expect(screen.getByTestId("backtest-history-row-backtest_a")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-history-row-backtest_b")).toBeInTheDocument();
    expect(screen.getByTestId("backtest-history-row-backtest_c")).toBeInTheDocument();

    const rowA = screen.getByTestId("backtest-history-row-backtest_a");
    const rowB = screen.getByTestId("backtest-history-row-backtest_b");
    expect(rowA).toBeInTheDocument();
    expect(rowB).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("backtest-history-compare-toggle-backtest_a"));
    fireEvent.click(screen.getByTestId("backtest-history-compare-toggle-backtest_b"));
    fireEvent.click(within(backtestHistoryCard).getByTestId("backtest-history-open-compare"));

    expect(navigateTo).toHaveBeenCalledTimes(1);
    const target = navigateTo.mock.calls[0][0];
    const ids = new URL(`https://example.test${target}`).searchParams.get("ids").split(",");
    expect(ids).toHaveLength(2);
    expect(ids).toContain("backtest_a");
    expect(ids).toContain("backtest_b");
  });
});
