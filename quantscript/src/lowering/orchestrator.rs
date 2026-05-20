use crate::analysis::analyze_script_module;
use crate::evaluator::normalize_script_module;
use crate::resolve::lower_script_to_typed_hir;
use crate::script::{CallArg, Expr, FunctionDecl, Item, ScriptModule, Stmt};
use anyhow::{anyhow, bail, Context, Result};
use std::collections::BTreeSet;
use qrpc_core::{
    AgentConfig, RiskConfig, RuntimeProtocolCoreConfig,
    GLOBAL_RISK_PROFILE_DEFAULT_MAX_EXCHANGE_LEVERAGE, GLOBAL_RISK_PROFILE_DEFAULT_MAX_POSITION,
    GLOBAL_RISK_PROFILE_DEFAULT_MAX_TOTAL_LEVERAGE,
    GLOBAL_RISK_PROFILE_DEFAULT_MIN_ACTION_INTERVAL_MS, GLOBAL_RISK_PROFILE_ID,
    PAPER_EXECUTION_PROFILE_DEFAULT_FEE_BPS, PAPER_EXECUTION_PROFILE_DEFAULT_SLIPPAGE_BPS,
    PAPER_EXECUTION_PROFILE_ID,
};

use super::binding_sources::infer_data_sources;
use super::bindings::collect_bindings;
use super::context::LoweringContext;
use super::diagnostics::format_diagnostics;
use super::intents::{canonicalize_data_sources, infer_intents, inferred_agent_params};
use super::universe::{detect_portfolio_rebalance_directive, expand_universe_constructs};

#[cfg(test)]
use qrpc_core::{Exchange, IntentKind, RebalanceSchedule, Symbol};

const ERR_MISSING_STRATEGY_FN: &str = "QPQSLOW006 QuantScript 必须声明 fn strategy() 入口函数。请在 .qs 文件中添加: fn strategy() { ... }";
const ERR_NO_FETCH_CALLS: &str =
    "QPQSLOW007 策略编译需要至少一个 fetch/get_data 调用";

#[derive(Debug, Clone)]
struct GlobalRiskProfileSpec {
    max_position_ratio: f64,
    max_total_leverage: f64,
    max_exchange_leverage: f64,
    min_action_interval_ms: u64,
}

#[derive(Debug, Clone)]
struct PaperExecutionProfileSpec {
    fee_bps: f64,
    slippage_bps: f64,
}

pub fn lower_script_to_runtime_config(module: &ScriptModule) -> Result<RuntimeProtocolCoreConfig> {
    lower_script_to_runtime_config_with_context(module, &LoweringContext::default())
}

pub fn lower_script_to_runtime_config_with_context(
    module: &ScriptModule,
    context: &LoweringContext,
) -> Result<RuntimeProtocolCoreConfig> {
    let normalized_module = normalize_script_module(module)
        .context("QPQSLOW000 脚本标准化失败")?;
    let rebalance_directive = detect_portfolio_rebalance_directive(&normalized_module, context)?;
    let expanded_module = expand_universe_constructs(&normalized_module, context)?;
    let resolved = lower_script_to_typed_hir(&expanded_module);
    let analysis = analyze_script_module(&expanded_module, &resolved);
    let mut diagnostics = resolved.diagnostics.clone();
    diagnostics.extend(analysis.diagnostics);
    let has_errors = diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == crate::DiagnosticSeverity::Error);
    if has_errors {
        bail!(
            "QuantScript 语义分析失败:\n{}",
            format_diagnostics(&diagnostics)
        );
    }
    let strategy = expanded_module
        .items
        .iter()
        .find_map(|item| match item {
            Item::Function(function) if function.name == "strategy" => Some(function),
            _ => None,
        })
        .ok_or_else(|| anyhow!(ERR_MISSING_STRATEGY_FN))?;
    let risk_profile = detect_global_risk_profile(strategy)?;
    let execution_profile = detect_paper_execution_profile(strategy)?;

    let mut inferred_data_sources = infer_data_sources(strategy, &resolved.callables)?;
    // B1-13: fetch 去重
    {
        let mut seen = BTreeSet::new();
        inferred_data_sources.retain(|ds| {
            let key = format!(
                "{}:{:?}:{}",
                ds.symbol.as_str(),
                ds.exchange,
                ds.interval.as_deref().unwrap_or("")
            );
            seen.insert(key)
        });
    }
    if inferred_data_sources.is_empty() {
        bail!(ERR_NO_FETCH_CALLS);
    }

    let (bindings, binding_diagnostics) = collect_bindings(
        strategy,
        &inferred_data_sources,
        resolved.functions.clone(),
        resolved.expr_semantics.clone(),
        resolved.callables.clone(),
    )?;
    diagnostics.extend(binding_diagnostics);
    let data_sources = canonicalize_data_sources(&inferred_data_sources, &bindings);
    let intents = infer_intents(strategy, &bindings, &data_sources)?;

    let agent = AgentConfig {
        agent_id: "agent_script_main".into(),
        name: "Script Main Agent".into(),
        input_intent_ids: intents
            .iter()
            .map(|intent| intent.intent_id.clone())
            .collect(),
        rebalance_symbols: rebalance_directive
            .as_ref()
            .map(|directive| directive.symbols.clone())
            .unwrap_or_default(),
        rebalance_schedule: rebalance_directive
            .as_ref()
            .and_then(|directive| directive.schedule.clone()),
        rebalance_allocation_kind: rebalance_directive
            .as_ref()
            .map(|directive| directive.allocation_kind.clone()),
        rebalance_rank_method: rebalance_directive
            .as_ref()
            .and_then(|directive| directive.rank_method.clone()),
        rebalance_score_normalize: rebalance_directive
            .as_ref()
            .and_then(|directive| directive.score_normalize.clone()),
        rebalance_target_weights: rebalance_directive
            .as_ref()
            .map(|directive| directive.target_weights.clone())
            .unwrap_or_default(),
        params: inferred_agent_params(&intents, rebalance_directive.as_ref()),
        enabled: true,
    };

    let max_position_ratio = risk_profile
        .as_ref()
        .map(|profile| profile.max_position_ratio)
        .unwrap_or_else(|| {
            rebalance_directive
                .as_ref()
                .map(|_| 1.0)
                .unwrap_or(GLOBAL_RISK_PROFILE_DEFAULT_MAX_POSITION)
        });
    let risk = RiskConfig {
        risk_id: "risk_script_global".into(),
        name: "Script Global Risk".into(),
        observed_agent_ids: vec![agent.agent_id.clone()],
        max_position_ratio,
        max_single_weight: None,
        max_concentration_ratio: None,
        max_symbol_net_exposure_ratio: None,
        max_portfolio_net_exposure_ratio: None,
        max_turnover: None,
        min_trade_weight: None,
        max_new_positions_per_rebalance: None,
        max_total_leverage: risk_profile
            .as_ref()
            .map(|profile| profile.max_total_leverage)
            .unwrap_or(GLOBAL_RISK_PROFILE_DEFAULT_MAX_TOTAL_LEVERAGE),
        max_exchange_leverage: risk_profile
            .as_ref()
            .map(|profile| profile.max_exchange_leverage)
            .unwrap_or(GLOBAL_RISK_PROFILE_DEFAULT_MAX_EXCHANGE_LEVERAGE),
        min_action_interval_ms: risk_profile
            .as_ref()
            .map(|profile| profile.min_action_interval_ms)
            .unwrap_or(GLOBAL_RISK_PROFILE_DEFAULT_MIN_ACTION_INTERVAL_MS),
        enabled: true,
    };

    Ok(RuntimeProtocolCoreConfig {
        data_sources,
        intents,
        agents: vec![agent],
        risks: vec![risk],
        initial_cash_balance: 100_000.0,
        taker_fee_bps: execution_profile
            .as_ref()
            .map(|profile| profile.fee_bps)
            .unwrap_or(PAPER_EXECUTION_PROFILE_DEFAULT_FEE_BPS),
        default_slippage_bps: execution_profile
            .as_ref()
            .map(|profile| profile.slippage_bps)
            .unwrap_or(PAPER_EXECUTION_PROFILE_DEFAULT_SLIPPAGE_BPS),
        total_cost_buffer_bps: 20.0,
    })
}

