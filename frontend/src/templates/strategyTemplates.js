import { createSampleGraph } from "../graph/createGraph";
import { createNodeFromModule } from "../graph/createNode";
import { attachQuantScriptArtifacts } from "../graph/quantscript";
import { validateGraph } from "../graph/validation";

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function connect(sourceNode, sourcePort, targetNode, targetPort) {
  return {
    id: `edge_${sourceNode.id}_${targetNode.id}_${sourcePort}_${targetPort}`,
    source_node_id: sourceNode.id,
    source_port: sourcePort,
    target_node_id: targetNode.id,
    target_port: targetPort,
    edge_type: `${sourceNode.type}_to_${targetNode.type}`
  };
}

function finalizeTemplateGraph(graph, registry) {
  const normalized = attachQuantScriptArtifacts(graph);
  return {
    ...normalized,
    validation_state: validateGraph(normalized, registry)
  };
}

function withTemplateMetadata(graph, template) {
  const now = Date.now();
  graph.metadata.graph_id = `template_${template.id}_${now}`;
  graph.metadata.name = template.defaultName;
  graph.metadata.description = template.description;
  graph.metadata.updated_at = now;
  graph.metadata.created_at = now;
  graph.metadata.template_id = template.id;
  graph.metadata.template_label = template.title;
  return graph;
}

function buildTrendTemplate(registry, template) {
  const graph = clone(createSampleGraph(registry));
  withTemplateMetadata(graph, template);
  const kline = graph.nodes.find((node) => node.module_key === "builtin.data.kline");
  const entryIntent = graph.nodes.find((node) => node.module_key === "builtin.intent.double_ma");
  const exitIntent = graph.nodes.find((node) => node.module_key === "builtin.intent.ma_deviation");
  const agent = graph.nodes.find((node) => node.module_key === "builtin.agent.weighted");
  const risk = graph.nodes.find((node) => node.module_key === "builtin.risk.global");
  const execution = graph.nodes.find((node) => node.module_key === "builtin.execution.paper");
  const runtime = graph.nodes.find((node) => node.module_key === "builtin.runtime.control");

  runtime.name = "趋势运行控制";

  kline.name = "BTC 趋势行情数据";
  kline.config.exchange = "okx";
  kline.config.instrument = "BTCUSDT";
  kline.config.timeframe = "1d";
  kline.config.window_size = 240;

  entryIntent.name = "趋势入场意图";
  entryIntent.config.fast_period = 12;
  entryIntent.config.slow_period = 36;
  entryIntent.config.entry_ratio = 0.05;

  exitIntent.name = "偏离离场意图";
  exitIntent.config.lookback = 8;
  exitIntent.config.baseline_period = 30;
  exitIntent.config.threshold_ratio = 0.35;

  agent.name = "趋势配置代理";
  agent.config.decision_threshold = 0.015;
  agent.config.max_quantity_ratio = 0.65;

  risk.name = "趋势组合风控";
  risk.config.max_position = 0.6;
  risk.config.max_total_leverage = 3;

  execution.name = "趋势模拟执行";
  execution.config.slippage_bps = 6;

  return finalizeTemplateGraph(graph, registry);
}

function buildRsiTemplate(registry, template) {
  const runtimeNode = createNodeFromModule(registry.getByKey("builtin.runtime.control"));
  const kline = createNodeFromModule(registry.getByKey("builtin.data.kline"));
  const rsi = createNodeFromModule(registry.getByKey("builtin.intent.rsi"));
  const agent = createNodeFromModule(registry.getByKey("builtin.agent.weighted"));
  const risk = createNodeFromModule(registry.getByKey("builtin.risk.global"));
  const execution = createNodeFromModule(registry.getByKey("builtin.execution.paper"));
  const now = Date.now();

  runtimeNode.name = "均值回归运行控制";
  runtimeNode.config.mode = "paper";

  kline.name = "ETH 均值回归行情数据";
  kline.config.exchange = "okx";
  kline.config.instrument = "ETHUSDT";
  kline.config.timeframe = "4h";
  kline.config.window_size = 180;

  rsi.name = "RSI 回归意图";
  rsi.config.period = 10;
  rsi.config.oversold_threshold = 45;
  rsi.config.overbought_threshold = 55;

  agent.name = "回归配置代理";
  agent.config.decision_threshold = 0.015;
  agent.config.max_quantity_ratio = 0.5;

  risk.name = "回归风控策略";
  risk.config.max_position = 0.45;
  risk.config.max_total_leverage = 2.5;

  execution.name = "回归模拟执行";
  execution.config.slippage_bps = 4;

  const graph = {
    metadata: {
      graph_id: `template_${template.id}_${now}`,
      name: template.defaultName,
      description: template.description,
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
      template_id: template.id,
      template_label: template.title,
      artifacts: {}
    },
    nodes: [runtimeNode, kline, rsi, agent, risk, execution],
    edges: [
      connect(kline, "market_data_out", rsi, "data_input"),
      connect(rsi, "intent_out", agent, "intent_input"),
      connect(agent, "agent_out", risk, "agent_input"),
      connect(risk, "risk_out", execution, "risk_input")
    ],
    validation_state: {
      is_valid: false,
      is_runnable: false,
      node_issues: {},
      edge_issues: {},
      graph_issues: [],
      issue_counts: { error: 0, warning: 0, info: 0 },
      last_validated_at: null
    },
    compile_summary: {
      compilable: false,
      last_compile_id: null,
      last_compile_at: null,
      topology_order: [],
      outputs: {
        data_sources: 0,
        intent_generators: 0,
        agents: 0,
        risk_controls: 0,
        executions: 0
      },
      warnings: [],
      errors: []
    }
  };

  return finalizeTemplateGraph(graph, registry);
}

