import { describe, expect, it } from "vitest";
import {
  buildDiagnosticsExplanationEntries,
  getEventExplanationSummary
} from "./runtimeExplanation";

describe("runtimeExplanation", () => {
  const graph = {
    nodes: [
      { id: "risk_node", name: "Risk Guard" },
      { id: "execution_node", name: "Execution Desk" },
      { id: "data_node", name: "Price Feed" }
    ]
  };

  const diagnostics = {
    node_details: {
      risk_node: {
        node_id: "risk_node",
        explanation_summary: "Risk clamp applied before execution.",
        risk_detail_rows: [{ key: "limit_triggered", label: "触发限制", value: "max_single_weight" }],
        order_detail_rows: [],
        data_quality_rows: []
      },
      execution_node: {
        node_id: "execution_node",
        explanation_summary: "Execution plan sized from portfolio target diff.",
        risk_detail_rows: [],
        order_detail_rows: [{ key: "sizing_source", label: "定量来源", value: "portfolio_target_diff" }],
        data_quality_rows: []
      },
      data_node: {
        node_id: "data_node",
        explanation_summary: "BTCUSDT quote quality delayed with 2 missing intervals.",
        risk_detail_rows: [],
        order_detail_rows: [],
        data_quality_rows: [{ key: "gap_count", label: "缺口数量", value: "2" }]
      }
    }
  };

  it("builds explanation entries from the persisted diagnostics detail family only", () => {
    expect(buildDiagnosticsExplanationEntries(graph, diagnostics, "risk")).toEqual([
      {
        nodeId: "risk_node",
        nodeName: "Risk Guard",
        explanationSummary: "Risk clamp applied before execution.",
        rows: [{ key: "limit_triggered", label: "触发限制", value: "max_single_weight" }]
      }
    ]);
    expect(buildDiagnosticsExplanationEntries(graph, diagnostics, "order")).toEqual([
      {
        nodeId: "execution_node",
        nodeName: "Execution Desk",
        explanationSummary: "Execution plan sized from portfolio target diff.",
        rows: [{ key: "sizing_source", label: "定量来源", value: "portfolio_target_diff" }]
      }
    ]);
    expect(buildDiagnosticsExplanationEntries(graph, diagnostics, "dataQuality")).toEqual([
      {
        nodeId: "data_node",
        nodeName: "Price Feed",
        explanationSummary: "BTCUSDT quote quality delayed with 2 missing intervals.",
        rows: [{ key: "gap_count", label: "缺口数量", value: "2" }]
      }
    ]);
  });

  it("returns null when the explanation only duplicates the event summary", () => {
    expect(
      getEventExplanationSummary({
        summary: "Filled immediately.",
        payload: {
          explanation_summary: "Filled immediately."
        }
      })
    ).toBeNull();
  });

  it("prefers structured explanation summary and falls back to reason_text", () => {
    expect(
      getEventExplanationSummary({
        summary: "Order planned.",
        payload: {
          explanation_summary: "Execution plan sized from portfolio target diff.",
          reason_text: "legacy reason text"
        }
      })
    ).toBe("Execution plan sized from portfolio target diff.");
    expect(
      getEventExplanationSummary({
        summary: "Order planned.",
        payload: {
          reason_text: "legacy reason text"
        }
      })
    ).toBe("legacy reason text");
  });
});
