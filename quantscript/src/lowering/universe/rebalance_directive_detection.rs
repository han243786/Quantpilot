use anyhow::{anyhow, bail, Result};
use qrpc_core::RebalanceSchedule;
use std::collections::BTreeMap;

use crate::script::{CallArg, Expr, Item, MatchArmBody, ScriptModule, Stmt};

use super::super::context::{
    InstrumentPoolRebalanceRule, InstrumentPoolSpec, InstrumentPoolWeightingRule, LoweringContext,
    PortfolioRebalanceDirective,
};
use super::super::shared::{arg_string_optional, find_arg, ArgSelector};
use super::{evaluate_universe_expr, UniverseValue};

const ERR_MISSING_STRATEGY_FN: &str = "QPQSLOW006 rebalance 等 universe 操作必须在 fn strategy() 入口函数内使用。请声明: fn strategy() { ... }";
const ERR_MULTI_REBALANCE: &str = "QPQSLOW008 QuantScript 当前最多支持一个 rebalance(...) 指令";
const ERR_REBALANCE_FREQUENCY: &str =
    "QPQSLOW009 rebalance(..., every=...) 当前仅支持 \"1d\"、\"slow\" 或 \"weekly\"";
const ERR_REBALANCE_ALLOCATION_FORM: &str =
    "QPQSLOW013 rebalance(...) 需要分配函数。有效的分配函数: equal_weight(selection), fixed_weights(selection, weights=[...]), rank_weight(selection, method=\"linear\"), score_weight(selection, normalize=\"sum\")";
const ERR_REBALANCE_ALLOCATION_UNIVERSE: &str =
    "QPQSLOW014 rebalance 分配需要一个 universe 表达式。有效的 universe 值: symbols([...]), universe(), filter(...), sort_by(...), top(...)";
const ERR_REBALANCE_EMPTY_SELECTION: &str = "QPQSLOW015 rebalance 分配需要至少一个标的";
const ERR_FIXED_WEIGHTS_COUNT: &str =
    "QPQSLOW016 fixed_weights(..., weights=[...]) 需要每个选定标的对应一个权重";
const ERR_FIXED_WEIGHTS_NEGATIVE: &str =
    "QPQSLOW017 fixed_weights(..., weights=[...]) 不允许负数权重";
const ERR_FIXED_WEIGHTS_TOTAL: &str =
    "QPQSLOW018 fixed_weights(..., weights=[...]) 需要总权重大于 0";
const ERR_RANK_WEIGHT_METHOD: &str =
    "QPQSLOW019 rank_weight(..., method=...) 当前仅支持 \"linear\" 或 \"inverse_rank\"";
const ERR_SCORE_WEIGHT_NORMALIZE: &str =
    "QPQSLOW020 score_weight(..., normalize=...) 当前仅支持 \"sum\"";
const ERR_FIXED_WEIGHTS_LITERAL: &str = "QPQSLOW021 weights=... 当前需要数值列表字面量";

pub(super) fn detect_portfolio_rebalance_directive(
    module: &ScriptModule,
    context: &LoweringContext,
) -> Result<Option<PortfolioRebalanceDirective>> {
    detect_portfolio_rebalance_directive_with_mode(module, context, false)
}

pub(super) fn extract_instrument_pool_spec(
    module: &ScriptModule,
    context: &LoweringContext,
) -> Result<Option<InstrumentPoolSpec>> {
    Ok(
        detect_portfolio_rebalance_directive_with_mode(module, context, true)?
            .map(|directive| directive.instrument_pool),
    )
}

fn detect_portfolio_rebalance_directive_with_mode(
    module: &ScriptModule,
    context: &LoweringContext,
    best_effort_pool_extraction: bool,
) -> Result<Option<PortfolioRebalanceDirective>> {
    let strategy = module
        .items
        .iter()
        .find_map(|item| match item {
            Item::Function(function) if function.name == "strategy" => Some(function),
            _ => None,
        })
        .ok_or_else(|| anyhow!(ERR_MISSING_STRATEGY_FN))?;
    let mut directive = None;
    let mut universe_bindings = BTreeMap::new();
    collect_portfolio_rebalance_directive_from_stmts(
        &strategy.body,
        context,
        best_effort_pool_extraction,
        &mut universe_bindings,
        &mut directive,
    )?;
    Ok(directive)
}

