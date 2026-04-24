import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { act, render, screen } from "@testing-library/react";
import PropertyPanel from "./PropertyPanel";
import { useGraphStore } from "../store/graphStore";

describe("PropertyPanel information architecture", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("groups graph overview mode into configuration, compile, and source sections", () => {
    act(() => {
      useGraphStore.setState({
        selectedNodeId: null,
        selectedEdgeId: null
      });
    });

    render(<PropertyPanel />);

    expect(screen.getByTestId("property-section-graph-config")).toBeInTheDocument();
    expect(screen.getByTestId("property-section-diagnostics")).toBeInTheDocument();
    expect(screen.getByTestId("property-section-source")).toBeInTheDocument();
  });

  it("groups node mode into setup, compile, runtime, and source sections", () => {
    act(() => {
      useGraphStore.setState({
        selectedNodeId: "node_runtime",
        selectedEdgeId: null,
        graph: {
          ...useGraphStore.getState().graph,
          nodes: [
            {
              id: "node_runtime",
              module_key: "builtin.runtime.control",
              name: "Runtime Controller",
              config: {},
              runtime_state: {
                status: "running",
                last_event_type: "runtime_event",
                last_message: "Heartbeat",
                last_event_time: "2026-04-14T10:00:00Z",
                metrics: {
                  heartbeat_count: 4
                }
              }
            }
          ],
          edges: [],
          validation_state: {
            ...useGraphStore.getState().graph.validation_state,
            node_issues: {
              node_runtime: []
            }
          }
        }
      });
    });

    render(<PropertyPanel />);

    expect(screen.getByTestId("property-section-node-params")).toBeInTheDocument();
    expect(screen.getByTestId("property-section-diagnostics")).toBeInTheDocument();
    expect(screen.getByTestId("property-section-node-runtime")).toBeInTheDocument();
    expect(screen.getByTestId("property-section-source")).toBeInTheDocument();
  });
});
