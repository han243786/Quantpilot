use crate::diagnostics::{Diagnostic, Span};
use crate::hir::{HirStmt, TypedHirModule};
use crate::resolve::ResolveResult;
use crate::script::{Expr, FunctionDecl, Item, MatchArmBody, ScriptModule, Stmt};
use crate::types::Type;

pub(super) fn collect_unsupported_construct_diagnostics(
    module: &ScriptModule,
    resolved: &ResolveResult,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for item in &module.items {
        match item {
            Item::Import(import_decl) => {
                if import_decl.names.is_none() && import_decl.module.contains(" as ") {
                    diagnostics.push(Diagnostic::error(
                        "QS0608",
                        "QuantScript 不支持简单的 `import foo as bar`；请使用 `from module import name as alias`",
                        Some(Span::module(import_decl.module.clone())),
                    ));
                }
            }
            Item::Function(function) => {
                if function.is_async {
                    diagnostics.push(Diagnostic::error(
                        "QS0601",
                        "QuantScript 不支持strategy() 函数体中的异步函数",
                        Some(Span::function(function.name.clone())),
                    ));
                }
                collect_unsupported_constructs_from_stmts(
                    &function.body,
                    &function.name,
                    &mut diagnostics,
                );
                if function_contains_direct_recursion(function, &function.name) {
                    diagnostics.push(Diagnostic::error(
                        "QS0605",
                        "QuantScript 不支持strategy() 函数体中的递归辅助调用",
                        Some(Span::function(function.name.clone())),
                    ));
                }
            }
            Item::TestBlock(_) => {}
        }
    }
    diagnostics.extend(collect_non_universe_for_loop_diagnostics(
        &resolved.module,
        &resolved.types,
    ));
    diagnostics
}

fn collect_unsupported_constructs_from_stmts(
    stmts: &[Stmt],
    function_name: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            Stmt::Let { value, .. } | Stmt::Expr(value) | Stmt::Return(Some(value)) => {
                collect_unsupported_constructs_from_expr(value, diagnostics);
            }
            Stmt::Return(None) => {}
            Stmt::EmitIntent { args } => {
                for arg in args {
                    collect_unsupported_constructs_from_expr(&arg.value, diagnostics);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                collect_unsupported_constructs_from_expr(condition, diagnostics);
                collect_unsupported_constructs_from_stmts(then_branch, function_name, diagnostics);
                for (branch_condition, branch) in else_if_branches {
                    collect_unsupported_constructs_from_expr(branch_condition, diagnostics);
                    collect_unsupported_constructs_from_stmts(branch, function_name, diagnostics);
                }
                if let Some(branch) = else_branch {
                    collect_unsupported_constructs_from_stmts(branch, function_name, diagnostics);
                }
            }
            Stmt::For { iterable, body, .. } => {
                collect_unsupported_constructs_from_expr(iterable, diagnostics);
                collect_unsupported_constructs_from_stmts(body, function_name, diagnostics);
            }
            Stmt::While { condition, body } => {
                diagnostics.push(Diagnostic::error(
                    "QS0603",
                    "QuantScript 不支持 strategy() 函数体中的 while 循环。请改用 for ... in ... 或在数据源上使用窗口聚合",
                    Some(Span::function(function_name.to_string())),
                ));
                collect_unsupported_constructs_from_expr(condition, diagnostics);
                collect_unsupported_constructs_from_stmts(body, function_name, diagnostics);
            }
            Stmt::Match { expr, arms } => {
                diagnostics.push(Diagnostic::error(
                    "QS0604",
                    "QuantScript 不支持strategy() 函数体中的 match 语句",
                    Some(Span::function(function_name.to_string())),
                ));
                collect_unsupported_constructs_from_expr(expr, diagnostics);
                for arm in arms {
                    match &arm.body {
                        MatchArmBody::Statement(stmt) => collect_unsupported_constructs_from_stmts(
                            std::slice::from_ref(stmt.as_ref()),
                            function_name,
                            diagnostics,
                        ),
                        MatchArmBody::Expr(expr) => {
                            collect_unsupported_constructs_from_expr(expr, diagnostics);
                        }
                    }
                }
            }
        }
    }
}

