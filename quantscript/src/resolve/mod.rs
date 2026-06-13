use crate::diagnostics::{Diagnostic, DiagnosticSeverity, Span};
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
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub struct ResolveResult {
    pub module: TypedHirModule,
    pub types: TypeArena,
    pub diagnostics: Vec<Diagnostic>,
    pub expr_semantics: BTreeMap<String, ResolvedExprSemantic>,
    pub callables: BTreeMap<String, ResolvedCallable>,
    pub functions: BTreeMap<String, ResolvedFunction>,
}

impl ResolveResult {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }
}

pub fn lower_script_to_typed_hir(module: &ScriptModule) -> ResolveResult {
    Resolver::default().resolve_module(module)
}

pub fn expr_semantic_key(expr: &Expr) -> String {
    format!("{expr:?}")
}

pub fn classify_series_capability_name(name: &str) -> Option<ResolvedSeriesCapabilityKind> {
    match name {
        "histogram" => Some(ResolvedSeriesCapabilityKind::Histogram),
        "first" => Some(ResolvedSeriesCapabilityKind::Boundary(
            ResolvedSeriesBoundaryKind::First,
        )),
        "last" => Some(ResolvedSeriesCapabilityKind::Boundary(
            ResolvedSeriesBoundaryKind::Last,
        )),
        "sum" => Some(ResolvedSeriesCapabilityKind::WindowAggregate(
            ResolvedWindowAggregateKind::Sum,
        )),
        "mean" | "avg" => Some(ResolvedSeriesCapabilityKind::WindowAggregate(
            ResolvedWindowAggregateKind::Mean,
        )),
        "std" | "stddev" => Some(ResolvedSeriesCapabilityKind::WindowAggregate(
            ResolvedWindowAggregateKind::StdDev,
        )),
        _ => None,
    }
}

pub fn classify_builtin_math_name(name: &str) -> Option<ResolvedBuiltinMathKind> {
    match name {
        "abs" => Some(ResolvedBuiltinMathKind::Abs),
        "max" | "min" | "pow" | "sqrt" | "variance" => Some(ResolvedBuiltinMathKind::Numeric),
        _ => None,
    }
}

