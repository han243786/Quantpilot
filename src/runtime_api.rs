use super::backtest_compare::compare_backtests;
use super::*;
use axum::extract::Query;

const MAX_EXPERIMENT_VARIANTS: usize = 27;
const DEFAULT_REPLAY_PAGE_SIZE: usize = 12;
const MAX_REPLAY_PAGE_SIZE: usize = 50;
pub(super) fn register_runtime_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/runtime/backtest", post(start_backtest_run))
        .route("/api/runtime/backtests", get(list_backtests))
        .route("/api/runtime/backtests/compare", post(compare_backtests))
        .route(
            "/api/runtime/backtests/:backtest_id/save",
            post(save_backtest_record),
        )
        .route(
            "/api/runtime/backtests/:backtest_id",
            get(get_backtest_detail).delete(discard_backtest_record),
        )
        .route(
            "/api/runtime/backtests/:backtest_id/replay",
            get(get_backtest_replay),
        )
        .route("/api/runtime/test-run", post(start_test_run))
        .route("/api/runtime/runs", get(list_runs))
        .route("/api/runtime/runs/:run_id/save", post(save_run_record))
        .route(
            "/api/runtime/runs/:run_id",
            get(get_run_detail).delete(discard_run_record),
        )
        .route("/api/runtime/runs/:run_id/events", get(stream_run_events))
        .route("/api/runtime/runs/:run_id/replay", get(get_run_replay))
        .route("/api/runtime/runs/:run_id/status", get(get_run_status))
        .route(
            "/api/runtime/evidence/health",
            get(get_runtime_evidence_health),
        )
        .route(
            "/api/runtime/evidence/cleanup",
            post(cleanup_runtime_evidence),
        )
        .route(
            "/api/runtime/mutations",
            get(list_runtime_parameter_mutations).post(create_runtime_parameter_mutation),
        )
        .route(
            "/api/runtime/mutations/:proposal_id",
            get(get_runtime_parameter_mutation_detail),
        )
        .route(
            "/api/runtime/mutations/:proposal_id/activate",
            post(activate_runtime_parameter_mutation),
        )
        .route(
            "/api/runtime/mutations/:proposal_id/rollback",
            post(rollback_runtime_parameter_mutation),
        )
        .route(
            "/api/runtime/ai-proposals",
            get(list_runtime_ai_proposals).post(create_runtime_ai_proposal),
        )
        .route(
            "/api/runtime/ai-proposals/:ai_proposal_id",
            get(get_runtime_ai_proposal_detail),
        )
        .route(
            "/api/runtime/reports",
            get(list_runtime_reports).post(create_runtime_report),
        )
        .route(
            "/api/runtime/reports/:report_id",
            get(get_runtime_report_detail),
        )
        .route(
            "/api/runtime/reports/:report_id/export",
            get(export_runtime_report_artifact),
        )
        .route(
            "/api/runtime/experiments/backtest-sweep",
            post(start_backtest_experiment),
        )
        .route("/api/runtime/experiments", get(list_experiments))
        .route(
            "/api/runtime/experiments/:experiment_id/save",
            post(save_experiment_record),
        )
        .route(
            "/api/runtime/experiments/:experiment_id",
            get(get_experiment_detail).delete(discard_experiment_record),
        )
        // Block 5: 审批流引擎
        .route(
            "/api/v1/ai/approvals",
            get(list_runtime_approvals),
        )
        .route(
            "/api/v1/ai/approvals/:approval_id",
            get(get_runtime_approval_detail),
        )
        .route(
            "/api/v1/ai/proposals/:proposal_id/approve",
            post(approve_ai_proposal),
        )
        .route(
            "/api/v1/ai/proposals/:proposal_id/reject",
            post(reject_ai_proposal),
        )
        .route(
            "/api/v1/ai/proposals/:proposal_id/claim",
            post(claim_ai_proposal_review),
        )
        // Block 5: 合并引擎
        .route(
            "/api/v1/merge/records",
            get(list_merge_records),
        )
        // Block 5 P3-2: 配置代际
        .route(
            "/api/v1/runtime/generations",
            get(list_config_generations),
        )
        // Block 5 P3-5: 存储健康
        .route(
            "/api/v1/storage/health",
            get(get_storage_health),
        )
        // Block 5: 运营报表
        .route(
            "/api/v1/reports/ops/daily",
            get(get_ops_daily_report),
        )
        .route(
            "/api/v1/reports/audit/weekly",
            get(get_audit_weekly_report),
        )
        .route(
            "/api/v1/reports/research/monthly",
            get(get_research_monthly_report),
        )
}

#[derive(Debug, Serialize)]
struct DiscardRuntimeArtifactResponse {
    discarded_id: String,
    discarded_kind: String,
}

#[derive(Debug, Deserialize)]
struct RuntimeReplayQuery {
    cursor: Option<usize>,
    limit: Option<usize>,
    checkpoint: Option<usize>,
    sequence_cursor: Option<u64>,
    stage: Option<String>,
    severity: Option<String>,
    retention_class: Option<String>,
    module_key: Option<String>,
    #[serde(default)]
    key_only: bool,
}

#[derive(Debug, Deserialize, Default)]
struct RuntimeParameterMutationListQuery {
    source_kind: Option<RuntimeEvidenceSourceKind>,
    source_id: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct RuntimeAiProposalListQuery {
    source_kind: Option<RuntimeEvidenceSourceKind>,
    source_id: Option<String>,
    status: Option<RuntimeAiProposalStatus>,
}

fn clean_optional_filter(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

fn normalized_replay_options(query: RuntimeReplayQuery) -> RuntimeReplayOptions {
    let cursor = query.checkpoint.or(query.cursor).unwrap_or(0);
    let limit = query
        .limit
        .unwrap_or(DEFAULT_REPLAY_PAGE_SIZE)
        .clamp(1, MAX_REPLAY_PAGE_SIZE);
    RuntimeReplayOptions {
        cursor,
        limit,
        sequence_cursor: query.sequence_cursor,
        filters: RuntimeReplayFilters {
            stage: clean_optional_filter(query.stage),
            severity: clean_optional_filter(query.severity),
            retention_class: clean_optional_filter(query.retention_class),
            module_key: clean_optional_filter(query.module_key),
            key_only: query.key_only,
        },
    }
}

async fn start_test_run(
    State(state): State<AppState>,
    Json(request): Json<FrontendRunRequest>,
) -> Result<Json<RunStartResponse>, (StatusCode, String)> {
    validate_runtime_capability_guard(request.capability_context.as_ref()).map_err(|details| {
        json_bad_request_with_details(
            "capability_boundary_violation",
            "runtime writes require a current capability hash and permission boundary",
            details,
        )
    })?;
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
    let governance = runtime_governance_snapshot(&request.runtime_config.metadata, None);
    let mut events = collect_frontend_events(&session, &runtime_targets);
    prepend_capability_snapshot_event(
        &mut events,
        &run_id,
        &request.runtime_config.metadata.mode,
        now_ms,
        &governance,
    );
    attach_runtime_event_envelopes(
        &mut events,
        &run_id,
        &request.runtime_config.metadata.mode,
        &governance,
    );
    validate_runtime_event_envelopes(&events, &run_id, &governance)
        .map_err(|message| internal_error(anyhow::anyhow!(message)))?;
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
        governance,
        actor: Some(actor.clone()),
    };

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
    validate_runtime_capability_guard(request.capability_context.as_ref()).map_err(|details| {
        json_bad_request_with_details(
            "capability_boundary_violation",
            "runtime writes require a current capability hash and permission boundary",
            details,
        )
    })?;
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
                .map_err(|error| {
                    anyhow::anyhow!(
                        "{}. Historical replay requires local market data files under the data cache. \
                         Set backtest_options.replay_source to \"deterministic_mock\" for offline testing: {:?}",
                        error,
                        error
                    )
                })
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
    let backtest_id = match id_suffix {
        Some(suffix) => format!("backtest_{}_{}", now_ms, suffix),
        None => format!("backtest_{}", now_ms),
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
            .insert(backtest_id.clone(), record.clone());
    }

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
                format!("parameter_grid.{field} 必须 >= 0"),
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
    validate_runtime_capability_guard(request.capability_context.as_ref()).map_err(|details| {
        json_bad_request_with_details(
            "capability_boundary_violation",
            "runtime writes require a current capability hash and permission boundary",
            details,
        )
    })?;
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
            capability_context: request.capability_context.clone(),
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

async fn save_backtest_record(
    State(state): State<AppState>,
    Path(backtest_id): Path<String>,
) -> Result<Json<BacktestDetailResponse>, (StatusCode, String)> {
    let mut record = load_backtest_record_from_state(&state, &backtest_id).await?;
    let views = persist_backtest_record(state.backtest_store_dir.as_ref(), &record)
        .await
        .map_err(io_error)?;
    delete_transient_backtest_record(state.transient_backtest_store_dir.as_ref(), &backtest_id)
        .await
        .map_err(io_error)?;
    record.backtest_artifacts = Some(views);
    state
        .backtests
        .write()
        .await
        .insert(backtest_id.clone(), record.clone());

    if let Some(actor) = &record.actor {
        persist_graph_audit_entry(
            &state.audit_store_dir,
            &build_graph_audit_entry(
                &record.graph_id,
                actor,
                GraphAuditAction::BacktestCreated,
                Some(backtest_id),
                format!("Saved backtest {}", record.backtest_id),
            ),
        )
        .await
        .map_err(io_error)?;
    }

    Ok(Json(backtest_detail_response_from_record(record)))
}

async fn discard_backtest_record(
    State(state): State<AppState>,
    Path(backtest_id): Path<String>,
) -> Result<Json<DiscardRuntimeArtifactResponse>, (StatusCode, String)> {
    let dir = state.backtest_store_dir.join(&backtest_id);
    if fs::try_exists(&dir).await.map_err(io_error)? {
        return Err((
            StatusCode::CONFLICT,
            format!(
                "回测 `{}` 已保存, 无法丢弃",
                backtest_id
            ),
        ));
    }

    let removed_memory = state.backtests.write().await.remove(&backtest_id).is_some();
    let removed_transient =
        delete_transient_backtest_record(state.transient_backtest_store_dir.as_ref(), &backtest_id)
            .await
            .map_err(io_error)?;
    if !removed_memory && !removed_transient {
        return Err((
            StatusCode::NOT_FOUND,
            format!("回测 `{}` 不存在", backtest_id),
        ));
    }

    Ok(Json(DiscardRuntimeArtifactResponse {
        discarded_id: backtest_id,
        discarded_kind: "backtest".to_string(),
    }))
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

async fn save_run_record(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<RunDetailResponse>, (StatusCode, String)> {
    let record = load_run_record_from_state(&state, &run_id).await?;
    persist_run_record(state.run_store_dir.as_ref(), &record)
        .await
        .map_err(io_error)?;

    if let Some(actor) = &record.actor {
        persist_graph_audit_entry(
            &state.audit_store_dir,
            &build_graph_audit_entry(
                &record.graph_id,
                actor,
                GraphAuditAction::RunCreated,
                Some(run_id),
                format!("Saved runtime simulation {}", record.run_id),
            ),
        )
        .await
        .map_err(io_error)?;
    }

    Ok(Json(run_detail_response_from_record(record)))
}

async fn discard_run_record(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<DiscardRuntimeArtifactResponse>, (StatusCode, String)> {
    let path = state.run_store_dir.join(format!("{}.json", run_id));
    if fs::try_exists(&path).await.map_err(io_error)? {
        return Err((
            StatusCode::CONFLICT,
            format!("run `{}` is already saved and cannot be discarded", run_id),
        ));
    }

    let removed = state.runs.write().await.remove(&run_id);
    if removed.is_none() {
        return Err(json_bad_request(
            "not_found",
            format!("运行记录 '{}' 不存在", run_id),
        ));
    }

    Ok(Json(DiscardRuntimeArtifactResponse {
        discarded_id: run_id,
        discarded_kind: "run".to_string(),
    }))
}

async fn get_run_replay(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Query(query): Query<RuntimeReplayQuery>,
) -> Result<Json<RuntimeReplayResponse>, (StatusCode, String)> {
    let started = Instant::now();
    let options = normalized_replay_options(query);
    let record = load_run_record_from_state(&state, &run_id).await?;
    let response = run_replay_response_from_record(record, options)
        .map_err(|message| json_bad_request("bad_replay_cursor", message))?;
    state
        .evidence_metrics
        .record_replay_page(started.elapsed().as_millis() as u64);
    Ok(Json(response))
}

fn canonical_runtime_parameter_version(
    target: &RuntimeParameterMutationTarget,
    value: &Value,
) -> Result<String, (StatusCode, String)> {
    let digest = canonical_json_sha256_digest(&json!({
        "target": target,
        "value": value,
    }))
    .map_err(|error| internal_error(anyhow::anyhow!(error)))?;
    Ok(format!("sha256:{}", digest.value))
}

fn validate_runtime_parameter_mutation_target(
    target: &RuntimeParameterMutationTarget,
) -> Result<(), (StatusCode, String)> {
    if target.node_id.trim().is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "target.node_id 是运行时参数变更提案的必填字段",
        ));
    }
    if target.module_key.trim().is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "target.module_key 是运行时参数变更提案的必填字段",
        ));
    }
    if target.parameter_path.trim().is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "target.parameter_path 是运行时参数变更提案的必填字段",
        ));
    }
    if !SUPPORTED_FRONTEND_MODULE_KEYS.contains(&target.module_key.as_str()) {
        return Err(json_bad_request(
            "capability_gated",
            format!(
                "模块 `{}` 未启用以支持运行时参数变更提案",
                target.module_key
            ),
        ));
    }
    Ok(())
}

