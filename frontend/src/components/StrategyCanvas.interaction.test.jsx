import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen } from "@testing-library/react";
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
  ReactFlow: ({ children, edges, nodes, onConnect, onNodeClick, onNodeDragStop, onPaneClick }) => (
    <div data-testid="react-flow">
      {nodes.map((node) => (
        <button
          key={node.id}
          data-testid={`rf-node-${node.id}`}
          type="button"
          onClick={(event) => onNodeClick?.(event, node)}
          onMouseUp={(event) =>
            onNodeDragStop?.(event, { ...node, position: { x: 640, y: 360 } })
          }
        >
          {node.id}
        </button>
      ))}
      {edges.map((edge) => (
        <span key={edge.id} data-testid={`rf-edge-${edge.id}`}>
          {edge.id}
        </span>
      ))}
      <button data-testid="rf-pane" type="button" onClick={() => onPaneClick?.()}>
        pane
      </button>
      <button
        data-testid="rf-connect"
        type="button"
        onClick={() =>
          onConnect?.({
            source: "node_1",
            sourceHandle: "signal",
            target: "node_2",
            targetHandle: "intent"
          })
        }
      >
        connect
      </button>
      {children}
    </div>
  ),
  ReactFlowProvider: ({ children }) => <>{children}</>,
  useReactFlow: () => reactFlowApi,
  useStore: (selector) => selector({ width: 1280, height: 900 }),
  useViewport: () => ({ x: 0, y: 0, zoom: 1 })
}));

describe("StrategyCanvas interactions", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    Object.values(reactFlowApi).forEach((mock) => mock.mockReset());

    act(() => {
      useGraphStore.setState(
        {
          ...initialState,
          selectedNodeId: null,
          selectedEdgeId: null,
          graph: {
            ...initialState.graph,
            metadata: {
              ...initialState.graph.metadata,
              editor: {
                ...(initialState.graph.metadata?.editor || {}),
                viewport: { x: 0, y: 0, zoom: 0.8 },
                recent_node_ids: []
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
              }
            ],
            edges: [
              {
                id: "edge_existing",
                source_node_id: "node_1",
                source_port: "signal",
                target_node_id: "node_2",
                target_port: "intent"
              }
            ],
            validation_state: {
              is_valid: true,
              is_runnable: true,
              node_issues: {},
              edge_issues: {},
              graph_issues: [],
              issue_counts: { error: 0, warning: 0, info: 0 },
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

  it("selects a clicked node for downstream property editing", () => {
    render(<StrategyCanvas />);

    fireEvent.click(screen.getByTestId("rf-node-node_2"));

    expect(useGraphStore.getState().selectedNodeId).toBe("node_2");
    expect(useGraphStore.getState().selectedEdgeId).toBeNull();
  });

  it("clears canvas selections from a pane click", () => {
    act(() => {
      useGraphStore.setState({ selectedNodeId: "node_1", selectedEdgeId: "edge_existing" });
    });
    render(<StrategyCanvas />);

    fireEvent.click(screen.getByTestId("rf-pane"));

    expect(useGraphStore.getState().selectedNodeId).toBeNull();
    expect(useGraphStore.getState().selectedEdgeId).toBeNull();
  });

  it("persists node position after a drag stop", () => {
    render(<StrategyCanvas />);

    fireEvent.mouseUp(screen.getByTestId("rf-node-node_1"));

    const movedNode = useGraphStore
      .getState()
      .graph.nodes.find((node) => node.id === "node_1");
    expect(movedNode.position).toEqual({ x: 640, y: 360 });
  });

  it("creates an edge from a connect gesture", () => {
    render(<StrategyCanvas />);

    fireEvent.click(screen.getByTestId("rf-connect"));

    expect(
      useGraphStore
        .getState()
        .graph.edges.some(
          (edge) =>
            edge.source_node_id === "node_1" &&
            edge.source_port === "signal" &&
            edge.target_node_id === "node_2" &&
            edge.target_port === "intent"
        )
    ).toBe(true);
  });

  it("removes a canvas-selected node and its attached edges", () => {
    render(<StrategyCanvas />);

    fireEvent.click(screen.getByTestId("rf-node-node_2"));
    act(() => {
      useGraphStore.getState().removeSelected();
    });

    const graph = useGraphStore.getState().graph;
    expect(graph.nodes.some((node) => node.id === "node_2")).toBe(false);
    expect(graph.edges.some((edge) => edge.target_node_id === "node_2")).toBe(false);
    expect(useGraphStore.getState().selectedNodeId).toBeNull();
  });
});
