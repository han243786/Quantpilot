use anyhow::{anyhow, bail, Result};
use qrpc_core::{Symbol, UniverseAssetMetadataPoint, UniverseAssetRecord, UniverseSnapshot};
use std::collections::{BTreeMap, BTreeSet};

use crate::script::{CallArg, Expr, FunctionDecl, Item, MatchArmBody, ScriptModule, Stmt};

use super::context::{
    InstrumentPoolEligibilityRule, InstrumentPoolSelectionKey, InstrumentPoolSelectionRule,
    InstrumentPoolSourceSpec, InstrumentPoolSpec, InstrumentPoolValue, LoweringContext,
    PortfolioRebalanceDirective, UniverseAssetMetrics,
};
use super::shared::{
    arg_number_optional, arg_string_optional, expr_string, find_arg, format_exchange,
    format_market_type, sanitize_id, ArgSelector,
};

const ERR_UNIVERSE_SNAPSHOT_REQUIRED: &str =
    "QPQSLOW010 编译时 universe 操作需要 universe_snapshot";
const ERR_UNIVERSE_SORT_KEY: &str = "QPQSLOW011 不支持的 universe 排序键";
const ERR_UNIVERSE_SORT_ORDER: &str = "QPQSLOW012 不支持的 universe 排序顺序";
const ERR_UNIVERSE_VALUE_REQUIRED: &str =
    "QPQSLOW025 universe 辅助函数需要 universe 值表达式。请将 symbols(...)、universe(...)、filter(...)、sort_by(...) 或 top(...) 作为参数传入";
const ERR_SYMBOLS_LIST_LITERAL: &str = "QPQSLOW026 symbols(...) 当前需要列表字面量";
const ERR_SYMBOLS_STRING_LITERAL: &str = "QPQSLOW027 symbols([...]) 当前需要字符串字面量";
const ERR_TOP_COUNT_REQUIRED: &str = "QPQSLOW028 top(...) 当前需要数值计数";

mod rebalance_directive_detection;

#[derive(Debug, Clone)]
struct UniverseValue {
    symbols: Vec<Symbol>,
    instrument_pool: InstrumentPoolSpec,
}

pub(crate) fn detect_portfolio_rebalance_directive(
    module: &ScriptModule,
    context: &LoweringContext,
) -> Result<Option<PortfolioRebalanceDirective>> {
    rebalance_directive_detection::detect_portfolio_rebalance_directive(module, context)
}

pub(crate) fn extract_instrument_pool_spec(
    module: &ScriptModule,
    context: &LoweringContext,
) -> Result<Option<InstrumentPoolSpec>> {
    rebalance_directive_detection::extract_instrument_pool_spec(module, context)
}

pub(crate) fn expand_universe_constructs(
    module: &ScriptModule,
    context: &LoweringContext,
) -> Result<ScriptModule> {
    let mut items = Vec::with_capacity(module.items.len());
    for item in &module.items {
        match item {
            Item::Function(function) if function.name == "strategy" => {
                let mut universe_bindings = BTreeMap::new();
                items.push(Item::Function(FunctionDecl {
                    is_async: function.is_async,
                    name: function.name.clone(),
                    params: function.params.clone(),
                    return_type: function.return_type.clone(),
                    body: expand_stmts(&function.body, context, &mut universe_bindings)?,
                }));
            }
            _ => items.push(item.clone()),
        }
    }
    Ok(ScriptModule { items })
}

