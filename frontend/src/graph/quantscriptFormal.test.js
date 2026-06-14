import { describe, expect, it } from "vitest";
import {
  canGenerateFormalQuantScript,
  formalDataRuntimeId,
  formalIntentSignalBindingName,
  generateFormalQuantScript
} from "./quantscriptFormal";

function buildFormalGraph(overrides = {}) {
  return {
    nodes: [
      {
        id: "1 price-feed",
        type: "data",
        module_key: "builtin.data.kline",
        config: {
          exchange: "okx",
          instrument: "BTCUSDT",
          timeframe: "1h",
          window_size: 144
        }
      },
      {
        id: "intent-rsi",
        type: "intent",
        module_key: "builtin.intent.rsi",
        config: {
          period: 14,
          oversold_threshold: 25,
          overbought_threshold: 75
        }
      },
      {
        id: "agent-main",
        type: "agent",
        config: { strategy: "weighted", decision_threshold: 0.11 }
      },
      {
        id: "risk-main",
        type: "risk",
        config: { profile_name: "global", max_position: 0.3, max_total_leverage: 2 }
      },
      {
        id: "execution-main",
        type: "execution",
        config: { profile_name: "paper", fee_bps: 8, slippage_bps: 3 }
      },
      {
        id: "runtime-main",
        type: "runtime",
        config: { mode: "paper" }
      }
    ],
    edges: [
      {
        source_node_id: "1 price-feed",
        target_node_id: "intent-rsi"
      }
    ],
    ...overrides
  };
}

describe("quantscriptFormal", () => {
  it("generates formal source and stable runtime bindings for supported graphs", () => {
    const graph = buildFormalGraph();

    expect(canGenerateFormalQuantScript(graph)).toBe(true);
    expect(formalDataRuntimeId(graph.nodes[0])).toBe("data_n_1_price_feed");
    expect(formalIntentSignalBindingName(graph.nodes[1])).toBe("intent_intent_rsi_signal");

    const script = generateFormalQuantScript(graph);

    expect(script).toContain(
      'let data_n_1_price_feed_series = fetch("BTCUSDT", exchange="okx", interval="1h", lookback=144)?'
    );
    expect(script).toContain("agent(\"weighted\", decision_threshold=0.11)");
    expect(script).toContain("risk.profile(\"global\", max_position=0.3, max_total_leverage=2)");
    expect(script).toContain("execution.profile(\"paper\", fee_bps=8, slippage_bps=3)");
    expect(script).toContain("runtime.mode(\"paper\")");
    expect(script).toContain("let intent_intent_rsi_signal = rsi(data_n_1_price_feed_series, 14)");
  });

  it("returns empty source when intent nodes cannot be formally lowered", () => {
    const graph = buildFormalGraph({
      edges: []
    });

    expect(canGenerateFormalQuantScript(graph)).toBe(false);
    expect(generateFormalQuantScript(graph)).toBe("");
  });
});