fn detect_global_risk_profile(strategy: &FunctionDecl) -> Result<Option<GlobalRiskProfileSpec>> {
    let mut detected = None;
    for stmt in &strategy.body {
        if let Some(call_args) = risk_profile_call_args(stmt) {
            if detected.is_some() {
                bail!("QuantScript 当前最多支持一个 risk.profile(...) 声明");
            }
            detected = Some(parse_global_risk_profile_args(call_args)?);
            continue;
        }
        if nested_risk_profile_call(stmt) {
            bail!("risk.profile(...) 必须作为 fn strategy() 中的顶级语句出现");
        }
    }
    Ok(detected)
}

fn risk_profile_call_args(stmt: &Stmt) -> Option<&[CallArg]> {
    let Stmt::Expr(Expr::Call { callee, args }) = stmt else {
        return None;
    };
    let Expr::Member { object, field } = callee.as_ref() else {
        return None;
    };
    if !matches!(object.as_ref(), Expr::Identifier(name) if name == "risk") || field != "profile" {
        return None;
    }
    Some(args.as_slice())
}

fn parse_global_risk_profile_args(args: &[CallArg]) -> Result<GlobalRiskProfileSpec> {
    let Some(first_arg) = args.first() else {
        bail!("risk.profile(...) 需要 `profile_id` 位置参数");
    };
    if first_arg.name.is_some() {
        bail!("risk.profile(...) 要求第一个参数为位置参数 profile id");
    }
    let Expr::String(profile_id) = &first_arg.value else {
        bail!("risk.profile(...) profile_id 必须是字符串字面量");
    };
    if profile_id != GLOBAL_RISK_PROFILE_ID {
        bail!(
            "risk.profile(...) 当前只支持 profile_id=\"{}\"",
            GLOBAL_RISK_PROFILE_ID
        );
    }

    let mut spec = GlobalRiskProfileSpec {
        max_position_ratio: GLOBAL_RISK_PROFILE_DEFAULT_MAX_POSITION,
        max_total_leverage: GLOBAL_RISK_PROFILE_DEFAULT_MAX_TOTAL_LEVERAGE,
        max_exchange_leverage: GLOBAL_RISK_PROFILE_DEFAULT_MAX_EXCHANGE_LEVERAGE,
        min_action_interval_ms: GLOBAL_RISK_PROFILE_DEFAULT_MIN_ACTION_INTERVAL_MS,
    };

    for arg in &args[1..] {
        let Some(name) = arg.name.as_deref() else {
            bail!("risk.profile(...) 在 profile_id 之后只支持具名关键字字段");
        };
        match name {
            "max_position" => {
                spec.max_position_ratio = risk_profile_number_field(name, &arg.value)?;
                if !spec.max_position_ratio.is_finite() || spec.max_position_ratio <= 0.0 {
                    bail!("risk.profile(..., max_position=...) 必须大于 0");
                }
            }
            "max_total_leverage" => {
                spec.max_total_leverage = risk_profile_number_field(name, &arg.value)?;
                if spec.max_total_leverage < 1.0 {
                    bail!("risk.profile(..., max_total_leverage=...) 必须大于等于 1");
                }
            }
            "max_exchange_leverage" => {
                spec.max_exchange_leverage = risk_profile_number_field(name, &arg.value)?;
                if spec.max_exchange_leverage < 1.0 {
                    bail!("risk.profile(..., max_exchange_leverage=...) 必须大于等于 1");
                }
            }
            "min_action_interval_ms" => {
                let value = risk_profile_number_field(name, &arg.value)?;
                if value < 0.0 || value.fract().abs() > f64::EPSILON {
                    bail!("risk.profile(..., min_action_interval_ms=...) 必须是非负整数");
                }
                spec.min_action_interval_ms = value as u64;
            }
            other => bail!(
                "risk.profile(...) 不支持关键字字段 `{}` 在当前运行时中",
                other
            ),
        }
    }

    Ok(spec)
}