fn expand_stmts(
    stmts: &[Stmt],
    context: &LoweringContext,
    universe_bindings: &mut BTreeMap<String, UniverseValue>,
) -> Result<Vec<Stmt>> {
    let mut expanded = Vec::new();
    for stmt in stmts {
        match stmt {
            Stmt::Let {
                pattern,
                ty,
                value,
                mutable,
            } => {
                if let Some(universe_value) =
                    evaluate_universe_expr(value, context, false, universe_bindings)?
                {
                    universe_bindings.insert(pattern.clone(), universe_value);
                    continue;
                }
                expanded.push(Stmt::Let {
                    pattern: pattern.clone(),
                    ty: ty.clone(),
                    value: substitute_universe_expr(value, universe_bindings),
                    mutable: *mutable,
                });
            }
            Stmt::For {
                pattern,
                iterable,
                body,
            } => {
                if let Some(universe_value) =
                    evaluate_universe_expr(iterable, context, false, universe_bindings)?
                {
                    for symbol in universe_value.symbols {
                        let renamed_body = uniquify_local_bindings(body, symbol.as_str());
                        let substituted_body = renamed_body
                            .iter()
                            .map(|stmt| substitute_symbol_in_stmt(stmt, pattern, &symbol))
                            .collect::<Vec<_>>();
                        let mut nested_bindings = universe_bindings.clone();
                        expanded.extend(expand_stmts(
                            &substituted_body,
                            context,
                            &mut nested_bindings,
                        )?);
                    }
                    continue;
                }

                let mut nested_bindings = universe_bindings.clone();
                expanded.push(Stmt::For {
                    pattern: pattern.clone(),
                    iterable: substitute_universe_expr(iterable, universe_bindings),
                    body: expand_stmts(body, context, &mut nested_bindings)?,
                });
            }
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                let mut then_bindings = universe_bindings.clone();
                let expanded_then = expand_stmts(then_branch, context, &mut then_bindings)?;
                let mut expanded_else_if = Vec::with_capacity(else_if_branches.len());
                for (branch_condition, branch) in else_if_branches {
                    let mut branch_bindings = universe_bindings.clone();
                    expanded_else_if.push((
                        substitute_universe_expr(branch_condition, universe_bindings),
                        expand_stmts(branch, context, &mut branch_bindings)?,
                    ));
                }
                let expanded_else = if let Some(branch) = else_branch {
                    let mut else_bindings = universe_bindings.clone();
                    Some(expand_stmts(branch, context, &mut else_bindings)?)
                } else {
                    None
                };
                expanded.push(Stmt::If {
                    condition: substitute_universe_expr(condition, universe_bindings),
                    then_branch: expanded_then,
                    else_if_branches: expanded_else_if,
                    else_branch: expanded_else,
                });
            }
            Stmt::While { condition, body } => {
                let mut nested_bindings = universe_bindings.clone();
                expanded.push(Stmt::While {
                    condition: substitute_universe_expr(condition, universe_bindings),
                    body: expand_stmts(body, context, &mut nested_bindings)?,
                });
            }
            Stmt::Match { expr, arms } => {
                let arms = arms
                    .iter()
                    .map(|arm| {
                        let body = match &arm.body {
                            MatchArmBody::Statement(stmt) => {
                                let mut nested_bindings = universe_bindings.clone();
                                MatchArmBody::Statement(Box::new(
                                    expand_stmts(
                                        std::slice::from_ref(stmt.as_ref()),
                                        context,
                                        &mut nested_bindings,
                                    )?
                                    .into_iter()
                                    .next()
                                    .unwrap_or(Stmt::Return(None)),
                                ))
                            }
                            MatchArmBody::Expr(expr) => MatchArmBody::Expr(
                                substitute_universe_expr(expr, universe_bindings),
                            ),
                        };
                        Ok(crate::script::MatchArm {
                            pattern: arm.pattern.clone(),
                            body,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                expanded.push(Stmt::Match {
                    expr: substitute_universe_expr(expr, universe_bindings),
                    arms,
                });
            }
            _ => expanded.push(substitute_universe_stmt(stmt, universe_bindings)),
        }
    }
    Ok(expanded)
}

fn uniquify_local_bindings(stmts: &[Stmt], suffix: &str) -> Vec<Stmt> {
    let mut binding_names = BTreeSet::new();
    collect_binding_names(stmts, &mut binding_names);
    let rename_map = binding_names
        .into_iter()
        .map(|name| {
            let renamed = format!("{}_{}", name, sanitize_id(suffix));
            (name, renamed)
        })
        .collect::<BTreeMap<_, _>>();
    stmts
        .iter()
        .map(|stmt| rename_stmt_bindings(stmt, &rename_map))
        .collect()
}

fn collect_binding_names(stmts: &[Stmt], out: &mut BTreeSet<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Let { pattern, .. } => {
                out.insert(pattern.clone());
            }
            Stmt::If {
                then_branch,
                else_if_branches,
                else_branch,
                ..
            } => {
                collect_binding_names(then_branch, out);
                for (_, branch) in else_if_branches {
                    collect_binding_names(branch, out);
                }
                if let Some(branch) = else_branch {
                    collect_binding_names(branch, out);
                }
            }
            Stmt::For { pattern, body, .. } => {
                out.insert(pattern.clone());
                collect_binding_names(body, out);
            }
            Stmt::While { body, .. } => collect_binding_names(body, out),
            Stmt::Match { arms, .. } => {
                for arm in arms {
                    if let MatchArmBody::Statement(stmt) = &arm.body {
                        collect_binding_names(std::slice::from_ref(stmt.as_ref()), out);
                    }
                }
            }
            _ => {}
        }
    }
}

fn rename_stmt_bindings(stmt: &Stmt, rename_map: &BTreeMap<String, String>) -> Stmt {
    match stmt {
        Stmt::Let {
            pattern,
            ty,
            value,
            mutable,
        } => Stmt::Let {
            pattern: renamed_identifier(pattern, rename_map),
            ty: ty.clone(),
            value: rename_expr_bindings(value, rename_map),
            mutable: *mutable,
        },
        Stmt::Return(value) => Stmt::Return(
            value
                .as_ref()
                .map(|expr| rename_expr_bindings(expr, rename_map)),
        ),
        Stmt::EmitIntent { args } => Stmt::EmitIntent {
            args: args
                .iter()
                .map(|arg| CallArg {
                    name: arg.name.clone(),
                    value: rename_expr_bindings(&arg.value, rename_map),
                })
                .collect(),
        },
        Stmt::If {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
        } => Stmt::If {
            condition: rename_expr_bindings(condition, rename_map),
            then_branch: then_branch
                .iter()
                .map(|stmt| rename_stmt_bindings(stmt, rename_map))
                .collect(),
            else_if_branches: else_if_branches
                .iter()
                .map(|(condition, branch)| {
                    (
                        rename_expr_bindings(condition, rename_map),
                        branch
                            .iter()
                            .map(|stmt| rename_stmt_bindings(stmt, rename_map))
                            .collect(),
                    )
                })
                .collect(),
            else_branch: else_branch.as_ref().map(|branch| {
                branch
                    .iter()
                    .map(|stmt| rename_stmt_bindings(stmt, rename_map))
                    .collect()
            }),
        },
        Stmt::For {
            pattern,
            iterable,
            body,
        } => Stmt::For {
            pattern: renamed_identifier(pattern, rename_map),
            iterable: rename_expr_bindings(iterable, rename_map),
            body: body
                .iter()
                .map(|stmt| rename_stmt_bindings(stmt, rename_map))
                .collect(),
        },
        Stmt::While { condition, body } => Stmt::While {
            condition: rename_expr_bindings(condition, rename_map),
            body: body
                .iter()
                .map(|stmt| rename_stmt_bindings(stmt, rename_map))
                .collect(),
        },
        Stmt::Match { expr, arms } => Stmt::Match {
            expr: rename_expr_bindings(expr, rename_map),
            arms: arms
                .iter()
                .map(|arm| crate::script::MatchArm {
                    pattern: arm.pattern.clone(),
                    body: match &arm.body {
                        MatchArmBody::Statement(stmt) => MatchArmBody::Statement(Box::new(
                            rename_stmt_bindings(stmt, rename_map),
                        )),
                        MatchArmBody::Expr(expr) => {
                            MatchArmBody::Expr(rename_expr_bindings(expr, rename_map))
                        }
                    },
                })
                .collect(),
        },
        Stmt::Expr(expr) => Stmt::Expr(rename_expr_bindings(expr, rename_map)),
    }
}

fn rename_expr_bindings(expr: &Expr, rename_map: &BTreeMap<String, String>) -> Expr {
    match expr {
        Expr::Identifier(name) => Expr::Identifier(renamed_identifier(name, rename_map)),
        Expr::List(items) => Expr::List(
            items
                .iter()
                .map(|item| rename_expr_bindings(item, rename_map))
                .collect(),
        ),
        Expr::Call { callee, args } => Expr::Call {
            callee: Box::new(rename_expr_bindings(callee, rename_map)),
            args: args
                .iter()
                .map(|arg| CallArg {
                    name: arg.name.clone(),
                    value: rename_expr_bindings(&arg.value, rename_map),
                })
                .collect(),
        },
        Expr::Member { object, field } => Expr::Member {
            object: Box::new(rename_expr_bindings(object, rename_map)),
            field: field.clone(),
        },
        Expr::Index { object, index } => Expr::Index {
            object: Box::new(rename_expr_bindings(object, rename_map)),
            index: Box::new(rename_expr_bindings(index, rename_map)),
        },
        Expr::Slice { object, start, end } => Expr::Slice {
            object: Box::new(rename_expr_bindings(object, rename_map)),
            start: start
                .as_ref()
                .map(|value| Box::new(rename_expr_bindings(value, rename_map))),
            end: end
                .as_ref()
                .map(|value| Box::new(rename_expr_bindings(value, rename_map))),
        },
        Expr::Unary { op, expr } => Expr::Unary {
            op: op.clone(),
            expr: Box::new(rename_expr_bindings(expr, rename_map)),
        },
        Expr::Binary { left, op, right } => Expr::Binary {
            left: Box::new(rename_expr_bindings(left, rename_map)),
            op: op.clone(),
            right: Box::new(rename_expr_bindings(right, rename_map)),
        },
        Expr::Range { start, end } => Expr::Range {
            start: Box::new(rename_expr_bindings(start, rename_map)),
            end: Box::new(rename_expr_bindings(end, rename_map)),
        },
        Expr::Await(inner) => Expr::Await(Box::new(rename_expr_bindings(inner, rename_map))),
        Expr::Try(inner) => Expr::Try(Box::new(rename_expr_bindings(inner, rename_map))),
        _ => expr.clone(),
    }
}

fn renamed_identifier(name: &str, rename_map: &BTreeMap<String, String>) -> String {
    rename_map
        .get(name)
        .cloned()
        .unwrap_or_else(|| name.to_string())
}

fn substitute_universe_stmt(
    stmt: &Stmt,
    universe_bindings: &BTreeMap<String, UniverseValue>,
) -> Stmt {
    match stmt {
        Stmt::Let {
            pattern,
            ty,
            value,
            mutable,
        } => Stmt::Let {
            pattern: pattern.clone(),
            ty: ty.clone(),
            value: substitute_universe_expr(value, universe_bindings),
            mutable: *mutable,
        },
        Stmt::Return(value) => Stmt::Return(
            value
                .as_ref()
                .map(|expr| substitute_universe_expr(expr, universe_bindings)),
        ),
        Stmt::EmitIntent { args } => Stmt::EmitIntent {
            args: args
                .iter()
                .map(|arg| CallArg {
                    name: arg.name.clone(),
                    value: substitute_universe_expr(&arg.value, universe_bindings),
                })
                .collect(),
        },
        Stmt::Expr(expr) => Stmt::Expr(substitute_universe_expr(expr, universe_bindings)),
        _ => stmt.clone(),
    }
}

fn substitute_universe_expr(
    expr: &Expr,
    universe_bindings: &BTreeMap<String, UniverseValue>,
) -> Expr {
    match expr {
        Expr::Identifier(name) if universe_bindings.contains_key(name) => expr.clone(),
        Expr::List(items) => Expr::List(
            items
                .iter()
                .map(|item| substitute_universe_expr(item, universe_bindings))
                .collect(),
        ),
        Expr::Call { callee, args } => Expr::Call {
            callee: Box::new(substitute_universe_expr(callee, universe_bindings)),
            args: args
                .iter()
                .map(|arg| CallArg {
                    name: arg.name.clone(),
                    value: substitute_universe_expr(&arg.value, universe_bindings),
                })
                .collect(),
        },
        Expr::Member { object, field } => Expr::Member {
            object: Box::new(substitute_universe_expr(object, universe_bindings)),
            field: field.clone(),
        },
        Expr::Index { object, index } => Expr::Index {
            object: Box::new(substitute_universe_expr(object, universe_bindings)),
            index: Box::new(substitute_universe_expr(index, universe_bindings)),
        },
        Expr::Slice { object, start, end } => Expr::Slice {
            object: Box::new(substitute_universe_expr(object, universe_bindings)),
            start: start
                .as_ref()
                .map(|value| Box::new(substitute_universe_expr(value, universe_bindings))),
            end: end
                .as_ref()
                .map(|value| Box::new(substitute_universe_expr(value, universe_bindings))),
        },
        Expr::Unary { op, expr } => Expr::Unary {
            op: op.clone(),
            expr: Box::new(substitute_universe_expr(expr, universe_bindings)),
        },
        Expr::Binary { left, op, right } => Expr::Binary {
            left: Box::new(substitute_universe_expr(left, universe_bindings)),
            op: op.clone(),
            right: Box::new(substitute_universe_expr(right, universe_bindings)),
        },
        Expr::Range { start, end } => Expr::Range {
            start: Box::new(substitute_universe_expr(start, universe_bindings)),
            end: Box::new(substitute_universe_expr(end, universe_bindings)),
        },
        Expr::Await(inner) => {
            Expr::Await(Box::new(substitute_universe_expr(inner, universe_bindings)))
        }
        Expr::Try(inner) => Expr::Try(Box::new(substitute_universe_expr(inner, universe_bindings))),
        _ => expr.clone(),
    }
}

fn instrument_pool_selection_key(key: &str) -> InstrumentPoolSelectionKey {
    match key {
        "symbol" => InstrumentPoolSelectionKey::Symbol,
        "market_cap" | "volume_24h" | "listing_age_days" => {
            InstrumentPoolSelectionKey::MetadataField(key.to_string())
        }
        other => InstrumentPoolSelectionKey::Feature(other.to_string()),
    }
}

fn substitute_symbol_in_stmt(stmt: &Stmt, pattern: &str, symbol: &Symbol) -> Stmt {
    match stmt {
        Stmt::Let {
            pattern: binding,
            ty,
            value,
            mutable,
        } => Stmt::Let {
            pattern: binding.clone(),
            ty: ty.clone(),
            value: substitute_symbol_in_expr(value, pattern, symbol),
            mutable: *mutable,
        },
        Stmt::Return(value) => Stmt::Return(
            value
                .as_ref()
                .map(|expr| substitute_symbol_in_expr(expr, pattern, symbol)),
        ),
        Stmt::EmitIntent { args } => Stmt::EmitIntent {
            args: args
                .iter()
                .map(|arg| CallArg {
                    name: arg.name.clone(),
                    value: substitute_symbol_in_expr(&arg.value, pattern, symbol),
                })
                .collect(),
        },
        Stmt::If {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
        } => Stmt::If {
            condition: substitute_symbol_in_expr(condition, pattern, symbol),
            then_branch: then_branch
                .iter()
                .map(|stmt| substitute_symbol_in_stmt(stmt, pattern, symbol))
                .collect(),
            else_if_branches: else_if_branches
                .iter()
                .map(|(condition, branch)| {
                    (
                        substitute_symbol_in_expr(condition, pattern, symbol),
                        branch
                            .iter()
                            .map(|stmt| substitute_symbol_in_stmt(stmt, pattern, symbol))
                            .collect(),
                    )
                })
                .collect(),
            else_branch: else_branch.as_ref().map(|branch| {
                branch
                    .iter()
                    .map(|stmt| substitute_symbol_in_stmt(stmt, pattern, symbol))
                    .collect()
            }),
        },
        Stmt::For {
            pattern: binding,
            iterable,
            body,
        } => Stmt::For {
            pattern: binding.clone(),
            iterable: substitute_symbol_in_expr(iterable, pattern, symbol),
            body: body
                .iter()
                .map(|stmt| substitute_symbol_in_stmt(stmt, pattern, symbol))
                .collect(),
        },
        Stmt::While { condition, body } => Stmt::While {
            condition: substitute_symbol_in_expr(condition, pattern, symbol),
            body: body
                .iter()
                .map(|stmt| substitute_symbol_in_stmt(stmt, pattern, symbol))
                .collect(),
        },
        Stmt::Match { expr, arms } => Stmt::Match {
            expr: substitute_symbol_in_expr(expr, pattern, symbol),
            arms: arms
                .iter()
                .map(|arm| crate::script::MatchArm {
                    pattern: arm.pattern.clone(),
                    body: match &arm.body {
                        MatchArmBody::Statement(stmt) => MatchArmBody::Statement(Box::new(
                            substitute_symbol_in_stmt(stmt, pattern, symbol),
                        )),
                        MatchArmBody::Expr(expr) => {
                            MatchArmBody::Expr(substitute_symbol_in_expr(expr, pattern, symbol))
                        }
                    },
                })
                .collect(),
        },
        Stmt::Expr(expr) => Stmt::Expr(substitute_symbol_in_expr(expr, pattern, symbol)),
    }
}

fn substitute_symbol_in_expr(expr: &Expr, pattern: &str, symbol: &Symbol) -> Expr {
    match expr {
        Expr::Identifier(name) if name == pattern => Expr::String(symbol.as_str().to_string()),
        Expr::List(items) => Expr::List(
            items
                .iter()
                .map(|item| substitute_symbol_in_expr(item, pattern, symbol))
                .collect(),
        ),
        Expr::Call { callee, args } => Expr::Call {
            callee: Box::new(substitute_symbol_in_expr(callee, pattern, symbol)),
            args: args
                .iter()
                .map(|arg| CallArg {
                    name: arg.name.clone(),
                    value: substitute_symbol_in_expr(&arg.value, pattern, symbol),
                })
                .collect(),
        },
        Expr::Member { object, field } => Expr::Member {
            object: Box::new(substitute_symbol_in_expr(object, pattern, symbol)),
            field: field.clone(),
        },
        Expr::Index { object, index } => Expr::Index {
            object: Box::new(substitute_symbol_in_expr(object, pattern, symbol)),
            index: Box::new(substitute_symbol_in_expr(index, pattern, symbol)),
        },
        Expr::Slice { object, start, end } => Expr::Slice {
            object: Box::new(substitute_symbol_in_expr(object, pattern, symbol)),
            start: start
                .as_ref()
                .map(|value| Box::new(substitute_symbol_in_expr(value, pattern, symbol))),
            end: end
                .as_ref()
                .map(|value| Box::new(substitute_symbol_in_expr(value, pattern, symbol))),
        },
        Expr::Unary { op, expr } => Expr::Unary {
            op: op.clone(),
            expr: Box::new(substitute_symbol_in_expr(expr, pattern, symbol)),
        },
        Expr::Binary { left, op, right } => Expr::Binary {
            left: Box::new(substitute_symbol_in_expr(left, pattern, symbol)),
            op: op.clone(),
            right: Box::new(substitute_symbol_in_expr(right, pattern, symbol)),
        },
        Expr::Range { start, end } => Expr::Range {
            start: Box::new(substitute_symbol_in_expr(start, pattern, symbol)),
            end: Box::new(substitute_symbol_in_expr(end, pattern, symbol)),
        },
        Expr::Await(inner) => {
            Expr::Await(Box::new(substitute_symbol_in_expr(inner, pattern, symbol)))
        }
        Expr::Try(inner) => Expr::Try(Box::new(substitute_symbol_in_expr(inner, pattern, symbol))),
        _ => expr.clone(),
    }
}

fn evaluate_universe_expr(
    expr: &Expr,
    context: &LoweringContext,
    best_effort_pool_extraction: bool,
    universe_bindings: &BTreeMap<String, UniverseValue>,
) -> Result<Option<UniverseValue>> {
    match expr {
        Expr::Identifier(name) => Ok(universe_bindings.get(name).cloned()),
        Expr::Call { callee, args } => {
            let Expr::Identifier(name) = callee.as_ref() else {
                return Ok(None);
            };
            match name.as_str() {
                "symbols" => decode_symbols_literal(args)
                    .map(|symbols| UniverseValue {
                        symbols,
                        instrument_pool: InstrumentPoolSpec {
                            source: InstrumentPoolSourceSpec::ExplicitSymbols,
                            eligibility_rules: Vec::new(),
                            feature_defs: Vec::new(),
                            selection_rule: None,
                            weighting_rule: None,
                            rebalance_rule: None,
                        },
                    })
                    .map(Some),
                "universe" => {
                    let snapshot = context
                        .universe_snapshot
                        .as_ref()
                        .ok_or_else(|| anyhow!(ERR_UNIVERSE_SNAPSHOT_REQUIRED))?;
                    let exchange_filter = arg_string_optional(args, ArgSelector::Named("exchange"))
                        .map(|value| value.to_ascii_lowercase());
                    let market_filter = arg_string_optional(args, ArgSelector::Named("market"))
                        .map(|value| value.to_ascii_lowercase());
                    let quote_filter = arg_string_optional(args, ArgSelector::Named("quote"))
                        .map(|value| value.to_ascii_uppercase());
                    let symbols = snapshot
                        .assets
                        .iter()
                        .filter(|asset| asset.enabled)
                        .filter(|asset| asset_is_listed_as_of(asset, snapshot.as_of_ms))
                        .filter(|asset| {
                            exchange_filter
                                .as_deref()
                                .map(|value| format_exchange(&asset.exchange) == value)
                                .unwrap_or(true)
                        })
                        .filter(|asset| {
                            market_filter
                                .as_deref()
                                .map(|value| format_market_type(&asset.market_type) == value)
                                .unwrap_or(true)
                        })
                        .filter(|asset| {
                            quote_filter
                                .as_deref()
                                .map(|value| asset.quote.as_deref() == Some(value))
                                .unwrap_or(true)
                        })
                        .map(|asset| asset.symbol.clone())
                        .collect::<Vec<_>>();
                    Ok(Some(UniverseValue {
                        symbols,
                        instrument_pool: InstrumentPoolSpec {
                            source: InstrumentPoolSourceSpec::Universe {
                                exchange: exchange_filter,
                                market: market_filter,
                                quote: quote_filter,
                            },
                            eligibility_rules: Vec::new(),
                            feature_defs: Vec::new(),
                            selection_rule: None,
                            weighting_rule: None,
                            rebalance_rule: None,
                        },
                    }))
                }
                "filter" => {
                    let mut universe_value = evaluate_universe_arg(
                        args,
                        context,
                        best_effort_pool_extraction,
                        universe_bindings,
                    )?;
                    let quote_filter = arg_string_optional(args, ArgSelector::Named("quote"))
                        .map(|value| value.to_ascii_uppercase());
                    let exchange_filter = arg_string_optional(args, ArgSelector::Named("exchange"))
                        .map(|value| value.to_ascii_lowercase());
                    let market_filter = arg_string_optional(args, ArgSelector::Named("market"))
                        .map(|value| value.to_ascii_lowercase());
                    let min_market_cap =
                        arg_number_optional(args, ArgSelector::Named("min_market_cap"));
                    let min_volume_24h =
                        arg_number_optional(args, ArgSelector::Named("min_volume_24h"));
                    let min_listing_age_days =
                        arg_number_optional(args, ArgSelector::Named("min_listing_age_days"));
                    if quote_filter.is_none()
                        && exchange_filter.is_none()
                        && market_filter.is_none()
                        && min_market_cap.is_none()
                        && min_volume_24h.is_none()
                        && min_listing_age_days.is_none()
                    {
                        return Ok(Some(universe_value));
                    }
                    let snapshot = context
                        .universe_snapshot
                        .as_ref()
                        .ok_or_else(|| anyhow!(ERR_UNIVERSE_SNAPSHOT_REQUIRED))?;
                    universe_value.symbols.retain(|symbol| {
                        snapshot
                            .assets
                            .iter()
                            .find(|asset| &asset.symbol == symbol)
                            .map(|asset| {
                                let metrics = universe_asset_metrics(snapshot, asset);
                                asset_is_listed_as_of(asset, snapshot.as_of_ms)
                                    && quote_filter
                                        .as_deref()
                                        .map(|value| asset.quote.as_deref() == Some(value))
                                        .unwrap_or(true)
                                    && exchange_filter
                                        .as_deref()
                                        .map(|value| format_exchange(&asset.exchange) == value)
                                        .unwrap_or(true)
                                    && market_filter
                                        .as_deref()
                                        .map(|value| {
                                            format_market_type(&asset.market_type) == value
                                        })
                                        .unwrap_or(true)
                                    && min_market_cap
                                        .map(|value| {
                                            metrics.market_cap.unwrap_or(f64::NEG_INFINITY) >= value
                                        })
                                        .unwrap_or(true)
                                    && min_volume_24h
                                        .map(|value| {
                                            metrics.volume_24h.unwrap_or(f64::NEG_INFINITY) >= value
                                        })
                                        .unwrap_or(true)
                                    && min_listing_age_days
                                        .map(|value| {
                                            metrics.listing_age_days.unwrap_or(f64::NEG_INFINITY)
                                                >= value
                                        })
                                        .unwrap_or(true)
                            })
                            .unwrap_or(false)
                    });
                    if let Some(value) = quote_filter {
                        universe_value.instrument_pool.eligibility_rules.push(
                            InstrumentPoolEligibilityRule {
                                field: "quote".into(),
                                op: "=".into(),
                                value: InstrumentPoolValue::String(value),
                            },
                        );
                    }
                    if let Some(value) = exchange_filter {
                        universe_value.instrument_pool.eligibility_rules.push(
                            InstrumentPoolEligibilityRule {
                                field: "exchange".into(),
                                op: "=".into(),
                                value: InstrumentPoolValue::String(value),
                            },
                        );
                    }
                    if let Some(value) = market_filter {
                        universe_value.instrument_pool.eligibility_rules.push(
                            InstrumentPoolEligibilityRule {
                                field: "market".into(),
                                op: "=".into(),
                                value: InstrumentPoolValue::String(value),
                            },
                        );
                    }
                    if let Some(value) = min_market_cap {
                        universe_value.instrument_pool.eligibility_rules.push(
                            InstrumentPoolEligibilityRule {
                                field: "market_cap".into(),
                                op: ">=".into(),
                                value: InstrumentPoolValue::Number(value),
                            },
                        );
                    }
                    if let Some(value) = min_volume_24h {
                        universe_value.instrument_pool.eligibility_rules.push(
                            InstrumentPoolEligibilityRule {
                                field: "volume_24h".into(),
                                op: ">=".into(),
                                value: InstrumentPoolValue::Number(value),
                            },
                        );
                    }
                    if let Some(value) = min_listing_age_days {
                        universe_value.instrument_pool.eligibility_rules.push(
                            InstrumentPoolEligibilityRule {
                                field: "listing_age_days".into(),
                                op: ">=".into(),
                                value: InstrumentPoolValue::Number(value),
                            },
                        );
                    }
                    Ok(Some(universe_value))
                }
                "sort_by" => {
                    let mut universe_value = evaluate_universe_arg(
                        args,
                        context,
                        best_effort_pool_extraction,
                        universe_bindings,
                    )?;
                    let key = arg_string_optional(args, ArgSelector::NamedOrPositional("key", 1))
                        .unwrap_or_else(|| "symbol".to_string())
                        .to_ascii_lowercase();
                    let order = arg_string_optional(args, ArgSelector::Named("order"))
                        .unwrap_or_else(|| "asc".to_string())
                        .to_ascii_lowercase();
                    let selection_key = instrument_pool_selection_key(&key);
                    universe_value.instrument_pool.selection_rule =
                        Some(InstrumentPoolSelectionRule {
                            kind: "ordered".into(),
                            key: Some(selection_key.clone()),
                            order: Some(order.clone()),
                            count: universe_value
                                .instrument_pool
                                .selection_rule
                                .as_ref()
                                .and_then(|rule| rule.count),
                        });
                    let sorted_desc = match selection_key {
                        InstrumentPoolSelectionKey::Symbol => {
                            universe_value
                                .symbols
                                .sort_by(|left, right| right.as_str().cmp(left.as_str()));
                            true
                        }
                        InstrumentPoolSelectionKey::MetadataField(ref field)
                            if field == "market_cap" =>
                        {
                            let snapshot = context
                                .universe_snapshot
                                .as_ref()
                                .ok_or_else(|| anyhow!(ERR_UNIVERSE_SNAPSHOT_REQUIRED))?;
                            universe_value.symbols.sort_by(|left, right| {
                                compare_symbols_by_metric(snapshot, left, right, |metrics| {
                                    metrics.market_cap
                                })
                            });
                            true
                        }
                        InstrumentPoolSelectionKey::MetadataField(ref field)
                            if field == "volume_24h" =>
                        {
                            let snapshot = context
                                .universe_snapshot
                                .as_ref()
                                .ok_or_else(|| anyhow!(ERR_UNIVERSE_SNAPSHOT_REQUIRED))?;
                            universe_value.symbols.sort_by(|left, right| {
                                compare_symbols_by_metric(snapshot, left, right, |metrics| {
                                    metrics.volume_24h
                                })
                            });
                            true
                        }
                        InstrumentPoolSelectionKey::MetadataField(ref field)
                            if field == "listing_age_days" =>
                        {
                            let snapshot = context
                                .universe_snapshot
                                .as_ref()
                                .ok_or_else(|| anyhow!(ERR_UNIVERSE_SNAPSHOT_REQUIRED))?;
                            universe_value.symbols.sort_by(|left, right| {
                                compare_symbols_by_metric(snapshot, left, right, |metrics| {
                                    metrics.listing_age_days
                                })
                            });
                            true
                        }
                        InstrumentPoolSelectionKey::MetadataField(field) => {
                            bail!("{ERR_UNIVERSE_SORT_KEY}: {field}")
                        }
                        InstrumentPoolSelectionKey::Feature(feature)
                            if best_effort_pool_extraction =>
                        {
                            let _ = feature;
                            false
                        }
                        InstrumentPoolSelectionKey::Feature(feature) => {
                            bail!("{ERR_UNIVERSE_SORT_KEY}: {feature}")
                        }
                    };
                    if order == "asc" && sorted_desc {
                        universe_value.symbols.reverse();
                    } else if order != "desc" && order != "asc" {
                        bail!("{ERR_UNIVERSE_SORT_ORDER}: {order}");
                    }
                    Ok(Some(universe_value))
                }
                "top" => {
                    let mut universe_value = evaluate_universe_arg(
                        args,
                        context,
                        best_effort_pool_extraction,
                        universe_bindings,
                    )?;
                    let count = arg_number_optional(args, ArgSelector::Positional(1))
                        .ok_or_else(|| anyhow!(ERR_TOP_COUNT_REQUIRED))?
                        .round()
                        .max(0.0) as usize;
                    universe_value.symbols.truncate(count);
                    let existing = universe_value.instrument_pool.selection_rule.take();
                    universe_value.instrument_pool.selection_rule = Some(match existing {
                        Some(rule) => InstrumentPoolSelectionRule {
                            kind: if rule.key.is_some() {
                                "ordered_top_n".into()
                            } else {
                                "top_n".into()
                            },
                            key: rule.key,
                            order: rule.order,
                            count: Some(count),
                        },
                        None => InstrumentPoolSelectionRule {
                            kind: "top_n".into(),
                            key: None,
                            order: None,
                            count: Some(count),
                        },
                    });
                    Ok(Some(universe_value))
                }
                _ => Ok(None),
            }
        }
        _ => Ok(None),
    }
}

fn evaluate_universe_arg(
    args: &[CallArg],
    context: &LoweringContext,
    best_effort_pool_extraction: bool,
    universe_bindings: &BTreeMap<String, UniverseValue>,
) -> Result<UniverseValue> {
    let expr = find_arg(args, ArgSelector::Positional(0))
        .ok_or_else(|| anyhow!(ERR_UNIVERSE_VALUE_REQUIRED))?;
    evaluate_universe_expr(
        expr,
        context,
        best_effort_pool_extraction,
        universe_bindings,
    )?
    .ok_or_else(|| anyhow!(ERR_UNIVERSE_VALUE_REQUIRED))
}

fn decode_symbols_literal(args: &[CallArg]) -> Result<Vec<Symbol>> {
    let expr = find_arg(args, ArgSelector::Positional(0))
        .ok_or_else(|| anyhow!(ERR_SYMBOLS_LIST_LITERAL))?;
    match expr {
        Expr::List(items) => items
            .iter()
            .map(|item| {
                expr_string(item)
                    .map(|value| Symbol::parse(&value))
                    .ok_or_else(|| anyhow!(ERR_SYMBOLS_STRING_LITERAL))
            })
            .collect(),
        _ => bail!(ERR_SYMBOLS_LIST_LITERAL),
    }
}

fn asset_record_for_symbol<'a>(
    snapshot: &'a UniverseSnapshot,
    symbol: &Symbol,
) -> Option<&'a UniverseAssetRecord> {
    snapshot.assets.iter().find(|asset| &asset.symbol == symbol)
}

