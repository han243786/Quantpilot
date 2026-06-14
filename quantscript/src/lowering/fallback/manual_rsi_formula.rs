use crate::script::{BinaryOp, Expr};
use anyhow::Result;
use qrpc_core::DataSourceConfig;

use super::super::binding_sources::{decode_smoothed_change_binding, SourcePeriodSmoothingMatch};
use super::super::bindings::{BindingEnv, IndicatorBinding, RsiMethod};
use super::super::semantic::{
    is_number_literal, resolve_expr_alias, resolved_balanced_smoothed_change_pair, ChangeKind,
    ChangeSmoothing,
};

#[derive(Debug, Clone, Copy)]
struct RsiRsPairMatch<'a> {
    rs_expr: &'a Expr,
    avg_gain_expr: &'a Expr,
    avg_loss_expr: &'a Expr,
}

pub(super) fn manual_rsi_from_expr(
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

pub(super) fn match_manual_rsi_formula(
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
