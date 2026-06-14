import { describe, expect, it } from "vitest";
import {
  buildQuantScriptLabelTargets,
  buildQuantScriptRuntimeTargets
} from "./quantscriptArtifactTargets";

function buildTargetGraph() {
  return {
    nodes: [
      {
        id: "data_feed",
        type: "data",
        module_key: "builtin.data.kline",
        name: "Price Feed",
        config: { instrument: "BTCUSDT", timeframe: "1d" }
      },
      {
        id: "intent_rsi",
        type: "intent",
        module_key: "builtin.intent.rsi",
        name: "RSI Entry",
        config: { period: 14 }
      },
      {
        id: "agent_main",
        type: "agent",
        name: "Agent",
        config: {}
      },
      {
        id: "risk_main",
        type: "risk",
        name: "Risk",
        config: {}
      },
      {
        id: "runtime_node",
        type: "runtime",
        name: "Runtime",
        config: {}
      },
      {
        id: "execution_node",
        type: "execution",
        name: "Execution",
        config: {}
      }
    ]
  };
}

describe("quantscriptArtifactTargets", () => {
  it("builds label targets for nodes, config fields, and formal bindings", () => {
    const targets = buildQuantScriptLabelTargets(buildTargetGraph());

    expect(targets.data_feed).toMatchObject({
      scope: "node",
      node_id: "data_feed",
      label: "Price Feed"
    });
    expect(targets["Price Feed.instrument"]).toMatchObject({
      scope: "node",
      node_id: "data_feed",
      field: "instrument",
      label: "Price Feed.instrument"
    });
    expect(targets.data_data_feed).toMatchObject({
      scope: "node",
      node_id: "data_feed"
    });
    expect(targets.intent_intent_rsi_signal).toMatchObject({
      scope: "node",
      node_id: "intent_rsi"
    });
  });

  it("builds runtime targets for formal source mapping and runtime endpoints", () => {
    expect(buildQuantScriptRuntimeTargets(buildTargetGraph())).toEqual({
      source_to_node: {
        data_data_feed: "data_feed",
        intent_intent_rsi: "intent_rsi",
        agent_script_main: "agent_main",
        risk_script_global: "risk_main"
      },
      runtime_node_id: "runtime_node",
      execution_node_id: "execution_node"
    });
  });
});
