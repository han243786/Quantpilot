import { describe, expect, it } from "vitest";
import {
  parseQuantScriptDiagnosticsFromMessage,
  resolveCompileDiagnosticTargetFromGraphArtifacts
} from "./graphStore";
import { useGraphStore } from "./graphStore";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";

function buildGraph() {
  return buildValidatedSampleGraph(useGraphStore.getState().registry, (graph) => {
    graph.metadata.graph_id = "diag_graph";
    graph.metadata.name = "Diagnostics Graph";
    const node = graph.nodes.find((item) => item.id === "data_feed") || graph.nodes[0];
    if (node) {
      node.id = "data_feed";
      node.type = "data";
      node.module_key = "builtin.data.kline";
      node.name = "Price Feed";
      node.config = {
        exchange: "binance",
        instrument: "BTCUSDT",
        timeframe: "1d",
        window_size: 20
      };
      node.output_ports = [{ key: "market_data_out" }];
    }
  });
}

describe("graphStore QuantScript diagnostic mapping", () => {
  it("maps artifact labels to node field targets", () => {
    const graph = buildGraph();
    expect(resolveCompileDiagnosticTargetFromGraphArtifacts(graph, "Price Feed.window_size")).toEqual(
      expect.objectContaining({
        scope: "node",
        node_id: "data_feed",
        field: "window_size",
        label: "Price Feed.window_size"
      })
    );
  });

  it("parses QuantScript compiler messages into actionable diagnostics", () => {
    const graph = buildGraph();
    const diagnostics = parseQuantScriptDiagnosticsFromMessage(
      "formal QuantScript semantic analysis failed:\nQS0501: warmup is insufficient [Price Feed.window_size]",
      graph
    );

    expect(diagnostics).toHaveLength(1);
    expect(diagnostics[0]).toEqual(
      expect.objectContaining({
        code: "QS0501",
        severity: "error",
        span_label: "Price Feed.window_size",
        target: expect.objectContaining({
          node_id: "data_feed",
          field: "window_size"
        })
      })
    );
  });
});
