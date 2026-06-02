import { describe, expect, it } from "vitest";
import { DEFAULT_CAPABILITIES, applyCapabilitiesToModules } from "../modules/builtinModules";
import { createModuleRegistry } from "../modules/moduleRegistry";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";
import { buildRuntimeConfig } from "./compileGraphRuntimeConfig";

function makeRegistry() {
  return createModuleRegistry(
    applyCapabilitiesToModules(DEFAULT_CAPABILITIES),
    DEFAULT_CAPABILITIES
  );
}

describe("compileGraphRuntimeConfig", () => {
  it("builds runtime config sections and source mappings for a valid graph", () => {
    const graph = buildValidatedSampleGraph(makeRegistry());
    const result = buildRuntimeConfig(graph, makeRegistry());

    expect(result.compileId).toMatch(/^compile_/);
    expect(result.errors).toEqual([]);
    expect(result.warnings).toEqual([]);
    expect(result.output.metadata).toEqual(
      expect.objectContaining({
        graph_id: graph.metadata.graph_id,
        name: graph.metadata.name,
        mode: "paper"
      })
    );
    expect(result.output.runtime_control).toEqual(expect.objectContaining({ module_key: "builtin.runtime.control" }));
    expect(result.output.data_sources.length).toBeGreaterThan(0);
    expect(result.output.intent_generators.length).toBeGreaterThan(0);
    expect(result.output.agents.length).toBeGreaterThan(0);
    expect(result.output.risk_controls.length).toBe(1);
    expect(result.output.executions.length).toBe(1);
    expect(Object.keys(result.output.mappings.source_id_to_node_id).length).toBe(
      graph.nodes.length
    );
  });

  it("collects local runtime config errors for unsupported rebalance symbols", () => {
    const graph = buildValidatedSampleGraph(makeRegistry(), (draft) => {
      const agentNode = draft.nodes.find((node) => node.type === "agent");
      agentNode.config.rebalance_symbols = "BTCUSDT, XRPUSDT";
      agentNode.config.rebalance_schedule = "every_1d";
      agentNode.config.rebalance_allocation_kind = "equal_weight";
    });

    const result = buildRuntimeConfig(graph, makeRegistry());

    expect(result.errors).toEqual(
      expect.arrayContaining([expect.stringContaining("XRPUSDT")])
    );
  });
});
