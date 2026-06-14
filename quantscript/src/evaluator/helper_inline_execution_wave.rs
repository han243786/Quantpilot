use super::folding_value_wave::{expr_bool, expr_iterable_items, fold_binary};
use super::{normalize_expr, EvalContext};
use crate::script::{
    parse_expr, BinaryOp, CallArg, Expr, FunctionDecl, MatchArm, MatchArmBody, Stmt,
};
use anyhow::{anyhow, bail, Result};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq)]
enum ExecOutcome {
    Continue,
    Return(Expr),
}

pub(super) fn maybe_inline_function(
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
    message.contains("发散的辅助函数状态")
        || message.contains("while 循环")
        || message.contains("无法以符号方式展开")
        || message.contains("辅助函数中不支持的语句")
}

const MAX_RECURSION_DEPTH: usize = 256;
const MAX_FOR_ITERATIONS: usize = 10000;

fn evaluate_function(
    function: &FunctionDecl,
    args: &[CallArg],
    context: &EvalContext,
    stack: &mut BTreeSet<String>,
) -> Result<Expr> {
    if stack.len() > MAX_RECURSION_DEPTH {
        bail!("QS 函数调用嵌套深度超过限制 ({})", MAX_RECURSION_DEPTH);
    }
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
    if items.len() > MAX_FOR_ITERATIONS {
        bail!(
            "for 循环迭代次数超过限制 ({} > {})",
            items.len(),
            MAX_FOR_ITERATIONS
        );
    }
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