pub fn classify_member_mutation_name(name: &str) -> Option<ResolvedMemberMutationKind> {
    match name {
        "push" => Some(ResolvedMemberMutationKind::Push),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedFunction {
    pub name: String,
    pub callable_kind: ResolvedCallableKind,
    pub param_names: Vec<String>,
    pub body: Vec<Stmt>,
    pub return_type: TypeId,
    pub return_expr: Option<Expr>,
    pub returned_list_target: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedCallableKind {
    BuiltinMath,
    FetchLike,
    Imported,
    UserFunction,
    ChangeHelper(ChangeHelperKind),
    IndicatorHelper(KnownIndicatorHelperKind),
    UniverseHelper(KnownUniverseHelperKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeHelperKind {
    Gain,
    Loss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownIndicatorHelperKind {
    MovingAverage(MovingAverageHelperKind),
    Rsi(RsiHelperKind),
    Macd,
    Momentum,
    ZScore,
    Atr,
    BollingerBands,
    Obv,
    Cmf,
    Adx,
    Stochastic,
    Cci,
    ParabolicSar,
    KeltnerChannel,
    DonchianChannel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownUniverseHelperKind {
    Symbols,
    Universe,
    Filter,
    SortBy,
    Top,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovingAverageHelperKind {
    Sma,
    Ema,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RsiHelperKind {
    Wilder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedChangeSmoothingKind {
    Wilder,
    Ema,
    Simple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedFetchSourceKind {
    KlineSeries,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedSeriesCapabilityKind {
    Histogram,
    Boundary(ResolvedSeriesBoundaryKind),
    WindowAggregate(ResolvedWindowAggregateKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedSeriesBoundaryKind {
    First,
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedWindowAggregateKind {
    Sum,
    Mean,
    StdDev,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedBuiltinMathKind {
    Abs,
    Numeric,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedMemberMutationKind {
    Push,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedExprSemantic {
    SeriesView(ResolvedSeriesViewKind),
    SeriesCapability(ResolvedSeriesCapabilityKind),
    WindowAggregateView(ResolvedWindowAggregateView),
    BoundaryLookbackPair {
        span: usize,
    },
    BalancedSmoothedChangePair {
        period: usize,
        smoothing: ResolvedChangeSmoothingKind,
    },
    ManualIndicatorFormula(ResolvedManualIndicatorFormula),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedSeriesViewKind {
    Current,
    First,
    Lookback(usize),
    Window(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedWindowAggregateView {
    pub aggregate_kind: ResolvedWindowAggregateKind,
    pub span: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedManualIndicatorFormula {
    Momentum {
        lookback: usize,
    },
    MovingAverage {
        span: usize,
    },
    MacdSignal {
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    },
    MacdHistogram {
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    },
    MacdLine {
        fast_period: usize,
        slow_period: usize,
    },
    ZScore {
        window: usize,
    },
}

#[derive(Debug, Clone)]
pub struct ResolvedCallable {
    pub name: String,
    pub kind: ResolvedCallableKind,
    pub change_smoothing_kind: Option<ResolvedChangeSmoothingKind>,
    pub fetch_source_kind: Option<ResolvedFetchSourceKind>,
    pub return_type: TypeId,
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

impl Resolver {
    fn resolve_module(mut self, module: &ScriptModule) -> ResolveResult {
        self.seed_imported_callables(module);
        self.seed_function_signatures(module);

        let imports = module
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Import(import_decl) => Some(HirImport {
                    module: import_decl.module.clone(),
                    version: import_decl.version.clone(),
                    names: import_decl
                        .names
                        .as_ref()
                        .map(|names| {
                            names
                                .iter()
                                .map(|name| HirImportName {
                                    name: name.name.clone(),
                                    alias: name.alias.clone(),
                                })
                                .collect()
                        })
                        .unwrap_or_default(),
                    span: Span::module(import_decl.module.clone()),
                }),
                _ => None,
            })
            .collect();

        let functions = module
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Function(function) => Some(self.resolve_function(function)),
                _ => None,
            })
            .collect();

        let test_blocks = module
            .items
            .iter()
            .filter_map(|item| match item {
                Item::TestBlock(test_block) => Some(HirTestBlock {
                    name: test_block.name.clone(),
                    cover: test_block.cover.clone(),
                    steps: test_block
                        .steps
                        .iter()
                        .map(|step| HirStepBlock {
                            name: step.name.clone(),
                            actions: step
                                .actions
                                .iter()
                                .map(|action| match action {
                                    crate::script::TestAction::Compile => HirTestAction::Compile,
                                    crate::script::TestAction::Run {
                                        mode,
                                        duration_secs,
                                        save,
                                    } => HirTestAction::Run {
                                        mode: mode.clone(),
                                        duration_secs: *duration_secs,
                                        save: *save,
                                    },
                                    crate::script::TestAction::Backtest {
                                        source,
                                        start,
                                        end,
                                        seed,
                                        save,
                                        volatility,
                                    } => HirTestAction::Backtest {
                                        source: source.clone(),
                                        start: start.clone(),
                                        end: end.clone(),
                                        seed: *seed,
                                        save: *save,
                                        volatility: *volatility,
                                    },
                                    crate::script::TestAction::Assert(expr) => {
                                        HirTestAction::Assert(expr.clone())
                                    }
                                    crate::script::TestAction::SaveRun => HirTestAction::SaveRun,
                                    crate::script::TestAction::Modify { node, param, value } => {
                                        HirTestAction::Modify {
                                            node: node.clone(),
                                            param: param.clone(),
                                            value: match value {
                                                crate::script::TestParamValue::Number(n) => {
                                                    HirTestParamValue::Number(*n)
                                                }
                                                crate::script::TestParamValue::String(s) => {
                                                    HirTestParamValue::String(s.clone())
                                                }
                                                crate::script::TestParamValue::Bool(b) => {
                                                    HirTestParamValue::Bool(*b)
                                                }
                                            },
                                        }
                                    }
                                    crate::script::TestAction::Wait {
                                        condition,
                                        timeout_secs,
                                    } => HirTestAction::Wait {
                                        condition: condition.clone(),
                                        timeout_secs: *timeout_secs,
                                    },
                                    crate::script::TestAction::CompareBacktests { left, right } => {
                                        HirTestAction::CompareBacktests {
                                            left: *left,
                                            right: *right,
                                        }
                                    }
                                    crate::script::TestAction::Debug(vars) => {
                                        HirTestAction::Debug(vars.clone())
                                    }
                                })
                                .collect(),
                        })
                        .collect(),
                }),
                _ => None,
            })
            .collect();

        ResolveResult {
            module: TypedHirModule {
                imports,
                functions,
                test_blocks,
            },
            types: self.types,
            diagnostics: self.diagnostics,
            expr_semantics: self.expr_semantics,
            callables: build_resolved_callables(module, &self.function_signatures),
            functions: build_resolved_functions(module, &self.function_signatures),
        }
    }

    fn seed_imported_callables(&mut self, module: &ScriptModule) {
        for item in &module.items {
            let Item::Import(import_decl) = item else {
                continue;
            };

            let Some(names) = &import_decl.names else {
                continue;
            };

            for name in names {
                let local_name = name.alias.clone().unwrap_or_else(|| name.name.clone());
                self.imported_callables.insert(local_name.clone());
                self.imported_callable_kinds
                    .insert(local_name, classify_imported_helper(&name.name));
            }
        }
    }

    fn seed_function_signatures(&mut self, module: &ScriptModule) {
        for item in &module.items {
            let Item::Function(function) = item else {
                continue;
            };

            let span = Span::function(function.name.clone());
            let return_type = function
                .return_type
                .as_deref()
                .map(|annotation| self.resolve_type(annotation, span.clone()))
                .unwrap_or_else(|| self.types.unknown());

            if self.function_signatures.contains_key(&function.name) {
                self.diagnostics.push(Diagnostic::error(
                    "QS0001",
                    format!("重复的函数定义: {}", function.name),
                    Some(span),
                ));
                continue;
            }

            self.function_signatures
                .insert(function.name.clone(), FunctionSignature { return_type });
        }
    }

    fn resolve_function(&mut self, function: &FunctionDecl) -> HirFunction {
        let span = Span::function(function.name.clone());
        let mut scope = BTreeMap::new();
        let mut params = Vec::new();

        for param in &function.params {
            let param_span = Span::binding(param.name.clone());
            let def_id = self.alloc_def_id();
            let ty = param
                .ty
                .as_deref()
                .map(|annotation| self.resolve_type(annotation, param_span.clone()))
                .unwrap_or_else(|| self.types.unknown());
            self.insert_binding(&mut scope, param.name.clone(), ty, None, &param_span);
            params.push(HirParam {
                def_id,
                name: param.name.clone(),
                ty,
                span: param_span,
            });
        }

        let return_type = self
            .function_signatures
            .get(&function.name)
            .map(|signature| signature.return_type)
            .unwrap_or_else(|| self.types.unknown());
        let body = self.lower_block(&function.body, &mut scope);

        HirFunction {
            def_id: self.alloc_def_id(),
            name: function.name.clone(),
            is_async: function.is_async,
            params,
            return_type,
            body,
            span,
        }
    }

    fn lower_block(
        &mut self,
        stmts: &[Stmt],
        scope: &mut BTreeMap<String, BindingInfo>,
    ) -> Vec<HirStmt> {
        stmts
            .iter()
            .map(|stmt| self.lower_stmt(stmt, scope))
            .collect()
    }

    fn lower_stmt(&mut self, stmt: &Stmt, scope: &mut BTreeMap<String, BindingInfo>) -> HirStmt {
        match stmt {
            Stmt::Let {
                pattern,
                ty,
                value,
                mutable,
            } => {
                let value_expr = value.clone();
                let value = self.lower_expr(value, scope);
                let span = Span::binding(pattern.clone());
                let binding_ty = ty
                    .as_deref()
                    .map(|annotation| self.resolve_type(annotation, span.clone()))
                    .unwrap_or(value.ty);
                let def_id = self.alloc_def_id();
                self.insert_binding(scope, pattern.clone(), binding_ty, Some(value_expr), &span);
                HirStmt::Let(HirLetStmt {
                    binding: HirBindingPattern {
                        def_id,
                        name: pattern.clone(),
                        ty: binding_ty,
                        span: span.clone(),
                    },
                    value,
                    mutable: *mutable,
                    span,
                })
            }
            Stmt::Return(expr) => {
                HirStmt::Return(expr.as_ref().map(|expr| self.lower_expr(expr, scope)))
            }
            Stmt::EmitIntent { args } => HirStmt::EmitIntent {
                args: args
                    .iter()
                    .map(|arg| HirCallArg {
                        name: arg.name.clone(),
                        value: self.lower_expr(&arg.value, scope),
                    })
                    .collect(),
                span: Span::expr("emit Intent"),
            },
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                let condition = self.lower_expr(condition, scope);
                let then_branch = self.lower_block(then_branch, &mut scope.clone());
                self.validate_condition_type(&condition, "if");
                let else_if_branches = else_if_branches
                    .iter()
                    .map(|(condition, stmts)| {
                        let condition = self.lower_expr(condition, scope);
                        self.validate_condition_type(&condition, "else if");
                        (condition, self.lower_block(stmts, &mut scope.clone()))
                    })
                    .collect();
                let else_branch = else_branch
                    .as_ref()
                    .map(|stmts| self.lower_block(stmts, &mut scope.clone()));
                HirStmt::If {
                    condition,
                    then_branch,
                    else_if_branches,
                    else_branch,
                    span: Span::expr("if"),
                }
            }
            Stmt::For {
                pattern,
                iterable,
                body,
            } => {
                let iterable = self.lower_expr(iterable, scope);
                let binding_ty = self.iteration_item_type(iterable.ty);
                let def_id = self.alloc_def_id();
                let span = Span::binding(pattern.clone());
                let mut nested_scope = scope.clone();
                self.insert_binding(&mut nested_scope, pattern.clone(), binding_ty, None, &span);
                let body = self.lower_block(body, &mut nested_scope);
                HirStmt::For {
                    binding: HirBindingPattern {
                        def_id,
                        name: pattern.clone(),
                        ty: binding_ty,
                        span,
                    },
                    iterable,
                    body,
                    span: Span::expr("for"),
                }
            }
            Stmt::While { condition, body } => HirStmt::While {
                condition: {
                    let condition = self.lower_expr(condition, scope);
                    self.validate_condition_type(&condition, "while");
                    condition
                },
                body: self.lower_block(body, &mut scope.clone()),
                span: Span::expr("while"),
            },
            Stmt::Match { expr, arms } => HirStmt::Match {
                expr: self.lower_expr(expr, scope),
                arms: arms
                    .iter()
                    .map(|arm| self.lower_match_arm(arm, scope))
                    .collect(),
                span: Span::expr("match"),
            },
            Stmt::Expr(expr) => HirStmt::Expr(self.lower_expr(expr, scope)),
        }
    }

    fn lower_match_arm(
        &mut self,
        arm: &MatchArm,
        scope: &BTreeMap<String, BindingInfo>,
    ) -> HirMatchArm {
        HirMatchArm {
            pattern: arm.pattern.clone(),
            body: match &arm.body {
                MatchArmBody::Statement(stmt) => {
                    HirMatchArmBody::Statement(Box::new(self.lower_stmt(stmt, &mut scope.clone())))
                }
                MatchArmBody::Expr(expr) => HirMatchArmBody::Expr(self.lower_expr(expr, scope)),
            },
            span: Span::expr(format!("match arm {}", arm.pattern)),
        }
    }

    fn lower_expr(&mut self, expr: &Expr, scope: &BTreeMap<String, BindingInfo>) -> HirExpr {
        let lowered = match expr {
            Expr::Raw(value) => {
                let ty = self.types.unknown();
                self.make_expr(HirExprKind::Raw(value.clone()), ty, expr)
            }
            Expr::Identifier(name) => {
                let ty = scope
                    .get(name)
                    .map(|binding| binding.ty)
                    .unwrap_or_else(|| {
                        if self.function_signatures.contains_key(name) {
                            self.types.unknown()
                        } else {
                            self.diagnostics.push(Diagnostic::error(
                                "QS0002",
                                format!("未解析的标识符: {name}"),
                                Some(Span::binding(name.clone())),
                            ));
                            self.types.unknown()
                        }
                    });
                self.make_expr(HirExprKind::Identifier(name.clone()), ty, expr)
            }
            Expr::Number(value) => {
                let ty = self.types.number();
                self.make_expr(HirExprKind::Number(*value), ty, expr)
            }
            Expr::String(value) => {
                let ty = self.types.string();
                self.make_expr(HirExprKind::String(value.clone()), ty, expr)
            }
            Expr::Bool(value) => {
                let ty = self.types.bool();
                self.make_expr(HirExprKind::Bool(*value), ty, expr)
            }
            Expr::List(items) => {
                let items = items
                    .iter()
                    .map(|item| self.lower_expr(item, scope))
                    .collect::<Vec<_>>();
                let item_ty = self.common_item_type(&items);
                let ty = self.types.list(item_ty);
                self.make_expr(HirExprKind::List(items), ty, expr)
            }
            Expr::Call { callee, args } => {
                let callee = self.lower_callee_expr(callee, scope);
                let args = args
                    .iter()
                    .map(|arg| HirCallArg {
                        name: arg.name.clone(),
                        value: self.lower_expr(&arg.value, scope),
                    })
                    .collect::<Vec<_>>();
                let ty = self.infer_call_type(callee.as_ref(), &args);
                self.make_expr(HirExprKind::Call { callee, args }, ty, expr)
            }
            Expr::Member { object, field } => {
                let object = if matches!(
                    object.as_ref(),
                    Expr::Identifier(name)
                        if (name == "risk" || name == "execution") && field == "profile"
                ) {
                    let ty = self.types.unknown();
                    self.make_expr(
                        HirExprKind::Identifier(match object.as_ref() {
                            Expr::Identifier(name) => name.clone(),
                            _ => unreachable!(),
                        }),
                        ty,
                        object.as_ref(),
                    )
                } else {
                    self.lower_expr(object, scope)
                };
                let ty = self.infer_member_capability_type(
                    field,
                    object.ty,
                    MemberCapabilityUse::Access,
                );
                self.make_expr(
                    HirExprKind::Member {
                        object: Box::new(object),
                        field: field.clone(),
                    },
                    ty,
                    expr,
                )
            }
            Expr::Index { object, index } => {
                let object = self.lower_expr(object, scope);
                let index = self.lower_expr(index, scope);
                let ty = self.index_result_type(object.ty);
                self.make_expr(
                    HirExprKind::Index {
                        object: Box::new(object),
                        index: Box::new(index),
                    },
                    ty,
                    expr,
                )
            }
            Expr::Slice { object, start, end } => {
                let object = self.lower_expr(object, scope);
                let start = start
                    .as_ref()
                    .map(|value| Box::new(self.lower_expr(value, scope)));
                let end = end
                    .as_ref()
                    .map(|value| Box::new(self.lower_expr(value, scope)));
                self.make_expr(
                    HirExprKind::Slice {
                        object: Box::new(object.clone()),
                        start,
                        end,
                    },
                    object.ty,
                    expr,
                )
            }
            Expr::Unary { op, expr: inner } => {
                let inner = self.lower_expr(inner, scope);
                let ty = self.infer_unary_type(op, inner.ty);
                self.make_expr(
                    HirExprKind::Unary {
                        op: op.clone(),
                        expr: Box::new(inner),
                    },
                    ty,
                    expr,
                )
            }
            Expr::Binary { left, op, right } => {
                let left = self.lower_expr(left, scope);
                let right = self.lower_expr(right, scope);
                let ty = self.infer_binary_type(op, left.ty, right.ty);
                self.make_expr(
                    HirExprKind::Binary {
                        left: Box::new(left),
                        op: op.clone(),
                        right: Box::new(right),
                    },
                    ty,
                    expr,
                )
            }
            Expr::Range { start, end } => {
                let start = self.lower_expr(start, scope);
                let end = self.lower_expr(end, scope);
                let number = self.types.number();
                let ty = self.types.list(number);
                self.make_expr(
                    HirExprKind::Range {
                        start: Box::new(start),
                        end: Box::new(end),
                    },
                    ty,
                    expr,
                )
            }
            Expr::Await(inner) => {
                let inner = self.lower_expr(inner, scope);
                self.make_expr(HirExprKind::Await(Box::new(inner.clone())), inner.ty, expr)
            }
            Expr::Try(inner) => {
                let inner = self.lower_expr(inner, scope);
                let ty = self.unwrap_maybe(inner.ty);
                self.make_expr(HirExprKind::Try(Box::new(inner)), ty, expr)
            }
        };

        if let Some(semantic) = self.infer_expr_semantic(expr, scope) {
            self.expr_semantics
                .insert(expr_semantic_key(expr), semantic);
        }

        lowered
    }

    fn lower_callee_expr(
        &mut self,
        expr: &Expr,
        scope: &BTreeMap<String, BindingInfo>,
    ) -> Box<HirExpr> {
        let lowered = match expr {
            Expr::Identifier(name) if self.callable_target(name).is_some() => {
                let ty = self.call_target_return_type(name);
                self.make_expr(HirExprKind::Identifier(name.clone()), ty, expr)
            }
            Expr::Identifier(name) if scope.contains_key(name) => {
                self.diagnostics.push(Diagnostic::error(
                    "QS0005",
                    format!("调用目标不是函数: {name}"),
                    Some(Span::binding(name.clone())),
                ));
                self.lower_expr(expr, scope)
            }
            Expr::Identifier(name) => {
                self.diagnostics.push(Diagnostic::error(
                    "QS0005",
                    format!("未知的函数调用目标: {name}"),
                    Some(Span::binding(name.clone())),
                ));
                let ty = self.types.unknown();
                self.make_expr(HirExprKind::Identifier(name.clone()), ty, expr)
            }
            _ => self.lower_expr(expr, scope),
        };
        Box::new(lowered)
    }

    fn infer_call_type(&mut self, callee: &HirExpr, args: &[HirCallArg]) -> TypeId {
        match &callee.kind {
            HirExprKind::Identifier(name) => match self.callable_target(name) {
                Some(CallableTarget::FetchLike) => {
                    let number = self.types.number();
                    let series = self.types.series(number);
                    self.types.maybe(series)
                }
                Some(CallableTarget::Builtin) | Some(CallableTarget::Imported) => {
                    // B1-4: 指标参数类型约束
                    if matches!(
                        name.as_str(),
                        "sma" | "ema" | "rsi" | "macd" | "momentum" | "zscore" | "z_score"
                    ) {
                        if let Some(first_arg) = args.first() {
                            let is_literal_number =
                                matches!(&first_arg.value.kind, HirExprKind::Number(_));
                            if is_literal_number {
                                self.diagnostics.push(Diagnostic::error(
                                    "QS0007",
                                    format!("{} 的第一个参数必须是 fetch() 或数据系列", name),
                                    Some(callee.span.clone()),
                                ));
                            }
                        }
                    }
                    self.infer_named_helper_return_type(name, args.first().map(|arg| arg.value.ty))
                }
                Some(CallableTarget::UserFunction(return_type)) => return_type,
                None => self.types.unknown(),
            },
            HirExprKind::Member { object, field } => {
                self.infer_member_capability_type(field, object.ty, MemberCapabilityUse::Call)
            }
            _ => self.types.unknown(),
        }
    }

    fn infer_member_capability_type(
        &mut self,
        field: &str,
        object_ty: TypeId,
        usage: MemberCapabilityUse,
    ) -> TypeId {
        match usage {
            MemberCapabilityUse::Call => {
                self.infer_named_helper_return_type(field, Some(object_ty))
            }
            MemberCapabilityUse::Access => match classify_series_capability_name(field) {
                Some(ResolvedSeriesCapabilityKind::Histogram) => {
                    let number = self.types.number();
                    self.types.series(number)
                }
                _ => self.types.unknown(),
            },
        }
    }

    fn infer_named_helper_return_type(
        &mut self,
        name: &str,
        first_arg_ty: Option<TypeId>,
    ) -> TypeId {
        match name {
            "symbols" | "universe" | "filter" | "sort_by" | "top" => return self.types.universe(),
            _ => {}
        }
        match classify_series_capability_name(name) {
            Some(ResolvedSeriesCapabilityKind::Boundary(_)) => {
                let item_ty = first_arg_ty
                    .map(|arg_ty| self.sequence_item_type(arg_ty))
                    .unwrap_or_else(|| self.types.unknown());
                self.types.maybe(item_ty)
            }
            Some(ResolvedSeriesCapabilityKind::WindowAggregate(_)) => self.types.number(),
            Some(ResolvedSeriesCapabilityKind::Histogram) => self.types.unknown(),
            None => match classify_builtin_math_name(name) {
                Some(ResolvedBuiltinMathKind::Abs | ResolvedBuiltinMathKind::Numeric) => {
                    self.types.number()
                }
                None => match name {
                    "field" | "resample" | "align" | "align_asof" => first_arg_ty
                        .map(|arg_ty| self.numeric_sequence_type(arg_ty))
                        .unwrap_or_else(|| self.types.unknown()),
                    "spread" => self.types.number(),
                    "gains" | "gain" | "up_moves" | "positive_changes" | "positive_deltas"
                    | "losses" | "loss" | "down_moves" | "negative_changes" | "negative_deltas" => {
                        first_arg_ty
                            .map(|arg_ty| self.numeric_sequence_type(arg_ty))
                            .unwrap_or_else(|| self.types.unknown())
                    }
                    "sma" | "ema" | "rma" | "wilders" | "smma" | "rsi" | "macd" | "momentum"
                    | "zscore" | "z_score" => self.types.number(),
                    _ => self.types.unknown(),
                },
            },
        }
    }

    fn infer_unary_type(&mut self, op: &UnaryOp, inner_ty: TypeId) -> TypeId {
        match op {
            UnaryOp::Negate => {
                if self.is_numeric(inner_ty) {
                    self.types.number()
                } else {
                    self.types.unknown()
                }
            }
            UnaryOp::Not => self.types.bool(),
        }
    }

    fn infer_binary_type(&mut self, op: &BinaryOp, left_ty: TypeId, right_ty: TypeId) -> TypeId {
        match op {
            BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::Modulo => {
                if self.is_numeric(left_ty) && self.is_numeric(right_ty) {
                    self.types.number()
                } else {
                    self.types.unknown()
                }
            }
            BinaryOp::Greater
            | BinaryOp::GreaterEqual
            | BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::And
            | BinaryOp::Or => self.types.bool(),
        }
    }

    fn common_item_type(&mut self, items: &[HirExpr]) -> TypeId {
        let first = items
            .first()
            .map(|item| item.ty)
            .unwrap_or_else(|| self.types.unknown());
        if items
            .iter()
            .all(|item| self.types.get(item.ty) == self.types.get(first))
        {
            first
        } else {
            self.types.unknown()
        }
    }

    fn index_result_type(&mut self, ty: TypeId) -> TypeId {
        match self.types.get(ty).clone() {
            Type::List(inner) | Type::Series(inner) | Type::Scalar(inner) | Type::Maybe(inner) => {
                self.types.intern(*inner)
            }
            Type::Universe => self.types.symbol(),
            _ => self.types.unknown(),
        }
    }

    fn sequence_item_type(&mut self, ty: TypeId) -> TypeId {
        match self.types.get(ty).clone() {
            Type::List(inner) | Type::Series(inner) => self.types.intern(*inner),
            Type::Universe => self.types.symbol(),
            Type::Maybe(inner) => {
                let inner_ty = self.types.intern(*inner);
                self.sequence_item_type(inner_ty)
            }
            _ => self.types.unknown(),
        }
    }

    fn numeric_sequence_type(&mut self, ty: TypeId) -> TypeId {
        match self.types.get(ty).clone() {
            Type::Series(_) => {
                let number = self.types.number();
                self.types.series(number)
            }
            Type::List(_) => {
                let number = self.types.number();
                self.types.list(number)
            }
            Type::Maybe(inner) => {
                let inner_ty = self.types.intern(*inner);
                let numeric_inner = self.numeric_sequence_type(inner_ty);
                self.types.maybe(numeric_inner)
            }
            _ => self.types.unknown(),
        }
    }

    fn iteration_item_type(&mut self, ty: TypeId) -> TypeId {
        match self.types.get(ty).clone() {
            Type::List(inner) | Type::Series(inner) => self.types.intern(*inner),
            Type::Universe => self.types.symbol(),
            _ => self.types.unknown(),
        }
    }

    fn unwrap_maybe(&mut self, ty: TypeId) -> TypeId {
        match self.types.get(ty).clone() {
            Type::Maybe(inner) => self.types.intern(*inner),
            _ => ty,
        }
    }

    fn is_numeric(&self, ty: TypeId) -> bool {
        match self.types.get(ty) {
            Type::Number => true,
            Type::Scalar(inner) => matches!(**inner, Type::Number),
            _ => false,
        }
    }

    fn insert_binding(
        &mut self,
        scope: &mut BTreeMap<String, BindingInfo>,
        name: String,
        ty: TypeId,
        value_expr: Option<Expr>,
        span: &Span,
    ) {
        // B1-10: 重复变量定义诊断
        if scope.contains_key(&name) {
            self.diagnostics.push(Diagnostic::warning(
                "QS0613",
                format!("重复的变量定义 '{}'", name),
                Some(span.clone()),
            ));
        }
        // B1-2: 变量遮蔽检测
        let is_known = self.function_signatures.contains_key(&name);
        if is_known || is_known_helper_function(&name) {
            self.diagnostics.push(Diagnostic::warning(
                "QS0600",
                format!("变量 '{}' 遮蔽了同名内置函数", name),
                Some(span.clone()),
            ));
        }
        scope.insert(name, BindingInfo { ty, value_expr });
    }

    fn validate_condition_type(&mut self, condition: &HirExpr, label: &str) {
        if matches!(self.types.get(condition.ty), Type::Bool | Type::Unknown) {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            "QS0006",
            format!("{label} 条件必须解析为 Bool 类型"),
            Some(condition.span.clone()),
        ));
    }

    fn resolve_type(&mut self, annotation: &str, span: Span) -> TypeId {
        match parse_type_annotation(annotation) {
            Ok(ty) => self.types.intern(ty),
            Err(message) => {
                self.diagnostics
                    .push(Diagnostic::error("QS0003", message, Some(span)));
                self.types.unknown()
            }
        }
    }

    fn infer_expr_semantic(
        &self,
        expr: &Expr,
        scope: &BTreeMap<String, BindingInfo>,
    ) -> Option<ResolvedExprSemantic> {
        if let Some(formula) = self.infer_manual_indicator_formula(expr, scope) {
            return Some(ResolvedExprSemantic::ManualIndicatorFormula(formula));
        }
        if let Some(window_aggregate) = self.infer_window_aggregate_view(expr) {
            return Some(ResolvedExprSemantic::WindowAggregateView(window_aggregate));
        }
        if let Some(span) = self.infer_boundary_lookback_pair(expr) {
            return Some(ResolvedExprSemantic::BoundaryLookbackPair { span });
        }
        if let Some((period, smoothing)) = self.infer_balanced_smoothed_change_pair(expr, scope) {
            return Some(ResolvedExprSemantic::BalancedSmoothedChangePair { period, smoothing });
        }

        match expr {
            Expr::Slice { start, end, .. } => {
                if end.is_some() {
                    return None;
                }
                let span = start.as_deref().and_then(series_window_span)?;
                Some(ResolvedExprSemantic::SeriesView(
                    ResolvedSeriesViewKind::Window(span),
                ))
            }
            Expr::Index { index, .. } => Some(ResolvedExprSemantic::SeriesView(
                series_index_view_kind(index)?,
            )),
            Expr::Call { callee, .. } => {
                let name = match callee.as_ref() {
                    Expr::Identifier(name) => name.as_str(),
                    Expr::Member { field, .. } => field.as_str(),
                    _ => return None,
                };
                match classify_series_capability_name(name)? {
                    ResolvedSeriesCapabilityKind::Boundary(ResolvedSeriesBoundaryKind::First) => {
                        Some(ResolvedExprSemantic::SeriesView(
                            ResolvedSeriesViewKind::First,
                        ))
                    }
                    ResolvedSeriesCapabilityKind::Boundary(ResolvedSeriesBoundaryKind::Last) => {
                        Some(ResolvedExprSemantic::SeriesView(
                            ResolvedSeriesViewKind::Current,
                        ))
                    }
                    capability @ ResolvedSeriesCapabilityKind::WindowAggregate(_) => {
                        Some(ResolvedExprSemantic::SeriesCapability(capability))
                    }
                    ResolvedSeriesCapabilityKind::Histogram => None,
                }
            }
            Expr::Member { field, .. } => {
                let capability = classify_series_capability_name(field)?;
                Some(ResolvedExprSemantic::SeriesCapability(capability))
            }
            _ => None,
        }
    }

    fn infer_manual_indicator_formula(
        &self,
        expr: &Expr,
        scope: &BTreeMap<String, BindingInfo>,
    ) -> Option<ResolvedManualIndicatorFormula> {
        if let Some((fast_period, slow_period, signal_period)) =
            self.infer_manual_macd_histogram_formula(expr, scope)
        {
            return Some(ResolvedManualIndicatorFormula::MacdHistogram {
                fast_period,
                slow_period,
                signal_period,
            });
        }
        if let Some((fast_period, slow_period)) = self.infer_manual_macd_line_formula(expr, scope) {
            return Some(ResolvedManualIndicatorFormula::MacdLine {
                fast_period,
                slow_period,
            });
        }
        if let Some((fast_period, slow_period, signal_period)) =
            self.infer_manual_macd_signal_formula(expr, scope)
        {
            return Some(ResolvedManualIndicatorFormula::MacdSignal {
                fast_period,
                slow_period,
                signal_period,
            });
        }
        if let Some(lookback) = self.infer_boundary_lookback_pair(expr) {
            let Expr::Binary { op, .. } = expr else {
                return None;
            };
            if matches!(op, BinaryOp::Subtract | BinaryOp::Divide) {
                return Some(ResolvedManualIndicatorFormula::Momentum { lookback });
            }
        }

        let Expr::Binary { left, op, right } = expr else {
            return None;
        };
        if !matches!(op, BinaryOp::Divide) {
            return None;
        }

        if let Some(span) = self.infer_manual_moving_average_formula(left, right, scope) {
            return Some(ResolvedManualIndicatorFormula::MovingAverage { span });
        }
        if let Some(window) = self.infer_manual_zscore_formula(left, right, scope) {
            return Some(ResolvedManualIndicatorFormula::ZScore { window });
        }

        None
    }

    fn infer_manual_macd_histogram_formula(
        &self,
        expr: &Expr,
        scope: &BTreeMap<String, BindingInfo>,
    ) -> Option<(usize, usize, usize)> {
        let expr = self.resolve_alias_expr(expr, scope);
        let Expr::Binary {
            left,
            op: BinaryOp::Subtract,
            right,
        } = expr
        else {
            return None;
        };

        let (line_target, fast_period, slow_period) =
            self.infer_manual_macd_line_shape(left, scope)?;
        let (signal_target, signal_fast, signal_slow, signal_period) =
            self.infer_signal_line_shape(right, scope)?;
        if line_target != signal_target || fast_period != signal_fast || slow_period != signal_slow
        {
            return None;
        }

        Some((fast_period, slow_period, signal_period))
    }

    fn infer_manual_macd_line_formula(
        &self,
        expr: &Expr,
        scope: &BTreeMap<String, BindingInfo>,
    ) -> Option<(usize, usize)> {
        let (_, fast_period, slow_period) = self.infer_manual_macd_line_shape(expr, scope)?;
        Some((fast_period, slow_period))
    }

    fn infer_manual_macd_signal_formula(
        &self,
        expr: &Expr,
        scope: &BTreeMap<String, BindingInfo>,
    ) -> Option<(usize, usize, usize)> {
        let (_, fast_period, slow_period, signal_period) =
            self.infer_signal_line_shape(expr, scope)?;
        Some((fast_period, slow_period, signal_period))
    }

    fn infer_manual_macd_line_shape<'a>(
        &self,
        expr: &'a Expr,
        scope: &'a BTreeMap<String, BindingInfo>,
    ) -> Option<(&'a Expr, usize, usize)> {
        let expr = self.resolve_alias_expr(expr, scope);
        let Expr::Binary {
            left,
            op: BinaryOp::Subtract,
            right,
        } = expr
        else {
            return None;
        };
        let (left_target, left_period) = self.infer_ema_call(left, scope)?;
        let (right_target, right_period) = self.infer_ema_call(right, scope)?;
        if left_target != right_target {
            return None;
        }
        let fast_period = left_period.min(right_period);
        let slow_period = left_period.max(right_period);
        if fast_period == slow_period {
            return None;
        }
        Some((left_target, fast_period, slow_period))
    }

    fn infer_signal_line_shape<'a>(
        &self,
        expr: &'a Expr,
        scope: &'a BTreeMap<String, BindingInfo>,
    ) -> Option<(&'a Expr, usize, usize, usize)> {
        let expr = self.resolve_alias_expr(expr, scope);
        let Expr::Call { callee, args } = expr else {
            return None;
        };
        let name = match callee.as_ref() {
            Expr::Identifier(name) => name.as_str(),
            Expr::Member { field, .. } => field.as_str(),
            _ => return None,
        };
        if !self.is_ema_like_name(name) {
            return None;
        }
        let macd_line_expr = args.iter().find(|arg| arg.name.is_none())?;
        let (target, fast_period, slow_period) =
            self.infer_manual_macd_line_shape(&macd_line_expr.value, scope)?;
        let signal_period = args
            .iter()
            .filter(|arg| arg.name.is_none())
            .nth(1)
            .and_then(|arg| expr_number(&arg.value))
            .map(|value| value.round() as usize)?;
        if signal_period == 0 {
            return None;
        }

        Some((target, fast_period, slow_period, signal_period))
    }

    fn infer_ema_call<'a>(
        &self,
        expr: &'a Expr,
        scope: &'a BTreeMap<String, BindingInfo>,
    ) -> Option<(&'a Expr, usize)> {
        let expr = self.resolve_alias_expr(expr, scope);
        let Expr::Call { callee, args } = expr else {
            return None;
        };
        let name = match callee.as_ref() {
            Expr::Identifier(name) => name.as_str(),
            Expr::Member { field, .. } => field.as_str(),
            _ => return None,
        };
        if !self.is_ema_like_name(name) {
            return None;
        }
        let target = self.series_capability_target_expr(expr, callee.as_ref(), args.as_slice())?;
        let period = args
            .iter()
            .filter(|arg| arg.name.is_none())
            .nth(1)
            .and_then(|arg| expr_number(&arg.value))
            .map(|value| value.round() as usize)?;
        if period == 0 {
            return None;
        }
        Some((target, period))
    }

    fn infer_manual_moving_average_formula(
        &self,
        left: &Expr,
        right: &Expr,
        scope: &BTreeMap<String, BindingInfo>,
    ) -> Option<usize> {
        let ResolvedExprSemantic::WindowAggregateView(window_aggregate) =
            self.infer_expr_semantic(left, scope)?
        else {
            return None;
        };
        if window_aggregate.aggregate_kind != ResolvedWindowAggregateKind::Sum {
            return None;
        }
        let period = expr_number(right).map(|value| value.round() as usize)?;
        if period == 0 || period != window_aggregate.span {
            return None;
        }
        Some(period)
    }

    fn infer_manual_zscore_formula(
        &self,
        left: &Expr,
        right: &Expr,
        scope: &BTreeMap<String, BindingInfo>,
    ) -> Option<usize> {
        let Expr::Binary {
            left: current_expr,
            op: BinaryOp::Subtract,
            right: mean_expr,
        } = left
        else {
            return None;
        };

        let (_, current_view) = self.infer_series_view_shape(current_expr)?;
        if current_view != ResolvedSeriesViewKind::Current {
            return None;
        }

        let ResolvedExprSemantic::WindowAggregateView(mean_view) =
            self.infer_expr_semantic(mean_expr, scope)?
        else {
            return None;
        };
        let ResolvedExprSemantic::WindowAggregateView(std_view) =
            self.infer_expr_semantic(right, scope)?
        else {
            return None;
        };
        if mean_view.aggregate_kind != ResolvedWindowAggregateKind::Mean
            || std_view.aggregate_kind != ResolvedWindowAggregateKind::StdDev
            || mean_view.span != std_view.span
        {
            return None;
        }

        let (current_target, _) = self.infer_series_view_shape(current_expr)?;
        let (mean_target, ResolvedSeriesViewKind::Window(_)) =
            self.infer_series_view_shape(self.window_aggregate_target_expr(mean_expr)?)?
        else {
            return None;
        };
        let (std_target, ResolvedSeriesViewKind::Window(_)) =
            self.infer_series_view_shape(self.window_aggregate_target_expr(right)?)?
        else {
            return None;
        };
        if current_target != mean_target || current_target != std_target {
            return None;
        }

        Some(mean_view.span)
    }

    fn infer_window_aggregate_view(&self, expr: &Expr) -> Option<ResolvedWindowAggregateView> {
        let (aggregate_kind, target_expr) = match expr {
            Expr::Call { callee, args } => {
                let name = match callee.as_ref() {
                    Expr::Identifier(name) => name.as_str(),
                    Expr::Member { field, .. } => field.as_str(),
                    _ => return None,
                };
                let ResolvedSeriesCapabilityKind::WindowAggregate(aggregate_kind) =
                    classify_series_capability_name(name)?
                else {
                    return None;
                };
                let target_expr =
                    self.series_capability_target_expr(expr, callee.as_ref(), args.as_slice())?;
                (aggregate_kind, target_expr)
            }
            Expr::Member { field, object } => {
                let ResolvedSeriesCapabilityKind::WindowAggregate(aggregate_kind) =
                    classify_series_capability_name(field)?
                else {
                    return None;
                };
                (aggregate_kind, object.as_ref())
            }
            _ => return None,
        };

        let (_, ResolvedSeriesViewKind::Window(span)) =
            self.infer_series_view_shape(target_expr)?
        else {
            return None;
        };
        Some(ResolvedWindowAggregateView {
            aggregate_kind,
            span,
        })
    }

    fn infer_boundary_lookback_pair(&self, expr: &Expr) -> Option<usize> {
        let Expr::Binary { left, op, right } = expr else {
            return None;
        };
        if !matches!(op, BinaryOp::Subtract | BinaryOp::Divide) {
            return None;
        }

        let (left_target, left_view) = self.infer_series_view_shape(left)?;
        let (right_target, right_view) = self.infer_series_view_shape(right)?;
        if left_target != right_target {
            return None;
        }

        match (left_view, right_view) {
            (ResolvedSeriesViewKind::Current, ResolvedSeriesViewKind::Lookback(span)) => Some(span),
            _ => None,
        }
    }

    fn infer_balanced_smoothed_change_pair(
        &self,
        expr: &Expr,
        scope: &BTreeMap<String, BindingInfo>,
    ) -> Option<(usize, ResolvedChangeSmoothingKind)> {
        let expr = self.resolve_alias_expr(expr, scope);
        let Expr::Binary {
            left,
            op: BinaryOp::Divide,
            right,
        } = expr
        else {
            return None;
        };

        let (left_target, left_period, left_smoothing, left_kind) =
            self.infer_smoothed_change_binding(left, scope)?;
        let (right_target, right_period, right_smoothing, right_kind) =
            self.infer_smoothed_change_binding(right, scope)?;
        if left_target != right_target
            || left_period != right_period
            || left_smoothing != right_smoothing
            || left_kind != ChangeHelperKind::Gain
            || right_kind != ChangeHelperKind::Loss
        {
            return None;
        }

        Some((left_period, left_smoothing))
    }

    fn infer_smoothed_change_binding<'a>(
        &self,
        expr: &'a Expr,
        scope: &'a BTreeMap<String, BindingInfo>,
    ) -> Option<(
        &'a Expr,
        usize,
        ResolvedChangeSmoothingKind,
        ChangeHelperKind,
    )> {
        let expr = self.resolve_alias_expr(expr, scope);
        let Expr::Call { callee, args } = expr else {
            return None;
        };
        let name = match callee.as_ref() {
            Expr::Identifier(name) => name.as_str(),
            Expr::Member { field, .. } => field.as_str(),
            _ => return None,
        };
        let smoothing = classify_change_smoothing_kind(name)?;
        let change_expr = args.iter().find(|arg| arg.name.is_none())?;
        let (target, change_kind) = self.infer_change_source_call(&change_expr.value, scope)?;
        let period = args
            .iter()
            .filter(|arg| arg.name.is_none())
            .nth(1)
            .and_then(|arg| expr_number(&arg.value))
            .map(|value| value.round() as usize)?;
        if period == 0 {
            return None;
        }
        Some((target, period, smoothing, change_kind))
    }

    fn infer_change_source_call<'a>(
        &self,
        expr: &'a Expr,
        scope: &'a BTreeMap<String, BindingInfo>,
    ) -> Option<(&'a Expr, ChangeHelperKind)> {
        let expr = self.resolve_alias_expr(expr, scope);
        let Expr::Call { callee, args } = expr else {
            return None;
        };
        let name = match callee.as_ref() {
            Expr::Identifier(name) => name.as_str(),
            Expr::Member { field, .. } => field.as_str(),
            _ => return None,
        };
        let kind = match self.imported_callable_kinds.get(name).copied() {
            Some(ResolvedCallableKind::ChangeHelper(kind)) => kind,
            _ => match classify_imported_helper(name) {
                ResolvedCallableKind::ChangeHelper(kind) => kind,
                _ => return None,
            },
        };
        let target = args.iter().find(|arg| arg.name.is_none())?;
        Some((&target.value, kind))
    }

    fn resolve_alias_expr<'a>(
        &self,
        expr: &'a Expr,
        scope: &'a BTreeMap<String, BindingInfo>,
    ) -> &'a Expr {
        let mut current = expr;
        let mut seen = BTreeSet::new();
        while let Expr::Identifier(name) = current {
            let Some(binding) = scope.get(name) else {
                break;
            };
            let Some(value_expr) = binding.value_expr.as_ref() else {
                break;
            };
            if !seen.insert(name.clone()) {
                break;
            }
            current = value_expr;
        }
        current
    }

    fn infer_series_view_shape<'a>(
        &self,
        expr: &'a Expr,
    ) -> Option<(&'a Expr, ResolvedSeriesViewKind)> {
        match expr {
            Expr::Slice { object, start, end } => {
                if end.is_some() {
                    return None;
                }
                let span = start.as_deref().and_then(series_window_span)?;
                Some((object.as_ref(), ResolvedSeriesViewKind::Window(span)))
            }
            Expr::Index { object, index } => {
                Some((object.as_ref(), series_index_view_kind(index)?))
            }
            Expr::Call { callee, args } => {
                let name = match callee.as_ref() {
                    Expr::Identifier(name) => name.as_str(),
                    Expr::Member { field, .. } => field.as_str(),
                    _ => return None,
                };
                let target_expr =
                    self.series_capability_target_expr(expr, callee.as_ref(), args.as_slice())?;
                match classify_series_capability_name(name)? {
                    ResolvedSeriesCapabilityKind::Boundary(ResolvedSeriesBoundaryKind::First) => {
                        Some((target_expr, ResolvedSeriesViewKind::First))
                    }
                    ResolvedSeriesCapabilityKind::Boundary(ResolvedSeriesBoundaryKind::Last) => {
                        Some((target_expr, ResolvedSeriesViewKind::Current))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn series_capability_target_expr<'a>(
        &self,
        expr: &'a Expr,
        callee: &'a Expr,
        args: &'a [CallArg],
    ) -> Option<&'a Expr> {
        match expr {
            Expr::Call { .. } => args
                .iter()
                .find(|arg| arg.name.is_none())
                .map(|arg| &arg.value)
                .or_else(|| match callee {
                    Expr::Member { object, .. } if args.is_empty() => Some(object.as_ref()),
                    _ => None,
                }),
            Expr::Member { object, .. } => Some(object.as_ref()),
            _ => None,
        }
    }

    fn window_aggregate_target_expr<'a>(&self, expr: &'a Expr) -> Option<&'a Expr> {
        match expr {
            Expr::Call { callee, args } => {
                let ResolvedSeriesCapabilityKind::WindowAggregate(_) =
                    classify_series_capability_name(match callee.as_ref() {
                        Expr::Identifier(name) => name.as_str(),
                        Expr::Member { field, .. } => field.as_str(),
                        _ => return None,
                    })?
                else {
                    return None;
                };
                self.series_capability_target_expr(expr, callee.as_ref(), args.as_slice())
            }
            Expr::Member { field, object } => {
                let ResolvedSeriesCapabilityKind::WindowAggregate(_) =
                    classify_series_capability_name(field)?
                else {
                    return None;
                };
                Some(object.as_ref())
            }
            _ => None,
        }
    }

    fn is_ema_like_name(&self, name: &str) -> bool {
        matches!(
            self.imported_callable_kinds.get(name).copied(),
            Some(ResolvedCallableKind::IndicatorHelper(
                KnownIndicatorHelperKind::MovingAverage(MovingAverageHelperKind::Ema)
            ))
        ) || matches!(
            classify_imported_helper(name),
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::MovingAverage(
                MovingAverageHelperKind::Ema
            ))
        )
    }

    fn make_expr(&mut self, kind: HirExprKind, ty: TypeId, expr: &Expr) -> HirExpr {
        HirExpr {
            expr_id: self.alloc_expr_id(),
            kind,
            ty,
            span: Span::expr(self.expr_label(expr)),
        }
    }

    fn expr_label(&self, expr: &Expr) -> String {
        match expr {
            Expr::Raw(value) => format!("raw {value}"),
            Expr::Identifier(name) => name.clone(),
            Expr::Number(value) => value.to_string(),
            Expr::String(value) => value.clone(),
            Expr::Bool(value) => value.to_string(),
            Expr::List(_) => "list".into(),
            Expr::Call { .. } => "call".into(),
            Expr::Member { field, .. } => format!("member .{field}"),
            Expr::Index { .. } => "index".into(),
            Expr::Slice { .. } => "slice".into(),
            Expr::Unary { .. } => "unary".into(),
            Expr::Binary { .. } => "binary".into(),
            Expr::Range { .. } => "range".into(),
            Expr::Await(_) => "await".into(),
            Expr::Try(_) => "try".into(),
        }
    }

    fn alloc_def_id(&mut self) -> DefId {
        let id = self.next_def_id;
        self.next_def_id += 1;
        DefId(id)
    }

    fn alloc_expr_id(&mut self) -> ExprId {
        let id = self.next_expr_id;
        self.next_expr_id += 1;
        ExprId(id)
    }

    fn callable_target(&self, name: &str) -> Option<CallableTarget> {
        match name {
            "fetch" | "get_data" => Some(CallableTarget::FetchLike),
            "abs" | "avg" | "first" | "last" | "max" | "mean" | "min" | "pow" | "sqrt" | "std"
            | "stddev" | "sum" | "variance" => Some(CallableTarget::Builtin),
            _ => self
                .function_signatures
                .get(name)
                .map(|signature| CallableTarget::UserFunction(signature.return_type))
                .or_else(|| {
                    self.imported_callables
                        .contains(name)
                        .then_some(CallableTarget::Imported)
                })
                .or_else(|| is_known_helper_function(name).then_some(CallableTarget::Imported)),
        }
    }

    fn call_target_return_type(&mut self, name: &str) -> TypeId {
        match self.callable_target(name) {
            Some(CallableTarget::FetchLike) => {
                let number = self.types.number();
                let series = self.types.series(number);
                self.types.maybe(series)
            }
            Some(CallableTarget::Builtin) | Some(CallableTarget::Imported) => self.types.unknown(),
            Some(CallableTarget::UserFunction(return_type)) => return_type,
            None => self.types.unknown(),
        }
    }
}

fn expr_number(expr: &Expr) -> Option<f64> {
    match expr {
        Expr::Number(value) => Some(*value),
        Expr::Unary {
            op: UnaryOp::Negate,
            expr,
        } => expr_number(expr).map(|value| -value),
        _ => None,
    }
}

fn expr_integer(expr: &Expr) -> Option<isize> {
    expr_number(expr).map(|value| value.round() as isize)
}

fn series_index_view_kind(expr: &Expr) -> Option<ResolvedSeriesViewKind> {
    match expr_integer(expr)? {
        0 => Some(ResolvedSeriesViewKind::Current),
        value if value > 0 => Some(ResolvedSeriesViewKind::Lookback(value as usize)),
        _ => None,
    }
}

fn series_window_span(expr: &Expr) -> Option<usize> {
    match expr_integer(expr)? {
        value if value > 0 => Some(value as usize),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MemberCapabilityUse {
    Access,
    Call,
}

fn build_resolved_functions(
    module: &ScriptModule,
    signatures: &BTreeMap<String, FunctionSignature>,
) -> BTreeMap<String, ResolvedFunction> {
    module
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Function(function) => Some((
                function.name.clone(),
                ResolvedFunction {
                    name: function.name.clone(),
                    callable_kind: ResolvedCallableKind::UserFunction,
                    param_names: function
                        .params
                        .iter()
                        .map(|param| param.name.clone())
                        .collect(),
                    body: function.body.clone(),
                    return_type: signatures
                        .get(&function.name)
                        .map(|signature| signature.return_type)
                        .unwrap_or(TypeId(0)),
                    return_expr: first_return_expr(function),
                    returned_list_target: first_return_expr(function)
                        .as_ref()
                        .and_then(|expr| returned_list_target(function, expr)),
                },
            )),
            _ => None,
        })
        .collect()
}

fn build_resolved_callables(
    module: &ScriptModule,
    signatures: &BTreeMap<String, FunctionSignature>,
) -> BTreeMap<String, ResolvedCallable> {
    let mut callables = BTreeMap::new();

    for (name, kind, smoothing_kind, fetch_source_kind) in [
        (
            "fetch",
            ResolvedCallableKind::FetchLike,
            None,
            Some(ResolvedFetchSourceKind::KlineSeries),
        ),
        (
            "get_data",
            ResolvedCallableKind::FetchLike,
            None,
            Some(ResolvedFetchSourceKind::KlineSeries),
        ),
        ("abs", ResolvedCallableKind::BuiltinMath, None, None),
        ("avg", ResolvedCallableKind::BuiltinMath, None, None),
        ("first", ResolvedCallableKind::BuiltinMath, None, None),
        ("last", ResolvedCallableKind::BuiltinMath, None, None),
        ("max", ResolvedCallableKind::BuiltinMath, None, None),
        ("mean", ResolvedCallableKind::BuiltinMath, None, None),
        ("min", ResolvedCallableKind::BuiltinMath, None, None),
        ("pow", ResolvedCallableKind::BuiltinMath, None, None),
        ("sqrt", ResolvedCallableKind::BuiltinMath, None, None),
        ("std", ResolvedCallableKind::BuiltinMath, None, None),
        ("stddev", ResolvedCallableKind::BuiltinMath, None, None),
        ("sum", ResolvedCallableKind::BuiltinMath, None, None),
        ("variance", ResolvedCallableKind::BuiltinMath, None, None),
        (
            "sma",
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::MovingAverage(
                MovingAverageHelperKind::Sma,
            )),
            Some(ResolvedChangeSmoothingKind::Simple),
            None,
        ),
        (
            "ema",
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::MovingAverage(
                MovingAverageHelperKind::Ema,
            )),
            Some(ResolvedChangeSmoothingKind::Ema),
            None,
        ),
        (
            "rsi",
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Rsi(
                RsiHelperKind::Wilder,
            )),
            None,
            None,
        ),
        (
            "macd",
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Macd),
            None,
            None,
        ),
        (
            "momentum",
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Momentum),
            None,
            None,
        ),
        (
            "zscore",
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::ZScore),
            None,
            None,
        ),
        (
            "z_score",
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::ZScore),
            None,
            None,
        ),
        (
            "symbols",
            ResolvedCallableKind::UniverseHelper(KnownUniverseHelperKind::Symbols),
            None,
            None,
        ),
        (
            "universe",
            ResolvedCallableKind::UniverseHelper(KnownUniverseHelperKind::Universe),
            None,
            None,
        ),
        (
            "filter",
            ResolvedCallableKind::UniverseHelper(KnownUniverseHelperKind::Filter),
            None,
            None,
        ),
        (
            "sort_by",
            ResolvedCallableKind::UniverseHelper(KnownUniverseHelperKind::SortBy),
            None,
            None,
        ),
        (
            "top",
            ResolvedCallableKind::UniverseHelper(KnownUniverseHelperKind::Top),
            None,
            None,
        ),
        ("equal_weight", ResolvedCallableKind::Imported, None, None),
        ("fixed_weights", ResolvedCallableKind::Imported, None, None),
        ("rank_weight", ResolvedCallableKind::Imported, None, None),
        ("score_weight", ResolvedCallableKind::Imported, None, None),
        ("rebalance", ResolvedCallableKind::Imported, None, None),
        (
            "rma",
            ResolvedCallableKind::Imported,
            Some(ResolvedChangeSmoothingKind::Wilder),
            None,
        ),
        (
            "wilders",
            ResolvedCallableKind::Imported,
            Some(ResolvedChangeSmoothingKind::Wilder),
            None,
        ),
        (
            "smma",
            ResolvedCallableKind::Imported,
            Some(ResolvedChangeSmoothingKind::Wilder),
            None,
        ),
        ("field", ResolvedCallableKind::Imported, None, None),
        ("resample", ResolvedCallableKind::Imported, None, None),
        ("align", ResolvedCallableKind::Imported, None, None),
        ("align_asof", ResolvedCallableKind::Imported, None, None),
        ("spread", ResolvedCallableKind::Imported, None, None),
        (
            "gains",
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Gain),
            None,
            None,
        ),
        (
            "gain",
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Gain),
            None,
            None,
        ),
        (
            "up_moves",
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Gain),
            None,
            None,
        ),
        (
            "positive_changes",
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Gain),
            None,
            None,
        ),
        (
            "positive_deltas",
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Gain),
            None,
            None,
        ),
        (
            "losses",
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Loss),
            None,
            None,
        ),
        (
            "loss",
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Loss),
            None,
            None,
        ),
        (
            "down_moves",
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Loss),
            None,
            None,
        ),
        (
            "negative_changes",
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Loss),
            None,
            None,
        ),
        (
            "negative_deltas",
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Loss),
            None,
            None,
        ),
    ] {
        seed_callable(
            &mut callables,
            name,
            kind,
            smoothing_kind,
            fetch_source_kind,
            TypeId(0),
        );
    }

    for item in &module.items {
        if let Item::Import(import_decl) = item {
            if let Some(names) = &import_decl.names {
                for name in names {
                    let local_name = name.alias.clone().unwrap_or_else(|| name.name.clone());
                    callables
                        .entry(local_name.clone())
                        .or_insert(ResolvedCallable {
                            name: local_name,
                            kind: classify_imported_helper(&name.name),
                            change_smoothing_kind: classify_change_smoothing_kind(&name.name),
                            fetch_source_kind: classify_fetch_source_kind(&name.name),
                            return_type: TypeId(0),
                        });
                }
            }
        }
    }

    for (name, signature) in signatures {
        callables.insert(
            name.clone(),
            ResolvedCallable {
                name: name.clone(),
                kind: ResolvedCallableKind::UserFunction,
                change_smoothing_kind: None,
                fetch_source_kind: None,
                return_type: signature.return_type,
            },
        );
    }

    callables
}

