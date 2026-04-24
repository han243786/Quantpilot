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
  const workspaceTabbar = document.querySelector(".strategy-workspace-tabbar");
  const workspaceTabs = within(workspaceTabbar).getAllByRole("button");
  fireEvent.click(workspaceTabs[index]);
}

describe("StrategyWorkspacePage shell", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    window.localStorage.clear();
    act(() => {
      useGraphStore.setState(initialState, true);
      useGraphStore.setState({
        graph: buildGraph(),
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

    await waitFor(() => {
      expect(screen.getByTestId("strategy-workspace-overview-tab")).toBeInTheDocument();
      expect(screen.getByTestId("workspace-primary-controls-section")).toBeInTheDocument();
      expect(screen.getByTestId("workspace-readiness-section")).toBeInTheDocument();
      expect(screen.getByTestId("workspace-research-section")).toBeInTheDocument();
      expect(screen.getByTestId("workspace-persisted-versions-section")).toBeInTheDocument();
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

    expect(within(inspectorNav).getByRole("button", { name: /Config/i })).toBeInTheDocument();
    expect(within(inspectorNav).getByRole("button", { name: /Checks/i })).toBeInTheDocument();
    expect(within(inspectorNav).getByRole("button", { name: /Source/i })).toBeInTheDocument();
    expect(screen.getByTestId("strategy-canvas-stub")).toBeInTheDocument();
    expect(screen.getByTestId("strategy-params-panel-stub")).toBeInTheDocument();
  });

  it("switches inspector lanes and expands secondary disclosures", async () => {
    const { container } = render(<StrategyWorkspacePage strategyId="workspace_test_graph" />);

    await openWorkspaceMode(1);
    await waitFor(() => {
      expect(container.querySelector(".workspace-inspector-nav")).toBeTruthy();
    });
    const inspectorNav = container.querySelector(".workspace-inspector-nav");

    fireEvent.click(within(inspectorNav).getByRole("button", { name: /Source/i }));
    await waitFor(() => {
      expect(activeInspectorLabel(container)).toBe("Source");
    });

    const disclosureButton = screen.getByRole("button", { name: /Show Config lane/i });
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

    expect(activeInspectorLabel(container)).toBe("Config");
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
    await openWorkspaceMode(2);
    await waitFor(() => {
      expect(screen.getByTestId("strategy-workspace-diagnostics-tab")).toBeInTheDocument();
      expect(screen.getByTestId("workspace-priority-repair-queue-section")).toBeInTheDocument();
      expect(screen.getByTestId("workspace-structured-diagnostics-section")).toBeInTheDocument();
      expect(screen.getByTestId("diagnostics-panel-stub")).toBeInTheDocument();
    });
  });

  it("renders the research shell with the event stream console", async () => {
    render(<StrategyWorkspacePage strategyId="workspace_test_graph" />);

    await openWorkspaceMode(3);
    await waitFor(() => {
      expect(screen.getByTestId("strategy-workspace-research-tab")).toBeInTheDocument();
      expect(screen.getByTestId("workspace-run-backtest-controls-section")).toBeInTheDocument();
      expect(screen.getByTestId("top-toolbar-stub")).toBeInTheDocument();
      expect(screen.getByTestId("strategy-research-console-stub")).toBeInTheDocument();
    });
  });
});
