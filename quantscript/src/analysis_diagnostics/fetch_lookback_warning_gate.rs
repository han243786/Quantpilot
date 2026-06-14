use super::arg_number_named;
use crate::diagnostics::{Diagnostic, Span};
use crate::script::{Expr, Item, MatchArmBody, ScriptModule, Stmt};

pub(super) fn collect_fetch_lookback_warnings(module: &ScriptModule) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for item in &module.items {
        if let Item::Function(function) = item {
            collect_fetch_lookback_warnings_from_stmts(&function.body, &mut diagnostics);
        }
    }
    diagnostics
}

fn collect_fetch_lookback_warnings_from_stmts(stmts: &[Stmt], diagnostics: &mut Vec<Diagnostic>) {
    for stmt in stmts {
        match stmt {
            Stmt::Let { value, .. } | Stmt::Expr(value) | Stmt::Return(Some(value)) => {
                collect_fetch_lookback_warnings_from_expr(value, diagnostics);
            }
            Stmt::Return(None) => {}
            Stmt::EmitIntent { args } => {
                for arg in args {
                    collect_fetch_lookback_warnings_from_expr(&arg.value, diagnostics);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                collect_fetch_lookback_warnings_from_expr(condition, diagnostics);
                collect_fetch_lookback_warnings_from_stmts(then_branch, diagnostics);
                for (cond, branch) in else_if_branches {
                    collect_fetch_lookback_warnings_from_expr(cond, diagnostics);
                    collect_fetch_lookback_warnings_from_stmts(branch, diagnostics);
                }
                if let Some(branch) = else_branch {
                    collect_fetch_lookback_warnings_from_stmts(branch, diagnostics);
                }
            }
            Stmt::For { iterable, body, .. } => {
                collect_fetch_lookback_warnings_from_expr(iterable, diagnostics);
                collect_fetch_lookback_warnings_from_stmts(body, diagnostics);
            }
            Stmt::While { condition, body } => {
                collect_fetch_lookback_warnings_from_expr(condition, diagnostics);
                collect_fetch_lookback_warnings_from_stmts(body, diagnostics);
            }
            Stmt::Match { expr, arms } => {
                collect_fetch_lookback_warnings_from_expr(expr, diagnostics);
                for arm in arms {
                    match &arm.body {
                        MatchArmBody::Statement(stmt) => {
                            collect_fetch_lookback_warnings_from_stmts(
                                std::slice::from_ref(stmt.as_ref()),
                                diagnostics,
                            );
                        }
                        MatchArmBody::Expr(expr) => {
                            collect_fetch_lookback_warnings_from_expr(expr, diagnostics);
                        }
                    }
                }
            }
        }
    }
}

fn collect_fetch_lookback_warnings_from_expr(expr: &Expr, diagnostics: &mut Vec<Diagnostic>) {
    if let Expr::Call { callee, args } = expr {
        let callee_name = match callee.as_ref() {
            Expr::Identifier(name) => name.as_str(),
            _ => return,
        };
        if matches!(callee_name, "fetch" | "get_data") {
            if let Some(value) = arg_number_named(args, "lookback") {
                if value < 1.0 {
                    diagnostics.push(Diagnostic::warning(
                        "QS0503",
                        format!("fetch lookback={} 灏忎簬 1, 宸茶嚜鍔ㄨ涓?1", value),
                        Some(Span::expr("fetch.lookback")),
                    ));
                }
            }
        }
    }

    match expr {
        Expr::List(items) => {
            for item in items {
                collect_fetch_lookback_warnings_from_expr(item, diagnostics);
            }
        }
        Expr::Call { callee, args } => {
            collect_fetch_lookback_warnings_from_expr(callee, diagnostics);
            for arg in args {
                collect_fetch_lookback_warnings_from_expr(&arg.value, diagnostics);
            }
        }
        Expr::Member { object, .. }
        | Expr::Await(object)
        | Expr::Try(object)
        | Expr::Unary { expr: object, .. } => {
            collect_fetch_lookback_warnings_from_expr(object, diagnostics);
        }
        Expr::Index { object, index } => {
            collect_fetch_lookback_warnings_from_expr(object, diagnostics);
            collect_fetch_lookback_warnings_from_expr(index, diagnostics);
        }
        Expr::Slice { object, start, end } => {
            collect_fetch_lookback_warnings_from_expr(object, diagnostics);
            if let Some(start) = start {
                collect_fetch_lookback_warnings_from_expr(start, diagnostics);
            }
            if let Some(end) = end {
                collect_fetch_lookback_warnings_from_expr(end, diagnostics);
            }
        }
        Expr::Binary { left, right, .. }
        | Expr::Range {
            start: left,
            end: right,
        } => {
            collect_fetch_lookback_warnings_from_expr(left, diagnostics);
            collect_fetch_lookback_warnings_from_expr(right, diagnostics);
        }
        Expr::Raw(_) | Expr::Identifier(_) | Expr::Number(_) | Expr::String(_) | Expr::Bool(_) => {}
    }
}
