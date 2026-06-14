import { createNodeFromModule } from "./createNode";
import {
  createGraphEdge,
  createInitialCompileSummary,
  createInitialValidationState
} from "./graphFactoryDefaults";

export function createSampleGraph(registry) {
  const runtimeNode = createNodeFromModule(registry.getByKey("builtin.runtime.control"));
  const kline = createNodeFromModule(registry.getByKey("builtin.data.kline"));
  const longBuy = createNodeFromModule(registry.getByKey("builtin.intent.double_ma"));
  const longSell = createNodeFromModule(registry.getByKey("builtin.intent.ma_deviation"));
  const longAgent = createNodeFromModule(registry.getByKey("builtin.agent.weighted"));
  const risk = createNodeFromModule(registry.getByKey("builtin.risk.global"));
  const execution = createNodeFromModule(registry.getByKey("builtin.execution.paper"));

  kline.name = "OKX BTC 日线行情";
  kline.config.exchange = "okx";
  kline.config.instrument = "BTCUSDT";
  kline.config.timeframe = "1d";
  kline.config.window_size = 200;
  longBuy.name = "双均线入场";
  longBuy.config.fast_period = 20;
  longBuy.config.slow_period = 50;
  longBuy.config.entry_ratio = 0.2;
  longSell.name = "趋势退出";
  longSell.config.lookback = 20;
  longSell.config.baseline_period = 50;
  longSell.config.threshold_ratio = 1.0;
  longAgent.name = "趋势决策代理";
  longAgent.config.decision_threshold = 0.05;
  longAgent.config.max_quantity_ratio = 0.2;
  risk.name = "全局风控";
  risk.config.max_position = 0.2;
  risk.config.max_total_leverage = 3;
  risk.config.max_exchange_leverage = 3;
  risk.config.min_action_interval_ms = 100;
  execution.name = "模拟执行";
  execution.config.mode = "paper";
  execution.config.slippage_bps = 5;
  runtimeNode.name = "运行控制";
  runtimeNode.config.mode = "paper";

  const edges = [
    createGraphEdge(kline, "market_data_out", longBuy, "data_input"),
    createGraphEdge(kline, "market_data_out", longSell, "data_input"),
    createGraphEdge(longBuy, "intent_out", longAgent, "intent_input"),
    createGraphEdge(longSell, "intent_out", longAgent, "intent_input"),
    createGraphEdge(longAgent, "agent_out", risk, "agent_input"),
    createGraphEdge(risk, "risk_out", execution, "risk_input")
  ];

  return {
    metadata: {
      graph_id: `graph_${Date.now()}`,
      name: "OKX 双均线趋势策略图",
      description: "使用 OKX V5 HTTP 行情接口驱动的最小可运行趋势策略图。",
      version: "1.0.0",
      created_at: Date.now(),
      updated_at: Date.now(),
      runtime_binding: {
        current_run_id: null,
        last_compile_id: null
      },
      editor: {
        viewport: { x: 0, y: 0, zoom: 0.8 },
        recent_node_ids: []
      },
      source_mode: "graph",
      artifacts: {}
    },
    nodes: [runtimeNode, kline, longBuy, longSell, longAgent, risk, execution],
    edges,
    validation_state: createInitialValidationState(),
    compile_summary: createInitialCompileSummary()
  };
}

export function createEmptyGraph(registry) {
  const now = Date.now();

  return {
    metadata: {
      graph_id: `graph_${now}`,
      name: "未命名策略图",
      description: "",
      version: "1.0.0",
      created_at: now,
      updated_at: now,
      runtime_binding: {
        current_run_id: null,
        last_compile_id: null
      },
      editor: {
        viewport: { x: 0, y: 0, zoom: 0.8 },
        recent_node_ids: []
      },
      source_mode: "graph",
      artifacts: {}
    },
    nodes: [],
    edges: [],
    validation_state: createInitialValidationState(),
    compile_summary: createInitialCompileSummary()
  };
}
