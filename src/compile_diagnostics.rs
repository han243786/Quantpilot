use super::*;

pub(super) fn compile_diagnostic_from_script_diagnostic(
    diagnostic: &quantscript::Diagnostic,
) -> CompileDiagnostic {
    CompileDiagnostic {
        code: diagnostic.code.to_string(),
        severity: match diagnostic.severity {
            quantscript::DiagnosticSeverity::Error => CompileDiagnosticSeverity::Error,
            quantscript::DiagnosticSeverity::Warning => CompileDiagnosticSeverity::Warning,
        },
        message: diagnostic.message.clone(),
        span_label: diagnostic.span.as_ref().map(|span| span.label.clone()),
        target: None,
        hint: None,
    }
}

pub(super) fn api_error_detail_from_script_diagnostic(
    diagnostic: &quantscript::Diagnostic,
) -> ApiErrorDetail {
    ApiErrorDetail {
        code: diagnostic.code.to_string(),
        target: None,
        message: diagnostic.message.clone(),
        span_label: diagnostic.span.as_ref().map(|span| span.label.clone()),
        reason: None,
    }
}

pub(super) fn compile_diagnostic_from_strategy_ir_error(
    code: impl Into<String>,
    message: impl Into<String>,
    target: Option<CompileDiagnosticTarget>,
    hint: Option<String>,
) -> CompileDiagnostic {
    CompileDiagnostic {
        code: code.into(),
        severity: CompileDiagnosticSeverity::Error,
        message: message.into(),
        span_label: None,
        target,
        hint,
    }
}

pub(super) fn api_error_detail_from_compile_diagnostic(
    diagnostic: &CompileDiagnostic,
) -> ApiErrorDetail {
    ApiErrorDetail {
        code: diagnostic.code.clone(),
        target: diagnostic.target.as_ref().and_then(|target| {
            if let (Some(label), Some(field)) = (&target.label, &target.field) {
                return Some(format!("{label}.{field}"));
            }
            target
                .node_id
                .clone()
                .or_else(|| target.field.clone())
                .or_else(|| target.label.clone())
        }),
        message: diagnostic.message.clone(),
        span_label: diagnostic.span_label.clone(),
        reason: diagnostic.hint.clone(),
    }
}

pub(super) fn strategy_ir_signal_target(
    signal_id: Option<&str>,
    field: Option<&str>,
) -> CompileDiagnosticTarget {
    CompileDiagnosticTarget {
        scope: CompileDiagnosticTargetScope::Graph,
        node_id: None,
        edge_id: None,
        field: field.map(str::to_string),
        label: signal_id.map(str::to_string),
    }
}

pub(super) fn strategy_ir_diagnostic_from_validation_message(message: &str) -> CompileDiagnostic {
    let field = message
        .split_whitespace()
        .next()
        .filter(|item| item.contains('.'))
        .map(str::to_string);
    compile_diagnostic_from_strategy_ir_error(
        "QPSTRAT001",
        message,
        Some(CompileDiagnosticTarget {
            scope: CompileDiagnosticTargetScope::Graph,
            node_id: None,
            edge_id: None,
            field,
            label: None,
        }),
        None,
    )
}

pub(super) fn strategy_ir_diagnostic_from_lowering_error(message: &str) -> CompileDiagnostic {
    let code = message
        .split_whitespace()
        .next()
        .filter(|token| token.starts_with("CUSTOM") || token.starts_with("QPSTRATSPREAD"))
        .unwrap_or("QPSTRAT002");
    let signal_id = message
        .split("signal `")
        .nth(1)
        .and_then(|rest| rest.split('`').next());
    let (field, hint) = match code {
        "CUSTOM001" | "CUSTOM002" | "CUSTOM003" | "CUSTOM004" | "CUSTOM005" | "CUSTOM006"
        | "CUSTOM007" | "CUSTOM008" | "CUSTOM009" | "CUSTOM010" | "CUSTOM011"
        | "CUSTOM012" => (
            Some("params.custom_expr"),
            Some(
                "Custom indicators are restricted to the admitted expression subset and must lower into Core IR."
                    .to_string(),
            ),
        ),
        "QPSTRATSPREAD001" => (
            Some("params.spread_output_code"),
            Some(
                "Use `spread_output_code = 1` so Strategy IR matches the current graph/runtime spread threshold slice."
                    .to_string(),
            ),
        ),
        "QPSTRATSPREAD002" => (
            Some("params.max_time_diff_ms"),
            Some(
                "Set `max_time_diff_ms` to a positive value for the current Strategy IR spread threshold slice."
                    .to_string(),
            ),
        ),
        "QPSTRATSPREAD003" => (
            Some("condition"),
            Some(
                "Use a one-sided buy threshold such as `spread_signal > 5` or `spread_signal >= 5`."
                    .to_string(),
            ),
        ),
        "QPSTRATSPREAD004" => (
            Some("indicator.inputs"),
            Some(
                "Provide exactly two spread inputs so Strategy IR matches the current graph/runtime spread threshold slice."
                    .to_string(),
            ),
        ),
        _ => (None, None),
    };

    compile_diagnostic_from_strategy_ir_error(
        code,
        message,
        Some(strategy_ir_signal_target(signal_id, field)),
        hint,
    )
}

