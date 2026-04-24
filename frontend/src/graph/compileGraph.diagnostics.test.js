import { describe, expect, it } from "vitest";
import { compileGraph } from "./compileGraph";

function buildGraph(overrides = {}) {
  return {
    metadata: {
      graph_id: "diag_test_graph",
      name: "Diagnostics Test Graph",
      version: "1.0.0",
      runtime_binding: {
        current_run_id: null,
        last_compile_id: null
      },
      editor: {
        viewport: { x: 0, y: 0, zoom: 0.8 }
      },
      artifacts: {},
      ...overrides.metadata
    },
    nodes: overrides.nodes || [],
    edges: overrides.edges || [],
    validation_state: {
      is_valid: true,
      is_runnable: true,
      issue_counts: { error: 0, warning: 0, info: 0 },
      graph_issues: [],
      node_issues: {},
      edge_issues: {},
      ...overrides.validation_state
    }
  };
}

describe("compileGraph diagnostics", () => {
  it("returns structured diagnostics for blocking compile errors", () => {
    const result = compileGraph(buildGraph());

    expect(result.compile_summary.compilable).toBe(false);
    expect(result.graph.metadata.artifacts.quantscript.formal_source).toContain("fn strategy()");
    expect(result.compile_summary.diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          severity: "error",
          code: "GRAPH_COMPILE_ERROR"
        })
      ])
    );
    expect(result.compile_summary.errors.length).toBeGreaterThan(0);
  });
});
