import { fireEvent, render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import StrategyBacktestsPage from "./StrategyBacktestsPage";

const { navigateTo } = vi.hoisted(() => ({
  navigateTo: vi.fn()
}));

const loadGraphById = vi.fn();
const handleRefreshBacktestHistory = vi.fn(async () => {});
const clearBacktestCompareSelection = vi.fn();
const loadBacktestDetail = vi.fn();

vi.mock("../router", () => ({
  navigateTo,
  strategiesPath: () => "/strategies",
  strategyWorkspacePath: (strategyId) => `/strategies/${strategyId}`,
  strategyBacktestsPath: (strategyId) => `/strategies/${strategyId}/backtests`,
  backtestDetailPath: (backtestId, strategyId = "") =>
    strategyId ? `/backtests/${backtestId}?strategy=${strategyId}` : `/backtests/${backtestId}`,
  backtestComparePath: (ids, strategyId = "") =>
    strategyId
      ? `/backtests/compare?ids=${ids.join(",")}&strategy=${strategyId}`
      : `/backtests/compare?ids=${ids.join(",")}`
}));

vi.mock("../components/StrategyBacktestsPanel", () => ({
  default: ({ onOpenBacktestDetail, compareSelection, filteredBacktests }) => (
    <div data-testid="strategy-backtests-panel-stub">
      <span>{`filtered:${filteredBacktests.length}`}</span>
      <span>{`compare:${compareSelection.length}`}</span>
      <button type="button" onClick={() => onOpenBacktestDetail("bt_alpha_01")}>
        Open backtest detail
      </button>
    </div>
  )
}));

vi.mock("../hooks/useStrategyWorkspaceSharedModel", () => ({
  useStrategyWorkspaceSharedModel: () => ({
    graph: {
      metadata: {
        name: "Alpha strategy",
        graph_id: "other_strategy",
        runtime_binding: {
          last_compile_id: "compile_alpha_001"
        }
      },
      compile_summary: {
        protocol_name: "quantpilot/runtime-config/v1",
        config_hash: "cfg_alpha_001"
      }
    },
    runtime: {
      backtestHistoryStatus: "ready"
    },
    loadGraphById
  })
}));

vi.mock("../hooks/useStrategyResearchUiState", () => ({
  useStrategyResearchUiState: () => ({
    backtestFilters: {
      backtestHistoryFilter: "alpha_strategy",
      backtestPageSize: 6
    }
  })
}));

vi.mock("../hooks/strategyResearchSelectors", () => ({
  useStrategyResearchSelectors: () => ({
    graph: {
      metadata: {
        name: "Alpha strategy",
        graph_id: "other_strategy",
        runtime_binding: {
          last_compile_id: "compile_alpha_001"
        }
      },
      compile_summary: {
        protocol_name: "quantpilot/runtime-config/v1",
        config_hash: "cfg_alpha_001"
      }
    },
    runtime: {
      backtestHistoryStatus: "ready"
    },
    filteredBacktests: [
      {
        backtest_id: "bt_alpha_01",
        created_at_ms: 1710000000000,
        summary: { total_return_ratio: 0.12 },
        filters: { dataset_labels: ["BTC-1h"] }
      }
    ],
    pagedBacktests: [
      {
        backtest_id: "bt_alpha_01",
        created_at_ms: 1710000000000,
        summary: { total_return_ratio: 0.12 },
        filters: { dataset_labels: ["BTC-1h"] }
      }
    ],
    backtestCurrentPage: 1,
    backtestTotalPages: 1,
    compareSelection: ["bt_alpha_01", "bt_alpha_02"],
    selectedBacktestSummary: null,
    backtestSummary: { total_return_ratio: 0.12 },
    backtestStartedAt: 1710000000000,
    backtestEndedAt: 1710003600000
  })
}));

vi.mock("../hooks/useStrategyResearchActions", () => ({
  useStrategyResearchActions: () => ({
    handleRefreshBacktestHistory,
    clearBacktestCompareSelection,
    loadBacktestDetail,
    toggleBacktestCompareSelection: vi.fn(),
    setBacktestHistoryFilter: vi.fn(),
    setBacktestCompileFilter: vi.fn(),
    setBacktestDatasetFilter: vi.fn(),
    setBacktestParameterFilter: vi.fn(),
    setBacktestFromTime: vi.fn(),
    setBacktestToTime: vi.fn(),
    setBacktestPage: vi.fn(),
    setBacktestPageSize: vi.fn()
  })
}));

describe("StrategyBacktestsPage", () => {
  beforeEach(() => {
    navigateTo.mockReset();
    loadGraphById.mockReset();
    handleRefreshBacktestHistory.mockClear();
    clearBacktestCompareSelection.mockClear();
  });

  it("loads strategy context and routes workspace, compare, and detail actions", () => {
    render(<StrategyBacktestsPage strategyId="alpha_strategy" />);

    const hero = screen.getByTestId("strategy-backtests-hero");
    const routeBar = screen.getByRole("navigation", { name: "Strategy navigation" });

    expect(hero).toHaveTextContent("alpha_strategy");
    expect(routeBar).toBeInTheDocument();
    expect(within(routeBar).getAllByRole("button")).toHaveLength(1);
    expect(screen.getByTestId("strategy-backtests-panel-stub")).toBeInTheDocument();
    expect(loadGraphById).toHaveBeenCalledWith("alpha_strategy");

    fireEvent.click(screen.getByTestId("strategy-backtests-workspace-button"));
    expect(navigateTo).toHaveBeenCalledWith("/strategies/alpha_strategy");

    fireEvent.click(screen.getByTestId("strategy-backtests-compare-button"));
    expect(navigateTo).toHaveBeenCalledWith(
      "/backtests/compare?ids=bt_alpha_01,bt_alpha_02&strategy=alpha_strategy"
    );

    fireEvent.click(screen.getByRole("button", { name: "Open backtest detail" }));
    expect(navigateTo).toHaveBeenCalledWith("/backtests/bt_alpha_01?strategy=alpha_strategy");
  });
});
