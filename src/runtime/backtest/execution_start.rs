use super::*;

#[path = "legacy_dispatch.rs"]
mod legacy_dispatch;
#[path = "v4_projection.rs"]
mod v4_projection;
#[path = "v4_request_resolution.rs"]
mod v4_request_resolution;
#[path = "v4_runtime_execution.rs"]
mod v4_runtime_execution;

use legacy_dispatch::{
    prepare_legacy_backtest_dispatch, run_legacy_backtest_dispatch, LegacyBacktestDispatchOutput,
};
use v4_projection::{
    build_v4_backtest_output, frontend_events_from_v4_backtest_artifact,
    v4_equity_curve_from_artifact,
};
use v4_request_resolution::{
    is_v4_backtest_request, resolve_v4_backtest_graph, resolve_v4_backtest_market_event_type,
    resolve_v4_backtest_symbols,
};
use v4_runtime_execution::run_v4_backtest_runtime_execution;

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
    let legacy_dispatch_plan = prepare_legacy_backtest_dispatch(graph_json, request)?;
    let protocol_name = legacy_dispatch_plan.compiled.protocol_name.clone();
    let config_hash = legacy_dispatch_plan.compiled.config_hash.clone();
    let now_ms = current_time_ms();
    let actor = normalize_actor_identity(request.actor.clone());
    let _collaboration = collaboration_with_run_actor(
        &state.graph_store_dir,
        &request.runtime_config.metadata.graph_id,
        &actor,
    )
    .await?;
    let LegacyBacktestDispatchOutput {
        compiled,
        artifacts,
        replay_source,
        resolved_execution_assumptions,
        resolved_execution_assumption_sources,
        backtest,
    } = run_legacy_backtest_dispatch(legacy_dispatch_plan, request, now_ms).await?;
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
    let tick_replay = request
        .backtest_options
        .replay_mode
        .as_deref()
        .map(|mode| mode.eq_ignore_ascii_case("tick_replay"))
        .unwrap_or(false);
    let v4_artifact = run_v4_backtest_runtime_execution(
        expanded_graph,
        &symbols,
        &event_type,
        now_ms,
        tick_replay,
    )
    .await?;

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
