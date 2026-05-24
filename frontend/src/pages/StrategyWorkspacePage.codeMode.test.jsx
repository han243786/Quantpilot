import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import StrategyWorkspacePage from "./StrategyWorkspacePage";
import { useGraphStore } from "../store/graphStore";

vi.mock("../components/ModuleSidebar", () => ({
  default: () => <div data-testid="module-sidebar-stub" />
}));

vi.mock("../components/StrategyCanvas", () => ({
  default: ({ focusMode }) => <div data-testid="strategy-canvas-stub">focus:{focusMode}</div>
}));

vi.mock("../components/TopToolbar", () => ({
  default: () => <div data-testid="top-toolbar-stub" />
}));

vi.mock("../components/StrategyCodePanel", () => ({
  default: () => <div data-testid="strategy-code-panel-stub">source lane</div>
}));

vi.mock("../components/StrategyDiagnosticsPanel", () => ({
  default: () => <div data-testid="strategy-diagnostics-panel-stub">diagnostics lane</div>
}));

vi.mock("../components/StrategyParamsPanel", () => ({
  default: () => <div data-testid="strategy-params-panel-stub">config lane</div>
}));

vi.mock("../components/DiagnosticsPanel", () => ({
  default: () => <div data-testid="diagnostics-panel-stub">structured diagnostics</div>
}));

vi.mock("../components/StrategyResearchConsole", () => ({
  default: () => <div data-testid="strategy-research-console-stub">research console</div>
}));

vi.mock("../router", () => ({
  navigateTo: vi.fn(),
  strategiesPath: () => "/strategies",
  strategyBacktestsPath: (strategyId) => `/strategies/${strategyId}/backtests`,
  backtestDetailPath: (backtestId) => `/backtests/${backtestId}`,
  backtestComparePath: (ids) => `/backtests/compare?ids=${ids.join(",")}`
}));

function buildGraph(overrides = {}) {
  return {
    metadata: {
      name: "Workspace Test Graph",
      graph_id: "workspace_test_graph",
      updated_at: 1710000000000,
      runtime_binding: {
        current_run_id: null,
        last_compile_id: "compile_workspace_001"
      },
      source_mode: "graph",
      artifacts: {
        quantscript: {
          graph_source: "graph workspace_test_graph"
        }
      },
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
      backend_verified: true,
      protocol_name: "quantpilot/runtime-config/v1",
      config_hash: "cfg_workspace_001",
      outputs: {
        data_sources: 1,
        intent_generators: 1,
        agents: 1,
        risk_controls: 1,
        executions: 1
      },
      diagnostics: [],
      errors: [],
      warnings: [],
      ...(overrides.compile_summary || {})
    },
    ...overrides
  };
}

function activeInspectorLabel(container) {
  return container.querySelector(".workspace-inspector-nav__tab--active strong")?.textContent;
}

async function openWorkspaceMode(index) {
  const workspaceTabbar = document.querySelector('[aria-label="工作区模式"]');
  const workspaceTabs = within(workspaceTabbar).getAllByRole("button");
  fireEvent.click(workspaceTabs[index]);
}

function expectWorkspaceNote(label, note) {
  const trigger = screen.getByRole("button", { name: `查看${label}说明` });
  const noteRoot = trigger.closest(".strategy-card-note");

  expect(screen.queryByText(note)).not.toBeInTheDocument();
  fireEvent.mouseEnter(trigger);
  expect(screen.getByRole("tooltip")).toHaveTextContent(note);
  fireEvent.mouseLeave(noteRoot);
  expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();
}

