use crate::script::{
    parse_expr, BinaryOp, CallArg, Expr, FunctionDecl, Item, MatchArm, MatchArmBody, ScriptModule,
    Stmt, UnaryOp,
};
use anyhow::{anyhow, bail, Result};
use std::collections::{BTreeMap, BTreeSet};

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

#[derive(Debug, Clone, PartialEq)]
enum ExecOutcome {
    Continue,
    Return(Expr),
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

fn maybe_inline_function(
    callee: &Expr,
    args: &[CallArg],
    context: &EvalContext,
    stack: &mut BTreeSet<String>,
) -> Result<Option<Expr>> {
    let Expr::Identifier(name) = callee else {
        return Ok(None);
    };
    let Some(function) = context.functions.get(name) else {
        return Ok(None);
    };
    if !stack.insert(name.clone()) {
        bail!("不支持递归的 QuantScript 函数: {name}");
    }
    let result = evaluate_function(function, args, context, stack);
    stack.remove(name);
    match result {
        Ok(expr) => Ok(Some(expr)),
        Err(error) if should_skip_inline(&error) => Ok(None),
        Err(error) => Err(error),
    }
}

fn should_skip_inline(error: &anyhow::Error) -> bool {
    let message = error.to_string();
    message.contains("symbolic branching")
        || message.contains("symbolic while-loop")
        || message.contains("unable to symbolically expand for-loop iterable")
        || message.contains("unsupported statement in helper function")
}

fn evaluate_function(
    function: &FunctionDecl,
    args: &[CallArg],
    context: &EvalContext,
    stack: &mut BTreeSet<String>,
) -> Result<Expr> {
    let mut env = BTreeMap::new();
    for (index, param) in function.params.iter().enumerate() {
        let value = args
            .get(index)
            .map(|arg| arg.value.clone())
            .ok_or_else(|| anyhow!("缺少参数 {} 用于 {}", param.name, function.name))?;
        env.insert(param.name.clone(), value);
    }

    execute_function_body(&function.body, &mut env, context, stack)
}

fn execute_function_body(
    stmts: &[Stmt],
    env: &mut BTreeMap<String, Expr>,
    context: &EvalContext,
    stack: &mut BTreeSet<String>,
) -> Result<Expr> {
    let mut last_expr = None;
    for stmt in stmts {
        match execute_stmt(stmt, env, context, stack, &mut last_expr)? {
            ExecOutcome::Continue => {}
            ExecOutcome::Return(expr) => return Ok(expr),
        }
    }

    last_expr.ok_or_else(|| anyhow!("辅助函数必须返回一个值"))
}

fn execute_stmt(
    stmt: &Stmt,
    env: &mut BTreeMap<String, Expr>,
    context: &EvalContext,
    stack: &mut BTreeSet<String>,
    last_expr: &mut Option<Expr>,
) -> Result<ExecOutcome> {
    match stmt {
        Stmt::Let { pattern, value, .. } => {
            let normalized = normalize_expr(value, env, context, stack)?;
            env.insert(pattern.clone(), normalized);
            Ok(ExecOutcome::Continue)
        }
        Stmt::Return(Some(value)) => Ok(ExecOutcome::Return(normalize_expr(
            value, env, context, stack,
        )?)),
        Stmt::Return(None) => Ok(ExecOutcome::Return(Expr::Raw("null".into()))),
        Stmt::Expr(expr) => {
            *last_expr = Some(normalize_expr(expr, env, context, stack)?);
            Ok(ExecOutcome::Continue)
        }
        Stmt::If {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
        } => execute_if_stmt(
            IfStmtParts {
                condition,
                then_branch,
                else_if_branches,
                else_branch: else_branch.as_deref(),
            },
            env,
            context,
            stack,
            last_expr,
        ),
        Stmt::For {
            pattern,
            iterable,
            body,
        } => execute_for_stmt(pattern, iterable, body, env, context, stack, last_expr),
        Stmt::While { condition, body } => {
            execute_while_stmt(condition, body, env, context, stack, last_expr)
        }
        Stmt::Match { expr, arms } => {
            execute_match_stmt(expr, arms, env, context, stack, last_expr)
        }
        other => bail!("辅助函数中不支持的语句: {other:?}"),
    }
}

struct IfStmtParts<'a> {
    condition: &'a Expr,
    then_branch: &'a [Stmt],
    else_if_branches: &'a [(Expr, Vec<Stmt>)],
    else_branch: Option<&'a [Stmt]>,
}

