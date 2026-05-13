use crate::resolve::{
    ResolvedExprSemantic, ResolvedManualIndicatorFormula, ResolvedSeriesCapabilityKind,
    ResolvedWindowAggregateKind,
};
use crate::script::{BinaryOp, Expr};
use anyhow::Result;
use qrpc_core::DataSourceConfig;

use super::binding_sources::{
    decode_series_position_view, decode_series_window_view, decode_smoothed_change_binding,
    decode_window_binding, resolve_data_source_ref, SeriesViewAccess, SourcePeriodSmoothingMatch,
    SourceSpanMatch,
};
use super::bindings::{
    resolve_indicator_binding, BindingEnv, IndicatorBinding, MovingAverageMethod, RsiMethod,
};
use super::semantic::{
    boundary_lookback_target_expr, is_number_literal, manual_macd_line_target_expr,
    manual_moving_average_target_expr, manual_zscore_target_expr, resolve_expr_alias,
    resolved_balanced_smoothed_change_pair, resolved_boundary_lookback_match,
    resolved_expr_semantic, resolved_manual_indicator_formula,
    resolved_manual_moving_average_match, resolved_manual_zscore_match, resolved_sum_window_match,
    series_capability_target_expr, ChangeKind, ChangeSmoothing,
};
use super::shared::expr_number;

#[derive(Debug, Clone, Copy)]
struct RsiRsPairMatch<'a> {
    rs_expr: &'a Expr,
    avg_gain_expr: &'a Expr,
    avg_loss_expr: &'a Expr,
}

#[derive(Debug, Clone)]
struct EmaSpreadMatch {
    source: DataSourceConfig,
    fast_period: usize,
    slow_period: usize,
}

#[derive(Debug, Clone)]
struct MacdLineSignalMatch {
    source: DataSourceConfig,
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
}

// Permanent fallback: RSI still keeps its outer formula shell in lowering.
// resolve only standardizes the stable core parameter layer, while the final
// shell check and RsiMethod mapping remain runtime-facing work here.
pub(crate) fn resolve_manual_formula_binding(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    if let Some(binding) = manual_rsi_from_expr(expr, env, data_sources)? {
        return Ok(Some(binding));
    }

    match expr {
        Expr::Binary {
            left,
            op: BinaryOp::Subtract,
            right,
        } => {
            if let Some(binding) =
                manual_macd_histogram_from_expr(expr, left, right, env, data_sources)?
            {
                return Ok(Some(binding));
            }
            if let Some(binding) = manual_macd_line_from_expr(expr, left, right, env, data_sources)?
            {
                return Ok(Some(binding));
            }
            if let Some(binding) = manual_momentum_from_expr(expr, env, data_sources)? {
                return Ok(Some(binding));
            }
            manual_momentum_ratio_from_subtract_division(expr, env, data_sources)
        }
        Expr::Binary {
            left,
            op: BinaryOp::Divide,
            right,
        } => {
            if let Some(binding) = manual_zscore_from_expr(expr, left, right, env, data_sources)? {
                return Ok(Some(binding));
            }
            if let Some(binding) = manual_momentum_ratio_from_division(expr, env, data_sources)? {
                return Ok(Some(binding));
            }
            moving_average_from_expr(expr, left, right, env, data_sources)
        }
        _ => Ok(None),
    }
}

pub(crate) fn manual_rsi_from_expr(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    let Some(matched) = match_manual_rsi_formula(expr, env, data_sources)? else {
        return Ok(None);
    };

    Ok(Some(IndicatorBinding::Rsi {
        source: matched.source,
        period: matched.period,
        method: match matched.smoothing {
            ChangeSmoothing::Wilder => RsiMethod::Wilder,
            ChangeSmoothing::Ema => RsiMethod::Ema,
            ChangeSmoothing::Simple => RsiMethod::Cutler,
        },
    }))
}

