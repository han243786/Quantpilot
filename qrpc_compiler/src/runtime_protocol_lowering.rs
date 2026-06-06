use anyhow::Result;
use qrpc_core::{
    AgentConfig, DataKind, IntentConfig, IntentKind, RebalanceSchedule as RuntimeRebalanceSchedule,
    RiskConfig, RuntimeProtocolCoreConfig,
};
use qrpc_core_ir::{
    indicator_threshold_compare_expr, moving_average_compare_expr, AgentPolicy, AgentPolicyKind,
    ComparisonOp, CoreIndicatorKind, CoreMetadata, CoreStrategyIr, CoreTimeInForce, DataBinding,
    DataBindingKind, ExecutionRule, ExecutionSizingKind, IndicatorNode,
    RebalanceSchedule as CoreRebalanceSchedule, RiskPolicy, ScalarExpr, SeriesExpr, SignalKind,
    SignalRule, SpreadValueKind,
};
use std::collections::BTreeMap;

use super::{
    decode_align_direction, decode_series_field, decode_spread_output, lower_runtime_spread_spec,
    runtime_intent_is_spread, validate_runtime_protocol_config,
};

pub(super) fn lower_runtime_protocol_to_core_ir_with_metadata(
    config: &RuntimeProtocolCoreConfig,
    metadata: CoreMetadata,
) -> Result<CoreStrategyIr> {
    validate_runtime_protocol_config(config)?;

    let mut core_ir = CoreStrategyIr::new(
        metadata,
        ExecutionRule {
            execution_id: "execution.paper".to_string(),
            venue_kind: "paper".to_string(),
            sizing_kind: ExecutionSizingKind::EquityNotionalRatio,
            slippage_bps: config.default_slippage_bps,
            taker_fee_bps: config.taker_fee_bps,
            total_cost_buffer_bps: config.total_cost_buffer_bps,
            time_in_force: CoreTimeInForce::Gtc,
            params: BTreeMap::from([
                (
                    "initial_cash_balance".to_string(),
                    serde_json::Value::from(config.initial_cash_balance),
                ),
                (
                    "taker_fee_bps".to_string(),
                    serde_json::Value::from(config.taker_fee_bps),
                ),
                (
                    "default_slippage_bps".to_string(),
                    serde_json::Value::from(config.default_slippage_bps),
                ),
                (
                    "total_cost_buffer_bps".to_string(),
                    serde_json::Value::from(config.total_cost_buffer_bps),
                ),
            ]),
        },
    );

    core_ir.data_bindings = config
        .data_sources
        .iter()
        .map(|source| {
            let mut source_hints = BTreeMap::from([
                (
                    "exchange".to_string(),
                    format!("{:?}", source.exchange).to_lowercase(),
                ),
                (
                    "symbol".to_string(),
                    format!("{:?}", source.symbol).to_uppercase(),
                ),
                (
                    "timeframe".to_string(),
                    source.interval.clone().unwrap_or_else(|| "1d".to_string()),
                ),
            ]);
            if source.ping_enabled {
                source_hints.insert("ping_enabled".to_string(), "true".to_string());
            }
            if let Some(request_interval_ms) = source.request_interval_ms {
                source_hints.insert(
                    "request_interval_ms".to_string(),
                    request_interval_ms.to_string(),
                );
            }
            DataBinding {
                data_id: source.data_id.clone(),
                kind: match source.kind {
                    DataKind::KlineSeries => DataBindingKind::KlineSeries,
                    DataKind::Quote => DataBindingKind::Quote,
                },
                source_hints,
            }
        })
        .collect();

    core_ir.indicators = config
        .intents
        .iter()
        .filter(|intent| intent.enabled)
        .map(lower_runtime_intent_to_indicator)
        .collect::<Result<Vec<_>>>()?;

    core_ir.signal_rules = config
        .intents
        .iter()
        .filter(|intent| intent.enabled)
        .map(lower_runtime_intent_to_signal_rule)
        .collect::<Vec<_>>();
    core_ir.agent_policies = config
        .agents
        .iter()
        .filter(|agent| agent.enabled)
        .map(|agent| lower_runtime_agent_to_policy(agent, &config.intents))
        .collect();
    core_ir.risk_policies = config
        .risks
        .iter()
        .filter(|risk| risk.enabled)
        .map(lower_runtime_risk_to_policy)
        .collect();

    // v1.0.1: DAG 环检测 — 编译前校验策略图无环
    core_ir
        .validate_dag()
        .map_err(|errs| anyhow::anyhow!("策略图 DAG 校验失败: {:?}", errs))?;

    Ok(core_ir)
}

