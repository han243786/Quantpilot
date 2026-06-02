import {
  agentUsesPortfolioRebalance,
  jsonValue,
  normalizeRebalanceAllocationKind,
  normalizeRebalanceRankMethod,
  normalizeRebalanceSchedule,
  normalizeRebalanceScoreNormalize,
  parseCsvNumbers,
  parseCsvStrings
} from "./compileGraphSupport";

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

export function buildCoreIr(graph, output) {
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
