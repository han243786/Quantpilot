use crate::resolve::{
    ChangeHelperKind, ResolvedCallable, ResolvedCallableKind, ResolvedExprSemantic,
    ResolvedFetchSourceKind, ResolvedSeriesCapabilityKind, ResolvedSeriesViewKind,
    ResolvedWindowAggregateKind,
};
use crate::script::{BinaryOp, CallArg, Expr, FunctionDecl, MatchArmBody, Stmt};
use anyhow::{anyhow, bail, Result};
use qrpc_core::{DataKind, DataSourceConfig, Exchange, MarketType, Symbol};
use std::collections::{BTreeMap, BTreeSet};

use super::bindings::BindingEnv;
use super::semantic::{
    resolve_expr_alias, resolved_change_smoothing_kind, resolved_expr_semantic,
    resolved_fetch_source_kind, resolved_window_aggregate_view, series_capability_target_expr,
    series_view_target_expr, ChangeKind, ChangeSmoothing,
};
use super::shared::{
    arg_number_optional, arg_string_optional, expr_number, find_arg, format_exchange, sanitize_id,
    ArgSelector,
};
use super::source_recovery::gain_loss_source_binding;

const ERR_INDICATOR_SOURCE_REQUIRED: &str =
    "QPQSLOW022 indicator helper requires a fetch/get_data source as its first arg";
const ERR_INDICATOR_POSITIVE_WINDOW: &str =
    "QPQSLOW023 indicator period/lookback/window arguments must be present, numeric, and greater than 0";
const ERR_MOVING_AVERAGE_SOURCE_REQUIRED: &str =
    "QPQSLOW024 moving-average helpers require a fetch/get_data source as their first arg, except ema(...) may also consume a recognized MACD line";

pub(crate) fn infer_data_sources(
    strategy: &FunctionDecl,
    callables: &BTreeMap<String, ResolvedCallable>,
) -> Result<Vec<DataSourceConfig>> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    collect_data_from_stmts(&strategy.body, &mut seen, &mut out, callables)?;
    Ok(out)
}

