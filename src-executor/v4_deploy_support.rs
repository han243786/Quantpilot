use axum::http::StatusCode;
use std::collections::BTreeMap;
use std::sync::Arc;

use qrpc_core::CoreStrategyIr;
use qrpc_core_ir::{
    v4::{
        CapabilitySupportSource, ExecutionCapabilityKind, PluginKind, PluginManifestSpec,
        PluginNetworkPermission, PluginRuntimePermission, PluginSideEffect, QsScalarTypeKind,
        QsTypeRef, RuntimeTradingMode, V4MachineGraphContract, V4StaticContractBundle,
        VenueCapabilityMatrix,
    },
    CoreMetadata, CoreSourceKind, CoreTimeInForce, ExecutionRule, ExecutionSizingKind,
};

use super::append_audit;
use super::executor_state::{
    ActiveStrategy, ExecutionMode, ExecutorState, RuntimeKind, StrategyStatus,
};

#[derive(Debug, serde::Deserialize)]
struct V4StrategyDeployRequest {
    #[serde(default)]
    strategy_id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    runtime_kind: Option<String>,
    #[serde(default)]
    runtime_version: Option<String>,
    #[serde(default)]
    graph: Option<V4MachineGraphContract>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    params: Option<BTreeMap<String, serde_json::Value>>,
    #[serde(default)]
    params_snapshot: Option<BTreeMap<String, serde_json::Value>>,
    #[serde(default)]
    execution_mode: Option<String>,
    #[serde(default)]
    strategy_config_preflight: Option<serde_json::Value>,
}

pub(super) fn is_v4_deploy_request(body: &serde_json::Value) -> bool {
    let runtime = body
        .get("runtime_kind")
        .or_else(|| body.get("runtime_version"))
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    runtime.eq_ignore_ascii_case("v4")
        || body.get("graph").is_some()
        || body.get("source").is_some()
}

pub(super) fn deploy_v4_strategy(
    state: &Arc<ExecutorState>,
    body: serde_json::Value,
) -> Result<serde_json::Value, (axum::http::StatusCode, String)> {
    let request: V4StrategyDeployRequest = serde_json::from_value(body).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            format!("v4 策略部署请求解析失败: {}", e),
        )
    })?;
    let runtime_label = request
        .runtime_kind
        .as_deref()
        .or(request.runtime_version.as_deref())
        .unwrap_or("v4");
    if !runtime_label.eq_ignore_ascii_case("v4") {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            format!("不支持的 runtime_kind: {}", runtime_label),
        ));
    }
    let graph = resolve_v4_deploy_graph(&request)?;
    let graph_id = graph.graph_id.clone();
    let strategy_id = request
        .strategy_id
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| graph_id.clone());
    let name = request
        .name
        .clone()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            graph
                .metadata
                .get("name")
                .and_then(|value| value.as_str())
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| graph_id.clone());
    let params = request
        .params
        .or(request.params_snapshot)
        .unwrap_or_default();
    let subscribed_symbols = extract_v4_subscribed_symbols(&graph);
    let graph_json = serde_json::to_value(&graph).map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("v4 graph 序列化失败: {}", e),
        )
    })?;
    let execution_mode = parse_execution_mode(request.execution_mode.as_deref())?;
    let strategy = ActiveStrategy {
        strategy_id: strategy_id.clone(),
        name,
        runtime_kind: RuntimeKind::V4,
        core_ir: empty_core_ir(&strategy_id),
        v4_graph: Some(graph),
        graph_json,
        params,
        status: StrategyStatus::Loaded,
        subscribed_symbols,
        execution_mode,
        strategy_config_preflight: request.strategy_config_preflight,
    };
    state.register(strategy).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            format!("v4 策略注册失败: {:#}", e),
        )
    })?;
    append_audit(
        state,
        "load_strategy",
        Some(strategy_id.clone()),
        serde_json::json!({
            "source": "executor_v4_deploy",
            "runtime_kind": "v4",
            "execution_mode": execution_mode.as_str(),
            "execution_mode_label": execution_mode.display_label(),
        }),
    );
    Ok(serde_json::json!({
        "status": "loaded",
        "strategy_id": strategy_id,
        "runtime_kind": "v4",
        "runtime_version": "v4",
        "execution_mode": execution_mode.as_str(),
        "graph_id": graph_id,
    }))
}

fn parse_execution_mode(
    value: Option<&str>,
) -> Result<ExecutionMode, (axum::http::StatusCode, String)> {
    let raw = value.unwrap_or("paper_simulated");
    match ExecutionMode::from_api_label(raw) {
        Some(mode) => Ok(mode),
        None if raw.eq_ignore_ascii_case("live")
            || raw.eq_ignore_ascii_case("live_actual")
            || raw.eq_ignore_ascii_case("live_simulated") =>
        {
            Err((
                axum::http::StatusCode::BAD_REQUEST,
                "真实资金或真实账户上下文 execution_mode 已延后；请使用 paper_simulated / paper_actual"
                    .to_string(),
            ))
        }
        None => Err((
            axum::http::StatusCode::BAD_REQUEST,
            format!("不支持的 execution_mode: {}", raw),
        )),
    }
}