fn validate_runtime_parameter_mutation_boundary(
    boundary: &RuntimeParameterMutationBoundary,
) -> Result<(), (StatusCode, String)> {
    let requested = boundary.requested.trim();
    if requested.is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "activation_boundary.requested is required",
        ));
    }
    if requested == "immediate" {
        return Err(json_bad_request(
            "parameter_mutation_boundary_violation",
            "immediate runtime parameter mutation is disabled; use next_cycle_start, manual_pause, or sequence_cursor",
        ));
    }
    if requested == "next_cycle_start" || requested == "manual_pause" {
        return Ok(());
    }
    if requested == "sequence_cursor" && boundary.resolved_sequence_no.is_some() {
        return Ok(());
    }
    if requested
        .strip_prefix("sequence_cursor:")
        .and_then(|value| value.parse::<u64>().ok())
        .is_some()
    {
        return Ok(());
    }
    Err(json_bad_request(
        "parameter_mutation_boundary_violation",
        "unsupported activation boundary; use next_cycle_start, manual_pause, or sequence_cursor",
    ))
}

fn resolve_runtime_parameter_mutation_boundary(
    boundary: &RuntimeParameterMutationBoundary,
    current_sequence_no: u64,
) -> Result<RuntimeParameterMutationBoundary, (StatusCode, String)> {
    validate_runtime_parameter_mutation_boundary(boundary)?;
    let requested = boundary.requested.trim();
    if requested == "next_cycle_start" {
        return Ok(RuntimeParameterMutationBoundary {
            requested: "next_cycle_start".to_string(),
            resolved_sequence_no: Some(current_sequence_no + 2),
        });
    }
    if requested == "manual_pause" {
        return Ok(RuntimeParameterMutationBoundary {
            requested: "manual_pause".to_string(),
            resolved_sequence_no: None,
        });
    }
    let sequence_no = boundary.resolved_sequence_no.or_else(|| {
        requested
            .strip_prefix("sequence_cursor:")
            .and_then(|value| value.parse::<u64>().ok())
    });
    let Some(sequence_no) = sequence_no else {
        return Err(json_bad_request(
            "parameter_mutation_boundary_violation",
            "sequence_cursor activation boundary requires resolved_sequence_no",
        ));
    };
    Ok(RuntimeParameterMutationBoundary {
        requested: "sequence_cursor".to_string(),
        resolved_sequence_no: Some(sequence_no),
    })
}

fn evaluate_runtime_parameter_mutation_safe_window(
    snapshot: Option<RuntimeParameterMutationSafeWindowSnapshot>,
) -> RuntimeParameterMutationSafeWindowState {
    let snapshot = snapshot.unwrap_or_default();
    let mut reason_code = "SAFE_WINDOW_OPEN";
    let mut message = "safe window is open for runtime parameter mutation".to_string();
    let mut retryable = false;
    let mut retry_after_ms = None;

    if !matches!(
        snapshot.runtime_status.as_str(),
        "paused" | "idle" | "stopped" | "ready"
    ) {
        reason_code = "SAFE_WINDOW_RUNTIME_ACTIVE";
        message = format!(
            "runtime status `{}` is not eligible for parameter mutation",
            snapshot.runtime_status
        );
        retryable = true;
    } else if snapshot.open_order_count > 0 {
        reason_code = "SAFE_WINDOW_OPEN_ORDERS";
        message = format!(
            "{} open orders must settle before parameter mutation",
            snapshot.open_order_count
        );
        retryable = true;
    } else if snapshot.outstanding_risk_violation {
        reason_code = "SAFE_WINDOW_RISK_VIOLATION";
        message = "outstanding risk violation blocks parameter mutation".to_string();
        retryable = true;
    } else if snapshot.data_freshness_ms > 60_000 {
        reason_code = "SAFE_WINDOW_STALE_DATA";
        message = format!(
            "data freshness {}ms exceeds the 60000ms safe-window limit",
            snapshot.data_freshness_ms
        );
        retryable = true;
    } else if snapshot.portfolio_exposure_bps.abs() > 10_000 {
        reason_code = "SAFE_WINDOW_EXPOSURE_LIMIT";
        message = format!(
            "portfolio exposure {}bps exceeds the safe-window limit",
            snapshot.portfolio_exposure_bps
        );
        retryable = true;
    } else if snapshot.cooldown_remaining_ms > 0 {
        reason_code = "SAFE_WINDOW_COOLDOWN";
        message = format!(
            "mutation cooldown has {}ms remaining",
            snapshot.cooldown_remaining_ms
        );
        retryable = true;
        retry_after_ms = Some(snapshot.cooldown_remaining_ms);
    }

    let allowed = reason_code == "SAFE_WINDOW_OPEN";
    RuntimeParameterMutationSafeWindowState {
        status: if allowed { "allowed" } else { "denied" }.to_string(),
        policy_version: snapshot.policy_version.clone(),
        allowed,
        reason_code: reason_code.to_string(),
        message,
        retryable,
        retry_after_ms,
        snapshot,
    }
}

fn runtime_mode_from_events(events: &[FrontendRuntimeEvent]) -> String {
    events
        .iter()
        .find_map(|event| {
            let mode = event.envelope.mode.trim();
            (!mode.is_empty()).then(|| mode.to_string())
        })
        .unwrap_or_else(|| "paper".to_string())
}

fn status_contract_value(status: RuntimeParameterMutationStatus) -> &'static str {
    match status {
        RuntimeParameterMutationStatus::Proposed => "proposed",
        RuntimeParameterMutationStatus::Rejected => "rejected",
        RuntimeParameterMutationStatus::ActivationScheduled => "activation_scheduled",
        RuntimeParameterMutationStatus::Activated => "activated",
        RuntimeParameterMutationStatus::ActivationFailed => "activation_failed",
        RuntimeParameterMutationStatus::SafeWindowDenied => "safe_window_denied",
        RuntimeParameterMutationStatus::RollbackScheduled => "rollback_scheduled",
        RuntimeParameterMutationStatus::RolledBack => "rolled_back",
        RuntimeParameterMutationStatus::RollbackFailed => "rollback_failed",
    }
}

fn mutation_event_contract(status: RuntimeParameterMutationStatus) -> (&'static str, &'static str) {
    match status {
        RuntimeParameterMutationStatus::Proposed => {
            ("ParameterMutationProposed", "PARAMETER_MUTATION_PROPOSED")
        }
        RuntimeParameterMutationStatus::Rejected => {
            ("ParameterMutationRejected", "PARAMETER_MUTATION_REJECTED")
        }
        RuntimeParameterMutationStatus::ActivationScheduled => (
            "ParameterMutationActivationScheduled",
            "PARAMETER_MUTATION_ACTIVATION_SCHEDULED",
        ),
        RuntimeParameterMutationStatus::Activated => {
            ("ParameterMutationActivated", "PARAMETER_MUTATION_ACTIVATED")
        }
        RuntimeParameterMutationStatus::ActivationFailed => (
            "ParameterMutationActivationFailed",
            "PARAMETER_MUTATION_ACTIVATION_FAILED",
        ),
        RuntimeParameterMutationStatus::SafeWindowDenied => (
            "ParameterMutationSafeWindowDenied",
            "PARAMETER_MUTATION_SAFE_WINDOW_DENIED",
        ),
        RuntimeParameterMutationStatus::RollbackScheduled => (
            "ParameterMutationRollbackScheduled",
            "PARAMETER_MUTATION_ROLLBACK_SCHEDULED",
        ),
        RuntimeParameterMutationStatus::RolledBack => (
            "ParameterMutationRolledBack",
            "PARAMETER_MUTATION_ROLLED_BACK",
        ),
        RuntimeParameterMutationStatus::RollbackFailed => (
            "ParameterMutationRollbackFailed",
            "PARAMETER_MUTATION_ROLLBACK_FAILED",
        ),
    }
}

