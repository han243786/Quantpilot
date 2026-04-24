use super::backtest_compare::compare_backtests;
use super::*;

const MAX_EXPERIMENT_VARIANTS: usize = 27;
const DEFAULT_REPLAY_PAGE_SIZE: usize = 12;
const MAX_REPLAY_PAGE_SIZE: usize = 50;
pub(super) fn register_runtime_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/runtime/backtest", post(start_backtest_run))
        .route("/api/runtime/backtests", get(list_backtests))
        .route("/api/runtime/backtests/compare", post(compare_backtests))
        .route(
            "/api/runtime/backtests/:backtest_id",
            get(get_backtest_detail),
        )
        .route(
            "/api/runtime/backtests/:backtest_id/replay",
            get(get_backtest_replay),
        )
        .route("/api/runtime/test-run", post(start_test_run))
        .route("/api/runtime/runs", get(list_runs))
        .route("/api/runtime/runs/:run_id", get(get_run_detail))
        .route("/api/runtime/runs/:run_id/events", get(stream_run_events))
        .route("/api/runtime/runs/:run_id/replay", get(get_run_replay))
        .route("/api/runtime/runs/:run_id/status", get(get_run_status))
        .route(
            "/api/runtime/experiments/backtest-sweep",
            post(start_backtest_experiment),
        )
        .route("/api/runtime/experiments", get(list_experiments))
        .route(
            "/api/runtime/experiments/:experiment_id",
            get(get_experiment_detail),
        )
}

#[derive(Debug, Deserialize)]
struct RuntimeReplayQuery {
    cursor: Option<usize>,
    limit: Option<usize>,
    checkpoint: Option<usize>,
}

fn normalized_replay_window(query: RuntimeReplayQuery) -> (usize, usize) {
    let cursor = query.checkpoint.or(query.cursor).unwrap_or(0);
    let limit = query
        .limit
        .unwrap_or(DEFAULT_REPLAY_PAGE_SIZE)
        .clamp(1, MAX_REPLAY_PAGE_SIZE);
    (cursor, limit)
}

async fn start_test_run(
    State(state): State<AppState>,
    Json(request): Json<FrontendRunRequest>,
) -> Result<Json<RunStartResponse>, (StatusCode, String)> {
    validate_runtime_config_capabilities(&request.runtime_config).map_err(|details| {
        json_bad_request_with_details(
            "capability_gated",
            "runtime config uses capabilities that are not enabled in the current beta",
            details,
        )
    })?;
    let mapped = map_frontend_runtime_config(&request.runtime_config).map_err(internal_error)?;
    let compiled =
        compile_runtime_protocol_config(&mapped.runtime_protocol).map_err(internal_error)?;
    let mut sandbox = RealTimeSandbox::new(RuntimeCoordinator::new(compiled));
    let now_ms = current_time_ms();
    sandbox.start().map_err(internal_error)?;
    let session = sandbox
        .run_session(now_ms, now_ms + RUN_WINDOW_MS)
        .map_err(internal_error)?;
    let run_id = format!("run_{}", now_ms);
    let runtime_targets = merge_runtime_targets(&request.runtime_targets, &mapped);
    let events = collect_frontend_events(&session, &runtime_targets);
    let account = account_summary(&session);
    let graph_id = request.runtime_config.metadata.graph_id.clone();
    let compile_id = request.runtime_config.metadata.compile_id.clone();
    let actor = normalize_actor_identity(request.actor);
    let _collaboration = collaboration_with_run_actor(&state.graph_store_dir, &graph_id, &actor)?;

    let record = RunRecord {
        run_id: run_id.clone(),
        graph_id: graph_id.clone(),
        compile_id: compile_id.clone(),
        created_at_ms: now_ms,
        events: events.clone(),
        account: account.clone(),
        session,
        actor: Some(actor.clone()),
    };

    persist_run_record(state.run_store_dir.as_ref(), &record)
        .await
        .map_err(io_error)?;
    persist_graph_audit_entry(
        &state.audit_store_dir,
        &build_graph_audit_entry(
            &graph_id,
            &actor,
            GraphAuditAction::RunCreated,
            Some(run_id.clone()),
            format!("Started runtime simulation {run_id}"),
        ),
    )
    .await
    .map_err(io_error)?;
    state.runs.write().await.insert(run_id.clone(), record);

    Ok(Json(run_start_response(
        run_id,
        graph_id,
        compile_id,
        events.len(),
    )))
}

async fn start_backtest_run(
    State(state): State<AppState>,
    Json(request): Json<FrontendRunRequest>,
) -> Result<Json<BacktestRunResponse>, (StatusCode, String)> {
    let record = execute_backtest_request(&state, &request, None).await?;
    Ok(Json(backtest_run_response(
        record.backtest_id,
        record.graph_id,
        record.compile_id,
        record.protocol_name,
        record.config_hash,
        record.events.len(),
        record.account,
        record
            .backtest_artifacts
            .expect("backtest artifact views should exist for run responses"),
    )))
}

