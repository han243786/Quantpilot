import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import BacktestHistorySection from "./BacktestHistorySection";

const { navigateTo } = vi.hoisted(() => ({
  navigateTo: vi.fn()
}));

vi.mock("../router", () => ({
  backtestComparePath: (ids, graphId = "") =>
    graphId
      ? `/backtests/compare?ids=${ids.join(",")}&strategy=${graphId}`
      : `/backtests/compare?ids=${ids.join(",")}`,
  navigateTo
}));

function buildProps(overrides = {}) {
  return {
    detailMode: false,
    graph: {
      metadata: {
        graph_id: "graph_alpha"
      }
    },
    runtime: {
      selectedBacktestId: "bt_001",
      diagnostics: null,
      backtestHistoryStatus: "ready"
    },
    backtestHistoryFilter: "graph_alpha",
    backtestCompileFilter: "",
    backtestDatasetFilter: "",
    backtestParameterFilter: "",
    backtestFromTime: "",
    backtestToTime: "",
    backtestPageSize: 6,
    pagedBacktests: [
      {
        backtest_id: "bt_001",
        graph_id: "graph_alpha",
        compile_id: "compile_001",
        created_at_ms: 1_710_000_000_000,
        summary: {
          total_return_ratio: 0.12,
          trade_count: 5,
          risk_adjusted: {
            sharpe_ratio: 1.2
          }
        },
        filters: {
          dataset_labels: ["BTCUSDT-1h"],
          execution_assumptions_tag: {
            label: "fee=10 slip=5",
            sources_label: "backend"
          },
          started_at_ms: 1_710_000_000_000,
          ended_at_ms: 1_710_003_600_000,
          replay_source: "snapshot"
        }
      }
    ],
    filteredBacktests: [{ backtest_id: "bt_001" }],
    backtestCurrentPage: 2,
    backtestTotalPages: 3,
    compareSelection: ["bt_001", "bt_002"],
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
    onOpenBacktestDetail: vi.fn(),
    ...overrides
  };
}

describe("BacktestHistorySection", () => {
  it("renders backtest history controls and wires refresh, filters, compare, detail, and pagination", () => {
    navigateTo.mockClear();
    const props = buildProps();
    const { container } = render(<BacktestHistorySection {...props} />);

    expect(screen.getByTestId("backtest-history-card")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("backtest-history-refresh"));
    expect(props.handleRefreshBacktestHistory).toHaveBeenCalledTimes(1);

    const filterInputs = container.querySelectorAll(".history-filter-grid-backtest input");
    fireEvent.change(filterInputs[0], { target: { value: "graph_beta" } });
    fireEvent.change(filterInputs[1], { target: { value: "compile_beta" } });
    fireEvent.change(filterInputs[2], { target: { value: "ETHUSDT" } });
    fireEvent.change(filterInputs[3], { target: { value: "fee=20" } });
    fireEvent.change(filterInputs[4], { target: { value: "2024-03-01T00:00" } });
    fireEvent.change(filterInputs[5], { target: { value: "2024-03-02T00:00" } });

    expect(props.setBacktestHistoryFilter).toHaveBeenCalledWith("graph_beta");
    expect(props.setBacktestCompileFilter).toHaveBeenCalledWith("compile_beta");
    expect(props.setBacktestDatasetFilter).toHaveBeenCalledWith("ETHUSDT");
    expect(props.setBacktestParameterFilter).toHaveBeenCalledWith("fee=20");
    expect(props.setBacktestFromTime).toHaveBeenCalledWith("2024-03-01T00:00");
    expect(props.setBacktestToTime).toHaveBeenCalledWith("2024-03-02T00:00");

    fireEvent.change(
      container.querySelector(".history-control-bar-backtest .history-page-size-select"),
      {
        target: { value: "20" }
      }
    );
    expect(props.setBacktestPageSize).toHaveBeenCalledWith(20);

    fireEvent.click(screen.getByTestId("backtest-history-open-compare"));
    expect(navigateTo).toHaveBeenCalledWith(
      "/backtests/compare?ids=bt_001,bt_002&strategy=graph_alpha"
    );

    fireEvent.click(screen.getByTestId("backtest-history-reset"));
    expect(props.setBacktestHistoryFilter).toHaveBeenCalledWith("");
    expect(props.setBacktestCompileFilter).toHaveBeenCalledWith("");
    expect(props.setBacktestDatasetFilter).toHaveBeenCalledWith("");
    expect(props.setBacktestParameterFilter).toHaveBeenCalledWith("");
    expect(props.setBacktestFromTime).toHaveBeenCalledWith("");
    expect(props.setBacktestToTime).toHaveBeenCalledWith("");
    expect(props.setBacktestPage).toHaveBeenCalledWith(1);
    expect(props.clearBacktestCompareSelection).toHaveBeenCalledTimes(1);

    fireEvent.click(screen.getByTestId("backtest-history-compare-toggle-bt_001"));
    expect(props.toggleBacktestCompareSelection).toHaveBeenCalledWith("bt_001");

    fireEvent.click(screen.getByTestId("backtest-history-row-bt_001").querySelector(".run-history-item"));
    expect(props.onOpenBacktestDetail).toHaveBeenCalledWith("bt_001");

    const paginationButtons = container.querySelectorAll(".history-pagination button");
    fireEvent.click(paginationButtons[0]);
    fireEvent.click(paginationButtons[1]);
    expect(props.setBacktestPage).toHaveBeenCalledWith(1);
    expect(props.setBacktestPage).toHaveBeenCalledWith(3);
  });

  it("falls back to loading backtest detail when no routed detail handler is provided", () => {
    const props = buildProps({ onOpenBacktestDetail: null });
    render(<BacktestHistorySection {...props} />);

    fireEvent.click(screen.getByTestId("backtest-history-row-bt_001").querySelector(".run-history-item"));

    expect(props.loadBacktestDetail).toHaveBeenCalledWith("bt_001");
  });

  it("does not render inside detail mode", () => {
    const { container } = render(<BacktestHistorySection {...buildProps({ detailMode: true })} />);

    expect(container).toBeEmptyDOMElement();
  });
});
