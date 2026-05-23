use crate::resolve::ResolvedWindowAggregateKind;
use crate::script::{BinaryOp, CallArg, Expr, FunctionDecl, MatchArmBody, Stmt};
use anyhow::{anyhow, bail, Result};
use qrpc_core::{DataSourceConfig, IntentConfig, IntentKind, RebalanceSchedule};
use qrpc_core_ir::{indicator_threshold_compare_expr, moving_average_compare_expr, ComparisonOp};
use std::collections::{BTreeMap, BTreeSet};

use super::binding_sources::{
    comparison_parts, decode_series_position_view, decode_window_binding, format_symbol,
    parse_call, parse_symbol_lossy, resolve_data_source_ref, SeriesViewAccess,
};
use super::bindings::{resolve_indicator_binding, BindingEnv, IndicatorBinding};
use super::context::PortfolioRebalanceDirective;
use super::semantic::{resolve_expr_alias, rsi_method_code};
use super::shared::{
    arg_number_optional, arg_string_optional, expr_number, find_arg, sanitize_id, ArgSelector,
};

const ERR_UNSUPPORTED_CONDITIONAL_EMIT: &str =
    "QPQSLOW001 不支持的条件下发 Intent 编译: 条件必须映射到支持的指标或价差意图";
const ERR_NO_EXECUTABLE_INTENTS: &str = "QPQSLOW002 无法从策略中编译出可执行的 emit Intent(...)";
const ERR_EMIT_REQUIRES_DATA_SOURCE: &str = "QPQSLOW003 emit Intent 需要至少一个数据源";
const ERR_EMIT_REQUIRES_ACTION: &str = "QPQSLOW005 emit Intent 需要指定动作";
const STRUCTURED_COMPARISON_SHAPE_KEY: &str = "comparison_shape_code";
const STRUCTURED_COMPARISON_OP_KEY: &str = "comparison_op_code";
const STRUCTURED_COMPARISON_THRESHOLD_KEY: &str = "comparison_threshold";

#[derive(Debug, Clone)]
struct SpreadSeriesOperand {
    source: DataSourceConfig,
    field_code: f64,
    resample_period_ms: Option<u64>,
    resample_agg_code: Option<f64>,
    window_size: usize,
    window_agg_code: Option<f64>,
    align_direction_code: Option<f64>,
    tolerance_ms: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpreadOutputKind {
    Ratio,
    Bps,
    Absolute,
}

#[derive(Debug, Clone)]
struct SpreadMatch {
    left: SpreadSeriesOperand,
    right: SpreadSeriesOperand,
    align_direction_code: Option<f64>,
    tolerance_ms: Option<f64>,
}

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
    let mut intents = BTreeMap::<String, IntentConfig>::new();
    collect_intents_from_stmts(&strategy.body, None, bindings, data_sources, &mut intents)?;
    if intents.is_empty() {
        bail!(ERR_NO_EXECUTABLE_INTENTS);
    }
    Ok(intents.into_values().collect())
}

