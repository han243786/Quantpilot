import { describe, expect, it } from "vitest";
import {
  buildGraphEdgeIndex,
  resolveNodeEdges,
  summarizeGraphNodeTypes
} from "./validationRules";

describe("validationRules", () => {
  it("indexes graph edges by source and target for node-local validation", () => {
    const edges = [
      { id: "edge_a_b", source_node_id: "a", target_node_id: "b" },
      { id: "edge_c_b", source_node_id: "c", target_node_id: "b" },
      { id: "edge_b_d", source_node_id: "b", target_node_id: "d" }
    ];

    const index = buildGraphEdgeIndex(edges);

    expect(resolveNodeEdges(index, "b")).toEqual({
      incoming: [edges[0], edges[1]],
      outgoing: [edges[2]]
    });
    expect(resolveNodeEdges(index, "missing")).toEqual({
      incoming: [],
      outgoing: []
    });
  });

  it("summarizes graph node topology gates without scanning at call sites", () => {
    expect(
      summarizeGraphNodeTypes([
        { type: "runtime" },
        { type: "data" },
        { type: "intent" },
        { type: "agent" },
        { type: "risk" },
        { type: "execution" },
        { type: "execution" }
      ])
    ).toEqual({
      runtimeCount: 1,
      hasExecution: true,
      hasRisk: true,
      hasAgent: true,
      hasIntent: true,
      hasData: true,
      executionCount: 2
    });
  });
});
