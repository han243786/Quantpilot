use super::*;

#[path = "v4_projection.rs"]
mod v4_projection;

use v4_projection::{
    build_v4_backtest_output, frontend_events_from_v4_backtest_artifact,
    v4_equity_curve_from_artifact,
};

pub(crate) async fn start_backtest_run(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Json(request): Json<FrontendRunRequest>,
) -> Result<Json<BacktestRunResponse>, (StatusCode, String)> {
    let record = execute_backtest_request(&state, &user_id, &request, None).await?;
    Ok(Json(backtest_run_response(
        record.backtest_id,
        record.graph_id,
        record.compile_id,
        record.protocol_name,
        record.config_hash,
        record.events.len(),
        record.account,
        // SAFETY: build_backtest_artifact_views 成功才会进入此分支, .backtest_artifacts 必定为 Some
        record
            .backtest_artifacts
            .ok_or_else(|| internal_error(anyhow::anyhow!("回测工件视图缺失")))?,
    )))
}

pub(super) async fn execute_backtest_request(
    state: &AppState,
    user_id: &auth::UserId,
    request: &FrontendRunRequest,
    id_suffix: Option<&str>,
) -> Result<BacktestRecord, (StatusCode, String)> {
    validate_runtime_capability_guard(request.capability_context.as_ref()).map_err(|details| {
        json_bad_request_with_details(
            "capability_boundary_violation",
            "运行时写入需要当前的能力哈希和权限边界",
            details,
        )
    })?;
    validate_runtime_config_capabilities(&request.runtime_config).map_err(|details| {
        json_bad_request_with_details(
            "capability_gated",
            "运行时配置使用了当前 Beta 版本未启用的能力",
            details,
        )
    })?;
    validate_backtest_execution_assumption_overrides(&request.backtest_options)
        .map_err(|message| json_bad_request("bad_request", message))?;
    let graph_json = request.graph_json.as_ref().ok_or_else(|| {
        json_bad_request(
            "bad_request",
            "回测请求必须包含 graph_json，请从图编辑器发起",
        )
    })?;
    if is_v4_backtest_request(request, graph_json) {
        return execute_v4_backtest_request(state, user_id, request, graph_json, id_suffix).await;
    }
    let qs_protocol = compile_runtime_protocol_via_qs(graph_json)?;
    let runtime_protocol = apply_backtest_execution_assumption_overrides(
        &qs_protocol,
        request.backtest_options.execution_assumptions.as_ref(),
    );
    let compiled = compile_runtime_protocol_config(&runtime_protocol).map_err(internal_error)?;
    let resolved_execution_assumptions = resolved_backtest_execution_assumptions(
        &compiled.config,
        request.backtest_options.execution_assumptions.as_ref(),
    );
    let resolved_execution_assumption_sources = resolved_execution_assumption_sources(request);
    let protocol_name = compiled.protocol_name.clone();
    let config_hash = compiled.config_hash.clone();
    let now_ms = current_time_ms();
    let actor = normalize_actor_identity(request.actor.clone());
    let _collaboration = collaboration_with_run_actor(
        &state.graph_store_dir,
        &request.runtime_config.metadata.graph_id,
        &actor,
    )
    .await?;
    let artifacts = build_compile_artifact_bundle(
        &request.runtime_config.metadata.graph_id,
        &request.runtime_config.metadata.compile_id,
        &request.runtime_config.metadata.name,
        &request.runtime_config.metadata.mode,
        StrategyArtifactSourceKind::FrontendGraph,
        &request.runtime_config.metadata.graph_id,
        BTreeMap::new(),
        &compiled,
    )
    .map_err(internal_error)?;
    let replay_source = request.backtest_replay_source();
    let core_ir = compiled.core_ir.clone();
    let latency_override = resolved_execution_assumptions.latency_assumption_ms;
    // v2.3.3: 沙盒回测操作可能阻塞 (数据加载/HTTP请求)，移至 spawn_blocking 避免阻塞 tokio 线程
    let backtest = tokio::task::spawn_blocking(move || {
        let mut sandbox = match replay_source {
            FrontendBacktestReplaySource::HistoricalReplay => {
                FastBacktestSandbox::with_replay_from_core_ir(core_ir.clone(), now_ms)
                    .map_err(|error| {
                        anyhow::anyhow!(
                            "{}. 历史重放需要本地市场数据文件 (位于 data cache 目录下)。\
                             离线测试请设置 backtest_options.replay_source = \"deterministic_mock\"",
                            error,
                        )
                    })
            }
            FrontendBacktestReplaySource::DeterministicMock => {
                FastBacktestSandbox::with_mock_replay_from_core_ir_and_test_mode(
                    core_ir,
                    now_ms,
                    DeterministicTestMode::replay_defaults(now_ms, BACKTEST_DETERMINISTIC_SEED),
                )
            }
        }
        .map_err(|e| internal_error(anyhow::anyhow!(e)))?;
        if let Some(latency_ms) = latency_override {
            sandbox.set_execution_assumptions(qrpc_runtime::slippage::ExecutionAssumptions {
                latency: qrpc_runtime::slippage::LatencyModel::Fixed { delay_ms: latency_ms },
                ..qrpc_runtime::slippage::ExecutionAssumptions::v1_0_7_compat()
            });
        }
        sandbox.start().map_err(|e| internal_error(anyhow::anyhow!(e)))?;
        sandbox.run_backtest().map_err(|e| internal_error(anyhow::anyhow!(e)))
    })
    .await
    .map_err(|e| internal_error(anyhow::anyhow!("回测任务被取消: {}", e)))??;
    let graph_targets = build_compile_runtime_targets_from_graph(graph_json);
    let runtime_targets = merge_runtime_targets(&request.runtime_targets, &graph_targets);
    // v1.3.7: 添加计数器后缀防止同一毫秒内ID碰撞
    static BACKTEST_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let backtest_id = match id_suffix {
        Some(suffix) => format!("backtest_{}_{}", now_ms, suffix),
        None => format!(
            "backtest_{}_{}",
            now_ms,
            BACKTEST_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ),
    };
    let governance =
        runtime_governance_snapshot(&request.runtime_config.metadata, Some(config_hash.as_str()));
    let mut events = collect_frontend_events_for_backtest(&backtest, &runtime_targets);
    prepend_capability_snapshot_event(
        &mut events,
        &backtest_id,
        &request.runtime_config.metadata.mode,
        now_ms,
        &governance,
    );
    attach_runtime_event_envelopes(
        &mut events,
        &backtest_id,
        &request.runtime_config.metadata.mode,
        &governance,
    );
    validate_runtime_event_envelopes(&events, &backtest_id, &governance)
        .map_err(|message| internal_error(anyhow::anyhow!(message)))?;
    let account = account_summary_from_portfolio(&backtest.final_portfolio);
    let backtest_spec = build_backtest_spec(
        &backtest_id,
        replay_source,
        request,
        &compiled,
        &artifacts,
        now_ms,
        resolved_execution_assumptions,
        resolved_execution_assumption_sources,
    );

    let record = BacktestRecord {
        backtest_id: backtest_id.clone(),
        graph_id: request.runtime_config.metadata.graph_id.clone(),
        compile_id: request.runtime_config.metadata.compile_id.clone(),
        created_at_ms: now_ms,
        protocol_name: protocol_name.clone(),
        config_hash: config_hash.clone(),
        account: account.clone(),
        events: events.clone(),
        backtest: backtest.clone(),
        backtest_spec: Some(backtest_spec.clone()),
        artifacts: Some(artifacts.clone()),
        backtest_artifacts: None,
        governance,
        actor: Some(actor.clone()),
        degraded: false,
    };

    let backtest_artifacts = build_backtest_artifact_views(&record).map_err(internal_error)?;
    let record = BacktestRecord {
        backtest_artifacts: Some(backtest_artifacts.clone()),
        ..record
    };
    let spilled = maybe_spill_transient_backtest_record(
        state.transient_backtest_store_dir.as_ref(),
        &record,
        state.transient_backtest_spill_threshold_bytes,
    )
    .await
    .map_err(io_error)?;
    if !spilled {
        state
            .backtests
            .write()
            .await
            .insert(auth::scoped_key(user_id, &backtest_id), record.clone());
    }

    safe_eprintln!(
        "[audit] 回测完成 — user={} backtest={} graph={} compile={} events={} trades={} return={:.2}%",
        user_id.0,
        backtest_id,
        request.runtime_config.metadata.graph_id,
        request.runtime_config.metadata.compile_id,
        record.events.len(),
        record.backtest.summary.trade_count,
        record.backtest.summary.total_return_ratio * 100.0
    );

    Ok(record)
}

