use super::*;

pub(super) fn is_v4_backtest_request(request: &FrontendRunRequest, graph_json: &Value) -> bool {
    request
        .backtest_options
        .runtime_kind
        .as_deref()
        .is_some_and(|kind| kind.eq_ignore_ascii_case("v4"))
        || graph_json
            .pointer("/metadata/artifacts/v4_machine_graph")
            .is_some()
        || graph_json.pointer("/metadata/v4_machine_graph").is_some()
        || graph_json
            .pointer("/metadata/artifacts/quantscript/formal_source")
            .and_then(Value::as_str)
            .is_some_and(|source| source.trim_start().starts_with("v4_strategy"))
}

pub(super) fn resolve_v4_backtest_graph(
    graph_json: &Value,
) -> Result<qrpc_core_ir::v4::V4MachineGraphContract, (StatusCode, String)> {
    for pointer in [
        "/metadata/artifacts/v4_machine_graph",
        "/metadata/v4_machine_graph",
        "/artifacts/v4_machine_graph",
    ] {
        if let Some(value) = graph_json.pointer(pointer) {
            let graph =
                serde_json::from_value::<qrpc_core_ir::v4::V4MachineGraphContract>(value.clone())
                    .map_err(|error| {
                    json_bad_request(
                        "v4_graph_invalid",
                        format!("failed to parse {pointer}: {error}"),
                    )
                })?;
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
            return Ok(graph);
        }
    }

    if let Some(source) = graph_json
        .pointer("/metadata/artifacts/quantscript/formal_source")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|source| !source.is_empty())
    {
        let audit = quantscript::audit_v4_quant_script_static(source, &runtime_v4_static_bundle());
        let handoff = quantscript::build_v4_qs_runtime_handoff(&audit);
        if !handoff.accepted_for_runtime_handoff {
            return Err(json_bad_request_with_code(
                "v4_runtime_handoff_rejected",
                crate::error_codes::ERR_QSC_CONTRACT_INVALID,
                format!(
                    "v4 QS backtest handoff rejected: {}",
                    handoff.diagnostics.join("; ")
                ),
            ));
        }
        return audit.parsed_graph.ok_or_else(|| {
            json_bad_request_with_code(
                "v4_graph_missing",
                crate::error_codes::ERR_QSC_CONTRACT_INVALID,
                "v4 QS static audit did not produce a machine graph",
            )
        });
    }

    let qs_protocol = compile_runtime_protocol_via_qs(graph_json)?;
    let compiled = compile_runtime_protocol_config(&qs_protocol).map_err(internal_error)?;
    let bridge = qrpc_core_ir::v4::bridge_core_ir_to_v4_machine_graph(&compiled.core_ir);
    bridge.graph.ok_or_else(|| {
        json_bad_request_with_code(
            "v4_graph_missing",
            crate::error_codes::ERR_QSC_CONTRACT_INVALID,
            format!(
                "core IR compatibility bridge could not produce a v4 graph: {:?}",
                bridge.diagnostics
            ),
        )
    })
}

pub(super) fn resolve_v4_backtest_symbols(
    request: &FrontendRunRequest,
    graph_json: &Value,
    graph: &qrpc_core_ir::v4::V4MachineGraphContract,
) -> Vec<String> {
    let request_symbols = request
        .backtest_options
        .symbols
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    if !request_symbols.is_empty() {
        return qrpc_runtime::normalize_v4_backtest_symbols(&request_symbols);
    }
    for value in [
        graph_json.pointer("/metadata/artifacts/v4_symbols"),
        graph.metadata.get("symbols"),
    ]
    .into_iter()
    .flatten()
    {
        if let Some(values) = value.as_array() {
            let symbols = values
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            if !symbols.is_empty() {
                return qrpc_runtime::normalize_v4_backtest_symbols(&symbols);
            }
        }
    }
    qrpc_runtime::normalize_v4_backtest_symbols(&[])
}

pub(super) fn resolve_v4_backtest_market_event_type(
    graph: &qrpc_core_ir::v4::V4MachineGraphContract,
) -> Result<String, (StatusCode, String)> {
    let Some(catalog) = &graph.event_catalog else {
        return Err(json_bad_request_with_code(
            "v4_event_catalog_missing",
            crate::error_codes::ERR_QSC_CONTRACT_INVALID,
            "v4 backtest requires MachineEventCatalog",
        ));
    };
    catalog
        .events
        .iter()
        .filter(|event| event.source_kind == qrpc_core_ir::v4::MachineEventSourceKind::MarketData)
        .find(|event| event.event_type.contains("bar") || event.event_type.contains("price"))
        .or_else(|| {
            catalog.events.iter().find(|event| {
                event.source_kind == qrpc_core_ir::v4::MachineEventSourceKind::MarketData
            })
        })
        .or_else(|| catalog.events.first())
        .map(|event| event.event_type.clone())
        .ok_or_else(|| {
            json_bad_request_with_code(
                "v4_event_catalog_missing",
                crate::error_codes::ERR_QSC_CONTRACT_INVALID,
                "v4 backtest requires at least one replayable event",
            )
        })
}
