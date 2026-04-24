import { describe, expect, it } from "vitest";
import { compileGraph } from "./compileGraph";
import { DEFAULT_CAPABILITIES, applyCapabilitiesToModules } from "../modules/builtinModules";
import { createModuleRegistry } from "../modules/moduleRegistry";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";

function makeRegistry() {
  return createModuleRegistry(
    applyCapabilitiesToModules(DEFAULT_CAPABILITIES),
    DEFAULT_CAPABILITIES
  );
}

describe("multi-symbol graph/runtime lowering", () => {
  it("lowers weighted agent rebalance config into portfolio rebalance core ir", () => {
    const registry = makeRegistry();
    const graph = buildValidatedSampleGraph(registry, (draft) => {
      const agentNode = draft.nodes.find((node) => node.type === "agent");
      agentNode.config.decision_threshold = 0.05;
      agentNode.config.max_quantity_ratio = 0.6;
      agentNode.config.rebalance_symbols = "BTCUSDT, ETHUSDT, SOLUSDT";
      agentNode.config.rebalance_schedule = "weekly";
      agentNode.config.rebalance_allocation_kind = "fixed_weights";
      agentNode.config.rebalance_target_weights = "0.5, 0.3, 0.2";
    });

    const result = compileGraph(graph, registry);
    const agentPolicy = result.core_ir.agent_policies[0];

    expect(result.compile_summary.compilable).toBe(true);
    expect(agentPolicy.kind).toBe("portfolio_rebalance");
    expect(agentPolicy.rebalance_symbols).toEqual(["BTCUSDT", "ETHUSDT", "SOLUSDT"]);
    expect(agentPolicy.rebalance_schedule).toBe("weekly");
    expect(agentPolicy.rebalance_allocation_kind).toBe("fixed_weights");
    expect(agentPolicy.rebalance_target_weights).toEqual([0.5, 0.3, 0.2]);
    expect(result.runtime_config.agents[0].config.rebalance_symbols).toBe(
      "BTCUSDT, ETHUSDT, SOLUSDT"
    );
  });

  it("rejects unsupported rebalance symbols during local graph compile", () => {
    const registry = makeRegistry();
    const graph = buildValidatedSampleGraph(registry, (draft) => {
      const agentNode = draft.nodes.find((node) => node.type === "agent");
      agentNode.config.rebalance_symbols = "BTCUSDT, XRPUSDT";
      agentNode.config.rebalance_schedule = "every_1d";
      agentNode.config.rebalance_allocation_kind = "equal_weight";
    });

    const result = compileGraph(graph, registry);

    expect(result.compile_summary.compilable).toBe(false);
    expect(result.compile_summary.errors).toEqual(
      expect.arrayContaining([expect.stringContaining("XRPUSDT")])
    );
  });
});