fn execute_if_stmt(
    parts: IfStmtParts<'_>,
    env: &mut BTreeMap<String, Expr>,
    context: &EvalContext,
    stack: &mut BTreeSet<String>,
    last_expr: &mut Option<Expr>,
) -> Result<ExecOutcome> {
    let IfStmtParts {
        condition,
        then_branch,
        else_if_branches,
        else_branch,
    } = parts;

    let normalized_condition = normalize_expr(condition, env, context, stack)?;
    if let Some(value) = expr_bool(&normalized_condition) {
        let mut selected = if value { Some(then_branch) } else { None };
        if selected.is_none() {
            for (expr, branch) in else_if_branches {
                let branch_condition = normalize_expr(expr, env, context, stack)?;
                if expr_bool(&branch_condition) == Some(true) {
                    selected = Some(branch);
                    break;
                }
            }
        }
        let branch = selected.or(else_branch).unwrap_or(&[]);
        return execute_block_in_place(branch, env, context, stack, last_expr);
    }

    let mut snapshots = Vec::new();
    snapshots.push(execute_block_snapshot(
        then_branch,
        env.clone(),
        context,
        stack,
        last_expr.clone(),
    )?);
    for (_, branch) in else_if_branches {
        snapshots.push(execute_block_snapshot(
            branch,
            env.clone(),
            context,
            stack,
            last_expr.clone(),
        )?);
    }
    if let Some(branch) = else_branch {
        snapshots.push(execute_block_snapshot(
            branch,
            env.clone(),
            context,
            stack,
            last_expr.clone(),
        )?);
    }

    if let Some(first) = snapshots.first() {
        if snapshots.iter().all(|item| item == first) {
            *env = first.env.clone();
            *last_expr = first.last_expr.clone();
            return Ok(first.outcome.clone());
        }
    }

    bail!("符号分支产生了发散的辅助函数状态")
}

fn execute_for_stmt(
    pattern: &str,
    iterable: &Expr,
    body: &[Stmt],
    env: &mut BTreeMap<String, Expr>,
    context: &EvalContext,
    stack: &mut BTreeSet<String>,
    last_expr: &mut Option<Expr>,
) -> Result<ExecOutcome> {
    let normalized_iterable = normalize_expr(iterable, env, context, stack)?;
    let items = expr_iterable_items(&normalized_iterable)
        .ok_or_else(|| anyhow!("无法以符号方式展开 for 循环的可迭代对象"))?;
    for item in items {
        env.insert(pattern.to_string(), item);
        match execute_block_in_place(body, env, context, stack, last_expr)? {
            ExecOutcome::Continue => {}
            outcome @ ExecOutcome::Return(_) => return Ok(outcome),
        }
    }
    Ok(ExecOutcome::Continue)
}

fn execute_while_stmt(
    condition: &Expr,
    body: &[Stmt],
    env: &mut BTreeMap<String, Expr>,
    context: &EvalContext,
    stack: &mut BTreeSet<String>,
    last_expr: &mut Option<Expr>,
) -> Result<ExecOutcome> {
    for _ in 0..1024 {
        let normalized_condition = normalize_expr(condition, env, context, stack)?;
        match expr_bool(&normalized_condition) {
            Some(true) => match execute_block_in_place(body, env, context, stack, last_expr)? {
                ExecOutcome::Continue => {}
                outcome @ ExecOutcome::Return(_) => return Ok(outcome),
            },
            Some(false) => return Ok(ExecOutcome::Continue),
            None => bail!("符号化 while 循环条件无法解析"),
        }
    }
    bail!("while 循环超出符号执行迭代限制")
}

