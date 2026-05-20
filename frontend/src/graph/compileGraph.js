import { DEFAULT_CAPABILITIES, normalizeCapabilities } from "../modules/builtinModules";
import { attachQuantScriptArtifacts, generateGraphQuantScript } from "./quantscript";

function capabilitySet(values, fallback) {
  return new Set(Array.isArray(values) && values.length > 0 ? values : fallback);
}

function supportMap(entries, keyField = "key") {
  return new Map(
    (Array.isArray(entries) ? entries : [])
      .filter((entry) => entry && typeof entry === "object" && entry[keyField])
      .map((entry) => [entry[keyField], entry])
  );
}

function capabilityEntryStatus(entry, fallbackSet, key) {
  if (entry) return entry.status === "supported";
  return fallbackSet.has(key);
}

function capabilityReason(entry, fallback = "") {
  return entry?.reason || fallback;
}

function jsonValue(value) {
  return value === undefined ? null : value;
}

function parseCsvStrings(value) {
  if (typeof value !== "string") {
    return [];
  }
  return value
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
}

function parseCsvNumbers(value) {
  return parseCsvStrings(value)
    .map((item) => Number(item))
    .filter((item) => Number.isFinite(item));
}

function normalizeOptionalString(value) {
  return typeof value === "string" ? value.trim() : "";
}

function normalizeRebalanceSchedule(value) {
  const normalized = normalizeOptionalString(value);
  if (!normalized) return null;
  if (["every_slow", "every_1d", "weekly"].includes(normalized)) {
    return normalized;
  }
  return "__invalid__";
}

function normalizeRebalanceAllocationKind(value) {
  const normalized = normalizeOptionalString(value);
  if (!normalized) return null;
  if (["equal_weight", "score_weight", "rank_weight", "fixed_weights"].includes(normalized)) {
    return normalized;
  }
  return "__invalid__";
}

function normalizeRebalanceRankMethod(value) {
  const normalized = normalizeOptionalString(value);
  if (!normalized) return null;
  if (["linear", "inverse_rank"].includes(normalized)) {
    return normalized;
  }
  return "__invalid__";
}

function normalizeRebalanceScoreNormalize(value) {
  const normalized = normalizeOptionalString(value);
  if (!normalized) return null;
  if (["sum"].includes(normalized)) {
    return normalized;
  }
  return "__invalid__";
}

function agentUsesPortfolioRebalance(config = {}) {
  return (
    parseCsvStrings(config.rebalance_symbols).length > 0 ||
    Boolean(normalizeOptionalString(config.rebalance_schedule)) ||
    Boolean(normalizeOptionalString(config.rebalance_allocation_kind)) ||
    Boolean(normalizeOptionalString(config.rebalance_rank_method)) ||
    Boolean(normalizeOptionalString(config.rebalance_score_normalize)) ||
    Boolean(normalizeOptionalString(config.rebalance_target_weights))
  );
}

function makeCompileDiagnostic(severity, message, code = null, target = null, hint = null) {
  return {
    source: "graph",
    code: code || (severity === "warning" ? "GRAPH_COMPILE_WARNING" : "GRAPH_COMPILE_ERROR"),
    severity,
    message,
    target,
    hint
  };
}

function buildLocalCompileDiagnostics(errors, warnings) {
  return [
    ...errors.map((message) => makeCompileDiagnostic("error", message)),
    ...warnings.map((message) => makeCompileDiagnostic("warning", message))
  ];
}

function lowerIntentModuleToCoreKind(moduleKey) {
  switch (moduleKey) {
    case "builtin.intent.double_ma":
    case "builtin.intent.ma_deviation":
      return "ma_cross";
    case "builtin.intent.rsi":
      return "rsi";
    case "builtin.intent.macd":
      return "macd";
    case "builtin.intent.momentum":
      return "momentum";
    case "builtin.intent.zscore":
      return "z_score";
    case "builtin.intent.spread_observer":
      return "spread";
    default:
      // v2.3.3 修复 S0-6: 未知意图不再静默降级为 ma_cross, 抛出明确错误
      throw new Error(
        `[compileGraph] 不支持的意图模块: "${moduleKey}"。` +
        `当前支持: builtin.intent.double_ma / ma_deviation / rsi / macd / momentum / zscore / spread_observer`
      );
  }
}

