use crate::diagnostics::{Diagnostic, Span};
use crate::script::{CallArg, Expr, Item, MatchArmBody, ScriptModule, Stmt, UnaryOp};

pub(super) fn collect_series_index_diagnostics(module: &ScriptModule) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for item in &module.items {
        if let Item::Function(function) = item {
            collect_series_index_from_stmts(&function.body, &mut diagnostics);
        }
    }
    diagnostics
}

fn collect_series_index_from_stmts(stmts: &[Stmt], diagnostics: &mut Vec<Diagnostic>) {
    for stmt in stmts {
        match stmt {
            Stmt::Let { value, .. } | Stmt::Expr(value) | Stmt::Return(Some(value)) => {
                collect_series_index_from_expr(value, diagnostics);
            }
            Stmt::Return(None) => {}
            Stmt::EmitIntent { args } => {
                for arg in args {
                    collect_series_index_from_expr(&arg.value, diagnostics);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                collect_series_index_from_expr(condition, diagnostics);
                collect_series_index_from_stmts(then_branch, diagnostics);
                for (branch_condition, branch) in else_if_branches {
                    collect_series_index_from_expr(branch_condition, diagnostics);
                    collect_series_index_from_stmts(branch, diagnostics);
                }
                if let Some(branch) = else_branch {
                    collect_series_index_from_stmts(branch, diagnostics);
                }
            }
            Stmt::For { iterable, body, .. } => {
                collect_series_index_from_expr(iterable, diagnostics);
                collect_series_index_from_stmts(body, diagnostics);
            }
            Stmt::While { condition, body } => {
                collect_series_index_from_expr(condition, diagnostics);
                collect_series_index_from_stmts(body, diagnostics);
            }
            Stmt::Match { expr, arms } => {
                collect_series_index_from_expr(expr, diagnostics);
                for arm in arms {
                    match &arm.body {
                        MatchArmBody::Statement(stmt) => {
                            collect_series_index_from_stmts(
                                std::slice::from_ref(stmt.as_ref()),
                                diagnostics,
                            );
                        }
                        MatchArmBody::Expr(expr) => {
                            collect_series_index_from_expr(expr, diagnostics);
                        }
                    }
                }
            }
        }
    }
}

fn collect_series_index_from_expr(expr: &Expr, diagnostics: &mut Vec<Diagnostic>) {
    match expr {
        Expr::Index { object, index } => {
            if expr_integer(index).is_some_and(|value| value < 0) {
                diagnostics.push(Diagnostic::error(
                    "QS0401",
                    "前视风险: 负数序列索引会访问未来 K 线；请使用 `series[0]` 获取最新 K 线，正数回溯获取历史",
                    Some(Span::expr("series[index]")),
                ));
            }
            collect_series_index_from_expr(object, diagnostics);
            collect_series_index_from_expr(index, diagnostics);
        }
        Expr::Slice { object, start, end } => {
            if end.is_none() {
                if start
                    .as_deref()
                    .and_then(expr_integer)
                    .is_some_and(|value| value < 0)
                {
                    diagnostics.push(Diagnostic::error(
                        "QS0401",
                        "前视风险: 负数 trailing-window 跨度意味着未来访问；请使用 `series[20..]` 获取 20 根 K 线的历史窗口",
                        Some(Span::expr("series[start..]")),
                    ));
                } else if start
                    .as_deref()
                    .and_then(expr_integer)
                    .is_some_and(|value| value == 0)
                {
                    diagnostics.push(Diagnostic::error(
                        "QS0403",
                        "trailing 窗口需要正数跨度；请使用 `series[1..]` 或更大的历史窗口",
                        Some(Span::expr("series[start..]")),
                    ));
                }
            }

            collect_series_index_from_expr(object, diagnostics);
            if let Some(start) = start {
                collect_series_index_from_expr(start, diagnostics);
            }
            if let Some(end) = end {
                collect_series_index_from_expr(end, diagnostics);
            }
        }
        Expr::List(items) => {
            for item in items {
                collect_series_index_from_expr(item, diagnostics);
            }
        }
        Expr::Call { callee, args } => {
            collect_series_index_from_expr(callee, diagnostics);
            for arg in args {
                collect_series_index_from_expr(&arg.value, diagnostics);
            }
        }
        Expr::Member { object, .. }
        | Expr::Await(object)
        | Expr::Try(object)
        | Expr::Unary { expr: object, .. } => {
            collect_series_index_from_expr(object, diagnostics);
        }
        Expr::Binary { left, right, .. }
        | Expr::Range {
            start: left,
            end: right,
        } => {
            collect_series_index_from_expr(left, diagnostics);
            collect_series_index_from_expr(right, diagnostics);
        }
        Expr::Raw(_) | Expr::Identifier(_) | Expr::Number(_) | Expr::String(_) | Expr::Bool(_) => {}
    }
}

pub(super) fn collect_centered_window_diagnostics(module: &ScriptModule) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for item in &module.items {
        if let Item::Function(function) = item {
            collect_centered_window_from_stmts(&function.body, &mut diagnostics);
        }
    }
    diagnostics
}