fn lower_runtime_intent_to_indicator(intent: &IntentConfig) -> Result<IndicatorNode> {
    let mut params = intent
        .params
        .iter()
        .map(|(key, value)| (key.clone(), serde_json::Value::from(*value)))
        .collect::<BTreeMap<_, _>>();
    if matches!(intent.kind, IntentKind::LongTermBuy) {
        params.insert(
            "intent_variant".to_string(),
            serde_json::Value::String("long_term_buy".to_string()),
        );
    }
    if matches!(intent.kind, IntentKind::LongTermSell) {
        params.insert(
            "intent_variant".to_string(),
            serde_json::Value::String("long_term_sell".to_string()),
        );
    }
    if matches!(intent.kind, IntentKind::SmaCrossover) {
        params.insert(
            "intent_variant".to_string(),
            serde_json::Value::String("sma_crossover".to_string()),
        );
    }
    Ok(IndicatorNode {
        indicator_id: intent.intent_id.clone(),
        kind: if runtime_intent_is_spread(intent) {
            CoreIndicatorKind::Spread
        } else {
            lower_runtime_intent_kind(&intent.kind)?
        },
        inputs: intent
            .input_data_ids
            .iter()
            .map(|data_id| SeriesExpr::DataRef {
                data_id: data_id.clone(),
            })
            .collect(),
        spread_spec: lower_runtime_spread_spec(intent),
        custom_expr: None,
        params,
    })
}

fn lower_runtime_intent_to_signal_rule(intent: &IntentConfig) -> SignalRule {
    SignalRule {
        signal_id: format!("{}_signal", intent.intent_id),
        indicator_id: intent.intent_id.clone(),
        signal_kind: match intent.kind {
            IntentKind::LongTermSell => SignalKind::Short,
            IntentKind::QuoteObserve => SignalKind::Observe,
            _ => SignalKind::Long,
        },
        condition: lower_runtime_intent_condition(intent).unwrap_or_else(|| ScalarExpr::RawText {
            source: describe_runtime_intent_condition(intent),
        }),
    }
}