function lowerIntentNodeToCondition(node) {
  switch (node.module_key) {
    case "builtin.intent.double_ma":
      return {
        kind: "raw_text",
        source: `ma_cross(fast=${node.config.fast_period}, slow=${node.config.slow_period}, entry_ratio=${node.config.entry_ratio})`
      };
    case "builtin.intent.ma_deviation":
      return {
        kind: "raw_text",
        source: `ma_deviation(lookback=${node.config.lookback}, baseline_period=${node.config.baseline_period}, threshold_ratio=${node.config.threshold_ratio})`
      };
    case "builtin.intent.rsi":
      return {
        kind: "raw_text",
        source: `rsi(period=${node.config.period}, oversold=${node.config.oversold_threshold}, overbought=${node.config.overbought_threshold})`
      };
    case "builtin.intent.macd":
      return {
        kind: "raw_text",
        source: `macd(fast=${node.config.fast_period}, slow=${node.config.slow_period}, signal=${node.config.signal_period})`
      };
    case "builtin.intent.momentum":
      return {
        kind: "raw_text",
        source: `momentum(lookback=${node.config.lookback}, threshold_ratio=${node.config.threshold_ratio})`
      };
    case "builtin.intent.zscore":
      return {
        kind: "raw_text",
        source: `zscore(window=${node.config.window}, entry_z=${node.config.entry_z})`
      };
    case "builtin.intent.spread_observer":
      return {
        kind: "raw_text",
        source:
          `spread_observe(` +
          `field_code=${node.config.field_code ?? 0}, ` +
          `align_direction_code=${node.config.align_direction_code ?? 0}, ` +
          `resample_period_ms=${node.config.resample_period_ms ?? 0}, ` +
          `window_size=${node.config.window_size ?? 1}, ` +
          `spread_output_code=${node.config.spread_output_code ?? 0}, ` +
          `max_time_diff_ms=${node.config.max_time_diff_ms ?? 5000})`
      };
    default:
      return {
        kind: "raw_text",
        source: node.module_key
      };
  }
}

function buildIntentParams(node) {
  const params = Object.fromEntries(
    Object.entries(node.config || {}).map(([key, value]) => [key, jsonValue(value)])
  );
  if (node.module_key === "builtin.intent.double_ma") {
    params.intent_variant = "long_term_buy";
  }
  if (node.module_key === "builtin.intent.ma_deviation") {
    params.intent_variant = "long_term_sell";
  }
  return params;
}

function spreadFieldName(fieldCode = 0) {
  switch (fieldCode) {
    case 1:
      return "bid_or_close";
    case 2:
      return "ask_or_close";
    case 3:
      return "close";
    case 4:
      return "open";
    case 5:
      return "high";
    case 6:
      return "low";
    case 7:
      return "volume";
    default:
      return "mid_or_close";
  }
}

function spreadAggregationName(aggCode = 0) {
  switch (aggCode) {
    case 1:
      return "mean";
    case 2:
      return "sum";
    case 3:
      return "min";
    case 4:
      return "max";
    case 5:
      return "std_dev";
    default:
      return "last";
  }
}

function spreadAlignName(directionCode = 0) {
  switch (directionCode) {
    case 1:
      return "forward";
    case 2:
      return "nearest";
    default:
      return "backward";
  }
}

function spreadOutputName(outputCode = 0) {
  switch (outputCode) {
    case 1:
      return "bps";
    case 2:
      return "absolute";
    default:
      return "ratio";
  }
}

function buildSpreadSeriesExpr(dataId, config = {}) {
  let expr = {
    kind: "data_field",
    data_id: dataId,
    field: spreadFieldName(config.field_code ?? 0)
  };

  if ((config.resample_period_ms ?? 0) > 0) {
    expr = {
      kind: "resample",
      input: expr,
      period_ms: config.resample_period_ms,
      agg: spreadAggregationName(config.resample_agg_code ?? 0)
    };
  }

  if ((config.window_size ?? 1) > 1) {
    expr = {
      kind: "window_agg",
      input: expr,
      window_size: config.window_size,
      agg: spreadAggregationName(config.window_agg_code ?? 1)
    };
  }

  return expr;
}

function buildSpreadSpec(node, inputEdges) {
  if (node.module_key !== "builtin.intent.spread_observer" || inputEdges.length !== 2) {
    return null;
  }

  const [leftEdge, rightEdge] = inputEdges;
  return {
    left: buildSpreadSeriesExpr(`data_${leftEdge.source_node_id}`, node.config),
    right: buildSpreadSeriesExpr(`data_${rightEdge.source_node_id}`, node.config),
    align: {
      direction: spreadAlignName(node.config.align_direction_code ?? 0),
      tolerance_ms: node.config.max_time_diff_ms ?? 5000
    },
    output: spreadOutputName(node.config.spread_output_code ?? 0)
  };
}