async fn execute_backtest_request(
    state: &AppState,
    request: &FrontendRunRequest,
    id_suffix: Option<&str>,
) -> Result<BacktestRecord, (StatusCode, String)> {
    validate_runtime_config_capabilities(&request.runtime_config).map_err(|details| {
        json_bad_request_with_details(
            "capability_gated",
            "runtime config uses capabilities that are not enabled in the current beta",
            details,
        )
    })?;
    validate_backtest_execution_assumption_overrides(&request.backtest_options)
        .map_err(|message| json_bad_request("bad_request", message))?;
    let mapped = map_frontend_runtime_config(&request.runtime_config).map_err(internal_error)?;
    let runtime_protocol = apply_backtest_execution_assumption_overrides(
        &mapped.runtime_protocol,
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
    )?;
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
    let mut sandbox = match replay_source {
        FrontendBacktestReplaySource::HistoricalReplay => {
            FastBacktestSandbox::with_replay_from_core_ir(compiled.core_ir.clone(), now_ms)
        }
        FrontendBacktestReplaySource::DeterministicMock => {
            FastBacktestSandbox::with_mock_replay_from_core_ir_and_test_mode(
                compiled.core_ir.clone(),
                now_ms,
                DeterministicTestMode::replay_defaults(now_ms, BACKTEST_DETERMINISTIC_SEED),
            )
        }
    }
    .map_err(internal_error)?;
    if let Some(latency_ms) = resolved_execution_assumptions.latency_assumption_ms {
        sandbox.set_latency_assumption_ms(latency_ms);
    }
    sandbox.start().map_err(internal_error)?;
    let backtest = sandbox.run_backtest().map_err(internal_error)?;
    let runtime_targets = merge_runtime_targets(&request.runtime_targets, &mapped);
    let events = collect_frontend_events_for_backtest(&backtest, &runtime_targets);
    let backtest_id = match id_suffix {
        Some(suffix) => format!("backtest_{}_{}", now_ms, suffix),
        None => format!("backtest_{}", now_ms),
    };
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
        actor: Some(actor.clone()),
    };

    let backtest_artifacts = persist_backtest_record(state.backtest_store_dir.as_ref(), &record)
        .await
        .map_err(io_error)?;
    let record = BacktestRecord {
        backtest_artifacts: Some(backtest_artifacts.clone()),
        ..record
    };
    state
        .backtests
        .write()
        .await
        .insert(backtest_id.clone(), record.clone());
    persist_graph_audit_entry(
        &state.audit_store_dir,
        &build_graph_audit_entry(
            &record.graph_id,
            &actor,
            GraphAuditAction::BacktestCreated,
            Some(backtest_id.clone()),
            format!("Started backtest {backtest_id}"),
        ),
    )
    .await
    .map_err(io_error)?;

    Ok(record)
}

fn normalize_experiment_float_axis(
    values: &[f64],
    base: f64,
    field: &str,
) -> Result<Vec<f64>, (StatusCode, String)> {
    let mut normalized = Vec::new();
    if values.is_empty() {
        normalized.push(base);
        return Ok(normalized);
    }

    for value in values {
        if *value < 0.0 {
            return Err(json_bad_request(
                "bad_request",
                format!("parameter_grid.{field} must be >= 0"),
            ));
        }
        if !normalized.contains(value) {
            normalized.push(*value);
        }
    }

    Ok(normalized)
}

fn normalize_experiment_latency_axis(values: &[u64], base: u64) -> Vec<u64> {
    let mut normalized = Vec::new();
    if values.is_empty() {
        normalized.push(base);
        return normalized;
    }

    for value in values {
        if !normalized.contains(value) {
            normalized.push(*value);
        }
    }

    normalized
}