fn collect_unsupported_constructs_from_expr(expr: &Expr, diagnostics: &mut Vec<Diagnostic>) {
    match expr {
        Expr::Await(inner) => {
            diagnostics.push(Diagnostic::error(
                "QS0602",
                "QuantScript 不支持strategy() 函数体中的 await 表达式",
                Some(Span::expr("await")),
            ));
            collect_unsupported_constructs_from_expr(inner, diagnostics);
        }
        Expr::Try(inner) => {
            if !is_supported_formal_try_target(inner) {
                diagnostics.push(Diagnostic::error(
                    "QS0607",
                    "QuantScript 在strategy() 函数体中仅支持对 fetch 类数据源表达式使用后缀 `?`",
                    Some(Span::expr("?")),
                ));
            }
            collect_unsupported_constructs_from_expr(inner, diagnostics);
        }
        Expr::List(items) => {
            for item in items {
                collect_unsupported_constructs_from_expr(item, diagnostics);
            }
        }
        Expr::Call { callee, args } => {
            if let Expr::Member { field, .. } = callee.as_ref() {
                if field == "push" {
                    diagnostics.push(Diagnostic::error(
                        "QS0609",
                        "QuantScript 不支持strategy() 函数体中使用 `.push(...)` 构建可变列表",
                        Some(Span::expr(".push")),
                    ));
                } else if matches!(field.as_str(), "ok" | "retryable") {
                    diagnostics.push(Diagnostic::error(
                        "QS0610",
                        "QuantScript 不支持strategy() 函数体中的 `.ok()` / `.retryable()` 辅助方法",
                        Some(Span::expr(field.clone())),
                    ));
                }
            }
            collect_unsupported_constructs_from_expr(callee, diagnostics);
            for arg in args {
                collect_unsupported_constructs_from_expr(&arg.value, diagnostics);
            }
        }
        Expr::Member { object, .. } | Expr::Unary { expr: object, .. } => {
            collect_unsupported_constructs_from_expr(object, diagnostics);
        }
        Expr::Index { object, index } => {
            collect_unsupported_constructs_from_expr(object, diagnostics);
            collect_unsupported_constructs_from_expr(index, diagnostics);
        }
        Expr::Slice { object, start, end } => {
            collect_unsupported_constructs_from_expr(object, diagnostics);
            if let Some(start) = start {
                collect_unsupported_constructs_from_expr(start, diagnostics);
            }
            if let Some(end) = end {
                collect_unsupported_constructs_from_expr(end, diagnostics);
            }
        }
        Expr::Binary { left, right, .. }
        | Expr::Range {
            start: left,
            end: right,
        } => {
            collect_unsupported_constructs_from_expr(left, diagnostics);
            collect_unsupported_constructs_from_expr(right, diagnostics);
        }
        Expr::Raw(_) | Expr::Identifier(_) | Expr::Number(_) | Expr::String(_) | Expr::Bool(_) => {}
    }
}

fn is_supported_formal_try_target(expr: &Expr) -> bool {
    match expr {
        Expr::Call { callee, .. } => {
            matches!(callee.as_ref(), Expr::Identifier(name) if matches!(name.as_str(), "fetch" | "get_data"))
        }
        Expr::Try(inner) | Expr::Await(inner) => is_supported_formal_try_target(inner),
        _ => false,
    }
}

fn function_contains_direct_recursion(function: &FunctionDecl, function_name: &str) -> bool {
    stmts_contain_direct_recursion(&function.body, function_name)
}

