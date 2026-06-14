use super::contains_emit_in_stmts;
use crate::diagnostics::{Diagnostic, Span};
use crate::script::{Expr, Item, MatchArmBody, ScriptModule, Stmt};

pub(super) fn check_strategy_has_fetch(module: &ScriptModule) -> Vec<Diagnostic> {
    for item in &module.items {
        if let Item::Function(function) = item {
            if function.name == "strategy" {
                if !contains_fetch_like_call_in_stmts(&function.body) {
                    return vec![Diagnostic::error(
                        "QS0610",
                        "策略函数必须包含至少一个 fetch() 调用来获取市场数据",
                        Some(Span::function("strategy")),
                    )];
                }
                return vec![];
            }
        }
    }
    vec![]
}

fn contains_fetch_like_call_in_stmts(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Stmt::Let { value, .. } | Stmt::Expr(value) | Stmt::Return(Some(value)) => {
            contains_fetch_like_call_in_expr(value)
        }
        Stmt::Return(None) => false,
        Stmt::EmitIntent { args } => args
            .iter()
            .any(|arg| contains_fetch_like_call_in_expr(&arg.value)),
        Stmt::If {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
        } => {
            contains_fetch_like_call_in_expr(condition)
                || contains_fetch_like_call_in_stmts(then_branch)
                || else_if_branches.iter().any(|(c, b)| {
                    contains_fetch_like_call_in_expr(c) || contains_fetch_like_call_in_stmts(b)
                })
                || else_branch
                    .as_ref()
                    .is_some_and(|b| contains_fetch_like_call_in_stmts(b))
        }
        Stmt::For { iterable, body, .. } => {
            contains_fetch_like_call_in_expr(iterable) || contains_fetch_like_call_in_stmts(body)
        }
        Stmt::While { condition, body } => {
            contains_fetch_like_call_in_expr(condition) || contains_fetch_like_call_in_stmts(body)
        }
        Stmt::Match { expr, arms } => {
            contains_fetch_like_call_in_expr(expr)
                || arms.iter().any(|arm| match &arm.body {
                    MatchArmBody::Statement(stmt) => {
                        contains_fetch_like_call_in_stmts(std::slice::from_ref(stmt.as_ref()))
                    }
                    MatchArmBody::Expr(expr) => contains_fetch_like_call_in_expr(expr),
                })
        }
    })
}

fn contains_fetch_like_call_in_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Call { callee, args } => {
            if matches!(
                callee.as_ref(),
                Expr::Identifier(name) if name == "fetch" || name == "get_data"
            ) {
                return true;
            }
            contains_fetch_like_call_in_expr(callee)
                || args
                    .iter()
                    .any(|arg| contains_fetch_like_call_in_expr(&arg.value))
        }
        Expr::List(items) => items.iter().any(contains_fetch_like_call_in_expr),
        Expr::Member { object, .. }
        | Expr::Await(object)
        | Expr::Try(object)
        | Expr::Unary { expr: object, .. } => contains_fetch_like_call_in_expr(object),
        Expr::Index { object, index } => {
            contains_fetch_like_call_in_expr(object) || contains_fetch_like_call_in_expr(index)
        }
        Expr::Slice { object, start, end } => {
            contains_fetch_like_call_in_expr(object)
                || start
                    .as_ref()
                    .is_some_and(|s| contains_fetch_like_call_in_expr(s))
                || end
                    .as_ref()
                    .is_some_and(|e| contains_fetch_like_call_in_expr(e))
        }
        Expr::Binary { left, right, .. }
        | Expr::Range {
            start: left,
            end: right,
        } => contains_fetch_like_call_in_expr(left) || contains_fetch_like_call_in_expr(right),
        Expr::Raw(_) | Expr::Identifier(_) | Expr::Number(_) | Expr::String(_) | Expr::Bool(_) => {
            false
        }
    }
}

pub(super) fn check_strategy_has_emit(module: &ScriptModule) -> Vec<Diagnostic> {
    for item in &module.items {
        if let Item::Function(function) = item {
            if function.name == "strategy" {
                if !contains_emit_in_stmts(&function.body) {
                    return vec![Diagnostic::error(
                        "QS0611",
                        "策略函数必须包含至少一个 emit Intent() 调用来输出交易信号",
                        Some(Span::function("strategy")),
                    )];
                }
                return vec![];
            }
        }
    }
    vec![]
}