fn build_experiment_overrides(
    request: &FrontendExperimentRequest,
) -> Result<Vec<FrontendExecutionAssumptionOverrides>, (StatusCode, String)> {
    let provided_values = request.parameter_grid.fee_bps.len()
        + request.parameter_grid.slippage_bps.len()
        + request.parameter_grid.latency_ms.len();
    if provided_values == 0 {
        return Err(json_bad_request(
            "bad_request",
            "parameter_grid must contain at least one execution-assumption value",
        ));
    }

    let mapped = map_frontend_runtime_config(&request.runtime_config).map_err(internal_error)?;
    let base = resolved_backtest_execution_assumptions(
        &mapped.runtime_protocol,
        request.backtest_options.execution_assumptions.as_ref(),
    );
    let fee_values = normalize_experiment_float_axis(
        &request.parameter_grid.fee_bps,
        base.taker_fee_bps,
        "fee_bps",
    )?;
    let slippage_values = normalize_experiment_float_axis(
        &request.parameter_grid.slippage_bps,
        base.default_slippage_bps,
        "slippage_bps",
    )?;
    let latency_values = normalize_experiment_latency_axis(
        &request.parameter_grid.latency_ms,
        base.latency_assumption_ms.unwrap_or(0),
    );

    let variant_count = fee_values.len() * slippage_values.len() * latency_values.len();
    if variant_count > MAX_EXPERIMENT_VARIANTS {
        return Err(json_bad_request(
            "bad_request",
            format!(
                "parameter sweep expands to {variant_count} variants, which exceeds the current limit of {MAX_EXPERIMENT_VARIANTS}"
            ),
        ));
    }

    let mut variants = Vec::with_capacity(variant_count);
    for fee_bps in fee_values {
        for slippage_bps in &slippage_values {
            for latency_ms in &latency_values {
                variants.push(FrontendExecutionAssumptionOverrides {
                    fee_bps: Some(fee_bps),
                    slippage_bps: Some(*slippage_bps),
                    latency_ms: Some(*latency_ms),
                });
            }
        }
    }

    Ok(variants)
}

async fn start_backtest_experiment(
    State(state): State<AppState>,
    Json(request): Json<FrontendExperimentRequest>,
) -> Result<Json<ExperimentDetailResponse>, (StatusCode, String)> {
    validate_runtime_config_capabilities(&request.runtime_config).map_err(|details| {
        json_bad_request_with_details(
            "capability_gated",
            "runtime config uses capabilities that are not enabled in the current beta",
            details,
        )
    })?;
    validate_backtest_execution_assumption_overrides(&request.backtest_options)
        .map_err(|message| json_bad_request("bad_request", message))?;

    let overrides = build_experiment_overrides(&request)?;
    let mapped = map_frontend_runtime_config(&request.runtime_config).map_err(internal_error)?;
    let base_execution_assumptions = resolved_backtest_execution_assumptions(
        &mapped.runtime_protocol,
        request.backtest_options.execution_assumptions.as_ref(),
    );
    let replay_source = request
        .backtest_options
        .replay_source
        .unwrap_or(FrontendBacktestReplaySource::HistoricalReplay);
    let experiment_id = format!("experiment_{}", current_time_ms());
    let created_at_ms = current_time_ms();
    let actor = normalize_actor_identity(request.actor.clone());
    let experiment_name = request
        .experiment_name
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());

    let mut variants = Vec::with_capacity(overrides.len());
    for (index, override_values) in overrides.iter().enumerate() {
        let variant_request = FrontendRunRequest {
            actor: request.actor.clone(),
            runtime_config: request.runtime_config.clone(),
            runtime_targets: request.runtime_targets.clone(),
            backtest_options: FrontendBacktestOptions {
                replay_source: Some(replay_source),
                execution_assumptions: Some(override_values.clone()),
            },
        };
        let record = execute_backtest_request(
            &state,
            &variant_request,
            Some(&format!("{}_v{}", experiment_id, index + 1)),
        )
        .await?;
        let summary = record
            .backtest_artifacts
            .as_ref()
            .map(|artifacts| artifacts.metrics.summary.clone())
            .unwrap_or_else(|| record.backtest.summary.clone());
        let execution_assumptions_tag = record
            .backtest_artifacts
            .as_ref()
            .and_then(|artifacts| artifacts.metrics.execution_assumptions.clone())
            .map(|module| module.list_tag);
        variants.push(ExperimentVariantSummary {
            variant_id: format!("variant_{}", index + 1),
            backtest_id: record.backtest_id,
            created_at_ms: record.created_at_ms,
            fee_bps: override_values.fee_bps.unwrap_or(0.0),
            slippage_bps: override_values.slippage_bps.unwrap_or(0.0),
            latency_ms: override_values.latency_ms.unwrap_or(0),
            summary,
            execution_assumptions_tag,
        });
    }

    let record = ExperimentRecord {
        experiment_id: experiment_id.clone(),
        graph_id: request.runtime_config.metadata.graph_id.clone(),
        compile_id: request.runtime_config.metadata.compile_id.clone(),
        created_at_ms,
        definition: ExperimentDefinitionSummary {
            experiment_name,
            replay_source,
            base_execution_assumptions: FrontendExecutionAssumptionOverrides {
                fee_bps: Some(base_execution_assumptions.taker_fee_bps),
                slippage_bps: Some(base_execution_assumptions.default_slippage_bps),
                latency_ms: base_execution_assumptions.latency_assumption_ms,
            },
            parameter_grid: request.parameter_grid,
        },
        variants,
        actor: Some(actor.clone()),
    };

    persist_experiment_record(state.experiment_store_dir.as_ref(), &record)
        .await
        .map_err(io_error)?;
    persist_graph_audit_entry(
        &state.audit_store_dir,
        &build_graph_audit_entry(
            &record.graph_id,
            &actor,
            GraphAuditAction::ExperimentCreated,
            Some(record.experiment_id.clone()),
            format!("Started backtest sweep {}", record.experiment_id),
        ),
    )
    .await
    .map_err(io_error)?;
    state
        .experiments
        .write()
        .await
        .insert(experiment_id, record.clone());

    Ok(Json(experiment_detail_response_from_record(record)))
}