fn collect_centered_window_from_stmts(stmts: &[Stmt], diagnostics: &mut Vec<Diagnostic>) {
    for stmt in stmts {
        match stmt {
            Stmt::Let { value, .. } | Stmt::Expr(value) | Stmt::Return(Some(value)) => {
                collect_centered_window_from_expr(value, diagnostics);
            }
            Stmt::Return(None) => {}
            Stmt::EmitIntent { args } => {
                for arg in args {
                    collect_centered_window_from_expr(&arg.value, diagnostics);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                collect_centered_window_from_expr(condition, diagnostics);
                collect_centered_window_from_stmts(then_branch, diagnostics);
                for (branch_condition, branch) in else_if_branches {
                    collect_centered_window_from_expr(branch_condition, diagnostics);
                    collect_centered_window_from_stmts(branch, diagnostics);
                }
                if let Some(branch) = else_branch {
                    collect_centered_window_from_stmts(branch, diagnostics);
                }
            }
            Stmt::For { iterable, body, .. } => {
                collect_centered_window_from_expr(iterable, diagnostics);
                collect_centered_window_from_stmts(body, diagnostics);
            }
            Stmt::While { condition, body } => {
                collect_centered_window_from_expr(condition, diagnostics);
                collect_centered_window_from_stmts(body, diagnostics);
            }
            Stmt::Match { expr, arms } => {
                collect_centered_window_from_expr(expr, diagnostics);
                for arm in arms {
                    match &arm.body {
                        MatchArmBody::Statement(stmt) => {
                            collect_centered_window_from_stmts(
                                std::slice::from_ref(stmt.as_ref()),
                                diagnostics,
                            );
                        }
                        MatchArmBody::Expr(expr) => {
                            collect_centered_window_from_expr(expr, diagnostics);
                        }
                    }
                }
            }
        }
    }
}

fn collect_centered_window_from_expr(expr: &Expr, diagnostics: &mut Vec<Diagnostic>) {
    if centered_window_uses_future_bars(expr) {
        diagnostics.push(Diagnostic::error(
            "QS0402",
            "前视风险: `center=true` 窗口使用了未来 K 线",
            Some(Span::expr("center=true")),
        ));
    }

    match expr {
        Expr::List(items) => {
            for item in items {
                collect_centered_window_from_expr(item, diagnostics);
            }
        }
        Expr::Call { callee, args } => {
            collect_centered_window_from_expr(callee, diagnostics);
            for arg in args {
                collect_centered_window_from_expr(&arg.value, diagnostics);
            }
        }
        Expr::Member { object, .. }
        | Expr::Await(object)
        | Expr::Try(object)
        | Expr::Unary { expr: object, .. } => {
            collect_centered_window_from_expr(object, diagnostics);
        }
        Expr::Index { object, index } => {
            collect_centered_window_from_expr(object, diagnostics);
            collect_centered_window_from_expr(index, diagnostics);
        }
        Expr::Slice { object, start, end } => {
            collect_centered_window_from_expr(object, diagnostics);
            if let Some(start) = start {
                collect_centered_window_from_expr(start, diagnostics);
            }
            if let Some(end) = end {
                collect_centered_window_from_expr(end, diagnostics);
            }
        }
        Expr::Binary { left, right, .. }
        | Expr::Range {
            start: left,
            end: right,
        } => {
            collect_centered_window_from_expr(left, diagnostics);
            collect_centered_window_from_expr(right, diagnostics);
        }
        Expr::Raw(_) | Expr::Identifier(_) | Expr::Number(_) | Expr::String(_) | Expr::Bool(_) => {}
    }
}

fn centered_window_uses_future_bars(expr: &Expr) -> bool {
    let Expr::Call { callee, args } = expr else {
        return false;
    };

    let call_name = match callee.as_ref() {
        Expr::Identifier(name) => name.as_str(),
        Expr::Member { field, .. } => field.as_str(),
        _ => return false,
    };

    if !matches!(
        call_name,
        "rolling_mean" | "rolling_sum" | "rolling_std" | "rolling_stddev"
    ) {
        return false;
    }

    arg_bool_named(args, "center").unwrap_or(false)
}

fn arg_bool_named(args: &[CallArg], name: &str) -> Option<bool> {
    args.iter()
        .find(|arg| arg.name.as_deref() == Some(name))
        .and_then(|arg| match &arg.value {
            Expr::Bool(value) => Some(*value),
            _ => None,
        })
}

fn expr_integer(expr: &Expr) -> Option<isize> {
    match expr {
        Expr::Number(value) => Some(value.round() as isize),
        Expr::Unary {
            op: UnaryOp::Negate,
            expr,
        } => expr_integer(expr).map(|value| -value),
        _ => None,
    }
}