fn detect_paper_execution_profile(
    strategy: &FunctionDecl,
) -> Result<Option<PaperExecutionProfileSpec>> {
    let mut detected = None;
    for stmt in &strategy.body {
        if let Some(call_args) = execution_profile_call_args(stmt) {
            if detected.is_some() {
                bail!("QuantScript 当前最多支持一个 execution.profile(...) 声明");
            }
            detected = Some(parse_paper_execution_profile_args(call_args)?);
            continue;
        }
        if nested_execution_profile_call(stmt) {
            bail!("execution.profile(...) 必须作为 fn strategy() 中的顶级语句出现");
        }
    }
    Ok(detected)
}

fn execution_profile_call_args(stmt: &Stmt) -> Option<&[CallArg]> {
    let Stmt::Expr(Expr::Call { callee, args }) = stmt else {
        return None;
    };
    let Expr::Member { object, field } = callee.as_ref() else {
        return None;
    };
    if !matches!(object.as_ref(), Expr::Identifier(name) if name == "execution")
        || field != "profile"
    {
        return None;
    }
    Some(args.as_slice())
}

fn parse_paper_execution_profile_args(args: &[CallArg]) -> Result<PaperExecutionProfileSpec> {
    let Some(first_arg) = args.first() else {
        bail!("execution.profile(...) 需要 `profile_id` 位置参数");
    };
    if first_arg.name.is_some() {
        bail!("execution.profile(...) 要求第一个参数为位置参数 profile id");
    }
    let Expr::String(profile_id) = &first_arg.value else {
        bail!("execution.profile(...) profile_id 必须是字符串字面量");
    };
    if profile_id != PAPER_EXECUTION_PROFILE_ID {
        bail!(
            "execution.profile(...) 当前只支持 profile_id=\"{}\"",
            PAPER_EXECUTION_PROFILE_ID
        );
    }

    let mut spec = PaperExecutionProfileSpec {
        fee_bps: PAPER_EXECUTION_PROFILE_DEFAULT_FEE_BPS,
        slippage_bps: PAPER_EXECUTION_PROFILE_DEFAULT_SLIPPAGE_BPS,
    };

    for arg in &args[1..] {
        let Some(name) = arg.name.as_deref() else {
            bail!("execution.profile(...) 在 profile_id 之后只支持具名关键字字段");
        };
        match name {
            "fee_bps" => {
                spec.fee_bps = execution_profile_number_field(name, &arg.value)?;
                if spec.fee_bps < 0.0 {
                    bail!("execution.profile(..., fee_bps=...) 必须大于等于 0");
                }
            }
            "slippage_bps" => {
                spec.slippage_bps = execution_profile_number_field(name, &arg.value)?;
                if spec.slippage_bps < 0.0 {
                    bail!("execution.profile(..., slippage_bps=...) 必须大于等于 0");
                }
            }
            other => bail!(
                "execution.profile(...) 不支持关键字字段 `{}` 在当前运行时中",
                other
            ),
        }
    }

    Ok(spec)
}

fn execution_profile_number_field(name: &str, expr: &Expr) -> Result<f64> {
    let Expr::Number(value) = expr else {
        bail!(
            "execution.profile(..., {}=...) 必须是数值字面量",
            name
        );
    };
    Ok(*value)
}

fn risk_profile_number_field(name: &str, expr: &Expr) -> Result<f64> {
    let Expr::Number(value) = expr else {
        bail!("risk.profile(..., {}=...) 必须是数值字面量", name);
    };
    Ok(*value)
}

fn nested_risk_profile_call(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Let { value, .. } | Stmt::Expr(value) | Stmt::Return(Some(value)) => {
            expr_contains_risk_profile(value)
        }
        Stmt::Return(None) | Stmt::EmitIntent { .. } => false,
        Stmt::If {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
        } => {
            expr_contains_risk_profile(condition)
                || then_branch.iter().any(nested_risk_profile_call)
                || else_if_branches.iter().any(|(expr, body)| {
                    expr_contains_risk_profile(expr) || body.iter().any(nested_risk_profile_call)
                })
                || else_branch
                    .as_ref()
                    .map(|branch| branch.iter().any(nested_risk_profile_call))
                    .unwrap_or(false)
        }
        Stmt::For { iterable, body, .. } => {
            expr_contains_risk_profile(iterable) || body.iter().any(nested_risk_profile_call)
        }
        Stmt::While { condition, body } => {
            expr_contains_risk_profile(condition) || body.iter().any(nested_risk_profile_call)
        }
        Stmt::Match { expr, arms } => {
            expr_contains_risk_profile(expr)
                || arms.iter().any(|arm| match &arm.body {
                    crate::script::MatchArmBody::Statement(stmt) => nested_risk_profile_call(stmt),
                    crate::script::MatchArmBody::Expr(expr) => expr_contains_risk_profile(expr),
                })
        }
    }
}