function buildRebalanceTemplate(registry, template) {
  const graph = clone(createSampleGraph(registry));
  withTemplateMetadata(graph, template);
  const kline = graph.nodes.find((node) => node.module_key === "builtin.data.kline");
  const agent = graph.nodes.find((node) => node.module_key === "builtin.agent.weighted");
  const risk = graph.nodes.find((node) => node.module_key === "builtin.risk.global");
  const execution = graph.nodes.find((node) => node.module_key === "builtin.execution.paper");
  const runtime = graph.nodes.find((node) => node.module_key === "builtin.runtime.control");

  runtime.name = "组合运行控制";

  kline.name = "组合基准行情";
  kline.config.instrument = "BTCUSDT";
  kline.config.timeframe = "1d";
  kline.config.window_size = 120;

  agent.name = "多标的再平衡代理";
  agent.config.decision_threshold = 0.01;
  agent.config.max_quantity_ratio = 0.8;
  agent.config.rebalance_symbols = "BTCUSDT, ETHUSDT, SOLUSDT";
  agent.config.rebalance_schedule = "every_1d";
  agent.config.rebalance_allocation_kind = "fixed_weights";
  agent.config.rebalance_target_weights = "0.5, 0.3, 0.2";

  risk.name = "组合敞口风控";
  risk.config.max_position = 0.75;
  risk.config.max_total_leverage = 3;
  risk.config.max_concentration = 0.85;
  risk.config.max_symbol_net_exposure = 0.85;
  risk.config.max_portfolio_net_exposure = 0.95;

  execution.name = "组合模拟执行";
  execution.config.slippage_bps = 5;

  return finalizeTemplateGraph(graph, registry);
}

export const STRATEGY_TEMPLATE_LIBRARY = [
  {
    id: "dual_ma_trend",
    title: "双均线趋势",
    titleEn: "Dual MA Trend",
    category: "趋势",
    description:
      "双均线入场、均线偏离离场的趋势跟随起始策略图。",
    descriptionEn: "Trend-following starter with dual-MA entry and MA-deviation exit.",
    defaultName: "双均线趋势起始模板",
    supportedModules: [
      "builtin.data.kline",
      "builtin.intent.double_ma",
      "builtin.intent.ma_deviation",
      "builtin.agent.weighted",
      "builtin.risk.global",
      "builtin.execution.paper"
    ],
    symbols: ["BTCUSDT"],
    docAnchor: "dual-ma-trend"
  },
  {
    id: "rsi_reversion",
    title: "RSI 均值回归",
    titleEn: "RSI Mean Reversion",
    category: "均值回归",
    description:
      "面向单一 ETH 市场数据源的轻量 RSI 均值回归起始策略图。",
    descriptionEn: "Lightweight RSI mean-reversion starter targeting ETH market data.",
    defaultName: "RSI 均值回归起始模板",
    supportedModules: [
      "builtin.runtime.control",
      "builtin.data.kline",
      "builtin.intent.rsi",
      "builtin.agent.weighted",
      "builtin.risk.global",
      "builtin.execution.paper"
    ],
    symbols: ["ETHUSDT"],
    docAnchor: "rsi-mean-reversion"
  },
  {
    id: "multi_symbol_rebalance",
    title: "多标的再平衡",
    titleEn: "Multi-Symbol Rebalance",
    category: "组合",
    description:
      "使用 BTC、ETH、SOL 权重的多标的再平衡起始策略图。",
    descriptionEn: "Multi-symbol portfolio rebalance starter with BTC, ETH, and SOL weights.",
    defaultName: "多标的再平衡起始模板",
    supportedModules: [
      "builtin.data.kline",
      "builtin.intent.double_ma",
      "builtin.intent.ma_deviation",
      "builtin.agent.weighted",
      "builtin.risk.global",
      "builtin.execution.paper"
    ],
    symbols: ["BTCUSDT", "ETHUSDT", "SOLUSDT"],
    docAnchor: "multi-symbol-rebalance"
  }
];

export function buildStrategyTemplateGraph(templateId, registry) {
  const template = STRATEGY_TEMPLATE_LIBRARY.find((entry) => entry.id === templateId);
  if (!template) {
    throw new Error("Unknown strategy template.");
  }

  switch (templateId) {
    case "dual_ma_trend":
      return buildTrendTemplate(registry, template);
    case "rsi_reversion":
      return buildRsiTemplate(registry, template);
    case "multi_symbol_rebalance":
      return buildRebalanceTemplate(registry, template);
    default:
      throw new Error("Unknown strategy template.");
  }
}