fn collect_portfolio_rebalance_directive_from_stmts(
    stmts: &[Stmt],
    context: &LoweringContext,
    best_effort_pool_extraction: bool,
    universe_bindings: &mut BTreeMap<String, UniverseValue>,
    directive: &mut Option<PortfolioRebalanceDirective>,
) -> Result<()> {
    for stmt in stmts {
        match stmt {
            Stmt::Let { pattern, value, .. } => {
                if let Some(universe_value) = evaluate_universe_expr(
                    value,
                    context,
                    best_effort_pool_extraction,
                    universe_bindings,
                )? {
                    universe_bindings.insert(pattern.clone(), universe_value);
                }
            }
            Stmt::Expr(expr) => {
                if let Some(next) = portfolio_rebalance_directive_from_expr(
                    expr,
                    context,
                    best_effort_pool_extraction,
                    universe_bindings,
                )? {
                    if directive.replace(next).is_some() {
                        bail!(ERR_MULTI_REBALANCE);
                    }
                }
            }
            Stmt::If {
                then_branch,
                else_if_branches,
                else_branch,
                ..
            } => {
                let mut then_bindings = universe_bindings.clone();
                collect_portfolio_rebalance_directive_from_stmts(
                    then_branch,
                    context,
                    best_effort_pool_extraction,
                    &mut then_bindings,
                    directive,
                )?;
                for (_, branch) in else_if_branches {
                    let mut branch_bindings = universe_bindings.clone();
                    collect_portfolio_rebalance_directive_from_stmts(
                        branch,
                        context,
                        best_effort_pool_extraction,
                        &mut branch_bindings,
                        directive,
                    )?;
                }
                if let Some(branch) = else_branch {
                    let mut else_bindings = universe_bindings.clone();
                    collect_portfolio_rebalance_directive_from_stmts(
                        branch,
                        context,
                        best_effort_pool_extraction,
                        &mut else_bindings,
                        directive,
                    )?;
                }
            }
            Stmt::For { body, .. } | Stmt::While { body, .. } => {
                let mut nested_bindings = universe_bindings.clone();
                collect_portfolio_rebalance_directive_from_stmts(
                    body,
                    context,
                    best_effort_pool_extraction,
                    &mut nested_bindings,
                    directive,
                )?;
            }
            Stmt::Match { arms, .. } => {
                for arm in arms {
                    if let MatchArmBody::Statement(stmt) = &arm.body {
                        let mut nested_bindings = universe_bindings.clone();
                        collect_portfolio_rebalance_directive_from_stmts(
                            std::slice::from_ref(stmt.as_ref()),
                            context,
                            best_effort_pool_extraction,
                            &mut nested_bindings,
                            directive,
                        )?;
                    }
                }
            }
            Stmt::Return(_) | Stmt::EmitIntent { .. } => {}
        }
    }
    Ok(())
}

fn portfolio_rebalance_directive_from_expr(
    expr: &Expr,
    context: &LoweringContext,
    best_effort_pool_extraction: bool,
    universe_bindings: &BTreeMap<String, UniverseValue>,
) -> Result<Option<PortfolioRebalanceDirective>> {
    let Expr::Call { callee, args } = expr else {
        return Ok(None);
    };
    let Expr::Identifier(name) = callee.as_ref() else {
        return Ok(None);
    };
    if name != "rebalance" {
        return Ok(None);
    }

    let allocation_expr = find_arg(args, ArgSelector::Positional(0))
        .ok_or_else(|| anyhow!(ERR_REBALANCE_ALLOCATION_FORM))?;
    let Expr::Call {
        callee: allocation_callee,
        args: allocation_args,
    } = allocation_expr
    else {
        bail!(ERR_REBALANCE_ALLOCATION_FORM);
    };
    let Expr::Identifier(allocation_name) = allocation_callee.as_ref() else {
        bail!(ERR_REBALANCE_ALLOCATION_FORM);
    };
    let (universe_value, allocation_kind, rank_method, score_normalize, target_weights) =
        parse_rebalance_allocation_expr(
            allocation_name,
            allocation_args,
            context,
            best_effort_pool_extraction,
            universe_bindings,
        )?;

    let schedule =
        parse_rebalance_schedule(arg_string_optional(args, ArgSelector::Named("every")))?;
    let mut instrument_pool = universe_value.instrument_pool;
    instrument_pool.weighting_rule = Some(InstrumentPoolWeightingRule {
        kind: allocation_kind.clone(),
        method: rank_method.clone(),
        normalize: score_normalize.clone(),
        target_weights: target_weights.clone(),
    });
    instrument_pool.rebalance_rule = Some(InstrumentPoolRebalanceRule {
        every: schedule.clone(),
    });

    Ok(Some(PortfolioRebalanceDirective {
        symbols: universe_value.symbols,
        schedule,
        allocation_kind,
        rank_method,
        score_normalize,
        target_weights,
        instrument_pool,
    }))
}

fn parse_rebalance_schedule(value: Option<String>) -> Result<Option<RebalanceSchedule>> {
    match value.as_deref() {
        None => Ok(None),
        Some("slow") => Ok(Some(RebalanceSchedule::EverySlow)),
        Some("1d") => Ok(Some(RebalanceSchedule::Every1d)),
        Some("weekly") => Ok(Some(RebalanceSchedule::Weekly)),
        Some(_) => bail!(ERR_REBALANCE_FREQUENCY),
    }
}

type RebalanceAllocationParse = (
    UniverseValue,
    String,
    Option<String>,
    Option<String>,
    Vec<f64>,
);

