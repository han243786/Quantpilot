use crate::script::{BinaryOp, CallArg, Expr, FunctionDecl};
use anyhow::{anyhow, bail, Result};
use qrpc_core::{DataSourceConfig, IntentConfig, IntentKind, RebalanceSchedule};
use qrpc_core_ir::{moving_average_compare_expr, ComparisonOp};
use std::collections::{BTreeMap, BTreeSet};

use super::binding_sources::{comparison_parts, format_symbol, parse_symbol_lossy};
use super::bindings::{resolve_indicator_binding, BindingEnv, IndicatorBinding};
use super::context::PortfolioRebalanceDirective;
use super::shared::{arg_string_optional, expr_number, sanitize_id, ArgSelector};

const ERR_UNSUPPORTED_CONDITIONAL_EMIT: &str =
    "QPQSLOW001 不支持的条件下发 Intent 编译: 条件必须映射到支持的指标或价差意图";
const ERR_NO_EXECUTABLE_INTENTS: &str = "QPQSLOW002 无法从策略中编译出可执行的 emit Intent(...)";
const ERR_EMIT_REQUIRES_DATA_SOURCE: &str = "QPQSLOW003 emit Intent 需要至少一个数据源";
const ERR_EMIT_REQUIRES_ACTION: &str = "QPQSLOW005 emit Intent 需要指定动作";
const STRUCTURED_COMPARISON_SHAPE_KEY: &str = "comparison_shape_code";
const STRUCTURED_COMPARISON_OP_KEY: &str = "comparison_op_code";
const STRUCTURED_COMPARISON_THRESHOLD_KEY: &str = "comparison_threshold";

mod intent_collection_orchestration;
mod single_indicator_intent_inference;
mod spread_intent_inference;

pub(crate) fn canonicalize_data_sources(
    inferred: &[DataSourceConfig],
    bindings: &BindingEnv,
) -> Vec<DataSourceConfig> {
    let mut out = Vec::new();
    let mut seen_ids = BTreeSet::new();
    let bound_sources = bindings.data_by_name.values().cloned().collect::<Vec<_>>();
    let bound_signatures = bound_sources
        .iter()
        .map(data_source_signature)
        .collect::<BTreeSet<_>>();

    for source in bound_sources {
        if seen_ids.insert(source.data_id.clone()) {
            out.push(source);
        }
    }

    for source in inferred {
        if bound_signatures.contains(&data_source_signature(source)) {
            continue;
        }
        if seen_ids.insert(source.data_id.clone()) {
            out.push(source.clone());
        }
    }

    out
}

fn canonicalize_data_source_for_bindings(
    source: &DataSourceConfig,
    bindings: &BindingEnv,
) -> DataSourceConfig {
    let signature = data_source_signature(source);
    bindings
        .data_by_name
        .values()
        .find(|candidate| data_source_signature(candidate) == signature)
        .cloned()
        .unwrap_or_else(|| source.clone())
}

pub(crate) fn inferred_agent_params(
    intents: &[IntentConfig],
    rebalance: Option<&PortfolioRebalanceDirective>,
) -> BTreeMap<String, f64> {
    let mut params = BTreeMap::new();
    let spread_trigger_bps = intents
        .iter()
        .filter_map(|intent| {
            matches!(intent.kind, IntentKind::QuoteObserve)
                .then(|| intent.params.get("spread_trigger_bps").copied())
                .flatten()
        })
        .reduce(f64::min);
    if let Some(spread_trigger_bps) = spread_trigger_bps {
        params.insert("spread_trigger_bps".into(), spread_trigger_bps);
    }
    if let Some(rebalance) = rebalance {
        params.insert("portfolio_rebalance".into(), 1.0);
        params.insert("max_quantity_ratio".into(), 1.0);
        params.insert(
            "portfolio_rebalance_symbol_count".into(),
            rebalance.symbols.len() as f64,
        );
        let schedule_code = match rebalance.schedule {
            Some(RebalanceSchedule::EverySlow) | None => 0.0,
            Some(RebalanceSchedule::Every1d) => 1.0,
            Some(RebalanceSchedule::Weekly) => 7.0,
        };
        params.insert("rebalance_schedule_code".into(), schedule_code);
        if matches!(rebalance.schedule, Some(RebalanceSchedule::Every1d)) {
            params.insert("rebalance_every_days".into(), 1.0);
        }
    }
    params
}

