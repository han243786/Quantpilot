use crate::resolve::{ResolvedManualIndicatorFormula, ResolvedWindowAggregateKind};
use crate::script::{BinaryOp, Expr};
use anyhow::Result;
use qrpc_core::DataSourceConfig;

use super::super::binding_sources::{
    decode_series_position_view, decode_window_binding, resolve_data_source_ref, SeriesViewAccess,
    SourceSpanMatch,
};
use super::super::bindings::{BindingEnv, IndicatorBinding};
use super::super::semantic::{
    manual_zscore_target_expr, resolved_manual_indicator_formula, resolved_manual_zscore_match,
};

pub(super) fn manual_zscore_from_expr(
    expr: &Expr,
    left: &Expr,
    right: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    if let Some(ResolvedManualIndicatorFormula::ZScore { window }) =
        resolved_manual_indicator_formula(expr, env)
    {
        let Some(target_expr) = manual_zscore_target_expr(expr, env) else {
            return Ok(None);
        };
        let Some(source) = resolve_data_source_ref(target_expr, env, data_sources)? else {
            return Ok(None);
        };
        return Ok(Some(IndicatorBinding::ZScore { source, window }));
    }

    let Expr::Binary {
        left: numerator_left,
        op: BinaryOp::Subtract,
        right: numerator_right,
    } = left
    else {
        return Ok(None);
    };
    let Some(matched) = match_zscore_operands(
        expr,
        numerator_left,
        numerator_right,
        right,
        env,
        data_sources,
    )?
    else {
        return Ok(None);
    };

    Ok(Some(IndicatorBinding::ZScore {
        source: matched.source,
        window: matched.span,
    }))
}

pub(super) fn match_zscore_operands(
    expr: &Expr,
    current_expr: &Expr,
    mean_expr: &Expr,
    stddev_expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<SourceSpanMatch>> {
    if let Some((target_expr, span)) = resolved_manual_zscore_match(expr, env) {
        let Some(source) = resolve_data_source_ref(target_expr, env, data_sources)? else {
            return Ok(None);
        };
        return Ok(Some(SourceSpanMatch { source, span }));
    }

    legacy_zscore_operands(current_expr, mean_expr, stddev_expr, env, data_sources)
}

fn legacy_zscore_operands(
    current_expr: &Expr,
    mean_expr: &Expr,
    stddev_expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<SourceSpanMatch>> {
    let Some((current_source, current_position)) =
        decode_series_position_view(current_expr, env, data_sources)?
    else {
        return Ok(None);
    };
    if current_position != SeriesViewAccess::Current {
        return Ok(None);
    }

    let Some(mean_binding) = decode_window_binding(
        mean_expr,
        env,
        data_sources,
        ResolvedWindowAggregateKind::Mean,
    )?
    else {
        return Ok(None);
    };
    let Some(std_binding) = decode_window_binding(
        stddev_expr,
        env,
        data_sources,
        ResolvedWindowAggregateKind::StdDev,
    )?
    else {
        return Ok(None);
    };

    if current_source.data_id != mean_binding.source.data_id
        || current_source.data_id != std_binding.source.data_id
        || mean_binding.span != std_binding.span
    {
        return Ok(None);
    }

    Ok(Some(SourceSpanMatch {
        source: current_source,
        span: mean_binding.span,
    }))
}
