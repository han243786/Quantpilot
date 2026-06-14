import { describe, expect, it } from "vitest";
import { DEFAULT_CAPABILITIES, applyCapabilitiesToModules } from "../modules/builtinModules";
import { createModuleRegistry } from "../modules/moduleRegistry";
import { createEmptyGraph, createSampleGraph } from "./createGraph";

function makeRegistry() {
  return createModuleRegistry(
    applyCapabilitiesToModules(DEFAULT_CAPABILITIES),
    DEFAULT_CAPABILITIES
  );
}

describe("createGraph", () => {
  it("creates empty graphs with default validation and compile state", () => {
    const graph = createEmptyGraph(makeRegistry());

    expect(graph.nodes).toEqual([]);
    expect(graph.edges).toEqual([]);
    expect(graph.validation_state.issue_counts).toEqual({ error: 0, warning: 0, info: 0 });
    expect(graph.compile_summary.outputs.executions).toBe(0);
    expect(graph.metadata.source_mode).toBe("graph");
  });

  it("creates the sample graph with runtime, strategy nodes, and seed edges", () => {
    const graph = createSampleGraph(makeRegistry());

    expect(graph.nodes.map((node) => node.type)).toEqual([
      "runtime",
      "data",
      "intent",
      "intent",
      "agent",
      "risk",
      "execution"
    ]);
    expect(graph.edges).toHaveLength(6);
    expect(graph.edges[0]).toEqual(
      expect.objectContaining({
        source_port: "market_data_out",
        target_port: "data_input",
        edge_type: "data_to_intent"
      })
    );
  });
});
