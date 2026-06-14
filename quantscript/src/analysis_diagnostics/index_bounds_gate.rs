use super::fetch_lookback;
use crate::diagnostics::{Diagnostic, Span};
use crate::script::{Expr, Item, MatchArmBody, ScriptModule, Stmt};
use std::collections::BTreeMap;

pub(super) fn build_data_source_map(module: &ScriptModule) -> BTreeMap<String, usize> {
    let mut map = BTreeMap::new();
    for item in &module.items {
        if let Item::Function(function) = item {
            for stmt in &function.body {
                if let Stmt::Let { pattern, value, .. } = stmt {
                    if let Some(lookback) = fetch_lookback(value) {
                        map.insert(pattern.clone(), lookback);
                    }
                }
            }
        }
    }
    map
}

pub(super) fn check_all_index_bounds(
    module: &ScriptModule,
    data_sources: &BTreeMap<String, usize>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for item in &module.items {
        if let Item::Function(function) = item {
            check_index_bounds_from_stmts(&function.body, data_sources, &mut diagnostics);
        }
    }
    diagnostics
}

fn check_index_bounds_from_stmts(
    stmts: &[Stmt],
    data_sources: &BTreeMap<String, usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            Stmt::Let { value, .. } | Stmt::Expr(value) | Stmt::Return(Some(value)) => {
                check_index_bounds_from_expr(value, data_sources, diagnostics);
            }
            Stmt::Return(None) => {}
            Stmt::EmitIntent { args } => {
                for arg in args {
                    check_index_bounds_from_expr(&arg.value, data_sources, diagnostics);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                check_index_bounds_from_expr(condition, data_sources, diagnostics);
                check_index_bounds_from_stmts(then_branch, data_sources, diagnostics);
                for (cond, branch) in else_if_branches {
                    check_index_bounds_from_expr(cond, data_sources, diagnostics);
                    check_index_bounds_from_stmts(branch, data_sources, diagnostics);
                }
                if let Some(branch) = else_branch {
                    check_index_bounds_from_stmts(branch, data_sources, diagnostics);
                }
            }
            Stmt::For { iterable, body, .. } => {
                check_index_bounds_from_expr(iterable, data_sources, diagnostics);
                check_index_bounds_from_stmts(body, data_sources, diagnostics);
            }
            Stmt::While { condition, body } => {
                check_index_bounds_from_expr(condition, data_sources, diagnostics);
                check_index_bounds_from_stmts(body, data_sources, diagnostics);
            }
            Stmt::Match { expr, arms } => {
                check_index_bounds_from_expr(expr, data_sources, diagnostics);
                for arm in arms {
                    match &arm.body {
                        MatchArmBody::Statement(stmt) => check_index_bounds_from_stmts(
                            std::slice::from_ref(stmt.as_ref()),
                            data_sources,
                            diagnostics,
                        ),
                        MatchArmBody::Expr(expr) => {
                            check_index_bounds_from_expr(expr, data_sources, diagnostics);
                        }
                    }
                }
            }
        }
    }
}

fn check_index_bounds_from_expr(
    expr: &Expr,
    data_sources: &BTreeMap<String, usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Expr::Index { object, index } = expr {
        if let (Expr::Identifier(var), Expr::Number(idx)) = (object.as_ref(), index.as_ref()) {
            if let Some(&lookback) = data_sources.get(var) {
                if *idx as usize >= lookback {
                    diagnostics.push(Diagnostic::warning(
                        "QS0404",
                        format!(
                            "索引 {} 可能超出变量 '{}' 的回看窗口 {}",
                            idx, var, lookback
                        ),
                        Some(Span::binding(var.clone())),
                    ));
                }
            }
        }
    }

    match expr {
        Expr::List(items) => {
            for item in items {
                check_index_bounds_from_expr(item, data_sources, diagnostics);
            }
        }
        Expr::Call { callee, args } => {
            check_index_bounds_from_expr(callee, data_sources, diagnostics);
            for arg in args {
                check_index_bounds_from_expr(&arg.value, data_sources, diagnostics);
            }
        }
        Expr::Member { object, .. }
        | Expr::Await(object)
        | Expr::Try(object)
        | Expr::Unary { expr: object, .. } => {
            check_index_bounds_from_expr(object, data_sources, diagnostics);
        }
        Expr::Index { object, index } => {
            check_index_bounds_from_expr(object, data_sources, diagnostics);
            check_index_bounds_from_expr(index, data_sources, diagnostics);
        }
        Expr::Slice { object, start, end } => {
            check_index_bounds_from_expr(object, data_sources, diagnostics);
            if let Some(start) = start {
                check_index_bounds_from_expr(start, data_sources, diagnostics);
            }
            if let Some(end) = end {
                check_index_bounds_from_expr(end, data_sources, diagnostics);
            }
        }
        Expr::Binary { left, right, .. }
        | Expr::Range {
            start: left,
            end: right,
        } => {
            check_index_bounds_from_expr(left, data_sources, diagnostics);
            check_index_bounds_from_expr(right, data_sources, diagnostics);
        }
        Expr::Raw(_) | Expr::Identifier(_) | Expr::Number(_) | Expr::String(_) | Expr::Bool(_) => {}
    }
}