fn lower_runtime_intent_condition(intent: &IntentConfig) -> Option<ScalarExpr> {
    match intent.kind {
        IntentKind::LongTermBuy | IntentKind::SmaCrossover => {
            let data_id = intent.input_data_ids.first()?;
            let fast_period = intent.params.get("fast_period")?.round() as usize;
            let slow_period = intent.params.get("slow_period")?.round() as usize;
            let entry_ratio = intent
                .params
                .get("entry_ratio")
                .copied()
                .unwrap_or_default();
            if (entry_ratio - 1.0).abs() > f64::EPSILON {
                return None;
            }
            let op = decode_runtime_comparison_op(
                intent.params.get("comparison_op_code").copied(),
                ComparisonOp::Gt,
            )?;
            if !matches!(op, ComparisonOp::Gt | ComparisonOp::Gte) {
                return None;
            }
            moving_average_compare_expr(data_id.clone(), fast_period, op, slow_period)
        }
        IntentKind::LongTermSell => {
            let data_id = intent.input_data_ids.first()?;
            let fast_period = intent.params.get("lookback")?.round() as usize;
            let slow_period = intent.params.get("baseline_period")?.round() as usize;
            let threshold_ratio = intent
                .params
                .get("threshold_ratio")
                .copied()
                .unwrap_or_default();
            if (threshold_ratio - 1.0).abs() > f64::EPSILON {
                return None;
            }
            let op = decode_runtime_comparison_op(
                intent.params.get("comparison_op_code").copied(),
                ComparisonOp::Lt,
            )?;
            if !matches!(op, ComparisonOp::Lt | ComparisonOp::Lte) {
                return None;
            }
            moving_average_compare_expr(data_id.clone(), fast_period, op, slow_period)
        }
        IntentKind::Rsi => {
            let indicator_id = intent.intent_id.clone();
            let oversold = intent.params.get("oversold_threshold").copied()?;
            let overbought = intent.params.get("overbought_threshold").copied()?;
            let shape = decode_runtime_comparison_shape(
                intent.params.get("comparison_shape_code").copied(),
            )?;
            let (default_op, threshold) = if (overbought - 70.0).abs() <= f64::EPSILON
                && (oversold - 30.0).abs() > f64::EPSILON
                && matches!(shape, RuntimeComparisonShape::Buy)
            {
                (ComparisonOp::Lt, oversold)
            } else if (oversold - 30.0).abs() <= f64::EPSILON
                && (overbought - 70.0).abs() > f64::EPSILON
                && matches!(shape, RuntimeComparisonShape::Sell)
            {
                (ComparisonOp::Gt, overbought)
            } else {
                return None;
            };
            let op = decode_runtime_comparison_op(
                intent.params.get("comparison_op_code").copied(),
                default_op,
            )?;
            match op {
                ComparisonOp::Lt | ComparisonOp::Lte | ComparisonOp::Gt | ComparisonOp::Gte => {
                    indicator_threshold_compare_expr(indicator_id, op, threshold)
                }
                ComparisonOp::Eq => None,
            }
        }
        IntentKind::Momentum => {
            let indicator_id = intent.intent_id.clone();
            let shape = decode_runtime_comparison_shape(
                intent.params.get("comparison_shape_code").copied(),
            )?;
            let threshold = intent.params.get("comparison_threshold").copied()?;
            let default_op = match shape {
                RuntimeComparisonShape::Buy => ComparisonOp::Gt,
                RuntimeComparisonShape::Sell => ComparisonOp::Lt,
            };
            let op = decode_runtime_comparison_op(
                intent.params.get("comparison_op_code").copied(),
                default_op,
            )?;
            match (shape, op) {
                (RuntimeComparisonShape::Buy, ComparisonOp::Gt | ComparisonOp::Gte)
                | (RuntimeComparisonShape::Sell, ComparisonOp::Lt | ComparisonOp::Lte) => {
                    indicator_threshold_compare_expr(indicator_id, op, threshold)
                }
                _ => None,
            }
        }
        IntentKind::ZScore => {
            let indicator_id = intent.intent_id.clone();
            let shape = decode_runtime_comparison_shape(
                intent.params.get("comparison_shape_code").copied(),
            )?;
            let threshold = intent.params.get("comparison_threshold").copied()?;
            let default_op = match shape {
                RuntimeComparisonShape::Buy => ComparisonOp::Lt,
                RuntimeComparisonShape::Sell => ComparisonOp::Gt,
            };
            let op = decode_runtime_comparison_op(
                intent.params.get("comparison_op_code").copied(),
                default_op,
            )?;
            match (shape, op) {
                (RuntimeComparisonShape::Buy, ComparisonOp::Lt | ComparisonOp::Lte)
                | (RuntimeComparisonShape::Sell, ComparisonOp::Gt | ComparisonOp::Gte) => {
                    indicator_threshold_compare_expr(indicator_id, op, threshold)
                }
                _ => None,
            }
        }
        IntentKind::QuoteObserve => lower_runtime_spread_threshold_condition(intent),
        _ => None,
    }
}