fn resolve_v4_deploy_graph(
    request: &V4StrategyDeployRequest,
) -> Result<V4MachineGraphContract, (axum::http::StatusCode, String)> {
    if let Some(graph) = request.graph.clone() {
        graph.validate_static_contract().map_err(|errors| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                format!("v4 graph 静态契约失败: {}", errors.join("; ")),
            )
        })?;
        return Ok(graph);
    }

    let source = request
        .source
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or((
            axum::http::StatusCode::BAD_REQUEST,
            "v4 策略部署需要 graph 或 source".to_string(),
        ))?;
    let report = quantscript::audit_v4_quant_script_static(source, &executor_v4_static_bundle());
    let handoff = quantscript::build_v4_qs_runtime_handoff(&report);
    if !handoff.accepted_for_runtime_handoff {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            format!(
                "v4 QS runtime handoff rejected: {}",
                handoff.diagnostics.join("; ")
            ),
        ));
    }
    report.parsed_graph.ok_or((
        axum::http::StatusCode::BAD_REQUEST,
        "v4 QS static audit did not produce a machine graph".to_string(),
    ))
}

fn extract_v4_subscribed_symbols(graph: &V4MachineGraphContract) -> Vec<qrpc_core::Symbol> {
    let mut symbols = Vec::new();
    for key in ["symbol", "default_symbol"] {
        if let Some(symbol) = graph.metadata.get(key).and_then(|value| value.as_str()) {
            symbols.push(qrpc_core::Symbol::Other(symbol.to_string()));
        }
    }
    if let Some(values) = graph
        .metadata
        .get("symbols")
        .and_then(|value| value.as_array())
    {
        for value in values {
            if let Some(symbol) = value.as_str() {
                symbols.push(qrpc_core::Symbol::Other(symbol.to_string()));
            }
        }
    }
    symbols
}

pub(super) fn empty_core_ir(strategy_id: &str) -> CoreStrategyIr {
    CoreStrategyIr::new(
        CoreMetadata {
            strategy_id: strategy_id.to_string(),
            name: strategy_id.to_string(),
            source_kind: CoreSourceKind::RuntimeProtocol,
        },
        ExecutionRule {
            execution_id: format!("exec_{}", strategy_id),
            venue_kind: "paper".into(),
            sizing_kind: ExecutionSizingKind::EquityNotionalRatio,
            slippage_bps: 0.0,
            taker_fee_bps: 0.0,
            total_cost_buffer_bps: 0.0,
            time_in_force: CoreTimeInForce::Gtc,
            params: BTreeMap::new(),
        },
    )
}

pub(super) fn ensure_strategy_config_preflight_allows_start(
    strategy: &ActiveStrategy,
) -> Result<(), (StatusCode, String)> {
    if strategy.runtime_kind != RuntimeKind::V4 {
        return Ok(());
    }

    let Some(preflight) = strategy.strategy_config_preflight.as_ref() else {
        return Err(strategy_config_preflight_error(
            strategy,
            "strategy_config_preflight_missing",
            "v4 策略启动前缺少 strategy config preflight，已拒绝启动。",
            None,
        ));
    };

    let decision = json_path_str(preflight, &["decision"]).unwrap_or("blocked");
    if decision == "blocked" {
        return Err(strategy_config_preflight_error(
            strategy,
            "strategy_config_preflight_blocked",
            "strategy config preflight 已阻断启动。",
            Some(preflight),
        ));
    }

    if json_path_bool(preflight, &["can_live_execution"]).unwrap_or(false)
        || json_path_bool(
            preflight,
            &["artifact", "runtime_boundary", "live_execution_allowed"],
        )
        .unwrap_or(false)
    {
        return Err(strategy_config_preflight_error(
            strategy,
            "strategy_config_live_execution_forbidden",
            "live_execution_allowed=false 是执行端硬边界，已拒绝启动。",
            Some(preflight),
        ));
    }

    let mode_label = json_path_str(preflight, &["artifact", "runtime_boundary", "mode_label"])
        .unwrap_or("PaperSimulated");
    let (expected_mode_label, can_field, required_action) = match strategy.execution_mode {
        ExecutionMode::PaperSimulated => (
            "PaperSimulated",
            "can_paper_simulated",
            "start_paper_simulated",
        ),
        ExecutionMode::PaperActual => (
            "PaperActual",
            "can_paper_actual_demo",
            "start_paper_actual_demo",
        ),
    };

    if mode_label != expected_mode_label {
        return Err(strategy_config_preflight_error(
            strategy,
            "strategy_config_runtime_boundary_mismatch",
            "strategy config preflight 的运行边界与执行端策略模式不一致，已拒绝启动。",
            Some(preflight),
        ));
    }

    if !json_path_bool(preflight, &[can_field]).unwrap_or(false)
        || !json_array_contains(preflight, &["allowed_actions"], required_action)
    {
        return Err(strategy_config_preflight_error(
            strategy,
            "strategy_config_start_action_not_allowed",
            "strategy config preflight 未允许当前执行端启动动作，已拒绝启动。",
            Some(preflight),
        ));
    }

    Ok(())
}