fn execute_match_stmt(
    expr: &Expr,
    arms: &[MatchArm],
    env: &mut BTreeMap<String, Expr>,
    context: &EvalContext,
    stack: &mut BTreeSet<String>,
    last_expr: &mut Option<Expr>,
) -> Result<ExecOutcome> {
    let normalized_expr = normalize_expr(expr, env, context, stack)?;
    for arm in arms {
        if pattern_matches(&normalized_expr, &arm.pattern)? {
            return execute_match_arm_body(&arm.body, env, context, stack, last_expr);
        }
    }

    let mut snapshots = Vec::new();
    for arm in arms {
        snapshots.push(execute_match_arm_snapshot(
            &arm.body,
            env.clone(),
            context,
            stack,
            last_expr.clone(),
        )?);
    }
    if let Some(first) = snapshots.first() {
        if snapshots.iter().all(|item| item == first) {
            *env = first.env.clone();
            *last_expr = first.last_expr.clone();
            return Ok(first.outcome.clone());
        }
    }

    bail!("符号化 match 产生了发散的辅助函数状态")
}

fn execute_block_in_place(
    stmts: &[Stmt],
    env: &mut BTreeMap<String, Expr>,
    context: &EvalContext,
    stack: &mut BTreeSet<String>,
    last_expr: &mut Option<Expr>,
) -> Result<ExecOutcome> {
    for stmt in stmts {
        match execute_stmt(stmt, env, context, stack, last_expr)? {
            ExecOutcome::Continue => {}
            outcome @ ExecOutcome::Return(_) => return Ok(outcome),
        }
    }
    Ok(ExecOutcome::Continue)
}

#[derive(Debug, Clone, PartialEq)]
struct BlockSnapshot {
    env: BTreeMap<String, Expr>,
    outcome: ExecOutcome,
    last_expr: Option<Expr>,
}

fn execute_block_snapshot(
    stmts: &[Stmt],
    mut env: BTreeMap<String, Expr>,
    context: &EvalContext,
    stack: &mut BTreeSet<String>,
    mut last_expr: Option<Expr>,
) -> Result<BlockSnapshot> {
    let outcome = execute_block_in_place(stmts, &mut env, context, stack, &mut last_expr)?;
    Ok(BlockSnapshot {
        env,
        outcome,
        last_expr,
    })
}

fn execute_match_arm_snapshot(
    body: &MatchArmBody,
    mut env: BTreeMap<String, Expr>,
    context: &EvalContext,
    stack: &mut BTreeSet<String>,
    mut last_expr: Option<Expr>,
) -> Result<BlockSnapshot> {
    let outcome = execute_match_arm_body(body, &mut env, context, stack, &mut last_expr)?;
    Ok(BlockSnapshot {
        env,
        outcome,
        last_expr,
    })
}

fn execute_match_arm_body(
    body: &MatchArmBody,
    env: &mut BTreeMap<String, Expr>,
    context: &EvalContext,
    stack: &mut BTreeSet<String>,
    last_expr: &mut Option<Expr>,
) -> Result<ExecOutcome> {
    match body {
        MatchArmBody::Statement(stmt) => execute_stmt(stmt, env, context, stack, last_expr),
        MatchArmBody::Expr(expr) => {
            *last_expr = Some(normalize_expr(expr, env, context, stack)?);
            Ok(ExecOutcome::Continue)
        }
    }
}