fn collect_data_from_stmts(
    stmts: &[Stmt],
    seen: &mut BTreeSet<String>,
    out: &mut Vec<DataSourceConfig>,
    callables: &BTreeMap<String, ResolvedCallable>,
) -> Result<()> {
    for stmt in stmts {
        match stmt {
            Stmt::Let { pattern, value, .. } => {
                if let Some(source) = extract_data_source(value, callables)? {
                    push_data_source(
                        seen,
                        out,
                        alias_data_source(source, &data_runtime_id_from_binding(pattern)),
                    );
                } else {
                    walk_expr_children(value, &mut |child| {
                        if let Ok(Some(source)) = extract_data_source(child, callables) {
                            push_data_source(seen, out, source);
                        }
                    });
                }
            }
            Stmt::Expr(value) => {
                collect_data_from_expr(value, seen, out, callables)?;
            }
            Stmt::Return(Some(value)) => collect_data_from_expr(value, seen, out, callables)?,
            Stmt::EmitIntent { args } => {
                for arg in args {
                    collect_data_from_expr(&arg.value, seen, out, callables)?;
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                collect_data_from_expr(condition, seen, out, callables)?;
                collect_data_from_stmts(then_branch, seen, out, callables)?;
                for (condition, branch) in else_if_branches {
                    collect_data_from_expr(condition, seen, out, callables)?;
                    collect_data_from_stmts(branch, seen, out, callables)?;
                }
                if let Some(branch) = else_branch {
                    collect_data_from_stmts(branch, seen, out, callables)?;
                }
            }
            Stmt::For { iterable, body, .. } => {
                collect_data_from_expr(iterable, seen, out, callables)?;
                collect_data_from_stmts(body, seen, out, callables)?;
            }
            Stmt::While { condition, body } => {
                collect_data_from_expr(condition, seen, out, callables)?;
                collect_data_from_stmts(body, seen, out, callables)?;
            }
            Stmt::Match { expr, arms } => {
                collect_data_from_expr(expr, seen, out, callables)?;
                for arm in arms {
                    match &arm.body {
                        MatchArmBody::Statement(stmt) => collect_data_from_stmts(
                            std::slice::from_ref(stmt.as_ref()),
                            seen,
                            out,
                            callables,
                        )?,
                        MatchArmBody::Expr(expr) => {
                            collect_data_from_expr(expr, seen, out, callables)?
                        }
                    }
                }
            }
            Stmt::Return(None) => {}
        }
    }
    Ok(())
}

fn push_data_source(
    seen: &mut BTreeSet<String>,
    out: &mut Vec<DataSourceConfig>,
    source: DataSourceConfig,
) {
    if seen.insert(source.data_id.clone()) {
        out.push(source);
    }
}

fn collect_data_from_expr(
    expr: &Expr,
    seen: &mut BTreeSet<String>,
    out: &mut Vec<DataSourceConfig>,
    callables: &BTreeMap<String, ResolvedCallable>,
) -> Result<()> {
    if let Some(source) = extract_data_source(expr, callables)? {
        push_data_source(seen, out, source);
    }
    walk_expr(expr, &mut |child| {
        if let Ok(Some(source)) = extract_data_source(child, callables) {
            push_data_source(seen, out, source);
        }
    });
    Ok(())
}

fn extract_data_source(
    expr: &Expr,
    callables: &BTreeMap<String, ResolvedCallable>,
) -> Result<Option<DataSourceConfig>> {
    if let Expr::Try(inner) | Expr::Await(inner) = expr {
        return extract_data_source(inner, callables);
    }
    let Some((fn_name, args)) = parse_call(expr) else {
        return Ok(None);
    };
    let Some(source_kind) = resolved_fetch_source_kind(callables, &fn_name) else {
        return Ok(None);
    };

    fetch_like_data_source_from_call(source_kind, args)
}

fn fetch_like_data_source_from_call(
    source_kind: ResolvedFetchSourceKind,
    args: &[CallArg],
) -> Result<Option<DataSourceConfig>> {
    let request = decode_fetch_request(args);
    let symbol = parse_symbol_lossy(&request.symbol_name);
    let exchange = parse_exchange_lossy(&request.exchange_name);
    let canonical_symbol = format_symbol(&symbol);
    let data_id = format!(
        "script_{}_{}_{}",
        sanitize_id(format_exchange(&exchange)),
        sanitize_id(canonical_symbol),
        sanitize_id(&request.interval)
    );

    match source_kind {
        ResolvedFetchSourceKind::KlineSeries => Ok(Some(DataSourceConfig {
            data_id,
            exchange,
            symbol,
            market_type: MarketType::Spot,
            kind: DataKind::KlineSeries,
            days: Some(request.lookback.max(1)),
            interval: Some(request.interval),
            ping_enabled: false,
            request_interval_ms: None,
            enabled: true,
        })),
    }
}

pub(crate) fn resolve_data_source_ref(
    expr: &Expr,
    env: &BindingEnv,
    _data_sources: &[DataSourceConfig],
) -> Result<Option<DataSourceConfig>> {
    match expr {
        Expr::Identifier(name) => Ok(env.data_by_name.get(name).cloned()),
        Expr::Try(inner) | Expr::Await(inner) => resolve_data_source_ref(inner, env, _data_sources),
        _ => Ok(extract_data_source(expr, &env.callables)?),
    }
}

pub(crate) fn resolved_change_kind(env: &BindingEnv, fn_name: &str) -> Option<ChangeKind> {
    match env.callables.get(fn_name).map(|callable| callable.kind) {
        Some(ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Gain)) => Some(ChangeKind::Gain),
        Some(ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Loss)) => Some(ChangeKind::Loss),
        _ => None,
    }
}

pub(crate) fn parse_call(expr: &Expr) -> Option<(String, &[CallArg])> {
    if let Expr::Call { callee, args } = expr {
        match callee.as_ref() {
            Expr::Identifier(name) => Some((name.clone(), args.as_slice())),
            Expr::Member { field, .. } => Some((field.clone(), args.as_slice())),
            _ => None,
        }
    } else {
        None
    }
}

