import { describe, expect, it } from "vitest";
import { appendGraphCompileDiagnostics, buildTopology } from "./compileGraphTopology";

function makeGraph(edges, validationErrors = 0) {
  return {
    nodes: [{ id: "a" }, { id: "b" }, { id: "c" }],
    edges,
    validation_state: {
      issue_counts: { error: validationErrors }
    }
  };
}

describe("compileGraphTopology", () => {
  it("builds a deterministic topology order for acyclic graphs", () => {
    const graph = makeGraph([
      { source_node_id: "a", target_node_id: "b" },
      { source_node_id: "b", target_node_id: "c" }
    ]);

    expect(buildTopology(graph)).toEqual({
      topologyOrder: ["a", "b", "c"],
      hasCycle: false
    });
  });

  it("detects cycles and appends graph-level compile diagnostics", () => {
    const graph = makeGraph(
      [
        { source_node_id: "a", target_node_id: "b" },
        { source_node_id: "b", target_node_id: "a" }
      ],
      1
    );
    const errors = [];

    appendGraphCompileDiagnostics({ graph, topology: buildTopology(graph), errors });

    expect(errors).toEqual([
      "策略图存在循环依赖，无法编译。",
      "策略图校验未通过，无法编译。"
    ]);
  });
});