async fn execute_v4_backtest_request(
    state: &AppState,
    user_id: &auth::UserId,
    request: &FrontendRunRequest,
    graph_json: &Value,
    id_suffix: Option<&str>,
) -> Result<BacktestRecord, (StatusCode, String)> {
    let now_ms = current_time_ms();
    let actor = normalize_actor_identity(request.actor.clone());
    let _collaboration = collaboration_with_run_actor(
        &state.graph_store_dir,
        &request.runtime_config.metadata.graph_id,
        &actor,
    )
    .await?;

    static V4_BACKTEST_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let backtest_id = match id_suffix {
        Some(suffix) => format!("backtest_{}_{}", now_ms, suffix),
        None => format!(
            "backtest_{}_v4_{}",
            now_ms,
            V4_BACKTEST_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ),
    };

    let graph = resolve_v4_backtest_graph(graph_json)?;
    let symbols = resolve_v4_backtest_symbols(request, graph_json, &graph);
    let expanded_graph =
        qrpc_runtime::expand_v4_graph_for_symbols(&graph, &symbols).map_err(internal_error)?;
    let event_type = resolve_v4_backtest_market_event_type(&expanded_graph)?;
    let bars = qrpc_runtime::build_v4_deterministic_replay_bars(&symbols, now_ms, &event_type);
    let tick_replay = request
        .backtest_options
        .replay_mode
        .as_deref()
        .map(|mode| mode.eq_ignore_ascii_case("tick_replay"))
        .unwrap_or(false);
    let ticks = if tick_replay {
        bars.iter()
            .enumerate()
            .map(|(index, bar)| qrpc_runtime::V4BacktestTickInput {
                venue_id: bar.venue_id.clone(),
                symbol: bar.symbol.clone(),
                price: bar.close,
                size: 1.0,
                ts_ms: bar.ts_ms,
                sequence: index as u64,
                event_type: event_type.clone(),
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let v4_artifact = tokio::task::spawn_blocking(move || {
        let mut runtime = qrpc_runtime::V4PaperSimulatedRuntime::new_for_backtest(
            expanded_graph,
            runtime_simulated_v4_matrix("paper-local"),
            vec![qrpc_core_ir::v4::ExecutionCapabilityKind::Market],
        )
        .map_err(internal_error)?;
        if tick_replay {
            runtime
                .run_backtest_ticks(&ticks)
                .map_err(|error| internal_error(anyhow::anyhow!(error)))
        } else {
            runtime
                .run_backtest_bars(&bars)
                .map_err(|error| internal_error(anyhow::anyhow!(error)))
        }
    })
    .await
    .map_err(|error| internal_error(anyhow::anyhow!("v4 backtest task cancelled: {error}")))??;

    let config_hash = qrpc_core::canonical_json_sha256_digest(graph_json)
        .map(|digest| digest.value)
        .unwrap_or_else(|_| format!("v4_backtest_{}", now_ms));
    let governance =
        runtime_governance_snapshot(&request.runtime_config.metadata, Some(config_hash.as_str()));
    let equity_curve = v4_equity_curve_from_artifact(&v4_artifact);
    if equity_curve.is_empty() {
        return Err(json_bad_request(
            "v4_backtest_no_execution_data",
            "v4 回测没有执行数据：最终快照缺少 simulated_execution.asset_curve",
        ));
    }

    let mut events =
        frontend_events_from_v4_backtest_artifact(&backtest_id, &v4_artifact, &equity_curve);
    prepend_capability_snapshot_event(
        &mut events,
        &backtest_id,
        &request.runtime_config.metadata.mode,
        now_ms,
        &governance,
    );
    attach_runtime_event_envelopes(
        &mut events,
        &backtest_id,
        &request.runtime_config.metadata.mode,
        &governance,
    );
    validate_runtime_event_envelopes(&events, &backtest_id, &governance)
        .map_err(|message| internal_error(anyhow::anyhow!(message)))?;

    let backtest = build_v4_backtest_output(&v4_artifact, equity_curve);
    let account = account_summary_from_portfolio(&backtest.final_portfolio);
    let record = BacktestRecord {
        backtest_id: backtest_id.clone(),
        graph_id: request.runtime_config.metadata.graph_id.clone(),
        compile_id: request.runtime_config.metadata.compile_id.clone(),
        created_at_ms: now_ms,
        protocol_name: "quantpilot/v4-backtest-runtime".to_string(),
        config_hash: config_hash.clone(),
        account: account.clone(),
        events: events.clone(),
        backtest,
        backtest_spec: None,
        artifacts: None,
        backtest_artifacts: None,
        governance,
        actor: Some(actor.clone()),
        degraded: false,
    };

    let backtest_artifacts = build_backtest_artifact_views(&record).map_err(internal_error)?;
    let record = BacktestRecord {
        backtest_artifacts: Some(backtest_artifacts.clone()),
        ..record
    };
    let spilled = maybe_spill_transient_backtest_record(
        state.transient_backtest_store_dir.as_ref(),
        &record,
        state.transient_backtest_spill_threshold_bytes,
    )
    .await
    .map_err(io_error)?;
    if !spilled {
        state
            .backtests
            .write()
            .await
            .insert(auth::scoped_key(user_id, &backtest_id), record.clone());
    }

    safe_eprintln!(
        "[audit] v4 backtest complete user={} backtest={} graph={} symbols={} events={} trajectory={}",
        user_id.0,
        backtest_id,
        request.runtime_config.metadata.graph_id,
        v4_artifact.symbols.len(),
        record.events.len(),
        v4_artifact.machine_trajectory.len()
    );

    Ok(record)
}

fn is_v4_backtest_request(request: &FrontendRunRequest, graph_json: &Value) -> bool {
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

fn resolve_v4_backtest_graph(
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

fn resolve_v4_backtest_symbols(
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

fn resolve_v4_backtest_market_event_type(
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
