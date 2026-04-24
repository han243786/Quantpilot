use crate::resolve::{
    KnownIndicatorHelperKind, MovingAverageHelperKind, ResolvedCallable, ResolvedExprSemantic,
    ResolvedFunction, ResolvedManualIndicatorFormula, ResolvedSeriesCapabilityKind, RsiHelperKind,
};
use crate::script::{CallArg, Expr, FunctionDecl};
use anyhow::{bail, Result};
use qrpc_core::DataSourceConfig;
use std::collections::BTreeMap;

use super::binding_sources::{
    decode_macd_args, decode_momentum_args, decode_moving_average_args, decode_rsi_args,
    decode_zscore_args, parse_call, resolve_data_source_ref,
};
use super::fallback::resolve_manual_formula_binding;
use super::helper_env::{
    collect_bindings_from_stmts, empty_binding_env, hydrate_helper_function_env,
};
use super::semantic::{
    manual_macd_signal_target_expr, resolved_expr_semantic, resolved_manual_indicator_formula,
};

const ERR_MOVING_AVERAGE_SOURCE_REQUIRED: &str =
    "QPQSLOW024 moving-average helpers require a fetch/get_data source as their first arg, except ema(...) may also consume a recognized MACD line";

#[derive(Debug, Clone)]
pub(crate) struct BindingEnv {
    pub(crate) data_by_name: BTreeMap<String, DataSourceConfig>,
    pub(crate) indicator_by_name: BTreeMap<String, IndicatorBinding>,
    pub(crate) expr_by_name: BTreeMap<String, Expr>,
    pub(crate) expr_semantics: BTreeMap<String, ResolvedExprSemantic>,
    pub(crate) callables: BTreeMap<String, ResolvedCallable>,
    pub(crate) functions: BTreeMap<String, ResolvedFunction>,
}

