use crate::diagnostics::{Diagnostic, Span};
use crate::script::{Expr, Item, MatchArmBody, ScriptModule, Stmt};

const KNOWN_SYMBOLS: &[&str] = &[
    "BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT", "ADAUSDT", "DOGEUSDT", "XRPUSDT",
];

pub(super) fn check_fetch_symbol_whitelist(module: &ScriptModule) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for item in &module.items {
        if let Item::Function(function) = item {
            check_fetch_symbol_whitelist_from_stmts(&function.body, &mut diagnostics);
        }
    }
    diagnostics
}

fn check_fetch_symbol_whitelist_from_stmts(stmts: &[Stmt], diagnostics: &mut Vec<Diagnostic>) {
    for stmt in stmts {
        match stmt {
            Stmt::Let { value, .. } | Stmt::Expr(value) | Stmt::Return(Some(value)) => {
                check_fetch_symbol_whitelist_from_expr(value, diagnostics);
            }
            Stmt::Return(None) => {}
            Stmt::EmitIntent { args } => {
                for arg in args {
                    check_fetch_symbol_whitelist_from_expr(&arg.value, diagnostics);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                check_fetch_symbol_whitelist_from_expr(condition, diagnostics);
                check_fetch_symbol_whitelist_from_stmts(then_branch, diagnostics);
                for (cond, branch) in else_if_branches {
                    check_fetch_symbol_whitelist_from_expr(cond, diagnostics);
                    check_fetch_symbol_whitelist_from_stmts(branch, diagnostics);
                }
                if let Some(branch) = else_branch {
                    check_fetch_symbol_whitelist_from_stmts(branch, diagnostics);
                }
            }
            Stmt::For { iterable, body, .. } => {
                check_fetch_symbol_whitelist_from_expr(iterable, diagnostics);
                check_fetch_symbol_whitelist_from_stmts(body, diagnostics);
            }
            Stmt::While { condition, body } => {
                check_fetch_symbol_whitelist_from_expr(condition, diagnostics);
                check_fetch_symbol_whitelist_from_stmts(body, diagnostics);
            }
            Stmt::Match { expr, arms } => {
                check_fetch_symbol_whitelist_from_expr(expr, diagnostics);
                for arm in arms {
                    match &arm.body {
                        MatchArmBody::Statement(stmt) => {
                            check_fetch_symbol_whitelist_from_stmts(
                                std::slice::from_ref(stmt.as_ref()),
                                diagnostics,
                            );
                        }
                        MatchArmBody::Expr(expr) => {
                            check_fetch_symbol_whitelist_from_expr(expr, diagnostics);
                        }
                    }
                }
            }
        }
    }
}

fn check_fetch_symbol_whitelist_from_expr(expr: &Expr, diagnostics: &mut Vec<Diagnostic>) {
    if let Expr::Call { callee, args } = expr {
        let callee_name = match callee.as_ref() {
            Expr::Identifier(name) => name.as_str(),
            _ => return,
        };
        if matches!(callee_name, "fetch" | "get_data") {
            if let Some(symbol_str) = args.first().and_then(|arg| match &arg.value {
                Expr::String(s) => Some(s.as_str()),
                _ => None,
            }) {
                if !KNOWN_SYMBOLS.contains(&symbol_str.to_uppercase().as_str()) {
                    diagnostics.push(Diagnostic::warning(
                        "QS0505",
                        format!(
                            "未知交易对 '{}' 不在已验证列表中。已验证: BTCUSDT, ETHUSDT, SOLUSDT",
                            symbol_str
                        ),
                        Some(Span::expr("instrument")),
                    ));
                }
            }
        }
    }

    match expr {
        Expr::List(items) => {
            for item in items {
                check_fetch_symbol_whitelist_from_expr(item, diagnostics);
            }
        }
        Expr::Call { callee, args } => {
            check_fetch_symbol_whitelist_from_expr(callee, diagnostics);
            for arg in args {
                check_fetch_symbol_whitelist_from_expr(&arg.value, diagnostics);
            }
        }
        Expr::Member { object, .. }
        | Expr::Await(object)
        | Expr::Try(object)
        | Expr::Unary { expr: object, .. } => {
            check_fetch_symbol_whitelist_from_expr(object, diagnostics);
        }
        Expr::Index { object, index } => {
            check_fetch_symbol_whitelist_from_expr(object, diagnostics);
            check_fetch_symbol_whitelist_from_expr(index, diagnostics);
        }
        Expr::Slice { object, start, end } => {
            check_fetch_symbol_whitelist_from_expr(object, diagnostics);
            if let Some(start) = start {
                check_fetch_symbol_whitelist_from_expr(start, diagnostics);
            }
            if let Some(end) = end {
                check_fetch_symbol_whitelist_from_expr(end, diagnostics);
            }
        }
        Expr::Binary { left, right, .. }
        | Expr::Range {
            start: left,
            end: right,
        } => {
            check_fetch_symbol_whitelist_from_expr(left, diagnostics);
            check_fetch_symbol_whitelist_from_expr(right, diagnostics);
        }
        Expr::Raw(_) | Expr::Identifier(_) | Expr::Number(_) | Expr::String(_) | Expr::Bool(_) => {}
    }
}
