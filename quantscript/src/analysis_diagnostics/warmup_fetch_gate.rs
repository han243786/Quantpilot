use crate::diagnostics::{Diagnostic, Span};
use crate::resolve::{
    ResolveResult, ResolvedExprSemantic, ResolvedManualIndicatorFormula, ResolvedSeriesViewKind,
};
use crate::script::{CallArg, Expr, FunctionDecl, Item, MatchArmBody, ScriptModule, Stmt, UnaryOp};

pub(super) fn infer_required_warmup_bars(resolved: &ResolveResult) -> usize {
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

pub(super) fn collect_warmup_diagnostics(
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

pub(super) fn fetch_lookback(expr: &Expr) -> Option<usize> {
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

pub(in crate::analysis_diagnostics) fn arg_number_named(
    args: &[CallArg],
    name: &str,
) -> Option<f64> {
    args.iter()
        .find(|arg| arg.name.as_deref() == Some(name))
        .and_then(|arg| match &arg.value {
            Expr::Number(value) => Some(*value),
            Expr::Unary { op, expr } => match (op, expr.as_ref()) {
                (UnaryOp::Negate, Expr::Number(value)) => Some(-value),
                _ => None,
            },
            _ => None,
        })
}