async fn list_backtests(
    State(state): State<AppState>,
) -> Result<Json<Vec<BacktestListItem>>, (StatusCode, String)> {
    let records = list_backtest_records(state.backtest_store_dir.as_ref())
        .await
        .map_err(io_error)?;
    let mut items = records
        .into_iter()
        .map(backtest_list_item_from_record)
        .collect::<Vec<_>>();
    items.sort_by(|left, right| right.created_at_ms.cmp(&left.created_at_ms));
    Ok(Json(items))
}

async fn get_backtest_detail(
    State(state): State<AppState>,
    Path(backtest_id): Path<String>,
) -> Result<Json<BacktestDetailResponse>, (StatusCode, String)> {
    let record = load_backtest_record_from_state(&state, &backtest_id).await?;
    Ok(Json(backtest_detail_response_from_record(record)))
}

async fn list_runs(
    State(state): State<AppState>,
) -> Result<Json<Vec<RunListItem>>, (StatusCode, String)> {
    let records = list_run_records(state.run_store_dir.as_ref())
        .await
        .map_err(io_error)?;
    let mut items = records
        .into_iter()
        .map(run_list_item_from_record)
        .collect::<Vec<_>>();
    items.sort_by(|left, right| right.created_at_ms.cmp(&left.created_at_ms));
    Ok(Json(items))
}

async fn list_experiments(
    State(state): State<AppState>,
) -> Result<Json<Vec<ExperimentListItem>>, (StatusCode, String)> {
    let records = list_experiment_records(state.experiment_store_dir.as_ref())
        .await
        .map_err(io_error)?;
    let mut items = records
        .into_iter()
        .map(experiment_list_item_from_record)
        .collect::<Vec<_>>();
    items.sort_by(|left, right| right.created_at_ms.cmp(&left.created_at_ms));
    Ok(Json(items))
}

async fn get_run_detail(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<RunDetailResponse>, (StatusCode, String)> {
    let record = load_run_record_from_state(&state, &run_id).await?;
    Ok(Json(run_detail_response_from_record(record)))
}

async fn get_run_replay(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Query(query): Query<RuntimeReplayQuery>,
) -> Result<Json<RuntimeReplayResponse>, (StatusCode, String)> {
    let (cursor, limit) = normalized_replay_window(query);
    let record = load_run_record_from_state(&state, &run_id).await?;
    Ok(Json(run_replay_response_from_record(record, cursor, limit)))
}

async fn get_experiment_detail(
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<ExperimentDetailResponse>, (StatusCode, String)> {
    let record = load_experiment_record_from_state(&state, &experiment_id).await?;
    Ok(Json(experiment_detail_response_from_record(record)))
}

async fn get_backtest_replay(
    State(state): State<AppState>,
    Path(backtest_id): Path<String>,
    Query(query): Query<RuntimeReplayQuery>,
) -> Result<Json<RuntimeReplayResponse>, (StatusCode, String)> {
    let (cursor, limit) = normalized_replay_window(query);
    let record = load_backtest_record_from_state(&state, &backtest_id).await?;
    Ok(Json(backtest_replay_response_from_record(
        record, cursor, limit,
    )))
}

async fn stream_run_events(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)>
{
    let record = load_run_record_from_state(&state, &run_id).await?;

    let stream = stream! {
        yield Ok(json_sse_event("run_started", serde_json::json!({
            "run_id": record.run_id,
            "graph_id": record.graph_id,
            "compile_id": record.compile_id,
            "status": "started"
        })));

        for event in record.events {
            yield Ok(json_sse_event("runtime_event", &event));
            sleep(Duration::from_millis(SSE_EVENT_DELAY_MS)).await;
        }

        yield Ok(json_sse_event("account", &record.account));

        yield Ok(json_sse_event("run_completed", serde_json::json!({
            "run_id": record.run_id,
            "status": "completed",
            "event_count": record.session.slow_cycle.runtime_events.len() + record.session.fast_cycle.runtime_events.len(),
        })));
    };

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(5))
            .text("keepalive"),
    ))
}

async fn get_run_status(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<RunStatusResponse>, (StatusCode, String)> {
    let record = load_run_record_from_state(&state, &run_id).await?;
    Ok(Json(run_status_response_from_record(record)))
}
use axum::extract::Query;