fn pattern_matches(expr: &Expr, pattern: &str) -> Result<bool> {
    if pattern == "_" {
        return Ok(true);
    }

    let pattern_expr = parse_expr(pattern).unwrap_or_else(|_| Expr::Raw(pattern.to_string()));
    let equality = fold_binary(expr.clone(), BinaryOp::Equal, pattern_expr)?;
    Ok(expr_bool(&equality) == Some(true))
}

fn expr_iterable_items(expr: &Expr) -> Option<Vec<Expr>> {
    match expr {
        Expr::List(items) => Some(items.clone()),
        Expr::Range { start, end } => {
            let start = expr_number(start)?.round() as i64;
            let end = expr_number(end)?.round() as i64;
            Some(
                (start..end)
                    .map(|value| Expr::Number(value as f64))
                    .collect(),
            )
        }
        _ => None,
    }
}

fn fold_unary(op: UnaryOp, expr: Expr) -> Result<Expr> {
    match (op.clone(), expr.clone()) {
        (UnaryOp::Negate, Expr::Number(value)) => Ok(Expr::Number(-value)),
        (UnaryOp::Not, Expr::Bool(value)) => Ok(Expr::Bool(!value)),
        _ => Ok(Expr::Unary {
            op,
            expr: Box::new(expr),
        }),
    }
}

fn fold_index(object: Expr, index: Expr) -> Result<Expr> {
    if let Some(position) = expr_integer(&index) {
        if let Expr::List(items) = &object {
            if let Some(item) = list_index(items, position) {
                return Ok(item.clone());
            }
        }
    }

    Ok(Expr::Index {
        object: Box::new(object),
        index: Box::new(index),
    })
}

fn fold_slice(object: Expr, start: Option<Box<Expr>>, end: Option<Box<Expr>>) -> Result<Expr> {
    if let Expr::List(items) = &object {
        let slice = list_slice(
            items,
            start.as_deref().and_then(expr_integer),
            end.as_deref().and_then(expr_integer),
        );
        if let Some(slice) = slice {
            return Ok(Expr::List(slice.to_vec()));
        }
    }

    Ok(Expr::Slice {
        object: Box::new(object),
        start,
        end,
    })
}

fn fold_binary(left: Expr, op: BinaryOp, right: Expr) -> Result<Expr> {
    if let (Some(lhs), Some(rhs)) = (expr_number(&left), expr_number(&right)) {
        return Ok(match op {
            BinaryOp::Add => Expr::Number(lhs + rhs),
            BinaryOp::Subtract => Expr::Number(lhs - rhs),
            BinaryOp::Multiply => Expr::Number(lhs * rhs),
            BinaryOp::Divide => Expr::Number(lhs / rhs),
            BinaryOp::Modulo => Expr::Number(lhs % rhs),
            BinaryOp::Greater => Expr::Bool(lhs > rhs),
            BinaryOp::GreaterEqual => Expr::Bool(lhs >= rhs),
            BinaryOp::Less => Expr::Bool(lhs < rhs),
            BinaryOp::LessEqual => Expr::Bool(lhs <= rhs),
            BinaryOp::Equal => Expr::Bool((lhs - rhs).abs() <= f64::EPSILON),
            BinaryOp::NotEqual => Expr::Bool((lhs - rhs).abs() > f64::EPSILON),
            _ => Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
        });
    }

    if let (Some(lhs), Some(rhs)) = (expr_bool(&left), expr_bool(&right)) {
        return Ok(match op {
            BinaryOp::And => Expr::Bool(lhs && rhs),
            BinaryOp::Or => Expr::Bool(lhs || rhs),
            BinaryOp::Equal => Expr::Bool(lhs == rhs),
            BinaryOp::NotEqual => Expr::Bool(lhs != rhs),
            _ => Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
        });
    }

    if let (Some(lhs), Some(rhs)) = (expr_string(&left), expr_string(&right)) {
        return Ok(match op {
            BinaryOp::Add => Expr::String(format!("{lhs}{rhs}")),
            BinaryOp::Equal => Expr::Bool(lhs == rhs),
            BinaryOp::NotEqual => Expr::Bool(lhs != rhs),
            _ => Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
        });
    }

    Ok(Expr::Binary {
        left: Box::new(left),
        op,
        right: Box::new(right),
    })
}