pub(crate) fn arg_data_source_optional(
    args: &[CallArg],
    selector: ArgSelector<'_>,
    env: &BindingEnv,
    _data_sources: &[DataSourceConfig],
) -> Result<Option<DataSourceConfig>> {
    match find_arg(args, selector) {
        Some(expr) => resolve_data_source_ref(expr, env, _data_sources),
        None => Ok(None),
    }
}

pub(crate) fn arg_data_source_required(
    args: &[CallArg],
    selector: ArgSelector<'_>,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
    error_message: &str,
) -> Result<DataSourceConfig> {
    arg_data_source_optional(args, selector, env, data_sources)?
        .ok_or_else(|| anyhow!(error_message.to_string()))
}

#[derive(Debug, Clone)]
struct FetchRequest {
    symbol_name: String,
    exchange_name: String,
    interval: String,
    lookback: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct SmoothedChangeBindingArgs {
    pub(crate) source: DataSourceConfig,
    pub(crate) period: usize,
    pub(crate) smoothing: ChangeSmoothing,
}

#[derive(Debug, Clone)]
pub(crate) struct MovingAverageCallArgs<'a> {
    pub(crate) source: Option<DataSourceConfig>,
    pub(crate) source_expr: &'a Expr,
    pub(crate) period: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct RsiCallArgs {
    pub(crate) source: DataSourceConfig,
    pub(crate) period: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct MacdCallArgs {
    pub(crate) source: DataSourceConfig,
    pub(crate) fast_period: usize,
    pub(crate) slow_period: usize,
    pub(crate) signal_period: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct MomentumCallArgs {
    pub(crate) source: DataSourceConfig,
    pub(crate) lookback: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct ZScoreCallArgs {
    pub(crate) source: DataSourceConfig,
    pub(crate) window: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct SourcePeriodSmoothingMatch {
    pub(crate) source: DataSourceConfig,
    pub(crate) period: usize,
    pub(crate) smoothing: ChangeSmoothing,
}

#[derive(Debug, Clone)]
pub(crate) struct SourceSpanMatch {
    pub(crate) source: DataSourceConfig,
    pub(crate) span: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct SeriesViewMatch {
    pub(crate) source: DataSourceConfig,
    pub(crate) access: SeriesViewAccess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SeriesViewAccess {
    Current,
    First,
    Lookback(usize),
    Window(usize),
}

fn decode_fetch_request(args: &[CallArg]) -> FetchRequest {
    let symbol_name = arg_string_optional(args, ArgSelector::Positional(0))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "BTCUSDT".to_string());
    let exchange_name = arg_string_optional(args, ArgSelector::Named("exchange"))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "binance".to_string());
    let interval = arg_string_optional(args, ArgSelector::Named("interval"))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "1d".to_string());
    let lookback = arg_number_optional(args, ArgSelector::Named("lookback"))
        .map(|value| value.round().max(1.0) as u32)
        .unwrap_or(200);

    FetchRequest {
        symbol_name,
        exchange_name,
        interval,
        lookback,
    }
}

pub(crate) fn decode_moving_average_args<'a>(
    args: &'a [CallArg],
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
    fn_name: &str,
) -> Result<MovingAverageCallArgs<'a>> {
    let source_expr = find_arg(args, ArgSelector::Positional(0))
        .ok_or_else(|| anyhow!(ERR_MOVING_AVERAGE_SOURCE_REQUIRED))?;
    let source = resolve_data_source_ref(source_expr, env, data_sources)?;
    let period = decode_positive_usize_arg(
        args,
        ArgSelector::Positional(1),
        &format!("{fn_name} period"),
    )?;

    Ok(MovingAverageCallArgs {
        source,
        source_expr,
        period,
    })
}

pub(crate) fn decode_rsi_args(
    args: &[CallArg],
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<RsiCallArgs> {
    let source = arg_data_source_required(
        args,
        ArgSelector::Positional(0),
        env,
        data_sources,
        ERR_INDICATOR_SOURCE_REQUIRED,
    )?;
    let period = decode_positive_usize_arg(args, ArgSelector::Positional(1), "rsi period")?;

    Ok(RsiCallArgs { source, period })
}

pub(crate) fn decode_macd_args(
    args: &[CallArg],
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<MacdCallArgs> {
    let source = arg_data_source_required(
        args,
        ArgSelector::Positional(0),
        env,
        data_sources,
        ERR_INDICATOR_SOURCE_REQUIRED,
    )?;
    let fast_period =
        decode_positive_usize_arg(args, ArgSelector::Positional(1), "macd fast period")?;
    let slow_period =
        decode_positive_usize_arg(args, ArgSelector::Positional(2), "macd slow period")?;
    let signal_period =
        decode_positive_usize_arg(args, ArgSelector::Positional(3), "macd signal period")?;

    Ok(MacdCallArgs {
        source,
        fast_period,
        slow_period,
        signal_period,
    })
}

pub(crate) fn decode_momentum_args(
    args: &[CallArg],
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<MomentumCallArgs> {
    let source = arg_data_source_required(
        args,
        ArgSelector::Positional(0),
        env,
        data_sources,
        ERR_INDICATOR_SOURCE_REQUIRED,
    )?;
    let lookback =
        decode_positive_usize_arg(args, ArgSelector::Positional(1), "momentum lookback")?;

    Ok(MomentumCallArgs { source, lookback })
}

pub(crate) fn decode_zscore_args(
    args: &[CallArg],
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<ZScoreCallArgs> {
    let source = arg_data_source_required(
        args,
        ArgSelector::Positional(0),
        env,
        data_sources,
        ERR_INDICATOR_SOURCE_REQUIRED,
    )?;
    let window = decode_positive_usize_arg(args, ArgSelector::Positional(1), "zscore window")?;

    Ok(ZScoreCallArgs { source, window })
}

pub(crate) fn decode_smoothed_change_binding(
    expr: &Expr,
    kind: ChangeKind,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<SmoothedChangeBindingArgs>> {
    let expr = resolve_expr_alias(expr, env).unwrap_or(expr);
    let Some((fn_name, args)) = parse_call(expr) else {
        return Ok(None);
    };
    let Some(smoothing) = resolved_change_smoothing_kind(env, &fn_name) else {
        return Ok(None);
    };
    if args.len() < 2 {
        return Ok(None);
    }

    let Some(source_expr) = find_arg(args, ArgSelector::Positional(0)) else {
        return Ok(None);
    };
    let Some((source, change_kind)) = gain_loss_source_binding(source_expr, env, data_sources)?
    else {
        return Ok(None);
    };
    if change_kind != kind {
        return Ok(None);
    }

    let Some(period_value) = arg_number_optional(args, ArgSelector::Positional(1)) else {
        return Ok(None);
    };
    let period = period_value.round() as usize;
    if period == 0 {
        return Ok(None);
    }

    Ok(Some(SmoothedChangeBindingArgs {
        source,
        period,
        smoothing,
    }))
}

pub(crate) fn decode_window_binding(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
    aggregate_kind: ResolvedWindowAggregateKind,
) -> Result<Option<SourceSpanMatch>> {
    if let Some(window_aggregate) = resolved_window_aggregate_view(expr, env) {
        if window_aggregate.aggregate_kind == aggregate_kind {
            let Some(target_expr) = series_capability_target_expr(expr, env) else {
                return Ok(None);
            };
            let Some(windowed) = decode_series_window_view(target_expr, env, data_sources)? else {
                return Ok(None);
            };
            return Ok(Some(SourceSpanMatch {
                source: windowed.source,
                span: window_aggregate.span,
            }));
        }
    }

    if !matches!(
        resolved_expr_semantic(expr, env),
        Some(ResolvedExprSemantic::SeriesCapability(
            ResolvedSeriesCapabilityKind::WindowAggregate(kind)
        )) if kind == aggregate_kind
    ) {
        return Ok(None);
    }

    let Some(target_expr) = series_capability_target_expr(expr, env) else {
        return Ok(None);
    };
    decode_series_window_view(target_expr, env, data_sources)
}

pub(crate) fn decode_series_view(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<SeriesViewMatch>> {
    let Some(view_kind) = (match resolved_expr_semantic(expr, env) {
        Some(ResolvedExprSemantic::SeriesView(view_kind)) => Some(view_kind),
        _ => None,
    }) else {
        return Ok(None);
    };

    let Some(target_expr) = series_view_target_expr(expr, env) else {
        return Ok(None);
    };
    let Some(source) = resolve_data_source_ref(target_expr, env, data_sources)? else {
        return Ok(None);
    };

    let access = match view_kind {
        ResolvedSeriesViewKind::Current => SeriesViewAccess::Current,
        ResolvedSeriesViewKind::First => SeriesViewAccess::First,
        ResolvedSeriesViewKind::Lookback(span) => SeriesViewAccess::Lookback(span),
        ResolvedSeriesViewKind::Window(span) => SeriesViewAccess::Window(span),
    };

    Ok(Some(SeriesViewMatch { source, access }))
}

pub(crate) fn decode_series_window_view(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<SourceSpanMatch>> {
    let Some(view) = decode_series_view(expr, env, data_sources)? else {
        return Ok(None);
    };
    match view.access {
        SeriesViewAccess::Window(span) => Ok(Some(SourceSpanMatch {
            source: view.source,
            span,
        })),
        SeriesViewAccess::Current | SeriesViewAccess::First | SeriesViewAccess::Lookback(_) => {
            Ok(None)
        }
    }
}

pub(crate) fn decode_series_position_view(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<(DataSourceConfig, SeriesViewAccess)>> {
    let Some(view) = decode_series_view(expr, env, data_sources)? else {
        return Ok(None);
    };
    match view.access {
        SeriesViewAccess::Current | SeriesViewAccess::First | SeriesViewAccess::Lookback(_) => {
            Ok(Some((view.source, view.access)))
        }
        SeriesViewAccess::Window(_) => Ok(None),
    }
}

fn decode_positive_usize_arg(
    args: &[CallArg],
    selector: ArgSelector<'_>,
    label: &str,
) -> Result<usize> {
    let expr = find_arg(args, selector)
        .ok_or_else(|| anyhow!("{ERR_INDICATOR_POSITIVE_WINDOW}: {label}"))?;
    let value =
        expr_number(expr).ok_or_else(|| anyhow!("{ERR_INDICATOR_POSITIVE_WINDOW}: {label}"))?;
    let rounded = value.round();
    if rounded <= 0.0 {
        bail!("{ERR_INDICATOR_POSITIVE_WINDOW}: {label}");
    }
    Ok(rounded as usize)
}

pub(crate) fn comparison_parts(expr: &Expr) -> Option<(&Expr, BinaryOp, &Expr)> {
    if let Expr::Binary { left, op, right } = expr {
        if matches!(
            op,
            BinaryOp::Greater
                | BinaryOp::GreaterEqual
                | BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Equal
                | BinaryOp::NotEqual
        ) {
            return Some((left.as_ref(), op.clone(), right.as_ref()));
        }
    }
    None
}

pub(crate) fn walk_expr(expr: &Expr, visitor: &mut impl FnMut(&Expr)) {
    visitor(expr);
    match expr {
        Expr::Call { callee, args } => {
            walk_expr(callee, visitor);
            for arg in args {
                walk_expr(&arg.value, visitor);
            }
        }
        Expr::Member { object, .. }
        | Expr::Await(object)
        | Expr::Try(object)
        | Expr::Unary { expr: object, .. } => walk_expr(object, visitor),
        Expr::Index { object, index } => {
            walk_expr(object, visitor);
            walk_expr(index, visitor);
        }
        Expr::Slice { object, start, end } => {
            walk_expr(object, visitor);
            if let Some(start) = start {
                walk_expr(start, visitor);
            }
            if let Some(end) = end {
                walk_expr(end, visitor);
            }
        }
        Expr::Binary { left, right, .. }
        | Expr::Range {
            start: left,
            end: right,
        } => {
            walk_expr(left, visitor);
            walk_expr(right, visitor);
        }
        Expr::List(items) => {
            for item in items {
                walk_expr(item, visitor);
            }
        }
        Expr::Raw(_) | Expr::Identifier(_) | Expr::Number(_) | Expr::String(_) | Expr::Bool(_) => {}
    }
}

fn walk_expr_children(expr: &Expr, visitor: &mut impl FnMut(&Expr)) {
    match expr {
        Expr::Call { callee, args } => {
            walk_expr(callee, visitor);
            for arg in args {
                walk_expr(&arg.value, visitor);
            }
        }
        Expr::Member { object, .. }
        | Expr::Await(object)
        | Expr::Try(object)
        | Expr::Unary { expr: object, .. } => walk_expr(object, visitor),
        Expr::Index { object, index } => {
            walk_expr(object, visitor);
            walk_expr(index, visitor);
        }
        Expr::Slice { object, start, end } => {
            walk_expr(object, visitor);
            if let Some(start) = start {
                walk_expr(start, visitor);
            }
            if let Some(end) = end {
                walk_expr(end, visitor);
            }
        }
        Expr::Binary { left, right, .. }
        | Expr::Range {
            start: left,
            end: right,
        } => {
            walk_expr(left, visitor);
            walk_expr(right, visitor);
        }
        Expr::List(items) => {
            for item in items {
                walk_expr(item, visitor);
            }
        }
        Expr::Raw(_) | Expr::Identifier(_) | Expr::Number(_) | Expr::String(_) | Expr::Bool(_) => {}
    }
}

fn alias_data_source(mut source: DataSourceConfig, runtime_id: &str) -> DataSourceConfig {
    if !runtime_id.is_empty() {
        source.data_id = runtime_id.to_string();
    }
    source
}

fn data_runtime_id_from_binding(pattern: &str) -> String {
    let sanitized = sanitize_id(pattern);
    if sanitized.starts_with("data_") && sanitized.ends_with("_series") {
        sanitized
            .strip_suffix("_series")
            .unwrap_or(&sanitized)
            .to_string()
    } else {
        String::new()
    }
}

pub(crate) fn parse_symbol_lossy(value: &str) -> Symbol {
    Symbol::parse(value)
}

fn parse_exchange_lossy(value: &str) -> Exchange {
    match value.to_ascii_lowercase().as_str() {
        "okx" => Exchange::Okx,
        _ => Exchange::Binance,
    }
}

pub(crate) fn format_symbol(symbol: &Symbol) -> &str {
    symbol.as_str()
}

#[cfg(test)]
mod tests {
    use super::super::bindings::collect_bindings;
    use super::super::diagnostics::format_diagnostics;
    use super::*;
    use crate::evaluator::normalize_script_module;
    use crate::parse_quant_script_module;
    use crate::resolve::lower_script_to_typed_hir;
    use crate::script::{FunctionDecl, Item};

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
        let bindings = collect_bindings(
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

    #[test]
    fn decodes_series_view_matchers_directly() {
        let (strategy, data_sources, bindings) = prepare_strategy_bindings(
            r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let latest = closes[0]
    let first_close = first(closes)
    let last_close = closes.last()
    let scope = closes[20..]
}
"#,
        );

        let latest = find_let_expr(&strategy.body, "latest");
        let first_close = find_let_expr(&strategy.body, "first_close");
        let last_close = find_let_expr(&strategy.body, "last_close");
        let scope = find_let_expr(&strategy.body, "scope");

        let latest_view = decode_series_position_view(latest, &bindings, &data_sources)
            .unwrap()
            .unwrap();
        let first_view = decode_series_position_view(first_close, &bindings, &data_sources)
            .unwrap()
            .unwrap();
        let last_view = decode_series_position_view(last_close, &bindings, &data_sources)
            .unwrap()
            .unwrap();
        let scope_view = decode_series_window_view(scope, &bindings, &data_sources)
            .unwrap()
            .unwrap();

        assert_eq!(latest_view.0.data_id, "script_binance_btcusdt_1d");
        assert_eq!(latest_view.1, SeriesViewAccess::Current);
        assert_eq!(first_view.1, SeriesViewAccess::First);
        assert_eq!(last_view.1, SeriesViewAccess::Current);
        assert_eq!(scope_view.source.data_id, "script_binance_btcusdt_1d");
        assert_eq!(scope_view.span, 20);
    }
}
