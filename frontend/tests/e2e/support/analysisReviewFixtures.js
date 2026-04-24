import { backendCapabilitiesFixture } from "../../../src/test/fixtures/capabilities/capabilityFallbacks";
import { backendCompileOkFixture } from "../../../src/test/fixtures/runtime/capabilityRejections";
import { buildBacktestSuccessFixture } from "../../../src/test/fixtures/runtime/backtestSuccess";
import { buildRunSuccessFixture } from "../../../src/test/fixtures/runtime/runSuccess";
import { createApiMockHarness } from "./apiHarness";

export const REVIEW_GRAPH_ID = "visual_review_graph";
export const REVIEW_COMPILE_ID = "compile_visual_001";

export function buildReviewGraphFixture() {
  const now = 1_700_000_000_000;
  return {
    metadata: {
      graph_id: REVIEW_GRAPH_ID,
      name: "Cross-Market Review Strategy",
      version: "1.0.0",
      created_at: now,
      updated_at: now,
      runtime_binding: {
        current_run_id: "run_smoke_001",
        last_compile_id: REVIEW_COMPILE_ID
      },
      editor: {
        viewport: { x: -160, y: -48, zoom: 0.74 }
      },
      source_mode: "graph",
      artifacts: {
        quantscript: {
          graph_source: "strategy_graph visual_review_graph {}"
        },
        strategy_ir: {
          source: JSON.stringify(
            {
              strategy_id: REVIEW_GRAPH_ID,
              signals: ["double_ma", "momentum"],
              risk: "global"
            },
            null,
            2
          )
        }
      }
    },
    nodes: [
      {
        id: "node_runtime_1",
        type: "runtime",
        module_key: "builtin.runtime.control",
        name: "Runtime Control",
        position: { x: 40, y: 32 },
        config: { mode: "paper" },
        input_ports: [],
        output_ports: [],
        ui_state: { collapsed: false },
        runtime_state: {
          status: "completed",
          last_event_type: "PortfolioUpdated",
          last_event_time: now,
          last_message: "Runtime finished with replay artifacts.",
          metrics: { event_count: 4, runtime_mode: "paper" },
          error: null
        }
      },
      {
        id: "node_data_1",
        type: "data",
        module_key: "builtin.data.kline",
        name: "Binance BTCUSDT 1m",
        position: { x: 140, y: 160 },
        config: { exchange: "binance", symbol: "BTCUSDT", interval: "1m", lookback: 240 },
        input_ports: [],
        output_ports: [{ key: "market_data_out", label: "Market Data" }],
        ui_state: { collapsed: false },
        runtime_state: {
          status: "completed",
          last_event_type: "DataUpdated",
          last_event_time: now,
          last_message: "Market data healthy.",
          metrics: { latest_price: 50200 },
          error: null
        }
      },
      {
        id: "node_intent_1",
        type: "intent",
        module_key: "builtin.intent.double_ma",
        name: "Fast / Slow Trend",
        position: { x: 430, y: 132 },
        config: { fast_period: 12, slow_period: 48, threshold: 0.02 },
        input_ports: [{ key: "data_input", label: "Data" }],
        output_ports: [{ key: "intent_out", label: "Intent" }],
        ui_state: { collapsed: false },
        runtime_state: {
          status: "completed",
          last_event_type: "IntentTriggered",
          last_event_time: now,
          last_message: "Signal strength 0.82.",
          metrics: { signal_strength: 0.82 },
          error: null
        }
      },
      {
        id: "node_intent_2",
        type: "intent",
        module_key: "builtin.intent.momentum",
        name: "Momentum Check",
        position: { x: 430, y: 326 },
        config: { lookback: 24, threshold: 0.6 },
        input_ports: [{ key: "data_input", label: "Data" }],
        output_ports: [{ key: "intent_out", label: "Intent" }],
        ui_state: { collapsed: false },
        runtime_state: {
          status: "waiting",
          last_event_type: "IntentEvaluated",
          last_event_time: now,
          last_message: "Momentum review pending.",
          metrics: { confidence: 0.71 },
          error: null
        }
      },
      {
        id: "node_agent_1",
        type: "agent",
        module_key: "builtin.agent.weighted",
        name: "Weighted Decision Agent",
        position: { x: 740, y: 218 },
        config: { decision_threshold: 0.1, max_quantity_ratio: 0.25 },
        input_ports: [{ key: "intent_input", label: "Intent" }],
        output_ports: [{ key: "agent_out", label: "Agent Output" }],
        ui_state: { collapsed: false },
        runtime_state: {
          status: "completed",
          last_event_type: "AgentDecisionProduced",
          last_event_time: now,
          last_message: "Net long bias accepted.",
          metrics: { score: 0.74 },
          error: null
        }
      },
      {
        id: "node_risk_1",
        type: "risk",
        module_key: "builtin.risk.global",
        name: "Global Risk Guard",
        position: { x: 1045, y: 218 },
        config: { max_position: 0.25, max_leverage: 2 },
        input_ports: [{ key: "agent_input", label: "Agent Input" }],
        output_ports: [{ key: "risk_out", label: "Risk Output" }],
        ui_state: { collapsed: false },
        runtime_state: {
          status: "completed",
          last_event_type: "RiskDecisionProduced",
          last_event_time: now,
          last_message: "Exposure stayed within limits.",
          metrics: { risk_action: "allow" },
          error: null
        }
      },
      {
        id: "node_execution_1",
        type: "execution",
        module_key: "builtin.execution.paper",
        name: "Paper Execution",
        position: { x: 1340, y: 218 },
        config: { slippage_bps: 5, taker_fee_bps: 10 },
        input_ports: [{ key: "risk_input", label: "Risk Input" }],
        output_ports: [{ key: "execution_out", label: "Execution Output" }],
        ui_state: { collapsed: false },
        runtime_state: {
          status: "completed",
          last_event_type: "ExecutionFilled",
          last_event_time: now,
          last_message: "Executed 0.10 BTC buy.",
          metrics: { fill_qty: 0.1, fill_price: 50200 },
          error: null
        }
      }
    ],
    edges: [
      {
        id: "edge_data_double_ma",
        source_node_id: "node_data_1",
        source_port: "market_data_out",
        target_node_id: "node_intent_1",
        target_port: "data_input",
        edge_type: "data_to_intent"
      },
      {
        id: "edge_data_momentum",
        source_node_id: "node_data_1",
        source_port: "market_data_out",
        target_node_id: "node_intent_2",
        target_port: "data_input",
        edge_type: "data_to_intent"
      },
      {
        id: "edge_double_ma_agent",
        source_node_id: "node_intent_1",
        source_port: "intent_out",
        target_node_id: "node_agent_1",
        target_port: "intent_input",
        edge_type: "intent_to_agent"
      },
      {
        id: "edge_momentum_agent",
        source_node_id: "node_intent_2",
        source_port: "intent_out",
        target_node_id: "node_agent_1",
        target_port: "intent_input",
        edge_type: "intent_to_agent"
      },
      {
        id: "edge_agent_risk",
        source_node_id: "node_agent_1",
        source_port: "agent_out",
        target_node_id: "node_risk_1",
        target_port: "agent_input",
        edge_type: "agent_to_risk"
      },
      {
        id: "edge_risk_execution",
        source_node_id: "node_risk_1",
        source_port: "risk_out",
        target_node_id: "node_execution_1",
        target_port: "risk_input",
        edge_type: "risk_to_execution"
      }
    ],
    validation_state: {
      is_valid: true,
      is_runnable: true,
      node_issues: {},
      edge_issues: {},
      graph_issues: [],
      issue_counts: { error: 0, warning: 1, info: 1 },
      last_validated_at: now
    },
    compile_summary: {
      compilable: true,
      last_compile_id: REVIEW_COMPILE_ID,
      last_compile_at: now,
      topology_order: [
        "node_data_1",
        "node_intent_1",
        "node_intent_2",
        "node_agent_1",
        "node_risk_1",
        "node_execution_1"
      ],
      outputs: {
        data_sources: 1,
        intent_generators: 2,
        agents: 1,
        risk_controls: 1,
        executions: 1
      },
      warnings: ["Strategy IR preflight passed with one advisory note."],
      errors: [],
      runtime_source: "runtime_compile"
    }
  };
}