#[derive(Debug, Clone)]
pub(crate) enum IndicatorBinding {
    MovingAverage {
        source: DataSourceConfig,
        period: usize,
        method: MovingAverageMethod,
    },
    Rsi {
        source: DataSourceConfig,
        period: usize,
        method: RsiMethod,
    },
    MacdLine {
        source: DataSourceConfig,
        fast_period: usize,
        slow_period: usize,
    },
    MacdSignal {
        source: DataSourceConfig,
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    },
    Macd {
        source: DataSourceConfig,
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    },
    Momentum {
        source: DataSourceConfig,
        lookback: usize,
    },
    ZScore {
        source: DataSourceConfig,
        window: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MovingAverageMethod {
    Sma,
    Ema,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RsiMethod {
    Wilder,
    Ema,
    Cutler,
}

pub(crate) fn collect_bindings(
    strategy: &FunctionDecl,
    data_sources: &[DataSourceConfig],
    functions: BTreeMap<String, ResolvedFunction>,
    expr_semantics: BTreeMap<String, ResolvedExprSemantic>,
    callables: BTreeMap<String, ResolvedCallable>,
) -> Result<BindingEnv> {
    let mut env = empty_binding_env(functions, expr_semantics, callables);
    collect_bindings_from_stmts(&strategy.body, &mut env, data_sources)?;
    Ok(env)
}

pub(crate) fn resolve_indicator_binding(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    if let Some(binding) = resolve_manual_formula_binding(expr, env, data_sources)? {
        return Ok(Some(binding));
    }

    if let Some(binding) = resolve_indicator_binding_from_direct_forms(expr, env, data_sources)? {
        return Ok(Some(binding));
    }

    resolve_indicator_binding_from_call(expr, env, data_sources)
}

fn resolve_indicator_binding_from_direct_forms(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    match expr {
        Expr::Identifier(name) => Ok(env.indicator_by_name.get(name).cloned()),
        Expr::Try(inner) | Expr::Await(inner) => {
            resolve_indicator_binding(inner, env, data_sources)
        }
        Expr::Member { object, .. }
            if matches!(
                resolved_expr_semantic(expr, env),
                Some(ResolvedExprSemantic::SeriesCapability(
                    ResolvedSeriesCapabilityKind::Histogram
                ))
            ) =>
        {
            resolve_indicator_binding(object, env, data_sources)
        }
        _ => Ok(None),
    }
}

fn resolve_indicator_binding_from_call(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    match expr {
        Expr::Call { .. } => indicator_from_call(expr, env, data_sources),
        _ => Ok(None),
    }
}

fn indicator_from_call(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    let Some((fn_name, args)) = parse_call(expr) else {
        return Ok(None);
    };

    match resolved_indicator_kind(env, &fn_name) {
        Some(KnownIndicatorHelperKind::MovingAverage(method_kind)) => {
            indicator_from_moving_average_call(&fn_name, method_kind, args, env, data_sources)
        }
        Some(KnownIndicatorHelperKind::Rsi(method_kind)) => {
            indicator_from_rsi_call(method_kind, args, env, data_sources)
        }
        Some(KnownIndicatorHelperKind::Macd) => indicator_from_macd_call(args, env, data_sources),
        Some(KnownIndicatorHelperKind::Momentum) => {
            indicator_from_momentum_call(args, env, data_sources)
        }
        Some(KnownIndicatorHelperKind::ZScore) => {
            indicator_from_zscore_call(args, env, data_sources)
        }
        None => helper_function_indicator_binding(&fn_name, args, env, data_sources),
    }
}

fn helper_function_indicator_binding(
    name: &str,
    args: &[CallArg],
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    let Some(function) = env.functions.get(name) else {
        return Ok(None);
    };
    let Some(helper_env) = hydrate_helper_function_env(function, args, env, data_sources, true)?
    else {
        return Ok(None);
    };
    let Some(return_expr) = function.return_expr.as_ref() else {
        return Ok(None);
    };
    resolve_indicator_binding(return_expr, &helper_env, data_sources)
}

fn indicator_from_moving_average_call(
    fn_name: &str,
    method_kind: MovingAverageHelperKind,
    args: &[CallArg],
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    let decoded = decode_moving_average_args(args, env, data_sources, fn_name)?;
    let method = match method_kind {
        MovingAverageHelperKind::Sma => MovingAverageMethod::Sma,
        MovingAverageHelperKind::Ema => MovingAverageMethod::Ema,
    };

    if let Some(source) = decoded.source {
        return Ok(Some(IndicatorBinding::MovingAverage {
            source,
            period: decoded.period,
            method,
        }));
    }

    if matches!(method_kind, MovingAverageHelperKind::Ema) {
        let call_expr = Expr::Call {
            callee: Box::new(Expr::Identifier(fn_name.to_string())),
            args: args.to_vec(),
        };
        if let Some(ResolvedManualIndicatorFormula::MacdSignal {
            fast_period,
            slow_period,
            signal_period,
        }) = resolved_manual_indicator_formula(&call_expr, env)
        {
            let Some(target_expr) = manual_macd_signal_target_expr(&call_expr, env) else {
                return Ok(None);
            };
            let Some(source) = resolve_data_source_ref(target_expr, env, data_sources)? else {
                return Ok(None);
            };
            return Ok(Some(IndicatorBinding::MacdSignal {
                source,
                fast_period,
                slow_period,
                signal_period,
            }));
        }
        if let Some(IndicatorBinding::MacdLine {
            source,
            fast_period,
            slow_period,
        }) = resolve_indicator_binding(decoded.source_expr, env, data_sources)?
        {
            return Ok(Some(IndicatorBinding::MacdSignal {
                source,
                fast_period,
                slow_period,
                signal_period: decoded.period,
            }));
        }
    }

    bail!("{ERR_MOVING_AVERAGE_SOURCE_REQUIRED}: {fn_name}")
}

fn indicator_from_rsi_call(
    method_kind: RsiHelperKind,
    args: &[CallArg],
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    let decoded = decode_rsi_args(args, env, data_sources)?;
    Ok(Some(IndicatorBinding::Rsi {
        source: decoded.source,
        period: decoded.period,
        method: match method_kind {
            RsiHelperKind::Wilder => RsiMethod::Wilder,
        },
    }))
}

fn indicator_from_macd_call(
    args: &[CallArg],
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    let decoded = decode_macd_args(args, env, data_sources)?;
    Ok(Some(IndicatorBinding::Macd {
        source: decoded.source,
        fast_period: decoded.fast_period,
        slow_period: decoded.slow_period,
        signal_period: decoded.signal_period,
    }))
}

fn indicator_from_momentum_call(
    args: &[CallArg],
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    let decoded = decode_momentum_args(args, env, data_sources)?;
    Ok(Some(IndicatorBinding::Momentum {
        source: decoded.source,
        lookback: decoded.lookback,
    }))
}

fn indicator_from_zscore_call(
    args: &[CallArg],
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    let decoded = decode_zscore_args(args, env, data_sources)?;
    Ok(Some(IndicatorBinding::ZScore {
        source: decoded.source,
        window: decoded.window,
    }))
}

fn resolved_indicator_kind(env: &BindingEnv, fn_name: &str) -> Option<KnownIndicatorHelperKind> {
    match env.callables.get(fn_name).map(|callable| callable.kind) {
        Some(crate::resolve::ResolvedCallableKind::IndicatorHelper(kind)) => Some(kind),
        _ => None,
    }
}
