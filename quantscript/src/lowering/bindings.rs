use crate::diagnostics::{Diagnostic, Span};
use crate::resolve::{
    KnownIndicatorHelperKind, MovingAverageHelperKind, ResolvedCallable, ResolvedExprSemantic,
    ResolvedFunction, ResolvedManualIndicatorFormula, ResolvedSeriesCapabilityKind, RsiHelperKind,
};
use crate::script::{CallArg, Expr, FunctionDecl};
use anyhow::{anyhow, bail, Result};
use qrpc_core::DataSourceConfig;
use std::cell::RefCell;
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
    "QPQSLOW024 移动平均辅助函数需要 fetch/get_data 数据源作为第一个参数, ema(...) 可接受已识别的 MACD 线";

pub(crate) struct BindingEnv {
    pub(crate) data_by_name: BTreeMap<String, DataSourceConfig>,
    pub(crate) indicator_by_name: BTreeMap<String, IndicatorBinding>,
    pub(crate) expr_by_name: BTreeMap<String, Expr>,
    pub(crate) expr_semantics: BTreeMap<String, ResolvedExprSemantic>,
    pub(crate) callables: BTreeMap<String, ResolvedCallable>,
    pub(crate) functions: BTreeMap<String, ResolvedFunction>,
    pub(crate) diagnostics: RefCell<Vec<Diagnostic>>,
}

impl Clone for BindingEnv {
    fn clone(&self) -> Self {
        BindingEnv {
            data_by_name: self.data_by_name.clone(),
            indicator_by_name: self.indicator_by_name.clone(),
            expr_by_name: self.expr_by_name.clone(),
            expr_semantics: self.expr_semantics.clone(),
            callables: self.callables.clone(),
            functions: self.functions.clone(),
            diagnostics: RefCell::new(Vec::new()),
        }
    }
}

