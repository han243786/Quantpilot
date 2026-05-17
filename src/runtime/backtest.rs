async fn start_backtest_run(
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

async fn execute_backtest_request(
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
    let graph_json = request.graph_json.as_ref()
        .ok_or_else(|| json_bad_request("bad_request", "回测请求必须包含 graph_json，请从图编辑器发起"))?;
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
    ).await?;
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
                        "{}. 历史重放需要本地市场数据文件 (位于 data cache 目录下)。\
                         离线测试请设置 backtest_options.replay_source = \"deterministic_mock\"",
                        error,
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
        sandbox.set_execution_assumptions(qrpc_runtime::slippage::ExecutionAssumptions {
            latency: qrpc_runtime::slippage::LatencyModel::Fixed { delay_ms: latency_ms },
            ..qrpc_runtime::slippage::ExecutionAssumptions::v1_0_7_compat()
        });
    }
    sandbox.start().map_err(internal_error)?;
    let backtest = sandbox.run_backtest().map_err(internal_error)?;
    let graph_targets = build_compile_runtime_targets_from_graph(graph_json);
    let runtime_targets = merge_runtime_targets(&request.runtime_targets, &graph_targets);
    // v1.3.7: 添加计数器后缀防止同一毫秒内ID碰撞
    static BACKTEST_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let backtest_id = match id_suffix {
        Some(suffix) => format!("backtest_{}_{}", now_ms, suffix),
        None => format!("backtest_{}_{}", now_ms, BACKTEST_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)),
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
    qs_protocol: &RuntimeProtocolCoreConfig,
) -> Result<Vec<FrontendExecutionAssumptionOverrides>, (StatusCode, String)> {
    let provided_values = request.parameter_grid.fee_bps.len()
        + request.parameter_grid.slippage_bps.len()
        + request.parameter_grid.latency_ms.len();
    if provided_values == 0 {
        return Err(json_bad_request(
            "bad_request",
            "参数网格必须至少包含一个执行假设值",
        ));
    }

    let base = resolved_backtest_execution_assumptions(
        qs_protocol,
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
                "参数扫描展开为 {variant_count} 个变体，超出当前限制 {MAX_EXPERIMENT_VARIANTS}"
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

    let graph_json = request.graph_json.as_ref()
        .ok_or_else(|| json_bad_request("bad_request", "实验请求必须包含 graph_json，请从图编辑器发起"))?;
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
                execution_assumptions: Some(override_values.clone()),
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
async fn list_backtests(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<BacktestListItem>>, (StatusCode, String)> {
    let records = list_backtest_records(state.backtest_store_dir.as_ref())
        .await
        .map_err(io_error)?;
    let mut items = records
        .into_iter()
        .map(backtest_list_item_from_record)
        .collect::<Vec<_>>();
    items.sort_by(|left, right| right.created_at_ms.cmp(&left.created_at_ms));
    Ok(Json(paginate(items, pagination)))
}

async fn get_backtest_detail(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(backtest_id): Path<String>,
) -> Result<Json<BacktestDetailResponse>, (StatusCode, String)> {
    let record = load_backtest_record_from_state(&state, &user_id, &backtest_id).await?;
    Ok(Json(backtest_detail_response_from_record(record)))
}

async fn save_backtest_record(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(backtest_id): Path<String>,
) -> Result<Json<BacktestDetailResponse>, (StatusCode, String)> {
    let mut record = load_backtest_record_from_state(&state, &user_id, &backtest_id).await?;
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
        .insert(auth::scoped_key(&user_id, &backtest_id), record.clone());

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
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(backtest_id): Path<String>,
) -> Result<Json<DiscardRuntimeArtifactResponse>, (StatusCode, String)> {
    // v1.1.2: 路径遍历防护
    let safe_id = sanitize_storage_path_segment(&backtest_id);
    let dir = state.backtest_store_dir.join(&safe_id);
    if fs::try_exists(&dir).await.map_err(io_error)? {
        return Err((
            StatusCode::CONFLICT,
            format!(
                "回测 `{}` 已保存, 无法丢弃",
                backtest_id
            ),
        ));
    }

    let scoped_backtest_id = auth::scoped_key(&user_id, &backtest_id);
    let removed_memory = state.backtests.write().await.remove(&scoped_backtest_id).is_some();
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
async fn list_experiments(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<ExperimentListItem>>, (StatusCode, String)> {
    let records = list_experiment_records(state.experiment_store_dir.as_ref())
        .await
        .map_err(io_error)?;
    let mut items = records
        .into_iter()
        .map(experiment_list_item_from_record)
        .collect::<Vec<_>>();
    items.sort_by(|left, right| right.created_at_ms.cmp(&left.created_at_ms));
    Ok(Json(paginate(items, pagination)))
}

async fn get_experiment_detail(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<ExperimentDetailResponse>, (StatusCode, String)> {
    let record = load_experiment_record_from_state(&state, &user_id, &experiment_id).await?;
    Ok(Json(experiment_detail_response_from_record(record)))
}

async fn save_experiment_record(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<ExperimentDetailResponse>, (StatusCode, String)> {
    let record = load_experiment_record_from_state(&state, &user_id, &experiment_id).await?;

    for variant in &record.variants {
        let variant_record = load_backtest_record_from_state(&state, &user_id, &variant.backtest_id).await?;
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
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<DiscardRuntimeArtifactResponse>, (StatusCode, String)> {
    // v1.1.9: 路径遍历防护
    let safe_id = sanitize_storage_path_segment(&experiment_id);
    let path = state
        .experiment_store_dir
        .join(format!("{}.json", safe_id));
    if fs::try_exists(&path).await.map_err(io_error)? {
        return Err((
            StatusCode::CONFLICT,
            format!(
                "experiment `{}` is already saved and cannot be discarded",
                experiment_id
            ),
        ));
    }

    let scoped_experiment_id = auth::scoped_key(&user_id, &experiment_id);
    let removed = state.experiments.write().await.remove(&scoped_experiment_id);
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
            let scoped = auth::scoped_key(&user_id, backtest_id);
            backtests.remove(&scoped);
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
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(backtest_id): Path<String>,
    Query(query): Query<RuntimeReplayQuery>,
) -> Result<Json<RuntimeReplayResponse>, (StatusCode, String)> {
    let started = Instant::now();
    let options = normalized_replay_options(query);
    let record = load_backtest_record_from_state(&state, &user_id, &backtest_id).await?;
    let response = backtest_replay_response_from_record(record, options)
        .map_err(|message| json_bad_request("bad_replay_cursor", message))?;
    state
        .evidence_metrics
        .record_replay_page(started.elapsed().as_millis() as u64);
    Ok(Json(response))
}

