import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import StrategyHubPage from "./StrategyHubPage";
import { useGraphStore } from "../store/graphStore";

const originalFetch = global.fetch;
const { navigateTo } = vi.hoisted(() => ({
  navigateTo: vi.fn()
}));

vi.mock("../router", () => ({
  navigateTo,
  strategyWorkspacePath: (strategyId) => `/strategies/${strategyId}`,
  strategyBacktestsPath: (strategyId) => `/strategies/${strategyId}/backtests`,
  backtestDetailPath: (backtestId) => `/backtests/${backtestId}`,
  backtestComparePath: (ids) => `/backtests/compare?ids=${ids.join(",")}`
}));

function buildGraph(overrides = {}) {
  return {
    metadata: {
      name: "Alpha strategy",
      graph_id: "alpha_strategy",
      updated_at: 1710000000000,
      runtime_binding: {
        current_run_id: null,
        last_compile_id: "compile_alpha_001"
      },
      source_mode: "graph",
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
      edge_issues: {},
      ...(overrides.validation_state || {})
    },
    compile_summary: {
      compilable: true,
      protocol_name: "quantpilot/runtime-config/v1",
      config_hash: "cfg_alpha_001",
      diagnostics: [],
      ...(overrides.compile_summary || {})
    },
    ...overrides
  };
}

async function waitForHubRosterTable() {
  await screen.findByTestId("strategy-hub-page");
  await waitFor(() => {
    expect(screen.getByTestId("strategy-hub-roster-table")).toBeInTheDocument();
  });
  return screen.getByTestId("strategy-hub-roster-table");
}

const STRATEGY_HUB_CARD_NOTES = [
  {
    label: "策略清单",
    note: "让列表保持在总览管理层面：先筛查策略，再只在必要时深入到单个工作区。"
  },
  {
    label: "策略驾驶舱",
    note: "在保持清单与活动面板可见的同时，持续聚焦当前选中的这一条策略。"
  },
  {
    label: "近期研究活动",
    note: "让回测直接留在首页可见，避免研究线索被埋进单个策略页面。"
  },
  {
    label: "近期运行活动",
    note: "不离开策略中心即可跟踪模拟记录与编译 ID。"
  }
];

function expectCardNoteTooltip(label, note) {
  const trigger = screen.getByRole("button", { name: `查看${label}说明` });
  const noteRoot = trigger.closest(".strategy-card-note");

  fireEvent.mouseEnter(trigger);
  expect(screen.getByRole("tooltip")).toHaveTextContent(note);

  fireEvent.mouseLeave(noteRoot);
  expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();
}

