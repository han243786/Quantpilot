use super::*;

impl Resolver {
    pub(super) fn infer_expr_semantic(
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

    pub(super) fn infer_manual_indicator_formula(
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

    pub(super) fn infer_manual_macd_histogram_formula(
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

    pub(super) fn infer_manual_macd_line_formula(
        &self,
        expr: &Expr,
        scope: &BTreeMap<String, BindingInfo>,
    ) -> Option<(usize, usize)> {
        let (_, fast_period, slow_period) = self.infer_manual_macd_line_shape(expr, scope)?;
        Some((fast_period, slow_period))
    }

    pub(super) fn infer_manual_macd_signal_formula(
        &self,
        expr: &Expr,
        scope: &BTreeMap<String, BindingInfo>,
    ) -> Option<(usize, usize, usize)> {
        let (_, fast_period, slow_period, signal_period) =
            self.infer_signal_line_shape(expr, scope)?;
        Some((fast_period, slow_period, signal_period))
    }

    pub(super) fn infer_manual_macd_line_shape<'a>(
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

    pub(super) fn infer_signal_line_shape<'a>(
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

    pub(super) fn infer_ema_call<'a>(
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

    pub(super) fn infer_manual_moving_average_formula(
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

    pub(super) fn infer_manual_zscore_formula(
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

    pub(super) fn infer_window_aggregate_view(
        &self,
        expr: &Expr,
    ) -> Option<ResolvedWindowAggregateView> {
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

    pub(super) fn infer_boundary_lookback_pair(&self, expr: &Expr) -> Option<usize> {
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

    pub(super) fn infer_balanced_smoothed_change_pair(
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

    pub(super) fn infer_smoothed_change_binding<'a>(
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

    pub(super) fn infer_change_source_call<'a>(
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

    pub(super) fn resolve_alias_expr<'a>(
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

    pub(super) fn infer_series_view_shape<'a>(
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

    pub(super) fn series_capability_target_expr<'a>(
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

    pub(super) fn window_aggregate_target_expr<'a>(&self, expr: &'a Expr) -> Option<&'a Expr> {
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

    pub(super) fn is_ema_like_name(&self, name: &str) -> bool {
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
}
