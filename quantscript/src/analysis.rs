use crate::diagnostics::{Diagnostic, DiagnosticSeverity, Span};
use crate::hir::{HirStmt, TypedHirModule};
use crate::resolve::{
    ResolveResult, ResolvedExprSemantic, ResolvedManualIndicatorFormula, ResolvedSeriesViewKind,
};
use crate::script::{CallArg, Expr, FunctionDecl, Item, MatchArmBody, ScriptModule, Stmt, UnaryOp};
use crate::types::Type;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScriptAnalysis {
    pub required_warmup_bars: usize,
    pub diagnostics: Vec<Diagnostic>,
}

impl ScriptAnalysis {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }
}

pub fn analyze_script_module(module: &ScriptModule, resolved: &ResolveResult) -> ScriptAnalysis {
    let required_warmup_bars = infer_required_warmup_bars(resolved);
    let mut diagnostics = Vec::new();
    diagnostics.extend(collect_unsupported_construct_diagnostics(module, resolved));
    diagnostics.extend(collect_series_index_diagnostics(module));
    diagnostics.extend(collect_centered_window_diagnostics(module));
    diagnostics.extend(collect_warmup_diagnostics(module, required_warmup_bars));
    diagnostics.extend(check_strategy_has_fetch(module));
    diagnostics.extend(check_strategy_has_emit(module));
    diagnostics.extend(collect_fetch_lookback_warnings(module));
    diagnostics.extend(detect_indirect_recursion(module));
    let data_source_map = build_data_source_map(module);
    diagnostics.extend(check_all_index_bounds(module, &data_source_map));
    diagnostics.extend(check_dead_code_emit(module));
    diagnostics.extend(check_fetch_symbol_whitelist(module));

    ScriptAnalysis {
        required_warmup_bars,
        diagnostics,
    }
}