fn seed_callable(
    callables: &mut BTreeMap<String, ResolvedCallable>,
    name: &str,
    kind: ResolvedCallableKind,
    change_smoothing_kind: Option<ResolvedChangeSmoothingKind>,
    fetch_source_kind: Option<ResolvedFetchSourceKind>,
    return_type: TypeId,
) {
    callables
        .entry(name.to_string())
        .or_insert(ResolvedCallable {
            name: name.to_string(),
            kind,
            change_smoothing_kind,
            fetch_source_kind,
            return_type,
        });
}

fn classify_imported_helper(name: &str) -> ResolvedCallableKind {
    match name {
        "fetch" | "get_data" => ResolvedCallableKind::FetchLike,
        "sma" => ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::MovingAverage(
            MovingAverageHelperKind::Sma,
        )),
        "ema" => ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::MovingAverage(
            MovingAverageHelperKind::Ema,
        )),
        "rsi" => ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Rsi(
            RsiHelperKind::Wilder,
        )),
        "macd" => ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Macd),
        "momentum" => ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Momentum),
        "atr" => ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Atr),
        "bollinger" | "bb" => {
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::BollingerBands)
        }
        "obv" => ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Obv),
        "cmf" => ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Cmf),
        "adx" => ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Adx),
        "stoch" | "stochastic" => {
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Stochastic)
        }
        "cci" => ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Cci),
        "psar" | "parabolic_sar" => {
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::ParabolicSar)
        }
        "keltner" => {
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::KeltnerChannel)
        }
        "donchian" => {
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::DonchianChannel)
        }
        "zscore" | "z_score" => {
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::ZScore)
        }
        "symbols" => ResolvedCallableKind::UniverseHelper(KnownUniverseHelperKind::Symbols),
        "universe" => ResolvedCallableKind::UniverseHelper(KnownUniverseHelperKind::Universe),
        "filter" => ResolvedCallableKind::UniverseHelper(KnownUniverseHelperKind::Filter),
        "sort_by" => ResolvedCallableKind::UniverseHelper(KnownUniverseHelperKind::SortBy),
        "top" => ResolvedCallableKind::UniverseHelper(KnownUniverseHelperKind::Top),
        "gains" | "gain" | "up_moves" | "positive_changes" | "positive_deltas" => {
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Gain)
        }
        "losses" | "loss" | "down_moves" | "negative_changes" | "negative_deltas" => {
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Loss)
        }
        "field" | "resample" | "align" | "align_asof" | "spread" => ResolvedCallableKind::Imported,
        "abs" | "avg" | "first" | "last" | "max" | "mean" | "min" | "pow" | "sqrt" | "std"
        | "stddev" | "sum" | "variance" => ResolvedCallableKind::BuiltinMath,
        _ => ResolvedCallableKind::Imported,
    }
}