fn strategy_config_preflight_error(
    strategy: &ActiveStrategy,
    code: &str,
    message: &str,
    preflight: Option<&serde_json::Value>,
) -> (StatusCode, String) {
    (
        StatusCode::LOCKED,
        serde_json::json!({
            "error": code,
            "message": message,
            "strategy_id": strategy.strategy_id,
            "runtime_kind": strategy.runtime_kind.as_str(),
            "execution_mode": strategy.execution_mode.as_str(),
            "execution_mode_label": strategy.execution_mode.display_label(),
            "preflight_decision": preflight.and_then(|value| json_path_str(value, &["decision"])),
            "artifact_digest": preflight
                .and_then(|value| json_path_str(value, &["artifact", "artifact_digest"])),
        })
        .to_string(),
    )
}

pub(super) fn json_path_str<'a>(value: &'a serde_json::Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str()
}

fn json_path_bool(value: &serde_json::Value, path: &[&str]) -> Option<bool> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_bool()
}

fn json_array_contains(value: &serde_json::Value, path: &[&str], expected: &str) -> bool {
    let mut current = value;
    for key in path {
        let Some(next) = current.get(*key) else {
            return false;
        };
        current = next;
    }
    current
        .as_array()
        .is_some_and(|items| items.iter().any(|item| item.as_str() == Some(expected)))
}

fn executor_v4_static_bundle() -> V4StaticContractBundle {
    V4StaticContractBundle {
        venue_matrices: vec![
            executor_v4_market_matrix("paper-local"),
            executor_v4_market_matrix("paper-simulated"),
        ],
        plugin_manifests: vec![executor_v4_sample_plugin_manifest()],
        ..V4StaticContractBundle::default()
    }
}

fn executor_v4_market_matrix(venue_id: impl Into<String>) -> VenueCapabilityMatrix {
    let mut matrix = qrpc_core_ir::v4::unsupported_v4_first_wave_matrix(venue_id);
    for entry in &mut matrix.capabilities {
        if matches!(
            entry.capability,
            ExecutionCapabilityKind::Market
                | ExecutionCapabilityKind::Limit
                | ExecutionCapabilityKind::StopMarket
                | ExecutionCapabilityKind::StopLimit
                | ExecutionCapabilityKind::TakeProfitMarket
                | ExecutionCapabilityKind::TakeProfitLimit
                | ExecutionCapabilityKind::Gtc
                | ExecutionCapabilityKind::Ioc
                | ExecutionCapabilityKind::Fok
                | ExecutionCapabilityKind::Day
                | ExecutionCapabilityKind::Gtd
                | ExecutionCapabilityKind::PostOnly
                | ExecutionCapabilityKind::ReduceOnly
                | ExecutionCapabilityKind::CloseOnly
                | ExecutionCapabilityKind::ClientOrderId
                | ExecutionCapabilityKind::OpenLong
                | ExecutionCapabilityKind::CloseLong
                | ExecutionCapabilityKind::OpenShort
                | ExecutionCapabilityKind::CloseShort
        ) {
            entry.source = CapabilitySupportSource::RuntimeSimulated;
            entry.supported_modes = vec![RuntimeTradingMode::PaperSimulated];
        }
    }
    matrix
}

fn executor_v4_sample_plugin_manifest() -> PluginManifestSpec {
    PluginManifestSpec {
        plugin_id: "pure.indicator.zscore".to_string(),
        name: "ZScore".to_string(),
        version: "0.1.0".to_string(),
        kind: PluginKind::Pure,
        input_schema: Some(QsTypeRef::List {
            item: Box::new(QsTypeRef::Scalar {
                scalar: QsScalarTypeKind::Price,
            }),
            max_items: 256,
        }),
        output_schema: Some(QsTypeRef::Scalar {
            scalar: QsScalarTypeKind::Decimal,
        }),
        deterministic: true,
        side_effect: PluginSideEffect::None,
        runtime_permission: PluginRuntimePermission::None,
        network_permission: PluginNetworkPermission::None,
        capability_matrix: None,
        test_fixture_id: "fixture.zscore.basic".to_string(),
    }
}
