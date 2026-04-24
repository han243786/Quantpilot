import { describe, expect, it } from "vitest";
import { buildNodeCardData, formatNodeMetricLabel } from "./nodeCardPresentation";

function createRegistry(moduleDef) {
  return {
    getByKey(moduleKey) {
      return moduleKey === moduleDef.module_key ? moduleDef : null;
    }
  };
}

describe("nodeCardPresentation", () => {
  it("formats data-node metrics with explicit health, freshness, latency, and gap truth", () => {
    expect(
      formatNodeMetricLabel({
        type: "data",
        runtime_state: {
          metrics: {
            latest_price: 123.45,
            source_status: "实时"
          }
        }
      })
    ).toBe("价格 123.45 · 实时");

    expect(
      formatNodeMetricLabel({
        type: "data",
        runtime_state: {
          metrics: {
            latest_price: 123.45,
            source_health: "Delayed",
            freshness_ms: 90000,
            stale_after_ms: 60000,
            source_latency_ms: 250,
            gap_count: 2
          }
        }
      })
    ).toBe("价格 123.45 · 延迟 · 新鲜 90000/60000ms · 延迟 250ms · 缺口 2");
  });

  it("formats risk-node metrics with active guard summaries", () => {
    expect(
      formatNodeMetricLabel({
        type: "risk",
        runtime_state: {
          metrics: {
            risk_action: "reject",
            risk_score: "0.9",
            limit_triggered: "max_portfolio_net_exposure_ratio",
            concentration_ratio: 0.42,
            max_symbol_net_exposure_ratio: 0.25,
            portfolio_net_exposure_ratio: 0.6
          }
        }
      })
    ).toBe(
      "reject 0.9 · 限制 max_portfolio_net_exposure_ratio · 集中度 42.0% · 单标的净敞口 25.0% · 组合净敞口 60.0%"
    );
  });

  it("precomputes summary, quick fields, issue and highlight state", () => {
    const moduleDef = {
      module_key: "builtin.intent.demo",
      display_name: "示例意图",
      node: {
        summary_fields: ["fast", "slow", "source"],
        quick_fields: ["fast", "source"]
      },
      config_schema: {
        fields: [
          { key: "fast", label: "快线", type: "number" },
          {
            key: "source",
            label: "数据源",
            type: "select",
            options: [{ value: "close", label: "收盘价" }]
          }
        ]
      }
    };
    const node = {
      id: "node_1",
      type: "intent",
      name: "双均线",
      module_key: "builtin.intent.demo",
      config: { fast: 5, slow: 20, source: "close" },
      input_ports: [{ key: "in" }],
      output_ports: [{ key: "out" }],
      runtime_state: {
        status: "running",
        metrics: { signal_direction: "long", signal_strength: "0.8" }
      },
      ui_state: { collapsed: false }
    };

    const data = buildNodeCardData({
      node,
      registry: createRegistry(moduleDef),
      nodeIssues: { node_1: [{ message: "需要连接执行模块" }] },
      highlightedNodeIds: ["node_1"],
      simplified: true,
      focusMode: "issues",
      focusedNodeIds: new Set(["node_1"])
    });

    expect(data.title).toBe("双均线");
    expect(data.subtitle).toBe("示例意图");
    expect(data.summaryValues).toEqual(["5", "20"]);
    expect(data.quickFieldDefinitions).toEqual([
      { key: "fast", label: "快线", type: "number", options: [], value: 5 },
      {
        key: "source",
        label: "数据源",
        type: "select",
        options: [{ value: "close", label: "收盘价" }],
        value: "close"
      }
    ]);
    expect(data.issueMessage).toBe("需要连接执行模块");
    expect(data.metricLabel).toBe("long 0.8");
    expect(data.highlighted).toBe(true);
    expect(data.runtimeStatus).toBe("running");
    expect(data.focusMode).toBe("issues");
    expect(data.dimmed).toBe(false);
  });

  it("dims nodes that are outside the active focus set", () => {
    const moduleDef = {
      module_key: "builtin.intent.demo",
      display_name: "示例意图",
      node: { summary_fields: [], quick_fields: [] },
      config_schema: { fields: [] }
    };
    const node = {
      id: "node_2",
      type: "intent",
      name: "未聚焦节点",
      module_key: "builtin.intent.demo",
      config: {},
      input_ports: [],
      output_ports: [],
      runtime_state: { status: "idle", metrics: {} },
      ui_state: { collapsed: false }
    };

    const data = buildNodeCardData({
      node,
      registry: createRegistry(moduleDef),
      nodeIssues: {},
      highlightedNodeIds: [],
      simplified: false,
      focusMode: "recent",
      focusedNodeIds: new Set(["another"])
    });

    expect(data.focusMode).toBe(null);
    expect(data.dimmed).toBe(true);
  });

  it("marks recommendation roles for repair paths and next-fix nodes", () => {
    const moduleDef = {
      module_key: "builtin.execution.paper",
      display_name: "Execution module",
      node: { summary_fields: [], quick_fields: [] },
      config_schema: { fields: [] }
    };
    const node = {
      id: "node_3",
      type: "execution",
      name: "Paper",
      module_key: "builtin.execution.paper",
      config: {},
      input_ports: [],
      output_ports: [],
      runtime_state: { status: "idle", metrics: {} },
      ui_state: { collapsed: false }
    };

    const pathData = buildNodeCardData({
      node,
      registry: createRegistry(moduleDef),
      nodeIssues: { node_3: [{ message: "broken execution" }] },
      highlightedNodeIds: [],
      simplified: false,
      recommendedNodeIds: new Set(["node_3", "node_4"]),
      repairPathNodeIds: ["node_2", "node_3", "node_4"],
      selectedNodeId: "node_2"
    });
    const endData = buildNodeCardData({
      node: {
        ...node,
        id: "node_4",
        name: "Risk guard",
        type: "risk",
        module_key: "builtin.risk.guard"
      },
      registry: createRegistry({ ...moduleDef, module_key: "builtin.risk.guard" }),
      nodeIssues: { node_4: [{ message: "broken risk" }] },
      highlightedNodeIds: [],
      simplified: false,
      recommendedNodeIds: new Set(["node_3", "node_4"]),
      repairPathNodeIds: ["node_2", "node_3", "node_4"],
      selectedNodeId: "node_2"
    });

    expect(pathData.recommendationRole).toBe("path");
    expect(endData.recommendationRole).toBe("path-end");
  });
});
