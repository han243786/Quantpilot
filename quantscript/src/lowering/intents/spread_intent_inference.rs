use crate::resolve::ResolvedWindowAggregateKind;
use crate::script::{BinaryOp, CallArg, Expr};
use anyhow::Result;
use qrpc_core::{DataSourceConfig, IntentConfig, IntentKind};
use std::collections::BTreeMap;

use super::super::binding_sources::{
    decode_series_position_view, decode_window_binding, format_symbol, parse_call,
    resolve_data_source_ref, SeriesViewAccess,
};
use super::super::bindings::BindingEnv;
use super::super::semantic::resolve_expr_alias;
use super::super::shared::{
    arg_number_optional, arg_string_optional, expr_number, find_arg, sanitize_id, ArgSelector,
};
use super::{
    canonicalize_data_source_for_bindings, comparison_op_code, comparison_op_from_binary_relation,
    comparison_shape_code, STRUCTURED_COMPARISON_OP_KEY, STRUCTURED_COMPARISON_SHAPE_KEY,
    STRUCTURED_COMPARISON_THRESHOLD_KEY,
};

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

pub(super) fn spread_intent_from_condition(
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
