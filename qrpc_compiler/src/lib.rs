mod runtime_protocol_validation;

use anyhow::{anyhow, bail, Result};
use qrpc_core::{
    AgentConfig, CompiledRuntimeProtocol, DataKind, IntentConfig, IntentKind,
    RebalanceSchedule as RuntimeRebalanceSchedule, RiskConfig, RuntimeProtocolCoreConfig,
    StrategyIr, GLOBAL_RISK_PROFILE_DEFAULT_MAX_EXCHANGE_LEVERAGE,
    GLOBAL_RISK_PROFILE_DEFAULT_MAX_TOTAL_LEVERAGE,
    GLOBAL_RISK_PROFILE_DEFAULT_MIN_ACTION_INTERVAL_MS, PAPER_EXECUTION_PROFILE_DEFAULT_FEE_BPS,
};
use qrpc_core_ir::{
    indicator_threshold_compare_expr, moving_average_compare_expr, AgentPolicy, AgentPolicyKind,
    AlignAsofSpec, AlignDirection, ArithmeticOp, ComparisonOp, CoreIndicatorKind, CoreMetadata,
    CoreSourceKind, CoreStrategyIr, CoreTimeInForce, CustomExprSpec, CustomValueExpr, DataBinding,
    DataBindingKind, ExecutionRule, ExecutionSizingKind, IndicatorNode,
    RebalanceSchedule as CoreRebalanceSchedule, RiskPolicy, ScalarExpr, SeriesAggregation,
    SeriesExpr, SeriesField, SignalKind, SignalRule, SpreadSpec, SpreadValueKind,
    CUSTOM_EXPR_V1_VERSION,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub fn validate_runtime_protocol_config(config: &RuntimeProtocolCoreConfig) -> Result<()> {
    runtime_protocol_validation::validate_runtime_protocol_config(config)
}

pub fn compile_runtime_protocol_config(
    config: &RuntimeProtocolCoreConfig,
) -> Result<CompiledRuntimeProtocol> {
    compile_runtime_protocol_config_with_metadata(
        config,
        CoreMetadata {
            strategy_id: "runtime_protocol".to_string(),
            name: "Runtime Protocol Lowered Strategy".to_string(),
            source_kind: CoreSourceKind::RuntimeProtocol,
        },
    )
}

pub fn compile_runtime_protocol_config_with_metadata(
    config: &RuntimeProtocolCoreConfig,
    metadata: CoreMetadata,
) -> Result<CompiledRuntimeProtocol> {
    validate_runtime_protocol_config(config)?;
    let core_ir = lower_runtime_protocol_to_core_ir_with_metadata(config, metadata)?;

    let canonical = serde_json::to_vec(config)?;
    let mut hasher = Sha256::new();
    hasher.update(&canonical);
    let hash = format!("runtime-spec-{:x}", hasher.finalize());

    Ok(CompiledRuntimeProtocol {
        protocol_name: "quantpilot/minimal-sim/v1".to_string(),
        config_hash: hash,
        config: config.clone(),
        core_ir,
    })
}

pub fn lower_runtime_protocol_to_core_ir(
    config: &RuntimeProtocolCoreConfig,
) -> Result<CoreStrategyIr> {
    lower_runtime_protocol_to_core_ir_with_metadata(
        config,
        CoreMetadata {
            strategy_id: "runtime_protocol".to_string(),
            name: "Runtime Protocol Lowered Strategy".to_string(),
            source_kind: CoreSourceKind::RuntimeProtocol,
        },
    )
}

pub fn lower_runtime_protocol_to_core_ir_with_metadata(
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

pub fn lower_strategy_ir_to_core_ir(strategy_ir: &StrategyIr) -> Result<CoreStrategyIr> {
    strategy_ir
        .validate()
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;
    let data_requirement_kinds = strategy_ir
        .data_requirements
        .iter()
        .map(|requirement| (requirement.data_id.clone(), requirement.data_type.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut core_ir = CoreStrategyIr::new(
        CoreMetadata {
            strategy_id: strategy_ir.metadata.strategy_id.clone(),
            name: strategy_ir.metadata.name.clone(),
            source_kind: CoreSourceKind::StrategyIr,
        },
        ExecutionRule {
            execution_id: "strategy_ir.execution".to_string(),
            venue_kind: match &strategy_ir.execution.venue_type {
                qrpc_core::KnownOrUnknown::Known(value) => value.clone(),
                qrpc_core::KnownOrUnknown::Unknown(_) => "unknown".to_string(),
            },
            sizing_kind: ExecutionSizingKind::EquityNotionalRatio,
            slippage_bps: strategy_ir
                .execution_profile
                .as_ref()
                .and_then(|profile| profile.slippage_bps)
                .unwrap_or(0.0),
            taker_fee_bps: strategy_ir
                .execution_profile
                .as_ref()
                .and_then(|profile| profile.fee_bps)
                .unwrap_or(PAPER_EXECUTION_PROFILE_DEFAULT_FEE_BPS),
            total_cost_buffer_bps: 0.0,
            time_in_force: strategy_ir
                .execution
                .time_in_force
                .as_ref()
                .and_then(|value| match value {
                    qrpc_core::KnownOrUnknown::Known(value) => Some(parse_time_in_force(value)),
                    qrpc_core::KnownOrUnknown::Unknown(_) => None,
                })
                .unwrap_or(CoreTimeInForce::Gtc),
            params: BTreeMap::new(),
        },
    );

    core_ir.data_bindings = strategy_ir
        .data_requirements
        .iter()
        .map(|requirement| DataBinding {
            data_id: requirement.data_id.clone(),
            kind: match requirement.data_type {
                qrpc_core::DataRequirementType::Kline => DataBindingKind::KlineSeries,
                qrpc_core::DataRequirementType::Quote => DataBindingKind::Quote,
                _ => DataBindingKind::KlineSeries,
            },
            source_hints: BTreeMap::new(),
        })
        .collect();

    core_ir.indicators = strategy_ir
        .signals
        .iter()
        .map(|signal| {
            Ok(IndicatorNode {
                indicator_id: signal.signal_id.clone(),
                kind: lower_strategy_indicator_kind(&signal.indicator.kind)?,
                inputs: signal
                    .indicator
                    .inputs
                    .iter()
                    .map(|input| SeriesExpr::DataRef {
                        data_id: input.clone(),
                    })
                    .collect(),
                spread_spec: lower_strategy_spread_spec(signal)?,
                custom_expr: lower_strategy_custom_expr(signal, &data_requirement_kinds)?,
                params: signal.indicator.params.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    core_ir.signal_rules = strategy_ir
        .logic
        .entry_rules
        .iter()
        .map(|rule| lower_strategy_logic_rule(rule, &strategy_ir.signals))
        .collect::<Result<Vec<_>>>()?;
    let strategy_risk_profile = strategy_ir.risk_profile.as_ref();
    let max_position_ratio = strategy_risk_profile
        .and_then(|profile| profile.max_position)
        .unwrap_or_else(|| known_or_unknown_f64(&strategy_ir.risk_rules.max_position_ratio, 1.0));
    let max_total_leverage = strategy_risk_profile
        .and_then(|profile| profile.max_total_leverage)
        .unwrap_or(GLOBAL_RISK_PROFILE_DEFAULT_MAX_TOTAL_LEVERAGE);
    let max_exchange_leverage = strategy_risk_profile
        .and_then(|profile| profile.max_exchange_leverage)
        .unwrap_or(GLOBAL_RISK_PROFILE_DEFAULT_MAX_EXCHANGE_LEVERAGE);
    let min_action_interval_ms = strategy_risk_profile
        .and_then(|profile| profile.min_action_interval_ms)
        .unwrap_or(GLOBAL_RISK_PROFILE_DEFAULT_MIN_ACTION_INTERVAL_MS);

    core_ir.agent_policies.push(AgentPolicy {
        agent_id: "strategy_ir.agent".to_string(),
        name: if strategy_ir.logic.rebalance_rule.is_some()
            && matches!(
                strategy_ir.logic.position_sizing.method,
                qrpc_core::PositionSizingMethod::EqualWeight
            ) {
            "Strategy IR Portfolio Rebalance Agent".to_string()
        } else {
            "Strategy IR Weighted Agent".to_string()
        },
        kind: if strategy_ir.logic.rebalance_rule.is_some()
            && matches!(
                strategy_ir.logic.position_sizing.method,
                qrpc_core::PositionSizingMethod::EqualWeight
            ) {
            AgentPolicyKind::PortfolioRebalance
        } else {
            AgentPolicyKind::WeightedSignals
        },
        input_signal_ids: strategy_ir
            .signals
            .iter()
            .map(|signal| signal.signal_id.clone())
            .collect(),
        rebalance_symbols: vec![],
        rebalance_schedule: strategy_ir
            .logic
            .rebalance_rule
            .as_ref()
            .and_then(|rule| match &rule.frequency {
                qrpc_core::KnownOrUnknown::Known(value) => lower_rebalance_schedule(value).ok(),
                qrpc_core::KnownOrUnknown::Unknown(_) => None,
            }),
        rebalance_allocation_kind: None,
        rebalance_rank_method: None,
        rebalance_score_normalize: None,
        rebalance_target_weights: vec![],
        decision_threshold: Some(0.05),
        max_quantity_ratio: max_position_ratio,
        spread_trigger_bps: None,
        enabled: true,
    });
    core_ir.risk_policies.push(RiskPolicy {
        policy_id: "strategy_ir.risk".to_string(),
        name: "Strategy IR Risk".to_string(),
        observed_agent_ids: vec!["strategy_ir.agent".to_string()],
        max_position_ratio,
        max_single_weight: None,
        max_concentration_ratio: None,
        max_symbol_net_exposure_ratio: None,
        max_portfolio_net_exposure_ratio: None,
        max_turnover: None,
        min_trade_weight: None,
        max_new_positions_per_rebalance: None,
        max_total_leverage,
        max_exchange_leverage,
        min_action_interval_ms,
        enabled: true,
        max_cross_symbol_leverage: None,
    });

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

fn lower_rebalance_schedule(value: &str) -> Result<CoreRebalanceSchedule> {
    match value {
        "slow" => Ok(CoreRebalanceSchedule::EverySlow),
        "1d" => Ok(CoreRebalanceSchedule::Every1d),
        "weekly" => Ok(CoreRebalanceSchedule::Weekly),
        other => bail!("不支持的重新平衡计划: {other}"),
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

fn lower_strategy_indicator_kind(kind: &qrpc_core::IndicatorKind) -> Result<CoreIndicatorKind> {
    match kind {
        qrpc_core::IndicatorKind::MaCross => Ok(CoreIndicatorKind::MaCross),
        qrpc_core::IndicatorKind::Rsi => Ok(CoreIndicatorKind::Rsi),
        qrpc_core::IndicatorKind::Macd => Ok(CoreIndicatorKind::Macd),
        qrpc_core::IndicatorKind::Momentum => Ok(CoreIndicatorKind::Momentum),
        qrpc_core::IndicatorKind::Spread => Ok(CoreIndicatorKind::Spread),
        qrpc_core::IndicatorKind::ZScore => Ok(CoreIndicatorKind::ZScore),
        qrpc_core::IndicatorKind::Custom => Ok(CoreIndicatorKind::Custom),
        qrpc_core::IndicatorKind::QuoteObserve => Ok(CoreIndicatorKind::QuoteObserve),
        qrpc_core::IndicatorKind::Atr => Ok(CoreIndicatorKind::Atr),
        qrpc_core::IndicatorKind::BollingerBands => Ok(CoreIndicatorKind::BollingerBands),
        qrpc_core::IndicatorKind::Obv => Ok(CoreIndicatorKind::Obv),
        qrpc_core::IndicatorKind::Cmf => Ok(CoreIndicatorKind::Cmf),
        qrpc_core::IndicatorKind::Adx => Ok(CoreIndicatorKind::Adx),
        qrpc_core::IndicatorKind::Stochastic => Ok(CoreIndicatorKind::Stochastic),
        qrpc_core::IndicatorKind::Cci => Ok(CoreIndicatorKind::Cci),
        qrpc_core::IndicatorKind::ParabolicSar => Ok(CoreIndicatorKind::ParabolicSar),
        qrpc_core::IndicatorKind::KeltnerChannel => Ok(CoreIndicatorKind::KeltnerChannel),
        qrpc_core::IndicatorKind::DonchianChannel => Ok(CoreIndicatorKind::DonchianChannel),
    }
}

fn lower_logic_action(action: &qrpc_core::LogicAction) -> SignalKind {
    match action {
        qrpc_core::LogicAction::OpenLong => SignalKind::Long,
        qrpc_core::LogicAction::CloseLong | qrpc_core::LogicAction::OpenShort => SignalKind::Short,
        _ => SignalKind::Raw,
    }
}

fn lower_strategy_logic_rule(
    rule: &qrpc_core::LogicRule,
    signals: &[qrpc_core::SignalDefinition],
) -> Result<SignalRule> {
    let matched_signal = signals
        .iter()
        .find(|signal| rule.condition.contains(&signal.signal_id));

    let indicator_id = matched_signal
        .map(|signal| signal.signal_id.clone())
        .or_else(|| signals.first().map(|signal| signal.signal_id.clone()))
        .unwrap_or_else(|| "unknown_signal".to_string());

    let condition = if let Some(signal) = matched_signal {
        lower_strategy_logic_condition(rule, signal)?.unwrap_or_else(|| ScalarExpr::RawText {
            source: rule.condition.clone(),
        })
    } else {
        ScalarExpr::RawText {
            source: rule.condition.clone(),
        }
    };

    Ok(SignalRule {
        signal_id: rule.rule_id.clone(),
        indicator_id,
        signal_kind: lower_logic_action(&rule.action),
        condition,
    })
}

fn lower_strategy_logic_condition(
    rule: &qrpc_core::LogicRule,
    signal: &qrpc_core::SignalDefinition,
) -> Result<Option<ScalarExpr>> {
    match signal.indicator.kind {
        qrpc_core::IndicatorKind::MaCross => Ok(lower_strategy_ma_cross_condition(rule, signal)),
        qrpc_core::IndicatorKind::Rsi => {
            Ok(lower_strategy_indicator_threshold_condition(rule, signal))
        }
        qrpc_core::IndicatorKind::Momentum => {
            Ok(lower_strategy_indicator_threshold_condition(rule, signal))
        }
        qrpc_core::IndicatorKind::ZScore => {
            Ok(lower_strategy_indicator_threshold_condition(rule, signal))
        }
        qrpc_core::IndicatorKind::Spread => lower_strategy_spread_threshold_condition(rule, signal),
        _ => Ok(None),
    }
}

fn lower_strategy_spread_threshold_condition(
    rule: &qrpc_core::LogicRule,
    signal: &qrpc_core::SignalDefinition,
) -> Result<Option<ScalarExpr>> {
    if signal.indicator.inputs.len() != 2 {
        bail!(
            "QPSTRATSPREAD004 信号 `{}` 当前需要恰好两个 spread 输入",
            signal.signal_id
        );
    }

    let spread_spec = lower_strategy_spread_spec(signal)?.ok_or_else(|| {
        anyhow!(
            "QPSTRATSPREAD004 信号 `{}` 当前需要恰好两个 spread 输入",
            signal.signal_id
        )
    })?;

    if !matches!(spread_spec.output, SpreadValueKind::Bps) {
        bail!(
            "QPSTRATSPREAD001 信号 `{}` 当前仅支持 spread_output_code=bps",
            signal.signal_id
        );
    }

    if spread_spec.align.tolerance_ms == 0 {
        bail!(
            "QPSTRATSPREAD002 信号 `{}` 当前需要一个正数的 max_time_diff_ms 容差",
            signal.signal_id
        );
    }

    if !matches!(rule.action, qrpc_core::LogicAction::OpenLong) {
        bail!(
            "QPSTRATSPREAD003 信号 `{}` 当前仅支持使用 `>` 或 `>=` 的单一方向买入形态，需带明确的数值阈值",
            signal.signal_id
        );
    }

    let (left, op, right) = parse_strategy_signal_compare(&rule.condition).ok_or_else(|| {
        anyhow!(
            "QPSTRATSPREAD003 信号 `{}` 当前仅支持使用 `>` 或 `>=` 的单一方向买入形态，需带明确的数值阈值",
            signal.signal_id
        )
    })?;

    if left != signal.signal_id || !matches!(op, ComparisonOp::Gt | ComparisonOp::Gte) {
        bail!(
            "QPSTRATSPREAD003 信号 `{}` 当前仅支持使用 `>` 或 `>=` 的单一方向买入形态，需带明确的数值阈值",
            signal.signal_id
        );
    }

    Ok(indicator_threshold_compare_expr(
        signal.signal_id.clone(),
        op,
        right,
    ))
}

fn lower_strategy_ma_cross_condition(
    rule: &qrpc_core::LogicRule,
    signal: &qrpc_core::SignalDefinition,
) -> Option<ScalarExpr> {
    if !matches!(rule.action, qrpc_core::LogicAction::OpenLong) {
        return None;
    }
    let (left, op, right) = parse_strategy_signal_compare(&rule.condition)?;
    if left != signal.signal_id {
        return None;
    }
    if !matches!(op, ComparisonOp::Gt | ComparisonOp::Gte) {
        return None;
    }
    if right.abs() > f64::EPSILON {
        return None;
    }
    let data_id = signal.indicator.inputs.first()?.clone();
    let fast_period =
        strategy_indicator_usize_param(&signal.indicator.params, &["fast", "fast_period"])?;
    let slow_period =
        strategy_indicator_usize_param(&signal.indicator.params, &["slow", "slow_period"])?;
    moving_average_compare_expr(data_id, fast_period, op, slow_period)
}

fn lower_strategy_indicator_threshold_condition(
    rule: &qrpc_core::LogicRule,
    signal: &qrpc_core::SignalDefinition,
) -> Option<ScalarExpr> {
    if !matches!(rule.action, qrpc_core::LogicAction::OpenLong) {
        return None;
    }
    let (left, op, right) = parse_strategy_signal_compare(&rule.condition)?;
    if left != signal.signal_id {
        return None;
    }
    match signal.indicator.kind {
        qrpc_core::IndicatorKind::Rsi => {
            if matches!(rule.action, qrpc_core::LogicAction::OpenLong) {
                if !matches!(op, ComparisonOp::Lt | ComparisonOp::Lte) {
                    return None;
                }
            } else if matches!(rule.action, qrpc_core::LogicAction::OpenShort) {
                if !matches!(op, ComparisonOp::Gt | ComparisonOp::Gte) {
                    return None;
                }
            } else {
                return None;
            }
        }
        qrpc_core::IndicatorKind::Momentum => {
            if !matches!(op, ComparisonOp::Gt | ComparisonOp::Gte) {
                return None;
            }
        }
        qrpc_core::IndicatorKind::ZScore => {
            if !matches!(op, ComparisonOp::Lt | ComparisonOp::Lte) {
                return None;
            }
        }
        _ => return None,
    }
    indicator_threshold_compare_expr(signal.signal_id.clone(), op, right)
}

fn parse_strategy_signal_compare(condition: &str) -> Option<(String, ComparisonOp, f64)> {
    let trimmed = condition.trim();
    // 拒绝复合条件，防止静默忽略 "A > 5 and B > 10" 的第二部分
    let op_count = trimmed.matches(">=").count()
        + trimmed.matches("<=").count()
        + trimmed.chars().filter(|c| *c == '>' || *c == '<').count()
        - trimmed.matches(">=").count()
        - trimmed.matches("<=").count();
    // 分别统计 == (只计独立的 ==, 不计 >= <= 中的 =)
    let eq_count = trimmed.matches("==").count();
    let op_count = op_count + eq_count;
    if op_count > 1 {
        return None; // 复合条件拒绝，由上层作为 unknown_signal 处理
    }
    for (needle, op) in [
        (">=", ComparisonOp::Gte),
        ("<=", ComparisonOp::Lte),
        (">", ComparisonOp::Gt),
        ("<", ComparisonOp::Lt),
        ("==", ComparisonOp::Eq),
    ] {
        if let Some((left, right)) = trimmed.split_once(needle) {
            let threshold = right.trim().parse::<f64>().ok()?;
            return Some((left.trim().to_string(), op, threshold));
        }
    }
    None
}

fn strategy_indicator_usize_param(
    params: &BTreeMap<String, Value>,
    keys: &[&str],
) -> Option<usize> {
    keys.iter().find_map(|key| {
        params
            .get(*key)?
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
    })
}

fn parse_time_in_force(value: &str) -> CoreTimeInForce {
    match value.to_ascii_lowercase().as_str() {
        "ioc" => CoreTimeInForce::Ioc,
        "fok" => CoreTimeInForce::Fok,
        _ => CoreTimeInForce::Gtc,
    }
}

fn known_or_unknown_f64(value: &qrpc_core::KnownOrUnknown<f64>, default: f64) -> f64 {
    match value {
        qrpc_core::KnownOrUnknown::Known(value) => *value,
        qrpc_core::KnownOrUnknown::Unknown(_) => default,
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

fn runtime_intent_is_spread(intent: &IntentConfig) -> bool {
    matches!(intent.kind, IntentKind::QuoteObserve) && intent.input_data_ids.len() >= 2
}

fn lower_runtime_spread_spec(intent: &IntentConfig) -> Option<SpreadSpec> {
    runtime_intent_is_spread(intent)
        .then(|| build_spread_spec(&intent.input_data_ids, &intent.params))
        .flatten()
}

fn lower_strategy_spread_spec(signal: &qrpc_core::SignalDefinition) -> Result<Option<SpreadSpec>> {
    if !matches!(signal.indicator.kind, qrpc_core::IndicatorKind::Spread) {
        return Ok(None);
    }

    let mut params = BTreeMap::new();
    for (key, value) in &signal.indicator.params {
        if let Some(number) = value.as_f64() {
            params.insert(key.clone(), number);
        }
    }
    Ok(build_spread_spec(&signal.indicator.inputs, &params))
}

fn lower_strategy_custom_expr(
    signal: &qrpc_core::SignalDefinition,
    data_requirement_kinds: &BTreeMap<String, qrpc_core::DataRequirementType>,
) -> Result<Option<CustomExprSpec>> {
    if !matches!(signal.indicator.kind, qrpc_core::IndicatorKind::Custom) {
        return Ok(None);
    }

    let Some(raw_spec) = signal.indicator.params.get("custom_expr") else {
        bail!(
            "CUSTOM001 信号 `{}` 缺少 params.custom_expr",
            signal.signal_id
        );
    };

    let spec = serde_json::from_value::<CustomExprSpec>(raw_spec.clone()).map_err(|err| {
        anyhow!(
            "CUSTOM002 信号 `{}` 包含无效的 custom_expr 负载: {}",
            signal.signal_id,
            err
        )
    })?;

    validate_custom_expr_spec(
        &spec,
        signal,
        data_requirement_kinds,
        &signal.indicator.inputs.iter().cloned().collect(),
    )?;
    Ok(Some(spec))
}

fn validate_custom_expr_spec(
    spec: &CustomExprSpec,
    signal: &qrpc_core::SignalDefinition,
    data_requirement_kinds: &BTreeMap<String, qrpc_core::DataRequirementType>,
    declared_inputs: &BTreeSet<String>,
) -> Result<()> {
    if spec.schema_version != CUSTOM_EXPR_V1_VERSION {
        bail!(
            "CUSTOM003 信号 `{}` 必须使用 schema_version `{}`",
            signal.signal_id,
            CUSTOM_EXPR_V1_VERSION
        );
    }
    if matches!(spec.signal_kind, SignalKind::Raw) {
        bail!(
            "CUSTOM004 信号 `{}` 不能使用 raw signal_kind",
            signal.signal_id
        );
    }
    if !(0.0..=1.0).contains(&spec.confidence) {
        bail!(
            "CUSTOM005 信号 `{}` 置信度必须在 [0, 1] 范围内",
            signal.signal_id
        );
    }

    validate_custom_value_expr(
        &spec.predicate.left,
        signal,
        data_requirement_kinds,
        declared_inputs,
    )?;
    validate_custom_value_expr(
        &spec.predicate.right,
        signal,
        data_requirement_kinds,
        declared_inputs,
    )?;
    if let Some(strength) = &spec.strength {
        validate_custom_value_expr(strength, signal, data_requirement_kinds, declared_inputs)?;
    }

    Ok(())
}

fn validate_custom_value_expr(
    expr: &CustomValueExpr,
    signal: &qrpc_core::SignalDefinition,
    data_requirement_kinds: &BTreeMap<String, qrpc_core::DataRequirementType>,
    declared_inputs: &BTreeSet<String>,
) -> Result<()> {
    match expr {
        CustomValueExpr::Number { .. } => Ok(()),
        CustomValueExpr::Input { data_id, .. } => {
            validate_custom_data_id(data_id, signal, data_requirement_kinds, declared_inputs)
        }
        CustomValueExpr::WindowAgg {
            data_id,
            field,
            window_size,
            ..
        } => {
            validate_custom_data_id(data_id, signal, data_requirement_kinds, declared_inputs)?;
            if *window_size == 0 || *window_size > 512 {
                bail!(
                    "CUSTOM008 信号 `{}` window_size 必须在 [1, 512] 范围内",
                    signal.signal_id
                );
            }
            let Some(data_kind) = data_requirement_kinds.get(data_id) else {
                bail!(
                    "CUSTOM006 信号 `{}` 引用了未知的 data_id `{}`",
                    signal.signal_id,
                    data_id
                );
            };
            if !matches!(data_kind, qrpc_core::DataRequirementType::Kline) {
                bail!(
                    "CUSTOM009 信号 `{}` 仅允许对 kline 输入进行窗口聚合",
                    signal.signal_id
                );
            }
            validate_window_agg_field(*field, signal)?;
            Ok(())
        }
        CustomValueExpr::Binary { left, op, right } => {
            validate_custom_value_expr(left, signal, data_requirement_kinds, declared_inputs)?;
            validate_custom_value_expr(right, signal, data_requirement_kinds, declared_inputs)?;
            if matches!(op, ArithmeticOp::Div)
                && matches!(right.as_ref(), CustomValueExpr::Number { value } if value.abs() <= f64::EPSILON)
            {
                bail!("CUSTOM010 信号 `{}` 不能除以字面零值", signal.signal_id);
            }
            Ok(())
        }
        CustomValueExpr::Unary { value, .. } => {
            validate_custom_value_expr(value, signal, data_requirement_kinds, declared_inputs)
        }
    }
}

fn validate_custom_data_id(
    data_id: &str,
    signal: &qrpc_core::SignalDefinition,
    data_requirement_kinds: &BTreeMap<String, qrpc_core::DataRequirementType>,
    declared_inputs: &BTreeSet<String>,
) -> Result<()> {
    if !declared_inputs.contains(data_id) {
        bail!(
            "CUSTOM006 信号 `{}` 引用了未声明的输入 `{}`",
            signal.signal_id,
            data_id
        );
    }
    if !data_requirement_kinds.contains_key(data_id) {
        bail!(
            "CUSTOM007 信号 `{}` 引用了缺失的数据要求 `{}`",
            signal.signal_id,
            data_id
        );
    }
    Ok(())
}

fn validate_window_agg_field(
    field: SeriesField,
    signal: &qrpc_core::SignalDefinition,
) -> Result<()> {
    match field {
        SeriesField::MidOrClose
        | SeriesField::Close
        | SeriesField::Open
        | SeriesField::High
        | SeriesField::Low
        | SeriesField::Volume => Ok(()),
        SeriesField::BidOrClose | SeriesField::AskOrClose => bail!(
            "CUSTOM011 信号 `{}` 不能对 kline 窗口聚合报价侧字段",
            signal.signal_id
        ),
    }
}

fn build_spread_spec(
    input_data_ids: &[String],
    params: &BTreeMap<String, f64>,
) -> Option<SpreadSpec> {
    if input_data_ids.len() != 2 {
        return None;
    }

    Some(SpreadSpec {
        left: build_spread_series_expr("left", &input_data_ids[0], params),
        right: build_spread_series_expr("right", &input_data_ids[1], params),
        align: AlignAsofSpec {
            direction: decode_align_direction(
                params
                    .get("align_direction_code")
                    .copied()
                    .unwrap_or_default() as u64,
            ),
            tolerance_ms: params
                .get("max_time_diff_ms")
                .copied()
                .unwrap_or(5_000.0)
                .max(0.0)
                .round() as u64,
        },
        output: decode_spread_output(
            params
                .get("spread_output_code")
                .copied()
                .unwrap_or_default() as u64,
        ),
    })
}

fn build_spread_series_expr(
    side: &str,
    data_id: &str,
    params: &BTreeMap<String, f64>,
) -> SeriesExpr {
    let field = decode_series_field(param_with_fallback(params, side, "field_code", 0.0) as u64);
    let resample_period_ms = param_with_fallback(params, side, "resample_period_ms", 0.0)
        .max(0.0)
        .round() as u64;
    let resample_agg =
        decode_series_aggregation(
            param_with_fallback(params, side, "resample_agg_code", 0.0) as u64
        );
    let window_size = param_with_fallback(params, side, "window_size", 1.0)
        .max(1.0)
        .round() as usize;
    let window_agg =
        decode_series_aggregation(param_with_fallback(params, side, "window_agg_code", 1.0) as u64);
    let mut expr = SeriesExpr::DataField {
        data_id: data_id.to_string(),
        field,
    };
    if resample_period_ms > 0 {
        expr = SeriesExpr::Resample {
            input: Box::new(expr),
            period_ms: resample_period_ms,
            agg: resample_agg,
        };
    }
    if window_size > 1 {
        expr = SeriesExpr::WindowAgg {
            input: Box::new(expr),
            window_size,
            agg: window_agg,
        };
    }
    expr
}

fn param_with_fallback(params: &BTreeMap<String, f64>, side: &str, key: &str, default: f64) -> f64 {
    params
        .get(&format!("{side}_{key}"))
        .copied()
        .or_else(|| params.get(key).copied())
        .unwrap_or(default)
}

fn decode_series_field(code: u64) -> SeriesField {
    match code {
        1 => SeriesField::BidOrClose,
        2 => SeriesField::AskOrClose,
        3 => SeriesField::Close,
        4 => SeriesField::Open,
        5 => SeriesField::High,
        6 => SeriesField::Low,
        7 => SeriesField::Volume,
        _ => SeriesField::MidOrClose,
    }
}

fn decode_series_aggregation(code: u64) -> SeriesAggregation {
    match code {
        1 => SeriesAggregation::Mean,
        2 => SeriesAggregation::Sum,
        3 => SeriesAggregation::Min,
        4 => SeriesAggregation::Max,
        5 => SeriesAggregation::StdDev,
        _ => SeriesAggregation::Last,
    }
}

fn decode_align_direction(code: u64) -> AlignDirection {
    match code {
        1 => AlignDirection::Forward,
        2 => AlignDirection::Nearest,
        _ => AlignDirection::Backward,
    }
}

fn decode_spread_output(code: u64) -> SpreadValueKind {
    match code {
        1 => SpreadValueKind::Bps,
        2 => SpreadValueKind::Absolute,
        _ => SpreadValueKind::Ratio,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_core::{
        DataRequirement, DataRequirementType, DataSourceConfig, Exchange, IndicatorDefinition,
        IndicatorKind, KnownOrUnknown, LogicAction, LogicRule, MarketType, PositionSizing,
        PositionSizingMethod, PositionSizingUnit, RiskConfig, SignalDefinition, StrategyExecution,
        StrategyIr, StrategyLogic, StrategyMetadata, StrategyRiskRules, StrategySource,
        StrategySourceType, Symbol,
    };
    use serde_json::json;

    #[test]
    fn rejects_agent_without_risk_owner() {
        let mut config = sample_config();
        config.risks[0].observed_agent_ids.clear();
        let err = validate_runtime_protocol_config(&config).unwrap_err();
        assert!(err.to_string().contains("必须至少观察一个代理"));
    }

    #[test]
    fn rejects_non_finite_execution_costs() {
        let mut config = sample_config();
        config.taker_fee_bps = f64::INFINITY;
        let err = validate_runtime_protocol_config(&config).unwrap_err();
        assert!(err.to_string().contains("有限数"));

        let mut config = sample_config();
        config.default_slippage_bps = f64::NAN;
        let err = validate_runtime_protocol_config(&config).unwrap_err();
        assert!(err.to_string().contains("有限数"));
    }

    #[test]
    fn rejects_quote_intent_bound_to_kline_source() {
        let mut config = sample_config();
        config.intents[2].input_data_ids = vec!["binance_btc_150d_1d".into()];
        let err = validate_runtime_protocol_config(&config).unwrap_err();
        assert!(err.to_string().contains("期望 Quote 输入"));
    }

    #[test]
    fn compiles_valid_minimal_spec() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        assert_eq!(compiled.protocol_name, "quantpilot/minimal-sim/v1");
    }

    #[test]
    fn lowers_runtime_protocol_into_core_ir() {
        let core_ir = lower_runtime_protocol_to_core_ir(&sample_config()).unwrap();
        assert_eq!(core_ir.ir_version, qrpc_core_ir::CORE_IR_V1_VERSION);
        assert_eq!(core_ir.indicators.len(), 3);
        assert_eq!(core_ir.signal_rules.len(), 3);
        assert_eq!(core_ir.agent_policies.len(), 1);
        assert_eq!(core_ir.risk_policies.len(), 1);
    }

    #[test]
    fn lowers_runtime_portfolio_rebalance_agent_into_core_ir() {
        let mut config = sample_config();
        config.agents[0]
            .params
            .insert("portfolio_rebalance".into(), 1.0);
        config.agents[0].rebalance_symbols = vec![Symbol::BtcUsdt, Symbol::parse("ETHUSDT")];
        config.agents[0].rebalance_schedule = Some(RuntimeRebalanceSchedule::Every1d);

        let core_ir = lower_runtime_protocol_to_core_ir(&config).unwrap();

        assert_eq!(
            core_ir.agent_policies[0].kind,
            AgentPolicyKind::PortfolioRebalance
        );
        assert_eq!(
            core_ir.agent_policies[0].rebalance_symbols,
            vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()]
        );
        assert_eq!(
            core_ir.agent_policies[0].rebalance_schedule,
            Some(CoreRebalanceSchedule::Every1d)
        );
        assert_eq!(
            core_ir.agent_policies[0]
                .rebalance_allocation_kind
                .as_deref(),
            None
        );
    }

    #[test]
    fn lowers_runtime_portfolio_rebalance_allocation_metadata_into_core_ir() {
        let mut config = sample_config();
        config.agents[0]
            .params
            .insert("portfolio_rebalance".into(), 1.0);
        config.agents[0].rebalance_symbols = vec![
            Symbol::BtcUsdt,
            Symbol::parse("ETHUSDT"),
            Symbol::parse("SOLUSDT"),
        ];
        config.agents[0].rebalance_allocation_kind = Some("rank_weight".into());
        config.agents[0].rebalance_rank_method = Some("inverse_rank".into());

        let core_ir = lower_runtime_protocol_to_core_ir(&config).unwrap();

        assert_eq!(
            core_ir.agent_policies[0]
                .rebalance_allocation_kind
                .as_deref(),
            Some("rank_weight")
        );
        assert_eq!(
            core_ir.agent_policies[0].rebalance_rank_method.as_deref(),
            Some("inverse_rank")
        );
        assert!(core_ir.agent_policies[0]
            .rebalance_target_weights
            .is_empty());
    }

    #[test]
    fn lowers_multi_quote_observe_into_spread_indicator() {
        let mut config = sample_config();
        config.intents = vec![IntentConfig {
            intent_id: "intent_spread".into(),
            name: "Spread".into(),
            kind: IntentKind::QuoteObserve,
            input_data_ids: vec!["binance_btc_quote".into(), "okx_btc_quote".into()],
            params: BTreeMap::from([
                ("max_time_diff_ms".into(), 5_000.0),
                ("field_code".into(), 0.0),
                ("resample_period_ms".into(), 60_000.0),
                ("resample_agg_code".into(), 0.0),
                ("window_size".into(), 3.0),
                ("window_agg_code".into(), 1.0),
                ("spread_output_code".into(), 1.0),
            ]),
            enabled: true,
        }];
        config.agents = vec![AgentConfig {
            agent_id: "agent_arb".into(),
            name: "Arb".into(),
            input_intent_ids: vec!["intent_spread".into()],
            rebalance_symbols: vec![],
            rebalance_schedule: None,
            rebalance_allocation_kind: None,
            rebalance_rank_method: None,
            rebalance_score_normalize: None,
            rebalance_target_weights: vec![],
            params: BTreeMap::from([("spread_trigger_bps".into(), 30.0)]),
            enabled: true,
        }];
        config.risks = vec![RiskConfig {
            risk_id: "risk_global".into(),
            name: "Global Risk".into(),
            observed_agent_ids: vec!["agent_arb".into()],
            max_position_ratio: 0.2,
            max_single_weight: None,
            max_concentration_ratio: None,
            max_symbol_net_exposure_ratio: None,
            max_portfolio_net_exposure_ratio: None,
            max_turnover: None,
            min_trade_weight: None,
            max_new_positions_per_rebalance: None,
            max_total_leverage: 3.0,
            max_exchange_leverage: 3.0,
            min_action_interval_ms: 1_000,
            enabled: true,
        }];

        let core_ir = lower_runtime_protocol_to_core_ir(&config).unwrap();

        assert_eq!(core_ir.indicators.len(), 1);
        assert_eq!(core_ir.indicators[0].kind, CoreIndicatorKind::Spread);
        assert!(core_ir.indicators[0].spread_spec.is_some());
        assert_eq!(
            core_ir.agent_policies[0].kind,
            AgentPolicyKind::CrossVenueArbitrage
        );
    }

    #[test]
    fn lowers_one_sided_spread_bps_threshold_into_structured_compare_condition() {
        let mut config = sample_config();
        config.intents = vec![IntentConfig {
            intent_id: "intent_spread".into(),
            name: "Spread".into(),
            kind: IntentKind::QuoteObserve,
            input_data_ids: vec!["binance_btc_quote".into(), "okx_btc_quote".into()],
            params: BTreeMap::from([
                ("max_time_diff_ms".into(), 5_000.0),
                ("field_code".into(), 0.0),
                ("align_direction_code".into(), 0.0),
                ("spread_output_code".into(), 1.0),
                ("comparison_shape_code".into(), 1.0),
                ("comparison_op_code".into(), 2.0),
                ("comparison_threshold".into(), 5.0),
            ]),
            enabled: true,
        }];
        config.agents = vec![AgentConfig {
            agent_id: "agent_arb".into(),
            name: "Arb".into(),
            input_intent_ids: vec!["intent_spread".into()],
            rebalance_symbols: vec![],
            rebalance_schedule: None,
            rebalance_allocation_kind: None,
            rebalance_rank_method: None,
            rebalance_score_normalize: None,
            rebalance_target_weights: vec![],
            params: BTreeMap::from([("spread_trigger_bps".into(), 30.0)]),
            enabled: true,
        }];
        config.risks[0].observed_agent_ids = vec!["agent_arb".into()];

        let core_ir = lower_runtime_protocol_to_core_ir(&config).unwrap();

        assert_eq!(
            core_ir.signal_rules[0].condition,
            ScalarExpr::Compare {
                left: Box::new(ScalarExpr::Ref {
                    name: "intent_spread".into(),
                }),
                op: ComparisonOp::Gt,
                right: Box::new(ScalarExpr::Number { value: 5.0 }),
            }
        );
    }

    #[test]
    fn accepts_mixed_quote_and_kline_inputs_for_spread() {
        let mut config = sample_config();
        config.intents = vec![IntentConfig {
            intent_id: "intent_cross_source".into(),
            name: "Cross Source Spread".into(),
            kind: IntentKind::QuoteObserve,
            input_data_ids: vec!["binance_btc_quote".into(), "binance_btc_150d_1d".into()],
            params: BTreeMap::from([
                ("field_code".into(), 0.0),
                ("window_size".into(), 5.0),
                ("window_agg_code".into(), 1.0),
            ]),
            enabled: true,
        }];
        config.agents[0].input_intent_ids = vec!["intent_cross_source".into()];

        let core_ir = lower_runtime_protocol_to_core_ir(&config).unwrap();

        assert_eq!(core_ir.indicators[0].kind, CoreIndicatorKind::Spread);
        assert!(core_ir.indicators[0].spread_spec.is_some());
    }

    #[test]
    fn lowers_strategy_ir_into_core_ir() {
        let strategy_ir = StrategyIr {
            ir_version: qrpc_core::STRATEGY_IR_V0_VERSION.to_string(),
            metadata: StrategyMetadata {
                strategy_id: "paper_dual_ma_v1".into(),
                name: "Dual Moving Average Trend Strategy".into(),
                summary:
                    "Go long when the fast moving average crosses above the slow moving average."
                        .into(),
                source: StrategySource {
                    source_type: StrategySourceType::ManualPaperAnalysis,
                    paper_title: "A Dual Moving Average Strategy".into(),
                    paper_reference: Some("doi:10.0000/example".into()),
                },
                authors: vec!["QuantPilot".into()],
                tags: vec!["trend".into(), "moving_average".into()],
            },
            signals: vec![SignalDefinition {
                signal_id: "ma_cross".into(),
                name: "MA Cross".into(),
                indicator: IndicatorDefinition {
                    kind: IndicatorKind::MaCross,
                    inputs: vec!["btc_1d".into()],
                    params: BTreeMap::from([
                        ("fast".into(), json!(20)),
                        ("slow".into(), json!(50)),
                    ]),
                },
                transforms: vec![],
            }],
            logic: StrategyLogic {
                entry_rules: vec![LogicRule {
                    rule_id: "entry_rule".into(),
                    condition: "ma_cross > 0".into(),
                    action: LogicAction::OpenLong,
                }],
                exit_rules: vec![],
                position_sizing: PositionSizing {
                    method: PositionSizingMethod::FixedRatio,
                    value: KnownOrUnknown::Known(0.2),
                    unit: PositionSizingUnit::PortfolioRatio,
                },
                rebalance_rule: None,
            },
            risk_rules: StrategyRiskRules {
                max_position_ratio: KnownOrUnknown::Known(0.2),
                stop_loss_ratio: KnownOrUnknown::Known(0.05),
                take_profit_ratio: None,
                max_drawdown_ratio: None,
                max_trades_per_day: None,
                notes: vec![],
            },
            risk_profile: None,
            data_requirements: vec![DataRequirement {
                data_id: "btc_1d".into(),
                venue: KnownOrUnknown::Known("binance".into()),
                symbol: KnownOrUnknown::Known("BTCUSDT".into()),
                data_type: DataRequirementType::Kline,
                granularity: KnownOrUnknown::Known("1d".into()),
                lookback: KnownOrUnknown::Known(200),
                fields: vec!["close".into()],
            }],
            execution: StrategyExecution {
                venue_type: KnownOrUnknown::Known("paper".into()),
                order_type: KnownOrUnknown::Known("market".into()),
                time_in_force: None,
                slippage_model: KnownOrUnknown::Known("fixed_bps".into()),
                latency_assumption_ms: None,
                capital_base: None,
            },
            execution_profile: None,
            gap_annotations: vec![],
            unknowns: vec![],
        };
        let core_ir = lower_strategy_ir_to_core_ir(&strategy_ir).unwrap();
        assert_eq!(core_ir.metadata.strategy_id, "paper_dual_ma_v1");
        assert_eq!(core_ir.indicators.len(), 1);
        assert_eq!(core_ir.agent_policies.len(), 1);
        assert_eq!(core_ir.risk_policies.len(), 1);
        assert_eq!(
            core_ir.signal_rules[0].condition,
            ScalarExpr::Compare {
                left: Box::new(ScalarExpr::Series {
                    expr: SeriesExpr::WindowAgg {
                        input: Box::new(SeriesExpr::DataField {
                            data_id: "btc_1d".into(),
                            field: SeriesField::Close,
                        }),
                        window_size: 20,
                        agg: SeriesAggregation::Mean,
                    },
                }),
                op: ComparisonOp::Gt,
                right: Box::new(ScalarExpr::Series {
                    expr: SeriesExpr::WindowAgg {
                        input: Box::new(SeriesExpr::DataField {
                            data_id: "btc_1d".into(),
                            field: SeriesField::Close,
                        }),
                        window_size: 50,
                        agg: SeriesAggregation::Mean,
                    },
                }),
            }
        );
    }

    #[test]
    fn lowers_equal_weight_rebalance_strategy_ir_into_portfolio_rebalance_agent() {
        let mut strategy_ir = sample_strategy_ir_with_indicator(IndicatorKind::MaCross);
        strategy_ir.logic.position_sizing = PositionSizing {
            method: PositionSizingMethod::EqualWeight,
            value: KnownOrUnknown::Known(1.0),
            unit: PositionSizingUnit::PortfolioRatio,
        };
        strategy_ir.logic.rebalance_rule = Some(qrpc_core::RebalanceRule {
            frequency: KnownOrUnknown::Known("1d".into()),
            condition: None,
        });

        let core_ir = lower_strategy_ir_to_core_ir(&strategy_ir).unwrap();

        assert_eq!(
            core_ir.agent_policies[0].kind,
            AgentPolicyKind::PortfolioRebalance
        );
    }

    #[test]
    fn lowers_restricted_custom_indicator_into_core_ir() {
        let mut strategy_ir = sample_strategy_ir_with_indicator(IndicatorKind::Custom);
        strategy_ir.signals[0].indicator.params = BTreeMap::from([(
            "custom_expr".into(),
            json!({
                "schema_version": CUSTOM_EXPR_V1_VERSION,
                "signal_kind": "long",
                "predicate": {
                    "left": {
                        "kind": "window_agg",
                        "data_id": "btc_1d",
                        "field": "close",
                        "window_size": 3,
                        "agg": "mean"
                    },
                    "op": "gt",
                    "right": {
                        "kind": "number",
                        "value": 105.0
                    }
                },
                "strength": {
                    "kind": "binary",
                    "op": "sub",
                    "left": {
                        "kind": "input",
                        "data_id": "btc_1d",
                        "field": "close"
                    },
                    "right": {
                        "kind": "number",
                        "value": 100.0
                    }
                },
                "confidence": 0.9
            }),
        )]);

        let core_ir = lower_strategy_ir_to_core_ir(&strategy_ir).unwrap();

        assert_eq!(core_ir.indicators[0].kind, CoreIndicatorKind::Custom);
        assert!(core_ir.indicators[0].custom_expr.is_some());
        assert_eq!(
            core_ir.indicators[0]
                .custom_expr
                .as_ref()
                .unwrap()
                .schema_version,
            CUSTOM_EXPR_V1_VERSION
        );
    }

    #[test]
    fn rejects_custom_indicator_with_undeclared_input_reference() {
        let mut strategy_ir = sample_strategy_ir_with_indicator(IndicatorKind::Custom);
        strategy_ir.signals[0].indicator.params = BTreeMap::from([(
            "custom_expr".into(),
            json!({
                "schema_version": CUSTOM_EXPR_V1_VERSION,
                "signal_kind": "long",
                "predicate": {
                    "left": {
                        "kind": "input",
                        "data_id": "other_data",
                        "field": "close"
                    },
                    "op": "gt",
                    "right": {
                        "kind": "number",
                        "value": 100.0
                    }
                }
            }),
        )]);

        let err = lower_strategy_ir_to_core_ir(&strategy_ir).unwrap_err();
        assert!(err.to_string().contains("CUSTOM006"));
    }

    fn sample_config() -> RuntimeProtocolCoreConfig {
        RuntimeProtocolCoreConfig {
            data_sources: vec![
                DataSourceConfig {
                    data_id: "binance_btc_150d_1d".into(),
                    exchange: Exchange::Binance,
                    symbol: Symbol::BtcUsdt,
                    market_type: MarketType::Spot,
                    kind: DataKind::KlineSeries,
                    days: Some(150),
                    interval: Some("1d".into()),
                    ping_enabled: false,
                    request_interval_ms: None,
                    enabled: true,
                },
                DataSourceConfig {
                    data_id: "binance_btc_quote".into(),
                    exchange: Exchange::Binance,
                    symbol: Symbol::BtcUsdt,
                    market_type: MarketType::Spot,
                    kind: DataKind::Quote,
                    days: None,
                    interval: None,
                    ping_enabled: false,
                    request_interval_ms: None,
                    enabled: true,
                },
                DataSourceConfig {
                    data_id: "okx_btc_quote".into(),
                    exchange: Exchange::Okx,
                    symbol: Symbol::BtcUsdt,
                    market_type: MarketType::Spot,
                    kind: DataKind::Quote,
                    days: None,
                    interval: None,
                    ping_enabled: false,
                    request_interval_ms: None,
                    enabled: true,
                },
            ],
            intents: vec![
                IntentConfig {
                    intent_id: "intent_long_buy".into(),
                    name: "Long Buy".into(),
                    kind: IntentKind::LongTermBuy,
                    input_data_ids: vec!["binance_btc_150d_1d".into()],
                    params: Default::default(),
                    enabled: true,
                },
                IntentConfig {
                    intent_id: "intent_long_sell".into(),
                    name: "Long Sell".into(),
                    kind: IntentKind::LongTermSell,
                    input_data_ids: vec!["binance_btc_150d_1d".into()],
                    params: Default::default(),
                    enabled: true,
                },
                IntentConfig {
                    intent_id: "intent_binance_quote".into(),
                    name: "Binance Quote".into(),
                    kind: IntentKind::QuoteObserve,
                    input_data_ids: vec!["binance_btc_quote".into()],
                    params: Default::default(),
                    enabled: true,
                },
            ],
            agents: vec![AgentConfig {
                agent_id: "agent_long_term".into(),
                name: "Long Term".into(),
                input_intent_ids: vec!["intent_long_buy".into(), "intent_long_sell".into()],
                rebalance_symbols: vec![],
                rebalance_schedule: None,
                rebalance_allocation_kind: None,
                rebalance_rank_method: None,
                rebalance_score_normalize: None,
                rebalance_target_weights: vec![],
                params: Default::default(),
                enabled: true,
            }],
            risks: vec![RiskConfig {
                risk_id: "risk_global".into(),
                name: "Global Risk".into(),
                observed_agent_ids: vec!["agent_long_term".into()],
                max_position_ratio: 0.2,
                max_single_weight: None,
                max_concentration_ratio: None,
                max_symbol_net_exposure_ratio: None,
                max_portfolio_net_exposure_ratio: None,
                max_turnover: None,
                min_trade_weight: None,
                max_new_positions_per_rebalance: None,
                max_total_leverage: 3.0,
                max_exchange_leverage: 3.0,
                min_action_interval_ms: 1_000,
                enabled: true,
            }],
            initial_cash_balance: 100_000.0,
            taker_fee_bps: 10.0,
            default_slippage_bps: 5.0,
            total_cost_buffer_bps: 20.0,
        }
    }

    #[test]
    fn runtime_protocol_lowering_carries_data_request_controls_into_source_hints() {
        let mut config = sample_config();
        config.data_sources[0].ping_enabled = true;
        config.data_sources[0].request_interval_ms = Some(2_500);

        let core_ir = lower_runtime_protocol_to_core_ir(&config).unwrap();
        let source_hints = &core_ir.data_bindings[0].source_hints;

        assert_eq!(
            source_hints.get("ping_enabled").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            source_hints.get("request_interval_ms").map(String::as_str),
            Some("2500")
        );
    }

    fn sample_strategy_ir_with_indicator(kind: IndicatorKind) -> StrategyIr {
        StrategyIr {
            ir_version: qrpc_core::STRATEGY_IR_V0_VERSION.to_string(),
            metadata: StrategyMetadata {
                strategy_id: "paper_dual_ma_v1".into(),
                name: "Dual Moving Average Trend Strategy".into(),
                summary:
                    "Go long when the fast moving average crosses above the slow moving average."
                        .into(),
                source: StrategySource {
                    source_type: StrategySourceType::ManualPaperAnalysis,
                    paper_title: "A Dual Moving Average Strategy".into(),
                    paper_reference: Some("doi:10.0000/example".into()),
                },
                authors: vec!["QuantPilot".into()],
                tags: vec!["trend".into(), "moving_average".into()],
            },
            signals: vec![SignalDefinition {
                signal_id: "ma_cross".into(),
                name: "MA Cross".into(),
                indicator: IndicatorDefinition {
                    kind,
                    inputs: vec!["btc_1d".into()],
                    params: BTreeMap::new(),
                },
                transforms: vec![],
            }],
            logic: StrategyLogic {
                entry_rules: vec![LogicRule {
                    rule_id: "entry_rule".into(),
                    condition: "ma_cross > 0".into(),
                    action: LogicAction::OpenLong,
                }],
                exit_rules: vec![],
                position_sizing: PositionSizing {
                    method: PositionSizingMethod::FixedRatio,
                    value: KnownOrUnknown::Known(0.2),
                    unit: PositionSizingUnit::PortfolioRatio,
                },
                rebalance_rule: None,
            },
            risk_rules: StrategyRiskRules {
                max_position_ratio: KnownOrUnknown::Known(0.2),
                stop_loss_ratio: KnownOrUnknown::Known(0.05),
                take_profit_ratio: None,
                max_drawdown_ratio: None,
                max_trades_per_day: None,
                notes: vec![],
            },
            risk_profile: None,
            data_requirements: vec![DataRequirement {
                data_id: "btc_1d".into(),
                venue: KnownOrUnknown::Known("binance".into()),
                symbol: KnownOrUnknown::Known("BTCUSDT".into()),
                data_type: DataRequirementType::Kline,
                granularity: KnownOrUnknown::Known("1d".into()),
                lookback: KnownOrUnknown::Known(200),
                fields: vec!["close".into()],
            }],
            execution: StrategyExecution {
                venue_type: KnownOrUnknown::Known("paper".into()),
                order_type: KnownOrUnknown::Known("market".into()),
                time_in_force: None,
                slippage_model: KnownOrUnknown::Known("fixed_bps".into()),
                latency_assumption_ms: None,
                capital_base: None,
            },
            execution_profile: None,
            gap_annotations: vec![],
            unknowns: vec![],
        }
    }

    #[test]
    fn strategy_ir_risk_profile_lowers_to_global_risk_policy_shape() {
        let mut strategy_ir = sample_strategy_ir_with_indicator(IndicatorKind::Rsi);
        strategy_ir.risk_profile = Some(qrpc_core::StrategyRiskProfileRef {
            profile_id: qrpc_core::GLOBAL_RISK_PROFILE_ID.to_string(),
            max_position: Some(0.35),
            max_total_leverage: Some(4.0),
            max_exchange_leverage: Some(5.0),
            min_action_interval_ms: Some(250),
        });

        let core_ir = lower_strategy_ir_to_core_ir(&strategy_ir).unwrap();

        assert_eq!(core_ir.agent_policies[0].max_quantity_ratio, 0.35);
        assert_eq!(core_ir.risk_policies[0].max_position_ratio, 0.35);
        assert_eq!(core_ir.risk_policies[0].max_total_leverage, 4.0);
        assert_eq!(core_ir.risk_policies[0].max_exchange_leverage, 5.0);
        assert_eq!(core_ir.risk_policies[0].min_action_interval_ms, 250);
    }

    #[test]
    fn strategy_ir_execution_profile_lowers_to_paper_execution_rule_shape() {
        let mut strategy_ir = sample_strategy_ir_with_indicator(IndicatorKind::Momentum);
        strategy_ir.execution_profile = Some(qrpc_core::StrategyExecutionProfileRef {
            profile_id: qrpc_core::PAPER_EXECUTION_PROFILE_ID.to_string(),
            fee_bps: Some(12.5),
            slippage_bps: Some(7.5),
        });

        let core_ir = lower_strategy_ir_to_core_ir(&strategy_ir).unwrap();

        assert_eq!(core_ir.execution.venue_kind, "paper");
        assert_eq!(core_ir.execution.taker_fee_bps, 12.5);
        assert_eq!(core_ir.execution.slippage_bps, 7.5);
    }

    #[test]
    fn runtime_protocol_metadata_can_be_overridden_for_core_ir() {
        let config = sample_config();
        let compiled = compile_runtime_protocol_config_with_metadata(
            &config,
            CoreMetadata {
                strategy_id: "formal_graph".into(),
                name: "Formal Graph".into(),
                source_kind: CoreSourceKind::FormalQuantScript,
            },
        )
        .unwrap();

        assert_eq!(compiled.core_ir.metadata.strategy_id, "formal_graph");
        assert_eq!(compiled.core_ir.metadata.name, "Formal Graph");
        assert_eq!(
            compiled.core_ir.metadata.source_kind,
            CoreSourceKind::FormalQuantScript
        );
    }

    #[test]
    fn runtime_protocol_ma_intent_lowers_structured_series_compare_condition() {
        let mut config = sample_config();
        config.intents[0].params = BTreeMap::from([
            ("fast_period".into(), 20.0),
            ("slow_period".into(), 100.0),
            ("entry_ratio".into(), 1.0),
            ("comparison_op_code".into(), 2.0),
        ]);

        let compiled = compile_runtime_protocol_config(&config).unwrap();
        let condition = &compiled.core_ir.signal_rules[0].condition;

        assert_eq!(
            condition,
            &ScalarExpr::Compare {
                left: Box::new(ScalarExpr::Series {
                    expr: SeriesExpr::WindowAgg {
                        input: Box::new(SeriesExpr::DataField {
                            data_id: "binance_btc_150d_1d".into(),
                            field: SeriesField::Close,
                        }),
                        window_size: 20,
                        agg: SeriesAggregation::Mean,
                    },
                }),
                op: ComparisonOp::Gt,
                right: Box::new(ScalarExpr::Series {
                    expr: SeriesExpr::WindowAgg {
                        input: Box::new(SeriesExpr::DataField {
                            data_id: "binance_btc_150d_1d".into(),
                            field: SeriesField::Close,
                        }),
                        window_size: 100,
                        agg: SeriesAggregation::Mean,
                    },
                }),
            }
        );
    }

    #[test]
    fn runtime_protocol_rsi_intent_lowers_structured_threshold_condition_for_one_sided_shape() {
        let mut config = sample_config();
        config.intents.push(IntentConfig {
            intent_id: "intent_rsi".into(),
            name: "RSI".into(),
            kind: IntentKind::Rsi,
            input_data_ids: vec!["binance_btc_150d_1d".into()],
            params: BTreeMap::from([
                ("period".into(), 14.0),
                ("oversold_threshold".into(), 25.0),
                ("overbought_threshold".into(), 70.0),
                ("comparison_shape_code".into(), 1.0),
                ("comparison_op_code".into(), 0.0),
            ]),
            enabled: true,
        });

        let compiled = compile_runtime_protocol_config(&config).unwrap();
        let condition = &compiled.core_ir.signal_rules[3].condition;

        assert_eq!(
            condition,
            &ScalarExpr::Compare {
                left: Box::new(ScalarExpr::Ref {
                    name: "intent_rsi".into(),
                }),
                op: ComparisonOp::Lt,
                right: Box::new(ScalarExpr::Number { value: 25.0 }),
            }
        );
    }

    #[test]
    fn runtime_protocol_momentum_intent_lowers_structured_threshold_condition_for_one_sided_shape()
    {
        let mut config = sample_config();
        config.intents.push(IntentConfig {
            intent_id: "intent_momentum".into(),
            name: "Momentum".into(),
            kind: IntentKind::Momentum,
            input_data_ids: vec!["binance_btc_150d_1d".into()],
            params: BTreeMap::from([
                ("lookback".into(), 20.0),
                ("threshold_ratio".into(), 0.03),
                ("comparison_shape_code".into(), 1.0),
                ("comparison_op_code".into(), 2.0),
                ("comparison_threshold".into(), 0.03),
            ]),
            enabled: true,
        });

        let compiled = compile_runtime_protocol_config(&config).unwrap();
        let condition = &compiled.core_ir.signal_rules[3].condition;

        assert_eq!(
            condition,
            &ScalarExpr::Compare {
                left: Box::new(ScalarExpr::Ref {
                    name: "intent_momentum".into(),
                }),
                op: ComparisonOp::Gt,
                right: Box::new(ScalarExpr::Number { value: 0.03 }),
            }
        );
    }

    #[test]
    fn strategy_ir_momentum_rule_lowers_structured_threshold_condition_for_one_sided_shape() {
        let mut strategy_ir = sample_strategy_ir_with_indicator(IndicatorKind::Momentum);
        strategy_ir.signals[0].signal_id = "momentum_signal".into();
        strategy_ir.signals[0].name = "Momentum Signal".into();
        strategy_ir.signals[0].indicator.params = BTreeMap::from([("lookback".into(), json!(20))]);
        strategy_ir.logic.entry_rules[0].condition = "momentum_signal > 0.03".into();
        strategy_ir.logic.entry_rules[0].action = LogicAction::OpenLong;

        let compiled = lower_strategy_ir_to_core_ir(&strategy_ir).unwrap();
        let condition = &compiled.signal_rules[0].condition;

        assert_eq!(
            condition,
            &ScalarExpr::Compare {
                left: Box::new(ScalarExpr::Ref {
                    name: "momentum_signal".into(),
                }),
                op: ComparisonOp::Gt,
                right: Box::new(ScalarExpr::Number { value: 0.03 }),
            }
        );
    }

    #[test]
    fn strategy_ir_rsi_rule_lowers_structured_threshold_condition_for_one_sided_shape() {
        let mut strategy_ir = sample_strategy_ir_with_indicator(IndicatorKind::Rsi);
        strategy_ir.signals[0].signal_id = "rsi_signal".into();
        strategy_ir.signals[0].name = "RSI Signal".into();
        strategy_ir.signals[0].indicator.params = BTreeMap::from([("period".into(), json!(14))]);
        strategy_ir.logic.entry_rules[0].condition = "rsi_signal < 25".into();
        strategy_ir.logic.entry_rules[0].action = LogicAction::OpenLong;

        let compiled = lower_strategy_ir_to_core_ir(&strategy_ir).unwrap();
        let condition = &compiled.signal_rules[0].condition;

        assert_eq!(
            condition,
            &ScalarExpr::Compare {
                left: Box::new(ScalarExpr::Ref {
                    name: "rsi_signal".into(),
                }),
                op: ComparisonOp::Lt,
                right: Box::new(ScalarExpr::Number { value: 25.0 }),
            }
        );
    }

    #[test]
    fn strategy_ir_ma_cross_rule_lowers_structured_series_compare_condition() {
        let mut strategy_ir = sample_strategy_ir_with_indicator(IndicatorKind::MaCross);
        strategy_ir.signals[0].indicator.params =
            BTreeMap::from([("fast".into(), json!(20)), ("slow".into(), json!(50))]);
        strategy_ir.logic.entry_rules[0].condition = "ma_cross > 0".into();
        strategy_ir.logic.entry_rules[0].action = LogicAction::OpenLong;

        let compiled = lower_strategy_ir_to_core_ir(&strategy_ir).unwrap();
        let condition = &compiled.signal_rules[0].condition;

        assert_eq!(
            condition,
            &ScalarExpr::Compare {
                left: Box::new(ScalarExpr::Series {
                    expr: SeriesExpr::WindowAgg {
                        input: Box::new(SeriesExpr::DataField {
                            data_id: "btc_1d".into(),
                            field: SeriesField::Close,
                        }),
                        window_size: 20,
                        agg: SeriesAggregation::Mean,
                    },
                }),
                op: ComparisonOp::Gt,
                right: Box::new(ScalarExpr::Series {
                    expr: SeriesExpr::WindowAgg {
                        input: Box::new(SeriesExpr::DataField {
                            data_id: "btc_1d".into(),
                            field: SeriesField::Close,
                        }),
                        window_size: 50,
                        agg: SeriesAggregation::Mean,
                    },
                }),
            }
        );
    }

    #[test]
    fn strategy_ir_zscore_rule_lowers_structured_threshold_condition_for_one_sided_shape() {
        let mut strategy_ir = sample_strategy_ir_with_indicator(IndicatorKind::ZScore);
        strategy_ir.signals[0].signal_id = "zscore_signal".into();
        strategy_ir.signals[0].name = "ZScore Signal".into();
        strategy_ir.signals[0].indicator.params = BTreeMap::from([("window".into(), json!(20))]);
        strategy_ir.logic.entry_rules[0].condition = "zscore_signal < -2".into();
        strategy_ir.logic.entry_rules[0].action = LogicAction::OpenLong;

        let compiled = lower_strategy_ir_to_core_ir(&strategy_ir).unwrap();
        let condition = &compiled.signal_rules[0].condition;

        assert_eq!(
            condition,
            &ScalarExpr::Compare {
                left: Box::new(ScalarExpr::Ref {
                    name: "zscore_signal".into(),
                }),
                op: ComparisonOp::Lt,
                right: Box::new(ScalarExpr::Number { value: -2.0 }),
            }
        );
    }

    #[test]
    fn strategy_ir_spread_rule_lowers_structured_threshold_condition_for_one_sided_bps_shape() {
        let mut strategy_ir = sample_strategy_ir_with_indicator(IndicatorKind::Spread);
        strategy_ir.signals[0].signal_id = "spread_signal".into();
        strategy_ir.signals[0].name = "Spread Signal".into();
        strategy_ir.signals[0].indicator.inputs =
            vec!["binance_btc_quote".into(), "okx_btc_quote".into()];
        strategy_ir.signals[0].indicator.params = BTreeMap::from([
            ("align_direction_code".into(), json!(0)),
            ("max_time_diff_ms".into(), json!(5_000)),
            ("spread_output_code".into(), json!(1)),
        ]);
        strategy_ir.logic.entry_rules[0].condition = "spread_signal > 5".into();
        strategy_ir.logic.entry_rules[0].action = LogicAction::OpenLong;
        strategy_ir.data_requirements = vec![
            DataRequirement {
                data_id: "binance_btc_quote".into(),
                venue: KnownOrUnknown::Known("binance".into()),
                symbol: KnownOrUnknown::Known("BTCUSDT".into()),
                data_type: DataRequirementType::Quote,
                granularity: KnownOrUnknown::Known("1m".into()),
                lookback: KnownOrUnknown::Known(200),
                fields: vec!["bid".into(), "ask".into(), "mid".into()],
            },
            DataRequirement {
                data_id: "okx_btc_quote".into(),
                venue: KnownOrUnknown::Known("okx".into()),
                symbol: KnownOrUnknown::Known("BTCUSDT".into()),
                data_type: DataRequirementType::Quote,
                granularity: KnownOrUnknown::Known("1m".into()),
                lookback: KnownOrUnknown::Known(200),
                fields: vec!["bid".into(), "ask".into(), "mid".into()],
            },
        ];

        let compiled = lower_strategy_ir_to_core_ir(&strategy_ir).unwrap();
        let condition = &compiled.signal_rules[0].condition;

        assert_eq!(
            condition,
            &ScalarExpr::Compare {
                left: Box::new(ScalarExpr::Ref {
                    name: "spread_signal".into(),
                }),
                op: ComparisonOp::Gt,
                right: Box::new(ScalarExpr::Number { value: 5.0 }),
            }
        );
    }

    #[test]
    fn strategy_ir_spread_rule_rejects_non_bps_output() {
        let mut strategy_ir = sample_strategy_ir_with_indicator(IndicatorKind::Spread);
        strategy_ir.signals[0].signal_id = "spread_signal".into();
        strategy_ir.signals[0].indicator.inputs =
            vec!["binance_btc_quote".into(), "okx_btc_quote".into()];
        strategy_ir.signals[0].indicator.params = BTreeMap::from([
            ("align_direction_code".into(), json!(0)),
            ("max_time_diff_ms".into(), json!(5_000)),
            ("spread_output_code".into(), json!(0)),
        ]);
        strategy_ir.logic.entry_rules[0].condition = "spread_signal > 5".into();
        strategy_ir.logic.entry_rules[0].action = LogicAction::OpenLong;
        strategy_ir.data_requirements = vec![
            DataRequirement {
                data_id: "binance_btc_quote".into(),
                venue: KnownOrUnknown::Known("binance".into()),
                symbol: KnownOrUnknown::Known("BTCUSDT".into()),
                data_type: DataRequirementType::Quote,
                granularity: KnownOrUnknown::Known("1m".into()),
                lookback: KnownOrUnknown::Known(200),
                fields: vec!["bid".into(), "ask".into(), "mid".into()],
            },
            DataRequirement {
                data_id: "okx_btc_quote".into(),
                venue: KnownOrUnknown::Known("okx".into()),
                symbol: KnownOrUnknown::Known("BTCUSDT".into()),
                data_type: DataRequirementType::Quote,
                granularity: KnownOrUnknown::Known("1m".into()),
                lookback: KnownOrUnknown::Known(200),
                fields: vec!["bid".into(), "ask".into(), "mid".into()],
            },
        ];

        let error = lower_strategy_ir_to_core_ir(&strategy_ir).unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with("QPSTRATSPREAD001 信号 `spread_signal`"),
            "{}",
            error
        );
    }

    #[test]
    fn runtime_protocol_zscore_intent_lowers_structured_threshold_condition_for_one_sided_shape() {
        let mut config = sample_config();
        config.intents.push(IntentConfig {
            intent_id: "intent_zscore".into(),
            name: "ZScore".into(),
            kind: IntentKind::ZScore,
            input_data_ids: vec!["binance_btc_150d_1d".into()],
            params: BTreeMap::from([
                ("window".into(), 20.0),
                ("entry_z".into(), 2.0),
                ("comparison_shape_code".into(), 1.0),
                ("comparison_op_code".into(), 0.0),
                ("comparison_threshold".into(), -2.0),
            ]),
            enabled: true,
        });

        let compiled = compile_runtime_protocol_config(&config).unwrap();
        let condition = &compiled.core_ir.signal_rules[3].condition;

        assert_eq!(
            condition,
            &ScalarExpr::Compare {
                left: Box::new(ScalarExpr::Ref {
                    name: "intent_zscore".into(),
                }),
                op: ComparisonOp::Lt,
                right: Box::new(ScalarExpr::Number { value: -2.0 }),
            }
        );
    }
}
