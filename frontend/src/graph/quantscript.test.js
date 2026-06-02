import { describe, expect, it } from "vitest";
import {
  attachQuantScriptArtifacts,
  generateFormalQuantScript,
  generateGraphQuantScript
} from "./quantscript";

describe("generateFormalQuantScript", () => {
  it("preserves exchange and runtime targets on formal artifacts", () => {
    const graph = attachQuantScriptArtifacts({
      metadata: {
        graph_id: "formal_exchange_graph",
        name: "Formal Exchange Graph",
        version: "1.0.0",
        runtime_binding: { current_run_id: null, last_compile_id: null },
        editor: { viewport: { x: 0, y: 0, zoom: 0.8 } },
        artifacts: {}
      },
      nodes: [
        {
          id: "data_feed",
          type: "data",
          module_key: "builtin.data.kline",
          name: "Price Feed",
          position: { x: 0, y: 0 },
          config: {
            exchange: "okx",
            instrument: "BTCUSDT",
            timeframe: "1d",
            window_size: 120
          },
          input_ports: [],
          output_ports: [{ key: "market_data_out" }],
          ui_state: { collapsed: false },
          runtime_state: { status: "idle", last_event_type: null, last_event_time: null, last_message: "", metrics: {}, error: null }
        },
        {
          id: "intent_rsi",
          type: "intent",
          module_key: "builtin.intent.rsi",
          name: "RSI Entry",
          position: { x: 320, y: 0 },
          config: {
            period: 14,
            oversold_threshold: 30,
            overbought_threshold: 70
          },
          input_ports: [{ key: "data_input" }],
          output_ports: [{ key: "intent_out" }],
          ui_state: { collapsed: false },
          runtime_state: { status: "idle", last_event_type: null, last_event_time: null, last_message: "", metrics: {}, error: null }
        }
      ],
      edges: [
        {
          id: "edge_data_rsi",
          source_node_id: "data_feed",
          source_port: "market_data_out",
          target_node_id: "intent_rsi",
          target_port: "data_input",
          edge_type: "data_to_intent"
        }
      ],
      validation_state: {
        is_valid: true,
        is_runnable: true,
        issue_counts: { error: 0, warning: 0, info: 0 },
        graph_issues: [],
        node_issues: {},
        edge_issues: {}
      },
      compile_summary: {}
    });

    expect(graph.metadata.artifacts.quantscript.formal_source).toContain(
      'let data_data_feed_series = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=120)?'
    );
    expect(graph.metadata.artifacts.quantscript.formal_source).toContain(
      "let intent_intent_rsi_signal = rsi(data_data_feed_series, 14)"
    );
    expect(graph.metadata.artifacts.quantscript.runtime_targets).toEqual({
      source_to_node: {
        data_data_feed: "data_feed",
        intent_intent_rsi: "intent_rsi"
      },
      runtime_node_id: null,
      execution_node_id: null
    });
    expect(generateFormalQuantScript(graph)).toBe(graph.metadata.artifacts.quantscript.formal_source);
    expect(generateGraphQuantScript(graph)).toBe(graph.metadata.artifacts.quantscript.graph_source);
  });
});
