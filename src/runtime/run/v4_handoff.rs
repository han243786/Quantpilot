use crate::runtime::RunInProgressGuard;
use crate::{current_time_ms, internal_error, json_bad_request_with_code, AppState};
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub(crate) struct V4RuntimeRunRequest {
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    graph: Option<qrpc_core_ir::v4::V4MachineGraphContract>,
    #[serde(default)]
    initial_event: Option<qrpc_runtime::V4RuntimeInputEvent>,
}

#[derive(Debug, Serialize)]
pub(crate) struct V4RuntimeRunDiagnostic {
    severity: String,
    code: String,
    message: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct V4RuntimeRunResponse {
    run_id: String,
    graph_id: String,
    event_count: usize,
    output: qrpc_runtime::V4PaperSimulatedRunOutput,
    #[serde(skip_serializing_if = "Option::is_none")]
    handoff: Option<V4RuntimeRunHandoff>,
    diagnostics: Vec<V4RuntimeRunDiagnostic>,
}

#[derive(Debug, Serialize)]
pub(crate) struct V4RuntimeRunHandoff {
    schema_version: String,
    accepted_for_runtime_handoff: bool,
    graph_id: Option<String>,
    venue_id: Option<String>,
    runtime_mode: Option<qrpc_core_ir::v4::RuntimeTradingMode>,
    paper_simulated_start_allowed: bool,
    provider_order_submission_attached: bool,
    runtime_attached: bool,
    lowering_attached: bool,
    diagnostics: Vec<String>,
}

pub(crate) async fn start_v4_runtime_run(
    State(state): State<AppState>,
    Json(request): Json<V4RuntimeRunRequest>,
) -> Result<Json<V4RuntimeRunResponse>, (StatusCode, String)> {
    if state
        .run_in_progress
        .swap(true, std::sync::atomic::Ordering::AcqRel)
    {
        return Err((
            StatusCode::CONFLICT,
            serde_json::to_string(&json!({
                "error": "runtime_busy",
                "error_code": crate::error_codes::ERR_QSC_CAPABILITY_GATED,
                "message": "runtime already has an active run; stop or wait before starting v4 simulation",
                "details": []
            }))
            .unwrap(),
        ));
    }
    let _run_guard = RunInProgressGuard(&state.run_in_progress);

    let now_ms = current_time_ms();
    let (graph, handoff, diagnostics, initial_event_override) =
        resolve_v4_runtime_run_graph(request)?;
    let graph_id = graph.graph_id.clone();
    let initial_event =
        initial_event_override.unwrap_or(handoff_initial_event(handoff.as_ref(), &graph, now_ms)?);

    let mut runtime = qrpc_runtime::V4PaperSimulatedRuntime::new_with_execution_capabilities(
        graph,
        runtime_simulated_v4_matrix("paper-local"),
        vec![qrpc_core_ir::v4::ExecutionCapabilityKind::Market],
    )
    .map_err(internal_error)?;
    let output = runtime
        .submit_event(initial_event)
        .map_err(internal_error)?;

    Ok(Json(V4RuntimeRunResponse {
        run_id: format!("v4_run_{}", now_ms),
        graph_id,
        event_count: output.events.len(),
        output,
        handoff: handoff.as_ref().map(v4_runtime_handoff_response),
        diagnostics,
    }))
}

fn resolve_v4_runtime_run_graph(
    request: V4RuntimeRunRequest,
) -> Result<
    (
        qrpc_core_ir::v4::V4MachineGraphContract,
        Option<quantscript::V4QsRuntimeHandoffReport>,
        Vec<V4RuntimeRunDiagnostic>,
        Option<qrpc_runtime::V4RuntimeInputEvent>,
    ),
    (StatusCode, String),
> {
    let initial_event = request.initial_event;
    if let Some(graph) = request.graph {
        graph.validate_static_contract().map_err(|errors| {
            json_bad_request_with_code(
                "v4_graph_invalid",
                crate::error_codes::ERR_QSC_CONTRACT_INVALID,
                format!(
                    "v4 machine graph failed static validation: {}",
                    errors.join("; ")
                ),
            )
        })?;
        return Ok((graph, None, Vec::new(), initial_event));
    }

    let Some(source) = request
        .source
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return Err(json_bad_request_with_code(
            "v4_source_missing",
            crate::error_codes::ERR_QSC_EMPTY_INTENT,
            "v4 runtime run requires `source` or `graph`",
        ));
    };

