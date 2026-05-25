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
  graph.metadata.template_runtime_version = template.runtimeVersion || "v3";
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

function v4Field(name, typeName, required = true) {
  return { name, type_name: typeName, required, nullable: false };
}

function createV4TemplateMachineGraph(template, variant = {}) {
  const graphId = `v4_${template.id}`;
  const symbols = template.symbols || ["BTCUSDT"];
  const observationId = `${graphId}.observation`;
  const decisionId = `${graphId}.decision`;
  const executionId = `${graphId}.execution`;
  const marketEvent = "market.bar_closed";
  const observedEvent = `${graphId}.observation_ready`;
  const approvedEvent = `${graphId}.risk_approved`;
  const baseFields = [
    v4Field("venue_id", "venue"),
    v4Field("symbol", "symbol"),
    v4Field("price", "price"),
    v4Field("close", "price"),
    v4Field("ts_ms", "u64")
  ];
  const orderFields = [
    ...baseFields,
    v4Field("action", "position_side"),
    v4Field("order_type", "order_type"),
    v4Field("quantity", "number"),
    v4Field("time_in_force", "time_in_force", false)
  ];

  return {
    schema_version: "quantpilot/machine-graph-contract/v1",
    graph_id: graphId,
    metadata: {
      default_symbol: symbols[0],
      symbols,
      action: variant.action || "buy",
      order_type: variant.orderType || "market",
      quantity: variant.quantity || 1,
      time_in_force: variant.timeInForce || "gtc"
    },
    machines: [
      {
        schema_version: "quantpilot/machine-contract/v1",
        machine_id: observationId,
        template: "observation",
        states: [
          { state_id: "waiting", initial: true, terminal: false },
          { state_id: "observed", initial: false, terminal: false }
        ],
        state_groups: [],
        transitions: [
          {
            transition_id: "observe_bar",
            from_state: "waiting",
            to_state: "observed",
            event: {
              event_type: marketEvent,
              source: "market.okx",
              freshness: "fresh_or_stale"
            },
            priority: 100,
            action: {
              emits: [observedEvent],
              memory_writes: ["venue_id", "symbol", "price", "close", "ts_ms"],
              diagnostics: ["v4 backtest observed deterministic bar"]
            }
          }
        ],
        memory: baseFields.map((field) => ({
          name: field.name,
          type_name: field.type_name,
          default_value:
            field.name === "symbol"
              ? symbols[0]
              : field.name === "venue_id"
                ? "paper-local"
                : field.name === "price" || field.name === "close"
                  ? 20000
                  : 0,
          nullable: false
        })),
        cache_policy: "return_last_then_recover",
        silence_policy: { kind: "soft_dormant_after", ttl_ms: 300000 },
        recovery_policy: "async_recover",
        priority: 100,
        metadata: { symbols }
      },
      {
        schema_version: "quantpilot/machine-contract/v1",
        machine_id: decisionId,
        template: "decision",
        states: [
          { state_id: "ready", initial: true, terminal: false },
          { state_id: "approved", initial: false, terminal: false }
        ],
        state_groups: [],
        transitions: [
          {
            transition_id: "approve_observation",
            from_state: "ready",
            to_state: "approved",
            event: {
              event_type: observedEvent,
              source: observationId,
              freshness: "fresh_or_stale"
            },
            priority: 9100,
            action: {
              emits: [approvedEvent],
              memory_writes: ["venue_id", "symbol", "price", "close", "ts_ms"],
              diagnostics: ["v4 backtest risk plane approved observation"]
            }
          }
        ],
        memory: orderFields.map((field) => ({
          name: field.name,
          type_name: field.type_name,
          default_value:
            field.name === "action"
              ? variant.action || "buy"
              : field.name === "order_type"
                ? variant.orderType || "market"
                : field.name === "quantity"
                  ? variant.quantity || 1
                  : field.name === "time_in_force"
                    ? variant.timeInForce || "gtc"
                    : field.name === "symbol"
                      ? symbols[0]
                      : field.name === "venue_id"
                        ? "paper-local"
                        : field.name === "price" || field.name === "close"
                          ? 20000
                          : 0,
          nullable: false
        })),
        cache_policy: "return_last_then_recover",
        silence_policy: { kind: "soft_dormant_after", ttl_ms: 300000 },
        recovery_policy: "async_recover",
        priority: 9200,
        metadata: { symbols }
      },
      {
        schema_version: "quantpilot/machine-contract/v1",
        machine_id: executionId,
        template: "execution",
        states: [
          { state_id: "idle", initial: true, terminal: false },
          { state_id: "submitted", initial: false, terminal: false }
        ],
        state_groups: [],
        transitions: [
          {
            transition_id: "submit_paper_order",
            from_state: "idle",
            to_state: "submitted",
            event: {
              event_type: approvedEvent,
              source: decisionId,
              freshness: "fresh_or_stale"
            },
            priority: 100,
            action: {
              emits: [],
              memory_writes: [],
              diagnostics: ["v4 backtest submitted local simulated order"]
            }
          }
        ],
        memory: [],
        cache_policy: "no_cache",
        silence_policy: { kind: "manual_only" },
        recovery_policy: "manual_recover",
        priority: 100,
        metadata: {
          core_execution_id: "paper-local",
          core_venue_kind: "paper-local",
          symbols
        }
      }
    ],
    edges: [
      {
        edge_id: "observation_to_decision",
        source_machine_id: observationId,
        target_machine_id: decisionId,
        event_type: observedEvent,
        activation: "always",
        required: true,
        metadata: {}
      },
      {
        edge_id: "decision_to_execution",
        source_machine_id: decisionId,
        target_machine_id: executionId,
        event_type: approvedEvent,
        activation: "runtime_gated",
        required: true,
        metadata: {}
      }
    ],
    event_catalog: {
      schema_version: "quantpilot/machine-event-catalog/v1",
      events: [
        {
          event_type: marketEvent,
          source_kind: "market_data",
          scope: "graph",
          payload_fields: baseFields,
          allowed_emitters: ["market.okx"],
          allowed_consumers: [observationId],
          replayable: true
        },
        {
          event_type: observedEvent,
          source_kind: "machine",
          scope: "graph",
          payload_fields: baseFields,
          allowed_emitters: [observationId],
          allowed_consumers: [decisionId],
          replayable: true
        },
        {
          event_type: approvedEvent,
          source_kind: "risk_plane",
          scope: "graph",
          payload_fields: orderFields,
          allowed_emitters: [decisionId],
          allowed_consumers: [executionId],
          replayable: true
        }
      ],
      metadata: { template_id: template.id }
    },
    risk_plane: {
      required: true,
      machine_ids: [decisionId],
      min_priority: 9000
    }
  };
}

