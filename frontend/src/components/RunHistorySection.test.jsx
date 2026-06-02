import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import RunHistorySection from "./RunHistorySection";

function buildProps(overrides = {}) {
  return {
    detailMode: false,
    graph: {
      metadata: {
        graph_id: "graph_alpha"
      }
    },
    runtime: {
      runId: null,
      diagnostics: null,
      governance: null,
      parameterMutations: [],
      historyStatus: "ready",
      selectedHistoryRunId: "run_001"
    },
    historyFilter: "graph_alpha",
    historyCompileFilter: "",
    historyFromTime: "",
    historyToTime: "",
    historyStatusFilter: "all",
    historySortOrder: "desc",
    historyPageSize: 6,
    pagedHistory: [
      {
        run_id: "run_001",
        graph_id: "graph_alpha",
        compile_id: "compile_001",
        created_at_ms: 1_710_000_000_000,
        status: "completed",
        event_count: 12
      }
    ],
    filteredHistory: [
      {
        run_id: "run_001"
      }
    ],
    currentPage: 2,
    totalPages: 3,
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
    ...overrides
  };
}

describe("RunHistorySection", () => {
  it("renders run history controls and wires refresh, filters, detail, and pagination", () => {
    const props = buildProps();
    const { container } = render(<RunHistorySection {...props} />);

    expect(screen.getByTestId("run-history-card")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("run-history-refresh"));
    expect(props.handleRefreshRunHistory).toHaveBeenCalledTimes(1);

    const filterInputs = container.querySelectorAll(".history-filter-grid-run input");
    fireEvent.change(filterInputs[0], { target: { value: "graph_beta" } });
    fireEvent.change(filterInputs[1], { target: { value: "compile_beta" } });
    fireEvent.change(filterInputs[2], { target: { value: "2024-03-01T00:00" } });
    fireEvent.change(filterInputs[3], { target: { value: "2024-03-02T00:00" } });

    expect(props.setRunHistoryFilter).toHaveBeenCalledWith("graph_beta");
    expect(props.setRunHistoryCompileFilter).toHaveBeenCalledWith("compile_beta");
    expect(props.setRunHistoryFromTime).toHaveBeenCalledWith("2024-03-01T00:00");
    expect(props.setRunHistoryToTime).toHaveBeenCalledWith("2024-03-02T00:00");

    const filterSelects = container.querySelectorAll(".history-filter-grid-run select");
    fireEvent.change(filterSelects[0], { target: { value: "error" } });
    fireEvent.change(filterSelects[1], { target: { value: "asc" } });

    expect(props.setRunHistoryStatusFilter).toHaveBeenCalledWith("error");
    expect(props.setRunHistorySortOrder).toHaveBeenCalledWith("asc");

    fireEvent.change(container.querySelector(".history-control-bar-run .history-page-size-select"), {
      target: { value: "10" }
    });
    expect(props.setRunHistoryPageSize).toHaveBeenCalledWith(10);

    const controlButtons = container.querySelectorAll(".history-control-bar-run button");
    fireEvent.click(controlButtons[1]);
    expect(props.setRunHistoryFilter).toHaveBeenCalledWith("graph_alpha");

    fireEvent.click(controlButtons[2]);
    expect(props.setRunHistoryFilter).toHaveBeenCalledWith("");
    expect(props.setRunHistoryCompileFilter).toHaveBeenCalledWith("");
    expect(props.setRunHistoryFromTime).toHaveBeenCalledWith("");
    expect(props.setRunHistoryToTime).toHaveBeenCalledWith("");
    expect(props.setRunHistoryStatusFilter).toHaveBeenCalledWith("all");
    expect(props.setRunHistorySortOrder).toHaveBeenCalledWith("desc");
    expect(props.setRunHistoryPage).toHaveBeenCalledWith(1);

    fireEvent.click(container.querySelector(".run-history-item"));
    expect(props.loadRunDetail).toHaveBeenCalledWith("run_001");

    const paginationButtons = container.querySelectorAll(".history-pagination button");
    fireEvent.click(paginationButtons[0]);
    fireEvent.click(paginationButtons[1]);
    expect(props.setRunHistoryPage).toHaveBeenCalledWith(1);
    expect(props.setRunHistoryPage).toHaveBeenCalledWith(3);
  });

  it("does not render inside detail mode", () => {
    const { container } = render(<RunHistorySection {...buildProps({ detailMode: true })} />);

    expect(container).toBeEmptyDOMElement();
  });
});