pub(super) fn formal_quantscript_diagnostic_from_lowering_error(
    message: &str,
) -> CompileDiagnostic {
    let code = message
        .split_whitespace()
        .next()
        .filter(|token| token.starts_with("QPQSLOW"))
        .unwrap_or("QPQSLOW999");
    let hint = match code {
        "QPQSLOW001" => Some(
            "Rewrite the conditional emit so it lowers to a supported indicator or spread intent, or keep the emit unconditional."
                .to_string(),
        ),
        "QPQSLOW002" => Some(
            "Keep at least one executable emit Intent(...) reachable from strategy, and make sure its condition lowers to a supported runtime intent shape."
                .to_string(),
        ),
        "QPQSLOW003" | "QPQSLOW007" => Some(
            "Add at least one fetch(...) or get_data(...) call that remains reachable from strategy lowering."
                .to_string(),
        ),
        "QPQSLOW004" => Some(
            "Use a supported runtime action such as BUY or SELL in emit Intent(...).".to_string(),
        ),
        "QPQSLOW005" => Some(
            "Provide a non-empty action argument to emit Intent(...).".to_string(),
        ),
        "QPQSLOW006" => Some(
            "Declare a top-level fn strategy() { ... } entrypoint for formal QuantScript."
                .to_string(),
        ),
        "QPQSLOW008" => Some(
            "Keep at most one rebalance(...) directive in formal QuantScript strategy()."
                .to_string(),
        ),
        "QPQSLOW009" => Some(
            "Use only supported rebalance(..., every=...) values such as \"1d\", \"slow\", or \"weekly\"."
                .to_string(),
        ),
        "QPQSLOW010" => Some(
            "Provide a compile-time universe_snapshot when using snapshot-dependent universe/filter/sort operations."
                .to_string(),
        ),
        "QPQSLOW011" => Some(
            "Use a supported sort_by key such as symbol, market_cap, volume_24h, or listing_age_days."
                .to_string(),
        ),
        "QPQSLOW012" => Some(
            "Use a supported sort order such as asc or desc.".to_string(),
        ),
        "QPQSLOW013" => Some(
            "Pass a supported allocation helper such as equal_weight(...), fixed_weights(...), rank_weight(...), or score_weight(...) into rebalance(...)."
                .to_string(),
        ),
        "QPQSLOW014" => Some(
            "Pass a universe expression or universe binding into the rebalance allocation helper."
                .to_string(),
        ),
        "QPQSLOW015" => Some(
            "Make sure the rebalance allocation resolves to at least one selected symbol."
                .to_string(),
        ),
        "QPQSLOW016" => Some(
            "Provide exactly one fixed weight per selected symbol.".to_string(),
        ),
        "QPQSLOW017" => Some(
            "Use only non-negative fixed weights.".to_string(),
        ),
        "QPQSLOW018" => Some(
            "Make the fixed weight total greater than zero.".to_string(),
        ),
        "QPQSLOW019" => Some(
            "Use a supported rank_weight method such as linear or inverse_rank.".to_string(),
        ),
        "QPQSLOW020" => Some(
            "Use the supported score_weight normalize mode \"sum\".".to_string(),
        ),
        "QPQSLOW021" => Some(
            "Provide weights as a numeric list literal.".to_string(),
        ),
        "QPQSLOW022" => Some(
            "Pass a fetch(...) or get_data(...) series as the first argument to indicator helpers such as rsi, macd, momentum, or zscore."
                .to_string(),
        ),
        "QPQSLOW023" => Some(
            "Use indicator periods, lookbacks, and windows greater than zero.".to_string(),
        ),
        "QPQSLOW024" => Some(
            "Pass a fetch/get_data series into moving-average helpers, or for ema(...) pass a recognized MACD line."
                .to_string(),
        ),
        "QPQSLOW025" => Some(
            "Pass a universe-valued expression such as symbols(...), universe(...), filter(...), sort_by(...), or top(...) into universe helpers."
                .to_string(),
        ),
        "QPQSLOW026" => Some(
            "Pass a list literal into symbols(...), for example symbols([\"BTCUSDT\", \"ETHUSDT\"])."
                .to_string(),
        ),
        "QPQSLOW027" => Some(
            "Use only string literals inside symbols([...]).".to_string(),
        ),
        "QPQSLOW028" => Some(
            "Pass a numeric count as the second positional argument to top(...), for example top(sort_by(...), 10)."
                .to_string(),
        ),
        _ => None,
    };
    compile_diagnostic_from_strategy_ir_error(
        code,
        message,
        Some(CompileDiagnosticTarget {
            scope: CompileDiagnosticTargetScope::Graph,
            node_id: None,
            edge_id: None,
            field: Some("formal_quantscript".to_string()),
            label: None,
        }),
        hint,
    )
}

