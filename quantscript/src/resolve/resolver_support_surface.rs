use super::*;

impl Resolver {
    pub(super) fn make_expr(&mut self, kind: HirExprKind, ty: TypeId, expr: &Expr) -> HirExpr {
        HirExpr {
            expr_id: self.alloc_expr_id(),
            kind,
            ty,
            span: Span::expr(self.expr_label(expr)),
        }
    }

    pub(super) fn expr_label(&self, expr: &Expr) -> String {
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

    pub(super) fn alloc_def_id(&mut self) -> DefId {
        let id = self.next_def_id;
        self.next_def_id += 1;
        DefId(id)
    }

    pub(super) fn alloc_expr_id(&mut self) -> ExprId {
        let id = self.next_expr_id;
        self.next_expr_id += 1;
        ExprId(id)
    }

    pub(super) fn callable_target(&self, name: &str) -> Option<CallableTarget> {
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

    pub(super) fn call_target_return_type(&mut self, name: &str) -> TypeId {
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

pub(super) fn expr_number(expr: &Expr) -> Option<f64> {
    match expr {
        Expr::Number(value) => Some(*value),
        Expr::Unary {
            op: UnaryOp::Negate,
            expr,
        } => expr_number(expr).map(|value| -value),
        _ => None,
    }
}

pub(super) fn expr_integer(expr: &Expr) -> Option<isize> {
    expr_number(expr).map(|value| value.round() as isize)
}

pub(super) fn series_index_view_kind(expr: &Expr) -> Option<ResolvedSeriesViewKind> {
    match expr_integer(expr)? {
        0 => Some(ResolvedSeriesViewKind::Current),
        value if value > 0 => Some(ResolvedSeriesViewKind::Lookback(value as usize)),
        _ => None,
    }
}

pub(super) fn series_window_span(expr: &Expr) -> Option<usize> {
    match expr_integer(expr)? {
        value if value > 0 => Some(value as usize),
        _ => None,
    }
}

pub(super) fn build_resolved_functions(
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

pub(super) fn first_return_expr(function: &FunctionDecl) -> Option<Expr> {
    function.body.iter().find_map(|stmt| match stmt {
        Stmt::Return(Some(expr)) => Some(expr.clone()),
        _ => None,
    })
}

pub(super) fn returned_list_target(function: &FunctionDecl, return_expr: &Expr) -> Option<String> {
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