    let audit = quantscript::audit_v4_quant_script_static(&source, &runtime_v4_static_bundle());
    let diagnostics = audit
        .diagnostics
        .iter()
        .map(|diagnostic| V4RuntimeRunDiagnostic {
            severity: format!("{:?}", diagnostic.severity).to_ascii_lowercase(),
            code: diagnostic.code.to_string(),
            message: diagnostic.message.clone(),
        })
        .collect::<Vec<_>>();
    let handoff = quantscript::build_v4_qs_runtime_handoff(&audit);
    if !handoff.accepted_for_runtime_handoff {
        return Err(json_bad_request_with_code(
            "v4_runtime_handoff_rejected",
            crate::error_codes::ERR_QSC_CONTRACT_INVALID,
            format!(
                "v4 QS runtime handoff rejected: {}",
                handoff.diagnostics.join("; ")
            ),
        ));
    }
    let graph = audit.parsed_graph.ok_or_else(|| {
        json_bad_request_with_code(
            "v4_graph_missing",
            crate::error_codes::ERR_QSC_CONTRACT_INVALID,
            "v4 QS static audit did not produce a machine graph",
        )
    })?;

    Ok((graph, Some(handoff), diagnostics, initial_event))
}

fn handoff_initial_event(
    _handoff: Option<&quantscript::V4QsRuntimeHandoffReport>,
    graph: &qrpc_core_ir::v4::V4MachineGraphContract,
    ts_ms: u64,
) -> Result<qrpc_runtime::V4RuntimeInputEvent, (StatusCode, String)> {
    let spec = graph
        .event_catalog
        .as_ref()
        .and_then(|catalog| {
            catalog
                .events
                .iter()
                .find(|event| {
                    event.source_kind == qrpc_core_ir::v4::MachineEventSourceKind::Runtime
                })
                .or_else(|| catalog.events.first())
        })
        .ok_or_else(|| {
            json_bad_request_with_code(
                "v4_event_catalog_missing",
                crate::error_codes::ERR_QSC_CONTRACT_INVALID,
                "v4 runtime run requires at least one declared event in MachineEventCatalog",
            )
        })?;
    let mut payload = serde_json::Map::new();
    for field in &spec.payload_fields {
        payload.insert(
            field.name.clone(),
            default_v4_payload_value(field, graph.graph_id.as_str()),
        );
    }
    Ok(qrpc_runtime::V4RuntimeInputEvent {
        event_type: spec.event_type.clone(),
        source: "runtime".to_string(),
        payload: Value::Object(payload),
        ts_ms,
    })
}

fn v4_runtime_handoff_response(
    handoff: &quantscript::V4QsRuntimeHandoffReport,
) -> V4RuntimeRunHandoff {
    V4RuntimeRunHandoff {
        schema_version: handoff.schema_version.clone(),
        accepted_for_runtime_handoff: handoff.accepted_for_runtime_handoff,
        graph_id: handoff.graph_id.clone(),
        venue_id: handoff.venue_id.clone(),
        runtime_mode: handoff.runtime_mode,
        paper_simulated_start_allowed: handoff.paper_simulated_start_allowed,
        provider_order_submission_attached: handoff.provider_order_submission_attached,
        runtime_attached: handoff.runtime_attached,
        lowering_attached: handoff.lowering_attached,
        diagnostics: handoff.diagnostics.clone(),
    }
}

