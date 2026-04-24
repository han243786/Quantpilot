import { describe, expect, it } from "vitest";
import { compileGraph } from "./compileGraph";
import { validateGraph } from "./validation";
import { DEFAULT_CAPABILITIES, applyCapabilitiesToModules } from "../modules/builtinModules";
import { createModuleRegistry } from "../modules/moduleRegistry";

function makeRegistry() {
  return createModuleRegistry(
    applyCapabilitiesToModules(DEFAULT_CAPABILITIES),
    DEFAULT_CAPABILITIES
  );
}

function makeNode(id, type, moduleKey, config = {}) {
  return {
    id,
    type,
    module_key: moduleKey,
    name: id,
    config,
    input_ports: [],
    output_ports: [],
    position: { x: 0, y: 0 },
    ui_state: { collapsed: false },
    runtime_state: {
      status: "idle",
      last_event_type: null,
      last_event_time: null,
      last_message: "",
      metrics: {},
      error: null
    }
  };
}

function makeSpreadGraph() {
  return {
    metadata: {
      graph_id: "spread_graph",
      name: "Spread Graph",
      version: "1.0.0",
      runtime_binding: { current_run_id: null, last_compile_id: null },
      editor: { viewport: { x: 0, y: 0, zoom: 1 } },
      artifacts: {}
    },
    nodes: [
      makeNode("quote_binance", "data", "builtin.data.quote", {
        exchange: "binance",
        instrument: "BTCUSDT",
        ping_enabled: true,
        request_interval_ms: 2500
      }),
      makeNode("quote_okx", "data", "builtin.data.quote", {
        exchange: "okx",
        instrument: "BTCUSDT"
      }),
      makeNode("intent_spread", "intent", "builtin.intent.spread_observer", {
        max_time_diff_ms: 5000,
        field_code: 0,
        align_direction_code: 0,
        resample_period_ms: 60000,
        resample_agg_code: 0,
        window_size: 3,
        window_agg_code: 1,
        spread_output_code: 1
      }),
      makeNode("agent_arb", "agent", "builtin.agent.arbitrage", {
        spread_trigger_bps: 30,
        max_quantity_ratio: 0.2
      }),
      makeNode("risk_1", "risk", "builtin.risk.global", {
        max_position: 0.2,
        max_concentration: 0.25,
        max_symbol_net_exposure: 0.22,
        max_portfolio_net_exposure: 0.45,
        max_turnover: 0.4,
        min_trade_weight: 0.02,
        max_new_positions_per_rebalance: 2,
        max_total_leverage: 3,
        max_exchange_leverage: 3,
        min_action_interval_ms: 100
      }),
      makeNode("execution_1", "execution", "builtin.execution.paper", {
        mode: "paper",
        slippage_bps: 5
      }),
      makeNode("runtime_1", "runtime", "builtin.runtime.control", {
        mode: "paper"
      })
    ],
    edges: [
      {
        id: "edge_quote_binance_spread",
        source_node_id: "quote_binance",
        source_port: "market_data_out",
        target_node_id: "intent_spread",
        target_port: "data_input",
        edge_type: "data_to_intent"
      },
      {
        id: "edge_quote_okx_spread",
        source_node_id: "quote_okx",
        source_port: "market_data_out",
        target_node_id: "intent_spread",
        target_port: "data_input",
        edge_type: "data_to_intent"
      },
      {
        id: "edge_spread_agent",
        source_node_id: "intent_spread",
        source_port: "intent_out",
        target_node_id: "agent_arb",
        target_port: "intent_input",
        edge_type: "intent_to_agent"
      },
      {
        id: "edge_agent_risk",
        source_node_id: "agent_arb",
        source_port: "agent_out",
        target_node_id: "risk_1",
        target_port: "agent_input",
        edge_type: "agent_to_risk"
      },
      {
        id: "edge_risk_execution",
        source_node_id: "risk_1",
        source_port: "risk_out",
        target_node_id: "execution_1",
        target_port: "risk_input",
        edge_type: "risk_to_execution"
      }
    ],
    validation_state: {
      is_valid: true,
      is_runnable: true,
      issue_counts: { error: 0, warning: 0, info: 0 },
      graph_issues: [],
      node_issues: {},
      edge_issues: {}
    }
  };
}

describe("spread graph support", () => {
  it("lowers spread observer to spread core ir", () => {
    const result = compileGraph(makeSpreadGraph(), makeRegistry());

    expect(result.compile_summary.compilable).toBe(true);
    expect(result.core_ir.indicators[0].kind).toBe("spread");
    expect(result.core_ir.indicators[0].spread_spec).toEqual(
      expect.objectContaining({
        output: "bps"
      })
    );
    expect(result.core_ir.signal_rules[0].signal_kind).toBe("observe");
    expect(result.core_ir.agent_policies[0].kind).toBe("cross_venue_arbitrage");
    expect(result.core_ir.data_bindings[0].source_hints.ping_enabled).toBe("true");
    expect(result.core_ir.data_bindings[0].source_hints.request_interval_ms).toBe("2500");
    expect(result.core_ir.risk_policies[0]).toEqual(
      expect.objectContaining({
        max_concentration_ratio: 0.25,
        max_symbol_net_exposure_ratio: 0.22,
        max_portfolio_net_exposure_ratio: 0.45,
        max_turnover: 0.4,
        min_trade_weight: 0.02,
        max_new_positions_per_rebalance: 2
      })
    );
  });

  it("rejects spread observer without two quote inputs", () => {
    const graph = makeSpreadGraph();
    graph.edges = graph.edges.filter((edge) => edge.id !== "edge_quote_okx_spread");

    const result = validateGraph(graph, makeRegistry());

    expect(result.node_issues.intent_spread).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          code: "SPREAD_INPUT_COUNT"
        })
      ])
    );
  });
});