fn fold_builtin_call(callee: Expr, args: Vec<CallArg>) -> Result<Expr> {
    if let Expr::Member { object, field } = &callee {
        if let Some(value) = fold_member_builtin(object, field, &args)? {
            return Ok(value);
        }
    }

    if let Expr::Identifier(name) = &callee {
        if let Some(value) = fold_identifier_builtin(name, &args)? {
            return Ok(value);
        }
    }

    Ok(Expr::Call {
        callee: Box::new(callee),
        args,
    })
}

fn fold_member_builtin(object: &Expr, field: &str, args: &[CallArg]) -> Result<Option<Expr>> {
    if args.is_empty() {
        match field {
            "len" => {
                if let Some(length) = expr_length(object) {
                    return Ok(Some(Expr::Number(length as f64)));
                }
            }
            "sum" => {
                if let Some(sum) = expr_sum(object) {
                    return Ok(Some(Expr::Number(sum)));
                }
            }
            "mean" | "avg" => {
                if let Some(mean) = expr_mean(object) {
                    return Ok(Some(Expr::Number(mean)));
                }
            }
            "min" => {
                if let Some(value) = expr_min(object) {
                    return Ok(Some(Expr::Number(value)));
                }
            }
            "max" => {
                if let Some(value) = expr_max(object) {
                    return Ok(Some(Expr::Number(value)));
                }
            }
            "std" | "stddev" => {
                if let Some(value) = expr_stddev(object) {
                    return Ok(Some(Expr::Number(value)));
                }
            }
            "variance" => {
                if let Some(value) = expr_variance(object) {
                    return Ok(Some(Expr::Number(value)));
                }
            }
            "first" => {
                if let Some(value) = expr_first(object) {
                    return Ok(Some(value));
                }
            }
            "last" => {
                if let Some(value) = expr_last(object) {
                    return Ok(Some(value));
                }
            }
            "ok" => {
                if is_fetch_expr(object) {
                    return Ok(Some(Expr::Bool(true)));
                }
            }
            "retryable" => {
                if is_fetch_expr(object) {
                    return Ok(Some(Expr::Bool(false)));
                }
            }
            _ => {}
        }
    }

    Ok(None)
}

fn fold_identifier_builtin(name: &str, args: &[CallArg]) -> Result<Option<Expr>> {
    let positional = args.iter().map(|arg| &arg.value).collect::<Vec<_>>();
    let result = match (name, positional.as_slice()) {
        ("abs", [value]) => expr_number(value).map(|item| Expr::Number(item.abs())),
        ("sqrt", [value]) => expr_number(value).map(|item| Expr::Number(item.sqrt())),
        ("sum", [value]) => expr_sum(value).map(Expr::Number),
        ("mean" | "avg", [value]) => expr_mean(value).map(Expr::Number),
        ("min", [value]) => expr_min(value).map(Expr::Number),
        ("max", [value]) => expr_max(value).map(Expr::Number),
        ("std" | "stddev", [value]) => expr_stddev(value).map(Expr::Number),
        ("variance", [value]) => expr_variance(value).map(Expr::Number),
        ("first", [value]) => expr_first(value),
        ("last", [value]) => expr_last(value),
        ("pow", [left, right]) => match (expr_number(left), expr_number(right)) {
            (Some(base), Some(exp)) => Some(Expr::Number(base.powf(exp))),
            _ => None,
        },
        _ => None,
    };
    Ok(result)
}

fn is_fetch_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Call { callee, .. } => matches!(
            callee.as_ref(),
            Expr::Identifier(name) if name == "fetch" || name == "get_data"
        ),
        Expr::Try(inner) | Expr::Await(inner) => is_fetch_expr(inner),
        _ => false,
    }
}

