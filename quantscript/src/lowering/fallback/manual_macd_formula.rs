use crate::resolve::ResolvedManualIndicatorFormula;
use crate::script::Expr;
use anyhow::Result;
use qrpc_core::DataSourceConfig;

use super::super::binding_sources::resolve_data_source_ref;
use super::super::bindings::{
    resolve_indicator_binding, BindingEnv, IndicatorBinding, MovingAverageMethod,
};
use super::super::semantic::{manual_macd_line_target_expr, resolved_manual_indicator_formula};

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

pub(super) fn manual_macd_line_from_expr(
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

pub(super) fn manual_macd_histogram_from_expr(
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