fn parse_rebalance_allocation_expr(
    allocation_name: &str,
    allocation_args: &[CallArg],
    context: &LoweringContext,
    best_effort_pool_extraction: bool,
    universe_bindings: &BTreeMap<String, UniverseValue>,
) -> Result<RebalanceAllocationParse> {
    match allocation_name {
        "equal_weight" => {
            let selection_expr = find_arg(allocation_args, ArgSelector::Positional(0))
                .ok_or_else(|| anyhow!(ERR_REBALANCE_ALLOCATION_UNIVERSE))?;
            let universe_value = evaluate_universe_expr(
                selection_expr,
                context,
                best_effort_pool_extraction,
                universe_bindings,
            )?
            .ok_or_else(|| anyhow!(ERR_REBALANCE_ALLOCATION_UNIVERSE))?;
            if universe_value.symbols.is_empty() {
                bail!(ERR_REBALANCE_EMPTY_SELECTION);
            }
            Ok((
                universe_value,
                "equal_weight".into(),
                None,
                None,
                Vec::new(),
            ))
        }
        "fixed_weights" => {
            let selection_expr = find_arg(allocation_args, ArgSelector::Positional(0))
                .ok_or_else(|| anyhow!(ERR_REBALANCE_ALLOCATION_UNIVERSE))?;
            let universe_value = evaluate_universe_expr(
                selection_expr,
                context,
                best_effort_pool_extraction,
                universe_bindings,
            )?
            .ok_or_else(|| anyhow!(ERR_REBALANCE_ALLOCATION_UNIVERSE))?;
            if universe_value.symbols.is_empty() {
                bail!(ERR_REBALANCE_EMPTY_SELECTION);
            }
            let weights_expr = allocation_args
                .iter()
                .find(|arg| arg.name.as_deref() == Some("weights"))
                .map(|arg| &arg.value)
                .ok_or_else(|| anyhow!(ERR_FIXED_WEIGHTS_LITERAL))?;
            let target_weights = evaluate_number_list_expr(weights_expr)?;
            if target_weights.len() != universe_value.symbols.len() {
                bail!(ERR_FIXED_WEIGHTS_COUNT);
            }
            if target_weights.iter().any(|weight| *weight < 0.0) {
                bail!(ERR_FIXED_WEIGHTS_NEGATIVE);
            }
            let total_weight = target_weights.iter().sum::<f64>();
            if total_weight <= f64::EPSILON {
                bail!(ERR_FIXED_WEIGHTS_TOTAL);
            }
            Ok((
                universe_value,
                "fixed_weights".into(),
                None,
                None,
                target_weights
                    .into_iter()
                    .map(|weight| weight / total_weight)
                    .collect(),
            ))
        }
        "rank_weight" => {
            let selection_expr = find_arg(allocation_args, ArgSelector::Positional(0))
                .ok_or_else(|| anyhow!(ERR_REBALANCE_ALLOCATION_UNIVERSE))?;
            let universe_value = evaluate_universe_expr(
                selection_expr,
                context,
                best_effort_pool_extraction,
                universe_bindings,
            )?
            .ok_or_else(|| anyhow!(ERR_REBALANCE_ALLOCATION_UNIVERSE))?;
            if universe_value.symbols.is_empty() {
                bail!(ERR_REBALANCE_EMPTY_SELECTION);
            }
            let method = arg_string_optional(allocation_args, ArgSelector::Named("method"))
                .unwrap_or_else(|| "linear".into());
            if method != "linear" && method != "inverse_rank" {
                bail!(ERR_RANK_WEIGHT_METHOD);
            }
            Ok((
                universe_value,
                "rank_weight".into(),
                Some(method),
                None,
                Vec::new(),
            ))
        }
        "score_weight" => {
            let selection_expr = find_arg(allocation_args, ArgSelector::Positional(0))
                .ok_or_else(|| anyhow!(ERR_REBALANCE_ALLOCATION_UNIVERSE))?;
            let universe_value = evaluate_universe_expr(
                selection_expr,
                context,
                best_effort_pool_extraction,
                universe_bindings,
            )?
            .ok_or_else(|| anyhow!(ERR_REBALANCE_ALLOCATION_UNIVERSE))?;
            if universe_value.symbols.is_empty() {
                bail!(ERR_REBALANCE_EMPTY_SELECTION);
            }
            let normalize = arg_string_optional(allocation_args, ArgSelector::Named("normalize"))
                .unwrap_or_else(|| "sum".into());
            if normalize != "sum" {
                bail!(ERR_SCORE_WEIGHT_NORMALIZE);
            }
            Ok((
                universe_value,
                "score_weight".into(),
                None,
                Some(normalize),
                Vec::new(),
            ))
        }
        _ => bail!(ERR_REBALANCE_ALLOCATION_FORM),
    }
}

fn evaluate_number_list_expr(expr: &Expr) -> Result<Vec<f64>> {
    let Expr::List(items) = expr else {
        bail!(ERR_FIXED_WEIGHTS_LITERAL);
    };
    items
        .iter()
        .map(|item| match item {
            Expr::Number(value) => Ok(*value),
            _ => bail!(ERR_FIXED_WEIGHTS_LITERAL),
        })
        .collect()
}
