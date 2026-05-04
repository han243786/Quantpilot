use super::*;

fn capability_detail(
    code: impl Into<String>,
    target: impl Into<String>,
    message: impl Into<String>,
    reason: Option<&str>,
) -> ApiErrorDetail {
    ApiErrorDetail {
        code: code.into(),
        target: Some(target.into()),
        message: message.into(),
        span_label: None,
        reason: reason.map(|item| item.to_string()),
    }
}

fn frontend_module_support_reason(module_key: &str) -> Option<&'static str> {
    unsupported_frontend_module_reasons()
        .get(module_key)
        .copied()
}

fn config_csv_values(config: &Value, key: &str) -> Vec<String> {
    config_str(config, key)
        .map(|value| {
            value
                .split(',')
                .map(|item| item.trim())
                .filter(|item| !item.is_empty())
                .map(|item| item.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub(super) fn validate_runtime_config_capabilities(
    input: &FrontendRuntimeConfig,
) -> Result<(), Vec<ApiErrorDetail>> {
    let mut details = Vec::new();

    if !SUPPORTED_RUNTIME_MODE_KEYS.contains(&input.metadata.mode.as_str()) {
        details.push(capability_detail(
            "unsupported_runtime_mode",
            "metadata.mode",
            format!(
                "runtime mode `{}` is not supported in the current beta",
                input.metadata.mode
            ),
            None,
        ));
    }

    if let Some(runtime_control) = &input.runtime_control {
        if !SUPPORTED_FRONTEND_MODULE_KEYS.contains(&runtime_control.module_key.as_str()) {
            details.push(capability_detail(
                "unsupported_module",
                runtime_control.id.clone(),
                format!(
                    "module `{}` is not enabled for compile in the current beta",
                    runtime_control.module_key
                ),
                frontend_module_support_reason(&runtime_control.module_key),
            ));
        }
    }

    for data in &input.data_sources {
        if !SUPPORTED_FRONTEND_MODULE_KEYS.contains(&data.module_key.as_str()) {
            details.push(capability_detail(
                "unsupported_module",
                data.id.clone(),
                format!(
                    "module `{}` is not enabled for compile in the current beta",
                    data.module_key
                ),
                frontend_module_support_reason(&data.module_key),
            ));
        }

        if let Some(exchange) = config_str(&data.config, "exchange") {
            if !SUPPORTED_EXCHANGES.contains(&exchange) {
                details.push(capability_detail(
                    "unsupported_exchange",
                    data.id.clone(),
                    format!(
                        "exchange `{}` is not supported in the current beta",
                        exchange
                    ),
                    None,
                ));
            }
        }

        if let Some(symbol) = config_str(&data.config, "instrument") {
            if !SUPPORTED_SYMBOLS.contains(&symbol) {
                details.push(capability_detail(
                    "unsupported_symbol",
                    data.id.clone(),
                    format!("symbol `{}` is not supported in the current beta", symbol),
                    None,
                ));
            }
        }
    }

    for intent in &input.intent_generators {
        if !SUPPORTED_FRONTEND_MODULE_KEYS.contains(&intent.module_key.as_str()) {
            details.push(capability_detail(
                "unsupported_module",
                intent.id.clone(),
                format!(
                    "module `{}` is not enabled for compile in the current beta",
                    intent.module_key
                ),
                frontend_module_support_reason(&intent.module_key),
            ));
        }
    }

    for agent in &input.agents {
        if !SUPPORTED_FRONTEND_MODULE_KEYS.contains(&agent.module_key.as_str()) {
            details.push(capability_detail(
                "unsupported_module",
                agent.id.clone(),
                format!(
                    "module `{}` is not enabled for compile in the current beta",
                    agent.module_key
                ),
                frontend_module_support_reason(&agent.module_key),
            ));
        }

        for symbol in config_csv_values(&agent.config, "rebalance_symbols") {
            if !SUPPORTED_SYMBOLS.contains(&symbol.as_str()) {
                details.push(capability_detail(
                    "unsupported_symbol",
                    format!("{}.config.rebalance_symbols", agent.id),
                    format!(
                        "rebalance symbol `{}` is not supported in the current beta",
                        symbol
                    ),
                    None,
                ));
            }
        }

        if let Some(schedule) = config_str(&agent.config, "rebalance_schedule") {
            if !schedule.is_empty() && !matches!(schedule, "every_slow" | "every_1d" | "weekly") {
                details.push(capability_detail(
                    "invalid_rebalance_schedule",
                    format!("{}.config.rebalance_schedule", agent.id),
                    format!("rebalance schedule `{}` is not supported", schedule),
                    None,
                ));
            }
        }

        if let Some(raw_weights) = config_str(&agent.config, "rebalance_target_weights") {
            let valid = raw_weights
                .split(',')
                .map(|item| item.trim())
                .filter(|item| !item.is_empty())
                .all(|item| item.parse::<f64>().is_ok());
            if !valid {
                details.push(capability_detail(
                    "invalid_rebalance_target_weights",
                    format!("{}.config.rebalance_target_weights", agent.id),
                    "rebalance target weights must be comma-separated numbers".to_string(),
                    None,
                ));
            }
        }
    }

    for risk in &input.risk_controls {
        if !SUPPORTED_FRONTEND_MODULE_KEYS.contains(&risk.module_key.as_str()) {
            details.push(capability_detail(
                "unsupported_module",
                risk.id.clone(),
                format!(
                    "module `{}` is not enabled for compile in the current beta",
                    risk.module_key
                ),
                frontend_module_support_reason(&risk.module_key),
            ));
        }

        for (key, min, max) in [
            ("max_position", 0.0, 1.0),
            ("max_single_weight", 0.0, 1.0),
            ("max_concentration", 0.0, 1.0),
            ("max_symbol_net_exposure", 0.0, 1.0),
            ("max_portfolio_net_exposure", 0.0, 1.0),
        ] {
            if let Some(value) = config_f64(&risk.config, key) {
                if value < min || value > max {
                    details.push(capability_detail(
                        "invalid_risk_limit",
                        format!("{}.config.{}", risk.id, key),
                        format!("risk field `{}` must be between {} and {}", key, min, max),
                        None,
                    ));
                }
            }
        }

        for (key, min) in [
            ("max_turnover", 0.0),
            ("min_trade_weight", 0.0),
            ("max_total_leverage", 0.0),
            ("max_exchange_leverage", 0.0),
        ] {
            if let Some(value) = config_f64(&risk.config, key) {
                if value < min {
                    details.push(capability_detail(
                        "invalid_risk_limit",
                        format!("{}.config.{}", risk.id, key),
                        format!("risk field `{}` must be >= {}", key, min),
                        None,
                    ));
                }
            }
        }
    }

    for execution in &input.executions {
        if !SUPPORTED_FRONTEND_MODULE_KEYS.contains(&execution.module_key.as_str()) {
            details.push(capability_detail(
                "unsupported_module",
                execution.id.clone(),
                format!(
                    "module `{}` is not enabled for compile in the current beta",
                    execution.module_key
                ),
                frontend_module_support_reason(&execution.module_key),
            ));
        }

        if !SUPPORTED_RUNTIME_EXECUTION_MODULE_KEYS.contains(&execution.module_key.as_str()) {
            details.push(capability_detail(
                "unsupported_execution_module",
                execution.id.clone(),
                format!(
                    "execution module `{}` is not supported in the current beta",
                    execution.module_key
                ),
                None,
            ));
        }
    }

    if details.is_empty() {
        Ok(())
    } else {
        Err(details)
    }
}

fn is_valid_capability_hash(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .chars()
                .all(|ch| ch.is_ascii_digit() || matches!(ch, 'a'..='f'))
    })
}

pub(super) fn validate_runtime_capability_guard(
    context: Option<&FrontendCapabilityContext>,
) -> Result<(), Vec<ApiErrorDetail>> {
    let mut details = Vec::new();
    let expected = current_capability_context();
    let Some(context) = context else {
        return Err(vec![capability_detail(
            "missing_capability_context",
            "capability_context",
            "runtime write actions require the current capability context",
            Some("Fetch /api/capabilities and include schema_hash plus permission_boundary before creating runtime records."),
        )]);
    };

    if !is_valid_capability_hash(&context.schema_hash) {
        details.push(capability_detail(
            "malformed_capability_hash",
            "capability_context.schema_hash",
            "capability schema_hash must be formatted as sha256:<64 lowercase hex chars>",
            Some("Capability hashes outside the canonical sha256 namespace are treated as unsafe."),
        ));
    } else if context.schema_hash != expected.schema_hash {
        details.push(capability_detail(
            "stale_capability_hash",
            "capability_context.schema_hash",
            "capability schema_hash does not match the current backend capability contract",
            Some("Refresh /api/capabilities before starting run, backtest, or experiment writes."),
        ));
    }

    let permission = &context.permission_boundary;
    let expected_permission = &expected.permission_boundary;
    for (target, actual, expected_value) in [
        (
            "capability_context.permission_boundary.model_version",
            permission.model_version.as_str(),
            expected_permission.model_version.as_str(),
        ),
        (
            "capability_context.permission_boundary.execution_owner_module",
            permission.execution_owner_module.as_str(),
            expected_permission.execution_owner_module.as_str(),
        ),
        (
            "capability_context.permission_boundary.ai_write_policy",
            permission.ai_write_policy.as_str(),
            expected_permission.ai_write_policy.as_str(),
        ),
        (
            "capability_context.permission_boundary.plugin_network_default",
            permission.plugin_network_default.as_str(),
            expected_permission.plugin_network_default.as_str(),
        ),
        (
            "capability_context.permission_boundary.non_execution_order_access",
            permission.non_execution_order_access.as_str(),
            expected_permission.non_execution_order_access.as_str(),
        ),
    ] {
        if actual != expected_value {
            details.push(capability_detail(
                "permission_boundary_mismatch",
                target,
                format!("permission boundary field must be `{expected_value}`"),
                Some("Runtime writes fail closed when the frontend boundary is missing, stale, or less restrictive than the backend contract."),
            ));
        }
    }

    if permission.live_execution_allowed != expected_permission.live_execution_allowed {
        details.push(capability_detail(
            "permission_boundary_mismatch",
            "capability_context.permission_boundary.live_execution_allowed",
            "live execution must remain disabled in the current beta",
            Some("The current runtime boundary only allows paper execution."),
        ));
    }

    if details.is_empty() {
        Ok(())
    } else {
        Err(details)
    }
}

pub(super) fn validate_backtest_execution_assumption_overrides(
    options: &FrontendBacktestOptions,
) -> Result<(), String> {
    let Some(overrides) = options.execution_assumptions.as_ref() else {
        return Ok(());
    };

    if overrides.fee_bps.is_some_and(|value| value < 0.0) {
        return Err("backtest_options.execution_assumptions.fee_bps must be >= 0".to_string());
    }
    if overrides.slippage_bps.is_some_and(|value| value < 0.0) {
        return Err("backtest_options.execution_assumptions.slippage_bps must be >= 0".to_string());
    }

    Ok(())
}

pub(super) fn resolved_execution_assumption_sources(
    request: &FrontendRunRequest,
) -> ExecutionAssumptionSourceSummary {
    let overrides = request.backtest_options.execution_assumptions.as_ref();
    let execution_config = request
        .runtime_config
        .executions
        .first()
        .map(|node| &node.config);

    let fee_bps = if overrides.and_then(|value| value.fee_bps).is_some() {
        ExecutionAssumptionValueSource::RequestOverride
    } else if execution_config
        .and_then(|config| config.get("fee_bps"))
        .is_some()
    {
        ExecutionAssumptionValueSource::ProfileDefault
    } else {
        ExecutionAssumptionValueSource::BackendFallback
    };

    let slippage_bps = if overrides.and_then(|value| value.slippage_bps).is_some() {
        ExecutionAssumptionValueSource::RequestOverride
    } else if execution_config
        .and_then(|config| config.get("slippage_bps"))
        .is_some()
    {
        ExecutionAssumptionValueSource::ProfileDefault
    } else {
        ExecutionAssumptionValueSource::BackendFallback
    };

    let latency_ms = if overrides.and_then(|value| value.latency_ms).is_some() {
        ExecutionAssumptionValueSource::RequestOverride
    } else {
        ExecutionAssumptionValueSource::BackendFallback
    };

    ExecutionAssumptionSourceSummary {
        fee_bps,
        slippage_bps,
        latency_ms,
    }
}

pub(super) fn apply_backtest_execution_assumption_overrides(
    runtime_protocol: &RuntimeProtocolCoreConfig,
    overrides: Option<&FrontendExecutionAssumptionOverrides>,
) -> RuntimeProtocolCoreConfig {
    let Some(overrides) = overrides else {
        return runtime_protocol.clone();
    };

    let mut adjusted = runtime_protocol.clone();
    if let Some(fee_bps) = overrides.fee_bps {
        adjusted.taker_fee_bps = fee_bps;
    }
    if let Some(slippage_bps) = overrides.slippage_bps {
        adjusted.default_slippage_bps = slippage_bps;
    }
    adjusted
}

pub(super) fn resolved_backtest_execution_assumptions(
    runtime_protocol: &RuntimeProtocolCoreConfig,
    overrides: Option<&FrontendExecutionAssumptionOverrides>,
) -> ExecutionAssumptionSpec {
    let mut assumptions = ExecutionAssumptionSpec::from(runtime_protocol);
    if let Some(overrides) = overrides {
        if let Some(fee_bps) = overrides.fee_bps {
            assumptions.taker_fee_bps = fee_bps;
        }
        if let Some(slippage_bps) = overrides.slippage_bps {
            assumptions.default_slippage_bps = slippage_bps;
        }
        assumptions.latency_assumption_ms = overrides.latency_ms;
    }
    assumptions
}

pub(super) fn validate_graph_id(input: &str) -> anyhow::Result<()> {
    let valid = !input.is_empty()
        && input
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-');
    if valid {
        Ok(())
    } else {
        anyhow::bail!("graph_id must use only ASCII letters, numbers, '_' or '-'")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_runtime_config() -> FrontendRuntimeConfig {
        FrontendRuntimeConfig {
            metadata: FrontendMetadata {
                graph_id: "graph_quality".to_string(),
                compile_id: "compile_quality".to_string(),
                name: "Quality".to_string(),
                version: "1.0.0".to_string(),
                mode: "paper".to_string(),
            },
            data_sources: vec![],
            intent_generators: vec![],
            agents: vec![FrontendAgentConfig {
                id: "agent_agent_1".to_string(),
                module_key: "builtin.agent.weighted".to_string(),
                name: "Agent".to_string(),
                config: json!({
                    "decision_threshold": 0.05,
                    "max_quantity_ratio": 0.4,
                    "rebalance_symbols": "BTCUSDT, XRPUSDT",
                    "rebalance_schedule": "weekly",
                    "rebalance_target_weights": "0.7, bad"
                }),
                intent_refs: vec![],
            }],
            risk_controls: vec![],
            executions: vec![],
            runtime_control: None,
        }
    }

    #[test]
    fn validate_runtime_config_capabilities_rejects_invalid_multi_symbol_agent_config() {
        let error = validate_runtime_config_capabilities(&sample_runtime_config())
            .expect_err("invalid rebalance config should be rejected");
        assert!(error
            .iter()
            .any(|detail| detail.code == "unsupported_symbol"));
        assert!(error
            .iter()
            .any(|detail| detail.code == "invalid_rebalance_target_weights"));
    }

    #[test]
    fn validate_runtime_capability_guard_rejects_missing_context() {
        let error = validate_runtime_capability_guard(None)
            .expect_err("missing capability context should fail closed");

        assert_eq!(error[0].code, "missing_capability_context");
        assert_eq!(error[0].target.as_deref(), Some("capability_context"));
    }

    #[test]
    fn validate_runtime_capability_guard_rejects_malformed_hash() {
        let mut context = current_capability_context();
        context.schema_hash = "safe-fallback".to_string();

        let error = validate_runtime_capability_guard(Some(&context))
            .expect_err("malformed hash should fail closed");

        assert!(error
            .iter()
            .any(|detail| detail.code == "malformed_capability_hash"));
    }

    #[test]
    fn validate_runtime_capability_guard_rejects_permission_boundary_mismatch() {
        let mut context = current_capability_context();
        context.permission_boundary.live_execution_allowed = true;

        let error = validate_runtime_capability_guard(Some(&context))
            .expect_err("less restrictive permission boundary should fail closed");

        assert!(error.iter().any(|detail| {
            detail.code == "permission_boundary_mismatch"
                && detail.target.as_deref()
                    == Some("capability_context.permission_boundary.live_execution_allowed")
        }));
    }
}