function buildCoreIr(graph, output) {
  const dataBindings = graph.nodes
    .filter((node) => node.type === "data")
    .map((node) => ({
      data_id: output.mappings.source_id_to_node_id[`data_${node.id}`]
        ? `data_${node.id}`
        : `data_${node.id}`,
      kind: node.module_key === "builtin.data.quote" ? "quote" : "kline_series",
      source_hints: {
        exchange: node.config.exchange || "",
        symbol: node.config.instrument || "",
        timeframe: node.config.timeframe || "",
        ...(node.config.ping_enabled ? { ping_enabled: "true" } : {}),
        ...(Number(node.config.request_interval_ms || 0) > 0
          ? {
              request_interval_ms: String(Number(node.config.request_interval_ms))
            }
          : {})
      }
    }));

  const indicators = graph.nodes
    .filter((node) => node.type === "intent")
    .map((node) => {
      const inputEdges = graph.edges.filter((edge) => edge.target_node_id === node.id);
      return {
        indicator_id: `intent_${node.id}`,
        kind: lowerIntentModuleToCoreKind(node.module_key),
        inputs: inputEdges.map((edge) => ({
          kind: "data_ref",
          data_id: `data_${edge.source_node_id}`
        })),
        spread_spec: buildSpreadSpec(node, inputEdges),
        params: buildIntentParams(node)
      };
    });

  const signalRules = graph.nodes
    .filter((node) => node.type === "intent")
    .map((node) => ({
      signal_id: `signal_${node.id}`,
      indicator_id: `intent_${node.id}`,
      signal_kind:
        node.module_key === "builtin.intent.ma_deviation"
          ? "short"
          : node.module_key === "builtin.intent.spread_observer"
            ? "observe"
            : "long",
      condition: lowerIntentNodeToCondition(node)
    }));

  const agentPolicies = graph.nodes
    .filter((node) => node.type === "agent")
    .map((node) => {
      const rebalanceSymbols = parseCsvStrings(node.config.rebalance_symbols);
      const rebalanceSchedule = normalizeRebalanceSchedule(node.config.rebalance_schedule);
      const rebalanceAllocationKind = normalizeRebalanceAllocationKind(
        node.config.rebalance_allocation_kind
      );
      const rebalanceRankMethod = normalizeRebalanceRankMethod(node.config.rebalance_rank_method);
      const rebalanceScoreNormalize = normalizeRebalanceScoreNormalize(
        node.config.rebalance_score_normalize
      );
      const rebalanceTargetWeights = parseCsvNumbers(node.config.rebalance_target_weights);
      const isPortfolioRebalance =
        node.module_key === "builtin.agent.weighted" && agentUsesPortfolioRebalance(node.config);

      return {
        agent_id: `agent_${node.id}`,
        name: node.name,
        kind:
          node.module_key === "builtin.agent.arbitrage"
            ? "cross_venue_arbitrage"
            : isPortfolioRebalance
              ? "portfolio_rebalance"
              : "weighted_signals",
        input_signal_ids: graph.edges
          .filter((edge) => edge.target_node_id === node.id)
          .map((edge) => `intent_${edge.source_node_id}`),
        rebalance_symbols: isPortfolioRebalance ? rebalanceSymbols : [],
        rebalance_schedule: isPortfolioRebalance ? rebalanceSchedule : null,
        rebalance_allocation_kind: isPortfolioRebalance ? rebalanceAllocationKind : null,
        rebalance_rank_method: isPortfolioRebalance ? rebalanceRankMethod : null,
        rebalance_score_normalize: isPortfolioRebalance ? rebalanceScoreNormalize : null,
        rebalance_target_weights: isPortfolioRebalance ? rebalanceTargetWeights : [],
        decision_threshold:
          node.module_key === "builtin.agent.arbitrage"
            ? null
            : jsonValue(node.config.decision_threshold ?? 0.05),
        max_quantity_ratio: jsonValue(
          node.config.max_quantity_ratio ??
            (node.module_key === "builtin.agent.arbitrage" ? 0.5 : 0.8)
        ),
        spread_trigger_bps:
          node.module_key === "builtin.agent.arbitrage"
            ? jsonValue(node.config.spread_trigger_bps ?? 50)
            : null,
        enabled: node.enabled !== false
      };
    });

  const riskPolicies = graph.nodes
    .filter((node) => node.type === "risk")
    .map((node) => ({
      policy_id: `risk_${node.id}`,
      name: node.name,
      observed_agent_ids: graph.edges
        .filter((edge) => edge.target_node_id === node.id)
        .map((edge) => `agent_${edge.source_node_id}`),
      max_position_ratio: jsonValue(
        node.config.max_position ?? node.config.max_position_ratio ?? 0.2
      ),
      max_single_weight: jsonValue(node.config.max_single_weight ?? null),
      max_concentration_ratio: jsonValue(
        node.config.max_concentration ?? node.config.max_concentration_ratio ?? null
      ),
      max_symbol_net_exposure_ratio: jsonValue(
        node.config.max_symbol_net_exposure ??
          node.config.max_symbol_net_exposure_ratio ??
          null
      ),
      max_portfolio_net_exposure_ratio: jsonValue(
        node.config.max_portfolio_net_exposure ??
          node.config.max_portfolio_net_exposure_ratio ??
          null
      ),
      max_turnover: jsonValue(node.config.max_turnover ?? null),
      min_trade_weight: jsonValue(node.config.min_trade_weight ?? null),
      max_new_positions_per_rebalance: jsonValue(
        node.config.max_new_positions_per_rebalance ?? null
      ),
      max_total_leverage: jsonValue(node.config.max_total_leverage ?? 3),
      max_exchange_leverage: jsonValue(node.config.max_exchange_leverage ?? 3),
      min_action_interval_ms: jsonValue(node.config.min_action_interval_ms ?? 100),
      enabled: node.enabled !== false
    }));

  const runtimeNode = graph.nodes.find((node) => node.type === "runtime");
  const executionNode = graph.nodes.find((node) => node.type === "execution");

  return {
    ir_version: "quantpilot/core-ir/v1",
    metadata: {
      strategy_id: graph.metadata.graph_id,
      name: graph.metadata.name,
      source_kind: "frontend_graph"
    },
    data_bindings: dataBindings,
    indicators,
    signal_rules: signalRules,
    agent_policies: agentPolicies,
    risk_policies: riskPolicies,
    execution: {
      execution_id: executionNode ? `execution_${executionNode.id}` : "execution.paper",
      venue_kind: runtimeNode?.config?.mode || "paper",
      sizing_kind: "equity_notional_ratio",
      slippage_bps: jsonValue(executionNode?.config?.slippage_bps ?? 5),
      taker_fee_bps: 0,
      total_cost_buffer_bps: 0,
      time_in_force: "gtc",
      params: Object.fromEntries(
        Object.entries(executionNode?.config || {}).map(([key, value]) => [key, jsonValue(value)])
      )
    }
  };
}

