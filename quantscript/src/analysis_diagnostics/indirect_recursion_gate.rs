use crate::diagnostics::{Diagnostic, Span};
use crate::script::{Expr, FunctionDecl, Item, MatchArmBody, ScriptModule, Stmt};
use std::collections::{BTreeMap, BTreeSet};

// B1-11: 间接递归检测
pub(super) fn detect_indirect_recursion(module: &ScriptModule) -> Vec<Diagnostic> {
    let functions: Vec<&FunctionDecl> = module
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Function(f) => Some(f),
            _ => None,
        })
        .collect();

    if functions.is_empty() {
        return Vec::new();
    }

    let mut call_graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for f in &functions {
        let mut callees = BTreeSet::new();
        collect_callee_names_from_stmts(&f.body, &mut callees);
        call_graph.insert(f.name.clone(), callees);
    }

    let mut diagnostics = Vec::new();
    for start in call_graph.keys() {
        let mut visited = BTreeSet::new();
        let mut stack = vec![start.clone()];
        while let Some(current) = stack.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            if let Some(callees) = call_graph.get(&current) {
                for callee in callees {
                    if callee == start && start != &current {
                        diagnostics.push(Diagnostic::error(
                            "QS0605",
                            format!("检测到递归调用循环: {} 间接调用自身", start),
                            Some(Span::function(start.clone())),
                        ));
                    }
                    stack.push(callee.clone());
                }
            }
        }
    }
    diagnostics
}

fn collect_callee_names_from_stmts(stmts: &[Stmt], out: &mut BTreeSet<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Let { value, .. } | Stmt::Expr(value) | Stmt::Return(Some(value)) => {
                collect_callee_names_from_expr(value, out);
            }
            Stmt::Return(None) => {}
            Stmt::EmitIntent { args } => {
                for arg in args {
                    collect_callee_names_from_expr(&arg.value, out);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                collect_callee_names_from_expr(condition, out);
                collect_callee_names_from_stmts(then_branch, out);
                for (cond, branch) in else_if_branches {
                    collect_callee_names_from_expr(cond, out);
                    collect_callee_names_from_stmts(branch, out);
                }
                if let Some(branch) = else_branch {
                    collect_callee_names_from_stmts(branch, out);
                }
            }
            Stmt::For { iterable, body, .. } => {
                collect_callee_names_from_expr(iterable, out);
                collect_callee_names_from_stmts(body, out);
            }
            Stmt::While { condition, body } => {
                collect_callee_names_from_expr(condition, out);
                collect_callee_names_from_stmts(body, out);
            }
            Stmt::Match { expr, arms } => {
                collect_callee_names_from_expr(expr, out);
                for arm in arms {
                    match &arm.body {
                        MatchArmBody::Statement(stmt) => collect_callee_names_from_stmts(
                            std::slice::from_ref(stmt.as_ref()),
                            out,
                        ),
                        MatchArmBody::Expr(expr) => collect_callee_names_from_expr(expr, out),
                    }
                }
            }
        }
    }
}

fn collect_callee_names_from_expr(expr: &Expr, out: &mut BTreeSet<String>) {
    match expr {
        Expr::Call { callee, args } => {
            if let Expr::Identifier(name) = callee.as_ref() {
                out.insert(name.clone());
            }
            collect_callee_names_from_expr(callee, out);
            for arg in args {
                collect_callee_names_from_expr(&arg.value, out);
            }
        }
        Expr::List(items) => {
            for item in items {
                collect_callee_names_from_expr(item, out);
            }
        }
        Expr::Member { object, .. }
        | Expr::Await(object)
        | Expr::Try(object)
        | Expr::Unary { expr: object, .. } => {
            collect_callee_names_from_expr(object, out);
        }
        Expr::Index { object, index } => {
            collect_callee_names_from_expr(object, out);
            collect_callee_names_from_expr(index, out);
        }
        Expr::Slice { object, start, end } => {
            collect_callee_names_from_expr(object, out);
            if let Some(start) = start {
                collect_callee_names_from_expr(start, out);
            }
            if let Some(end) = end {
                collect_callee_names_from_expr(end, out);
            }
        }
        Expr::Binary { left, right, .. }
        | Expr::Range {
            start: left,
            end: right,
        } => {
            collect_callee_names_from_expr(left, out);
            collect_callee_names_from_expr(right, out);
        }
        Expr::Raw(_) | Expr::Identifier(_) | Expr::Number(_) | Expr::String(_) | Expr::Bool(_) => {}
    }
}
