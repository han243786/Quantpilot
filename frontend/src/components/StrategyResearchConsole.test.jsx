import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import StrategyResearchConsole from "./StrategyResearchConsole";

vi.mock("./EventStreamPanel", () => ({
  EventPanelIntro: ({ runtime, displayedEvents }) => (
    <div data-testid="research-intro">
      intro:{runtime.status}:{displayedEvents.length}
    </div>
  )
}));

vi.mock("../hooks/useStrategyResearchModel", () => ({
  useStrategyResearchModel: () => ({
    graph: { metadata: { graph_id: "workspace_test_graph" } },
    runtime: { status: "running" },
    displayedEvents: [{ event_id: "evt_1" }, { event_id: "evt_2" }],
    filteredEvents: [{ event_id: "evt_1" }],
    filteredHistory: [{ run_id: "run_1" }, { run_id: "run_2" }, { run_id: "run_3" }],
    filteredBacktests: [{ backtest_id: "bt_1" }, { backtest_id: "bt_2" }],
    eventTypes: ["all", "IntentTriggered", "ExecutionFilled"],
    openOrders: [{ order_id: "ord_1" }],
    compareSelection: ["bt_1"],
    dataQualitySummary: {
      value: "1/2",
      note: "data_node · 健康度 告警 · 缺口 2",
      tone: "warning",
      sourceHealthLabel: "告警"
    },
    selectedBacktestSummary: null,
    backtestSummary: null,
    backtestStartedAt: null,
    backtestEndedAt: null,
    pagedBacktests: [],
    backtestCurrentPage: 1,
    backtestTotalPages: 1,
    pagedHistory: [],
    currentPage: 1,
    totalPages: 1,
    panelNotice: null,
    setPanelNotice: vi.fn(),
    handleRefreshBacktestHistory: vi.fn(),
    setBacktestHistoryFilter: vi.fn(),
    setBacktestCompileFilter: vi.fn(),
    setBacktestDatasetFilter: vi.fn(),
    setBacktestParameterFilter: vi.fn(),
    setBacktestFromTime: vi.fn(),
    setBacktestToTime: vi.fn(),
    setBacktestPage: vi.fn(),
    setBacktestPageSize: vi.fn(),
    toggleBacktestCompareSelection: vi.fn(),
    clearBacktestCompareSelection: vi.fn(),
    loadBacktestDetail: vi.fn(),
    handleRefreshRunHistory: vi.fn(),
    setRunHistoryFilter: vi.fn(),
    setRunHistoryCompileFilter: vi.fn(),
    setRunHistoryFromTime: vi.fn(),
    setRunHistoryToTime: vi.fn(),
    setRunHistoryStatusFilter: vi.fn(),
    setRunHistorySortOrder: vi.fn(),
    setRunHistoryPage: vi.fn(),
    setRunHistoryPageSize: vi.fn(),
    loadRunDetail: vi.fn(),
    setEventTypeFilter: vi.fn(),
    setEventSearchTerm: vi.fn(),
    setSelectedNode: vi.fn()
  })
}));

vi.mock("./StrategyBacktestsPanel", () => ({
  default: ({ className = "", showSummary = true, showHistory = true }) => (
    <div data-testid="research-backtests" data-class={className}>
      backtests:{showSummary ? "summary" : "no-summary"}:{showHistory ? "history" : "no-history"}
    </div>
  )
}));

vi.mock("./StrategyRunsPanel", () => ({
  default: ({ className = "", showAccount = true, showHistory = true }) => (
    <div data-testid="research-runs" data-class={className}>
      runs:{showAccount ? "account" : "no-account"}:{showHistory ? "history" : "no-history"}
    </div>
  )
}));

vi.mock("./StrategyEventsPanel", () => ({
  default: ({ className = "" }) => (
    <div data-testid="research-events" data-class={className}>
      events
    </div>
  )
}));

describe("StrategyResearchConsole", () => {
  it("renders a primary research mode with a supporting side rail and data quality summary", () => {
    render(<StrategyResearchConsole />);

    expect(screen.getByTestId("strategy-research-console")).toBeInTheDocument();
    expect(screen.getByTestId("research-intro")).toHaveTextContent("intro:running:2");
    expect(screen.getByTestId("research-title")).toBeInTheDocument();
    expect(screen.getByTestId("research-tab-backtests")).toBeInTheDocument();
    expect(screen.getByTestId("research-tab-runs")).toBeInTheDocument();
    expect(screen.getByTestId("research-primary-mode")).toHaveTextContent("回测");
    expect(screen.getByTestId("research-data-quality-card")).toHaveTextContent("1/2");
    expect(screen.getByTestId("research-data-quality-pill")).toHaveTextContent("告警");
    expect(screen.getByTestId("research-data-quality-note")).toHaveTextContent("data_node");
    expect(screen.getByTestId("research-events")).toHaveTextContent("events");

    expect(screen.getAllByTestId("research-backtests")[0]).toHaveTextContent(
      "backtests:no-summary:history"
    );
    expect(screen.getAllByTestId("research-runs")[0]).toHaveTextContent("runs:account:no-history");

    fireEvent.click(screen.getByTestId("research-tab-runs"));

    expect(screen.getAllByTestId("research-runs")[0]).toHaveTextContent("runs:no-account:history");
    expect(screen.getAllByTestId("research-backtests")[0]).toHaveTextContent(
      "backtests:summary:no-history"
    );
    expect(screen.getByTestId("research-primary-mode")).toHaveTextContent("运行");
    expect(screen.getByTestId("research-main-panel")).toBeInTheDocument();
    expect(screen.getByTestId("research-side-panel")).toBeInTheDocument();
  });
});