fn stmts_contain_direct_recursion(stmts: &[Stmt], function_name: &str) -> bool {
    for stmt in stmts {
        match stmt {
            Stmt::Let { value, .. } | Stmt::Expr(value) | Stmt::Return(Some(value)) => {
                if expr_contains_direct_recursion(value, function_name) {
                    return true;
                }
            }
            Stmt::Return(None) => {}
            Stmt::EmitIntent { args } => {
                if args
                    .iter()
                    .any(|arg| expr_contains_direct_recursion(&arg.value, function_name))
                {
                    return true;
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                if expr_contains_direct_recursion(condition, function_name)
                    || stmts_contain_direct_recursion(then_branch, function_name)
                    || else_if_branches.iter().any(|(branch_condition, branch)| {
                        expr_contains_direct_recursion(branch_condition, function_name)
                            || stmts_contain_direct_recursion(branch, function_name)
                    })
                    || else_branch
                        .as_ref()
                        .is_some_and(|branch| stmts_contain_direct_recursion(branch, function_name))
                {
                    return true;
                }
            }
            Stmt::For { iterable, body, .. } => {
                if expr_contains_direct_recursion(iterable, function_name)
                    || stmts_contain_direct_recursion(body, function_name)
                {
                    return true;
                }
            }
            Stmt::While { condition, body } => {
                if expr_contains_direct_recursion(condition, function_name)
                    || stmts_contain_direct_recursion(body, function_name)
                {
                    return true;
                }
            }
            Stmt::Match { expr, arms } => {
                if expr_contains_direct_recursion(expr, function_name) {
                    return true;
                }
                for arm in arms {
                    match &arm.body {
                        MatchArmBody::Statement(stmt) => {
                            if stmts_contain_direct_recursion(
                                std::slice::from_ref(stmt.as_ref()),
                                function_name,
                            ) {
                                return true;
                            }
                        }
                        MatchArmBody::Expr(expr) => {
                            if expr_contains_direct_recursion(expr, function_name) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

fn expr_contains_direct_recursion(expr: &Expr, function_name: &str) -> bool {
    match expr {
        Expr::Call { callee, args } => {
            if matches!(callee.as_ref(), Expr::Identifier(name) if name == function_name) {
                return true;
            }
            expr_contains_direct_recursion(callee, function_name)
                || args
                    .iter()
                    .any(|arg| expr_contains_direct_recursion(&arg.value, function_name))
        }
        Expr::List(items) => items
            .iter()
            .any(|item| expr_contains_direct_recursion(item, function_name)),
        Expr::Member { object, .. } | Expr::Try(object) | Expr::Await(object) => {
            expr_contains_direct_recursion(object, function_name)
        }
        Expr::Unary { expr: object, .. } => expr_contains_direct_recursion(object, function_name),
        Expr::Index { object, index } => {
            expr_contains_direct_recursion(object, function_name)
                || expr_contains_direct_recursion(index, function_name)
        }
        Expr::Slice { object, start, end } => {
            expr_contains_direct_recursion(object, function_name)
                || start
                    .as_ref()
                    .is_some_and(|expr| expr_contains_direct_recursion(expr, function_name))
                || end
                    .as_ref()
                    .is_some_and(|expr| expr_contains_direct_recursion(expr, function_name))
        }
        Expr::Binary { left, right, .. }
        | Expr::Range {
            start: left,
            end: right,
        } => {
            expr_contains_direct_recursion(left, function_name)
                || expr_contains_direct_recursion(right, function_name)
        }
        Expr::Raw(_) | Expr::Identifier(_) | Expr::Number(_) | Expr::String(_) | Expr::Bool(_) => {
            false
        }
    }
}

fn collect_non_universe_for_loop_diagnostics(
    module: &TypedHirModule,
    types: &crate::types::TypeArena,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for function in &module.functions {
        collect_non_universe_for_loop_diagnostics_from_hir_stmts(
            &function.body,
            types,
            &mut diagnostics,
        );
    }
    diagnostics
}

fn collect_non_universe_for_loop_diagnostics_from_hir_stmts(
    stmts: &[HirStmt],
    types: &crate::types::TypeArena,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            HirStmt::For {
                iterable,
                body,
                span,
                ..
            } => {
                if !matches!(types.get(iterable.ty), Type::Universe) {
                    diagnostics.push(Diagnostic::error(
                        "QS0606",
                        "QuantScript 在strategy() 函数体中仅支持对 Universe 的 for 循环",
                        Some(span.clone()),
                    ));
                }
                collect_non_universe_for_loop_diagnostics_from_hir_stmts(body, types, diagnostics);
            }
            HirStmt::If {
                then_branch,
                else_if_branches,
                else_branch,
                ..
            } => {
                collect_non_universe_for_loop_diagnostics_from_hir_stmts(
                    then_branch,
                    types,
                    diagnostics,
                );
                for (_, branch) in else_if_branches {
                    collect_non_universe_for_loop_diagnostics_from_hir_stmts(
                        branch,
                        types,
                        diagnostics,
                    );
                }
                if let Some(branch) = else_branch {
                    collect_non_universe_for_loop_diagnostics_from_hir_stmts(
                        branch,
                        types,
                        diagnostics,
                    );
                }
            }
            HirStmt::While { body, .. } => {
                collect_non_universe_for_loop_diagnostics_from_hir_stmts(body, types, diagnostics);
            }
            HirStmt::Match { arms, .. } => {
                for arm in arms {
                    match &arm.body {
                        crate::hir::HirMatchArmBody::Statement(stmt) => {
                            collect_non_universe_for_loop_diagnostics_from_hir_stmts(
                                std::slice::from_ref(stmt.as_ref()),
                                types,
                                diagnostics,
                            );
                        }
                        crate::hir::HirMatchArmBody::Expr(_) => {}
                    }
                }
            }
            HirStmt::Let(_)
            | HirStmt::Return(_)
            | HirStmt::EmitIntent { .. }
            | HirStmt::Expr(_) => {}
        }
    }
}