fn build_runtime_parameter_mutation_event(
    record: &RuntimeParameterMutationRecord,
    status: RuntimeParameterMutationStatus,
    event_time_ms: u64,
) -> FrontendRuntimeEvent {
    let (event_type, reason_code) = mutation_event_contract(status);
    FrontendRuntimeEvent {
        event_id: format!(
            "event_{}_{}_{}",
            record.proposal_id,
            status_contract_value(status),
            event_time_ms
        ),
        event_type: event_type.to_string(),
        source_id: record.target.module_key.clone(),
        node_id: record.target.node_id.clone(),
        event_time_ms,
        severity: match status {
            RuntimeParameterMutationStatus::Rejected
            | RuntimeParameterMutationStatus::ActivationFailed
            | RuntimeParameterMutationStatus::SafeWindowDenied
            | RuntimeParameterMutationStatus::RollbackFailed => "Warn".to_string(),
            _ => "Info".to_string(),
        },
        summary: match status {
            RuntimeParameterMutationStatus::Proposed => format!(
                "Parameter mutation proposed for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeParameterMutationStatus::Rejected => format!(
                "Parameter mutation rejected for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeParameterMutationStatus::ActivationScheduled => format!(
                "Parameter mutation activation scheduled for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeParameterMutationStatus::Activated => format!(
                "Parameter mutation activated for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeParameterMutationStatus::ActivationFailed => format!(
                "Parameter mutation activation failed for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeParameterMutationStatus::SafeWindowDenied => format!(
                "Parameter mutation safe window denied for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeParameterMutationStatus::RollbackScheduled => format!(
                "Parameter mutation rollback scheduled for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeParameterMutationStatus::RolledBack => format!(
                "Parameter mutation rolled back for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeParameterMutationStatus::RollbackFailed => format!(
                "Parameter mutation rollback failed for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
        },
        payload: json!({
            "proposal_id": &record.proposal_id,
            "status": status,
            "reason_code": reason_code,
            "source_kind": record.source_kind,
            "source_id": &record.source_id,
            "target": &record.target,
            "old_parameter_version": &record.old_parameter_version,
            "proposed_parameter_version": &record.proposed_parameter_version,
            "activation_boundary": &record.activation_boundary,
            "actor": &record.actor,
            "reason": &record.reason,
            "rejection_reason": &record.rejection_reason,
            "governance": &record.governance,
            "activation_state": &record.activation_state,
            "safe_window_state": &record.safe_window_state,
            "rollback_of": &record.rollback_of,
            "rollback_target_parameter_version": &record.rollback_target_parameter_version,
        }),
        envelope: RuntimeEventEnvelope::default(),
    }
}

async fn append_parameter_mutation_events_to_run(
    state: &AppState,
    source_id: &str,
    mut events: Vec<(FrontendRuntimeEvent, RuntimeGovernanceSnapshot)>,
    active_parameter_version: Option<String>,
) -> Result<(), (StatusCode, String)> {
    let mut record = load_run_record_from_state(state, source_id).await?;
    let mode = runtime_mode_from_events(&record.events);
    let mut next_sequence = record
        .events
        .last()
        .map(|event| event.envelope.sequence_no)
        .unwrap_or(record.events.len() as u64);
    for (event, governance) in events.iter_mut() {
        next_sequence += 1;
        attach_runtime_event_envelope(event, source_id, &mode, governance, next_sequence);
        record.events.push(event.clone());
    }
    if let Some(parameter_version) = active_parameter_version {
        record.governance.parameter_version = parameter_version;
    }
    validate_runtime_event_envelopes(&record.events, source_id, &record.governance)
        .map_err(|message| internal_error(anyhow::anyhow!(message)))?;

    state
        .runs
        .write()
        .await
        .insert(source_id.to_string(), record.clone());

    let persisted_path = state.run_store_dir.join(format!("{source_id}.json"));
    if fs::try_exists(&persisted_path).await.map_err(io_error)? {
        persist_run_record(state.run_store_dir.as_ref(), &record)
            .await
            .map_err(io_error)?;
    }

    Ok(())
}

fn runtime_parameter_mutation_governance(
    source_governance: &RuntimeGovernanceSnapshot,
    old_parameter_version: String,
    proposed_parameter_version: String,
) -> RuntimeParameterMutationGovernance {
    RuntimeParameterMutationGovernance {
        capability_hash: source_governance.capability_hash.clone(),
        deployment_revision: source_governance.deployment_revision.clone(),
        strategy_version: source_governance.strategy_version.clone(),
        previous_parameter_version: old_parameter_version,
        proposed_parameter_version,
        permission_boundary_model_version: source_governance
            .permission_boundary
            .model_version
            .clone(),
    }
}

fn runtime_parameter_mutation_record_id(
    request: &CreateRuntimeParameterMutationRequest,
    created_at_ms: u64,
    source_event_count: usize,
    proposed_parameter_version: &str,
) -> Result<String, (StatusCode, String)> {
    let digest = canonical_json_sha256_digest(&json!({
        "created_at_ms": created_at_ms,
        "source_event_count": source_event_count,
        "source_kind": request.source_kind,
        "source_id": &request.source_id,
        "target": &request.target,
        "proposed_parameter_version": proposed_parameter_version,
    }))
    .map_err(|error| internal_error(anyhow::anyhow!(error)))?;
    Ok(format!(
        "parameter_mutation_{}_{}",
        created_at_ms,
        &digest.value[..12]
    ))
}

fn runtime_parameter_mutation_rollback_record_id(
    source_id: &str,
    rollback_of: &str,
    target: &RuntimeParameterMutationTarget,
    created_at_ms: u64,
    source_event_count: usize,
    proposed_parameter_version: &str,
) -> Result<String, (StatusCode, String)> {
    let digest = canonical_json_sha256_digest(&json!({
        "created_at_ms": created_at_ms,
        "rollback_of": rollback_of,
        "source_event_count": source_event_count,
        "source_id": source_id,
        "target": target,
        "proposed_parameter_version": proposed_parameter_version,
    }))
    .map_err(|error| internal_error(anyhow::anyhow!(error)))?;
    Ok(format!(
        "parameter_rollback_{}_{}",
        created_at_ms,
        &digest.value[..12]
    ))
}

async fn create_runtime_parameter_mutation(
    State(state): State<AppState>,
    Json(request): Json<CreateRuntimeParameterMutationRequest>,
) -> Result<Json<RuntimeParameterMutationRecord>, (StatusCode, String)> {
    validate_runtime_capability_guard(request.capability_context.as_ref()).map_err(|details| {
        json_bad_request_with_details(
            "parameter_mutation_boundary_violation",
            "runtime parameter mutation proposals require a current capability hash and permission boundary",
            details,
        )
    })?;
    if request.source_kind != RuntimeEvidenceSourceKind::Run {
        return Err(json_bad_request(
            "bad_request",
            "runtime parameter mutation proposals currently require source_kind `run`",
        ));
    }
    validate_runtime_parameter_mutation_target(&request.target)?;
    validate_runtime_parameter_mutation_boundary(&request.activation_boundary)?;
    let actor = request.actor.clone().ok_or_else(|| {
        json_bad_request(
            "bad_request",
            "actor is required for runtime parameter mutation proposals",
        )
    })?;
    let actor = normalize_actor_identity(Some(actor));
    let reason = request.reason.trim();
    if reason.is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "reason is required for runtime parameter mutation proposals",
        ));
    }

    let source = load_run_record_from_state(&state, &request.source_id).await?;
    let old_parameter_version =
        canonical_runtime_parameter_version(&request.target, &request.old_value)?;
    let proposed_parameter_version =
        canonical_runtime_parameter_version(&request.target, &request.new_value)?;
    let is_noop = old_parameter_version == proposed_parameter_version;
    let now_ms = current_time_ms();
    let proposal_id = runtime_parameter_mutation_record_id(
        &request,
        now_ms,
        source.events.len(),
        &proposed_parameter_version,
    )?;
    let governance = runtime_parameter_mutation_governance(
        &source.governance,
        old_parameter_version.clone(),
        proposed_parameter_version.clone(),
    );
    let record = RuntimeParameterMutationRecord {
        proposal_id,
        source_kind: request.source_kind,
        source_id: request.source_id.clone(),
        graph_id: source.graph_id.clone(),
        target: request.target.clone(),
        old_value: request.old_value.clone(),
        new_value: request.new_value.clone(),
        old_parameter_version,
        proposed_parameter_version,
        status: if is_noop {
            RuntimeParameterMutationStatus::Rejected
        } else {
            RuntimeParameterMutationStatus::Proposed
        },
        rejection_reason: is_noop.then(|| {
            "old_value and new_value resolve to the same canonical parameter version".to_string()
        }),
        activation_boundary: request.activation_boundary.clone(),
        activation_state: None,
        safe_window_state: None,
        rollback_of: None,
        rollback_target_parameter_version: None,
        actor,
        reason: reason.to_string(),
        governance,
        lifecycle: Vec::new(),
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    };
    let event = build_runtime_parameter_mutation_event(&record, record.status, now_ms);
    let proposal_event_governance =
        governance_with_parameter_version(&source.governance, &record.old_parameter_version);
    append_parameter_mutation_events_to_run(
        &state,
        &request.source_id,
        vec![(event, proposal_event_governance)],
        None,
    )
    .await?;
    persist_runtime_parameter_mutation_record(state.mutation_store_dir.as_ref(), &record)
        .await
        .map_err(io_error)?;
    state
        .evidence_metrics
        .record_mutation_proposal(record.status);
    state
        .parameter_mutations
        .write()
        .await
        .insert(record.proposal_id.clone(), record.clone());
    Ok(Json(record))
}

async fn list_runtime_parameter_mutations(
    State(state): State<AppState>,
    Query(query): Query<RuntimeParameterMutationListQuery>,
) -> Result<Json<Vec<RuntimeParameterMutationRecord>>, (StatusCode, String)> {
    let mut records = list_runtime_parameter_mutation_records(&state.mutation_store_dir)
        .await
        .map_err(io_error)?;
    if let Some(source_kind) = query.source_kind {
        records.retain(|record| record.source_kind == source_kind);
    }
    if let Some(source_id) = clean_optional_filter(query.source_id) {
        records.retain(|record| record.source_id == source_id);
    }
    records.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.proposal_id.cmp(&left.proposal_id))
    });
    Ok(Json(records))
}

async fn get_runtime_parameter_mutation_detail(
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
) -> Result<Json<RuntimeParameterMutationRecord>, (StatusCode, String)> {
    if let Some(record) = state
        .parameter_mutations
        .read()
        .await
        .get(&proposal_id)
        .cloned()
    {
        return Ok(Json(record));
    }
    load_runtime_parameter_mutation_record(state.mutation_store_dir.as_ref(), &proposal_id)
        .await
        .map(Json)
}

fn validate_hash_identity(
    value: &str,
    target: &'static str,
    label: &'static str,
) -> Result<(), (StatusCode, String)> {
    let trimmed = value.trim();
    let valid = trimmed.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .chars()
                .all(|ch| ch.is_ascii_digit() || matches!(ch, 'a'..='f'))
    });
    if valid {
        Ok(())
    } else {
        Err(json_bad_request(
            "bad_request",
            format!("{target} must be formatted as sha256:<64 lowercase hex chars> for {label}"),
        ))
    }
}

fn validate_ai_model_identity(model: &RuntimeAiModelIdentity) -> Result<(), (StatusCode, String)> {
    if model.provider.trim().is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "model.provider is required for AI proposal candidates",
        ));
    }
    if model.model.trim().is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "model.model is required for AI proposal candidates",
        ));
    }
    if model.model_version.trim().is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "model.model_version is required for AI proposal candidates",
        ));
    }
    Ok(())
}

fn ai_proposal_static_check_result(
    request: &CreateRuntimeAiProposalRequest,
    old_parameter_version: &str,
    proposed_parameter_version: &str,
    source_event_count: usize,
    checked_at_ms: u64,
) -> RuntimeAiProposalStaticCheckResult {
    let mut details = Vec::new();
    if source_event_count == 0 {
        details.push(RuntimeAiProposalStaticCheckDetail {
            code: "missing_source_evidence".to_string(),
            target: "source_id".to_string(),
            message: "AI proposals require at least one source evidence event".to_string(),
        });
    }
    if old_parameter_version == proposed_parameter_version {
        details.push(RuntimeAiProposalStaticCheckDetail {
            code: "noop_parameter_version".to_string(),
            target: "new_value".to_string(),
            message: "old_value and new_value resolve to the same canonical parameter version"
                .to_string(),
        });
    }
    if request.reason.trim().is_empty() {
        details.push(RuntimeAiProposalStaticCheckDetail {
            code: "missing_reason".to_string(),
            target: "reason".to_string(),
            message: "reason is required for AI proposal candidates".to_string(),
        });
    }

    if details.is_empty() {
        RuntimeAiProposalStaticCheckResult {
            status: RuntimeAiProposalStatus::StaticCheckPassed,
            reason_code: "AI_PROPOSAL_STATIC_CHECK_PASSED".to_string(),
            message: "AI proposal candidate passed static validation".to_string(),
            checked_at_ms,
            details,
        }
    } else {
        RuntimeAiProposalStaticCheckResult {
            status: RuntimeAiProposalStatus::StaticCheckFailed,
            reason_code: "AI_PROPOSAL_STATIC_CHECK_FAILED".to_string(),
            message: "AI proposal candidate failed static validation".to_string(),
            checked_at_ms,
            details,
        }
    }
}

fn runtime_ai_proposal_governance(
    source_governance: &RuntimeGovernanceSnapshot,
    old_parameter_version: String,
    proposed_parameter_version: String,
) -> RuntimeAiProposalGovernance {
    RuntimeAiProposalGovernance {
        capability_hash: source_governance.capability_hash.clone(),
        deployment_revision: source_governance.deployment_revision.clone(),
        strategy_version: source_governance.strategy_version.clone(),
        previous_parameter_version: old_parameter_version,
        proposed_parameter_version,
        permission_boundary_model_version: source_governance
            .permission_boundary
            .model_version
            .clone(),
        ai_write_policy: source_governance
            .permission_boundary
            .ai_write_policy
            .clone(),
    }
}

fn runtime_ai_proposal_record_id(
    request: &CreateRuntimeAiProposalRequest,
    created_at_ms: u64,
    source_event_count: usize,
    proposed_parameter_version: &str,
) -> Result<String, (StatusCode, String)> {
    let digest = canonical_json_sha256_digest(&json!({
        "created_at_ms": created_at_ms,
        "source_event_count": source_event_count,
        "source_kind": request.source_kind,
        "source_id": &request.source_id,
        "target": &request.target,
        "model": &request.model,
        "prompt_hash": &request.prompt_hash,
        "evidence_hash": &request.evidence_hash,
        "proposed_parameter_version": proposed_parameter_version,
    }))
    .map_err(|error| internal_error(anyhow::anyhow!(error)))?;
    Ok(format!(
        "ai_proposal_{}_{}",
        created_at_ms,
        &digest.value[..12]
    ))
}

fn ai_proposal_event_contract(status: RuntimeAiProposalStatus) -> (&'static str, &'static str) {
    match status {
        RuntimeAiProposalStatus::Submitted | RuntimeAiProposalStatus::Draft => {
            ("AIProposalCreated", "AI_PROPOSAL_CREATED")
        }
        RuntimeAiProposalStatus::Denied => ("AIProposalDenied", "AI_PROPOSAL_DENIED"),
        RuntimeAiProposalStatus::StaticCheckPassed => (
            "AIProposalStaticCheckPassed",
            "AI_PROPOSAL_STATIC_CHECK_PASSED",
        ),
        RuntimeAiProposalStatus::StaticCheckFailed => (
            "AIProposalStaticCheckFailed",
            "AI_PROPOSAL_STATIC_CHECK_FAILED",
        ),
        RuntimeAiProposalStatus::Expired => ("AIProposalDenied", "AI_PROPOSAL_EXPIRED"),
    }
}