fn data_source_signature(source: &DataSourceConfig) -> String {
    format!(
        "{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
        source.exchange,
        source.symbol,
        source.market_type,
        source.kind,
        source.days,
        source.interval
    )
}

pub(crate) fn infer_intents(
    strategy: &FunctionDecl,
    bindings: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Vec<IntentConfig>> {
    intent_collection_orchestration::infer_intents(strategy, bindings, data_sources)
}

fn intents_from_condition(
    condition: &Expr,
    action: &str,
    bindings: &BindingEnv,
) -> Result<Vec<IntentConfig>> {
    let Some((left, op, right)) = comparison_parts(condition) else {
        return Ok(Vec::new());
    };
    let runtime_id_hint = intent_runtime_id_hint(left, right, bindings);

    if let Some(intent) = moving_average_ratio_intent(
        left,
        op.clone(),
        right,
        action,
        bindings,
        runtime_id_hint.as_deref(),
    )? {
        return Ok(vec![intent]);
    }
    if let Some(intent) = moving_average_ratio_intent(
        right,
        normalize_relation(op.clone(), false),
        left,
        action,
        bindings,
        runtime_id_hint.as_deref(),
    )? {
        return Ok(vec![intent]);
    }

    if let Some(intent) = spread_intent_from_condition(
        left,
        op.clone(),
        right,
        bindings,
        runtime_id_hint.as_deref(),
    )? {
        return Ok(vec![intent]);
    }
    if let Some(intent) = spread_intent_from_condition(
        right,
        normalize_relation(op.clone(), false),
        left,
        bindings,
        runtime_id_hint.as_deref(),
    )? {
        return Ok(vec![intent]);
    }

    if let (Some(left_ma), Some(right_ma)) = (
        resolve_indicator_binding(left, bindings, &[])?,
        resolve_indicator_binding(right, bindings, &[])?,
    ) {
        if let (
            IndicatorBinding::MovingAverage {
                source: left_source,
                period: left_period,
                ..
            },
            IndicatorBinding::MovingAverage {
                source: right_source,
                period: right_period,
                ..
            },
        ) = (left_ma, right_ma)
        {
            let left_source = canonicalize_data_source_for_bindings(&left_source, bindings);
            let right_source = canonicalize_data_source_for_bindings(&right_source, bindings);
            if left_source.data_id == right_source.data_id
                && matches!(
                    op,
                    BinaryOp::Greater
                        | BinaryOp::GreaterEqual
                        | BinaryOp::Less
                        | BinaryOp::LessEqual
                )
            {
                let Some(comparison_op) = comparison_op_from_binary_relation(&op) else {
                    return Ok(Vec::new());
                };
                if moving_average_compare_expr(
                    left_source.data_id.clone(),
                    left_period,
                    comparison_op,
                    right_period,
                )
                .is_none()
                {
                    return Ok(Vec::new());
                }
                let fast_period = left_period.min(right_period) as f64;
                let slow_period = left_period.max(right_period) as f64;
                let instrument = format_symbol(&left_source.symbol);
                let intent = if action == "BUY" {
                    IntentConfig {
                        intent_id: format!("intent_{}_ma_entry", sanitize_id(instrument)),
                        name: format!("{instrument} MA Entry"),
                        kind: IntentKind::SmaCrossover,
                        input_data_ids: vec![left_source.data_id.clone()],
                        params: BTreeMap::from([
                            ("fast_period".into(), fast_period),
                            ("slow_period".into(), slow_period),
                            ("entry_ratio".into(), 0.2),
                            (
                                "comparison_op_code".into(),
                                comparison_op_code(comparison_op),
                            ),
                        ]),
                        enabled: true,
                    }
                } else if action == "SELL" {
                    IntentConfig {
                        intent_id: format!("intent_{}_ma_exit", sanitize_id(instrument)),
                        name: format!("{instrument} MA Exit"),
                        kind: IntentKind::LongTermSell,
                        input_data_ids: vec![left_source.data_id.clone()],
                        params: BTreeMap::from([
                            ("lookback".into(), fast_period),
                            ("baseline_period".into(), slow_period),
                            ("threshold_ratio".into(), 1.0),
                            (
                                "comparison_op_code".into(),
                                comparison_op_code(comparison_op),
                            ),
                        ]),
                        enabled: true,
                    }
                } else {
                    return Ok(Vec::new());
                };
                return Ok(vec![intent]);
            }
        }
    }

    let left_indicator = resolve_indicator_binding(left, bindings, &[])?;
    let right_indicator = resolve_indicator_binding(right, bindings, &[])?;
    let left_number = expr_number(left);
    let right_number = expr_number(right);

    match (left_indicator, right_indicator, left_number, right_number) {
        (Some(indicator), None, None, Some(threshold)) => single_indicator_intent(
            indicator,
            true,
            op,
            threshold,
            action,
            bindings,
            runtime_id_hint.as_deref(),
        ),
        (None, Some(indicator), Some(threshold), None) => single_indicator_intent(
            indicator,
            false,
            op,
            threshold,
            action,
            bindings,
            runtime_id_hint.as_deref(),
        ),
        _ => Ok(Vec::new()),
    }
}

fn intent_runtime_id_hint(left: &Expr, right: &Expr, env: &BindingEnv) -> Option<String> {
    signal_alias_name(left, env)
        .or_else(|| signal_alias_name(right, env))
        .and_then(intent_runtime_id_from_signal_binding)
}

fn signal_alias_name<'a>(expr: &'a Expr, env: &'a BindingEnv) -> Option<&'a str> {
    match expr {
        Expr::Identifier(name) => Some(name.as_str()),
        Expr::Try(inner) | Expr::Await(inner) => signal_alias_name(inner, env),
        _ => env
            .expr_by_name
            .iter()
            .find_map(|(name, candidate)| (candidate == expr).then_some(name.as_str())),
    }
}

fn single_indicator_intent(
    indicator: IndicatorBinding,
    indicator_on_left: bool,
    op: BinaryOp,
    threshold: f64,
    action: &str,
    bindings: &BindingEnv,
    runtime_id_hint: Option<&str>,
) -> Result<Vec<IntentConfig>> {
    single_indicator_intent_inference::single_indicator_intent(
        indicator,
        indicator_on_left,
        op,
        threshold,
        action,
        bindings,
        runtime_id_hint,
    )
}

fn spread_intent_from_condition(
    indicator_expr: &Expr,
    relation: BinaryOp,
    threshold_expr: &Expr,
    bindings: &BindingEnv,
    runtime_id_hint: Option<&str>,
) -> Result<Option<IntentConfig>> {
    spread_intent_inference::spread_intent_from_condition(
        indicator_expr,
        relation,
        threshold_expr,
        bindings,
        runtime_id_hint,
    )
}

fn moving_average_ratio_intent(
    indicator_expr: &Expr,
    relation: BinaryOp,
    threshold_expr: &Expr,
    action: &str,
    bindings: &BindingEnv,
    runtime_id_hint: Option<&str>,
) -> Result<Option<IntentConfig>> {
    let Some((source, fast_period, slow_period, ratio_scale)) =
        moving_average_ratio_binding(indicator_expr, bindings)?
    else {
        return Ok(None);
    };
    let source = canonicalize_data_source_for_bindings(&source, bindings);
    let Some(threshold) = expr_number(threshold_expr) else {
        return Ok(None);
    };
    if !matches!(
        (action, relation),
        ("BUY", BinaryOp::Greater | BinaryOp::GreaterEqual)
            | ("SELL", BinaryOp::Greater | BinaryOp::GreaterEqual)
    ) {
        return Ok(None);
    }

    let instrument = format_symbol(&source.symbol);
    let ratio = if ratio_scale {
        1.0 + threshold
    } else {
        threshold
    };
    if !ratio.is_finite() || ratio <= 0.0 {
        return Ok(None);
    }

    let intent = if action == "BUY" {
        IntentConfig {
            intent_id: runtime_id_hint
                .map(str::to_string)
                .unwrap_or_else(|| format!("intent_{}_ma_entry", sanitize_id(instrument))),
            name: format!("{instrument} MA Entry"),
            kind: IntentKind::LongTermBuy,
            input_data_ids: vec![source.data_id.clone()],
            params: BTreeMap::from([
                ("fast_period".into(), fast_period as f64),
                ("slow_period".into(), slow_period as f64),
                ("entry_ratio".into(), ratio),
            ]),
            enabled: true,
        }
    } else {
        IntentConfig {
            intent_id: runtime_id_hint
                .map(str::to_string)
                .unwrap_or_else(|| format!("intent_{}_ma_exit", sanitize_id(instrument))),
            name: format!("{instrument} MA Exit"),
            kind: IntentKind::LongTermSell,
            input_data_ids: vec![source.data_id.clone()],
            params: BTreeMap::from([
                ("lookback".into(), fast_period as f64),
                ("baseline_period".into(), slow_period as f64),
                ("threshold_ratio".into(), ratio),
            ]),
            enabled: true,
        }
    };

    Ok(Some(intent))
}

fn moving_average_ratio_binding(
    expr: &Expr,
    bindings: &BindingEnv,
) -> Result<Option<(DataSourceConfig, usize, usize, bool)>> {
    match expr {
        Expr::Binary {
            left,
            op: BinaryOp::Divide,
            right,
        } => {
            if let Some(binding) = moving_average_ratio_from_division(left, right, bindings, false)?
            {
                return Ok(Some(binding));
            }
            if let Expr::Binary {
                left: numerator_left,
                op: BinaryOp::Subtract,
                right: numerator_right,
            } = left.as_ref()
            {
                if let Some(binding) = moving_average_ratio_from_subtract_division(
                    numerator_left,
                    numerator_right,
                    right,
                    bindings,
                )? {
                    return Ok(Some(binding));
                }
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}

fn moving_average_ratio_from_division(
    left: &Expr,
    right: &Expr,
    bindings: &BindingEnv,
    ratio_scale: bool,
) -> Result<Option<(DataSourceConfig, usize, usize, bool)>> {
    let Some(left_binding) = resolve_indicator_binding(left, bindings, &[])? else {
        return Ok(None);
    };
    let Some(right_binding) = resolve_indicator_binding(right, bindings, &[])? else {
        return Ok(None);
    };
    let (
        IndicatorBinding::MovingAverage {
            source: left_source,
            period: left_period,
            ..
        },
        IndicatorBinding::MovingAverage {
            source: right_source,
            period: right_period,
            ..
        },
    ) = (left_binding, right_binding)
    else {
        return Ok(None);
    };

    if left_source.data_id != right_source.data_id {
        return Ok(None);
    }

    Ok(Some((
        left_source,
        left_period.min(right_period),
        left_period.max(right_period),
        ratio_scale,
    )))
}

fn moving_average_ratio_from_subtract_division(
    numerator_left: &Expr,
    numerator_right: &Expr,
    denominator: &Expr,
    bindings: &BindingEnv,
) -> Result<Option<(DataSourceConfig, usize, usize, bool)>> {
    let Some((source, fast_period, slow_period, _)) =
        moving_average_ratio_from_division(numerator_left, denominator, bindings, true)?
    else {
        return Ok(None);
    };
    let Some(numerator_binding) = resolve_indicator_binding(numerator_right, bindings, &[])? else {
        return Ok(None);
    };
    let IndicatorBinding::MovingAverage {
        source: numerator_source,
        period: numerator_period,
        ..
    } = numerator_binding
    else {
        return Ok(None);
    };
    if numerator_source.data_id != source.data_id || numerator_period != slow_period {
        return Ok(None);
    }
    Ok(Some((source, fast_period, slow_period, true)))
}

fn merge_intent(intents: &mut BTreeMap<String, IntentConfig>, next: IntentConfig) {
    match intents.get_mut(&next.intent_id) {
        Some(existing) => {
            let existing_shape = existing
                .params
                .get(STRUCTURED_COMPARISON_SHAPE_KEY)
                .copied();
            let next_shape = next.params.get(STRUCTURED_COMPARISON_SHAPE_KEY).copied();
            existing.params.extend(next.params);
            if let (Some(existing_shape), Some(next_shape)) = (existing_shape, next_shape) {
                if (existing_shape - next_shape).abs() > f64::EPSILON {
                    existing.params.remove(STRUCTURED_COMPARISON_SHAPE_KEY);
                    existing.params.remove(STRUCTURED_COMPARISON_OP_KEY);
                    existing.params.remove(STRUCTURED_COMPARISON_THRESHOLD_KEY);
                }
            }
        }
        None => {
            intents.insert(next.intent_id.clone(), next);
        }
    }
}

fn legacy_intent_from_emit(
    args: &[CallArg],
    data_sources: &[DataSourceConfig],
) -> Result<IntentConfig> {
    let action = emit_action(args)?;
    let instrument = emit_instrument(args);
    let matching_source = data_sources
        .iter()
        .find(|source| source.symbol == parse_symbol_lossy(&instrument))
        .or_else(|| data_sources.first())
        .ok_or_else(|| anyhow!(ERR_EMIT_REQUIRES_DATA_SOURCE))?;

    let (kind, id, name) = match action.as_str() {
        "BUY" => (
            IntentKind::LongTermBuy,
            format!("intent_{}_buy", sanitize_id(&instrument)),
            format!("{instrument} Buy"),
        ),
        "SELL" => (
            IntentKind::LongTermSell,
            format!("intent_{}_sell", sanitize_id(&instrument)),
            format!("{instrument} Sell"),
        ),
        other => bail!(
            "QPQSLOW004 不支持的 Intent 动作 '{other}'。emit Intent(...) 的有效动作: BUY, SELL"
        ),
    };

    Ok(IntentConfig {
        intent_id: id,
        name,
        kind,
        input_data_ids: vec![matching_source.data_id.clone()],
        params: BTreeMap::new(),
        enabled: true,
    })
}

fn emit_action(args: &[CallArg]) -> Result<String> {
    arg_string_optional(args, ArgSelector::NamedOrPositional("action", 0))
        .map(|value| value.to_ascii_uppercase())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!(ERR_EMIT_REQUIRES_ACTION))
}

fn emit_instrument(args: &[CallArg]) -> String {
    arg_string_optional(args, ArgSelector::Named("instrument"))
        .unwrap_or_else(|| "BTCUSDT".to_string())
}

fn invert_condition(expr: &Expr) -> Option<Expr> {
    let (left, op, right) = comparison_parts(expr)?;
    let inverted = match op {
        BinaryOp::Greater => BinaryOp::LessEqual,
        BinaryOp::GreaterEqual => BinaryOp::Less,
        BinaryOp::Less => BinaryOp::GreaterEqual,
        BinaryOp::LessEqual => BinaryOp::Greater,
        BinaryOp::Equal => BinaryOp::NotEqual,
        BinaryOp::NotEqual => BinaryOp::Equal,
        _ => return None,
    };
    Some(Expr::Binary {
        left: Box::new(left.clone()),
        op: inverted,
        right: Box::new(right.clone()),
    })
}

fn normalize_relation(op: BinaryOp, indicator_on_left: bool) -> BinaryOp {
    if indicator_on_left {
        op
    } else {
        match op {
            BinaryOp::Greater => BinaryOp::Less,
            BinaryOp::GreaterEqual => BinaryOp::LessEqual,
            BinaryOp::Less => BinaryOp::Greater,
            BinaryOp::LessEqual => BinaryOp::GreaterEqual,
            other => other,
        }
    }
}

fn comparison_op_from_binary_relation(op: &BinaryOp) -> Option<ComparisonOp> {
    match op {
        BinaryOp::Greater => Some(ComparisonOp::Gt),
        BinaryOp::GreaterEqual => Some(ComparisonOp::Gte),
        BinaryOp::Less => Some(ComparisonOp::Lt),
        BinaryOp::LessEqual => Some(ComparisonOp::Lte),
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

fn comparison_shape_code(action: &str) -> Option<f64> {
    match action {
        "BUY" => Some(1.0),
        "SELL" => Some(2.0),
        _ => None,
    }
}

fn intent_runtime_id_from_signal_binding(name: &str) -> Option<String> {
    let sanitized = sanitize_id(name);
    let stripped = sanitized.strip_suffix("_signal")?;
    if stripped.is_empty() || !stripped.starts_with("intent_") {
        None
    } else {
        Some(stripped.to_string())
    }
}
