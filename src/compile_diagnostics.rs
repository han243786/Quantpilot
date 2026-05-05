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
            "将条件下发重写为支持的指标或价差意图，或保留下发为无条件。"
                .to_string(),
        ),
        "QPQSLOW002" => Some(
            "保持至少一个可执行的 emit Intent(...) 可从 strategy 到达，并确保其条件下层转换为支持的运行时意图形状。"
                .to_string(),
        ),
        "QPQSLOW003" | "QPQSLOW007" => Some(
            "添加至少一个 fetch(...) 或 get_data(...) 调用，使其在 strategy 下层转换中保持可达。"
                .to_string(),
        ),
        "QPQSLOW004" => Some(
            "在 emit Intent(...) 中使用支持的运行时操作，如 BUY 或 SELL。".to_string(),
        ),
        "QPQSLOW005" => Some(
            "为 emit Intent(...) 提供非空的 action 参数。".to_string(),
        ),
        "QPQSLOW006" => Some(
            "为正式 QuantScript 声明顶层 fn strategy() { ... } 入口点。"
                .to_string(),
        ),
        "QPQSLOW008" => Some(
            "在正式 QuantScript strategy() 中保持最多一个 rebalance(...) 指令。"
                .to_string(),
        ),
        "QPQSLOW009" => Some(
            "仅使用支持的 rebalance(..., every=...) 值，如 \"1d\"、\"slow\" 或 \"weekly\"。"
                .to_string(),
        ),
        "QPQSLOW010" => Some(
            "在使用依赖快照的 universe/filter/sort 操作时提供编译期 universe_snapshot。"
                .to_string(),
        ),
        "QPQSLOW011" => Some(
            "使用支持的 sort_by 键，如 symbol、market_cap、volume_24h 或 listing_age_days。"
                .to_string(),
        ),
        "QPQSLOW012" => Some(
            "使用支持的排序方向，如 asc 或 desc。".to_string(),
        ),
        "QPQSLOW013" => Some(
            "将支持的分配辅助函数（如 equal_weight(...)、fixed_weights(...)、rank_weight(...) 或 score_weight(...)）传入 rebalance(...)。"
                .to_string(),
        ),
        "QPQSLOW014" => Some(
            "将 universe 表达式或 universe 绑定传入 rebalance 分配辅助函数。"
                .to_string(),
        ),
        "QPQSLOW015" => Some(
            "确保 rebalance 分配解析为至少一个选中的交易对。"
                .to_string(),
        ),
        "QPQSLOW016" => Some(
            "为每个选中的交易对提供恰好一个固定权重。".to_string(),
        ),
        "QPQSLOW017" => Some(
            "仅使用非负固定权重。".to_string(),
        ),
        "QPQSLOW018" => Some(
            "确保固定权重总和大于零。".to_string(),
        ),
        "QPQSLOW019" => Some(
            "使用支持的 rank_weight 方法，如 linear 或 inverse_rank。".to_string(),
        ),
        "QPQSLOW020" => Some(
            "使用支持的 score_weight 归一化模式 \"sum\"。".to_string(),
        ),
        "QPQSLOW021" => Some(
            "以数字列表字面量形式提供权重。".to_string(),
        ),
        "QPQSLOW022" => Some(
            "将 fetch(...) 或 get_data(...) 序列作为第一个参数传入指标辅助函数（如 rsi、macd、momentum 或 zscore）。"
                .to_string(),
        ),
        "QPQSLOW023" => Some(
            "使用大于零的指标周期、回看窗口和窗口大小。".to_string(),
        ),
        "QPQSLOW024" => Some(
            "将 fetch/get_data 序列传入移动平均辅助函数，或对 ema(...) 传入可识别的 MACD 线。"
                .to_string(),
        ),
        "QPQSLOW025" => Some(
            "将 universe 值表达式（如 symbols(...)、universe(...)、filter(...)、sort_by(...) 或 top(...)）传入 universe 辅助函数。"
                .to_string(),
        ),
        "QPQSLOW026" => Some(
            "将列表字面量传入 symbols(...)，例如 symbols([\"BTCUSDT\", \"ETHUSDT\"])。"
                .to_string(),
        ),
        "QPQSLOW027" => Some(
            "仅在 symbols([...]) 内使用字符串字面量。".to_string(),
        ),
        "QPQSLOW028" => Some(
            "将数字计数作为第二个位置参数传入 top(...)，例如 top(sort_by(...), 10)。"
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