function buildTopology(graph) {
  const indegree = new Map(graph.nodes.map((node) => [node.id, 0]));
  const outgoing = new Map(graph.nodes.map((node) => [node.id, []]));

  graph.edges.forEach((edge) => {
    if (!indegree.has(edge.target_node_id) || !outgoing.has(edge.source_node_id)) return;
    indegree.set(edge.target_node_id, indegree.get(edge.target_node_id) + 1);
    outgoing.get(edge.source_node_id).push(edge.target_node_id);
  });

  const queue = graph.nodes
    .filter((node) => indegree.get(node.id) === 0)
    .map((node) => node.id);
  const order = [];

  while (queue.length > 0) {
    const nodeId = queue.shift();
    order.push(nodeId);
    for (const nextId of outgoing.get(nodeId) || []) {
      indegree.set(nextId, indegree.get(nextId) - 1);
      if (indegree.get(nextId) === 0) queue.push(nextId);
    }
  }

  return {
    topologyOrder: order,
    hasCycle: order.length !== graph.nodes.length
  };
}

export function compileGraph(graph, registry = null) {
  const capabilities = normalizeCapabilities(registry?.capabilities || DEFAULT_CAPABILITIES);
  const supportedRuntimeModes = capabilitySet(
    capabilities.runtime?.supported_modes,
    DEFAULT_CAPABILITIES.runtime.supported_modes
  );
  const supportedExecutionModules = capabilitySet(
    capabilities.runtime?.supported_execution_modules,
    DEFAULT_CAPABILITIES.runtime.supported_execution_modules
  );
  const supportedSymbols = capabilitySet(
    capabilities.market_data?.supported_symbols,
    DEFAULT_CAPABILITIES.market_data.supported_symbols
  );
  const supportedExchanges = capabilitySet(
    capabilities.market_data?.supported_exchanges,
    DEFAULT_CAPABILITIES.market_data.supported_exchanges
  );
  const runtimeModeSupport = supportMap(capabilities.runtime?.mode_support);
  const executionModuleSupport = supportMap(capabilities.runtime?.execution_module_support);
  const exchangeSupport = supportMap(capabilities.market_data?.exchange_support);
  const symbolSupport = supportMap(capabilities.market_data?.symbol_support);
  const frontendModuleSupport = supportMap(capabilities.frontend?.module_support, "module_key");

  const compileId = `compile_${Date.now()}`;
  const errors = [];
  const warnings = [];
  const output = {
    metadata: {
      graph_id: graph.metadata.graph_id,
      compile_id: compileId,
      name: graph.metadata.name,
      version: graph.metadata.version,
      mode: "paper"
    },
    data_sources: [],
    intent_generators: [],
    agents: [],
    risk_controls: [],
    executions: [],
    runtime_control: null,
    mappings: {
      source_id_to_node_id: {}
    }
  };

  const idMap = {};

  graph.nodes.forEach((node) => {
    const compiledId = `${node.type}_${node.id}`;
    idMap[node.id] = compiledId;
    output.mappings.source_id_to_node_id[compiledId] = node.id;
  });

  const runtimeNodes = graph.nodes.filter((node) => node.type === "runtime");
  if (runtimeNodes.length !== 1) {
    errors.push("策略图必须且只能包含一个运行控制节点。");
  }

  graph.nodes.forEach((node) => {
    const moduleDef = registry?.getByKey(node.module_key);
    const base = {
      id: idMap[node.id],
      module_key: node.module_key,
      name: node.name,
      config: node.config
    };

    if (moduleDef?.availability?.status === "unsupported") {
      const moduleSupportEntry = frontendModuleSupport.get(node.module_key);
      errors.push(
        `节点 ${node.name} 使用了当前未开放的模块 ${node.module_key}：${capabilityReason(moduleSupportEntry, moduleDef.availability.reason)}`
      );
    }

    if (node.type === "data") {
      const exchangeEntry = exchangeSupport.get(node.config.exchange);
      if (!capabilityEntryStatus(exchangeEntry, supportedExchanges, node.config.exchange)) {
        errors.push(
          `数据节点 ${node.name} 使用了未支持的交易所 ${node.config.exchange || "-"}。 ${capabilityReason(exchangeEntry, "")}`.trim()
        );
      }

      const symbolEntry = symbolSupport.get(node.config.instrument);
      if (!capabilityEntryStatus(symbolEntry, supportedSymbols, node.config.instrument)) {
        errors.push(
          `数据节点 ${node.name} 使用了未支持的交易对 ${node.config.instrument || "-"}。 ${capabilityReason(symbolEntry, "")}`.trim()
        );
      }

      output.data_sources.push(base);
    }

    if (node.type === "intent") {
      const inputRefs = graph.edges
        .filter((edge) => edge.target_node_id === node.id)
        .map((edge) => ({
          source_id: idMap[edge.source_node_id],
          source_port: edge.source_port,
          target_port: edge.target_port
        }));
      if (inputRefs.length === 0) {
        errors.push(`意图节点 ${node.name} 缺少数据输入。`);
      }
      output.intent_generators.push({ ...base, input_refs: inputRefs });
    }

    if (node.type === "agent") {
      const intentRefs = graph.edges
        .filter((edge) => edge.target_node_id === node.id)
        .map((edge) => idMap[edge.source_node_id]);
      if (intentRefs.length === 0) {
        errors.push(`代理节点 ${node.name} 缺少意图输入。`);
      }
      const isPortfolioRebalance =
        node.module_key === "builtin.agent.weighted" && agentUsesPortfolioRebalance(node.config);
      if (isPortfolioRebalance) {
        const rebalanceSymbols = parseCsvStrings(node.config.rebalance_symbols);
        const unsupportedSymbols = rebalanceSymbols.filter(
          (symbol) => !supportedSymbols.has(symbol)
        );
        if (unsupportedSymbols.length > 0) {
          errors.push(
            `代理节点 ${node.name} 使用了未支持的再平衡交易对：${unsupportedSymbols.join(", ")}。`
          );
        }

        if (normalizeRebalanceSchedule(node.config.rebalance_schedule) === "__invalid__") {
          errors.push(`代理节点 ${node.name} 的 rebalance cadence 值不合法。`);
        }
        const allocationKind = normalizeRebalanceAllocationKind(
          node.config.rebalance_allocation_kind
        );
        if (allocationKind === "__invalid__") {
          errors.push(`代理节点 ${node.name} 的 allocation rule 值不合法。`);
        }
        if (normalizeRebalanceRankMethod(node.config.rebalance_rank_method) === "__invalid__") {
          errors.push(`代理节点 ${node.name} 的 rank method 值不合法。`);
        }
        if (
          normalizeRebalanceScoreNormalize(node.config.rebalance_score_normalize) === "__invalid__"
        ) {
          errors.push(`代理节点 ${node.name} 的 score normalize 值不合法。`);
        }

        const weightsRaw = parseCsvStrings(node.config.rebalance_target_weights);
        const weights = parseCsvNumbers(node.config.rebalance_target_weights);
        if (weightsRaw.length !== weights.length) {
          errors.push(
            `代理节点 ${node.name} 的 target weights 必须是由逗号分隔的数字。`
          );
        }
        if (
          allocationKind === "fixed_weights" &&
          rebalanceSymbols.length > 0 &&
          weights.length !== rebalanceSymbols.length
        ) {
          errors.push(
            `代理节点 ${node.name} 的 fixed_weights 数量必须与 rebalance symbols 一致。`
          );
        }
      }
      output.agents.push({ ...base, intent_refs: intentRefs });
    }

    if (node.type === "risk") {
      const agentRefs = graph.edges
        .filter((edge) => edge.target_node_id === node.id)
        .map((edge) => idMap[edge.source_node_id]);
      if (agentRefs.length === 0) {
        errors.push(`风控节点 ${node.name} 缺少代理输入。`);
      }
      output.risk_controls.push({ ...base, agent_refs: agentRefs });
    }

    if (node.type === "execution") {
      const executionEntry = executionModuleSupport.get(node.module_key);
      if (!capabilityEntryStatus(executionEntry, supportedExecutionModules, node.module_key)) {
        errors.push(
          `执行节点 ${node.name} 使用了当前后端未支持的模块 ${node.module_key}。 ${capabilityReason(executionEntry, "")}`.trim()
        );
      }
      const riskEdge = graph.edges.find((edge) => edge.target_node_id === node.id);
      if (!riskEdge) {
        errors.push(`执行节点 ${node.name} 缺少风控输入。`);
      }
      output.executions.push({
        ...base,
        risk_ref: riskEdge ? idMap[riskEdge.source_node_id] : null
      });
    }

    if (node.type === "runtime") {
      output.runtime_control = base;
      output.metadata.mode = node.config.mode || "paper";
      const runtimeModeEntry = runtimeModeSupport.get(output.metadata.mode);
      if (!capabilityEntryStatus(runtimeModeEntry, supportedRuntimeModes, output.metadata.mode)) {
        errors.push(
          `当前仅支持这些运行模式：${[...supportedRuntimeModes].join(", ")}。 ${capabilityReason(runtimeModeEntry, "")}`.trim()
        );
      }
    }
  });

  if (output.executions.length !== 1) {
    errors.push("当前 beta 仅支持一个执行节点。");
  }

  const topology = buildTopology(graph);
  if (topology.hasCycle) {
    errors.push("策略图存在循环依赖，无法编译。");
  }

  if ((graph.validation_state?.issue_counts?.error || 0) > 0) {
    errors.push("策略图校验未通过，无法编译。");
  }

  const graphWithArtifacts = attachQuantScriptArtifacts(graph);
  const coreIr = buildCoreIr(graph, output);
  graphWithArtifacts.metadata.artifacts = {
    ...(graphWithArtifacts.metadata.artifacts || {}),
    core_ir: coreIr
  };
  const quantscript =
    graphWithArtifacts.metadata.artifacts.quantscript.graph_source ||
    generateGraphQuantScript(graph);
  const compilable = errors.length === 0;
  const diagnostics = buildLocalCompileDiagnostics(errors, warnings);

  return {
    compile_id: compileId,
    runtime_config: output,
    core_ir: coreIr,
    quantscript,
    graph: graphWithArtifacts,
    compile_summary: {
      compilable,
      last_compile_id: compileId,
      last_compile_at: Date.now(),
      topology_order: topology.topologyOrder,
      outputs: {
        data_sources: output.data_sources.length,
        intent_generators: output.intent_generators.length,
        agents: output.agents.length,
        risk_controls: output.risk_controls.length,
        executions: output.executions.length
      },
      diagnostics,
      warnings,
      errors
    }
  };
}
