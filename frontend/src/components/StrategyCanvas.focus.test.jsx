import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import StrategyCanvas from "./StrategyCanvas";
import { useGraphStore } from "../store/graphStore";

const reactFlowApi = {
  fitBounds: vi.fn(),
  fitView: vi.fn(),
  setCenter: vi.fn(),
  setViewport: vi.fn(),
  zoomIn: vi.fn(),
  zoomOut: vi.fn()
};

vi.mock("@xyflow/react", () => ({
  Background: () => null,
  Panel: ({ children }) => <div>{children}</div>,
  ReactFlow: ({ children }) => <div data-testid="react-flow">{children}</div>,
  ReactFlowProvider: ({ children }) => <>{children}</>,
  useReactFlow: () => reactFlowApi,
  useStore: (selector) => selector({ width: 1280, height: 900 }),
  useViewport: () => ({ x: 0, y: 0, zoom: 1 })
}));

describe("StrategyCanvas focus modes", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    reactFlowApi.fitBounds.mockReset();
    reactFlowApi.fitView.mockReset();
    reactFlowApi.setCenter.mockReset();
    reactFlowApi.setViewport.mockReset();
    reactFlowApi.zoomIn.mockReset();
    reactFlowApi.zoomOut.mockReset();

    act(() => {
      useGraphStore.setState(
        {
          ...initialState,
          selectedNodeId: "node_1",
          graph: {
            ...initialState.graph,
            metadata: {
              ...initialState.graph.metadata,
              editor: {
                ...(initialState.graph.metadata?.editor || {}),
                viewport: { x: 0, y: 0, zoom: 0.8 },
                recent_node_ids: ["node_2", "node_1", "node_3"]
              }
            },
            nodes: [
              {
                id: "node_1",
                type: "data",
                module_key: "builtin.data.kline",
                name: "Kline",
                position: { x: 100, y: 120 },
                input_ports: [],
                output_ports: [],
                config: {},
                ui_state: { collapsed: false },
                runtime_state: { status: "idle", metrics: {} }
              },
              {
                id: "node_2",
                type: "intent",
                module_key: "builtin.intent.double_ma",
                name: "Signal",
                position: { x: 460, y: 260 },
                input_ports: [],
                output_ports: [],
                config: {},
                ui_state: { collapsed: false },
                runtime_state: { status: "idle", metrics: {} }
              },
              {
                id: "node_3",
                type: "execution",
                module_key: "builtin.execution.paper",
                name: "Paper",
                position: { x: 820, y: 320 },
                input_ports: [],
                output_ports: [],
                config: {},
                ui_state: { collapsed: false },
                runtime_state: { status: "idle", metrics: {} }
              }
            ],
            edges: [],
            validation_state: {
              is_valid: false,
              is_runnable: false,
              node_issues: {
                node_2: [{ message: "missing execution" }],
                node_3: [{ message: "missing risk link" }]
              },
              edge_issues: {},
              graph_issues: [],
              issue_counts: { error: 2, warning: 0, info: 0 },
              last_validated_at: null
            },
            compile_summary: { diagnostics: [], errors: [], warnings: [] }
          }
        },
        true
      );
    });
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("switches focus modes, surfaces workbench context, and navigates between issue and recent targets", () => {
    render(<StrategyCanvas />);

    expect(screen.queryByTestId("canvas-workbench-strip")).not.toBeInTheDocument();
    expect(screen.getByTestId("canvas-focus-tab-selected")).toHaveAttribute("aria-selected", "true");

    fireEvent.click(screen.getByTestId("canvas-focus-tab-issues"));
    const issueNav = screen.getByTestId("canvas-focus-nav");
    const issueTargets = screen.getByTestId("canvas-focus-targets");
    expect(issueNav).toBeInTheDocument();
    expect(within(issueTargets).getByTestId("canvas-focus-target-node_2")).toHaveTextContent("Signal");
    expect(within(issueTargets).getByTestId("canvas-focus-target-node_3")).toHaveTextContent("Paper");
    expect(issueNav).toHaveTextContent("1 / 2");

    fireEvent.click(within(issueNav).getAllByRole("button")[1]);
    expect(useGraphStore.getState().selectedNodeId).toBe("node_3");
    expect(issueNav).toHaveTextContent("2 / 2");

    fireEvent.click(screen.getByTestId("canvas-focus-tab-recent"));
    const recentNav = screen.getByTestId("canvas-focus-nav");
    expect(recentNav).toBeInTheDocument();
    expect(recentNav).toHaveTextContent("3");

    fireEvent.click(screen.getByTestId("canvas-focus-target-node_1"));
    expect(useGraphStore.getState().selectedNodeId).toBe("node_1");
  });

  it("surfaces lane-aware recommendations and repair path for validate lane selections", () => {
    act(() => {
      useGraphStore.setState({
        ...useGraphStore.getState(),
        selectedNodeId: "node_2",
        graph: {
          ...useGraphStore.getState().graph,
          nodes: [
            ...useGraphStore.getState().graph.nodes,
            {
              id: "node_4",
              type: "risk",
              module_key: "builtin.risk.guard",
              name: "Risk guard",
              position: { x: 1120, y: 360 },
              input_ports: [],
              output_ports: [],
              config: {},
              ui_state: { collapsed: false },
              runtime_state: { status: "idle", metrics: {} }
            }
          ],
          edges: [
            {
              id: "edge_1",
              source_node_id: "node_2",
              target_node_id: "node_3",
              source_port: "signal",
              target_port: "intent"
            },
            {
              id: "edge_2",
              source_node_id: "node_3",
              target_node_id: "node_4",
              source_port: "orders",
              target_port: "orders"
            }
          ],
          validation_state: {
            ...useGraphStore.getState().graph.validation_state,
            node_issues: {
              node_3: [{ message: "missing execution guard" }],
              node_4: [{ message: "missing risk approval" }]
            },
            issue_counts: { error: 2, warning: 0, info: 0 }
          }
        }
      });
    });

    render(
      <StrategyCanvas
        workspaceContext={{
          laneId: "diagnostics",
          laneLabel: "Validate lane"
        }}
      />
    );

    expect(screen.queryByTestId("canvas-workbench-strip")).not.toBeInTheDocument();
    const recommendationPanel = screen.getByTestId("canvas-recommendation-panel");
    const recommendationTargets = screen.getByTestId("canvas-recommendation-targets");
    const repairPath = screen.getByTestId("canvas-repair-path");
    const repairPathSteps = screen.getByTestId("canvas-repair-path-steps");

    expect(recommendationPanel).toBeInTheDocument();
    expect(within(recommendationTargets).getByTestId("canvas-recommendation-target-node_2")).toHaveTextContent("Signal");
    expect(within(recommendationTargets).getByTestId("canvas-recommendation-target-node_3")).toHaveTextContent("Paper");
    expect(within(recommendationTargets).getByTestId("canvas-recommendation-target-node_4")).toHaveTextContent("Risk guard");
    expect(repairPath).toBeInTheDocument();
    expect(within(repairPathSteps).getByTestId("canvas-repair-path-node-node_2")).toHaveTextContent("Signal");
    expect(within(repairPathSteps).getByTestId("canvas-repair-path-node-node_3")).toHaveTextContent("Paper");
    expect(within(repairPathSteps).getByTestId("canvas-repair-path-node-node_4")).toHaveTextContent("Risk guard");

    fireEvent.click(screen.getByTestId("canvas-recommendation-target-node_3"));
    expect(useGraphStore.getState().selectedNodeId).toBe("node_3");
  });

  it("does not replay selected-node focus while viewport movement is being stored", async () => {
    render(<StrategyCanvas />);

    await waitFor(() => {
      expect(reactFlowApi.setCenter).toHaveBeenCalledTimes(1);
    });

    act(() => {
      useGraphStore.getState().updateEditorViewport({ x: -90, y: -60, zoom: 0.92 }, false);
    });

    await act(async () => {
      await new Promise((resolve) => {
        window.setTimeout(resolve, 50);
      });
    });

    expect(reactFlowApi.setCenter).toHaveBeenCalledTimes(1);
  });
});