fn compare_symbols_by_metric(
    snapshot: &UniverseSnapshot,
    left: &Symbol,
    right: &Symbol,
    selector: impl Fn(&UniverseAssetMetrics) -> Option<f64>,
) -> std::cmp::Ordering {
    let left_record = asset_record_for_symbol(snapshot, left);
    let right_record = asset_record_for_symbol(snapshot, right);
    let left_value = left_record
        .and_then(|asset| selector(&universe_asset_metrics(snapshot, asset)))
        .unwrap_or(f64::NEG_INFINITY);
    let right_value = right_record
        .and_then(|asset| selector(&universe_asset_metrics(snapshot, asset)))
        .unwrap_or(f64::NEG_INFINITY);
    right_value
        .partial_cmp(&left_value)
        .unwrap_or(std::cmp::Ordering::Equal)
        .then_with(|| left.as_str().cmp(right.as_str()))
}

fn universe_asset_metrics(
    snapshot: &UniverseSnapshot,
    asset: &UniverseAssetRecord,
) -> UniverseAssetMetrics {
    let point = latest_metadata_point_at_or_before(asset, snapshot.as_of_ms);
    let listing_age_days = if let Some(listed_at_ms) = asset.listed_at_ms {
        (snapshot.as_of_ms >= listed_at_ms)
            .then(|| (snapshot.as_of_ms - listed_at_ms) as f64 / 86_400_000.0)
    } else {
        point
            .and_then(|entry| entry.listing_age_days)
            .or(asset.listing_age_days)
    };

    UniverseAssetMetrics {
        market_cap: point
            .and_then(|entry| entry.market_cap)
            .or(asset.market_cap),
        volume_24h: point
            .and_then(|entry| entry.volume_24h)
            .or(asset.volume_24h),
        listing_age_days,
    }
}

