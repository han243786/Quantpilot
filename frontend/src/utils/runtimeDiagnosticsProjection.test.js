import { describe, expect, it } from "vitest";
import { buildRuntimeDiagnosticsProjection } from "./runtimeDiagnosticsProjection";

function buildGraph() {
  return {
    nodes: [
      {
        id: "risk",
        name: "Risk",
        type: "risk",
        runtime_state: {
          status: "warning",
          last_event_type: "RiskDecisionProduced",
          last_event_time: 1_710_000_008_000,
          last_message: "Risk decision produced",
          metrics: {},
          error: null
        }
      },
      {
        id: "execution",
        name: "Execution",
        type: "execution",
        runtime_state: {
          status: "running",
          last_event_type: "ExecutionPlanned",
          last_event_time: 1_710_000_010_000,
          last_message: "Execution planned",
          metrics: {},
          error: null
        }
      }
    ]
  };
}

describe("runtimeDiagnosticsProjection", () => {
  it("derives data quality rows from raw runtime events", () => {
    const runtime = {
      highlightedNodeIds: ["risk", "execution"],
      events: [
        {
          event_id: "evt_exec_1",
          event_type: "ExecutionPlanned",
          node_id: "execution",
          event_time_ms: 1_710_000_010_000,
          severity: "Info",
          summary: "Execution planned",
          payload: {
            side: "Buy",
            qty: 0.25,
            limit_price: 50_100,
            remaining_qty: 0.25,
            sizing_source: "portfolio_target_diff",
            order_type_decision_reason: "plan_executes_immediately_when_submitted",
            explanation_summary: "Execution plan sized from portfolio target diff.",
            order_previews: [
              {
                order_id: "ord_001",
                side: "Buy",
                qty: 0.25,
                order_type: "Market",
                order_type_decision_reason: "plan_executes_immediately_when_submitted"
              }
            ]
          }
        },
        {
          event_id: "evt_data_1",
          event_type: "DataUpdated",
          node_id: "execution",
          event_time_ms: 1_710_000_011_000,
          severity: "Warn",
          summary: "Data quality warning",
          payload: {
            latest_price: 50_100,
            source_status: "Stale",
            source_health: "Delayed",
            freshness_ms: 120_000,
            stale_after_ms: 60_000,
            source_latency_ms: 5_500,
            gap_count: 2,
            quality_flags: ["delayed_update", "gaps_detected"],
            explanation_summary: "BTCUSDT quote quality delayed with 2 missing intervals."
          }
        },
        {
          event_id: "evt_risk_1",
          event_type: "RiskDecisionProduced",
          node_id: "risk",
          event_time_ms: 1_710_000_008_000,
          severity: "Warn",
          summary: "Risk decision produced",
          payload: {
            status: "Clamped",
            limit_triggered: "max_single_weight",
            sizing_mode: "portfolio_targets",
            reason_text: "Clamped to global max single weight.",
            explanation_summary: "Risk clamp applied before execution.",
            pre_risk: {
              max_target_weight: 0.45,
              concentration_ratio: 0.45,
              max_symbol_net_exposure_ratio: 0.45,
              portfolio_net_exposure_ratio: 0.75,
              turnover_ratio: 0.62
            },
            post_risk: {
              max_target_weight: 0.2,
              concentration_ratio: 0.2,
              max_symbol_net_exposure_ratio: 0.2,
              portfolio_net_exposure_ratio: 0.45,
              turnover_ratio: 0.31
            }
          }
        }
      ]
    };

    const executionProjection = buildRuntimeDiagnosticsProjection(buildGraph(), runtime, "execution");
    expect(executionProjection.explanationSummary).toBe(
      "BTCUSDT quote quality delayed with 2 missing intervals."
    );
    expect(executionProjection.dataQualityRows).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ key: "source_health", value: "Delayed" }),
        expect.objectContaining({ key: "gap_count", value: "2" })
      ])
    );

    const riskProjection = buildRuntimeDiagnosticsProjection(buildGraph(), runtime, "risk");
    expect(riskProjection.explanationSummary).toBe("Risk clamp applied before execution.");
    expect(riskProjection.riskDetailRows).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ key: "limit_triggered", value: "max_single_weight" }),
        expect.objectContaining({ key: "pre_risk.max_target_weight", value: "0.4500" }),
        expect.objectContaining({ key: "post_risk.max_target_weight", value: "0.2000" }),
        expect.objectContaining({ key: "post_risk.portfolio_net_exposure_ratio", value: "0.4500" })
      ])
    );
  });

  it("falls back to the first active node when no explicit selection exists", () => {
    const runtime = {
      highlightedNodeIds: ["execution"],
      events: [
        {
          event_id: "evt_exec_1",
          event_type: "ExecutionPlanned",
          node_id: "execution",
          event_time_ms: 1_710_000_010_000,
          severity: "Info",
          summary: "Execution planned",
          payload: {
            side: "Buy",
            qty: 0.25,
            limit_price: 50_100,
            remaining_qty: 0.25
          }
        }
      ]
    };

    const projection = buildRuntimeDiagnosticsProjection(buildGraph(), runtime, null);

    expect(projection.selectedNodeId).toBe("execution");
    expect(projection.latestInputRows).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ key: "side", value: "Buy" }),
        expect.objectContaining({ key: "qty", value: "0.2500" })
      ])
    );
  });

  it("prefers structured runtime diagnostics when detail payload already projected them", () => {
    const runtime = {
      diagnostics: {
        source: "backtest_event_log",
        default_selected_node_id: "execution",
        active_nodes: [
          {
            node_id: "execution",
            latest_event_type: "ExecutionPlanned",
            latest_event_label: "执行计划",
            latest_event_time_ms: 1_710_000_010_000,
            event_count: 2
          }
        ],
        node_details: {
          execution: {
            node_id: "execution",
            latest_event: {
              event_id: "evt_exec_1",
              event_type: "ExecutionPlanned",
              label: "执行计划",
              summary: "Server-projected execution",
              tone: "info",
              severity: "Info",
              event_time_ms: 1_710_000_010_000
            },
            explanation_summary: "Server-projected execution explanation",
            latest_input_rows: [{ key: "qty", label: "数量", value: "0.2500" }],
            latest_output_rows: [{ key: "remaining_qty", label: "剩余数量", value: "0.2500" }],
            explanation_rows: [
              { key: "sizing_source", label: "定量来源", value: "portfolio_target_diff" }
            ],
            data_quality_rows: [
              { key: "source_health", label: "源健康", value: "Healthy" },
              { key: "freshness_ms", label: "新鲜度(ms)", value: "0" }
            ],
            order_detail_rows: [
              {
                key: "order_type_decision_reason",
                label: "下单语义",
                value: "plan_executes_immediately_when_submitted"
              }
            ],
            risk_detail_rows: [],
            latest_notice: null,
            recent_events: [],
            event_count: 2
          }
        }
      },
      highlightedNodeIds: ["execution"],
      events: [
        {
          event_id: "evt_exec_1",
          event_type: "ExecutionPlanned",
          node_id: "execution",
          event_time_ms: 1_710_000_010_000,
          severity: "Info",
          summary: "Local fallback should not win",
          payload: {
            qty: 0.25
          }
        }
      ]
    };

    const projection = buildRuntimeDiagnosticsProjection(buildGraph(), runtime, null);

    expect(projection.selectedNodeId).toBe("execution");
    expect(projection.latestEvent.summary).toBe("Server-projected execution");
    expect(projection.explanationSummary).toBe("Server-projected execution explanation");
    expect(projection.dataQualityRows).toEqual(
      expect.arrayContaining([expect.objectContaining({ key: "source_health", value: "Healthy" })])
    );
    expect(projection.orderDetailRows).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          key: "order_type_decision_reason",
          value: "plan_executes_immediately_when_submitted"
        })
      ])
    );
  });
});