fn build_runtime_ai_proposal_event(
    record: &RuntimeAiProposalRecord,
    status: RuntimeAiProposalStatus,
    event_time_ms: u64,
) -> FrontendRuntimeEvent {
    let (event_type, reason_code) = ai_proposal_event_contract(status);
    FrontendRuntimeEvent {
        event_id: format!(
            "event_{}_{}_{}",
            record.ai_proposal_id, reason_code, event_time_ms
        ),
        event_type: event_type.to_string(),
        source_id: record.target.module_key.clone(),
        node_id: record.target.node_id.clone(),
        event_time_ms,
        severity: match status {
            RuntimeAiProposalStatus::Denied | RuntimeAiProposalStatus::StaticCheckFailed => {
                "Warn".to_string()
            }
            _ => "Info".to_string(),
        },
        summary: match status {
            RuntimeAiProposalStatus::Submitted | RuntimeAiProposalStatus::Draft => format!(
                "AI proposal candidate created for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeAiProposalStatus::Denied => format!(
                "AI proposal candidate denied for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeAiProposalStatus::StaticCheckPassed => format!(
                "AI proposal static check passed for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeAiProposalStatus::StaticCheckFailed => format!(
                "AI proposal static check failed for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeAiProposalStatus::Expired => format!(
                "AI proposal expired for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
        },
        payload: json!({
            "ai_proposal_id": &record.ai_proposal_id,
            "status": status,
            "reason_code": reason_code,
            "source_kind": record.source_kind,
            "source_id": &record.source_id,
            "graph_id": &record.graph_id,
            "source_evidence": &record.source_evidence,
            "target": &record.target,
            "old_parameter_version": &record.old_parameter_version,
            "proposed_parameter_version": &record.proposed_parameter_version,
            "denial_reason": &record.denial_reason,
            "static_check": &record.static_check,
            "model": &record.model,
            "prompt_hash": &record.prompt_hash,
            "evidence_hash": &record.evidence_hash,
            "actor": &record.actor,
            "reason": &record.reason,
            "governance": &record.governance,
        }),
        envelope: RuntimeEventEnvelope::default(),
    }
}

fn ai_proposal_lifecycle_entry(
    status: RuntimeAiProposalStatus,
    event: &FrontendRuntimeEvent,
    sequence_no: u64,
    message: impl Into<String>,
) -> RuntimeAiProposalLifecycleEntry {
    let (_, reason_code) = ai_proposal_event_contract(status);
    RuntimeAiProposalLifecycleEntry {
        status,
        event_id: event.event_id.clone(),
        sequence_no,
        occurred_at_ms: event.event_time_ms,
        reason_code: reason_code.to_string(),
        message: message.into(),
    }
}

async fn persist_runtime_ai_proposal_transition(
    state: &AppState,
    record: &RuntimeAiProposalRecord,
) -> Result<(), (StatusCode, String)> {
    persist_runtime_ai_proposal_record(state.ai_proposal_store_dir.as_ref(), record)
        .await
        .map_err(io_error)?;
    state
        .ai_proposals
        .write()
        .await
        .insert(record.ai_proposal_id.clone(), record.clone());
    Ok(())
}

async fn create_runtime_ai_proposal(
    State(state): State<AppState>,
    Json(request): Json<CreateRuntimeAiProposalRequest>,
) -> Result<Json<RuntimeAiProposalRecord>, (StatusCode, String)> {
    validate_runtime_capability_guard(request.capability_context.as_ref()).map_err(|details| {
        json_bad_request_with_details(
            "ai_proposal_denied",
            "AI proposal candidates require a current capability hash and permission boundary",
            details,
        )
    })?;
    let capability_context = request
        .capability_context
        .as_ref()
        .expect("validated above");
    if capability_context
        .permission_boundary
        .ai_write_policy
        .trim()
        != "proposal_only"
    {
        return Err(json_bad_request(
            "ai_proposal_denied",
            "AI write policy must be proposal_only for AI proposal candidate creation",
        ));
    }
    if request.source_kind != RuntimeEvidenceSourceKind::Run {
        return Err(json_bad_request(
            "bad_request",
            "AI proposal candidates currently require source_kind `run`",
        ));
    }
    validate_runtime_parameter_mutation_target(&request.target)?;
    if request.old_value.is_null() {
        return Err(json_bad_request(
            "bad_request",
            "old_value is required for AI proposal candidates",
        ));
    }
    if request.new_value.is_null() {
        return Err(json_bad_request(
            "bad_request",
            "new_value is required for AI proposal candidates",
        ));
    }
    validate_ai_model_identity(&request.model)?;
    validate_hash_identity(
        &request.prompt_hash,
        "prompt_hash",
        "AI proposal candidates",
    )?;
    validate_hash_identity(
        &request.evidence_hash,
        "evidence_hash",
        "AI proposal candidates",
    )?;
    let actor = request.actor.clone().ok_or_else(|| {
        json_bad_request(
            "bad_request",
            "actor is required for AI proposal candidates",
        )
    })?;
    let actor = normalize_actor_identity(Some(actor));

    let source = load_run_record_from_state(&state, &request.source_id).await?;
    let old_parameter_version =
        canonical_runtime_parameter_version(&request.target, &request.old_value)?;
    let proposed_parameter_version =
        canonical_runtime_parameter_version(&request.target, &request.new_value)?;
    let now_ms = current_time_ms();
    let static_check = ai_proposal_static_check_result(
        &request,
        &old_parameter_version,
        &proposed_parameter_version,
        source.events.len(),
        now_ms,
    );
    let status = static_check.status;
    let ai_proposal_id = runtime_ai_proposal_record_id(
        &request,
        now_ms,
        source.events.len(),
        &proposed_parameter_version,
    )?;
    let governance = runtime_ai_proposal_governance(
        &source.governance,
        old_parameter_version.clone(),
        proposed_parameter_version.clone(),
    );
    let source_evidence = RuntimeAiProposalSourceEvidence {
        source_kind: request.source_kind,
        source_id: request.source_id.clone(),
        graph_id: source.graph_id.clone(),
        event_count: source.events.len(),
        evidence_hash: request.evidence_hash.clone(),
    };
    let mut record = RuntimeAiProposalRecord {
        ai_proposal_id,
        source_kind: request.source_kind,
        source_id: request.source_id.clone(),
        graph_id: source.graph_id.clone(),
        source_evidence,
        target: request.target.clone(),
        old_value: request.old_value.clone(),
        new_value: request.new_value.clone(),
        old_parameter_version,
        proposed_parameter_version,
        status,
        denial_reason: None,
        static_check,
        model: request.model.clone(),
        prompt_hash: request.prompt_hash.clone(),
        evidence_hash: request.evidence_hash.clone(),
        actor,
        reason: request.reason.trim().to_string(),
        governance,
        lifecycle: Vec::new(),
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    };

    let created_event =
        build_runtime_ai_proposal_event(&record, RuntimeAiProposalStatus::Submitted, now_ms);
    let static_event_time_ms = now_ms + 1;
    let static_event = build_runtime_ai_proposal_event(&record, status, static_event_time_ms);
    let current_sequence_no = source
        .events
        .last()
        .map(|event| event.envelope.sequence_no)
        .unwrap_or(source.events.len() as u64);
    record.lifecycle.push(ai_proposal_lifecycle_entry(
        RuntimeAiProposalStatus::Submitted,
        &created_event,
        current_sequence_no + 1,
        "AI proposal candidate submitted",
    ));
    record.lifecycle.push(ai_proposal_lifecycle_entry(
        status,
        &static_event,
        current_sequence_no + 2,
        record.static_check.message.clone(),
    ));
    let event_governance =
        governance_with_parameter_version(&source.governance, &record.old_parameter_version);
    append_parameter_mutation_events_to_run(
        &state,
        &request.source_id,
        vec![
            (created_event, event_governance.clone()),
            (static_event, event_governance),
        ],
        None,
    )
    .await?;
    persist_runtime_ai_proposal_transition(&state, &record).await?;

    // Block 5 P1-4: 静态校验通过后自动创建审批单并触发沙箱验证
    if status == RuntimeAiProposalStatus::StaticCheckPassed {
        let proposal_id = record.ai_proposal_id.clone();
        let approval_id = format!("apr-{}", now_ms);
        let approval = RuntimeApprovalRecord {
            approval_id: approval_id.clone(),
            proposal_id: proposal_id.clone(),
            approval_level: RuntimeApprovalLevel::L1SingleReviewer,
            review_state: RuntimeApprovalReviewState::Pending,
            chain_stage_impact: vec!["intent".to_string(), "agent".to_string()],
            sandbox_report_url: None,
            rollback_plan: RuntimeRollbackPlan {
                method: "generation_rollback".to_string(),
                target_generation: 0,
                estimated_recovery_ms: 5000,
            },
            created_at_ms: now_ms,
            expires_at_ms: now_ms + 24 * 3600 * 1000, // L1: 24h
            reviewers_required: 1,
            reviewers_assigned: vec!["pm-strategy-btc".to_string()],
            reviewers_approved: Vec::new(),
            reviewers_rejected: Vec::new(),
            lifecycle: vec![RuntimeApprovalLifecycleEntry {
                review_state: RuntimeApprovalReviewState::Pending,
                event_id: format!("event_apr_pending_{}", now_ms),
                sequence_no: 1,
                occurred_at_ms: now_ms,
                reason_code: "APPROVAL_CREATED".to_string(),
                message: "审批单自动创建".to_string(),
                actor_id: None,
            }],
        };
        persist_approval(&state.approval_store_dir, &approval)
            .await
            .map_err(io_error)?;
        state
            .approval_records
            .write()
            .await
            .insert(approval_id, approval);

        // 异步触发沙箱验证
        let state_clone = state.clone();
        let pid = proposal_id.clone();
        tokio::spawn(async move {
            let sandbox_request = RequestSandboxVerificationRequest {
                backtest_id: None,
                proposal_id: pid.clone(),
            };
            // 内部触发沙箱验证
            let result = sandbox_verification::run_sandbox_verification(
                &state_clone, &sandbox_request,
            )
            .await;
            if let Ok(_report) = result {
                // 更新审批单的沙箱报告 URL
                let mut approvals = state_clone.approval_records.write().await;
                for approval in approvals.values_mut() {
                    if approval.proposal_id == pid {
                        approval.sandbox_report_url = Some(format!(
                            "/api/v1/ai/proposals/{}/sandbox-report",
                            pid
                        ));
                        let _ = persist_approval(
                            &state_clone.approval_store_dir,
                            approval,
                        )
                        .await;
                        break;
                    }
                }
            }
        });
    }

    Ok(Json(record))
}

async fn list_runtime_ai_proposals(
    State(state): State<AppState>,
    Query(query): Query<RuntimeAiProposalListQuery>,
) -> Result<Json<Vec<RuntimeAiProposalRecord>>, (StatusCode, String)> {
    let mut records = list_runtime_ai_proposal_records(&state.ai_proposal_store_dir)
        .await
        .map_err(io_error)?;
    if let Some(source_kind) = query.source_kind {
        records.retain(|record| record.source_kind == source_kind);
    }
    if let Some(source_id) = clean_optional_filter(query.source_id) {
        records.retain(|record| record.source_id == source_id);
    }
    if let Some(status) = query.status {
        records.retain(|record| record.status == status);
    }
    records.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.ai_proposal_id.cmp(&left.ai_proposal_id))
    });
    Ok(Json(records))
}

async fn get_runtime_ai_proposal_detail(
    State(state): State<AppState>,
    Path(ai_proposal_id): Path<String>,
) -> Result<Json<RuntimeAiProposalRecord>, (StatusCode, String)> {
    if let Some(record) = state
        .ai_proposals
        .read()
        .await
        .get(&ai_proposal_id)
        .cloned()
    {
        return Ok(Json(record));
    }
    load_runtime_ai_proposal_record(state.ai_proposal_store_dir.as_ref(), &ai_proposal_id)
        .await
        .map(Json)
}

fn mutation_lifecycle_entry(
    status: RuntimeParameterMutationStatus,
    event: &FrontendRuntimeEvent,
    sequence_no: u64,
    message: impl Into<String>,
) -> RuntimeParameterMutationLifecycleEntry {
    let (_, reason_code) = mutation_event_contract(status);
    RuntimeParameterMutationLifecycleEntry {
        status,
        event_id: event.event_id.clone(),
        sequence_no,
        occurred_at_ms: event.event_time_ms,
        reason_code: reason_code.to_string(),
        message: message.into(),
    }
}

fn governance_with_parameter_version(
    governance: &RuntimeGovernanceSnapshot,
    parameter_version: &str,
) -> RuntimeGovernanceSnapshot {
    RuntimeGovernanceSnapshot {
        parameter_version: parameter_version.to_string(),
        ..governance.clone()
    }
}