fn collect_intents_from_stmts(
    stmts: &[Stmt],
    active_condition: Option<&Expr>,
    bindings: &BindingEnv,
    data_sources: &[DataSourceConfig],
    intents: &mut BTreeMap<String, IntentConfig>,
) -> Result<()> {
    for stmt in stmts {
        match stmt {
            Stmt::EmitIntent { args } => {
                let action = emit_action(args)?;
                let inferred = if let Some(condition) = active_condition {
                    intents_from_condition(condition, &action, bindings)?
                } else {
                    Vec::new()
                };
                if inferred.is_empty() {
                    if active_condition.is_some() {
                        bail!(ERR_UNSUPPORTED_CONDITIONAL_EMIT);
                    }
                    merge_intent(intents, legacy_intent_from_emit(args, data_sources)?);
                } else {
                    for intent in inferred {
                        merge_intent(intents, intent);
                    }
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                collect_intents_from_stmts(
                    then_branch,
                    Some(condition),
                    bindings,
                    data_sources,
                    intents,
                )?;
                for (condition, branch) in else_if_branches {
                    collect_intents_from_stmts(
                        branch,
                        Some(condition),
                        bindings,
                        data_sources,
                        intents,
                    )?;
                }
                if let Some(branch) = else_branch {
                    let else_condition = if else_if_branches.is_empty() {
                        invert_condition(condition)
                    } else {
                        None
                    };
                    collect_intents_from_stmts(
                        branch,
                        else_condition.as_ref(),
                        bindings,
                        data_sources,
                        intents,
                    )?;
                }
            }
            Stmt::For { body, .. } | Stmt::While { body, .. } => {
                collect_intents_from_stmts(
                    body,
                    active_condition,
                    bindings,
                    data_sources,
                    intents,
                )?;
            }
            Stmt::Match { arms, .. } => {
                for arm in arms {
                    if let MatchArmBody::Statement(stmt) = &arm.body {
                        collect_intents_from_stmts(
                            std::slice::from_ref(stmt.as_ref()),
                            active_condition,
                            bindings,
                            data_sources,
                            intents,
                        )?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
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
    let relation = normalize_relation(op, indicator_on_left);
    let absolute_threshold = threshold.abs();

    let intent = match indicator {
        IndicatorBinding::Rsi {
            source,
            period,
            method,
        } => {
            let source = canonicalize_data_source_for_bindings(&source, bindings);
            let instrument = format_symbol(&source.symbol);
            let comparison_op = match action {
                "BUY" if matches!(relation, BinaryOp::Less | BinaryOp::LessEqual) => {
                    comparison_op_from_binary_relation(&relation)
                }
                "SELL" if matches!(relation, BinaryOp::Greater | BinaryOp::GreaterEqual) => {
                    comparison_op_from_binary_relation(&relation)
                }
                _ => None,
            };
            let Some(comparison_op) = comparison_op else {
                return Ok(Vec::new());
            };
            let mut params = BTreeMap::from([
                ("period".into(), period as f64),
                ("smoothing_method".into(), rsi_method_code(method)),
                (
                    STRUCTURED_COMPARISON_OP_KEY.into(),
                    comparison_op_code(comparison_op),
                ),
                (
                    STRUCTURED_COMPARISON_SHAPE_KEY.into(),
                    comparison_shape_code(action).unwrap_or_default(),
                ),
            ]);
            match action {
                "BUY" if matches!(relation, BinaryOp::Less | BinaryOp::LessEqual) => {
                    if indicator_threshold_compare_expr(
                        runtime_id_hint
                            .map(str::to_string)
                            .unwrap_or_else(|| format!("intent_{}_rsi", sanitize_id(instrument))),
                        comparison_op,
                        threshold,
                    )
                    .is_none()
                    {
                        return Ok(Vec::new());
                    }
                    params.insert("oversold_threshold".into(), threshold);
                    params.insert("overbought_threshold".into(), 70.0);
                }
                "SELL" if matches!(relation, BinaryOp::Greater | BinaryOp::GreaterEqual) => {
                    if indicator_threshold_compare_expr(
                        runtime_id_hint
                            .map(str::to_string)
                            .unwrap_or_else(|| format!("intent_{}_rsi", sanitize_id(instrument))),
                        comparison_op,
                        threshold,
                    )
                    .is_none()
                    {
                        return Ok(Vec::new());
                    }
                    params.insert("oversold_threshold".into(), 30.0);
                    params.insert("overbought_threshold".into(), threshold);
                }
                _ => return Ok(Vec::new()),
            }
            IntentConfig {
                intent_id: runtime_id_hint
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("intent_{}_rsi", sanitize_id(instrument))),
                name: format!("{instrument} RSI"),
                kind: IntentKind::Rsi,
                input_data_ids: vec![source.data_id.clone()],
                params,
                enabled: true,
            }
        }
        IndicatorBinding::Macd {
            source,
            fast_period,
            slow_period,
            signal_period,
        } => {
            let source = canonicalize_data_source_for_bindings(&source, bindings);
            if !matches!(
                (action, relation),
                ("BUY", BinaryOp::Greater | BinaryOp::GreaterEqual)
                    | ("SELL", BinaryOp::Less | BinaryOp::LessEqual)
            ) {
                return Ok(Vec::new());
            }
            let instrument = format_symbol(&source.symbol);
            IntentConfig {
                intent_id: runtime_id_hint
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("intent_{}_macd", sanitize_id(instrument))),
                name: format!("{instrument} MACD"),
                kind: IntentKind::Macd,
                input_data_ids: vec![source.data_id.clone()],
                params: BTreeMap::from([
                    ("fast_period".into(), fast_period as f64),
                    ("slow_period".into(), slow_period as f64),
                    ("signal_period".into(), signal_period as f64),
                    ("histogram_threshold".into(), absolute_threshold),
                ]),
                enabled: true,
            }
        }
        IndicatorBinding::Momentum { source, lookback } => {
            let source = canonicalize_data_source_for_bindings(&source, bindings);
            let comparison_op = match action {
                "BUY" if matches!(relation, BinaryOp::Greater | BinaryOp::GreaterEqual) => {
                    comparison_op_from_binary_relation(&relation)
                }
                "SELL" if matches!(relation, BinaryOp::Less | BinaryOp::LessEqual) => {
                    comparison_op_from_binary_relation(&relation)
                }
                _ => None,
            };
            let Some(comparison_op) = comparison_op else {
                return Ok(Vec::new());
            };
            if !matches!(
                (action, relation),
                ("BUY", BinaryOp::Greater | BinaryOp::GreaterEqual)
                    | ("SELL", BinaryOp::Less | BinaryOp::LessEqual)
            ) {
                return Ok(Vec::new());
            }
            let instrument = format_symbol(&source.symbol);
            if indicator_threshold_compare_expr(
                runtime_id_hint
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("intent_{}_momentum", sanitize_id(instrument))),
                comparison_op,
                threshold,
            )
            .is_none()
            {
                return Ok(Vec::new());
            }
            IntentConfig {
                intent_id: runtime_id_hint
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("intent_{}_momentum", sanitize_id(instrument))),
                name: format!("{instrument} Momentum"),
                kind: IntentKind::Momentum,
                input_data_ids: vec![source.data_id.clone()],
                params: BTreeMap::from([
                    ("lookback".into(), lookback as f64),
                    ("threshold_ratio".into(), absolute_threshold),
                    (
                        STRUCTURED_COMPARISON_OP_KEY.into(),
                        comparison_op_code(comparison_op),
                    ),
                    (
                        STRUCTURED_COMPARISON_SHAPE_KEY.into(),
                        comparison_shape_code(action).unwrap_or_default(),
                    ),
                    (STRUCTURED_COMPARISON_THRESHOLD_KEY.into(), threshold),
                ]),
                enabled: true,
            }
        }
        IndicatorBinding::ZScore { source, window } => {
            let source = canonicalize_data_source_for_bindings(&source, bindings);
            let comparison_op = match action {
                "BUY" if matches!(relation, BinaryOp::Less | BinaryOp::LessEqual) => {
                    comparison_op_from_binary_relation(&relation)
                }
                "SELL" if matches!(relation, BinaryOp::Greater | BinaryOp::GreaterEqual) => {
                    comparison_op_from_binary_relation(&relation)
                }
                _ => None,
            };
            let Some(comparison_op) = comparison_op else {
                return Ok(Vec::new());
            };
            if !matches!(
                (action, relation),
                ("BUY", BinaryOp::Less | BinaryOp::LessEqual)
                    | ("SELL", BinaryOp::Greater | BinaryOp::GreaterEqual)
            ) {
                return Ok(Vec::new());
            }
            let instrument = format_symbol(&source.symbol);
            if indicator_threshold_compare_expr(
                runtime_id_hint
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("intent_{}_zscore", sanitize_id(instrument))),
                comparison_op,
                threshold,
            )
            .is_none()
            {
                return Ok(Vec::new());
            }
            IntentConfig {
                intent_id: runtime_id_hint
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("intent_{}_zscore", sanitize_id(instrument))),
                name: format!("{instrument} ZScore"),
                kind: IntentKind::ZScore,
                input_data_ids: vec![source.data_id.clone()],
                params: BTreeMap::from([
                    ("window".into(), window as f64),
                    ("entry_z".into(), absolute_threshold.max(0.1)),
                    (
                        STRUCTURED_COMPARISON_OP_KEY.into(),
                        comparison_op_code(comparison_op),
                    ),
                    (
                        STRUCTURED_COMPARISON_SHAPE_KEY.into(),
                        comparison_shape_code(action).unwrap_or_default(),
                    ),
                    (STRUCTURED_COMPARISON_THRESHOLD_KEY.into(), threshold),
                ]),
                enabled: true,
            }
        }
        IndicatorBinding::MovingAverage { .. }
        | IndicatorBinding::MacdLine { .. }
        | IndicatorBinding::MacdSignal { .. }
        | IndicatorBinding::Atr { .. }
        | IndicatorBinding::BollingerBands { .. }
        | IndicatorBinding::Obv { .. }
        | IndicatorBinding::Cmf { .. }
        | IndicatorBinding::Adx { .. }
        | IndicatorBinding::Stochastic { .. }
        | IndicatorBinding::Cci { .. }
        | IndicatorBinding::ParabolicSar { .. }
        | IndicatorBinding::KeltnerChannel { .. }
        | IndicatorBinding::DonchianChannel { .. } => return Ok(Vec::new()),
    };

    Ok(vec![intent])
}