fn describe_runtime_intent_condition(intent: &IntentConfig) -> String {
    match intent.kind {
        IntentKind::LongTermBuy | IntentKind::SmaCrossover => format!(
            "ma_cross(fast={}, slow={}, entry_ratio={})",
            intent
                .params
                .get("fast_period")
                .copied()
                .unwrap_or_default(),
            intent
                .params
                .get("slow_period")
                .copied()
                .unwrap_or_default(),
            intent
                .params
                .get("entry_ratio")
                .copied()
                .unwrap_or_default()
        ),
        IntentKind::LongTermSell => format!(
            "ma_deviation(lookback={}, baseline_period={}, threshold_ratio={})",
            intent.params.get("lookback").copied().unwrap_or_default(),
            intent
                .params
                .get("baseline_period")
                .copied()
                .unwrap_or_default(),
            intent
                .params
                .get("threshold_ratio")
                .copied()
                .unwrap_or_default()
        ),
        IntentKind::Rsi => format!(
            "rsi(period={}, oversold={}, overbought={})",
            intent.params.get("period").copied().unwrap_or_default(),
            intent
                .params
                .get("oversold_threshold")
                .copied()
                .unwrap_or_default(),
            intent
                .params
                .get("overbought_threshold")
                .copied()
                .unwrap_or_default()
        ),
        IntentKind::Macd => format!(
            "macd(fast={}, slow={}, signal={})",
            intent
                .params
                .get("fast_period")
                .copied()
                .unwrap_or_default(),
            intent
                .params
                .get("slow_period")
                .copied()
                .unwrap_or_default(),
            intent
                .params
                .get("signal_period")
                .copied()
                .unwrap_or_default()
        ),
        IntentKind::Momentum => format!(
            "momentum(lookback={}, threshold_ratio={})",
            intent.params.get("lookback").copied().unwrap_or_default(),
            intent
                .params
                .get("threshold_ratio")
                .copied()
                .unwrap_or_default()
        ),
        IntentKind::ZScore => format!(
            "zscore(window={}, entry_z={})",
            intent.params.get("window").copied().unwrap_or_default(),
            intent.params.get("entry_z").copied().unwrap_or_default()
        ),
        IntentKind::QuoteObserve => {
            if runtime_intent_is_spread(intent) {
                let field = decode_series_field(
                    intent.params.get("field_code").copied().unwrap_or_default() as u64,
                );
                let align = decode_align_direction(
                    intent
                        .params
                        .get("align_direction_code")
                        .copied()
                        .unwrap_or_default() as u64,
                );
                let output = decode_spread_output(
                    intent
                        .params
                        .get("spread_output_code")
                        .copied()
                        .unwrap_or_default() as u64,
                );
                format!(
                    "spread_observe(inputs={}, field={:?}, align={:?}, resample_ms={}, window={}, output={:?}, max_time_diff_ms={})",
                    intent.input_data_ids.len(),
                    field,
                    align,
                    intent
                        .params
                        .get("resample_period_ms")
                        .copied()
                        .unwrap_or_default()
                        .round() as u64,
                    intent
                        .params
                        .get("window_size")
                        .copied()
                        .unwrap_or_default()
                        .round() as usize,
                    output,
                    intent.params.get("max_time_diff_ms").copied().unwrap_or(5_000.0)
                )
            } else {
                "quote_observe(mid_price_delta)".to_string()
            }
        }
    }
}

fn lower_runtime_spread_threshold_condition(intent: &IntentConfig) -> Option<ScalarExpr> {
    let spread = lower_runtime_spread_spec(intent)?;
    if !matches!(spread.output, SpreadValueKind::Bps) {
        return None;
    }
    if spread.align.tolerance_ms == 0 {
        return None;
    }
    let shape =
        decode_runtime_comparison_shape(intent.params.get("comparison_shape_code").copied())?;
    if !matches!(shape, RuntimeComparisonShape::Buy) {
        return None;
    }
    let threshold = intent.params.get("comparison_threshold").copied()?;
    let op = decode_runtime_comparison_op(
        intent.params.get("comparison_op_code").copied(),
        ComparisonOp::Gt,
    )?;
    match op {
        ComparisonOp::Gt | ComparisonOp::Gte => {
            indicator_threshold_compare_expr(intent.intent_id.clone(), op, threshold)
        }
        ComparisonOp::Lt | ComparisonOp::Lte | ComparisonOp::Eq => None,
    }
}

fn decode_runtime_comparison_op(code: Option<f64>, default: ComparisonOp) -> Option<ComparisonOp> {
    match code.unwrap_or(comparison_op_code(default)).round() as i64 {
        0 => Some(ComparisonOp::Lt),
        1 => Some(ComparisonOp::Lte),
        2 => Some(ComparisonOp::Gt),
        3 => Some(ComparisonOp::Gte),
        4 => Some(ComparisonOp::Eq),
        _ => None,
    }
}

