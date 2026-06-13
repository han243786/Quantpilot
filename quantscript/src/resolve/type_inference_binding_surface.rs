use super::*;

impl Resolver {
    pub(super) fn infer_call_type(&mut self, callee: &HirExpr, args: &[HirCallArg]) -> TypeId {
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

    pub(super) fn infer_member_capability_type(
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

    pub(super) fn infer_named_helper_return_type(
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

    pub(super) fn infer_unary_type(&mut self, op: &UnaryOp, inner_ty: TypeId) -> TypeId {
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

    pub(super) fn infer_binary_type(
        &mut self,
        op: &BinaryOp,
        left_ty: TypeId,
        right_ty: TypeId,
    ) -> TypeId {
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

    pub(super) fn common_item_type(&mut self, items: &[HirExpr]) -> TypeId {
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

    pub(super) fn index_result_type(&mut self, ty: TypeId) -> TypeId {
        match self.types.get(ty).clone() {
            Type::List(inner) | Type::Series(inner) | Type::Scalar(inner) | Type::Maybe(inner) => {
                self.types.intern(*inner)
            }
            Type::Universe => self.types.symbol(),
            _ => self.types.unknown(),
        }
    }

    pub(super) fn sequence_item_type(&mut self, ty: TypeId) -> TypeId {
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

    pub(super) fn numeric_sequence_type(&mut self, ty: TypeId) -> TypeId {
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

    pub(super) fn iteration_item_type(&mut self, ty: TypeId) -> TypeId {
        match self.types.get(ty).clone() {
            Type::List(inner) | Type::Series(inner) => self.types.intern(*inner),
            Type::Universe => self.types.symbol(),
            _ => self.types.unknown(),
        }
    }

    pub(super) fn unwrap_maybe(&mut self, ty: TypeId) -> TypeId {
        match self.types.get(ty).clone() {
            Type::Maybe(inner) => self.types.intern(*inner),
            _ => ty,
        }
    }

    pub(super) fn is_numeric(&self, ty: TypeId) -> bool {
        match self.types.get(ty) {
            Type::Number => true,
            Type::Scalar(inner) => matches!(**inner, Type::Number),
            _ => false,
        }
    }

    pub(super) fn insert_binding(
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

    pub(super) fn validate_condition_type(&mut self, condition: &HirExpr, label: &str) {
        if matches!(self.types.get(condition.ty), Type::Bool | Type::Unknown) {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            "QS0006",
            format!("{label} 条件必须解析为 Bool 类型"),
            Some(condition.span.clone()),
        ));
    }

    pub(super) fn resolve_type(&mut self, annotation: &str, span: Span) -> TypeId {
        match parse_type_annotation(annotation) {
            Ok(ty) => self.types.intern(ty),
            Err(message) => {
                self.diagnostics
                    .push(Diagnostic::error("QS0003", message, Some(span)));
                self.types.unknown()
            }
        }
    }
}
