use crate::diagnostics::{Diagnostic, DiagnosticSeverity, Span};
use crate::hir::{HirStmt, TypedHirModule};
use crate::resolve::{
    ResolveResult, ResolvedExprSemantic, ResolvedManualIndicatorFormula, ResolvedSeriesViewKind,
};
use crate::script::{CallArg, Expr, FunctionDecl, Item, MatchArmBody, ScriptModule, Stmt, UnaryOp};
use crate::types::Type;

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
                        "formal QuantScript does not support plain `import foo as bar`; use `from module import name as alias` instead",
                        Some(Span::module(import_decl.module.clone())),
                    ));
                }
            }
            Item::Function(function) => {
                if function.is_async {
                    diagnostics.push(Diagnostic::error(
                        "QS0601",
                        "formal QuantScript does not support async functions in the executable trunk",
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
                        "formal QuantScript does not support recursive helper calls in the executable trunk",
                        Some(Span::function(function.name.clone())),
                    ));
                }
            }
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
                    "formal QuantScript does not support while loops in the executable trunk",
                    Some(Span::function(function_name.to_string())),
                ));
                collect_unsupported_constructs_from_expr(condition, diagnostics);
                collect_unsupported_constructs_from_stmts(body, function_name, diagnostics);
            }
            Stmt::Match { expr, arms } => {
                diagnostics.push(Diagnostic::error(
                    "QS0604",
                    "formal QuantScript does not support match statements in the executable trunk",
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
                "formal QuantScript does not support await expressions in the executable trunk",
                Some(Span::expr("await")),
            ));
            collect_unsupported_constructs_from_expr(inner, diagnostics);
        }
        Expr::Try(inner) => {
            if !is_supported_formal_try_target(inner) {
                diagnostics.push(Diagnostic::error(
                    "QS0607",
                    "formal QuantScript only supports postfix `?` on fetch-like data-source expressions in the executable trunk",
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
                        "formal QuantScript does not support mutable list-building with `.push(...)` in the executable trunk",
                        Some(Span::expr(".push")),
                    ));
                } else if matches!(field.as_str(), "ok" | "retryable") {
                    diagnostics.push(Diagnostic::error(
                        "QS0610",
                        "formal QuantScript does not support `.ok()` / `.retryable()` helper conveniences in the executable trunk",
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
                        "formal QuantScript only supports for-loops over Universe in the executable trunk",
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
                    "look-ahead risk: negative series indices access future bars; use `series[0]` for the latest bar and positive lookbacks for history",
                    None,
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
                        "look-ahead risk: negative trailing-window spans imply future access; use `series[20..]` for a 20-bar history window",
                        None,
                    ));
                } else if start
                    .as_deref()
                    .and_then(expr_integer)
                    .is_some_and(|value| value == 0)
                {
                    diagnostics.push(Diagnostic::error(
                        "QS0403",
                        "trailing windows require a positive span; use `series[1..]` or larger history windows",
                        None,
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
            "look-ahead risk: `center=true` windows use future bars",
            None,
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
                    "warmup is insufficient: strategy needs at least {required_warmup_bars} bars, but fetch only requests {available_bars}"
                ),
                None,
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
