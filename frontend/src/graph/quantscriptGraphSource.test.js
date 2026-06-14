import { describe, expect, it } from "vitest";
import { generateGraphQuantScript, generateNodeQuantScript } from "./quantscriptGraphSource";

function buildGraphSourceGraph() {
  return {
    metadata: {
      graph_id: "source_graph",
      name: "Source Graph",
      version: "1.2.3"
    },
    nodes: [
      {
        id: "runtime_node",
        type: "runtime",
        module_key: "builtin.runtime.control",
        name: "Runtime",
        config: { mode: "paper" },
        input_ports: [],
        output_ports: [{ key: "runtime_out" }]
      },
      {
        id: "intent_node",
        type: "intent",
        module_key: "builtin.intent.rsi",
        name: "RSI Entry",
        config: { period: 14, enabled: true, note: "entry" },
        input_ports: [{ key: "data_input" }],
        output_ports: [{ key: "intent_out" }]
      }
    ],
    edges: [
      {
        source_node_id: "runtime_node",
        source_port: "runtime_out",
        target_node_id: "intent_node",
        target_port: "data_input"
      }
    ]
  };
}

describe("quantscriptGraphSource", () => {
  it("generates node source with config and incoming connections", () => {
    const graph = buildGraphSourceGraph();

    expect(generateNodeQuantScript(graph.nodes[1], graph)).toContain("plugin intent_node uses builtin.intent.rsi");
    expect(generateNodeQuantScript(graph.nodes[1], graph)).toContain("    period: 14");
    expect(generateNodeQuantScript(graph.nodes[1], graph)).toContain("    enabled: true");
    expect(generateNodeQuantScript(graph.nodes[1], graph)).toContain("    note: \"entry\"");
    expect(generateNodeQuantScript(graph.nodes[1], graph)).toContain("- from: runtime_node.runtime_out");
  });

  it("generates graph source with metadata, nodes, and connections", () => {
    const source = generateGraphQuantScript(buildGraphSourceGraph());

    expect(source).toContain("strategy_graph source_graph {");
    expect(source).toContain("  name: \"Source Graph\"");
    expect(source).toContain("  version: \"1.2.3\"");
    expect(source).toContain("  mode: \"paper\"");
    expect(source).toContain("connect runtime_node.runtime_out -> intent_node.data_input");
  });
});