fn spread_intent_from_condition(
    indicator_expr: &Expr,
    relation: BinaryOp,
    threshold_expr: &Expr,
    bindings: &BindingEnv,
    runtime_id_hint: Option<&str>,
) -> Result<Option<IntentConfig>> {
    if !matches!(relation, BinaryOp::Greater | BinaryOp::GreaterEqual) {
        return Ok(None);
    }

    let Some(spread) = match_formal_admitted_spread_expr(indicator_expr, bindings, &[])? else {
        return Ok(None);
    };
    let Some(threshold) = expr_number(threshold_expr) else {
        return Ok(None);
    };
    let left_source = canonicalize_data_source_for_bindings(&spread.left.source, bindings);
    let right_source = canonicalize_data_source_for_bindings(&spread.right.source, bindings);
    if left_source.data_id == right_source.data_id {
        return Ok(None);
    }
    if left_source.symbol != right_source.symbol {
        return Ok(None);
    }

    let instrument = format_symbol(&left_source.symbol);
    let mut params = spread_params(&spread, &left_source, &right_source);
    let Some(comparison_op) = comparison_op_from_binary_relation(&relation) else {
        return Ok(None);
    };
    params.insert("spread_output_code".into(), 1.0);
    params.insert("spread_trigger_bps".into(), threshold);
    params.insert(
        STRUCTURED_COMPARISON_SHAPE_KEY.into(),
        comparison_shape_code("BUY").unwrap_or_default(),
    );
    params.insert(
        STRUCTURED_COMPARISON_OP_KEY.into(),
        comparison_op_code(comparison_op),
    );
    params.insert(STRUCTURED_COMPARISON_THRESHOLD_KEY.into(), threshold);

    Ok(Some(IntentConfig {
        intent_id: runtime_id_hint
            .map(str::to_string)
            .unwrap_or_else(|| format!("intent_{}_spread", sanitize_id(instrument))),
        name: format!("{instrument} Spread Observe"),
        kind: IntentKind::QuoteObserve,
        input_data_ids: vec![left_source.data_id.clone(), right_source.data_id.clone()],
        params,
        enabled: true,
    }))
}

