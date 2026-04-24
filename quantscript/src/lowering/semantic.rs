use crate::resolve::{
    expr_semantic_key, ResolvedCallable, ResolvedChangeSmoothingKind, ResolvedExprSemantic,
    ResolvedFetchSourceKind, ResolvedManualIndicatorFormula, ResolvedWindowAggregateView,
};
use crate::script::Expr;
use std::collections::BTreeMap;

use super::bindings::{BindingEnv, RsiMethod};
use super::shared::{arg_expr_required, expr_number, find_arg, ArgSelector};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChangeKind {
    Gain,
    Loss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChangeSmoothing {
    Wilder,
    Ema,
    Simple,
}

pub(crate) fn resolved_change_smoothing_kind(
    env: &BindingEnv,
    fn_name: &str,
) -> Option<ChangeSmoothing> {
    match env
        .callables
        .get(fn_name)
        .and_then(|callable| callable.change_smoothing_kind)
    {
        Some(ResolvedChangeSmoothingKind::Wilder) => Some(ChangeSmoothing::Wilder),
        Some(ResolvedChangeSmoothingKind::Ema) => Some(ChangeSmoothing::Ema),
        Some(ResolvedChangeSmoothingKind::Simple) => Some(ChangeSmoothing::Simple),
        None => None,
    }
}

pub(crate) fn resolved_fetch_source_kind(
    callables: &BTreeMap<String, ResolvedCallable>,
    fn_name: &str,
) -> Option<ResolvedFetchSourceKind> {
    callables
        .get(fn_name)
        .and_then(|callable| callable.fetch_source_kind)
}

pub(crate) fn resolve_expr_alias<'a>(expr: &'a Expr, env: &'a BindingEnv) -> Option<&'a Expr> {
    match expr {
        Expr::Identifier(name) => env.expr_by_name.get(name).or(Some(expr)),
        _ => Some(expr),
    }
}

pub(crate) fn resolved_expr_semantic(
    expr: &Expr,
    env: &BindingEnv,
) -> Option<ResolvedExprSemantic> {
    let expr = resolve_expr_alias(expr, env).unwrap_or(expr);
    env.expr_semantics.get(&expr_semantic_key(expr)).copied()
}

pub(crate) fn resolved_window_aggregate_view(
    expr: &Expr,
    env: &BindingEnv,
) -> Option<ResolvedWindowAggregateView> {
    match resolved_expr_semantic(expr, env) {
        Some(ResolvedExprSemantic::WindowAggregateView(view)) => Some(view),
        _ => None,
    }
}

pub(crate) fn resolved_sum_window_match<'a>(
    expr: &'a Expr,
    env: &'a BindingEnv,
) -> Option<(&'a Expr, usize)> {
    let view = resolved_window_aggregate_view(expr, env)?;
    if view.aggregate_kind != crate::resolve::ResolvedWindowAggregateKind::Sum {
        return None;
    }
    let target_expr = series_capability_target_expr(expr, env)?;
    Some((target_expr, view.span))
}

pub(crate) fn resolved_boundary_lookback_span(expr: &Expr, env: &BindingEnv) -> Option<usize> {
    match resolved_expr_semantic(expr, env) {
        Some(ResolvedExprSemantic::BoundaryLookbackPair { span }) => Some(span),
        _ => None,
    }
}

pub(crate) fn resolved_boundary_lookback_match<'a>(
    expr: &'a Expr,
    env: &'a BindingEnv,
) -> Option<(&'a Expr, usize)> {
    if let Some(span) = resolved_boundary_lookback_span(expr, env) {
        let target_expr = boundary_lookback_target_expr(expr, env)?;
        return Some((target_expr, span));
    }

    match resolved_manual_indicator_formula(expr, env) {
        Some(ResolvedManualIndicatorFormula::Momentum { lookback }) => {
            let target_expr = boundary_lookback_target_expr(expr, env)?;
            Some((target_expr, lookback))
        }
        _ => None,
    }
}

pub(crate) fn resolved_manual_indicator_formula(
    expr: &Expr,
    env: &BindingEnv,
) -> Option<ResolvedManualIndicatorFormula> {
    match resolved_expr_semantic(expr, env) {
        Some(ResolvedExprSemantic::ManualIndicatorFormula(formula)) => Some(formula),
        _ => None,
    }
}

pub(crate) fn resolved_manual_moving_average_match<'a>(
    expr: &'a Expr,
    env: &'a BindingEnv,
) -> Option<(&'a Expr, usize)> {
    match resolved_manual_indicator_formula(expr, env) {
        Some(ResolvedManualIndicatorFormula::MovingAverage { span }) => {
            let target_expr = manual_moving_average_target_expr(expr, env)?;
            Some((target_expr, span))
        }
        _ => None,
    }
}