function buildV4RuntimeTemplate(registry, template, variant = {}) {
  const graph = clone(createSampleGraph(registry));
  withTemplateMetadata(graph, template);
  graph.metadata.runtime_kind = "v4";
  graph.metadata.artifacts = {
    ...(graph.metadata.artifacts || {}),
    v4_machine_graph: createV4TemplateMachineGraph(template, variant),
    v4_symbols: template.symbols || ["BTCUSDT"]
  };
  const runtime = graph.nodes.find((node) => node.module_key === "builtin.runtime.control");
  const kline = graph.nodes.find((node) => node.module_key === "builtin.data.kline");
  const execution = graph.nodes.find((node) => node.module_key === "builtin.execution.paper");
  if (runtime) runtime.name = "v4 Runtime Control";
  if (kline) {
    kline.name = `${template.titleEn || template.title} Market Data`;
    kline.config.exchange = "okx";
    kline.config.instrument = template.symbols?.[0] || "BTCUSDT";
    kline.config.timeframe = variant.timeframe || "1h";
  }
  if (execution) {
    execution.name = "v4 Paper Simulated Execution";
    execution.config.slippage_bps = variant.slippageBps || 5;
  }
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
  },
  {
    id: "dual_ma_v4",
    title: "v4 Dual MA",
    titleEn: "v4 Dual MA",
    category: "v4",
    description: "v4 state-machine template with deterministic paper-simulated backtest evidence.",
    descriptionEn: "v4 state-machine template with deterministic paper-simulated backtest evidence.",
    defaultName: "v4 Dual MA Template",
    supportedModules: [
      "builtin.data.kline",
      "builtin.intent.double_ma",
      "builtin.agent.weighted",
      "builtin.risk.global",
      "builtin.execution.paper"
    ],
    symbols: ["BTCUSDT"],
    docAnchor: "dual-ma-v4",
    runtimeVersion: "v4",
    variant: { action: "buy", orderType: "market", quantity: 1, timeframe: "1h" }
  },
  {
    id: "grid_v4",
    title: "v4 Grid",
    titleEn: "v4 Grid",
    category: "v4",
    description: "v4 grid-style template for local simulated limit-capable execution evidence.",
    descriptionEn: "v4 grid-style template for local simulated limit-capable execution evidence.",
    defaultName: "v4 Grid Template",
    supportedModules: [
      "builtin.data.kline",
      "builtin.intent.ma_deviation",
      "builtin.agent.weighted",
      "builtin.risk.global",
      "builtin.execution.paper"
    ],
    symbols: ["BTCUSDT"],
    docAnchor: "grid-v4",
    runtimeVersion: "v4",
    variant: { action: "buy", orderType: "limit", quantity: 0.5, timeframe: "15m" }
  },
  {
    id: "stop_loss_v4",
    title: "v4 Stop Loss",
    titleEn: "v4 Stop Loss",
    category: "v4",
    description: "v4 stop-loss template that records Risk Plane and capability decisions.",
    descriptionEn: "v4 stop-loss template that records Risk Plane and capability decisions.",
    defaultName: "v4 Stop Loss Template",
    supportedModules: [
      "builtin.data.kline",
      "builtin.intent.rsi",
      "builtin.agent.weighted",
      "builtin.risk.global",
      "builtin.execution.paper"
    ],
    symbols: ["ETHUSDT"],
    docAnchor: "stop-loss-v4",
    runtimeVersion: "v4",
    variant: { action: "sell", orderType: "stop_market", quantity: 0.75, timeframe: "1h" }
  },
  {
    id: "multi_symbol_v4",
    title: "v4 Multi Symbol",
    titleEn: "v4 Multi Symbol",
    category: "v4",
    description: "v4 universe template that expands independent machine instances per symbol.",
    descriptionEn: "v4 universe template that expands independent machine instances per symbol.",
    defaultName: "v4 Multi Symbol Template",
    supportedModules: [
      "builtin.data.kline",
      "builtin.intent.double_ma",
      "builtin.agent.weighted",
      "builtin.risk.global",
      "builtin.execution.paper"
    ],
    symbols: ["BTCUSDT", "ETHUSDT", "SOLUSDT"],
    docAnchor: "multi-symbol-v4",
    runtimeVersion: "v4",
    variant: { action: "buy", orderType: "market", quantity: 0.4, timeframe: "1h" }
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
    case "dual_ma_v4":
    case "grid_v4":
    case "stop_loss_v4":
    case "multi_symbol_v4":
      return buildV4RuntimeTemplate(registry, template, template.variant || {});
    default:
      throw new Error("Unknown strategy template.");
  }
}
