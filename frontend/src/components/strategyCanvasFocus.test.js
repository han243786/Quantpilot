import { describe, expect, it } from "vitest";
import {
  buildCanvasFocusBounds,
  collectIssueNodeIds,
  collectRecentNodeIds,
  cycleCanvasFocusTarget,
  resolveCanvasRecommendations,
  resolveCanvasActiveTargetId,
  resolveCanvasFocusAnchorId,
  resolveCanvasFocusTargetIds
} from "./strategyCanvasFocus";

function createGraph(overrides = {}) {
  return {
    metadata: {
      editor: {
        recent_node_ids: ["node_b", "node_a", "missing"]
      },
      ...(overrides.metadata || {})
    },
    nodes: [
      { id: "node_a", type: "data", position: { x: 120, y: 160 } },
      { id: "node_b", type: "intent", position: { x: 440, y: 300 } },
      { id: "node_c", type: "execution", position: { x: 920, y: 520 } },
      { id: "node_d", type: "risk", position: { x: 1160, y: 620 } }
    ],
    edges: [
      { id: "edge_bc", source_node_id: "node_b", target_node_id: "node_c" },
      { id: "edge_cd", source_node_id: "node_c", target_node_id: "node_d" }
    ],
    validation_state: {
      node_issues: {
        node_c: [{ message: "broken" }],
        node_d: [{ message: "guard missing" }],
        ...(overrides.validation_state?.node_issues || {})
      },
      ...(overrides.validation_state || {})
    },
    ...overrides
  };
}

describe("strategyCanvasFocus", () => {
  it("collects issue and recent node ids in graph order", () => {
    const graph = createGraph();

    expect(collectIssueNodeIds(graph)).toEqual(["node_c", "node_d"]);
    expect(collectRecentNodeIds(graph)).toEqual(["node_b", "node_a"]);
  });

  it("resolves focus target ids for selected, issue and recent modes", () => {
    const graph = createGraph();

    expect(resolveCanvasFocusTargetIds(graph, "node_b", "selected")).toEqual(["node_b"]);
    expect(resolveCanvasFocusTargetIds(graph, "node_b", "issues")).toEqual([
      "node_c",
      "node_d"
    ]);
    expect(resolveCanvasFocusTargetIds(graph, "node_b", "recent")).toEqual([
      "node_b",
      "node_a"
    ]);
  });

  it("separates active target, anchor target and cycling logic", () => {
    const targetIds = ["node_b", "node_a", "node_c"];

    expect(resolveCanvasActiveTargetId(targetIds, "node_a")).toBe("node_a");
    expect(resolveCanvasActiveTargetId(targetIds, "missing")).toBe("node_b");
    expect(resolveCanvasFocusAnchorId(targetIds, "node_a", "issues")).toBe("node_a");
    expect(resolveCanvasFocusAnchorId(targetIds, "missing", "issues")).toBe(null);
    expect(resolveCanvasFocusAnchorId(["node_a"], "missing", "selected")).toBe("node_a");
    expect(cycleCanvasFocusTarget(targetIds, "node_a", 1)).toBe("node_c");
    expect(cycleCanvasFocusTarget(targetIds, "node_b", -1)).toBe("node_c");
  });

  it("builds a padded focus bounds rectangle for multiple nodes", () => {
    const graph = createGraph();

    expect(buildCanvasFocusBounds(graph.nodes, ["node_a", "node_b"])).toEqual({
      x: 0,
      y: 76,
      width: 810,
      height: 448
    });
  });

  it("builds diagnostics recommendations from lane and selected node context", () => {
    const graph = createGraph();

    expect(resolveCanvasRecommendations(graph, "node_b", "diagnostics")).toEqual({
      recommendedNodeIds: ["node_b", "node_c", "node_d"],
      pathNodeIds: ["node_b", "node_c", "node_d"],
      pathEdgeIds: ["edge_bc", "edge_cd"],
      issueNodeIds: ["node_c", "node_d"]
    });
  });
});
