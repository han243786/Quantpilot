use crate::{
    auth, compile_runtime_protocol_via_qs, current_time_ms, experiment_detail_response_from_record,
    io_error, json_bad_request, json_bad_request_with_details, normalize_actor_identity,
    persist_experiment_record, resolved_backtest_execution_assumptions,
    runtime::{backtest_experiment_sweep::build_experiment_overrides, execute_backtest_request},
    validate_backtest_execution_assumption_overrides, validate_runtime_capability_guard,
    validate_runtime_config_capabilities, AppState, ExperimentDefinitionSummary,
    ExperimentDetailResponse, ExperimentRecord, ExperimentVariantSummary, FrontendBacktestOptions,
    FrontendBacktestReplaySource, FrontendExecutionAssumptionOverrides, FrontendExperimentRequest,
    FrontendRunRequest,
};
use axum::{extract::State, http::StatusCode, Json};

pub(crate) async fn start_backtest_experiment(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Json(request): Json<FrontendExperimentRequest>,
) -> Result<Json<ExperimentDetailResponse>, (StatusCode, String)> {
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
            "实验请求必须包含 graph_json，请从图编辑器发起",
        )
    })?;
    let qs_protocol = compile_runtime_protocol_via_qs(graph_json)?;
    let base_execution_assumptions = resolved_backtest_execution_assumptions(
        &qs_protocol,
        request.backtest_options.execution_assumptions.as_ref(),
    );

    let overrides = build_experiment_overrides(&request, &qs_protocol)?;
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
            graph_json: request.graph_json.clone(),
            runtime_targets: request.runtime_targets.clone(),
            backtest_options: FrontendBacktestOptions {
                replay_source: Some(replay_source),
                replay_mode: request.backtest_options.replay_mode.clone(),
                execution_assumptions: Some(override_values.clone()),
                runtime_kind: request.backtest_options.runtime_kind.clone(),
                symbols: request.backtest_options.symbols.clone(),
            },
        };
        let record = execute_backtest_request(
            &state,
            &user_id,
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
        saved: false,
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

    // v2.1.0: 创建时即持久化实验元数据，防止崩溃丢失
    persist_experiment_record(&state.experiment_store_dir, &record)
        .await
        .map_err(io_error)?;
    state
        .experiments
        .write()
        .await
        .insert(auth::scoped_key(&user_id, &experiment_id), record.clone());

    Ok(Json(experiment_detail_response_from_record(record)))
}
