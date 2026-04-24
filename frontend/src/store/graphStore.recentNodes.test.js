import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { act } from "@testing-library/react";
import { useGraphStore } from "./graphStore";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";

describe("graphStore recent node tracking", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
      useGraphStore.setState({
        graph: buildValidatedSampleGraph(initialState.registry)
      });
    });
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("records nodes touched by config and edge edits", () => {
    const [firstNode, secondNode] = useGraphStore.getState().graph.nodes;

    act(() => {
      useGraphStore.getState().updateNodeName(firstNode.id, "Renamed node");
    });

    expect(useGraphStore.getState().graph.metadata.editor.recent_node_ids[0]).toBe(firstNode.id);

    act(() => {
      useGraphStore.getState().addEdge({
        source: firstNode.id,
        sourceHandle: firstNode.output_ports?.[0]?.key || "out",
        target: secondNode.id,
        targetHandle: secondNode.input_ports?.[0]?.key || "in"
      });
    });

    expect(useGraphStore.getState().graph.metadata.editor.recent_node_ids.slice(0, 2)).toEqual([
      firstNode.id,
      secondNode.id
    ]);
  });

  it("filters removed nodes out of the recent node list", () => {
    const [firstNode] = useGraphStore.getState().graph.nodes;

    act(() => {
      useGraphStore.getState().updateNodeConfig(firstNode.id, "exchange", "okx");
      useGraphStore.getState().setSelectedNode(firstNode.id);
      useGraphStore.getState().removeSelected();
    });

    expect(
      useGraphStore.getState().graph.metadata.editor.recent_node_ids.includes(firstNode.id)
    ).toBe(false);
  });
});
