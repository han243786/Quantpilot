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
mod resolver_test_harness;