fn nested_execution_profile_call(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Let { value, .. } | Stmt::Expr(value) | Stmt::Return(Some(value)) => {
            expr_contains_execution_profile(value)
        }
        Stmt::Return(None) | Stmt::EmitIntent { .. } => false,
        Stmt::If {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
        } => {
            expr_contains_execution_profile(condition)
                || then_branch.iter().any(nested_execution_profile_call)
                || else_if_branches.iter().any(|(expr, body)| {
                    expr_contains_execution_profile(expr)
                        || body.iter().any(nested_execution_profile_call)
                })
                || else_branch
                    .as_ref()
                    .map(|branch| branch.iter().any(nested_execution_profile_call))
                    .unwrap_or(false)
        }
        Stmt::For { iterable, body, .. } => {
            expr_contains_execution_profile(iterable)
                || body.iter().any(nested_execution_profile_call)
        }
        Stmt::While { condition, body } => {
            expr_contains_execution_profile(condition)
                || body.iter().any(nested_execution_profile_call)
        }
        Stmt::Match { expr, arms } => {
            expr_contains_execution_profile(expr)
                || arms.iter().any(|arm| match &arm.body {
                    crate::script::MatchArmBody::Statement(stmt) => {
                        nested_execution_profile_call(stmt)
                    }
                    crate::script::MatchArmBody::Expr(expr) => {
                        expr_contains_execution_profile(expr)
                    }
                })
        }
    }
}

fn expr_contains_risk_profile(expr: &Expr) -> bool {
    match expr {
        Expr::Call { callee, args } => {
            if matches!(
                callee.as_ref(),
                Expr::Member { object, field }
                    if matches!(object.as_ref(), Expr::Identifier(name) if name == "risk") && field == "profile"
            ) {
                return true;
            }
            expr_contains_risk_profile(callee)
                || args
                    .iter()
                    .any(|arg| expr_contains_risk_profile(&arg.value))
        }
        Expr::Member { object, .. }
        | Expr::Index { object, .. }
        | Expr::Unary { expr: object, .. }
        | Expr::Try(object)
        | Expr::Await(object) => expr_contains_risk_profile(object),
        Expr::Slice { object, start, end } => {
            expr_contains_risk_profile(object)
                || start
                    .as_ref()
                    .map(|expr| expr_contains_risk_profile(expr))
                    .unwrap_or(false)
                || end
                    .as_ref()
                    .map(|expr| expr_contains_risk_profile(expr))
                    .unwrap_or(false)
        }
        Expr::Binary { left, right, .. } => {
            expr_contains_risk_profile(left) || expr_contains_risk_profile(right)
        }
        Expr::Range { start, end } => {
            expr_contains_risk_profile(start) || expr_contains_risk_profile(end)
        }
        Expr::List(items) => items.iter().any(expr_contains_risk_profile),
        Expr::Raw(_) | Expr::Identifier(_) | Expr::String(_) | Expr::Number(_) | Expr::Bool(_) => {
            false
        }
    }
}