fn latest_metadata_point_at_or_before(
    asset: &UniverseAssetRecord,
    as_of_ms: u64,
) -> Option<&UniverseAssetMetadataPoint> {
    asset
        .metadata_history
        .iter()
        .filter(|entry| entry.as_of_ms <= as_of_ms)
        .max_by_key(|entry| entry.as_of_ms)
}

fn asset_is_listed_as_of(asset: &UniverseAssetRecord, as_of_ms: u64) -> bool {
    asset
        .listed_at_ms
        .map(|listed_at| listed_at <= as_of_ms)
        .unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_quant_script_module;
    use qrpc_core::{Exchange, MarketType, RebalanceSchedule, UniverseSnapshot};

    use crate::{InstrumentPoolRebalanceRule, InstrumentPoolWeightingRule};

    fn sample_universe_snapshot() -> UniverseSnapshot {
        UniverseSnapshot {
            snapshot_id: "pool_semantics_test".into(),
            as_of_ms: 1_710_000_000_000,
            assets: vec![
                UniverseAssetRecord {
                    symbol: Symbol::BtcUsdt,
                    exchange: Exchange::Binance,
                    market_type: MarketType::Spot,
                    quote: Some("USDT".into()),
                    listed_at_ms: Some(1_500_000_000_000),
                    enabled: true,
                    market_cap: Some(1_500_000_000_000.0),
                    volume_24h: Some(40_000_000_000.0),
                    listing_age_days: None,
                    metadata_history: Vec::new(),
                },
                UniverseAssetRecord {
                    symbol: Symbol::parse("ETHUSDT"),
                    exchange: Exchange::Binance,
                    market_type: MarketType::Spot,
                    quote: Some("USDT".into()),
                    listed_at_ms: Some(1_510_000_000_000),
                    enabled: true,
                    market_cap: Some(500_000_000_000.0),
                    volume_24h: Some(18_000_000_000.0),
                    listing_age_days: None,
                    metadata_history: Vec::new(),
                },
                UniverseAssetRecord {
                    symbol: Symbol::parse("SOLUSDT"),
                    exchange: Exchange::Binance,
                    market_type: MarketType::Spot,
                    quote: Some("USDT".into()),
                    listed_at_ms: Some(1_520_000_000_000),
                    enabled: true,
                    market_cap: Some(120_000_000_000.0),
                    volume_24h: Some(4_000_000_000.0),
                    listing_age_days: None,
                    metadata_history: Vec::new(),
                },
            ],
        }
    }

    #[test]
    fn detects_metadata_ranked_pool_semantics_for_rebalance_directive() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let base = universe(exchange="binance", market="spot", quote="USDT")
    let liquid = filter(base, min_volume_24h=1000000000, min_listing_age_days=180)
    let leaders = top(sort_by(liquid, key="market_cap", order="desc"), 2)
    rebalance(rank_weight(leaders, method="linear"), every="weekly")
}
"#,
        )
        .unwrap();

        let directive = detect_portfolio_rebalance_directive(
            &module,
            &LoweringContext {
                universe_snapshot: Some(sample_universe_snapshot()),
            },
        )
        .unwrap()
        .expect("expected rebalance directive");

        assert_eq!(
            directive.instrument_pool.source,
            InstrumentPoolSourceSpec::Universe {
                exchange: Some("binance".into()),
                market: Some("spot".into()),
                quote: Some("USDT".into()),
            }
        );
        assert_eq!(
            directive
                .instrument_pool
                .eligibility_rules
                .iter()
                .map(|rule| rule.field.as_str())
                .collect::<Vec<_>>(),
            vec!["volume_24h", "listing_age_days"]
        );
        assert_eq!(
            directive.instrument_pool.selection_rule,
            Some(InstrumentPoolSelectionRule {
                kind: "ordered_top_n".into(),
                key: Some(InstrumentPoolSelectionKey::MetadataField(
                    "market_cap".into()
                )),
                order: Some("desc".into()),
                count: Some(2),
            })
        );
        assert_eq!(
            directive.instrument_pool.weighting_rule,
            Some(InstrumentPoolWeightingRule {
                kind: "rank_weight".into(),
                method: Some("linear".into()),
                normalize: None,
                target_weights: Vec::new(),
            })
        );
        assert_eq!(
            directive.instrument_pool.rebalance_rule,
            Some(InstrumentPoolRebalanceRule {
                every: Some(RebalanceSchedule::Weekly),
            })
        );
    }

    #[test]
    fn classifies_factor_score_sort_key_as_feature_selection_key() {
        assert_eq!(
            instrument_pool_selection_key("factor_score"),
            InstrumentPoolSelectionKey::Feature("factor_score".into())
        );
    }
}