fn default_v4_payload_value(
    field: &qrpc_core_ir::v4::MachineEventPayloadField,
    graph_id: &str,
) -> Value {
    match field.type_name.trim().to_ascii_lowercase().as_str() {
        "string" | "symbol" | "venue" | "account" | "side" | "position_side" | "order_type"
        | "time_in_force" | "freshness" | "runtime_mode" | "order_permission" => {
            if field.name == "strategy_id" {
                Value::String(graph_id.to_string())
            } else {
                Value::String(field.name.clone())
            }
        }
        "bool" | "boolean" => Value::Bool(true),
        "u64" | "uint" => Value::Number(serde_json::Number::from(0_u64)),
        "i64" | "int" | "integer" => Value::Number(serde_json::Number::from(0_i64)),
        "f64" | "decimal" | "number" | "price" | "quantity" | "notional" | "percent" | "ratio"
        | "fee" | "slippage" | "leverage" => serde_json::Number::from_f64(0.0)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        "object" | "map" => json!({}),
        "array" | "list" => json!([]),
        _ if field.nullable => Value::Null,
        _ => Value::String(field.name.clone()),
    }
}

pub(super) fn runtime_v4_static_bundle() -> qrpc_core_ir::v4::V4StaticContractBundle {
    qrpc_core_ir::v4::V4StaticContractBundle {
        venue_matrices: vec![runtime_simulated_v4_matrix("paper-local")],
        ..qrpc_core_ir::v4::V4StaticContractBundle::default()
    }
}

pub(super) fn runtime_simulated_v4_matrix(
    venue_id: impl Into<String>,
) -> qrpc_core_ir::v4::VenueCapabilityMatrix {
    let mut matrix = qrpc_core_ir::v4::unsupported_v4_first_wave_matrix(venue_id);
    for entry in &mut matrix.capabilities {
        if matches!(
            entry.capability,
            qrpc_core_ir::v4::ExecutionCapabilityKind::Market
                | qrpc_core_ir::v4::ExecutionCapabilityKind::Limit
                | qrpc_core_ir::v4::ExecutionCapabilityKind::StopMarket
                | qrpc_core_ir::v4::ExecutionCapabilityKind::StopLimit
                | qrpc_core_ir::v4::ExecutionCapabilityKind::TakeProfitMarket
                | qrpc_core_ir::v4::ExecutionCapabilityKind::TakeProfitLimit
                | qrpc_core_ir::v4::ExecutionCapabilityKind::OcoBracket
                | qrpc_core_ir::v4::ExecutionCapabilityKind::TrailingStop
                | qrpc_core_ir::v4::ExecutionCapabilityKind::Gtc
                | qrpc_core_ir::v4::ExecutionCapabilityKind::Ioc
                | qrpc_core_ir::v4::ExecutionCapabilityKind::Fok
                | qrpc_core_ir::v4::ExecutionCapabilityKind::Day
                | qrpc_core_ir::v4::ExecutionCapabilityKind::Gtd
                | qrpc_core_ir::v4::ExecutionCapabilityKind::PostOnly
                | qrpc_core_ir::v4::ExecutionCapabilityKind::ReduceOnly
                | qrpc_core_ir::v4::ExecutionCapabilityKind::CloseOnly
                | qrpc_core_ir::v4::ExecutionCapabilityKind::ClientOrderId
                | qrpc_core_ir::v4::ExecutionCapabilityKind::OpenLong
                | qrpc_core_ir::v4::ExecutionCapabilityKind::CloseLong
                | qrpc_core_ir::v4::ExecutionCapabilityKind::OpenShort
                | qrpc_core_ir::v4::ExecutionCapabilityKind::CloseShort
                | qrpc_core_ir::v4::ExecutionCapabilityKind::CancelReplaceAmend
        ) {
            entry.source = qrpc_core_ir::v4::CapabilitySupportSource::RuntimeSimulated;
            entry.supported_modes = vec![qrpc_core_ir::v4::RuntimeTradingMode::PaperSimulated];
        }
    }
    matrix
}