pub(super) fn collect_formal_quantscript_pre_lowering_diagnostics(
    module: &quantscript::ScriptModule,
) -> Vec<CompileDiagnostic> {
    let mut diagnostics = Vec::new();
    let Some(strategy) = module.items.iter().find_map(|item| match item {
        quantscript::Item::Function(function) if function.name == "strategy" => Some(function),
        _ => None,
    }) else {
        return diagnostics;
    };

    collect_pre_lowering_emit_diagnostics_from_stmts(&strategy.body, &mut diagnostics);
    if contains_emit_intent_in_stmts(&strategy.body)
        && !contains_fetch_like_call_in_stmts(&strategy.body)
    {
        diagnostics.push(formal_quantscript_diagnostic_from_lowering_error(
            "QPQSLOW007 strategy lowering requires at least one fetch/get_data call",
        ));
    }
    diagnostics
}

pub(super) fn collect_pre_lowering_emit_diagnostics_from_stmts(
    stmts: &[quantscript::Stmt],
    diagnostics: &mut Vec<CompileDiagnostic>,
) {
    for stmt in stmts {
        match stmt {
            quantscript::Stmt::EmitIntent { args } => {
                collect_pre_lowering_emit_diagnostics_from_args(args, diagnostics);
            }
            quantscript::Stmt::If {
                then_branch,
                else_if_branches,
                else_branch,
                ..
            } => {
                collect_pre_lowering_emit_diagnostics_from_stmts(then_branch, diagnostics);
                for (_, branch) in else_if_branches {
                    collect_pre_lowering_emit_diagnostics_from_stmts(branch, diagnostics);
                }
                if let Some(branch) = else_branch {
                    collect_pre_lowering_emit_diagnostics_from_stmts(branch, diagnostics);
                }
            }
            quantscript::Stmt::For { body, .. } | quantscript::Stmt::While { body, .. } => {
                collect_pre_lowering_emit_diagnostics_from_stmts(body, diagnostics);
            }
            quantscript::Stmt::Match { arms, .. } => {
                for arm in arms {
                    if let quantscript::MatchArmBody::Statement(stmt) = &arm.body {
                        collect_pre_lowering_emit_diagnostics_from_stmts(
                            std::slice::from_ref(stmt.as_ref()),
                            diagnostics,
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

pub(super) fn collect_pre_lowering_emit_diagnostics_from_args(
    args: &[quantscript::CallArg],
    diagnostics: &mut Vec<CompileDiagnostic>,
) {
    let action = emit_action_arg_value(args).unwrap_or_default();
    if action.is_empty() {
        diagnostics.push(formal_quantscript_diagnostic_from_lowering_error(
            "QPQSLOW005 emit Intent requires action",
        ));
        return;
    }

    let normalized = action.to_ascii_uppercase();
    if normalized != "BUY" && normalized != "SELL" {
        diagnostics.push(formal_quantscript_diagnostic_from_lowering_error(&format!(
            "QPQSLOW004 unsupported Intent action for runtime lowering: {normalized}"
        )));
    }
}

pub(super) fn emit_action_arg_value(args: &[quantscript::CallArg]) -> Option<String> {
    for arg in args {
        if arg.name.as_deref() == Some("action") {
            return expr_string_literal_value(&arg.value);
        }
    }

    args.first()
        .and_then(|arg| expr_string_literal_value(&arg.value))
}

pub(super) fn expr_string_literal_value(expr: &quantscript::Expr) -> Option<String> {
    match expr {
        quantscript::Expr::String(value) => Some(value.clone()),
        _ => None,
    }
}

pub(super) fn contains_fetch_like_call_in_stmts(stmts: &[quantscript::Stmt]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        quantscript::Stmt::Let { value, .. } | quantscript::Stmt::Expr(value) => {
            contains_fetch_like_call_in_expr(value)
        }
        quantscript::Stmt::Return(Some(value)) => contains_fetch_like_call_in_expr(value),
        quantscript::Stmt::Return(None) => false,
        quantscript::Stmt::EmitIntent { args } => args
            .iter()
            .any(|arg| contains_fetch_like_call_in_expr(&arg.value)),
        quantscript::Stmt::If {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
        } => {
            contains_fetch_like_call_in_expr(condition)
                || contains_fetch_like_call_in_stmts(then_branch)
                || else_if_branches.iter().any(|(condition, branch)| {
                    contains_fetch_like_call_in_expr(condition)
                        || contains_fetch_like_call_in_stmts(branch)
                })
                || else_branch
                    .as_ref()
                    .is_some_and(|branch| contains_fetch_like_call_in_stmts(branch))
        }
        quantscript::Stmt::For { iterable, body, .. } => {
            contains_fetch_like_call_in_expr(iterable) || contains_fetch_like_call_in_stmts(body)
        }
        quantscript::Stmt::While { condition, body } => {
            contains_fetch_like_call_in_expr(condition) || contains_fetch_like_call_in_stmts(body)
        }
        quantscript::Stmt::Match { expr, arms } => {
            contains_fetch_like_call_in_expr(expr)
                || arms.iter().any(|arm| match &arm.body {
                    quantscript::MatchArmBody::Statement(stmt) => {
                        contains_fetch_like_call_in_stmts(std::slice::from_ref(stmt.as_ref()))
                    }
                    quantscript::MatchArmBody::Expr(expr) => contains_fetch_like_call_in_expr(expr),
                })
        }
    })
}

pub(super) fn contains_emit_intent_in_stmts(stmts: &[quantscript::Stmt]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        quantscript::Stmt::EmitIntent { .. } => true,
        quantscript::Stmt::If {
            then_branch,
            else_if_branches,
            else_branch,
            ..
        } => {
            contains_emit_intent_in_stmts(then_branch)
                || else_if_branches
                    .iter()
                    .any(|(_, branch)| contains_emit_intent_in_stmts(branch))
                || else_branch
                    .as_ref()
                    .is_some_and(|branch| contains_emit_intent_in_stmts(branch))
        }
        quantscript::Stmt::For { body, .. } | quantscript::Stmt::While { body, .. } => {
            contains_emit_intent_in_stmts(body)
        }
        quantscript::Stmt::Match { arms, .. } => arms.iter().any(|arm| match &arm.body {
            quantscript::MatchArmBody::Statement(stmt) => {
                contains_emit_intent_in_stmts(std::slice::from_ref(stmt.as_ref()))
            }
            quantscript::MatchArmBody::Expr(_) => false,
        }),
        _ => false,
    })
}

pub(super) fn contains_fetch_like_call_in_expr(expr: &quantscript::Expr) -> bool {
    match expr {
        quantscript::Expr::Call { callee, args } => {
            if let quantscript::Expr::Identifier(name) = callee.as_ref() {
                if matches!(name.as_str(), "fetch" | "get_data") {
                    return true;
                }
            }
            contains_fetch_like_call_in_expr(callee)
                || args
                    .iter()
                    .any(|arg| contains_fetch_like_call_in_expr(&arg.value))
        }
        quantscript::Expr::Member { object, .. }
        | quantscript::Expr::Unary { expr: object, .. }
        | quantscript::Expr::Try(object)
        | quantscript::Expr::Index { object, .. } => contains_fetch_like_call_in_expr(object),
        quantscript::Expr::Binary { left, right, .. } => {
            contains_fetch_like_call_in_expr(left) || contains_fetch_like_call_in_expr(right)
        }
        quantscript::Expr::List(items) => items.iter().any(contains_fetch_like_call_in_expr),
        quantscript::Expr::Await(inner) => contains_fetch_like_call_in_expr(inner),
        quantscript::Expr::Slice { object, start, end } => {
            contains_fetch_like_call_in_expr(object)
                || start
                    .as_ref()
                    .is_some_and(|expr| contains_fetch_like_call_in_expr(expr))
                || end
                    .as_ref()
                    .is_some_and(|expr| contains_fetch_like_call_in_expr(expr))
        }
        _ => false,
    }
}
