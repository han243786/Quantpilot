mod callable_classification_surface;
mod public_type_surface;
mod resolver_orchestration_surface;
mod resolver_support_surface;
mod semantic_inference_surface;
mod type_inference_binding_surface;

use crate::diagnostics::{Diagnostic, Span};
use crate::hir::{
    DefId, ExprId, HirBindingPattern, HirCallArg, HirExpr, HirExprKind, HirFunction, HirImport,
    HirImportName, HirLetStmt, HirMatchArm, HirMatchArmBody, HirParam, HirStepBlock, HirStmt,
    HirTestAction, HirTestBlock, HirTestParamValue, TypedHirModule,
};
use crate::script::{
    BinaryOp, CallArg, Expr, FunctionDecl, Item, MatchArm, MatchArmBody, ScriptModule, Stmt,
    UnaryOp,
};
use crate::types::{parse_type_annotation, Type, TypeArena, TypeId};
use callable_classification_surface::{
    build_resolved_callables, classify_change_smoothing_kind, classify_imported_helper,
    is_known_helper_function,
};
pub use callable_classification_surface::{
    classify_builtin_math_name, classify_member_mutation_name, classify_series_capability_name,
};
pub use public_type_surface::{
    ChangeHelperKind, KnownIndicatorHelperKind, KnownUniverseHelperKind, MovingAverageHelperKind,
    ResolveResult, ResolvedBuiltinMathKind, ResolvedCallable, ResolvedCallableKind,
    ResolvedChangeSmoothingKind, ResolvedExprSemantic, ResolvedFetchSourceKind, ResolvedFunction,
    ResolvedManualIndicatorFormula, ResolvedMemberMutationKind, ResolvedSeriesBoundaryKind,
    ResolvedSeriesCapabilityKind, ResolvedSeriesViewKind, ResolvedWindowAggregateKind,
    ResolvedWindowAggregateView, RsiHelperKind,
};
use resolver_support_surface::{
    build_resolved_functions, expr_number, series_index_view_kind, series_window_span,
};
use std::collections::{BTreeMap, BTreeSet};

pub fn lower_script_to_typed_hir(module: &ScriptModule) -> ResolveResult {
    Resolver::default().resolve_module(module)
}

pub fn expr_semantic_key(expr: &Expr) -> String {
    format!("{expr:?}")
}

#[derive(Debug, Clone)]
struct FunctionSignature {
    return_type: TypeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CallableTarget {
    Builtin,
    FetchLike,
    Imported,
    UserFunction(TypeId),
}

#[derive(Debug, Clone)]
struct BindingInfo {
    ty: TypeId,
    value_expr: Option<Expr>,
}

#[derive(Debug, Default)]
struct Resolver {
    next_def_id: u32,
    next_expr_id: u32,
    types: TypeArena,
    diagnostics: Vec<Diagnostic>,
    expr_semantics: BTreeMap<String, ResolvedExprSemantic>,
    function_signatures: BTreeMap<String, FunctionSignature>,
    imported_callables: BTreeSet<String>,
    imported_callable_kinds: BTreeMap<String, ResolvedCallableKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MemberCapabilityUse {
    Access,
    Call,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::DiagnosticSeverity;
    use crate::{parse_expr, parse_quant_script_module};

    #[test]
    fn lowers_ast_into_typed_hir() {
        let module = parse_quant_script_module(
            r#"
import math

fn helper(value: Number) -> Number {
    return value + 1
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=30)?
    let signal = helper(closes.mean())
    if signal > 0 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let resolved = lower_script_to_typed_hir(&module);
        assert!(!resolved.has_errors(), "{:?}", resolved.diagnostics);
        assert_eq!(resolved.module.functions.len(), 2);

        let strategy = resolved
            .module
            .functions
            .iter()
            .find(|function| function.name == "strategy")
            .unwrap();
        let HirStmt::Let(let_stmt) = &strategy.body[0] else {
            panic!("expected let binding for closes");
        };
        assert_eq!(
            resolved.types.get(let_stmt.binding.ty),
            &Type::Series(Box::new(Type::Number))
        );
    }

    #[test]
    fn reports_duplicate_function_definitions() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    return
}

fn strategy() {
    return
}
"#,
        )
        .unwrap();