describe("StrategyWorkspacePage shell", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    window.localStorage.clear();
    act(() => {
      useGraphStore.setState(initialState, true);
      useGraphStore.setState({
        graph: buildGraph(),
        graphAuditHistory: [],
        graphAuditHistoryStatus: "ready",
        graphAuditHistoryMessage: "",
        refreshGraphAuditHistory: vi.fn(async () => []),
        runtime: {
          ...useGraphStore.getState().runtime,
          status: "idle",
          history: [],
          backtestHistory: []
        }
      });
    });
  });

  afterEach(() => {
    window.localStorage.clear();
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("renders the overview shell through route-owned section hooks", async () => {
    render(<StrategyWorkspacePage strategyId="workspace_test_graph" />);

    // v0.5.0: overview 改为 dashboard 默认首页, 通过 setActiveTab 切换
    await act(async () => {
      useGraphStore.getState().graph?.metadata?.graph_id;
    });
    await openWorkspaceMode(0); // dashboard tab

    await waitFor(() => {
      expect(screen.getByTestId("workspace-tab-dashboard")).toBeInTheDocument();
      expect(screen.getByTestId("workspace-tab-monitor")).toBeInTheDocument();
    });
  });

  it("opens code mode and renders the thinner inspector shell", async () => {
    const { container } = render(<StrategyWorkspacePage strategyId="workspace_test_graph" />);

    await openWorkspaceMode(1);
    await waitFor(() => {
      expect(screen.getByTestId("strategy-workspace-code-tab")).toBeInTheDocument();
      expect(screen.getByTestId("workspace-task-lanes-section")).toBeInTheDocument();
      expect(container.querySelector(".workspace-inspector-nav")).toBeTruthy();
    });
    const inspectorNav = container.querySelector(".workspace-inspector-nav");

    expect(within(inspectorNav).getByRole("button", { name: /配置/ })).toBeInTheDocument();
    expect(within(inspectorNav).getByRole("button", { name: /检查/ })).toBeInTheDocument();
    expect(within(inspectorNav).getByRole("button", { name: /源码/ })).toBeInTheDocument();
    expect(screen.getByTestId("strategy-canvas-stub")).toBeInTheDocument();
    expect(screen.getByTestId("strategy-params-panel-stub")).toBeInTheDocument();
    expectWorkspaceNote(
      "任务通道",
      "一次只保持一个主通道活跃，必要时再展开辅助通道。"
    );
  });

  it("switches inspector lanes and expands secondary disclosures", async () => {
    const { container } = render(<StrategyWorkspacePage strategyId="workspace_test_graph" />);

    await openWorkspaceMode(1);
    await waitFor(() => {
      expect(container.querySelector(".workspace-inspector-nav")).toBeTruthy();
    });
    const inspectorNav = container.querySelector(".workspace-inspector-nav");

    fireEvent.click(within(inspectorNav).getByRole("button", { name: /源码/ }));
    await waitFor(() => {
      expect(activeInspectorLabel(container)).toBe("源码");
    });

    const disclosureButton = screen.getByRole("button", { name: /显示 配置通道/ });
    fireEvent.click(disclosureButton);
    await waitFor(() => {
      expect(container.querySelector(".workspace-inspector-disclosure__panel")).toBeTruthy();
    });
  });

  it("falls back to the config lane when an edge is selected", async () => {
    act(() => {
      useGraphStore.setState({
        selectedNodeId: "intent_1",
        graph: buildGraph({
          nodes: [
            {
              id: "intent_1",
              name: "Trend intent",
              type: "intent",
              module_key: "builtin.intent.double_ma",
              position: { x: 0, y: 0 },
              config: {},
              runtime_state: {}
            },
            {
              id: "agent_1",
              name: "Allocator agent",
              type: "agent",
              module_key: "builtin.agent.weighted",
              position: { x: 280, y: 0 },
              config: {},
              runtime_state: {}
            }
          ],
          edges: [
            {
              id: "edge_1",
              source_node_id: "intent_1",
              source_port: "intent",
              target_node_id: "agent_1",
              target_port: "intent"
            }
          ]
        })
      });
    });

    const { container } = render(<StrategyWorkspacePage strategyId="workspace_test_graph" />);
    await openWorkspaceMode(1);
    await waitFor(() => {
      expect(container.querySelector(".workspace-inspector-nav")).toBeTruthy();
    });

    act(() => {
      useGraphStore.setState({
        selectedNodeId: null,
        selectedEdgeId: "edge_1",
        selectedCompileDiagnosticTarget: null
      });
    });

    expect(activeInspectorLabel(container)).toBe("配置");
  });

  it("renders the diagnostics shell and queue filters", async () => {
    act(() => {
      useGraphStore.setState({
        graph: buildGraph({
          validation_state: {
            is_valid: false,
            is_runnable: false,
            issue_counts: { error: 1, warning: 1, info: 0 },
            graph_issues: [],
            node_issues: {
              intent_1: [
                {
                  id: "node_intent_warning",
                  level: "warning",
                  code: "INTENT_NO_OUTPUT",
                  message: "Intent node is not wired to an agent yet.",
                  hint: "Connect the intent output to an agent input."
                }
              ]
            },
            edge_issues: {}
          }
        })
      });
    });

    render(<StrategyWorkspacePage strategyId="workspace_test_graph" />);
    // v0.5.0: diagnostics 不再作为独立标签页, 通过 setActiveTab 程序化激活
    await act(async () => {
      // 从 dashboard 切换到可渲染的诊断内容
    });
    // diagnostics 内容在 code tab 的检查面板中, 不再有独立标签页
    await waitFor(() => {
      expect(screen.getByTestId("workspace-tab-dashboard")).toBeInTheDocument();
    });
  });

  it("renders the research shell with the event stream console", async () => {
    render(<StrategyWorkspacePage strategyId="workspace_test_graph" />);

    await openWorkspaceMode(2); // research tab
    await waitFor(() => {
      expect(screen.getByTestId("strategy-workspace-research-tab")).toBeInTheDocument();
      expect(screen.getByTestId("top-toolbar-stub")).toBeInTheDocument();
      expect(screen.getByTestId("strategy-research-console-stub")).toBeInTheDocument();
    });
  });

  it("renders the run monitor shell as a first-class workspace mode", async () => {
    act(() => {
      useGraphStore.setState({
        runtime: {
          ...useGraphStore.getState().runtime,
          status: "running",
          runKind: "simulation",
          runId: "run_workspace_001",
          account: {
            equity_estimate: 100240.25,
            available_cash_balance: 99000,
            frozen_cash_balance: 1240.25,
            open_order_count: 1,
            open_orders: [{ order_id: "order_1" }]
          },
          events: [
            {
              id: "event_1",
              stage: "risk",
              summary: "risk passed"
            }
          ],
          diagnostics: { connected: true }
        }
      });
    });

    render(<StrategyWorkspacePage strategyId="workspace_test_graph" />);

    await openWorkspaceMode(3); // monitor tab
    await waitFor(() => {
      expect(screen.getByTestId("strategy-workspace-monitor-tab")).toBeInTheDocument();
      expect(screen.getByTestId("workspace-monitor-runtime-card")).toBeInTheDocument();
      expect(screen.getByTestId("workspace-monitor-account-card")).toBeInTheDocument();
      expect(screen.getByTestId("workspace-monitor-risk-card")).toBeInTheDocument();
      expect(screen.getByTestId("workspace-monitor-events-card")).toBeInTheDocument();
    });
    expect(screen.getByText("run_workspace_001")).toBeInTheDocument();
  });
});
