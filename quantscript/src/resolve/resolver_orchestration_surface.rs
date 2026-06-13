use super::*;

impl Resolver {
    pub(super) fn resolve_module(mut self, module: &ScriptModule) -> ResolveResult {
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

    pub(super) fn seed_imported_callables(&mut self, module: &ScriptModule) {
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

    pub(super) fn seed_function_signatures(&mut self, module: &ScriptModule) {
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

    pub(super) fn resolve_function(&mut self, function: &FunctionDecl) -> HirFunction {
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

    pub(super) fn lower_block(
        &mut self,
        stmts: &[Stmt],
        scope: &mut BTreeMap<String, BindingInfo>,
    ) -> Vec<HirStmt> {
        stmts
            .iter()
            .map(|stmt| self.lower_stmt(stmt, scope))
            .collect()
    }

    pub(super) fn lower_stmt(
        &mut self,
        stmt: &Stmt,
        scope: &mut BTreeMap<String, BindingInfo>,
    ) -> HirStmt {
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

    pub(super) fn lower_match_arm(
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

    pub(super) fn lower_expr(
        &mut self,
        expr: &Expr,
        scope: &BTreeMap<String, BindingInfo>,
    ) -> HirExpr {
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

    pub(super) fn lower_callee_expr(
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
}
