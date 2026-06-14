use anyhow::Result;
use qrpc_core::Symbol;
use std::collections::{BTreeMap, BTreeSet};

use crate::script::{CallArg, Expr, FunctionDecl, Item, MatchArmBody, ScriptModule, Stmt};

use super::super::context::LoweringContext;
use super::super::shared::sanitize_id;
use super::{evaluate_universe_expr, UniverseValue};

pub(super) fn expand_universe_constructs(
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