describe("StrategyHubPage", () => {
  const initialState = useGraphStore.getState();
  let fetchMock;
  let confirmMock;
  let dateNowMock;

  beforeEach(() => {
    navigateTo.mockReset();
    confirmMock = vi.spyOn(window, "confirm").mockReturnValue(true);
    dateNowMock = vi.spyOn(Date, "now").mockReturnValue(1710000999000);
    fetchMock = vi.fn(async (url, options = {}) => {
      if (url === "/api/graphs/alpha_strategy/reveal") {
        return {
          ok: true,
          json: async () => ({ graph_id: "alpha_strategy", path: "storage/graphs/alpha_strategy.qs" }),
          text: async () => ""
        };
      }
      if (url === "/api/graphs/alpha_strategy" && options.method === "DELETE") {
        return {
          ok: true,
          json: async () => ({ graph_id: "alpha_strategy", deleted: true }),
          text: async () => ""
        };
      }
      if (url === "/api/graphs") {
        return {
          ok: true,
          json: async () => [],
          text: async () => ""
        };
      }
      return {
        ok: true,
        json: async () => ({}),
        text: async () => ""
      };
    });
    global.fetch = fetchMock;

    act(() => {
      useGraphStore.setState(initialState, true);
      useGraphStore.setState({
        graph: buildGraph(),
        graphIndex: [
          {
            graph_id: "alpha_strategy",
            name: "Alpha strategy",
            updated_at: 1710000000000,
            path: "storage/graphs/alpha_strategy.qs"
          }
        ],
        graphIndexStatus: "ready",
        runtime: {
          ...useGraphStore.getState().runtime,
          historyStatus: "ready",
          backtestHistoryStatus: "ready",
          history: [
            {
              run_id: "run_alpha_01",
              graph_id: "alpha_strategy",
              compile_id: "compile_alpha_001",
              created_at_ms: 1710000100000
            },
            {
              run_id: "run_beta_01",
              graph_id: "beta_strategy",
              compile_id: "compile_beta_003",
              created_at_ms: 1710000300000
            }
          ],
          backtestHistory: [
            {
              backtest_id: "bt_alpha_01",
              graph_id: "alpha_strategy",
              compile_id: "compile_alpha_001",
              config_hash: "cfg_alpha_001",
              protocol_name: "quantpilot/runtime-config/v1",
              created_at_ms: 1710000200000,
              summary: { total_return_ratio: 0.12 },
              filters: { dataset_labels: ["BTC-1h"] }
            },
            {
              backtest_id: "bt_beta_01",
              graph_id: "beta_strategy",
              compile_id: "compile_beta_003",
              config_hash: "cfg_beta_003",
              protocol_name: "quantpilot/runtime-config/v1",
              created_at_ms: 1710000400000,
              summary: { total_return_ratio: -0.03 },
              filters: { dataset_labels: ["ETH-4h"] }
            }
          ],
          backtestCompareSelection: ["bt_beta_01"]
        }
      });
    });
  });

  afterEach(() => {
    vi.useRealTimers();
    confirmMock?.mockRestore();
    dateNowMock?.mockRestore();
    if (originalFetch) {
      global.fetch = originalFetch;
    } else {
      delete global.fetch;
    }
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("renders the hub layout, activity panels, and workspace entry actions", async () => {
    const { container } = render(<StrategyHubPage />);

    const rosterTable = await waitForHubRosterTable();

    expect(screen.getByTestId("strategy-hub-page")).toBeInTheDocument();
    expect(screen.getByTestId("strategy-hub-hero")).toBeInTheDocument();
    const statusStrip = container.querySelector(".strategy-hub-status-strip");
    expect(statusStrip).toBeInTheDocument();
    expect(statusStrip.children).toHaveLength(8);
    expect(statusStrip.querySelector(".strategy-hub-ops")).not.toBeInTheDocument();
    expect(await screen.findByTestId("strategy-template-library")).toBeInTheDocument();
    expect(rosterTable).toBeInTheDocument();
    expect(container.querySelector(".strategy-inspector-card")).toBeInTheDocument();
    const hubGrid = container.querySelector(".strategy-hub-grid");
    expect(hubGrid).toBeInTheDocument();
    expect(hubGrid.children[0]).toHaveClass("strategy-hub-main");
    expect(hubGrid.children[0]).toContainElement(screen.getByTestId("strategy-template-library"));
    expect(hubGrid.children[0]).toContainElement(rosterTable);
    expect(hubGrid.children[1]).toHaveClass("strategy-inspector-card");
    expect(await screen.findByTestId("strategy-hub-activity-card-backtest")).toBeInTheDocument();
    expect(await screen.findByTestId("strategy-hub-activity-card-run")).toBeInTheDocument();
    expect(rosterTable).not.toHaveTextContent("beta_strategy");
    expect(
      [...screen.getByTestId("strategy-hub-hero").querySelectorAll(".strategy-task-group__label")].map(
        (item) => item.textContent
      )
    ).toEqual([]);
    STRATEGY_HUB_CARD_NOTES.forEach(({ note }) => {
      expect(screen.queryByText(note)).not.toBeInTheDocument();
    });
    STRATEGY_HUB_CARD_NOTES.forEach(({ label, note }) => {
      expectCardNoteTooltip(label, note);
    });

    fireEvent.click(screen.getByTestId("strategy-hub-roster-row-select-alpha_strategy"));
    fireEvent.click(screen.getByTestId("strategy-hub-open-current-workspace"));
    expect(navigateTo).toHaveBeenCalledWith("/strategies/alpha_strategy");

    fireEvent.click(screen.getByTestId("strategy-hub-open-blank-workspace"));
    expect(navigateTo).toHaveBeenCalledWith("/strategies/graph_1710000999000");

    expect(screen.getByTestId("strategy-hub-roster-row-shell-alpha_strategy")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("strategy-hub-roster-action-alpha_strategy-open-workspace"));
    expect(navigateTo).toHaveBeenCalledWith("/strategies/alpha_strategy");

    fireEvent.click(screen.getByTestId("strategy-hub-roster-action-alpha_strategy-open-backtests"));
    expect(navigateTo).toHaveBeenCalledWith("/strategies/alpha_strategy/backtests");

    fireEvent.click(screen.getByTestId("strategy-hub-roster-action-alpha_strategy-reveal-file"));
    expect(fetchMock).toHaveBeenCalledWith("/api/graphs/alpha_strategy/reveal", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({})
    });

    fireEvent.click(screen.getByTestId("strategy-hub-roster-action-alpha_strategy-delete-strategy"));
    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith("/api/graphs/alpha_strategy", {
        method: "DELETE"
      });
    });
    expect(confirmMock).toHaveBeenCalledWith(
      "确认删除策略“Alpha strategy”？此操作会移除策略文件和版本记录。"
    );
  });

  it("keeps the inline note hover, pin, drag, and close behavior intact", async () => {
    const { container } = render(<StrategyHubPage />);

    await waitForHubRosterTable();
    vi.useFakeTimers();

    const trigger = container.querySelector(".strategy-inline-note__trigger-text");
    fireEvent.mouseEnter(trigger);
    expect(screen.getByRole("tooltip")).toBeInTheDocument();

    const noteRoot = trigger.closest(".strategy-inline-note");
    fireEvent.mouseLeave(noteRoot);
    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();

    fireEvent.mouseEnter(trigger);
    const bridge = screen.getByTestId("strategy-inline-note-bridge");
    fireEvent.mouseEnter(bridge);
    expect(screen.getByRole("tooltip")).toBeInTheDocument();

    const popup = screen.getByRole("tooltip");
    fireEvent.mouseEnter(popup);

    act(() => {
      vi.advanceTimersByTime(3000);
    });

    const closeButton = container.querySelector(".strategy-inline-note__close");
    expect(closeButton).toBeInTheDocument();

    noteRoot.getBoundingClientRect = () => ({
      x: 20,
      y: 30,
      left: 20,
      top: 30,
      right: 220,
      bottom: 230,
      width: 200,
      height: 200
    });
    popup.getBoundingClientRect = () => ({
      x: 50,
      y: 120,
      left: 50,
      top: 120,
      right: 410,
      bottom: 280,
      width: 360,
      height: 160
    });

    fireEvent.mouseDown(popup, { clientX: 120, clientY: 160 });
    fireEvent.mouseMove(window, { clientX: 170, clientY: 210 });

    expect(popup).toHaveStyle({ left: "80px", top: "140px" });

    fireEvent.mouseLeave(noteRoot);
    expect(screen.getByRole("tooltip")).toBeInTheDocument();

    fireEvent.click(closeButton);
    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();
  });

  it("keeps metric card notes open after the popup is pinned", async () => {
    const { container } = render(<StrategyHubPage />);

    await waitForHubRosterTable();
    vi.useFakeTimers();

    const trigger = screen.getByRole("button", { name: "查看策略文件说明" });
    fireEvent.mouseEnter(trigger);

    const popup = screen.getByRole("tooltip");
    expect(popup).toHaveTextContent("当前由后端真实追踪");

    fireEvent.mouseEnter(popup);

    act(() => {
      vi.advanceTimersByTime(3000);
    });

    const closeButton = screen.getByRole("button", { name: "关闭策略文件说明" });
    expect(closeButton).toBeInTheDocument();

    const noteRoot = trigger.closest(".strategy-card-note");
    fireEvent.mouseLeave(noteRoot);
    expect(screen.getByRole("tooltip")).toBeInTheDocument();

    fireEvent.click(closeButton);
    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();

    expect(container.querySelector(".strategy-card-note__pin-progress")).not.toBeInTheDocument();
  });

  it("does not treat the fallback in-memory graph as a tracked hub strategy when no real files exist", async () => {
    act(() => {
      useGraphStore.setState({
        graph: buildGraph({
          metadata: {
            graph_id: "legacy_sample_graph",
            name: "legacy sample graph"
          }
        }),
        graphIndex: [],
        graphIndexStatus: "ready",
        runtime: {
          ...useGraphStore.getState().runtime,
          historyStatus: "ready",
          backtestHistoryStatus: "ready",
          history: [],
          backtestHistory: [],
          backtestCompareSelection: []
        }
      });
    });

    const { container } = render(<StrategyHubPage />);

    const rosterTable = await waitForHubRosterTable();

    expect(container.querySelectorAll(".strategy-row-shell")).toHaveLength(0);
    expect(rosterTable).not.toHaveTextContent("legacy sample graph");
  });
});