fn comparison_op_code(op: ComparisonOp) -> f64 {
    match op {
        ComparisonOp::Lt => 0.0,
        ComparisonOp::Lte => 1.0,
        ComparisonOp::Gt => 2.0,
        ComparisonOp::Gte => 3.0,
        ComparisonOp::Eq => 4.0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeComparisonShape {
    Buy,
    Sell,
}

fn decode_runtime_comparison_shape(code: Option<f64>) -> Option<RuntimeComparisonShape> {
    match code?.round() as i64 {
        1 => Some(RuntimeComparisonShape::Buy),
        2 => Some(RuntimeComparisonShape::Sell),
        _ => None,
    }
}

fn lower_runtime_risk_to_policy(risk: &RiskConfig) -> RiskPolicy {
    RiskPolicy {
        policy_id: risk.risk_id.clone(),
        name: risk.name.clone(),
        observed_agent_ids: risk.observed_agent_ids.clone(),
        max_position_ratio: risk.max_position_ratio,
        max_single_weight: risk.max_single_weight,
        max_concentration_ratio: risk.max_concentration_ratio,
        max_symbol_net_exposure_ratio: risk.max_symbol_net_exposure_ratio,
        max_portfolio_net_exposure_ratio: risk.max_portfolio_net_exposure_ratio,
        max_turnover: risk.max_turnover,
        min_trade_weight: risk.min_trade_weight,
        max_new_positions_per_rebalance: risk.max_new_positions_per_rebalance,
        max_total_leverage: risk.max_total_leverage,
        max_exchange_leverage: risk.max_exchange_leverage,
        min_action_interval_ms: risk.min_action_interval_ms,
        enabled: risk.enabled,
        max_cross_symbol_leverage: None,
    }
}

fn lower_runtime_rebalance_schedule_to_core_ir(
    schedule: RuntimeRebalanceSchedule,
) -> CoreRebalanceSchedule {
    match schedule {
        RuntimeRebalanceSchedule::EverySlow => CoreRebalanceSchedule::EverySlow,
        RuntimeRebalanceSchedule::Every1d => CoreRebalanceSchedule::Every1d,
        RuntimeRebalanceSchedule::Weekly => CoreRebalanceSchedule::Weekly,
    }
}

fn lower_runtime_agent_to_policy(agent: &AgentConfig, intents: &[IntentConfig]) -> AgentPolicy {
    let referenced_intents = intents
        .iter()
        .filter(|intent| agent.input_intent_ids.contains(&intent.intent_id))
        .collect::<Vec<_>>();
    let all_observe = !referenced_intents.is_empty()
        && referenced_intents
            .iter()
            .all(|intent| matches!(intent.kind, IntentKind::QuoteObserve));
    let portfolio_rebalance = agent
        .params
        .get("portfolio_rebalance")
        .copied()
        .unwrap_or_default()
        > 0.5;

    AgentPolicy {
        agent_id: agent.agent_id.clone(),
        name: agent.name.clone(),
        kind: if all_observe {
            AgentPolicyKind::CrossVenueArbitrage
        } else if portfolio_rebalance {
            AgentPolicyKind::PortfolioRebalance
        } else {
            AgentPolicyKind::WeightedSignals
        },
        input_signal_ids: agent.input_intent_ids.clone(),
        rebalance_symbols: agent
            .rebalance_symbols
            .iter()
            .map(|symbol| symbol.as_str().to_string())
            .collect(),
        rebalance_schedule: agent
            .rebalance_schedule
            .clone()
            .map(lower_runtime_rebalance_schedule_to_core_ir),
        rebalance_allocation_kind: agent.rebalance_allocation_kind.clone(),
        rebalance_rank_method: agent.rebalance_rank_method.clone(),
        rebalance_score_normalize: agent.rebalance_score_normalize.clone(),
        rebalance_target_weights: agent.rebalance_target_weights.clone(),
        decision_threshold: (!all_observe).then(|| {
            agent
                .params
                .get("decision_threshold")
                .copied()
                .unwrap_or(0.05)
        }),
        max_quantity_ratio: agent.params.get("max_quantity_ratio").copied().unwrap_or(
            if all_observe {
                0.5
            } else if portfolio_rebalance {
                1.0
            } else {
                0.8
            },
        ),
        spread_trigger_bps: all_observe.then(|| {
            agent
                .params
                .get("spread_trigger_bps")
                .copied()
                .unwrap_or(50.0)
        }),
        enabled: agent.enabled,
    }
}

fn lower_runtime_intent_kind(kind: &IntentKind) -> Result<CoreIndicatorKind> {
    match kind {
        IntentKind::LongTermBuy | IntentKind::LongTermSell | IntentKind::SmaCrossover => {
            Ok(CoreIndicatorKind::MaCross)
        }
        IntentKind::Rsi => Ok(CoreIndicatorKind::Rsi),
        IntentKind::Macd => Ok(CoreIndicatorKind::Macd),
        IntentKind::Momentum => Ok(CoreIndicatorKind::Momentum),
        IntentKind::ZScore => Ok(CoreIndicatorKind::ZScore),
        IntentKind::QuoteObserve => Ok(CoreIndicatorKind::QuoteObserve),
    }
}