export async function installAnalysisReviewMocks(page) {
  const api = await createApiMockHarness(page);
  const graphFixture = buildReviewGraphFixture();
  const backtestPrimary = buildBacktestSuccessFixture({
    graphId: REVIEW_GRAPH_ID,
    compileId: REVIEW_COMPILE_ID,
    backtestId: "backtest_smoke_001"
  });
  const backtestSecondary = buildBacktestSuccessFixture({
    graphId: REVIEW_GRAPH_ID,
    compileId: "compile_visual_002",
    backtestId: "backtest_compare_002"
  });
  const runFixture = buildRunSuccessFixture({
    graphId: REVIEW_GRAPH_ID,
    compileId: REVIEW_COMPILE_ID,
    runId: "run_smoke_001"
  });

  await api.json("**/api/capabilities", backendCapabilitiesFixture);
  await api.json("**/api/quantscript/formal/compile", backendCompileOkFixture);
  await api.json("**/api/runtime/compile", backendCompileOkFixture);
  await api.json("**/api/graphs/latest", graphFixture);
  await api.json(`**/api/graphs/${REVIEW_GRAPH_ID}`, graphFixture);
  await api.json("**/api/runtime/runs", runFixture.historyResponse);
  await api.handle("**/api/runtime/runs/*", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json; charset=utf-8",
      body: JSON.stringify(runFixture.detailResponse)
    });
  });
  await api.json("**/api/runtime/backtests", [
    ...backtestPrimary.historyResponse,
    ...backtestSecondary.historyResponse
  ]);
  await api.handle("**/api/runtime/backtests/*", async (route) => {
    const backtestId = decodeURIComponent(new URL(route.request().url()).pathname.split("/").pop());
    const fixture = backtestId === "backtest_compare_002" ? backtestSecondary : backtestPrimary;
    await route.fulfill({
      status: 200,
      contentType: "application/json; charset=utf-8",
      body: JSON.stringify(fixture.detailResponse)
    });
  });
  await api.installGuard();

  return { api };
}