        let resolved = lower_script_to_typed_hir(&module);
        assert!(resolved.has_errors());
        assert!(resolved
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "QS0001"));
    }

    #[test]
    fn reports_unresolved_identifiers() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    if missing_signal {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let resolved = lower_script_to_typed_hir(&module);
        assert!(resolved.has_errors());
        assert!(resolved.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "QS0002"
                && diagnostic.message.contains("missing_signal")
                && diagnostic.severity == DiagnosticSeverity::Error
        }));
    }

    #[test]
    fn reports_unknown_function_call_targets() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let signal = unknown_helper(1)
    if signal > 0 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let resolved = lower_script_to_typed_hir(&module);
        assert!(resolved.has_errors());
        assert!(resolved.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "QS0005" && diagnostic.message.contains("unknown_helper")
        }));
    }

    #[test]
    fn reports_non_bool_conditions() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    if 42 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let resolved = lower_script_to_typed_hir(&module);
        assert!(resolved.has_errors());
        assert!(resolved.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "QS0006" && diagnostic.message.contains("条件必须解析为 Bool 类型")
        }));
    }

    #[test]
    fn accepts_first_and_last_as_known_call_targets() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=30)?
    let first_close = first(closes)
    let last_close = last(closes)
    if last_close > first_close {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let resolved = lower_script_to_typed_hir(&module);
        assert!(!resolved.has_errors(), "{:?}", resolved.diagnostics);
        assert!(resolved.callables.contains_key("first"));
        assert!(resolved.callables.contains_key("last"));
    }

    #[test]
    fn infers_builtin_and_imported_helper_call_types() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=30)?
    let first_close = first(closes)
    let last_close = last(closes)
    let avg_price = mean(closes)
    let gain_series = gains(closes)
    let avg_gain = wilders(gain_series, 14)
    if avg_price > 0 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let resolved = lower_script_to_typed_hir(&module);
        assert!(!resolved.has_errors(), "{:?}", resolved.diagnostics);

        let strategy = resolved
            .module
            .functions
            .iter()
            .find(|function| function.name == "strategy")
            .unwrap();

        let mut let_types = BTreeMap::new();
        for stmt in &strategy.body {
            if let HirStmt::Let(let_stmt) = stmt {
                let_types.insert(
                    let_stmt.binding.name.clone(),
                    resolved.types.get(let_stmt.binding.ty).clone(),
                );
            }
        }

        assert_eq!(
            let_types.get("first_close"),
            Some(&Type::Maybe(Box::new(Type::Number)))
        );
        assert_eq!(
            let_types.get("last_close"),
            Some(&Type::Maybe(Box::new(Type::Number)))
        );
        assert_eq!(let_types.get("avg_price"), Some(&Type::Number));
        assert_eq!(
            let_types.get("gain_series"),
            Some(&Type::Series(Box::new(Type::Number)))
        );
        assert_eq!(let_types.get("avg_gain"), Some(&Type::Number));
    }

    #[test]
    fn keeps_member_and_call_style_helper_types_consistent() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=30)?
    let call_mean = mean(closes)
    let member_mean = closes.mean()
    let call_first = first(closes)
    let member_first = closes.first()
}
"#,
        )
        .unwrap();

        let resolved = lower_script_to_typed_hir(&module);
        assert!(!resolved.has_errors(), "{:?}", resolved.diagnostics);

        let strategy = resolved
            .module
            .functions
            .iter()
            .find(|function| function.name == "strategy")
            .unwrap();

        let mut let_types = BTreeMap::new();
        for stmt in &strategy.body {
            if let HirStmt::Let(let_stmt) = stmt {
                let_types.insert(
                    let_stmt.binding.name.clone(),
                    resolved.types.get(let_stmt.binding.ty).clone(),
                );
            }
        }

        assert_eq!(let_types.get("call_mean"), let_types.get("member_mean"));
        assert_eq!(let_types.get("call_first"), let_types.get("member_first"));
        assert_eq!(let_types.get("call_mean"), Some(&Type::Number));
        assert_eq!(
            let_types.get("call_first"),
            Some(&Type::Maybe(Box::new(Type::Number)))
        );
    }

    #[test]
    fn infers_histogram_as_member_capability_type() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=30)?
    let hist = macd(closes, 12, 26, 9).histogram
}
"#,
        )
        .unwrap();

        let resolved = lower_script_to_typed_hir(&module);
        assert!(!resolved.has_errors(), "{:?}", resolved.diagnostics);

        let strategy = resolved
            .module
            .functions
            .iter()
            .find(|function| function.name == "strategy")
            .unwrap();

        let hist_ty = strategy
            .body
            .iter()
            .find_map(|stmt| match stmt {
                HirStmt::Let(let_stmt) if let_stmt.binding.name == "hist" => {
                    Some(resolved.types.get(let_stmt.binding.ty).clone())
                }
                _ => None,
            })
            .unwrap();

        assert_eq!(hist_ty, Type::Series(Box::new(Type::Number)));
    }

    #[test]
    fn records_standardized_expr_semantics_for_series_capabilities() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=30)?
    let avg_gain = wilders(gains(closes), 14)
    let avg_loss = wilders(losses(closes), 14)
    let rs = avg_gain / avg_loss
    let scope = closes[20..]
    let first_close = first(closes)
    let last_close = closes.last()
    let avg_price = closes[20..].mean()
    let delta = closes.last() - closes[14]
    let average = closes[20..].sum() / 20
    let macd_line = ema(closes, 12) - ema(closes, 26)
    let signal_line = ema(macd_line, 9)
    let macd_hist = macd_line - signal_line
    let score = (closes[0] - closes[20..].mean()) / closes[20..].stddev()
    let hist = macd(closes, 12, 26, 9).histogram
}
"#,
        )
        .unwrap();

        let resolved = lower_script_to_typed_hir(&module);
        assert!(!resolved.has_errors(), "{:?}", resolved.diagnostics);

        let scope = parse_expr("closes[20..]").unwrap();
        let rs = parse_expr("avg_gain / avg_loss").unwrap();
        let first_close = parse_expr("first(closes)").unwrap();
        let last_close = parse_expr("closes.last()").unwrap();
        let avg_price = parse_expr("closes[20..].mean()").unwrap();
        let delta = parse_expr("closes.last() - closes[14]").unwrap();
        let average = parse_expr("closes[20..].sum() / 20").unwrap();
        let macd_line = parse_expr("ema(closes, 12) - ema(closes, 26)").unwrap();
        let signal_line = parse_expr("ema(macd_line, 9)").unwrap();
        let macd_hist = parse_expr("macd_line - signal_line").unwrap();
        let score =
            parse_expr("(closes[0] - closes[20..].mean()) / closes[20..].stddev()").unwrap();
        let histogram = parse_expr("macd(closes, 12, 26, 9).histogram").unwrap();

        assert_eq!(
            resolved.expr_semantics.get(&expr_semantic_key(&scope)),
            Some(&ResolvedExprSemantic::SeriesView(
                ResolvedSeriesViewKind::Window(20)
            ))
        );
        assert_eq!(
            resolved.expr_semantics.get(&expr_semantic_key(&rs)),
            Some(&ResolvedExprSemantic::BalancedSmoothedChangePair {
                period: 14,
                smoothing: ResolvedChangeSmoothingKind::Wilder,
            })
        );
        assert_eq!(
            resolved
                .expr_semantics
                .get(&expr_semantic_key(&first_close)),
            Some(&ResolvedExprSemantic::SeriesView(
                ResolvedSeriesViewKind::First
            ))
        );
        assert_eq!(
            resolved.expr_semantics.get(&expr_semantic_key(&last_close)),
            Some(&ResolvedExprSemantic::SeriesView(
                ResolvedSeriesViewKind::Current
            ))
        );
        assert_eq!(
            resolved.expr_semantics.get(&expr_semantic_key(&avg_price)),
            Some(&ResolvedExprSemantic::WindowAggregateView(
                ResolvedWindowAggregateView {
                    aggregate_kind: ResolvedWindowAggregateKind::Mean,
                    span: 20,
                }
            ))
        );
        assert_eq!(
            resolved.expr_semantics.get(&expr_semantic_key(&delta)),
            Some(&ResolvedExprSemantic::ManualIndicatorFormula(
                ResolvedManualIndicatorFormula::Momentum { lookback: 14 }
            ))
        );
        assert_eq!(
            resolved.expr_semantics.get(&expr_semantic_key(&average)),
            Some(&ResolvedExprSemantic::ManualIndicatorFormula(
                ResolvedManualIndicatorFormula::MovingAverage { span: 20 }
            ))
        );
        assert_eq!(
            resolved.expr_semantics.get(&expr_semantic_key(&macd_line)),
            Some(&ResolvedExprSemantic::ManualIndicatorFormula(
                ResolvedManualIndicatorFormula::MacdLine {
                    fast_period: 12,
                    slow_period: 26,
                }
            ))
        );
        assert_eq!(
            resolved
                .expr_semantics
                .get(&expr_semantic_key(&signal_line)),
            Some(&ResolvedExprSemantic::ManualIndicatorFormula(
                ResolvedManualIndicatorFormula::MacdSignal {
                    fast_period: 12,
                    slow_period: 26,
                    signal_period: 9,
                }
            ))
        );
        assert_eq!(
            resolved.expr_semantics.get(&expr_semantic_key(&macd_hist)),
            Some(&ResolvedExprSemantic::ManualIndicatorFormula(
                ResolvedManualIndicatorFormula::MacdHistogram {
                    fast_period: 12,
                    slow_period: 26,
                    signal_period: 9,
                }
            ))
        );
        assert_eq!(
            resolved.expr_semantics.get(&expr_semantic_key(&score)),
            Some(&ResolvedExprSemantic::ManualIndicatorFormula(
                ResolvedManualIndicatorFormula::ZScore { window: 20 }
            ))
        );
        assert_eq!(
            resolved.expr_semantics.get(&expr_semantic_key(&histogram)),
            Some(&ResolvedExprSemantic::SeriesCapability(
                ResolvedSeriesCapabilityKind::Histogram
            ))
        );

        let boundary = parse_expr("closes.last() - closes[14]").unwrap();
        let resolver = Resolver::default();
        assert_eq!(resolver.infer_boundary_lookback_pair(&boundary), Some(14));
    }

    #[test]
    fn resolves_universe_helpers_and_for_binding_types() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    let base = universe(exchange="binance", market="spot", quote="USDT")
    let selected = top(sort_by(base, key="market_cap", order="desc"), 2)
    for s in selected {
        let closes = fetch(s, interval="1d", lookback=200)?
        emit Intent("BUY", instrument=s, quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        let resolved = lower_script_to_typed_hir(&module);
        assert!(!resolved.has_errors(), "{:?}", resolved.diagnostics);

        let strategy = resolved
            .module
            .functions
            .iter()
            .find(|function| function.name == "strategy")
            .unwrap();
        let mut let_types = BTreeMap::new();
        let mut loop_binding_type = None;
        for stmt in &strategy.body {
            match stmt {
                HirStmt::Let(let_stmt) => {
                    let_types.insert(
                        let_stmt.binding.name.clone(),
                        resolved.types.get(let_stmt.binding.ty).clone(),
                    );
                }
                HirStmt::For { binding, .. } => {
                    loop_binding_type = Some(resolved.types.get(binding.ty).clone());
                }
                _ => {}
            }
        }

        assert_eq!(let_types.get("base"), Some(&Type::Universe));
        assert_eq!(let_types.get("selected"), Some(&Type::Universe));
        assert_eq!(loop_binding_type, Some(Type::Symbol));
    }
}