pub(crate) fn match_manual_rsi_formula(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<SourcePeriodSmoothingMatch>> {
    if let Some(alias) = resolve_expr_alias(expr, env) {
        if !std::ptr::eq(alias, expr) {
            return match_manual_rsi_formula(alias, env, data_sources);
        }
    }

    let Some(rs_pair) = match_rsi_rs_pair(expr, env) else {
        return Ok(None);
    };
    if let Some((period, smoothing)) = resolved_balanced_smoothed_change_pair(rs_pair.rs_expr, env)
    {
        let Some(source) = balanced_smoothed_change_pair_source(
            rs_pair.avg_gain_expr,
            rs_pair.avg_loss_expr,
            env,
            data_sources,
        )?
        else {
            return Ok(None);
        };
        return Ok(Some(SourcePeriodSmoothingMatch {
            source,
            period,
            smoothing,
        }));
    }
    let Some(smoothed_pair) = match_balanced_smoothed_change_pair(
        rs_pair.avg_gain_expr,
        rs_pair.avg_loss_expr,
        env,
        data_sources,
    )?
    else {
        return Ok(None);
    };

    Ok(Some(SourcePeriodSmoothingMatch {
        source: smoothed_pair.source,
        period: smoothed_pair.period,
        smoothing: smoothed_pair.smoothing,
    }))
}

fn match_rsi_rs_pair<'a>(expr: &'a Expr, env: &'a BindingEnv) -> Option<RsiRsPairMatch<'a>> {
    let expr = resolve_expr_alias(expr, env)?;
    match expr {
        Expr::Binary {
            left,
            op: BinaryOp::Subtract,
            right,
        } if is_number_literal(left, 100.0) => {
            let Expr::Binary {
                left: numerator,
                op: BinaryOp::Divide,
                right: denominator,
            } = right.as_ref()
            else {
                return None;
            };
            if !is_number_literal(numerator, 100.0) {
                return None;
            }
            match_rs_pair_from_denominator(denominator, env)
        }
        _ => None,
    }
}

fn match_rs_pair_from_denominator<'a>(
    expr: &'a Expr,
    env: &'a BindingEnv,
) -> Option<RsiRsPairMatch<'a>> {
    let expr = resolve_expr_alias(expr, env)?;
    let Expr::Binary {
        left,
        op: BinaryOp::Add,
        right,
    } = expr
    else {
        return None;
    };

    if is_number_literal(left, 1.0) {
        match_rs_pair_expr(right, env)
    } else if is_number_literal(right, 1.0) {
        match_rs_pair_expr(left, env)
    } else {
        None
    }
}

fn match_rs_pair_expr<'a>(expr: &'a Expr, env: &'a BindingEnv) -> Option<RsiRsPairMatch<'a>> {
    let expr = resolve_expr_alias(expr, env)?;
    let Expr::Binary {
        left,
        op: BinaryOp::Divide,
        right,
    } = expr
    else {
        return None;
    };
    Some(RsiRsPairMatch {
        rs_expr: expr,
        avg_gain_expr: left.as_ref(),
        avg_loss_expr: right.as_ref(),
    })
}