impl std::fmt::Debug for BindingEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BindingEnv")
            .field("data_by_name", &self.data_by_name)
            .field("indicator_by_name", &self.indicator_by_name)
            .field("expr_by_name", &self.expr_by_name)
            .field("diagnostics_count", &self.diagnostics.borrow().len())
            .finish()
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
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
    Atr {
        source: DataSourceConfig,
        period: usize,
    },
    BollingerBands {
        source: DataSourceConfig,
        period: usize,
        multiplier: f64,
    },
    Obv {
        source: DataSourceConfig,
    },
    Cmf {
        source: DataSourceConfig,
        period: usize,
    },
    Adx {
        source: DataSourceConfig,
        period: usize,
    },
    Stochastic {
        source: DataSourceConfig,
        k_period: usize,
        d_period: usize,
    },
    Cci {
        source: DataSourceConfig,
        period: usize,
    },
    ParabolicSar {
        source: DataSourceConfig,
        step: f64,
        max_step: f64,
    },
    KeltnerChannel {
        source: DataSourceConfig,
        period: usize,
        multiplier: f64,
    },
    DonchianChannel {
        source: DataSourceConfig,
        period: usize,
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
) -> Result<(BindingEnv, Vec<Diagnostic>)> {
    let mut env = empty_binding_env(functions, expr_semantics, callables);
    collect_bindings_from_stmts(&strategy.body, &mut env, data_sources)?;
    let diagnostics = env.diagnostics.take();
    Ok((env, diagnostics))
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
        Some(KnownIndicatorHelperKind::Atr) => {
            indicator_from_atr_call(args, env, data_sources)
        }
        Some(KnownIndicatorHelperKind::BollingerBands) => {
            indicator_from_bollinger_call(args, env, data_sources)
        }
        Some(KnownIndicatorHelperKind::Obv) => {
            indicator_from_obv_call(args, env, data_sources)
        }
        Some(KnownIndicatorHelperKind::Cmf) => {
            indicator_from_cmf_call(args, env, data_sources)
        }
        Some(KnownIndicatorHelperKind::Adx) => {
            indicator_from_generic_call(args, env, data_sources, "adx", |s, p| {
                IndicatorBinding::Adx { source: s, period: p }
            })
        }
        Some(KnownIndicatorHelperKind::Stochastic) => {
            indicator_from_stoch_call(args, env, data_sources)
        }
        Some(KnownIndicatorHelperKind::Cci) => {
            indicator_from_generic_call(args, env, data_sources, "cci", |s, p| {
                IndicatorBinding::Cci { source: s, period: p }
            })
        }
        Some(KnownIndicatorHelperKind::ParabolicSar) => {
            indicator_from_psar_call(args, env, data_sources)
        }
        Some(KnownIndicatorHelperKind::KeltnerChannel) => {
            indicator_from_generic_call(args, env, data_sources, "keltner", |s, p| {
                IndicatorBinding::KeltnerChannel { source: s, period: p, multiplier: 2.0 }
            })
        }
        Some(KnownIndicatorHelperKind::DonchianChannel) => {
            indicator_from_generic_call(args, env, data_sources, "donchian", |s, p| {
                IndicatorBinding::DonchianChannel { source: s, period: p }
            })
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

fn indicator_from_atr_call(
    args: &[CallArg],
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    let source = binding_source_from_arg(args.first(), env, data_sources)?;
    let period = arg_as_usize(args.get(1), env)?.unwrap_or(14);
    Ok(Some(IndicatorBinding::Atr { source, period }))
}

fn indicator_from_bollinger_call(
    args: &[CallArg],
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    let source = binding_source_from_arg(args.first(), env, data_sources)?;
    let period = arg_as_usize(args.get(1), env)?.unwrap_or(20);
    let multiplier = arg_as_f64(args.get(2)).unwrap_or(2.0);
    Ok(Some(IndicatorBinding::BollingerBands { source, period, multiplier }))
}

fn indicator_from_obv_call(
    args: &[CallArg],
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    let source = binding_source_from_arg(args.first(), env, data_sources)?;
    Ok(Some(IndicatorBinding::Obv { source }))
}

fn indicator_from_cmf_call(
    args: &[CallArg],
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    let source = binding_source_from_arg(args.first(), env, data_sources)?;
    let period = arg_as_usize(args.get(1), env)?.unwrap_or(20);
    Ok(Some(IndicatorBinding::Cmf { source, period }))
}

fn binding_source_from_arg(
    arg: Option<&CallArg>,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<DataSourceConfig> {
    let expr = arg.map(|a| &a.value).ok_or_else(|| anyhow!("缺少必需的参数"))?;
    resolve_data_source_ref(expr, env, data_sources)?
        .ok_or_else(|| anyhow!("无法从参数解析数据源"))
}

fn arg_as_usize(arg: Option<&CallArg>, env: &BindingEnv) -> Result<Option<usize>> {
    match arg {
        Some(CallArg {
            value: Expr::Number(n),
            ..
        }) => {
            if *n < 1.0 {
                anyhow::bail!("QS0504 指标周期必须 >= 1, 当前值: {}", n);
            }
            if n.fract().abs() > f64::EPSILON {
                env.diagnostics.borrow_mut().push(Diagnostic::warning(
                    "QS0502",
                    format!("指标周期 {} 将被截断为整数 {}，小数部分已忽略", n, *n as usize),
                    Some(Span::expr("period")),
                ));
            }
            Ok(Some(*n as usize))
        }
        _ => Ok(None),
    }
}

fn arg_as_f64(arg: Option<&CallArg>) -> Option<f64> {
    match arg?.value {
        Expr::Number(n) => Some(n),
        _ => None,
    }
}

fn indicator_from_generic_call<F>(
    args: &[CallArg],
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
    _name: &str,
    make: F,
) -> Result<Option<IndicatorBinding>>
where
    F: FnOnce(DataSourceConfig, usize) -> IndicatorBinding,
{
    let source = binding_source_from_arg(args.first(), env, data_sources)?;
    let period = arg_as_usize(args.get(1), env)?.unwrap_or(14);
    Ok(Some(make(source, period)))
}

fn indicator_from_stoch_call(
    args: &[CallArg],
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    let source = binding_source_from_arg(args.first(), env, data_sources)?;
    let k_period = arg_as_usize(args.get(1), env)?.unwrap_or(14);
    let d_period = arg_as_usize(args.get(2), env)?.unwrap_or(3);
    Ok(Some(IndicatorBinding::Stochastic { source, k_period, d_period }))
}

fn indicator_from_psar_call(
    args: &[CallArg],
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<IndicatorBinding>> {
    let source = binding_source_from_arg(args.first(), env, data_sources)?;
    const DEFAULT_PSAR_STEP: f64 = 0.02;
    const DEFAULT_PSAR_MAX_STEP: f64 = 0.2;
    let step = arg_as_f64(args.get(1)).unwrap_or(DEFAULT_PSAR_STEP);
    let max_step = arg_as_f64(args.get(2)).unwrap_or(DEFAULT_PSAR_MAX_STEP);
    Ok(Some(IndicatorBinding::ParabolicSar { source, step, max_step }))
}

fn resolved_indicator_kind(env: &BindingEnv, fn_name: &str) -> Option<KnownIndicatorHelperKind> {
    match env.callables.get(fn_name).map(|callable| callable.kind) {
        Some(crate::resolve::ResolvedCallableKind::IndicatorHelper(kind)) => Some(kind),
        _ => None,
    }
}
