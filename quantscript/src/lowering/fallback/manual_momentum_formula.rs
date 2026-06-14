use crate::resolve::ResolvedManualIndicatorFormula;
use crate::script::{BinaryOp, Expr};
use anyhow::Result;
use qrpc_core::DataSourceConfig;

use super::super::binding_sources::{
    decode_series_position_view, resolve_data_source_ref, SeriesViewAccess, SourceSpanMatch,
};
use super::super::bindings::{BindingEnv, IndicatorBinding};
use super::super::semantic::{
    boundary_lookback_target_expr, resolve_expr_alias, resolved_manual_indicator_formula,
};
use super::super::shared::expr_number;

pub(super) fn manual_momentum_from_expr(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    if let Some(ResolvedManualIndicatorFormula::Momentum { lookback }) =
        resolved_manual_indicator_formula(expr, env)
    {
        let Some(target_expr) = boundary_lookback_target_expr(expr, env) else {
            return Ok(None);
        };
        let Some(source) = resolve_data_source_ref(target_expr, env, data_sources)? else {
            return Ok(None);
        };
        return Ok(Some(IndicatorBinding::Momentum { source, lookback }));
    }

    momentum_from_span_match(expr, env, data_sources)
}

pub(super) fn manual_momentum_ratio_from_division(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    if let Some(ResolvedManualIndicatorFormula::Momentum { lookback }) =
        resolved_manual_indicator_formula(expr, env)
    {
        let Some(target_expr) = boundary_lookback_target_expr(expr, env) else {
            return Ok(None);
        };
        let Some(source) = resolve_data_source_ref(target_expr, env, data_sources)? else {
            return Ok(None);
        };
        return Ok(Some(IndicatorBinding::Momentum { source, lookback }));
    }

    momentum_from_span_match(expr, env, data_sources)
}

pub(super) fn manual_momentum_ratio_from_subtract_division(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    let Expr::Binary {
        left,
        op: BinaryOp::Subtract,
        right,
    } = expr
    else {
        return Ok(None);
    };
    let Some(offset) = expr_number(right) else {
        return Ok(None);
    };
    if (offset - 1.0).abs() > f64::EPSILON {
        return Ok(None);
    }
    let Expr::Binary {
        left: _ratio_left,
        op: BinaryOp::Divide,
        right: _ratio_right,
    } = left.as_ref()
    else {
        return Ok(None);
    };
    let Some(latest_binding) = manual_momentum_ratio_from_division(left, env, data_sources)? else {
        return Ok(None);
    };
    Ok(Some(latest_binding))
}

fn momentum_from_span_match(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    let matched = if let Some(matched) =
        super::resolve_boundary_lookback_source_span(expr, env, data_sources)?
    {
        Some(matched)
    } else {
        legacy_latest_lookback_pair(expr, env, data_sources)?
    };
    let Some(matched) = matched else {
        return Ok(None);
    };

    Ok(Some(IndicatorBinding::Momentum {
        source: matched.source,
        lookback: matched.span,
    }))
}

fn legacy_latest_lookback_pair(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<SourceSpanMatch>> {
    let expr = resolve_expr_alias(expr, env).unwrap_or(expr);
    let Expr::Binary { left, right, .. } = expr else {
        return Ok(None);
    };
    let Some((left_source, left_position)) = decode_series_position_view(left, env, data_sources)?
    else {
        return Ok(None);
    };
    let Some((right_source, right_position)) =
        decode_series_position_view(right, env, data_sources)?
    else {
        return Ok(None);
    };
    if left_source.data_id != right_source.data_id || left_position != SeriesViewAccess::Current {
        return Ok(None);
    }

    let SeriesViewAccess::Lookback(span) = right_position else {
        return Ok(None);
    };

    Ok(Some(SourceSpanMatch {
        source: left_source,
        span,
    }))
}
