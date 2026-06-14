use crate::script::{CallArg, Expr, FunctionDecl, Item, MatchArmBody, ScriptModule, Stmt};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};

mod folding_value_wave;
mod helper_inline_execution_wave;

use folding_value_wave::{fold_binary, fold_builtin_call, fold_index, fold_slice, fold_unary};
use helper_inline_execution_wave::maybe_inline_function;

pub fn normalize_script_module(module: &ScriptModule) -> Result<ScriptModule> {
    let functions = module
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Function(function) => Some((function.name.clone(), function.clone())),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    let context = EvalContext { functions };

    let items = module
        .items
        .iter()
        .map(|item| match item {
            Item::Import(import) => Ok(Item::Import(import.clone())),
            Item::Function(function) => Ok(Item::Function(normalize_function(function, &context)?)),
            Item::TestBlock(test_block) => Ok(Item::TestBlock(test_block.clone())),
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(ScriptModule { items })
}

#[derive(Debug, Clone)]
struct EvalContext {
    functions: BTreeMap<String, FunctionDecl>,
}

fn normalize_function(function: &FunctionDecl, context: &EvalContext) -> Result<FunctionDecl> {
    Ok(FunctionDecl {
        is_async: function.is_async,
        name: function.name.clone(),
        params: function.params.clone(),
        return_type: function.return_type.clone(),
        body: normalize_block(
            &function.body,
            &mut BTreeMap::new(),
            context,
            &mut BTreeSet::new(),
        )?,
    })
}

fn normalize_block(
    stmts: &[Stmt],
    env: &mut BTreeMap<String, Expr>,
    context: &EvalContext,
    stack: &mut BTreeSet<String>,
) -> Result<Vec<Stmt>> {
    stmts
        .iter()
        .map(|stmt| normalize_stmt(stmt, env, context, stack))
        .collect()
}

fn normalize_stmt(
    stmt: &Stmt,
    env: &mut BTreeMap<String, Expr>,
    context: &EvalContext,
    stack: &mut BTreeSet<String>,
) -> Result<Stmt> {
    match stmt {
        Stmt::Let {
            pattern,
            ty,
            value,
            mutable,
        } => {
            let normalized = normalize_expr(value, env, context, stack)?;
            env.insert(pattern.clone(), normalized.clone());
            Ok(Stmt::Let {
                pattern: pattern.clone(),
                ty: ty.clone(),
                value: normalized,
                mutable: *mutable,
            })
        }
        Stmt::Return(value) => Ok(Stmt::Return(
            value
                .as_ref()
                .map(|expr| normalize_expr(expr, env, context, stack))
                .transpose()?,
        )),
        Stmt::EmitIntent { args } => Ok(Stmt::EmitIntent {
            args: args
                .iter()
                .map(|arg| {
                    Ok(CallArg {
                        name: arg.name.clone(),
                        value: normalize_expr(&arg.value, env, context, stack)?,
                    })
                })
                .collect::<Result<Vec<_>>>()?,
        }),
        Stmt::If {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
        } => Ok(Stmt::If {
            condition: normalize_expr(condition, env, context, stack)?,
            then_branch: normalize_block(then_branch, &mut env.clone(), context, stack)?,
            else_if_branches: else_if_branches
                .iter()
                .map(|(expr, branch)| {
                    Ok((
                        normalize_expr(expr, env, context, stack)?,
                        normalize_block(branch, &mut env.clone(), context, stack)?,
                    ))
                })
                .collect::<Result<Vec<_>>>()?,
            else_branch: else_branch
                .as_ref()
                .map(|branch| normalize_block(branch, &mut env.clone(), context, stack))
                .transpose()?,
        }),
        Stmt::For {
            pattern,
            iterable,
            body,
        } => Ok(Stmt::For {
            pattern: pattern.clone(),
            iterable: normalize_expr(iterable, env, context, stack)?,
            body: normalize_block(body, &mut env.clone(), context, stack)?,
        }),
        Stmt::While { condition, body } => Ok(Stmt::While {
            condition: normalize_expr(condition, env, context, stack)?,
            body: normalize_block(body, &mut env.clone(), context, stack)?,
        }),
        Stmt::Match { expr, arms } => Ok(Stmt::Match {
            expr: normalize_expr(expr, env, context, stack)?,
            arms: arms
                .iter()
                .map(|arm| {
                    Ok(crate::script::MatchArm {
                        pattern: arm.pattern.clone(),
                        body: match &arm.body {
                            MatchArmBody::Statement(stmt) => MatchArmBody::Statement(Box::new(
                                normalize_stmt(stmt, &mut env.clone(), context, stack)?,
                            )),
                            MatchArmBody::Expr(expr) => {
                                MatchArmBody::Expr(normalize_expr(expr, env, context, stack)?)
                            }
                        },
                    })
                })
                .collect::<Result<Vec<_>>>()?,
        }),
        Stmt::Expr(expr) => Ok(Stmt::Expr(normalize_expr(expr, env, context, stack)?)),
    }
}

fn normalize_expr(
    expr: &Expr,
    env: &BTreeMap<String, Expr>,
    context: &EvalContext,
    stack: &mut BTreeSet<String>,
) -> Result<Expr> {
    match expr {
        Expr::Identifier(name) => Ok(env
            .get(name)
            .cloned()
            .unwrap_or_else(|| Expr::Identifier(name.clone()))),
        Expr::Number(_) | Expr::String(_) | Expr::Bool(_) | Expr::Raw(_) => Ok(expr.clone()),
        Expr::List(items) => Ok(Expr::List(
            items
                .iter()
                .map(|item| normalize_expr(item, env, context, stack))
                .collect::<Result<Vec<_>>>()?,
        )),
        Expr::Unary { op, expr } => {
            let normalized = normalize_expr(expr, env, context, stack)?;
            fold_unary(op.clone(), normalized)
        }
        Expr::Binary { left, op, right } => {
            let normalized_left = normalize_expr(left, env, context, stack)?;
            let normalized_right = normalize_expr(right, env, context, stack)?;
            fold_binary(normalized_left, op.clone(), normalized_right)
        }
        Expr::Member { object, field } => Ok(Expr::Member {
            object: Box::new(normalize_expr(object, env, context, stack)?),
            field: field.clone(),
        }),
        Expr::Call { callee, args } => {
            let normalized_callee = normalize_expr(callee, env, context, stack)?;
            let normalized_args = args
                .iter()
                .map(|arg| {
                    Ok(CallArg {
                        name: arg.name.clone(),
                        value: normalize_expr(&arg.value, env, context, stack)?,
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            if let Some(inlined) =
                maybe_inline_function(&normalized_callee, &normalized_args, context, stack)?
            {
                return Ok(inlined);
            }

            fold_builtin_call(normalized_callee, normalized_args)
        }
        Expr::Index { object, index } => {
            let normalized_object = normalize_expr(object, env, context, stack)?;
            let normalized_index = normalize_expr(index, env, context, stack)?;
            fold_index(normalized_object, normalized_index)
        }
        Expr::Slice { object, start, end } => {
            let normalized_object = normalize_expr(object, env, context, stack)?;
            let normalized_start = start
                .as_ref()
                .map(|expr| normalize_expr(expr, env, context, stack).map(Box::new))
                .transpose()?;
            let normalized_end = end
                .as_ref()
                .map(|expr| normalize_expr(expr, env, context, stack).map(Box::new))
                .transpose()?;
            fold_slice(normalized_object, normalized_start, normalized_end)
        }
        Expr::Range { start, end } => Ok(Expr::Range {
            start: Box::new(normalize_expr(start, env, context, stack)?),
            end: Box::new(normalize_expr(end, env, context, stack)?),
        }),
        Expr::Await(inner) => Ok(Expr::Await(Box::new(normalize_expr(
            inner, env, context, stack,
        )?))),
        Expr::Try(inner) => Ok(Expr::Try(Box::new(normalize_expr(
            inner, env, context, stack,
        )?))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_quant_script_module;

    #[test]
    fn normalizes_user_defined_indicator_helper_calls() {
        let module = parse_quant_script_module(
            r#"
fn custom_rsi(data, period) {
    let p = period
    return rsi(data, p)
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let signal = custom_rsi(closes, 14)
    if signal < 30 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let normalized = normalize_script_module(&module).unwrap();
        let strategy = normalized
            .items
            .iter()
            .find_map(|item| match item {
                Item::Function(function) if function.name == "strategy" => Some(function),
                _ => None,
            })
            .unwrap();
        let Stmt::Let { value, .. } = &strategy.body[1] else {
            panic!("expected normalized let");
        };
        assert!(matches!(value, Expr::Call { .. }));
    }

    #[test]
    fn folds_fetch_health_helpers_in_conditions() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let data = fetch("BTCUSDT", interval="1d", lookback=120)?
    if data.ok() {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let normalized = normalize_script_module(&module).unwrap();
        let strategy = normalized
            .items
            .iter()
            .find_map(|item| match item {
                Item::Function(function) if function.name == "strategy" => Some(function),
                _ => None,
            })
            .unwrap();
        let Stmt::If { condition, .. } = &strategy.body[1] else {
            panic!("expected if");
        };
        assert_eq!(condition, &Expr::Bool(true));
    }

    #[test]
    fn executes_for_loop_inside_helper_function() {
        let module = parse_quant_script_module(
            r#"
fn accumulate(values) {
    let total = 0
    for item in values {
        let total = total + item
    }
    return total
}

fn strategy() {
    let score = accumulate([1, 2, 3, 4])
    return score
}
"#,
        )
        .unwrap();

        let normalized = normalize_script_module(&module).unwrap();
        let strategy = normalized
            .items
            .iter()
            .find_map(|item| match item {
                Item::Function(function) if function.name == "strategy" => Some(function),
                _ => None,
            })
            .unwrap();
        let Stmt::Let { value, .. } = &strategy.body[0] else {
            panic!("expected normalized let");
        };
        assert_eq!(value, &Expr::Number(10.0));
    }

    #[test]
    fn executes_while_loop_inside_helper_function() {
        let module = parse_quant_script_module(
            r#"
fn grow_until(limit) {
    let value = 1
    while value < limit {
        let value = value + 2
    }
    return value
}

fn strategy() {
    let score = grow_until(6)
    return score
}
"#,
        )
        .unwrap();

        let normalized = normalize_script_module(&module).unwrap();
        let strategy = normalized
            .items
            .iter()
            .find_map(|item| match item {
                Item::Function(function) if function.name == "strategy" => Some(function),
                _ => None,
            })
            .unwrap();
        let Stmt::Let { value, .. } = &strategy.body[0] else {
            panic!("expected normalized let");
        };
        assert_eq!(value, &Expr::Number(7.0));
    }

    #[test]
    fn merges_symbolic_if_when_all_branches_converge() {
        let module = parse_quant_script_module(
            r#"
fn stable_signal(period) {
    if period > 20 {
        return 14
    } else {
        return 14
    }
}

fn strategy() {
    let signal = stable_signal(foo)
    return signal
}
"#,
        )
        .unwrap();

        let normalized = normalize_script_module(&module).unwrap();
        let strategy = normalized
            .items
            .iter()
            .find_map(|item| match item {
                Item::Function(function) if function.name == "strategy" => Some(function),
                _ => None,
            })
            .unwrap();
        let Stmt::Let { value, .. } = &strategy.body[0] else {
            panic!("expected normalized let");
        };
        assert_eq!(value, &Expr::Number(14.0));
    }

    #[test]
    fn executes_match_branch_inside_helper_function() {
        let module = parse_quant_script_module(
            r#"
fn resolve_window(mode) {
    match mode {
        "fast" => 12,
        "slow" => 26,
        _ => 9,
    }
}

fn strategy() {
    let signal = resolve_window("slow")
    return signal
}
"#,
        )
        .unwrap();

        let normalized = normalize_script_module(&module).unwrap();
        let strategy = normalized
            .items
            .iter()
            .find_map(|item| match item {
                Item::Function(function) if function.name == "strategy" => Some(function),
                _ => None,
            })
            .unwrap();
        let Stmt::Let { value, .. } = &strategy.body[0] else {
            panic!("expected normalized let");
        };
        assert_eq!(value, &Expr::Number(26.0));
    }

    #[test]
    fn folds_list_index_slice_and_math_builtins() {
        let module = parse_quant_script_module(
            r#"
fn stats(values) {
    let window = values[-3..]
    let latest = values[-1]
    return (latest - mean(window)) / stddev(window)
}

fn strategy() {
    let score = stats([1, 2, 3, 4, 5])
    return score
}
"#,
        )
        .unwrap();

        let normalized = normalize_script_module(&module).unwrap();
        let strategy = normalized
            .items
            .iter()
            .find_map(|item| match item {
                Item::Function(function) if function.name == "strategy" => Some(function),
                _ => None,
            })
            .unwrap();
        let Stmt::Let { value, .. } = &strategy.body[0] else {
            panic!("expected normalized let");
        };
        let Some(score) = folding_value_wave::expr_number(value) else {
            panic!("expected numeric score");
        };
        assert!(score.is_finite());
        assert!(score > 1.0);
    }
}