fn expr_length(expr: &Expr) -> Option<usize> {
    match expr {
        Expr::List(items) => Some(items.len()),
        Expr::Call { args, .. } if is_fetch_expr(expr) => args
            .iter()
            .find(|arg| arg.name.as_deref() == Some("lookback"))
            .and_then(|arg| expr_number(&arg.value))
            .map(|value| value.max(1.0) as usize),
        Expr::Try(inner) | Expr::Await(inner) => expr_length(inner),
        _ => None,
    }
}

fn expr_sum(expr: &Expr) -> Option<f64> {
    expr_numbers(expr).map(|values| values.into_iter().sum())
}

fn expr_mean(expr: &Expr) -> Option<f64> {
    let values = expr_numbers(expr)?;
    if values.is_empty() {
        return None;
    }
    Some(values.iter().sum::<f64>() / values.len() as f64)
}

fn expr_min(expr: &Expr) -> Option<f64> {
    expr_numbers(expr)?.into_iter().reduce(f64::min)
}

fn expr_max(expr: &Expr) -> Option<f64> {
    expr_numbers(expr)?.into_iter().reduce(f64::max)
}

fn expr_variance(expr: &Expr) -> Option<f64> {
    let values = expr_numbers(expr)?;
    if values.is_empty() {
        return None;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    Some(
        values
            .iter()
            .map(|value| {
                let delta = *value - mean;
                delta * delta
            })
            .sum::<f64>()
            / values.len() as f64,
    )
}

fn expr_stddev(expr: &Expr) -> Option<f64> {
    expr_variance(expr).map(f64::sqrt)
}

fn expr_first(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::List(items) => items.first().cloned(),
        _ => None,
    }
}

fn expr_last(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::List(items) => items.last().cloned(),
        _ => None,
    }
}

fn expr_numbers(expr: &Expr) -> Option<Vec<f64>> {
    match expr {
        Expr::List(items) => items.iter().map(expr_number).collect::<Option<Vec<_>>>(),
        _ => None,
    }
}

fn expr_number(expr: &Expr) -> Option<f64> {
    match expr {
        Expr::Number(value) => Some(*value),
        Expr::Unary {
            op: UnaryOp::Negate,
            expr,
        } => expr_number(expr).map(|value| -value),
        _ => None,
    }
}

fn expr_integer(expr: &Expr) -> Option<isize> {
    expr_number(expr).map(|value| value.round() as isize)
}

fn expr_bool(expr: &Expr) -> Option<bool> {
    match expr {
        Expr::Bool(value) => Some(*value),
        _ => None,
    }
}

fn expr_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::String(value) => Some(value.clone()),
        Expr::Identifier(value) => Some(value.clone()),
        _ => None,
    }
}

fn list_index(items: &[Expr], index: isize) -> Option<&Expr> {
    let len = items.len() as isize;
    let normalized = if index < 0 { len + index } else { index };
    if normalized < 0 || normalized >= len {
        return None;
    }
    items.get(normalized as usize)
}

fn list_slice(items: &[Expr], start: Option<isize>, end: Option<isize>) -> Option<&[Expr]> {
    let len = items.len() as isize;
    let normalized_start = normalize_slice_bound(start.unwrap_or(0), len)?;
    let normalized_end = normalize_slice_bound(end.unwrap_or(len), len)?;
    if normalized_start > normalized_end {
        return Some(&items[0..0]);
    }
    items.get(normalized_start..normalized_end)
}

fn normalize_slice_bound(bound: isize, len: isize) -> Option<usize> {
    let normalized = if bound < 0 { len + bound } else { bound };
    let clamped = normalized.clamp(0, len);
    usize::try_from(clamped).ok()
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
        let Some(score) = expr_number(value) else {
            panic!("expected numeric score");
        };
        assert!(score.is_finite());
        assert!(score > 1.0);
    }
}
