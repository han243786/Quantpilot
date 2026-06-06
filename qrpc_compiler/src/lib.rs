mod runtime_protocol_lowering;
mod runtime_protocol_validation;
mod strategy_ir_lowering;

use anyhow::{bail, Result};
use qrpc_core::{
    CompiledRuntimeProtocol, IntentConfig, IntentKind, RuntimeProtocolCoreConfig, StrategyIr,
};
use qrpc_core_ir::{
    AlignAsofSpec, AlignDirection, ComparisonOp, CoreMetadata,
    CoreSourceKind, CoreStrategyIr,
    RebalanceSchedule as CoreRebalanceSchedule, SeriesAggregation,
    SeriesExpr, SeriesField, SpreadSpec, SpreadValueKind,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

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
    runtime_protocol_lowering::lower_runtime_protocol_to_core_ir_with_metadata(config, metadata)
}

pub fn lower_strategy_ir_to_core_ir(strategy_ir: &StrategyIr) -> Result<CoreStrategyIr> {
    strategy_ir_lowering::lower_strategy_ir_to_core_ir(strategy_ir)
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

fn lower_rebalance_schedule(value: &str) -> Result<CoreRebalanceSchedule> {
    match value {
        "slow" => Ok(CoreRebalanceSchedule::EverySlow),
        "1d" => Ok(CoreRebalanceSchedule::Every1d),
        "weekly" => Ok(CoreRebalanceSchedule::Weekly),
        other => bail!("不支持的重新平衡计划: {other}"),
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
        AgentConfig, DataKind, DataRequirement, DataRequirementType, DataSourceConfig, Exchange,
        IndicatorDefinition, IndicatorKind, KnownOrUnknown, LogicAction, LogicRule, MarketType,
        PositionSizing, PositionSizingMethod, PositionSizingUnit,
        RebalanceSchedule as RuntimeRebalanceSchedule, RiskConfig, SignalDefinition,
        StrategyExecution, StrategyIr, StrategyLogic, StrategyMetadata, StrategyRiskRules,
        StrategySource, StrategySourceType, Symbol,
    };
    use qrpc_core_ir::{
        AgentPolicyKind, CoreIndicatorKind, ScalarExpr, CUSTOM_EXPR_V1_VERSION,
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