fn match_balanced_smoothed_change_pair(
    gain_expr: &Expr,
    loss_expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<SourcePeriodSmoothingMatch>> {
    let Some(gain) =
        decode_smoothed_change_binding(gain_expr, ChangeKind::Gain, env, data_sources)?
    else {
        return Ok(None);
    };
    let Some(loss) =
        decode_smoothed_change_binding(loss_expr, ChangeKind::Loss, env, data_sources)?
    else {
        return Ok(None);
    };
    if gain.source.data_id != loss.source.data_id
        || gain.period != loss.period
        || gain.smoothing != loss.smoothing
    {
        return Ok(None);
    }

    Ok(Some(SourcePeriodSmoothingMatch {
        source: gain.source,
        period: gain.period,
        smoothing: gain.smoothing,
    }))
}

fn balanced_smoothed_change_pair_source(
    gain_expr: &Expr,
    loss_expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<DataSourceConfig>> {
    if let Some(gain) =
        decode_smoothed_change_binding(gain_expr, ChangeKind::Gain, env, data_sources)?
    {
        return Ok(Some(gain.source));
    }
    if let Some(loss) =
        decode_smoothed_change_binding(loss_expr, ChangeKind::Loss, env, data_sources)?
    {
        return Ok(Some(loss.source));
    }
    Ok(None)
}

pub(crate) fn manual_momentum_from_expr(
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

    let matched =
        if let Some(matched) = resolve_boundary_lookback_source_span(expr, env, data_sources)? {
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

pub(crate) fn manual_momentum_ratio_from_division(
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

    let matched =
        if let Some(matched) = resolve_boundary_lookback_source_span(expr, env, data_sources)? {
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

pub(crate) fn manual_momentum_ratio_from_subtract_division(
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

pub(crate) fn manual_macd_line_from_expr(
    expr: &Expr,
    left: &Expr,
    right: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    if let Some(ResolvedManualIndicatorFormula::MacdLine {
        fast_period,
        slow_period,
    }) = resolved_manual_indicator_formula(expr, env)
    {
        let Some(target_expr) = manual_macd_line_target_expr(expr, env) else {
            return Ok(None);
        };
        let Some(source) = resolve_data_source_ref(target_expr, env, data_sources)? else {
            return Ok(None);
        };
        return Ok(Some(IndicatorBinding::MacdLine {
            source,
            fast_period,
            slow_period,
        }));
    }

    let Some(matched) = match_ema_spread(left, right, env, data_sources)? else {
        return Ok(None);
    };

    Ok(Some(IndicatorBinding::MacdLine {
        source: matched.source,
        fast_period: matched.fast_period,
        slow_period: matched.slow_period,
    }))
}

pub(crate) fn manual_macd_histogram_from_expr(
    expr: &Expr,
    left: &Expr,
    right: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    if let Some(ResolvedManualIndicatorFormula::MacdHistogram {
        fast_period,
        slow_period,
        signal_period,
    }) = resolved_manual_indicator_formula(expr, env)
    {
        let Some(target_expr) = manual_macd_line_target_expr(left, env)
            .or_else(|| manual_macd_line_target_expr(right, env))
        else {
            return Ok(None);
        };
        let Some(source) = resolve_data_source_ref(target_expr, env, data_sources)? else {
            return Ok(None);
        };
        return Ok(Some(IndicatorBinding::Macd {
            source,
            fast_period,
            slow_period,
            signal_period,
        }));
    }

    let Some(matched) = match_macd_line_signal_pair(left, right, env, data_sources)? else {
        return Ok(None);
    };

    Ok(Some(IndicatorBinding::Macd {
        source: matched.source,
        fast_period: matched.fast_period,
        slow_period: matched.slow_period,
        signal_period: matched.signal_period,
    }))
}

pub(crate) fn manual_zscore_from_expr(
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

pub(crate) fn moving_average_from_expr(
    expr: &Expr,
    left: &Expr,
    right: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    if let Some(ResolvedManualIndicatorFormula::MovingAverage { span }) =
        resolved_manual_indicator_formula(expr, env)
    {
        let Some(target_expr) = manual_moving_average_target_expr(expr, env) else {
            return Ok(None);
        };
        let Some(windowed) = decode_series_window_view(target_expr, env, data_sources)? else {
            return Ok(None);
        };
        return Ok(Some(IndicatorBinding::MovingAverage {
            source: windowed.source,
            period: span,
            method: MovingAverageMethod::Sma,
        }));
    }

    let Some(matched) = match_manual_moving_average_window(expr, left, right, env, data_sources)?
    else {
        return Ok(None);
    };
    Ok(Some(IndicatorBinding::MovingAverage {
        source: matched.source,
        period: matched.span,
        method: MovingAverageMethod::Sma,
    }))
}

// Transitional fallback matchers: these already prefer ResolveResult semantics
// when available, but still keep local AST-shape recovery as a compatibility
// path. They are the first candidates for future promotion only if source
// recovery can be standardized without moving runtime interpretation forward.
fn match_sum_window_call(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<(DataSourceConfig, usize)>> {
    if let Some((target_expr, span)) = resolved_sum_window_match(expr, env) {
        let Some(windowed) = decode_series_window_view(target_expr, env, data_sources)? else {
            return Ok(None);
        };
        return Ok(Some((windowed.source, span)));
    }

    legacy_sum_window_call(expr, env, data_sources)
}

fn legacy_sum_window_call(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<(DataSourceConfig, usize)>> {
    if !matches!(
        resolved_expr_semantic(expr, env),
        Some(ResolvedExprSemantic::SeriesCapability(
            ResolvedSeriesCapabilityKind::WindowAggregate(ResolvedWindowAggregateKind::Sum)
        ))
    ) {
        return Ok(None);
    }
    let Some(target_expr) = series_capability_target_expr(expr, env) else {
        return Ok(None);
    };
    let Some(windowed) = decode_series_window_view(target_expr, env, data_sources)? else {
        return Ok(None);
    };
    Ok(Some((windowed.source, windowed.span)))
}

pub(crate) fn resolve_boundary_lookback_source_span(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<SourceSpanMatch>> {
    if let Some((target_expr, span)) = resolved_boundary_lookback_match(expr, env) {
        let Some(source) = resolve_data_source_ref(target_expr, env, data_sources)? else {
            return Ok(None);
        };
        return Ok(Some(SourceSpanMatch { source, span }));
    }

    Ok(None)
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

// Transitional fallback for older or partially normalized indicator forms.
// These matchers should keep shrinking as stable parameter semantics move into
// ResolveResult, but they still protect existing alias/helper shapes today.
fn match_ema_spread(
    left: &Expr,
    right: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<EmaSpreadMatch>> {
    let Some(left_binding) = resolve_indicator_binding(left, env, data_sources)? else {
        return Ok(None);
    };
    let Some(right_binding) = resolve_indicator_binding(right, env, data_sources)? else {
        return Ok(None);
    };

    let (
        IndicatorBinding::MovingAverage {
            source: left_source,
            period: left_period,
            method: left_method,
        },
        IndicatorBinding::MovingAverage {
            source: right_source,
            period: right_period,
            method: right_method,
        },
    ) = (left_binding, right_binding)
    else {
        return Ok(None);
    };

    if left_source.data_id != right_source.data_id
        || left_method != MovingAverageMethod::Ema
        || right_method != MovingAverageMethod::Ema
    {
        return Ok(None);
    }

    let fast_period = left_period.min(right_period);
    let slow_period = left_period.max(right_period);
    if fast_period == slow_period {
        return Ok(None);
    }

    Ok(Some(EmaSpreadMatch {
        source: left_source,
        fast_period,
        slow_period,
    }))
}

fn match_macd_line_signal_pair(
    left: &Expr,
    right: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<MacdLineSignalMatch>> {
    let Some(left_binding) = resolve_indicator_binding(left, env, data_sources)? else {
        return Ok(None);
    };
    let Some(right_binding) = resolve_indicator_binding(right, env, data_sources)? else {
        return Ok(None);
    };

    let (
        IndicatorBinding::MacdLine {
            source: left_source,
            fast_period: left_fast,
            slow_period: left_slow,
        },
        IndicatorBinding::MacdSignal {
            source: right_source,
            fast_period: right_fast,
            slow_period: right_slow,
            signal_period,
        },
    ) = (left_binding, right_binding)
    else {
        return Ok(None);
    };

    if left_source.data_id != right_source.data_id
        || left_fast != right_fast
        || left_slow != right_slow
    {
        return Ok(None);
    }

    Ok(Some(MacdLineSignalMatch {
        source: left_source,
        fast_period: left_fast,
        slow_period: left_slow,
        signal_period,
    }))
}

pub(crate) fn match_zscore_operands(
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

// Transitional fallback: the common moving-average span is already
// standardized in resolve, but this local matcher still recovers non-standard
// or partially normalized shapes that are not worth rejecting outright.
pub(crate) fn match_manual_moving_average_window(
    expr: &Expr,
    sum_expr: &Expr,
    period_expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<SourceSpanMatch>> {
    if let Some((target_expr, span)) = resolved_manual_moving_average_match(expr, env) {
        let Some(windowed) = decode_series_window_view(target_expr, env, data_sources)? else {
            return Ok(None);
        };
        return Ok(Some(SourceSpanMatch {
            source: windowed.source,
            span,
        }));
    }

    legacy_manual_moving_average_window(sum_expr, period_expr, env, data_sources)
}

fn legacy_manual_moving_average_window(
    sum_expr: &Expr,
    period_expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<SourceSpanMatch>> {
    let Some(period) = expr_number(period_expr).map(|value| value.round() as usize) else {
        return Ok(None);
    };
    if period == 0 {
        return Ok(None);
    }

    let Some((source, window)) = match_sum_window_call(sum_expr, env, data_sources)? else {
        return Ok(None);
    };
    if window != period {
        return Ok(None);
    }

    Ok(Some(SourceSpanMatch {
        source,
        span: period,
    }))
}

#[cfg(test)]
mod tests {
    use super::super::binding_sources::infer_data_sources;
    use super::super::bindings::{
        collect_bindings, resolve_indicator_binding, BindingEnv, IndicatorBinding,
    };
    use super::super::diagnostics::format_diagnostics;
    use super::*;
    use crate::evaluator::normalize_script_module;
    use crate::parse_quant_script_module;
    use crate::resolve::lower_script_to_typed_hir;
    use crate::script::{FunctionDecl, Item, Stmt};

    fn prepare_strategy_bindings(
        script: &str,
    ) -> (FunctionDecl, Vec<DataSourceConfig>, BindingEnv) {
        let module = parse_quant_script_module(script).unwrap();
        let normalized = normalize_script_module(&module).unwrap();
        let resolved = lower_script_to_typed_hir(&normalized);
        assert!(
            !resolved.has_errors(),
            "unexpected diagnostics: {}",
            format_diagnostics(&resolved.diagnostics)
        );
        let strategy = normalized
            .items
            .iter()
            .find_map(|item| match item {
                Item::Function(function) if function.name == "strategy" => Some(function.clone()),
                _ => None,
            })
            .unwrap();
        let data_sources = infer_data_sources(&strategy, &resolved.callables).unwrap();
        let (bindings, _) = collect_bindings(
            &strategy,
            &data_sources,
            resolved.functions.clone(),
            resolved.expr_semantics.clone(),
            resolved.callables.clone(),
        )
        .unwrap();
        (strategy, data_sources, bindings)
    }

    fn find_let_expr<'a>(stmts: &'a [Stmt], pattern: &str) -> &'a Expr {
        stmts
            .iter()
            .find_map(|stmt| match stmt {
                Stmt::Let {
                    pattern: stmt_pattern,
                    value,
                    ..
                } if stmt_pattern == pattern => Some(value),
                _ => None,
            })
            .unwrap_or_else(|| panic!("missing let binding: {pattern}"))
    }

    fn expect_binary_expr(expr: &Expr, op: BinaryOp) -> (&Expr, &Expr) {
        match expr {
            Expr::Binary {
                left,
                op: got,
                right,
            } if *got == op => (left.as_ref(), right.as_ref()),
            other => panic!("expected binary {op:?}, got {other:?}"),
        }
    }

    #[test]
    fn matches_manual_rsi_formula_directly() {
        let (strategy, data_sources, bindings) = prepare_strategy_bindings(
            r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let avg_gain = wilders(gains(closes), 14)
    let avg_loss = wilders(losses(closes), 14)
    let rs = avg_gain / avg_loss
    let score = 100 - 100 / (1 + rs)
}
"#,
        );

        let score = find_let_expr(&strategy.body, "score");
        let matched = match_manual_rsi_formula(score, &bindings, &data_sources)
            .unwrap()
            .unwrap();

        assert_eq!(matched.period, 14);
        assert_eq!(matched.smoothing, ChangeSmoothing::Wilder);
        assert_eq!(matched.source.data_id, "script_binance_btcusdt_1d");
    }

    #[test]
    fn resolves_manual_moving_average_window_from_resolve_first_contract() {
        let (strategy, data_sources, bindings) = prepare_strategy_bindings(
            r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let avg = closes[20..].sum() / 20
}
"#,
        );

        let avg = find_let_expr(&strategy.body, "avg");
        let (sum_expr, period_expr) = expect_binary_expr(avg, BinaryOp::Divide);
        let matched = match_manual_moving_average_window(
            avg,
            sum_expr,
            period_expr,
            &bindings,
            &data_sources,
        )
        .unwrap()
        .unwrap();

        assert_eq!(matched.span, 20);
        assert_eq!(matched.source.data_id, "script_binance_btcusdt_1d");
    }

    #[test]
    fn resolves_sum_window_call_from_resolve_first_contract() {
        let (strategy, data_sources, bindings) = prepare_strategy_bindings(
            r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let total = closes[20..].sum()
}
"#,
        );

        let total = find_let_expr(&strategy.body, "total");
        let matched = match_sum_window_call(total, &bindings, &data_sources)
            .unwrap()
            .unwrap();

        assert_eq!(matched.1, 20);
        assert_eq!(matched.0.data_id, "script_binance_btcusdt_1d");
    }

    #[test]
    fn resolves_boundary_lookback_source_span_from_resolve_first_contract() {
        let (strategy, data_sources, bindings) = prepare_strategy_bindings(
            r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = closes.last() - closes[14]
}
"#,
        );

        let score = find_let_expr(&strategy.body, "score");
        let matched = resolve_boundary_lookback_source_span(score, &bindings, &data_sources)
            .unwrap()
            .unwrap();

        assert_eq!(matched.span, 14);
        assert_eq!(matched.source.data_id, "script_binance_btcusdt_1d");
    }

    #[test]
    fn resolves_manual_macd_line_from_expr_directly() {
        let (strategy, data_sources, bindings) = prepare_strategy_bindings(
            r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let line = ema(closes, 12) - ema(closes, 26)
}
"#,
        );

        let line = find_let_expr(&strategy.body, "line");
        let binding = resolve_indicator_binding(line, &bindings, &data_sources)
            .unwrap()
            .unwrap();

        let IndicatorBinding::MacdLine {
            source,
            fast_period,
            slow_period,
        } = binding
        else {
            panic!("expected macd line binding");
        };

        assert_eq!(source.data_id, "script_binance_btcusdt_1d");
        assert_eq!(fast_period, 12);
        assert_eq!(slow_period, 26);
    }

    #[test]
    fn resolves_manual_macd_histogram_from_expr_directly() {
        let (strategy, data_sources, bindings) = prepare_strategy_bindings(
            r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let macd_line = ema(closes, 12) - ema(closes, 26)
    let signal_line = ema(macd_line, 9)
    let hist = macd_line - signal_line
}
"#,
        );

        let hist = find_let_expr(&strategy.body, "hist");
        let binding = resolve_indicator_binding(hist, &bindings, &data_sources)
            .unwrap()
            .unwrap();

        let IndicatorBinding::Macd {
            source,
            fast_period,
            slow_period,
            signal_period,
        } = binding
        else {
            panic!("expected macd histogram binding");
        };

        assert_eq!(source.data_id, "script_binance_btcusdt_1d");
        assert_eq!(fast_period, 12);
        assert_eq!(slow_period, 26);
        assert_eq!(signal_period, 9);
    }

    #[test]
    fn resolves_manual_macd_signal_from_expr_directly() {
        let (strategy, data_sources, bindings) = prepare_strategy_bindings(
            r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let macd_line = ema(closes, 12) - ema(closes, 26)
    let signal_line = ema(macd_line, 9)
}
"#,
        );

        let signal_line = find_let_expr(&strategy.body, "signal_line");
        let binding = resolve_indicator_binding(signal_line, &bindings, &data_sources)
            .unwrap()
            .unwrap();

        let IndicatorBinding::MacdSignal {
            source,
            fast_period,
            slow_period,
            signal_period,
        } = binding
        else {
            panic!("expected macd signal binding");
        };

        assert_eq!(source.data_id, "script_binance_btcusdt_1d");
        assert_eq!(fast_period, 12);
        assert_eq!(slow_period, 26);
        assert_eq!(signal_period, 9);
    }

    #[test]
    fn resolves_zscore_operands_from_resolve_first_contract() {
        let (strategy, data_sources, bindings) = prepare_strategy_bindings(
            r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let scope = closes[20..]
    let score = (closes.last() - scope.mean()) / scope.stddev()
}
"#,
        );

        let score = find_let_expr(&strategy.body, "score");
        let (numerator, denominator) = expect_binary_expr(score, BinaryOp::Divide);
        let (current, mean_expr) = expect_binary_expr(numerator, BinaryOp::Subtract);
        let matched = match_zscore_operands(
            score,
            current,
            mean_expr,
            denominator,
            &bindings,
            &data_sources,
        )
        .unwrap()
        .unwrap();

        assert_eq!(matched.span, 20);
        assert_eq!(matched.source.data_id, "script_binance_btcusdt_1d");
    }
}
