use super::contains_emit_in_stmts;
use crate::diagnostics::{Diagnostic, Span};
use crate::script::{Expr, Item, MatchArmBody, ScriptModule, Stmt};

pub(super) fn check_dead_code_emit(module: &ScriptModule) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for item in &module.items {
        if let Item::Function(function) = item {
            check_dead_code_emit_from_stmts(&function.body, &mut diagnostics);
        }
    }
    diagnostics
}

fn check_dead_code_emit_from_stmts(stmts: &[Stmt], diagnostics: &mut Vec<Diagnostic>) {
    for stmt in stmts {
        match stmt {
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                if is_constant_false(condition) && contains_emit_in_stmts(then_branch) {
                    diagnostics.push(Diagnostic::warning(
                        "QS0612",
                        "条件为 false 的 if 分支中的 emit 语句永远不会执行",
                        Some(Span::expr("emit")),
                    ));
                }
                check_dead_code_emit_from_stmts(then_branch, diagnostics);
                for (_, branch) in else_if_branches {
                    check_dead_code_emit_from_stmts(branch, diagnostics);
                }
                if let Some(branch) = else_branch {
                    check_dead_code_emit_from_stmts(branch, diagnostics);
                }
            }
            Stmt::For { body, .. } => {
                check_dead_code_emit_from_stmts(body, diagnostics);
            }
            Stmt::While { body, .. } => {
                check_dead_code_emit_from_stmts(body, diagnostics);
            }
            Stmt::Match { expr, arms } => {
                for arm in arms {
                    match &arm.body {
                        MatchArmBody::Statement(stmt) => check_dead_code_emit_from_stmts(
                            std::slice::from_ref(stmt.as_ref()),
                            diagnostics,
                        ),
                        MatchArmBody::Expr(_) => {}
                    }
                }
                let _ = expr;
            }
            _ => {}
        }
    }
}

fn is_constant_false(expr: &Expr) -> bool {
    matches!(expr, Expr::Bool(false))
}