fn expr_contains_execution_profile(expr: &Expr) -> bool {
    match expr {
        Expr::Call { callee, args } => {
            if matches!(
                callee.as_ref(),
                Expr::Member { object, field }
                    if matches!(object.as_ref(), Expr::Identifier(name) if name == "execution") && field == "profile"
            ) {
                return true;
            }
            expr_contains_execution_profile(callee)
                || args
                    .iter()
                    .any(|arg| expr_contains_execution_profile(&arg.value))
        }
        Expr::Member { object, .. }
        | Expr::Index { object, .. }
        | Expr::Unary { expr: object, .. }
        | Expr::Try(object)
        | Expr::Await(object) => expr_contains_execution_profile(object),
        Expr::Slice { object, start, end } => {
            expr_contains_execution_profile(object)
                || start
                    .as_ref()
                    .map(|expr| expr_contains_execution_profile(expr))
                    .unwrap_or(false)
                || end
                    .as_ref()
                    .map(|expr| expr_contains_execution_profile(expr))
                    .unwrap_or(false)
        }
        Expr::Binary { left, right, .. } => {
            expr_contains_execution_profile(left) || expr_contains_execution_profile(right)
        }
        Expr::Range { start, end } => {
            expr_contains_execution_profile(start) || expr_contains_execution_profile(end)
        }
        Expr::List(items) => items.iter().any(expr_contains_execution_profile),
        Expr::Raw(_) | Expr::Identifier(_) | Expr::String(_) | Expr::Number(_) | Expr::Bool(_) => {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_quant_script_module;
    use qrpc_compiler::compile_runtime_protocol_config;
    use qrpc_runtime::RuntimeCoordinator;

    const MA_SCRIPT: &str = r#"
import math
from data import fetch as get_data
from signals@1.2 import sma

fn strategy() {
    let closes = get_data("BTCUSDT", interval="1d", lookback=200)?
    let fast = sma(closes, 20)
    let slow = sma(closes, 60)

    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#;

    const RSI_SCRIPT: &str = r#"
from data import fetch as get_data
from signals import rsi

fn strategy() {
    let closes = get_data("BTCUSDT", interval="1d", lookback=200)?
    let r = rsi(closes, 14)
    if r < 30 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else if r > 70 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#;

    #[test]
    fn lowers_ma_cross_into_runtime_config() {
        let module = parse_quant_script_module(MA_SCRIPT).unwrap();
        let config = lower_script_to_runtime_config(&module).unwrap();
        assert_eq!(config.data_sources.len(), 1);
        assert_eq!(config.intents.len(), 2);
        assert!(config
            .intents
            .iter()
            .any(|intent| matches!(intent.kind, IntentKind::LongTermBuy | IntentKind::SmaCrossover)));
    }

    #[test]
    fn lowers_ma_cross_with_aliased_data_binding_into_consistent_input_ids() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let data_btc_daily_series = fetch("BTCUSDT", interval="1d", lookback=200)?
    let fast = sma(data_btc_daily_series, 20)
    let slow = sma(data_btc_daily_series, 50)

    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let config = lower_script_to_runtime_config(&module).unwrap();
        assert_eq!(config.data_sources[0].data_id, "data_btc_daily");
        assert!(config
            .intents
            .iter()
            .all(|intent| intent.input_data_ids == vec!["data_btc_daily".to_string()]));
    }

    #[test]
    fn lowers_rsi_thresholds_into_single_runtime_intent() {
        let module = parse_quant_script_module(RSI_SCRIPT).unwrap();
        let config = lower_script_to_runtime_config(&module).unwrap();
        assert_eq!(config.intents.len(), 1);
        assert_eq!(config.intents[0].kind, IntentKind::Rsi);
        assert_eq!(config.intents[0].params.get("period"), Some(&14.0));
        assert_eq!(
            config.intents[0].params.get("oversold_threshold"),
            Some(&30.0)
        );
        assert_eq!(
            config.intents[0].params.get("overbought_threshold"),
            Some(&70.0)
        );
    }

    #[test]
    fn lowered_script_runs_in_runtime() {
        let module = parse_quant_script_module(MA_SCRIPT).unwrap();
        let config = lower_script_to_runtime_config(&module).unwrap();
        let compiled = compile_runtime_protocol_config(&config).unwrap();
        let mut runtime = RuntimeCoordinator::new(compiled);
        let session = runtime
            .run_session(1_700_000_000_000, 1_700_000_005_000)
            .unwrap();
        assert!(!session.slow_cycle.intent_signals.is_empty());
    }

    #[test]
    fn lowers_user_defined_helper_function_calls() {
        let module = parse_quant_script_module(
            r#"
fn fast_ma(series, period) {
    return sma(series, period)
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let fast = fast_ma(closes, 20)
    let slow = sma(closes, 60)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let config = lower_script_to_runtime_config(&module).unwrap();
        assert!(config
            .intents
            .iter()
            .any(|intent| matches!(intent.kind, IntentKind::LongTermBuy | IntentKind::SmaCrossover)));
    }

    #[test]
    fn rejects_semantic_errors_before_runtime_lowering() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let signal = missing_helper(1)
    if 42 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    if signal > 0 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let err = lower_script_to_runtime_config(&module).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("QS0005"));
        assert!(message.contains("missing_helper"));
        assert!(message.contains("QS0006"));
    }

    #[test]
    fn lowers_manual_moving_average_helper_formula() {
        let module = parse_quant_script_module(
            r#"
fn moving_average(series, period) {
    let n = period
    return series[n..].sum() / n
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let fast = moving_average(closes, 20)
    let slow = moving_average(closes, 60)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let config = lower_script_to_runtime_config(&module).unwrap();
        assert!(config
            .intents
            .iter()
            .any(|intent| matches!(intent.kind, IntentKind::LongTermBuy | IntentKind::SmaCrossover)));
    }

    #[test]
    fn lowers_manual_momentum_formula() {
        let module = parse_quant_script_module(
            r#"
fn momentum_score(series, lookback) {
    return series[0] - series[lookback]
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = momentum_score(closes, 14)
    if score > 0 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let config = lower_script_to_runtime_config(&module).unwrap();
        assert!(config
            .intents
            .iter()
            .any(|intent| intent.kind == IntentKind::Momentum));
    }

    #[test]
    fn lowers_manual_zscore_formula() {
        let module = parse_quant_script_module(
            r#"
fn zscore_signal(series, window) {
    let scope = series[window..]
    return (series[0] - mean(scope)) / stddev(scope)
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = zscore_signal(closes, 20)
    if score > 2 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    } else if score < -2 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let config = lower_script_to_runtime_config(&module).unwrap();
        assert!(config
            .intents
            .iter()
            .any(|intent| intent.kind == IntentKind::ZScore));
    }

    #[test]
    fn lowers_manual_macd_histogram_formula() {
        let module = parse_quant_script_module(
            r#"
fn macd_hist(series, fast, slow, signal) {
    let macd_line = ema(series, fast) - ema(series, slow)
    let signal_line = ema(macd_line, signal)
    return macd_line - signal_line
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let hist = macd_hist(closes, 12, 26, 9)
    if hist > 0 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else if hist < 0 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let config = lower_script_to_runtime_config(&module).unwrap();
        assert!(config
            .intents
            .iter()
            .any(|intent| intent.kind == IntentKind::Macd));
    }

    #[test]
    fn lowers_manual_momentum_ratio_formula() {
        let module = parse_quant_script_module(
            r#"
fn momentum_ratio(series, lookback) {
    return (series[0] / series[lookback]) - 1
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = momentum_ratio(closes, 20)
    if score > 0.03 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else if score < -0.03 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let config = lower_script_to_runtime_config(&module).unwrap();
        assert!(config
            .intents
            .iter()
            .any(|intent| intent.kind == IntentKind::Momentum));
        let momentum = config
            .intents
            .iter()
            .find(|intent| intent.kind == IntentKind::Momentum)
            .unwrap();
        assert_eq!(momentum.params.get("lookback"), Some(&20.0));
        assert_eq!(momentum.params.get("threshold_ratio"), Some(&0.03));
    }

    #[test]
    fn lowers_manual_ma_gap_ratio_formula() {
        let module = parse_quant_script_module(
            r#"
fn ma_gap(series, fast, slow) {
    return (sma(series, fast) - sma(series, slow)) / sma(series, slow)
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let gap = ma_gap(closes, 20, 60)
    if gap > 0.02 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let config = lower_script_to_runtime_config(&module).unwrap();
        let entry = config
            .intents
            .iter()
            .find(|intent| intent.kind == IntentKind::LongTermBuy || intent.kind == IntentKind::SmaCrossover)
            .unwrap();
        assert_eq!(entry.params.get("fast_period"), Some(&20.0));
        assert_eq!(entry.params.get("slow_period"), Some(&60.0));
        assert_eq!(entry.params.get("entry_ratio"), Some(&1.02));
    }

    #[test]
    fn lowers_manual_rsi_formula_from_rs_ratio() {
        let module = parse_quant_script_module(
            r#"
fn manual_rsi(series, period) {
    let rs = rma(gains(series), period) / rma(losses(series), period)
    return 100 - (100 / (1 + rs))
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = manual_rsi(closes, 14)
    if score < 30 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else if score > 70 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let config = lower_script_to_runtime_config(&module).unwrap();
        let rsi = config
            .intents
            .iter()
            .find(|intent| intent.kind == IntentKind::Rsi)
            .unwrap();
        assert_eq!(rsi.params.get("period"), Some(&14.0));
        assert_eq!(rsi.params.get("oversold_threshold"), Some(&30.0));
        assert_eq!(rsi.params.get("overbought_threshold"), Some(&70.0));
    }

    #[test]
    fn lowers_manual_rsi_formula_with_avg_gain_loss_aliases() {
        let module = parse_quant_script_module(
            r#"
fn manual_rsi(series, period) {
    let avg_gain = wilders(gains(series), period)
    let avg_loss = wilders(losses(series), period)
    let rs = avg_gain / avg_loss
    return 100 - 100 / (1 + rs)
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = manual_rsi(closes, 21)
    if score < 35 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let config = lower_script_to_runtime_config(&module).unwrap();
        let rsi = config
            .intents
            .iter()
            .find(|intent| intent.kind == IntentKind::Rsi)
            .unwrap();
        assert_eq!(rsi.params.get("period"), Some(&21.0));
        assert_eq!(rsi.params.get("oversold_threshold"), Some(&35.0));
    }

    #[test]
    fn lowers_manual_ema_rsi_formula() {
        let module = parse_quant_script_module(
            r#"
fn ema_rsi(series, period) {
    let avg_gain = ema(gains(series), period)
    let avg_loss = ema(losses(series), period)
    let rs = avg_gain / avg_loss
    return 100 - 100 / (1 + rs)
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = ema_rsi(closes, 10)
    if score > 65 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let config = lower_script_to_runtime_config(&module).unwrap();
        let rsi = config
            .intents
            .iter()
            .find(|intent| intent.kind == IntentKind::Rsi)
            .unwrap();
        assert_eq!(rsi.params.get("period"), Some(&10.0));
        assert_eq!(rsi.params.get("smoothing_method"), Some(&1.0));
        assert_eq!(rsi.params.get("overbought_threshold"), Some(&65.0));
    }

    #[test]
    fn lowers_manual_cutler_rsi_formula() {
        let module = parse_quant_script_module(
            r#"
fn cutler_rsi(series, period) {
    let avg_gain = sma(gains(series), period)
    let avg_loss = sma(losses(series), period)
    let rs = avg_gain / avg_loss
    return 100 - 100 / (1 + rs)
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = cutler_rsi(closes, 12)
    if score < 28 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let config = lower_script_to_runtime_config(&module).unwrap();
        let rsi = config
            .intents
            .iter()
            .find(|intent| intent.kind == IntentKind::Rsi)
            .unwrap();
        assert_eq!(rsi.params.get("period"), Some(&12.0));
        assert_eq!(rsi.params.get("smoothing_method"), Some(&2.0));
        assert_eq!(rsi.params.get("oversold_threshold"), Some(&28.0));
    }

    #[test]
    fn rejects_manual_rsi_formula_from_loop_built_gain_loss_lists_in_formal_path() {
        let module = parse_quant_script_module(
            r#"
fn loop_gains(series) {
    let mut out = []
    for i in 1..series.len() {
        let diff = series[i] - series[i - 1]
        if diff > 0 {
            out.push(diff)
        } else {
            out.push(0)
        }
    }
    return out
}

fn loop_losses(series) {
    let mut out = []
    for i in 1..series.len() {
        let diff = series[i] - series[i - 1]
        if diff < 0 {
            out.push(-diff)
        } else {
            out.push(0)
        }
    }
    return out
}

fn manual_rsi(series, period) {
    let avg_gain = ema(loop_gains(series), period)
    let avg_loss = ema(loop_losses(series), period)
    let rs = avg_gain / avg_loss
    return 100 - 100 / (1 + rs)
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = manual_rsi(closes, 14)
    if score > 70 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let err = lower_script_to_runtime_config(&module).unwrap_err();
        assert!(err.to_string().contains("无法以符号方式展开 for 循环的可迭代对象"));
    }

    #[test]
    fn rejects_manual_rsi_formula_from_while_loop_gain_loss_lists_in_formal_path() {
        let module = parse_quant_script_module(
            r#"
fn while_gains(series) {
    let mut out = []
    let mut i = 1
    while i < series.len() {
        let diff = series[i] - series[i - 1]
        if diff > 0 {
            out.push(diff)
        } else {
            out.push(0)
        }
        let i = i + 1
    }
    return out
}

fn while_losses(series) {
    let mut out = []
    let mut i = 1
    while i < series.len() {
        let diff = series[i] - series[i - 1]
        if diff < 0 {
            out.push(abs(diff))
        } else {
            out.push(0)
        }
        let i = i + 1
    }
    return out
}

fn manual_rsi(series, period) {
    let avg_gain = sma(while_gains(series), period)
    let avg_loss = sma(while_losses(series), period)
    let rs = avg_gain / avg_loss
    return 100 - 100 / (1 + rs)
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = manual_rsi(closes, 9)
    if score < 25 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let err = lower_script_to_runtime_config(&module).unwrap_err();
        assert!(err.to_string().contains("while 循环"));
    }

    #[test]
    fn lowers_fetch_exchange_argument_into_data_source() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let data_price_feed_series = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let intent_entry_signal = rsi(data_price_feed_series, 14)
    if intent_entry_signal < 30 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let config = lower_script_to_runtime_config(&module).unwrap();
        assert_eq!(
            config
                .data_sources
                .iter()
                .map(|source| source.data_id.clone())
                .collect::<Vec<_>>(),
            vec!["data_price_feed".to_string()]
        );
        assert_eq!(config.data_sources[0].exchange, Exchange::Okx);
        assert_eq!(config.data_sources[0].data_id, "data_price_feed");
        assert_eq!(config.intents[0].intent_id, "intent_entry");
    }

    #[test]
    fn rejects_non_admitted_cross_source_spread_formula_for_formal_lowering() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let data_binance_series = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let data_okx_series = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let intent_spread_signal = (data_okx_series.last() - data_binance_series.last()) / data_binance_series.last()
    if intent_spread_signal > 0.005 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let err = lower_script_to_runtime_config(&module).unwrap_err();
        assert!(err.to_string().contains("QPQSLOW001"));
    }

    #[test]
    fn rejects_non_admitted_asymmetric_window_spread_formula_for_formal_lowering() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let data_binance_series = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let data_okx_series = fetch("BTCUSDT", exchange="okx", interval="5m", lookback=200)?
    let intent_spread_signal = (data_okx_series[3..].mean() - data_binance_series.last()) / data_binance_series.last()
    if intent_spread_signal > 0.004 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let err = lower_script_to_runtime_config(&module).unwrap_err();
        assert!(err.to_string().contains("QPQSLOW001"));
    }

    #[test]
    fn lowers_admitted_explicit_spread_helper_into_quote_observe() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let data_binance_series = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let data_okx_series = fetch("BTCUSDT", exchange="okx", interval="5m", lookback=200)?
    let buy_leg = align_asof(resample(field(data_binance_series, name="bid"), every="5m", agg="last"), direction="backward", tolerance_ms=10000)
    let sell_leg = align_asof(field(data_okx_series, name="ask"), direction="backward", tolerance_ms=10000)
    let intent_spread_signal = spread(buy_leg, sell_leg, output="bps")
    if intent_spread_signal > 45 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let config = lower_script_to_runtime_config(&module).unwrap();
        let intent = config
            .intents
            .iter()
            .find(|intent| intent.kind == IntentKind::QuoteObserve)
            .unwrap();

        assert_eq!(intent.params.get("spread_output_code"), Some(&1.0));
        assert_eq!(intent.params.get("spread_trigger_bps"), Some(&45.0));
        assert_eq!(intent.params.get("left_field_code"), Some(&1.0));
        assert_eq!(intent.params.get("right_field_code"), Some(&2.0));
        assert_eq!(
            intent.params.get("left_resample_period_ms"),
            Some(&300_000.0)
        );
        assert_eq!(intent.params.get("left_resample_agg_code"), Some(&0.0));
        assert_eq!(intent.params.get("align_direction_code"), Some(&0.0));
        assert_eq!(intent.params.get("max_time_diff_ms"), Some(&10_000.0));
        assert_eq!(intent.params.get("comparison_shape_code"), Some(&1.0));
        assert_eq!(intent.params.get("comparison_op_code"), Some(&2.0));
        assert_eq!(intent.params.get("comparison_threshold"), Some(&45.0));

        let compiled = compile_runtime_protocol_config(&config).unwrap();
        let spread_spec = compiled.core_ir.indicators[0].spread_spec.as_ref().unwrap();
        assert_eq!(spread_spec.output, qrpc_core_ir::SpreadValueKind::Bps);
        assert_eq!(
            spread_spec.align.direction,
            qrpc_core_ir::AlignDirection::Backward
        );
        assert_eq!(spread_spec.align.tolerance_ms, 10_000);
        match &spread_spec.left {
            qrpc_core_ir::SeriesExpr::Resample {
                period_ms,
                agg,
                input,
            } => {
                assert_eq!(*period_ms, 300_000);
                assert_eq!(*agg, qrpc_core_ir::SeriesAggregation::Last);
                match input.as_ref() {
                    qrpc_core_ir::SeriesExpr::DataField { field, .. } => {
                        assert_eq!(*field, qrpc_core_ir::SeriesField::BidOrClose);
                    }
                    other => panic!("expected data field under resample, got {other:?}"),
                }
            }
            other => panic!("expected resample left leg, got {other:?}"),
        }
        assert_eq!(
            compiled.core_ir.signal_rules[0].condition,
            qrpc_core_ir::ScalarExpr::Compare {
                left: Box::new(qrpc_core_ir::ScalarExpr::Ref {
                    name: "intent_spread".into(),
                }),
                op: qrpc_core_ir::ComparisonOp::Gt,
                right: Box::new(qrpc_core_ir::ScalarExpr::Number { value: 45.0 }),
            }
        );
    }

    #[test]
    fn rejects_non_admitted_helper_annotated_formula_spread() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let data_binance_series = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let data_okx_series = fetch("BTCUSDT", exchange="okx", interval="5m", lookback=200)?
    let buy_leg = field(data_binance_series, name="bid")
    let sell_leg = align(resample(field(data_okx_series, name="ask"), every="5m", agg="last"), direction="nearest", tolerance_ms=7500)
    let intent_spread_signal = (sell_leg - buy_leg) / buy_leg
    if intent_spread_signal > 0.0045 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let err = lower_script_to_runtime_config(&module).unwrap_err();
        assert!(err.to_string().contains("QPQSLOW001"));
    }

    #[test]
    fn lowers_equal_weight_rebalance_helper_into_portfolio_rebalance_agent() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let base = symbols(["BTCUSDT", "ETHUSDT"])
    rebalance(equal_weight(base), every="1d")
    for s in base {
        let closes = fetch(s, exchange="binance", interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)
        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }
}
"#,
        )
        .unwrap();

        let config = lower_script_to_runtime_config(&module).unwrap();
        assert_eq!(
            config.agents[0].params.get("portfolio_rebalance"),
            Some(&1.0)
        );
        assert_eq!(
            config.agents[0].params.get("max_quantity_ratio"),
            Some(&1.0)
        );
        assert_eq!(
            config.agents[0]
                .params
                .get("portfolio_rebalance_symbol_count"),
            Some(&2.0)
        );
        assert_eq!(
            config.agents[0].rebalance_symbols,
            vec![Symbol::BtcUsdt, Symbol::parse("ETHUSDT")]
        );
        assert_eq!(
            config.agents[0].rebalance_schedule,
            Some(RebalanceSchedule::Every1d)
        );
        assert_eq!(
            config.agents[0].rebalance_allocation_kind.as_deref(),
            Some("equal_weight")
        );
        assert_eq!(config.risks[0].max_position_ratio, 1.0);
        let compiled = compile_runtime_protocol_config(&config).unwrap();
        assert_eq!(
            compiled.core_ir.agent_policies[0].kind,
            qrpc_core_ir::AgentPolicyKind::PortfolioRebalance
        );
    }

    #[test]
    fn lowers_fixed_weights_rebalance_helper_into_portfolio_rebalance_agent() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let base = symbols(["BTCUSDT", "ETHUSDT"])
    rebalance(fixed_weights(base, weights=[0.7, 0.3]), every="slow")
    for s in base {
        let closes = fetch(s, exchange="binance", interval="1d", lookback=200)?
        if sma(closes, 20) > sma(closes, 100) {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }
}
"#,
        )
        .unwrap();

        let config = lower_script_to_runtime_config(&module).unwrap();
        assert_eq!(
            config.agents[0].rebalance_allocation_kind.as_deref(),
            Some("fixed_weights")
        );
        assert_eq!(config.agents[0].rebalance_target_weights, vec![0.7, 0.3]);
        assert_eq!(
            config.agents[0].rebalance_schedule,
            Some(RebalanceSchedule::EverySlow)
        );
    }

    #[test]
    fn lowers_rank_weight_rebalance_helper_into_portfolio_rebalance_agent() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let base = symbols(["BTCUSDT", "ETHUSDT", "SOLUSDT"])
    rebalance(rank_weight(base, method="inverse_rank"), every="1d")
    for s in base {
        let closes = fetch(s, exchange="binance", interval="1d", lookback=200)?
        if sma(closes, 20) > sma(closes, 100) {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }
}
"#,
        )
        .unwrap();

        let config = lower_script_to_runtime_config(&module).unwrap();
        assert_eq!(
            config.agents[0].rebalance_allocation_kind.as_deref(),
            Some("rank_weight")
        );
        assert_eq!(
            config.agents[0].rebalance_rank_method.as_deref(),
            Some("inverse_rank")
        );
    }

    #[test]
    fn lowers_score_weight_rebalance_helper_into_portfolio_rebalance_agent() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let base = symbols(["BTCUSDT", "ETHUSDT", "SOLUSDT"])
    rebalance(score_weight(base, normalize="sum"), every="1d")
    for s in base {
        let closes = fetch(s, exchange="binance", interval="1d", lookback=200)?
        if sma(closes, 20) > sma(closes, 100) {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }
}
"#,
        )
        .unwrap();

        let config = lower_script_to_runtime_config(&module).unwrap();
        assert_eq!(
            config.agents[0].rebalance_allocation_kind.as_deref(),
            Some("score_weight")
        );
        assert_eq!(
            config.agents[0].rebalance_score_normalize.as_deref(),
            Some("sum")
        );
    }

    #[test]
    fn rejects_rebalance_helper_with_unsupported_frequency() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let base = symbols(["BTCUSDT", "ETHUSDT"])
    rebalance(equal_weight(base), every="1h")
    for s in base {
        let closes = fetch(s, exchange="binance", interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)
        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }
}
"#,
        )
        .unwrap();

        let err = lower_script_to_runtime_config(&module).unwrap_err();
        assert!(err
            .to_string()
            .contains("rebalance(..., every=...) 当前仅支持"));
    }

    #[test]
    fn rejects_rank_weight_with_unsupported_method() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let base = symbols(["BTCUSDT", "ETHUSDT"])
    rebalance(rank_weight(base, method="weird"), every="1d")
    for s in base {
        let closes = fetch(s, exchange="binance", interval="1d", lookback=200)?
        if sma(closes, 20) > sma(closes, 100) {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }
}
"#,
        )
        .unwrap();

        let err = lower_script_to_runtime_config(&module).unwrap_err();
        assert!(err
            .to_string()
            .contains("rank_weight(..., method=...) 当前仅支持"));
    }

    #[test]
    fn lowers_weekly_rebalance_schedule_into_agent_schedule() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let base = symbols(["BTCUSDT", "ETHUSDT"])
    rebalance(equal_weight(base), every="weekly")
    for s in base {
        let closes = fetch(s, exchange="binance", interval="1d", lookback=200)?
        if sma(closes, 20) > sma(closes, 100) {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }
}
"#,
        )
        .unwrap();

        let config = lower_script_to_runtime_config(&module).unwrap();
        assert_eq!(
            config.agents[0].rebalance_schedule,
            Some(RebalanceSchedule::Weekly)
        );
    }
}
