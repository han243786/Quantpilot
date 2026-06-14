import { describe, expect, it } from "vitest";
import { parseGraphQuantScript } from "./quantscript";
import { parseGraphQuantScriptSource } from "./quantscriptParser";

const modules = {
  "builtin.data.kline": {
    module_key: "builtin.data.kline",
    category: "data",
    config_schema: {
      fields: [
        { key: "instrument", default: "ETHUSDT" },
        { key: "timeframe", default: "1d" },
        { key: "window_size", default: 200 }
      ]
    },
    ports: {
      inputs: [],
      outputs: [{ key: "market_data_out" }]
    }
  },
  "builtin.intent.rsi": {
    module_key: "builtin.intent.rsi",
    category: "intent",
    config_schema: {
      fields: [{ key: "period", default: 14 }]
    },
    ports: {
      inputs: [{ key: "data_input" }],
      outputs: [{ key: "intent_out" }]
    }
  }
};

const registry = {
  getByKey(key) {
    return modules[key] || null;
  },
  getByCategory(category) {
    return Object.values(modules).filter((moduleDef) => moduleDef.category === category);
  }
};

const source = `strategy_graph imported_graph {
  name: "Imported Graph"
  version: "2.0.0"
  mode: "paper"

  nodes:
    plugin data_feed uses builtin.data.kline
      name: "Price Feed"
      category: "data"
      config:
        instrument: "BTCUSDT"
        window_size: 120

    plugin intent_rsi uses builtin.intent.rsi
      name: "RSI Entry"
      category: "intent"
      config:
        period: 10
      inputs:
        - from: data_feed.market_data_out
          to: intent_rsi.data_input

  graph:
    connect data_feed.market_data_out -> intent_rsi.data_input
}`;

describe("quantscriptParser", () => {
  it("parses graph source into raw graph shape without attaching artifacts", () => {
    const previousGraph = {
      metadata: {
        description: "kept",
        created_at: 111,
        runtime_binding: { current_run_id: "run_1", last_compile_id: "compile_1" },
        editor: { viewport: { x: 1, y: 2, zoom: 0.5 } },
        artifacts: { existing: true }
      },
      nodes: [
        {
          id: "data_feed",
          module_key: "builtin.data.kline",
          type: "data",
          name: "Price Feed",
          position: { x: 10, y: 20 },
          config: { exchange: "okx" },
          ui_state: { collapsed: true },
          runtime_state: { status: "idle", metrics: { ticks: 1 } }
        }
      ],
      validation_state: { is_valid: true },
      compile_summary: { compilable: true }
    };

    const graph = parseGraphQuantScriptSource(source, registry, previousGraph);

    expect(graph.metadata).toMatchObject({
      graph_id: "imported_graph",
      name: "Imported Graph",
      description: "kept",
      version: "2.0.0",
      created_at: 111,
      runtime_binding: { current_run_id: "run_1", last_compile_id: "compile_1" },
      editor: { viewport: { x: 1, y: 2, zoom: 0.5 } },
      source_mode: "quantscript",
      artifacts: { existing: true }
    });
    expect(graph.nodes[0]).toMatchObject({
      id: "data_feed",
      type: "data",
      module_key: "builtin.data.kline",
      name: "Price Feed",
      position: { x: 10, y: 20 },
      config: {
        exchange: "okx",
        instrument: "BTCUSDT",
        timeframe: "1d",
        window_size: 120
      },
      ui_state: { collapsed: true },
      runtime_state: { status: "idle", metrics: { ticks: 1 } }
    });
    expect(graph.nodes[1]).toMatchObject({
      id: "intent_rsi",
      type: "intent",
      module_key: "builtin.intent.rsi",
      name: "RSI Entry",
      config: { period: 10 }
    });
    expect(graph.edges).toEqual([
      {
        id: "edge_data_feed_intent_rsi_market_data_out_data_input",
        source_node_id: "data_feed",
        source_port: "market_data_out",
        target_node_id: "intent_rsi",
        target_port: "data_input",
        edge_type: "data_feed-intent_rsi"
      }
    ]);
    expect(graph.metadata.artifacts.quantscript).toBeUndefined();
  });

  it("keeps the parent facade responsible for artifact attachment", () => {
    const graph = parseGraphQuantScript(source, registry);

    expect(graph.metadata.artifacts.quantscript.graph_source).toContain("strategy_graph imported_graph");
    expect(graph.metadata.artifacts.quantscript.node_sources.data_feed).toContain("plugin data_feed uses builtin.data.kline");
  });
});