fn match_formal_admitted_spread_expr(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<SpreadMatch>> {
    let expr = resolve_expr_alias(expr, env).unwrap_or(expr);
    let Some((fn_name, args)) = parse_call(expr) else {
        return Ok(None);
    };
    if fn_name != "spread" {
        return Ok(None);
    }

    let Some(left_expr) = find_arg(args, ArgSelector::Positional(0)) else {
        return Ok(None);
    };
    let Some(right_expr) = find_arg(args, ArgSelector::Positional(1)) else {
        return Ok(None);
    };
    let Some(left) = decode_formal_admitted_spread_operand(left_expr, env, data_sources)? else {
        return Ok(None);
    };
    let Some(right) = decode_formal_admitted_spread_operand(right_expr, env, data_sources)? else {
        return Ok(None);
    };
    if !matches!(
        arg_string_optional(args, ArgSelector::NamedOrPositional("output", 2))
            .as_deref()
            .and_then(parse_spread_output_kind),
        Some(SpreadOutputKind::Bps)
    ) {
        return Ok(None);
    }
    let Some(Some(align_direction_code)) =
        merge_optional_f64(left.align_direction_code, right.align_direction_code)
    else {
        return Ok(None);
    };
    let Some(Some(tolerance_ms)) = merge_optional_f64(left.tolerance_ms, right.tolerance_ms) else {
        return Ok(None);
    };
    if !tolerance_ms.is_finite() || tolerance_ms <= 0.0 {
        return Ok(None);
    }

    Ok(Some(SpreadMatch {
        left,
        right,
        align_direction_code: Some(align_direction_code),
        tolerance_ms: Some(tolerance_ms),
    }))
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

fn decode_spread_operand(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<SpreadSeriesOperand>> {
    if let Some(explicit) = decode_explicit_spread_operand(expr, env, data_sources)? {
        return Ok(Some(explicit));
    }
    if let Some((source, access)) = decode_series_position_view(expr, env, data_sources)? {
        if matches!(access, SeriesViewAccess::Current) {
            return Ok(Some(default_spread_operand(source)));
        }
    }

    for (aggregate_kind, agg_code) in [
        (ResolvedWindowAggregateKind::Mean, 1.0),
        (ResolvedWindowAggregateKind::Sum, 2.0),
        (ResolvedWindowAggregateKind::StdDev, 5.0),
    ] {
        if let Some(binding) = decode_window_binding(expr, env, data_sources, aggregate_kind)? {
            let mut operand = default_spread_operand(binding.source);
            operand.window_size = binding.span;
            operand.window_agg_code = Some(agg_code);
            return Ok(Some(operand));
        }
    }

    Ok(None)
}

fn decode_spread_operand_or_source(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<SpreadSeriesOperand>> {
    let expr = resolve_expr_alias(expr, env).unwrap_or(expr);
    if let Some(source) = resolve_data_source_ref(expr, env, data_sources)? {
        return Ok(Some(default_spread_operand(source)));
    }
    decode_spread_operand(expr, env, data_sources)
}

fn decode_explicit_spread_operand(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<SpreadSeriesOperand>> {
    let expr = resolve_expr_alias(expr, env).unwrap_or(expr);
    let Some((fn_name, args)) = parse_call(expr) else {
        return Ok(None);
    };
    let mut operand = match fn_name.as_str() {
        "field" => {
            let Some(target) = find_arg(args, ArgSelector::Positional(0)) else {
                return Ok(None);
            };
            let Some(mut operand) = decode_spread_operand_or_source(target, env, data_sources)?
            else {
                return Ok(None);
            };
            let Some(field_name) =
                arg_string_optional(args, ArgSelector::NamedOrPositional("name", 1))
                    .or_else(|| arg_string_optional(args, ArgSelector::Named("field")))
            else {
                return Ok(None);
            };
            let Some(field_code) = parse_field_code(&field_name) else {
                return Ok(None);
            };
            operand.field_code = field_code;
            operand
        }
        "resample" => {
            let Some(target) = find_arg(args, ArgSelector::Positional(0)) else {
                return Ok(None);
            };
            let Some(mut operand) = decode_spread_operand_or_source(target, env, data_sources)?
            else {
                return Ok(None);
            };
            let Some(period_ms) = parse_resample_period_ms(args) else {
                return Ok(None);
            };
            operand.resample_period_ms = Some(period_ms);
            operand.resample_agg_code = Some(
                arg_string_optional(args, ArgSelector::NamedOrPositional("agg", 2))
                    .as_deref()
                    .and_then(parse_aggregation_code)
                    .unwrap_or(0.0),
            );
            operand
        }
        "align" | "align_asof" => {
            let Some(target) = find_arg(args, ArgSelector::Positional(0)) else {
                return Ok(None);
            };
            let Some(mut operand) = decode_spread_operand_or_source(target, env, data_sources)?
            else {
                return Ok(None);
            };
            operand.align_direction_code = Some(
                arg_string_optional(args, ArgSelector::NamedOrPositional("direction", 1))
                    .as_deref()
                    .and_then(parse_align_direction_code)
                    .unwrap_or(0.0),
            );
            operand.tolerance_ms =
                arg_number_optional(args, ArgSelector::NamedOrPositional("tolerance_ms", 2));
            operand
        }
        _ => return Ok(None),
    };

    if operand.window_size == 0 {
        operand.window_size = 1;
    }

    Ok(Some(operand))
}

fn decode_formal_admitted_spread_operand(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<SpreadSeriesOperand>> {
    let expr = resolve_expr_alias(expr, env).unwrap_or(expr);
    let Some((fn_name, args)) = parse_call(expr) else {
        return Ok(None);
    };
    if fn_name != "align_asof" {
        return Ok(None);
    }

    let Some(target) = find_arg(args, ArgSelector::Positional(0)) else {
        return Ok(None);
    };
    let Some(mut operand) = decode_spread_operand_or_source(target, env, data_sources)? else {
        return Ok(None);
    };
    let Some(direction_arg) =
        arg_string_optional(args, ArgSelector::NamedOrPositional("direction", 1))
    else {
        return Ok(None);
    };
    let Some(direction_code) = parse_align_direction_code(&direction_arg) else {
        return Ok(None);
    };
    if (direction_code - 0.0).abs() > f64::EPSILON {
        return Ok(None);
    }
    let Some(tolerance_ms) =
        arg_number_optional(args, ArgSelector::NamedOrPositional("tolerance_ms", 2))
    else {
        return Ok(None);
    };
    if !tolerance_ms.is_finite() || tolerance_ms <= 0.0 {
        return Ok(None);
    }
    operand.align_direction_code = Some(direction_code);
    operand.tolerance_ms = Some(tolerance_ms);
    Ok(Some(operand))
}

fn default_spread_operand(source: DataSourceConfig) -> SpreadSeriesOperand {
    SpreadSeriesOperand {
        source,
        field_code: 3.0,
        resample_period_ms: None,
        resample_agg_code: None,
        window_size: 1,
        window_agg_code: None,
        align_direction_code: None,
        tolerance_ms: None,
    }
}

fn parse_spread_output_kind(value: &str) -> Option<SpreadOutputKind> {
    match value.trim().to_ascii_lowercase().as_str() {
        "ratio" => Some(SpreadOutputKind::Ratio),
        "bps" | "basis_points" | "basis-points" => Some(SpreadOutputKind::Bps),
        "absolute" | "abs" | "value" => Some(SpreadOutputKind::Absolute),
        _ => None,
    }
}

fn parse_field_code(value: &str) -> Option<f64> {
    match value.trim().to_ascii_lowercase().as_str() {
        "mid" | "mid_or_close" | "mid-close" | "mid_or_last" => Some(0.0),
        "bid" | "bid_or_close" | "bid-close" => Some(1.0),
        "ask" | "ask_or_close" | "ask-close" => Some(2.0),
        "close" | "last" => Some(3.0),
        "open" => Some(4.0),
        "high" => Some(5.0),
        "low" => Some(6.0),
        "volume" => Some(7.0),
        _ => None,
    }
}

fn parse_aggregation_code(value: &str) -> Option<f64> {
    match value.trim().to_ascii_lowercase().as_str() {
        "last" => Some(0.0),
        "mean" | "avg" | "average" => Some(1.0),
        "sum" => Some(2.0),
        "min" => Some(3.0),
        "max" => Some(4.0),
        "std" | "stddev" | "std_dev" | "stdev" => Some(5.0),
        _ => None,
    }
}

fn parse_align_direction_code(value: &str) -> Option<f64> {
    match value.trim().to_ascii_lowercase().as_str() {
        "backward" | "backfill" => Some(0.0),
        "forward" | "ffill" => Some(1.0),
        "nearest" => Some(2.0),
        _ => None,
    }
}

fn parse_resample_period_ms(args: &[CallArg]) -> Option<u64> {
    if let Some(period_ms) = arg_number_optional(args, ArgSelector::Named("period_ms")) {
        return Some(period_ms.max(0.0).round() as u64);
    }
    if let Some(period_ms) = arg_number_optional(args, ArgSelector::Named("every_ms")) {
        return Some(period_ms.max(0.0).round() as u64);
    }
    if let Some(period_ms) = arg_number_optional(args, ArgSelector::Positional(1)) {
        return Some(period_ms.max(0.0).round() as u64);
    }
    arg_string_optional(args, ArgSelector::NamedOrPositional("every", 1))
        .or_else(|| arg_string_optional(args, ArgSelector::Named("interval")))
        .as_deref()
        .and_then(interval_to_ms)
}

fn merge_optional_f64(left: Option<f64>, right: Option<f64>) -> Option<Option<f64>> {
    match (left, right) {
        (Some(left), Some(right)) if (left - right).abs() <= f64::EPSILON => Some(Some(left)),
        (Some(_), Some(_)) => None,
        (Some(value), None) | (None, Some(value)) => Some(Some(value)),
        (None, None) => Some(None),
    }
}

fn spread_params(
    spread: &SpreadMatch,
    left_source: &DataSourceConfig,
    right_source: &DataSourceConfig,
) -> BTreeMap<String, f64> {
    let inferred_resample_period_ms = inferred_resample_period_ms(left_source, right_source);
    let explicit_resample_period_ms = spread
        .left
        .resample_period_ms
        .into_iter()
        .chain(spread.right.resample_period_ms)
        .max()
        .unwrap_or_default();
    let mut params = BTreeMap::from([
        ("field_code".into(), spread.left.field_code),
        (
            "align_direction_code".into(),
            spread.align_direction_code.unwrap_or(0.0),
        ),
        (
            "resample_period_ms".into(),
            explicit_resample_period_ms.max(inferred_resample_period_ms) as f64,
        ),
        (
            "resample_agg_code".into(),
            spread
                .left
                .resample_agg_code
                .or(spread.right.resample_agg_code)
                .unwrap_or(0.0),
        ),
        (
            "max_time_diff_ms".into(),
            spread.tolerance_ms.unwrap_or(5_000.0),
        ),
        ("left_field_code".into(), spread.left.field_code),
        ("right_field_code".into(), spread.right.field_code),
        ("left_window_size".into(), spread.left.window_size as f64),
        ("right_window_size".into(), spread.right.window_size as f64),
    ]);
    if let Some(period_ms) = spread.left.resample_period_ms {
        params.insert("left_resample_period_ms".into(), period_ms as f64);
    }
    if let Some(period_ms) = spread.right.resample_period_ms {
        params.insert("right_resample_period_ms".into(), period_ms as f64);
    }
    if let Some(code) = spread.left.resample_agg_code {
        params.insert("left_resample_agg_code".into(), code);
    }
    if let Some(code) = spread.right.resample_agg_code {
        params.insert("right_resample_agg_code".into(), code);
    }
    if let Some(code) = spread.left.window_agg_code {
        params.insert("left_window_agg_code".into(), code);
    }
    if let Some(code) = spread.right.window_agg_code {
        params.insert("right_window_agg_code".into(), code);
    }
    params
}

fn inferred_resample_period_ms(
    left_source: &DataSourceConfig,
    right_source: &DataSourceConfig,
) -> u64 {
    let left = left_source
        .interval
        .as_deref()
        .and_then(interval_to_ms)
        .unwrap_or_default();
    let right = right_source
        .interval
        .as_deref()
        .and_then(interval_to_ms)
        .unwrap_or_default();
    if left > 0 && right > 0 && left != right {
        left.max(right)
    } else {
        0
    }
}

fn interval_to_ms(interval: &str) -> Option<u64> {
    let trimmed = interval.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (value, unit) = trimmed.split_at(trimmed.len().saturating_sub(1));
    let count = value.parse::<u64>().ok()?;
    match unit {
        "m" => Some(count * 60_000),
        "h" => Some(count * 60 * 60_000),
        "d" => Some(count * 24 * 60 * 60_000),
        _ => None,
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