fn classify_change_smoothing_kind(name: &str) -> Option<ResolvedChangeSmoothingKind> {
    match name {
        "rma" | "wilders" | "smma" => Some(ResolvedChangeSmoothingKind::Wilder),
        "ema" => Some(ResolvedChangeSmoothingKind::Ema),
        "sma" => Some(ResolvedChangeSmoothingKind::Simple),
        _ => None,
    }
}

fn classify_fetch_source_kind(name: &str) -> Option<ResolvedFetchSourceKind> {
    match name {
        "fetch" | "get_data" => Some(ResolvedFetchSourceKind::KlineSeries),
        _ => None,
    }
}

fn first_return_expr(function: &FunctionDecl) -> Option<Expr> {
    function.body.iter().find_map(|stmt| match stmt {
        Stmt::Return(Some(expr)) => Some(expr.clone()),
        _ => None,
    })
}

fn returned_list_target(function: &FunctionDecl, return_expr: &Expr) -> Option<String> {
    match return_expr {
        Expr::Identifier(name) => Some(name.clone()),
        Expr::List(items) if items.is_empty() => {
            let candidates = function
                .body
                .iter()
                .filter_map(|stmt| match stmt {
                    Stmt::Let {
                        pattern,
                        value: Expr::List(items),
                        mutable: true,
                        ..
                    } if items.is_empty() => Some(pattern.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            if candidates.len() == 1 {
                candidates.into_iter().next()
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_known_helper_function(name: &str) -> bool {
    matches!(
        name,
        "sma"
            | "ema"
            | "rma"
            | "wilders"
            | "smma"
            | "rsi"
            | "macd"
            | "momentum"
            | "zscore"
            | "gains"
            | "gain"
            | "losses"
            | "loss"
            | "up_moves"
            | "down_moves"
            | "positive_changes"
            | "negative_changes"
            | "positive_deltas"
            | "negative_deltas"
            | "field"
            | "resample"
            | "align"
            | "align_asof"
            | "spread"
            | "symbols"
            | "universe"
            | "filter"
            | "sort_by"
            | "top"
            | "equal_weight"
            | "fixed_weights"
            | "rank_weight"
            | "score_weight"
            | "rebalance"
            | "first"
            | "last"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
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
