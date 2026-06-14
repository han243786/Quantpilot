import { describe, expect, it } from "vitest";
import { DEFAULT_CAPABILITIES, applyCapabilitiesToModules } from "../modules/builtinModules";
import { createModuleRegistry } from "../modules/moduleRegistry";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";
import { buildCoreIr } from "./compileGraphCoreIr";

function makeRegistry() {
  return createModuleRegistry(
    applyCapabilitiesToModules(DEFAULT_CAPABILITIES),
    DEFAULT_CAPABILITIES
  );
}

function makeCompileOutput(graph) {
  return {
    mappings: {
      source_id_to_node_id: Object.fromEntries(
        graph.nodes.map((node) => [`${node.type}_${node.id}`, node.id])
      )
    }
  };
}

describe("compileGraphCoreIr", () => {
  it("lowers a validated graph into core ir sections", () => {
    const graph = buildValidatedSampleGraph(makeRegistry());
    const coreIr = buildCoreIr(graph, makeCompileOutput(graph));

    expect(coreIr.ir_version).toBe("quantpilot/core-ir/v1");
    expect(coreIr.metadata).toEqual(
      expect.objectContaining({
        strategy_id: graph.metadata.graph_id,
        source_kind: "frontend_graph"
      })
    );
    expect(coreIr.data_bindings.length).toBeGreaterThan(0);
    expect(coreIr.indicators.length).toBeGreaterThan(0);
    expect(coreIr.signal_rules.length).toBe(coreIr.indicators.length);
    expect(coreIr.agent_policies.length).toBeGreaterThan(0);
    expect(coreIr.risk_policies.length).toBeGreaterThan(0);
    expect(coreIr.execution).toEqual(
      expect.objectContaining({
        venue_kind: "paper",
        sizing_kind: "equity_notional_ratio"
      })
    );
  });

  it("keeps weighted rebalance config in portfolio policy output", () => {
    const graph = buildValidatedSampleGraph(makeRegistry(), (draft) => {
      const agentNode = draft.nodes.find((node) => node.type === "agent");
      agentNode.config.rebalance_symbols = "BTCUSDT, ETHUSDT, SOLUSDT";
      agentNode.config.rebalance_schedule = "weekly";
      agentNode.config.rebalance_allocation_kind = "fixed_weights";
      agentNode.config.rebalance_target_weights = "0.5, 0.3, 0.2";
    });

    const coreIr = buildCoreIr(graph, makeCompileOutput(graph));

    expect(coreIr.agent_policies[0]).toEqual(
      expect.objectContaining({
        kind: "portfolio_rebalance",
        rebalance_symbols: ["BTCUSDT", "ETHUSDT", "SOLUSDT"],
        rebalance_schedule: "weekly",
        rebalance_allocation_kind: "fixed_weights",
        rebalance_target_weights: [0.5, 0.3, 0.2]
      })
    );
  });
});