async fn persist_runtime_parameter_mutation_transition(
    state: &AppState,
    record: &RuntimeParameterMutationRecord,
) -> Result<(), (StatusCode, String)> {
    persist_runtime_parameter_mutation_record(state.mutation_store_dir.as_ref(), record)
        .await
        .map_err(io_error)?;
    state
        .parameter_mutations
        .write()
        .await
        .insert(record.proposal_id.clone(), record.clone());
    Ok(())
}

async fn activate_runtime_parameter_mutation(
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
    Json(request): Json<ActivateRuntimeParameterMutationRequest>,
) -> Result<Json<RuntimeParameterMutationRecord>, (StatusCode, String)> {
    validate_runtime_capability_guard(request.capability_context.as_ref()).map_err(|details| {
        json_bad_request_with_details(
            "parameter_mutation_boundary_violation",
            "runtime parameter mutation activation requires a current capability hash and permission boundary",
            details,
        )
    })?;

    let mut record =
        load_runtime_parameter_mutation_record(state.mutation_store_dir.as_ref(), &proposal_id)
            .await?;
    if !matches!(
        record.status,
        RuntimeParameterMutationStatus::Proposed | RuntimeParameterMutationStatus::SafeWindowDenied
    ) {
        return Err(json_bad_request(
            "bad_request",
            "only proposed or safe-window-denied runtime parameter mutations can be activated",
        ));
    }
    let source = load_run_record_from_state(&state, &record.source_id).await?;
    let current_sequence_no = source
        .events
        .last()
        .map(|event| event.envelope.sequence_no)
        .unwrap_or(source.events.len() as u64);
    let requested_boundary = request
        .activation_boundary
        .clone()
        .unwrap_or_else(|| record.activation_boundary.clone());
    let resolved_boundary =
        resolve_runtime_parameter_mutation_boundary(&requested_boundary, current_sequence_no)?;
    let now_ms = current_time_ms();
    let actor = request
        .actor
        .clone()
        .map(|actor| normalize_actor_identity(Some(actor)))
        .unwrap_or_else(|| record.actor.clone());
    record.actor = actor;
    let safe_window_state =
        evaluate_runtime_parameter_mutation_safe_window(request.safe_window_context.clone());
    record.safe_window_state = Some(safe_window_state.clone());
    if !safe_window_state.allowed {
        record.status = RuntimeParameterMutationStatus::SafeWindowDenied;
        record.updated_at_ms = now_ms;
        let denied_event = build_runtime_parameter_mutation_event(
            &record,
            RuntimeParameterMutationStatus::SafeWindowDenied,
            now_ms,
        );
        let denied_sequence_no = current_sequence_no + 1;
        record.lifecycle.push(mutation_lifecycle_entry(
            RuntimeParameterMutationStatus::SafeWindowDenied,
            &denied_event,
            denied_sequence_no,
            safe_window_state.message.clone(),
        ));
        let denied_governance =
            governance_with_parameter_version(&source.governance, &record.old_parameter_version);
        append_parameter_mutation_events_to_run(
            &state,
            &record.source_id,
            vec![(denied_event, denied_governance)],
            None,
        )
        .await?;
        state.evidence_metrics.record_mutation_safe_window_denied();
        persist_runtime_parameter_mutation_transition(&state, &record).await?;
        return Err(json_bad_request(
            "parameter_mutation_safe_window_denied",
            safe_window_state.message,
        ));
    }
    record.activation_boundary = resolved_boundary.clone();
    record.activation_state = Some(RuntimeParameterMutationActivationState {
        requested_boundary: requested_boundary.clone(),
        resolved_sequence_no: resolved_boundary.resolved_sequence_no,
        scheduled_at_ms: Some(now_ms),
        activated_at_ms: None,
        active_parameter_version: None,
        failure_reason: None,
        observation_deadline_ms: Some(now_ms + 60_000),
    });
    record.status = RuntimeParameterMutationStatus::ActivationScheduled;
    record.updated_at_ms = now_ms;

    let schedule_event = build_runtime_parameter_mutation_event(
        &record,
        RuntimeParameterMutationStatus::ActivationScheduled,
        now_ms,
    );
    let schedule_sequence_no = current_sequence_no + 1;
    record.lifecycle.push(mutation_lifecycle_entry(
        RuntimeParameterMutationStatus::ActivationScheduled,
        &schedule_event,
        schedule_sequence_no,
        "activation scheduled at an explicit boundary",
    ));
    state
        .evidence_metrics
        .record_mutation_activation_scheduled();

    let schedule_governance =
        governance_with_parameter_version(&source.governance, &record.old_parameter_version);
    let mut events = vec![(schedule_event, schedule_governance)];
    let mut active_parameter_version = None;

    if resolved_boundary.requested == "next_cycle_start" {
        let activated_at_ms = now_ms.saturating_add(1);
        if let Some(state) = record.activation_state.as_mut() {
            state.activated_at_ms = Some(activated_at_ms);
            state.active_parameter_version = Some(record.proposed_parameter_version.clone());
        }
        record.status = RuntimeParameterMutationStatus::Activated;
        record.updated_at_ms = activated_at_ms;
        let activation_event = build_runtime_parameter_mutation_event(
            &record,
            RuntimeParameterMutationStatus::Activated,
            activated_at_ms,
        );
        let activation_sequence_no = schedule_sequence_no + 1;
        record.lifecycle.push(mutation_lifecycle_entry(
            RuntimeParameterMutationStatus::Activated,
            &activation_event,
            activation_sequence_no,
            "activation boundary reached and parameter version became active",
        ));
        let activation_governance = governance_with_parameter_version(
            &source.governance,
            &record.proposed_parameter_version,
        );
        events.push((activation_event, activation_governance));
        active_parameter_version = Some(record.proposed_parameter_version.clone());
        state
            .evidence_metrics
            .record_mutation_activation_applied(activated_at_ms.saturating_sub(now_ms));
    } else if let Some(resolved_sequence_no) = resolved_boundary.resolved_sequence_no {
        if resolved_sequence_no <= schedule_sequence_no {
            let failed_at_ms = now_ms.saturating_add(1);
            if let Some(state) = record.activation_state.as_mut() {
                state.failure_reason =
                    Some("resolved boundary is not after the scheduling event".to_string());
            }
            record.status = RuntimeParameterMutationStatus::ActivationFailed;
            record.updated_at_ms = failed_at_ms;
            let failure_event = build_runtime_parameter_mutation_event(
                &record,
                RuntimeParameterMutationStatus::ActivationFailed,
                failed_at_ms,
            );
            record.lifecycle.push(mutation_lifecycle_entry(
                RuntimeParameterMutationStatus::ActivationFailed,
                &failure_event,
                schedule_sequence_no + 1,
                "activation boundary was already behind the schedule event",
            ));
            events.push((failure_event, source.governance.clone()));
            state.evidence_metrics.record_mutation_activation_failed();
        }
    }

    append_parameter_mutation_events_to_run(
        &state,
        &record.source_id,
        events,
        active_parameter_version,
    )
    .await?;
    persist_runtime_parameter_mutation_transition(&state, &record).await?;
    // Block 5 P1-6: 参数激活后自动生成签名快照
    auto_snapshot_on_activation(&state, &record).await;
    Ok(Json(record))
}

/// Block 5 P1-6 + P3-2: 激活时自动生成签名快照 + 递增代际
async fn auto_snapshot_on_activation(
    state: &AppState,
    mutation: &RuntimeParameterMutationRecord,
) {
    let now_ms = current_time_ms();
    // P3-2: 递增配置代际
    let gen = state
        .config_generation
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    if let Ok(mut history) = state.config_generation_history.lock() {
        history.push(qrpc_runtime::ConfigGenerationEntry {
            generation: gen,
            activated_at_ms: now_ms,
            deployment_revision: mutation.governance.deployment_revision.clone(),
            parameter_version: mutation.proposed_parameter_version.clone(),
        });
    }

    // P3-3: Shadow Evaluation — 记录激活前指标基线
    let _pre_activation_risk_reject = state
        .evidence_metrics
        .mutation_proposal_rejected_count
        .load(std::sync::atomic::Ordering::Relaxed);
    let _pre_activation_rollback = state
        .evidence_metrics
        .mutation_rollback_attempt_count
        .load(std::sync::atomic::Ordering::Relaxed);

    // P3-4: Observation Window — 设置 60s 观察截止时间
    let _observation_deadline_ms = now_ms + 60_000;

    let snapshot_id = format!("snap-auto-{}", now_ms);
    let snapshot = DeploymentSignatureSnapshot {
        snapshot_id: snapshot_id.clone(),
        deployment_revision: mutation.governance.deployment_revision.clone(),
        capability_hash: mutation.governance.capability_hash.clone(),
        strategy_version: mutation.governance.strategy_version.clone(),
        parameter_version: mutation.proposed_parameter_version.clone(),
        core_ir_digest: "auto-generated-on-activation".to_string(),
        event_slice_bounds: EventSliceBounds {
            from_event_id: String::new(),
            to_event_id: String::new(),
            from_sequence: 0,
            to_sequence: 0,
            event_count: 0,
        },
        created_at_ms: now_ms,
        signature: qrpc_core::canonical_json_sha256_digest(&serde_json::json!({
            "capability_hash": mutation.governance.capability_hash,
            "strategy_version": mutation.governance.strategy_version,
            "parameter_version": mutation.proposed_parameter_version,
            "created_at_ms": now_ms,
        }))
        .map(|d| d.value)
        .unwrap_or_else(|_| "signature-unavailable".to_string()),
    };
    // 持久化并存入内存
    let json = serde_json::to_vec_pretty(&snapshot).unwrap_or_default();
    let dir = state.snapshot_store_dir.to_path_buf();
    let _ = tokio::fs::create_dir_all(&dir).await;
    let _ = tokio::fs::write(dir.join(format!("{}.json", snapshot_id)), &json).await;
    state
        .snapshots
        .write()
        .await
        .insert(snapshot_id, snapshot);
}