pub(crate) fn resolved_manual_zscore_match<'a>(
    expr: &'a Expr,
    env: &'a BindingEnv,
) -> Option<(&'a Expr, usize)> {
    match resolved_manual_indicator_formula(expr, env) {
        Some(ResolvedManualIndicatorFormula::ZScore { window }) => {
            let target_expr = manual_zscore_target_expr(expr, env)?;
            Some((target_expr, window))
        }
        _ => None,
    }
}

pub(crate) fn resolved_balanced_smoothed_change_pair(
    expr: &Expr,
    env: &BindingEnv,
) -> Option<(usize, ChangeSmoothing)> {
    match resolved_expr_semantic(expr, env) {
        Some(ResolvedExprSemantic::BalancedSmoothedChangePair { period, smoothing }) => Some((
            period,
            match smoothing {
                ResolvedChangeSmoothingKind::Wilder => ChangeSmoothing::Wilder,
                ResolvedChangeSmoothingKind::Ema => ChangeSmoothing::Ema,
                ResolvedChangeSmoothingKind::Simple => ChangeSmoothing::Simple,
            },
        )),
        _ => None,
    }
}

pub(crate) fn series_view_target_expr<'a>(expr: &'a Expr, env: &'a BindingEnv) -> Option<&'a Expr> {
    let expr = resolve_expr_alias(expr, env).unwrap_or(expr);
    match expr {
        Expr::Slice { object, .. } | Expr::Index { object, .. } => Some(object.as_ref()),
        Expr::Call { callee, args } => {
            find_arg(args, ArgSelector::Positional(0)).or_else(|| match callee.as_ref() {
                Expr::Member { object, .. } if args.is_empty() => Some(object.as_ref()),
                _ => None,
            })
        }
        _ => None,
    }
}

pub(crate) fn series_capability_target_expr<'a>(
    expr: &'a Expr,
    env: &'a BindingEnv,
) -> Option<&'a Expr> {
    let expr = resolve_expr_alias(expr, env).unwrap_or(expr);
    match expr {
        Expr::Call { callee, args } => {
            find_arg(args, ArgSelector::Positional(0)).or_else(|| match callee.as_ref() {
                Expr::Member { object, .. } if args.is_empty() => Some(object.as_ref()),
                _ => None,
            })
        }
        Expr::Member { object, .. } => Some(object.as_ref()),
        _ => None,
    }
}

pub(crate) fn boundary_lookback_target_expr<'a>(
    expr: &'a Expr,
    env: &'a BindingEnv,
) -> Option<&'a Expr> {
    let expr = resolve_expr_alias(expr, env).unwrap_or(expr);
    let Expr::Binary { left, .. } = expr else {
        return None;
    };
    series_view_target_expr(left, env)
}

pub(crate) fn manual_moving_average_target_expr<'a>(
    expr: &'a Expr,
    env: &'a BindingEnv,
) -> Option<&'a Expr> {
    let expr = resolve_expr_alias(expr, env).unwrap_or(expr);
    let Expr::Binary { left, .. } = expr else {
        return None;
    };
    series_capability_target_expr(left, env)
}

pub(crate) fn manual_macd_line_target_expr<'a>(
    expr: &'a Expr,
    env: &'a BindingEnv,
) -> Option<&'a Expr> {
    let expr = resolve_expr_alias(expr, env).unwrap_or(expr);
    let Expr::Binary { left, .. } = expr else {
        return None;
    };
    series_capability_target_expr(left, env)
}

pub(crate) fn manual_macd_signal_target_expr<'a>(
    expr: &'a Expr,
    env: &'a BindingEnv,
) -> Option<&'a Expr> {
    let expr = resolve_expr_alias(expr, env).unwrap_or(expr);
    let Expr::Call { args, .. } = expr else {
        return None;
    };
    let line_expr = arg_expr_required(args, ArgSelector::Positional(0)).ok()?;
    manual_macd_line_target_expr(line_expr, env)
}

pub(crate) fn manual_zscore_target_expr<'a>(
    expr: &'a Expr,
    env: &'a BindingEnv,
) -> Option<&'a Expr> {
    let expr = resolve_expr_alias(expr, env).unwrap_or(expr);
    let Expr::Binary { left, .. } = expr else {
        return None;
    };
    let Expr::Binary {
        left: current_expr, ..
    } = left.as_ref()
    else {
        return None;
    };
    series_view_target_expr(current_expr, env)
}

pub(crate) fn is_number_literal(expr: &Expr, expected: f64) -> bool {
    expr_number(expr)
        .map(|value| (value - expected).abs() <= f64::EPSILON)
        .unwrap_or(false)
}

pub(crate) fn rsi_method_code(method: RsiMethod) -> f64 {
    match method {
        RsiMethod::Wilder => 0.0,
        RsiMethod::Ema => 1.0,
        RsiMethod::Cutler => 2.0,
    }
}