fn collect_unsupported_construct_diagnostics(
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

fn infer_required_warmup_bars(resolved: &ResolveResult) -> usize {
    resolved
        .expr_semantics
        .values()
        .filter_map(required_warmup_for_semantic)
        .max()
        .unwrap_or(0)
}

fn required_warmup_for_semantic(semantic: &ResolvedExprSemantic) -> Option<usize> {
    match semantic {
        ResolvedExprSemantic::SeriesView(view) => match view {
            ResolvedSeriesViewKind::Lookback(span) | ResolvedSeriesViewKind::Window(span) => {
                Some(*span)
            }
            ResolvedSeriesViewKind::Current | ResolvedSeriesViewKind::First => None,
        },
        ResolvedExprSemantic::WindowAggregateView(view) => Some(view.span),
        ResolvedExprSemantic::BoundaryLookbackPair { span } => Some(*span),
        ResolvedExprSemantic::BalancedSmoothedChangePair { period, .. } => Some(*period),
        ResolvedExprSemantic::ManualIndicatorFormula(formula) => match formula {
            ResolvedManualIndicatorFormula::Momentum { lookback } => Some(*lookback),
            ResolvedManualIndicatorFormula::MovingAverage { span } => Some(*span),
            ResolvedManualIndicatorFormula::MacdLine { slow_period, .. } => Some(*slow_period),
            ResolvedManualIndicatorFormula::MacdSignal {
                slow_period,
                signal_period,
                ..
            }
            | ResolvedManualIndicatorFormula::MacdHistogram {
                slow_period,
                signal_period,
                ..
            } => Some(*slow_period + *signal_period),
            ResolvedManualIndicatorFormula::ZScore { window } => Some(*window),
        },
        _ => None,
    }
}

fn collect_series_index_diagnostics(module: &ScriptModule) -> Vec<Diagnostic> {
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

fn collect_centered_window_diagnostics(module: &ScriptModule) -> Vec<Diagnostic> {
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

fn collect_warmup_diagnostics(
    module: &ScriptModule,
    required_warmup_bars: usize,
) -> Vec<Diagnostic> {
    if required_warmup_bars == 0 {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    for item in &module.items {
        let Item::Function(function) = item else {
            continue;
        };
        collect_warmup_from_function(function, required_warmup_bars, &mut diagnostics);
    }
    diagnostics
}

fn collect_warmup_from_function(
    function: &FunctionDecl,
    required_warmup_bars: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut fetch_lookbacks = Vec::new();
    collect_fetch_lookbacks_from_stmts(&function.body, &mut fetch_lookbacks);

    for available_bars in fetch_lookbacks {
        if available_bars < required_warmup_bars {
            diagnostics.push(Diagnostic::error(
                "QS0501",
                format!(
                    "预热不足: 策略至少需要 {required_warmup_bars} 根 K 线，但 fetch 仅请求了 {available_bars}"
                ),
                Some(Span::expr("fetch")),
            ));
        }
    }
}

fn collect_fetch_lookbacks_from_stmts(stmts: &[Stmt], out: &mut Vec<usize>) {
    for stmt in stmts {
        match stmt {
            Stmt::Let { value, .. } | Stmt::Expr(value) | Stmt::Return(Some(value)) => {
                collect_fetch_lookbacks_from_expr(value, out);
            }
            Stmt::Return(None) => {}
            Stmt::EmitIntent { args } => {
                for arg in args {
                    collect_fetch_lookbacks_from_expr(&arg.value, out);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                collect_fetch_lookbacks_from_expr(condition, out);
                collect_fetch_lookbacks_from_stmts(then_branch, out);
                for (branch_condition, branch) in else_if_branches {
                    collect_fetch_lookbacks_from_expr(branch_condition, out);
                    collect_fetch_lookbacks_from_stmts(branch, out);
                }
                if let Some(branch) = else_branch {
                    collect_fetch_lookbacks_from_stmts(branch, out);
                }
            }
            Stmt::For { iterable, body, .. } => {
                collect_fetch_lookbacks_from_expr(iterable, out);
                collect_fetch_lookbacks_from_stmts(body, out);
            }
            Stmt::While { condition, body } => {
                collect_fetch_lookbacks_from_expr(condition, out);
                collect_fetch_lookbacks_from_stmts(body, out);
            }
            Stmt::Match { expr, arms } => {
                collect_fetch_lookbacks_from_expr(expr, out);
                for arm in arms {
                    match &arm.body {
                        MatchArmBody::Statement(stmt) => {
                            collect_fetch_lookbacks_from_stmts(
                                std::slice::from_ref(stmt.as_ref()),
                                out,
                            );
                        }
                        MatchArmBody::Expr(expr) => collect_fetch_lookbacks_from_expr(expr, out),
                    }
                }
            }
        }
    }
}

fn collect_fetch_lookbacks_from_expr(expr: &Expr, out: &mut Vec<usize>) {
    if let Some(lookback) = fetch_lookback(expr) {
        out.push(lookback);
    }

    match expr {
        Expr::List(items) => {
            for item in items {
                collect_fetch_lookbacks_from_expr(item, out);
            }
        }
        Expr::Call { callee, args } => {
            collect_fetch_lookbacks_from_expr(callee, out);
            for arg in args {
                collect_fetch_lookbacks_from_expr(&arg.value, out);
            }
        }
        Expr::Member { object, .. }
        | Expr::Await(object)
        | Expr::Try(object)
        | Expr::Unary { expr: object, .. } => {
            collect_fetch_lookbacks_from_expr(object, out);
        }
        Expr::Index { object, index } => {
            collect_fetch_lookbacks_from_expr(object, out);
            collect_fetch_lookbacks_from_expr(index, out);
        }
        Expr::Slice { object, start, end } => {
            collect_fetch_lookbacks_from_expr(object, out);
            if let Some(start) = start {
                collect_fetch_lookbacks_from_expr(start, out);
            }
            if let Some(end) = end {
                collect_fetch_lookbacks_from_expr(end, out);
            }
        }
        Expr::Binary { left, right, .. }
        | Expr::Range {
            start: left,
            end: right,
        } => {
            collect_fetch_lookbacks_from_expr(left, out);
            collect_fetch_lookbacks_from_expr(right, out);
        }
        Expr::Raw(_) | Expr::Identifier(_) | Expr::Number(_) | Expr::String(_) | Expr::Bool(_) => {}
    }
}

fn fetch_lookback(expr: &Expr) -> Option<usize> {
    let Expr::Call { callee, args } = expr else {
        return None;
    };

    let callee_name = match callee.as_ref() {
        Expr::Identifier(name) => name.as_str(),
        _ => return None,
    };
    if !matches!(callee_name, "fetch" | "get_data") {
        return None;
    }

    arg_number_named(args, "lookback").map(|value| value.max(1.0) as usize)
}

fn arg_bool_named(args: &[CallArg], name: &str) -> Option<bool> {
    args.iter()
        .find(|arg| arg.name.as_deref() == Some(name))
        .and_then(|arg| match &arg.value {
            Expr::Bool(value) => Some(*value),
            _ => None,
        })
}

fn arg_number_named(args: &[CallArg], name: &str) -> Option<f64> {
    args.iter()
        .find(|arg| arg.name.as_deref() == Some(name))
        .and_then(|arg| match &arg.value {
            Expr::Number(value) => Some(*value),
            Expr::Unary { op, expr } => match (op, expr.as_ref()) {
                (crate::script::UnaryOp::Negate, Expr::Number(value)) => Some(-value),
                _ => None,
            },
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

// B1-1: 空 strategy 提前诊断
fn check_strategy_has_fetch(module: &ScriptModule) -> Vec<Diagnostic> {
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

// B1-5: 无 emit 诊断
fn check_strategy_has_emit(module: &ScriptModule) -> Vec<Diagnostic> {
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

fn contains_emit_in_stmts(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Stmt::EmitIntent { .. } => true,
        Stmt::If {
            then_branch,
            else_if_branches,
            else_branch,
            ..
        } => {
            contains_emit_in_stmts(then_branch)
                || else_if_branches
                    .iter()
                    .any(|(_, b)| contains_emit_in_stmts(b))
                || else_branch
                    .as_ref()
                    .is_some_and(|b| contains_emit_in_stmts(b))
        }
        Stmt::For { body, .. } => contains_emit_in_stmts(body),
        Stmt::While { body, .. } => contains_emit_in_stmts(body),
        Stmt::Match { arms, .. } => arms.iter().any(|arm| match &arm.body {
            MatchArmBody::Statement(stmt) => {
                contains_emit_in_stmts(std::slice::from_ref(stmt.as_ref()))
            }
            MatchArmBody::Expr(_) => false,
        }),
        _ => false,
    })
}

// B1-8: lookback=0 警告
fn collect_fetch_lookback_warnings(module: &ScriptModule) -> Vec<Diagnostic> {
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
                        format!("fetch lookback={} 小于 1, 已自动设为 1", value),
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

// B1-11: 间接递归检测
fn detect_indirect_recursion(module: &ScriptModule) -> Vec<Diagnostic> {
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

// B1-12: 数组越界索引警告
fn build_data_source_map(module: &ScriptModule) -> BTreeMap<String, usize> {
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

fn check_all_index_bounds(
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

// B1-14: dead code emit 警告
fn check_dead_code_emit(module: &ScriptModule) -> Vec<Diagnostic> {
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

// B1-15: 交易对白名单校验
const KNOWN_SYMBOLS: &[&str] = &[
    "BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT", "ADAUSDT", "DOGEUSDT", "XRPUSDT",
];

fn check_fetch_symbol_whitelist(module: &ScriptModule) -> Vec<Diagnostic> {
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