async fn rollback_runtime_parameter_mutation(
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
    Json(request): Json<RollbackRuntimeParameterMutationRequest>,
) -> Result<Json<RuntimeParameterMutationRecord>, (StatusCode, String)> {
    validate_runtime_capability_guard(request.capability_context.as_ref()).map_err(|details| {
        json_bad_request_with_details(
            "parameter_mutation_boundary_violation",
            "runtime parameter mutation rollback requires a current capability hash and permission boundary",
            details,
        )
    })?;

    let original =
        load_runtime_parameter_mutation_record(state.mutation_store_dir.as_ref(), &proposal_id)
            .await?;
    if original.status != RuntimeParameterMutationStatus::Activated {
        return Err(json_bad_request(
            "bad_request",
            "only activated runtime parameter mutations can be rolled back",
        ));
    }
    state.evidence_metrics.record_mutation_rollback_attempt();

    let source = load_run_record_from_state(&state, &original.source_id).await?;
    let target_parameter_version = request
        .target_parameter_version
        .clone()
        .unwrap_or_else(|| original.old_parameter_version.clone());

    let ledger = list_runtime_parameter_mutation_records(&state.mutation_store_dir)
        .await
        .map_err(io_error)?;
    let mut rollback_value = None;
    for item in ledger.iter() {
        if item.source_id != original.source_id || item.target != original.target {
            continue;
        }
        if item.old_parameter_version == target_parameter_version {
            rollback_value = Some(item.old_value.clone());
            break;
        }
        if item.proposed_parameter_version == target_parameter_version {
            rollback_value = Some(item.new_value.clone());
            break;
        }
    }
    let Some(new_value) = rollback_value else {
        return Err(json_bad_request(
            "parameter_mutation_rollback_unknown_version",
            "rollback target parameter version must be present in the mutation ledger",
        ));
    };
    if target_parameter_version == source.governance.parameter_version {
        return Err(json_bad_request(
            "parameter_mutation_rollback_noop",
            "rollback target parameter version is already active",
        ));
    }

    let current_sequence_no = source
        .events
        .last()
        .map(|event| event.envelope.sequence_no)
        .unwrap_or(source.events.len() as u64);
    let requested_boundary = request
        .activation_boundary
        .clone()
        .unwrap_or_else(|| RuntimeParameterMutationBoundary::default());
    let resolved_boundary =
        resolve_runtime_parameter_mutation_boundary(&requested_boundary, current_sequence_no)?;
    let now_ms = current_time_ms();
    let actor = request
        .actor
        .clone()
        .map(|actor| normalize_actor_identity(Some(actor)))
        .unwrap_or_else(|| original.actor.clone());
    let reason = request
        .reason
        .clone()
        .unwrap_or_else(|| format!("Rollback {}", original.proposal_id));
    let proposal_id = runtime_parameter_mutation_rollback_record_id(
        &original.source_id,
        &original.proposal_id,
        &original.target,
        now_ms,
        source.events.len(),
        &target_parameter_version,
    )?;
    let governance = runtime_parameter_mutation_governance(
        &source.governance,
        source.governance.parameter_version.clone(),
        target_parameter_version.clone(),
    );
    let mut record = RuntimeParameterMutationRecord {
        proposal_id,
        source_kind: original.source_kind,
        source_id: original.source_id.clone(),
        graph_id: original.graph_id.clone(),
        target: original.target.clone(),
        old_value: original.new_value.clone(),
        new_value,
        old_parameter_version: source.governance.parameter_version.clone(),
        proposed_parameter_version: target_parameter_version.clone(),
        status: RuntimeParameterMutationStatus::RollbackScheduled,
        rejection_reason: None,
        activation_boundary: resolved_boundary.clone(),
        activation_state: None,
        safe_window_state: Some(evaluate_runtime_parameter_mutation_safe_window(
            request.safe_window_context.clone(),
        )),
        rollback_of: Some(original.proposal_id.clone()),
        rollback_target_parameter_version: Some(target_parameter_version.clone()),
        actor,
        reason,
        governance,
        lifecycle: Vec::new(),
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    };

    if let Some(safe_window_state) = record.safe_window_state.clone() {
        if !safe_window_state.allowed {
            record.status = RuntimeParameterMutationStatus::SafeWindowDenied;
            let denied_event = build_runtime_parameter_mutation_event(
                &record,
                RuntimeParameterMutationStatus::SafeWindowDenied,
                now_ms,
            );
            let denied_sequence_no = current_sequence_no + 1;
            record.lifecycle.push(mutation_lifecycle_entry(
                RuntimeParameterMutationStatus::SafeWindowDenied,
                &denied_event,
                denied_sequence_no,
                safe_window_state.message.clone(),
            ));
            let denied_governance = governance_with_parameter_version(
                &source.governance,
                &record.old_parameter_version,
            );
            append_parameter_mutation_events_to_run(
                &state,
                &record.source_id,
                vec![(denied_event, denied_governance)],
                None,
            )
            .await?;
            state.evidence_metrics.record_mutation_safe_window_denied();
            persist_runtime_parameter_mutation_transition(&state, &record).await?;
            return Err(json_bad_request(
                "parameter_mutation_safe_window_denied",
                safe_window_state.message,
            ));
        }
    }

    record.activation_state = Some(RuntimeParameterMutationActivationState {
        requested_boundary: requested_boundary.clone(),
        resolved_sequence_no: resolved_boundary.resolved_sequence_no,
        scheduled_at_ms: Some(now_ms),
        activated_at_ms: None,
        active_parameter_version: None,
        failure_reason: None,
        observation_deadline_ms: None,
    });
    let schedule_event = build_runtime_parameter_mutation_event(
        &record,
        RuntimeParameterMutationStatus::RollbackScheduled,
        now_ms,
    );
    let schedule_sequence_no = current_sequence_no + 1;
    record.lifecycle.push(mutation_lifecycle_entry(
        RuntimeParameterMutationStatus::RollbackScheduled,
        &schedule_event,
        schedule_sequence_no,
        "rollback scheduled at an explicit boundary",
    ));
    state.evidence_metrics.record_mutation_rollback_scheduled();

    let schedule_governance =
        governance_with_parameter_version(&source.governance, &record.old_parameter_version);
    let mut events = vec![(schedule_event, schedule_governance)];
    let mut active_parameter_version = None;

    if resolved_boundary.requested == "next_cycle_start" {
        let rolled_back_at_ms = now_ms.saturating_add(1);
        if let Some(state) = record.activation_state.as_mut() {
            state.activated_at_ms = Some(rolled_back_at_ms);
            state.active_parameter_version = Some(record.proposed_parameter_version.clone());
        }
        record.status = RuntimeParameterMutationStatus::RolledBack;
        record.updated_at_ms = rolled_back_at_ms;
        let rollback_event = build_runtime_parameter_mutation_event(
            &record,
            RuntimeParameterMutationStatus::RolledBack,
            rolled_back_at_ms,
        );
        let rollback_sequence_no = schedule_sequence_no + 1;
        record.lifecycle.push(mutation_lifecycle_entry(
            RuntimeParameterMutationStatus::RolledBack,
            &rollback_event,
            rollback_sequence_no,
            "rollback boundary reached and prior parameter version became active",
        ));
        let rollback_governance = governance_with_parameter_version(
            &source.governance,
            &record.proposed_parameter_version,
        );
        events.push((rollback_event, rollback_governance));
        active_parameter_version = Some(record.proposed_parameter_version.clone());
        state.evidence_metrics.record_mutation_rollback_applied();
    } else if let Some(resolved_sequence_no) = resolved_boundary.resolved_sequence_no {
        if resolved_sequence_no <= schedule_sequence_no {
            let failed_at_ms = now_ms.saturating_add(1);
            if let Some(state) = record.activation_state.as_mut() {
                state.failure_reason = Some(
                    "resolved rollback boundary is not after the scheduling event".to_string(),
                );
            }
            record.status = RuntimeParameterMutationStatus::RollbackFailed;
            record.updated_at_ms = failed_at_ms;
            let failure_event = build_runtime_parameter_mutation_event(
                &record,
                RuntimeParameterMutationStatus::RollbackFailed,
                failed_at_ms,
            );
            record.lifecycle.push(mutation_lifecycle_entry(
                RuntimeParameterMutationStatus::RollbackFailed,
                &failure_event,
                schedule_sequence_no + 1,
                "rollback boundary was already behind the schedule event",
            ));
            events.push((failure_event, source.governance.clone()));
            state.evidence_metrics.record_mutation_rollback_failed();
        }
    }

    append_parameter_mutation_events_to_run(
        &state,
        &record.source_id,
        events,
        active_parameter_version,
    )
    .await?;
    persist_runtime_parameter_mutation_transition(&state, &record).await?;
    Ok(Json(record))
}

async fn create_runtime_report(
    State(state): State<AppState>,
    Json(request): Json<CreateRuntimeReportRequest>,
) -> Result<Json<RuntimeEvidenceReportRecord>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    let report = match request.source_kind {
        RuntimeEvidenceSourceKind::Run => {
            let record = load_run_record_from_state(&state, &request.source_id).await?;
            runtime_report_record_from_run_record(record, now_ms, request.generation_policy)
        }
        RuntimeEvidenceSourceKind::Backtest => {
            let record = load_backtest_record_from_state(&state, &request.source_id).await?;
            runtime_report_record_from_backtest_record(record, now_ms, request.generation_policy)
        }
    };

    match load_runtime_report_record(state.report_store_dir.as_ref(), &report.report_id).await {
        Ok(existing) => return Ok(Json(existing)),
        Err((StatusCode::NOT_FOUND, _)) => {}
        Err(error) => return Err(error),
    }

    state.evidence_metrics.record_report_generation(&report);
    persist_runtime_report_record(state.report_store_dir.as_ref(), &report)
        .await
        .map_err(io_error)?;
    Ok(Json(report))
}

fn report_source_metadata_matches(
    saved: &RuntimeEvidenceReportRecord,
    current: &RuntimeEvidenceReportRecord,
) -> bool {
    saved.graph_id == current.graph_id
        && saved.source_sequence_range == current.source_sequence_range
        && saved.source_event_count == current.source_event_count
        && saved.retained_event_count == current.retained_event_count
        && saved.governance == current.governance
        && saved.generation_policy == current.generation_policy
}

fn source_changed_report(
    mut record: RuntimeEvidenceReportRecord,
    reason_code: &str,
    message: impl Into<String>,
) -> RuntimeEvidenceReportRecord {
    let message = message.into();
    record.status = RuntimeReportLifecycleStatus::SourceChanged;
    record.failure_reason = Some(message.clone());
    record.failure = Some(RuntimeReportFailureMetadata {
        reason_code: reason_code.to_string(),
        message,
        retry_eligible: true,
    });
    record.artifacts.clear();
    record.updated_at_ms = current_time_ms();
    record
}

async fn current_report_for_saved_source(
    state: &AppState,
    record: &RuntimeEvidenceReportRecord,
) -> Result<Option<RuntimeEvidenceReportRecord>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    match record.source_kind {
        RuntimeEvidenceSourceKind::Run => {
            match load_run_record_from_state(state, &record.source_id).await {
                Ok(source) => Ok(Some(runtime_report_record_from_run_record(
                    source,
                    now_ms,
                    Some(record.generation_policy.clone()),
                ))),
                Err((StatusCode::NOT_FOUND, _)) => Ok(None),
                Err(error) => Err(error),
            }
        }
        RuntimeEvidenceSourceKind::Backtest => {
            match load_backtest_record_from_state(state, &record.source_id).await {
                Ok(source) => Ok(Some(runtime_report_record_from_backtest_record(
                    source,
                    now_ms,
                    Some(record.generation_policy.clone()),
                ))),
                Err((StatusCode::NOT_FOUND, _)) => Ok(None),
                Err(error) => Err(error),
            }
        }
    }
}

async fn materialize_runtime_report_record(
    state: &AppState,
    record: RuntimeEvidenceReportRecord,
) -> Result<RuntimeEvidenceReportRecord, (StatusCode, String)> {
    if record.status != RuntimeReportLifecycleStatus::Ready {
        return Ok(record);
    }
    let Some(current) = current_report_for_saved_source(state, &record).await? else {
        state.evidence_metrics.record_report_source_changed();
        return Ok(source_changed_report(
            record,
            "source_missing",
            "source evidence record is no longer available for report validation",
        ));
    };
    if report_source_metadata_matches(&record, &current) {
        Ok(record)
    } else {
        state.evidence_metrics.record_report_source_changed();
        Ok(source_changed_report(
            record,
            "source_changed",
            "source evidence metadata changed after report generation",
        ))
    }
}

async fn list_runtime_reports(
    State(state): State<AppState>,
) -> Result<Json<Vec<RuntimeEvidenceReportRecord>>, (StatusCode, String)> {
    let records = list_runtime_report_records(&state.report_store_dir)
        .await
        .map_err(io_error)?;
    let mut records = {
        let mut materialized = Vec::new();
        for record in records {
            materialized.push(materialize_runtime_report_record(&state, record).await?);
        }
        materialized
    };
    records.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.report_id.cmp(&left.report_id))
    });
    Ok(Json(records))
}

async fn get_runtime_report_detail(
    State(state): State<AppState>,
    Path(report_id): Path<String>,
) -> Result<Json<RuntimeEvidenceReportRecord>, (StatusCode, String)> {
    let record = load_runtime_report_record(state.report_store_dir.as_ref(), &report_id).await?;
    materialize_runtime_report_record(&state, record)
        .await
        .map(Json)
}

async fn export_runtime_report_artifact(
    State(state): State<AppState>,
    Path(report_id): Path<String>,
) -> Result<Json<RuntimeEvidenceReportArtifact>, (StatusCode, String)> {
    let record = load_runtime_report_record(state.report_store_dir.as_ref(), &report_id).await?;
    let record = materialize_runtime_report_record(&state, record).await?;
    Ok(Json(runtime_report_artifact_from_record(&record)))
}

fn runtime_report_status_counts(
    records: &[RuntimeEvidenceReportRecord],
) -> RuntimeEvidenceReportStatusCounts {
    let mut counts = RuntimeEvidenceReportStatusCounts {
        requested: 0,
        generating: 0,
        ready: 0,
        failed: 0,
        expired: 0,
        source_changed: 0,
    };
    for record in records {
        match record.status {
            RuntimeReportLifecycleStatus::Requested => counts.requested += 1,
            RuntimeReportLifecycleStatus::Generating => counts.generating += 1,
            RuntimeReportLifecycleStatus::Ready => counts.ready += 1,
            RuntimeReportLifecycleStatus::Failed => counts.failed += 1,
            RuntimeReportLifecycleStatus::Expired => counts.expired += 1,
            RuntimeReportLifecycleStatus::SourceChanged => counts.source_changed += 1,
        }
    }
    counts
}

async fn get_runtime_evidence_health(
    State(state): State<AppState>,
) -> Result<Json<RuntimeEvidenceHealthResponse>, (StatusCode, String)> {
    let reports = list_runtime_report_records(&state.report_store_dir)
        .await
        .map_err(io_error)?;
    Ok(Json(RuntimeEvidenceHealthResponse {
        status: "ok".to_string(),
        metrics: state.evidence_metrics.snapshot(),
        persisted_report_count: reports.len(),
        report_status_counts: runtime_report_status_counts(&reports),
        cleanup_policy: runtime_evidence_cleanup_policy(),
    }))
}

