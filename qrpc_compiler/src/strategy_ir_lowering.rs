use anyhow::{anyhow, bail, Result};
use qrpc_core::StrategyIr;
use qrpc_core::{
    PAPER_EXECUTION_PROFILE_DEFAULT_FEE_BPS,
    GLOBAL_RISK_PROFILE_DEFAULT_MAX_EXCHANGE_LEVERAGE,
    GLOBAL_RISK_PROFILE_DEFAULT_MAX_TOTAL_LEVERAGE,
    GLOBAL_RISK_PROFILE_DEFAULT_MIN_ACTION_INTERVAL_MS,
};
use qrpc_core_ir::{
    indicator_threshold_compare_expr, moving_average_compare_expr, AgentPolicy, AgentPolicyKind,
    ArithmeticOp, ComparisonOp, CoreIndicatorKind, CoreMetadata, CoreSourceKind, CoreStrategyIr,
    CoreTimeInForce, CustomExprSpec, CustomValueExpr, DataBinding, DataBindingKind, ExecutionRule,
    ExecutionSizingKind, IndicatorNode, RiskPolicy, ScalarExpr, SeriesExpr, SeriesField, SignalKind,
    SignalRule, SpreadSpec, SpreadValueKind, CUSTOM_EXPR_V1_VERSION,
};
use std::collections::{BTreeMap, BTreeSet};

use super::{
    build_spread_spec, lower_rebalance_schedule, parse_strategy_signal_compare,
    strategy_indicator_usize_param,
};


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