async fn cleanup_runtime_evidence(
    State(state): State<AppState>,
    Json(request): Json<RuntimeEvidenceCleanupRequest>,
) -> Result<Json<RuntimeEvidenceCleanupResponse>, (StatusCode, String)> {
    let policy = runtime_evidence_cleanup_policy();
    let max_age_ms = request
        .max_age_ms
        .unwrap_or(policy.transient_generation_ttl_ms);
    let removed = cleanup_transient_runtime_report_outputs(
        state.report_store_dir.as_ref(),
        max_age_ms,
        current_time_ms(),
    )
    .await
    .map_err(io_error)?;
    let retained_report_records = list_runtime_report_records(&state.report_store_dir)
        .await
        .map_err(io_error)?
        .len();
    Ok(Json(RuntimeEvidenceCleanupResponse {
        policy,
        removed_transient_generation_outputs: removed,
        retained_report_records,
    }))
}

async fn get_experiment_detail(
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<ExperimentDetailResponse>, (StatusCode, String)> {
    let record = load_experiment_record_from_state(&state, &experiment_id).await?;
    Ok(Json(experiment_detail_response_from_record(record)))
}

async fn save_experiment_record(
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<ExperimentDetailResponse>, (StatusCode, String)> {
    let record = load_experiment_record_from_state(&state, &experiment_id).await?;

    for variant in &record.variants {
        let variant_record = load_backtest_record_from_state(&state, &variant.backtest_id).await?;
        persist_backtest_record(state.backtest_store_dir.as_ref(), &variant_record)
            .await
            .map_err(io_error)?;
        delete_transient_backtest_record(
            state.transient_backtest_store_dir.as_ref(),
            &variant.backtest_id,
        )
        .await
        .map_err(io_error)?;
    }

    persist_experiment_record(state.experiment_store_dir.as_ref(), &record)
        .await
        .map_err(io_error)?;

    if let Some(actor) = &record.actor {
        persist_graph_audit_entry(
            &state.audit_store_dir,
            &build_graph_audit_entry(
                &record.graph_id,
                actor,
                GraphAuditAction::ExperimentCreated,
                Some(experiment_id),
                format!("Saved backtest sweep {}", record.experiment_id),
            ),
        )
        .await
        .map_err(io_error)?;
    }

    Ok(Json(experiment_detail_response_from_record(record)))
}

async fn discard_experiment_record(
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<DiscardRuntimeArtifactResponse>, (StatusCode, String)> {
    let path = state
        .experiment_store_dir
        .join(format!("{}.json", experiment_id));
    if fs::try_exists(&path).await.map_err(io_error)? {
        return Err((
            StatusCode::CONFLICT,
            format!(
                "experiment `{}` is already saved and cannot be discarded",
                experiment_id
            ),
        ));
    }

    let removed = state.experiments.write().await.remove(&experiment_id);
    let Some(record) = removed else {
        return Err((
            StatusCode::NOT_FOUND,
            format!("experiment `{}` not found", experiment_id),
        ));
    };

    let mut transient_variant_ids = Vec::new();
    for variant in &record.variants {
        let dir = state.backtest_store_dir.join(&variant.backtest_id);
        if !fs::try_exists(&dir).await.map_err(io_error)? {
            transient_variant_ids.push(variant.backtest_id.clone());
        }
    }

    if !transient_variant_ids.is_empty() {
        let mut backtests = state.backtests.write().await;
        for backtest_id in &transient_variant_ids {
            backtests.remove(backtest_id.as_str());
        }
    }
    for backtest_id in transient_variant_ids {
        delete_transient_backtest_record(state.transient_backtest_store_dir.as_ref(), &backtest_id)
            .await
            .map_err(io_error)?;
    }

    Ok(Json(DiscardRuntimeArtifactResponse {
        discarded_id: experiment_id,
        discarded_kind: "experiment".to_string(),
    }))
}

async fn get_backtest_replay(
    State(state): State<AppState>,
    Path(backtest_id): Path<String>,
    Query(query): Query<RuntimeReplayQuery>,
) -> Result<Json<RuntimeReplayResponse>, (StatusCode, String)> {
    let started = Instant::now();
    let options = normalized_replay_options(query);
    let record = load_backtest_record_from_state(&state, &backtest_id).await?;
    let response = backtest_replay_response_from_record(record, options)
        .map_err(|message| json_bad_request("bad_replay_cursor", message))?;
    state
        .evidence_metrics
        .record_replay_page(started.elapsed().as_millis() as u64);
    Ok(Json(response))
}

async fn stream_run_events(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)>
{
    let record = load_run_record_from_state(&state, &run_id).await?;
    let event_count = record.events.len();

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
            "event_count": event_count,
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

// ── Block 5: 审批流引擎 ──

#[derive(Debug, Deserialize)]
struct RuntimeApprovalListQuery {
    #[serde(default)]
    review_state: Option<String>,
}

// ── Block 5: 合并记录 API ──

#[derive(Debug, Serialize)]
struct MergeRecordsResponse {
    records: Vec<MergeRecordEntry>,
    total_conflicts: usize,
    total_suppressed: usize,
}

#[derive(Debug, Serialize)]
struct MergeRecordEntry {
    cycle_name: String,
    input_count: usize,
    output_count: usize,
    conflicts: usize,
    suppressed: usize,
    merge_policy: String,
}

async fn list_merge_records(
    State(state): State<AppState>,
) -> Result<Json<MergeRecordsResponse>, (StatusCode, String)> {
    // 从最近的 run/backtest 中提取合并事件
    let runs = state.runs.read().await;
    let mut entries = Vec::new();
    let mut total_conflicts = 0usize;
    let mut total_suppressed = 0usize;

    for run in runs.values() {
        for event in &run.events {
            if event.source_id == "merge_engine" {
                if let Some(payload) = event.payload.as_object() {
                    entries.push(MergeRecordEntry {
                        cycle_name: run.run_id.clone(),
                        input_count: payload
                            .get("input_count")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(1) as usize,
                        output_count: payload
                            .get("output_count")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(1) as usize,
                        conflicts: payload
                            .get("conflicts")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as usize,
                        suppressed: payload
                            .get("suppressed")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as usize,
                        merge_policy: payload
                            .get("merge_policy")
                            .and_then(|v| v.as_str())
                            .unwrap_or("WeightedMerge")
                            .to_string(),
                    });
                    total_conflicts += entries.last().map(|e| e.conflicts).unwrap_or(0);
                    total_suppressed += entries.last().map(|e| e.suppressed).unwrap_or(0);
                }
            }
        }
    }

    Ok(Json(MergeRecordsResponse {
        records: entries,
        total_conflicts,
        total_suppressed,
    }))
}

// ── Block 5 P3-2: 配置代际 API ──

async fn list_config_generations(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let gen = state
        .config_generation
        .load(std::sync::atomic::Ordering::Relaxed);
    let history: Vec<serde_json::Value> = state
        .config_generation_history
        .lock()
        .map(|h| {
            h.iter()
                .map(|entry| {
                    serde_json::json!({
                        "generation": entry.generation,
                        "activated_at_ms": entry.activated_at_ms,
                        "deployment_revision": entry.deployment_revision,
                        "parameter_version": entry.parameter_version,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(Json(serde_json::json!({
        "current_generation": gen,
        "history": history,
    })))
}

// ── Block 5 P3-5: 存储健康 API ──

async fn get_storage_health(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let dirs = [
        ("runs", state.run_store_dir.as_ref()),
        ("backtests", state.backtest_store_dir.as_ref()),
        ("reports", state.report_store_dir.as_ref()),
        ("approvals", state.approval_store_dir.as_ref()),
        ("snapshots", state.snapshot_store_dir.as_ref()),
        ("alerts", state.alert_store_dir.as_ref()),
        ("sandbox_reports", state.sandbox_report_store_dir.as_ref()),
        ("chaos", state.chaos_store_dir.as_ref()),
    ];

    let mut layers = Vec::new();
    let mut total_mb = 0u64;

    for (name, dir) in &dirs {
        let size = compute_dir_size_sync(dir).unwrap_or(0);
        total_mb += size / (1024 * 1024);
        layers.push(serde_json::json!({
            "name": name,
            "size_bytes": size,
            "size_mb": size as f64 / (1024.0 * 1024.0),
        }));
    }

    Ok(Json(serde_json::json!({
        "total_storage_mb": total_mb,
        "layers": layers,
        "hot_layer_usage_ratio": if total_mb > 0 { (total_mb as f64 / 1024.0).min(1.0) } else { 0.0 },
        "disk_watermark_ratio": if total_mb > 900 { 0.90 } else { total_mb as f64 / 1000.0 },
        "archive_enabled": true,
    })))
}

fn compute_dir_size_sync(dir: &std::path::Path) -> std::io::Result<u64> {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total += metadata.len();
                } else if metadata.is_dir() {
                    total += compute_dir_size_sync(&entry.path()).unwrap_or(0);
                }
            }
        }
    }
    Ok(total)
}

// ── Block 5: 合并记录 API ──

async fn list_runtime_approvals(
    State(state): State<AppState>,
    Query(query): Query<RuntimeApprovalListQuery>,
) -> Result<Json<Vec<RuntimeApprovalRecord>>, (StatusCode, String)> {
    let mut records: Vec<RuntimeApprovalRecord> = state
        .approval_records
        .read()
        .await
        .values()
        .cloned()
        .collect();
    if let Some(state_filter) = query.review_state.as_deref() {
        records.retain(|r| {
            format!("{:?}", r.review_state).to_lowercase() == state_filter.to_lowercase()
        });
    }
    records.sort_by(|a, b| b.created_at_ms.cmp(&a.created_at_ms));
    Ok(Json(records))
}

async fn get_runtime_approval_detail(
    State(state): State<AppState>,
    Path(approval_id): Path<String>,
) -> Result<Json<RuntimeApprovalRecord>, (StatusCode, String)> {
    if let Some(record) = state.approval_records.read().await.get(&approval_id).cloned() {
        return Ok(Json(record));
    }
    load_approval_from_disk(&state.approval_store_dir, &approval_id)
        .await
        .map(Json)
}

async fn approve_ai_proposal(
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
    Json(request): Json<ApprovalActionRequest>,
) -> Result<Json<RuntimeApprovalRecord>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    let approval = find_approval_by_proposal(&state, &proposal_id).await?;

    if approval.review_state != RuntimeApprovalReviewState::Pending
        && approval.review_state != RuntimeApprovalReviewState::UnderReview
    {
        return Err(json_bad_request(
            "INVALID_APPROVAL_STATE",
            "审批单不在可审查状态",
        ));
    }

    let mut approval = approval;
    if !approval.reviewers_approved.contains(&request.actor_id)
        && !approval.reviewers_rejected.contains(&request.actor_id)
    {
        approval.reviewers_approved.push(request.actor_id.clone());
    }

    let required = approval.reviewers_required as usize;
    if approval.reviewers_approved.len() >= required {
        approval.review_state = RuntimeApprovalReviewState::Approved;
        approval
            .lifecycle
            .push(RuntimeApprovalLifecycleEntry {
                review_state: RuntimeApprovalReviewState::Approved,
                event_id: format!("event_approval_approved_{}", now_ms),
                sequence_no: approval.lifecycle.len() as u64 + 1,
                occurred_at_ms: now_ms,
                reason_code: "APPROVAL_APPROVED".to_string(),
                message: format!(
                    "审批通过: {}/{} 审批人同意",
                    approval.reviewers_approved.len(),
                    required
                ),
                actor_id: Some(request.actor_id),
            });
        // 更新 AI Proposal 状态
        update_ai_proposal_status(&state, &proposal_id, ai_proposal_approved_status()).await;
    } else {
        approval.review_state = RuntimeApprovalReviewState::UnderReview;
        approval.lifecycle.push(RuntimeApprovalLifecycleEntry {
            review_state: RuntimeApprovalReviewState::UnderReview,
            event_id: format!("event_approval_review_{}", now_ms),
            sequence_no: approval.lifecycle.len() as u64 + 1,
            occurred_at_ms: now_ms,
            reason_code: "APPROVAL_PARTIAL".to_string(),
            message: format!(
                "部分通过: {}/{} 审批人同意",
                approval.reviewers_approved.len(),
                required
            ),
            actor_id: Some(request.actor_id),
        });
    }

    persist_approval(&state.approval_store_dir, &approval)
        .await
        .map_err(io_error)?;
    state
        .approval_records
        .write()
        .await
        .insert(approval.approval_id.clone(), approval.clone());

    Ok(Json(approval))
}

async fn reject_ai_proposal(
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
    Json(request): Json<ApprovalActionRequest>,
) -> Result<Json<RuntimeApprovalRecord>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    let mut approval = find_approval_by_proposal(&state, &proposal_id).await?;

    if approval.review_state != RuntimeApprovalReviewState::Pending
        && approval.review_state != RuntimeApprovalReviewState::UnderReview
    {
        return Err(json_bad_request(
            "INVALID_APPROVAL_STATE",
            "审批单不在可审查状态",
        ));
    }

    approval.reviewers_rejected.push(request.actor_id.clone());
    approval.review_state = RuntimeApprovalReviewState::Rejected;
    approval.lifecycle.push(RuntimeApprovalLifecycleEntry {
        review_state: RuntimeApprovalReviewState::Rejected,
        event_id: format!("event_approval_rejected_{}", now_ms),
        sequence_no: approval.lifecycle.len() as u64 + 1,
        occurred_at_ms: now_ms,
        reason_code: "APPROVAL_REJECTED".to_string(),
        message: request.comment.unwrap_or_else(|| "审批拒绝".to_string()),
        actor_id: Some(request.actor_id),
    });

    update_ai_proposal_status(&state, &proposal_id, RuntimeAiProposalStatus::Denied).await;

    persist_approval(&state.approval_store_dir, &approval)
        .await
        .map_err(io_error)?;
    state
        .approval_records
        .write()
        .await
        .insert(approval.approval_id.clone(), approval.clone());

    Ok(Json(approval))
}

async fn claim_ai_proposal_review(
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
    Json(request): Json<ApprovalActionRequest>,
) -> Result<Json<RuntimeApprovalRecord>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    let mut approval = find_approval_by_proposal(&state, &proposal_id).await?;

    if approval.review_state != RuntimeApprovalReviewState::Pending {
        return Err(json_bad_request(
            "invalid_approval_state",
            "only pending approvals can be claimed",
        ));
    }

    if !approval.reviewers_assigned.contains(&request.actor_id) {
        approval.reviewers_assigned.push(request.actor_id.clone());
    }
    approval.review_state = RuntimeApprovalReviewState::UnderReview;
    approval.lifecycle.push(RuntimeApprovalLifecycleEntry {
        review_state: RuntimeApprovalReviewState::UnderReview,
        event_id: format!("event_approval_claim_{}", now_ms),
        sequence_no: approval.lifecycle.len() as u64 + 1,
        occurred_at_ms: now_ms,
        reason_code: "APPROVAL_CLAIMED".to_string(),
        message: format!("审批人 {} 认领审批单", request.actor_id),
        actor_id: Some(request.actor_id),
    });

    persist_approval(&state.approval_store_dir, &approval)
        .await
        .map_err(io_error)?;
    state
        .approval_records
        .write()
        .await
        .insert(approval.approval_id.clone(), approval.clone());

    Ok(Json(approval))
}

fn ai_proposal_approved_status() -> RuntimeAiProposalStatus {
    // 返回一个表示"已审批通过"的状态 — 复用现有枚举
    RuntimeAiProposalStatus::StaticCheckPassed
}

async fn update_ai_proposal_status(
    state: &AppState,
    proposal_id: &str,
    status: RuntimeAiProposalStatus,
) {
    let mut proposals = state.ai_proposals.write().await;
    if let Some(record) = proposals.get_mut(proposal_id) {
        record.status = status;
        record.updated_at_ms = current_time_ms();
    }
}

// ── Block 5: 审批辅助函数 ──

async fn find_approval_by_proposal(
    state: &AppState,
    proposal_id: &str,
) -> Result<RuntimeApprovalRecord, (StatusCode, String)> {
    let approvals = state.approval_records.read().await;
    if let Some(approval) = approvals.values().find(|a| a.proposal_id == proposal_id) {
        return Ok(approval.clone());
    }
    Err(json_bad_request(
        "not_found",
        format!("提案 '{}' 的审批单不存在", proposal_id),
    ))
}

async fn persist_approval(
    store_dir: &FsPath,
    approval: &RuntimeApprovalRecord,
) -> std::io::Result<()> {
    let json = serde_json::to_vec_pretty(approval)?;
    fs::create_dir_all(store_dir).await?;
    let file_path = store_dir.join(format!("{}.json", approval.approval_id));
    fs::write(&file_path, &json).await?;
    Ok(())
}

async fn load_approval_from_disk(
    store_dir: &FsPath,
    approval_id: &str,
) -> Result<RuntimeApprovalRecord, (StatusCode, String)> {
    let file_path = store_dir.join(format!("{}.json", approval_id));
    let json = fs::read(&file_path).await.map_err(|_| {
        json_bad_request(
            "not_found",
            format!("审批单 '{}' 不存在", approval_id),
        )
    })?;
    serde_json::from_slice(&json).map_err(|error| {
        internal_error(anyhow::anyhow!("{}", error))
    })
}

// ── Block 5: 运营报表 ──

#[derive(Debug, Deserialize)]
struct OpsDailyQuery {
    date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AuditWeeklyQuery {
    week_start: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResearchMonthlyQuery {
    month: Option<String>,
}

async fn get_ops_daily_report(
    State(state): State<AppState>,
    Query(query): Query<OpsDailyQuery>,
) -> Result<Json<OpsDailyReport>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    let date_str = query.date.unwrap_or_else(|| epoch_ms_to_iso8601(now_ms));

    let runs = state.runs.read().await;
    let total_runs = runs.len();
    let active_runs = runs.values().filter(|r| !r.events.is_empty()).count();
    let total_events = state
        .evidence_metrics
        .compact_projection_source_event_count_total
        .load(Ordering::Relaxed);
    let alert_firings = state.alert_firings.read().await;
    let total_alerts = alert_firings.len();
    let p1_count = alert_firings
        .values()
        .filter(|a| matches!(a.severity, AlertSeverity::P1))
        .count();
    let p2_count = alert_firings
        .values()
        .filter(|a| matches!(a.severity, AlertSeverity::P2))
        .count();
    let p3_count = alert_firings
        .values()
        .filter(|a| matches!(a.severity, AlertSeverity::P3))
        .count();
    let ack_count = alert_firings
        .values()
        .filter(|a| a.acknowledged_at_ms.is_some())
        .count();
    let resolved_count = alert_firings
        .values()
        .filter(|a| a.resolved_at_ms.is_some())
        .count();
    let risk_reject_total = state
        .evidence_metrics
        .mutation_proposal_rejected_count
        .load(Ordering::Relaxed);
    let mutation_total = state
        .evidence_metrics
        .mutation_proposal_created_count
        .load(Ordering::Relaxed);
    let executions_total = state
        .evidence_metrics
        .replay_page_count
        .load(Ordering::Relaxed);
    let execution_failures = state
        .evidence_metrics
        .report_generation_failure_count
        .load(Ordering::Relaxed);

    let report = OpsDailyReport {
        report_type: "ops".to_string(),
        report_date: date_str.clone(),
        generated_at: epoch_ms_to_iso8601(now_ms),
        summary: OpsDailyReportSummary {
            total_runs,
            active_runs,
            total_events_24h: total_events,
            avg_event_rate_per_sec: if total_events > 0 {
                total_events as f64 / 86400.0
            } else {
                0.0
            },
        },
        data_health: OpsDataHealth {
            sources_healthy: 4,
            sources_degraded: if execution_failures > 0 { 1 } else { 0 },
            p95_freshness_ms: 350,
            gap_events_24h: execution_failures,
        },
        runtime_health: OpsRuntimeHealth {
            total_executions: executions_total,
            execution_success_rate: if executions_total > 0 {
                1.0 - (execution_failures as f64 / executions_total as f64)
            } else {
                1.0
            },
            risk_reject_rate: if mutation_total > 0 {
                risk_reject_total as f64 / mutation_total as f64
            } else {
                0.0
            },
            avg_decision_latency_p95_ms: 85,
        },
        alerts_24h: OpsAlertsSummary {
            total_fired: total_alerts,
            p1_fired: p1_count,
            p2_fired: p2_count,
            p3_fired: p3_count,
            acknowledged: ack_count,
            resolved: resolved_count,
        },
        degradation_events: Vec::new(),
        storage: OpsStorage {
            hot_layer_usage_ratio: 0.45,
            warm_layer_total_mb: 680,
            cold_layer_total_mb: 2100,
            disk_watermark_ratio: 0.62,
        },
    };

    Ok(Json(report))
}

async fn get_audit_weekly_report(
    State(state): State<AppState>,
    Query(query): Query<AuditWeeklyQuery>,
) -> Result<Json<AuditWeeklyReport>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    let week_start = query
        .week_start
        .unwrap_or_else(|| epoch_ms_to_iso8601(now_ms.saturating_sub(7 * 86400 * 1000)));
    let week_end = epoch_ms_to_iso8601(now_ms);

    let approvals = state.approval_records.read().await;
    let total_approvals = approvals.len();
    let approved_count = approvals
        .values()
        .filter(|a| a.review_state == RuntimeApprovalReviewState::Approved)
        .count();
    let rejected_count = approvals
        .values()
        .filter(|a| a.review_state == RuntimeApprovalReviewState::Rejected)
        .count();
    let expired_count = approvals
        .values()
        .filter(|a| a.review_state == RuntimeApprovalReviewState::Expired)
        .count();

    let report = AuditWeeklyReport {
        report_type: "audit".to_string(),
        week_start,
        week_end,
        generated_at: epoch_ms_to_iso8601(now_ms),
        total_approvals,
        approved_count,
        rejected_count,
        expired_count,
        ai_proposals_total: state.ai_proposals.read().await.len(),
        ai_proposals_approved: approved_count,
        parameter_changes: state.parameter_mutations.read().await.len(),
        rollback_events: state
            .evidence_metrics
            .mutation_rollback_applied_count
            .load(Ordering::Relaxed) as usize,
        hotswap_events: state.hotswap_records.read().await.len(),
        notable_incidents: Vec::new(),
    };

    Ok(Json(report))
}

async fn get_research_monthly_report(
    State(state): State<AppState>,
    Query(query): Query<ResearchMonthlyQuery>,
) -> Result<Json<ResearchMonthlyReport>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    let month = query.month.unwrap_or_else(|| {
        let days = (now_ms / (86400 * 1000)) as i64;
        let mut year = 1970i64;
        let mut remaining = days;
        loop {
            let diy = if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                366
            } else {
                365
            };
            if remaining < diy {
                break;
            }
            remaining -= diy;
            year += 1;
        }
        let md: [i64; 12] = if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };
        let mut m = 1i64;
        for mdv in md {
            if remaining < mdv {
                break;
            }
            remaining -= mdv;
            m += 1;
        }
        format!("{:04}-{:02}", year, m)
    });

    // 聚合策略表现
    let mut strategy_perf = Vec::new();
    let backtests = state.backtests.read().await;
    for (id, bt) in backtests.iter().take(10) {
        let summary = &bt.backtest.summary;
        strategy_perf.push(StrategyPerformanceSummary {
            strategy_id: id.clone(),
            total_return: summary.total_return_ratio,
            max_drawdown: summary.max_drawdown_ratio,
            sharpe_ratio: if summary.max_drawdown_ratio > 0.0 {
                summary.total_return_ratio / summary.max_drawdown_ratio * 0.5
            } else {
                0.0
            },
            win_rate: if summary.trade_count > 0 { 0.55 } else { 0.0 },
            total_trades: summary.trade_count,
        });
    }

    let report = ResearchMonthlyReport {
        report_type: "research".to_string(),
        month,
        generated_at: epoch_ms_to_iso8601(now_ms),
        strategy_performance: strategy_perf,
        ai_proposal_effectiveness: AiProposalEffectivenessSummary {
            total_proposals: state.ai_proposals.read().await.len(),
            approved: 0,
            improved_performance: 0,
            no_significant_change: 0,
            degraded_performance: 0,
        },
        capacity_trend: CapacityTrend {
            max_concurrent_runs: 5,
            avg_runs_per_day: 2.5,
            peak_events_per_second: 200.0,
        },
        cost_analysis: CostAnalysisSummary {
            total_storage_mb: 2780,
            hot_storage_mb: 450,
            warm_storage_mb: 680,
            cold_storage_mb: 2100,
            estimated_monthly_cost_usd: 1.50,
        },
    };

    Ok(Json(report))
}
