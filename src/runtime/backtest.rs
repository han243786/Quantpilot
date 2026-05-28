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

    static V4_BACKTEST_SEQ: std::sync::atomic::AtomicU64 =
        std::sync::atomic::AtomicU64::new(0);
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
        || graph_json
            .pointer("/metadata/v4_machine_graph")
            .is_some()
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
            let graph = serde_json::from_value::<qrpc_core_ir::v4::V4MachineGraphContract>(
                value.clone(),
            )
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
                    format!("v4 machine graph failed static validation: {}", errors.join("; ")),
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
        .filter(|event| {
            event.source_kind == qrpc_core_ir::v4::MachineEventSourceKind::MarketData
        })
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

fn build_v4_backtest_output(
    artifact: &qrpc_core_ir::v4::V4BacktestArtifact,
    equity_curve: Vec<qrpc_core::BacktestEquityPoint>,
) -> qrpc_core::BacktestOutput {
    let initial_equity = equity_curve
        .first()
        .map(|point| point.equity)
        .unwrap_or_default();
    let final_equity = equity_curve
        .last()
        .map(|point| point.equity)
        .unwrap_or(initial_equity);
    let net_profit = final_equity - initial_equity;
    let total_return_ratio = if initial_equity.abs() > f64::EPSILON {
        net_profit / initial_equity
    } else {
        0.0
    };
    let win_rate = v4_win_rate_from_equity_curve(&equity_curve);
    qrpc_core::BacktestOutput {
        mode: "v4_backtest".to_string(),
        started_at_ms: artifact.started_at_ms,
        ended_at_ms: artifact.ended_at_ms,
        elapsed_ms: Some(artifact.ended_at_ms.saturating_sub(artifact.started_at_ms)),
        sessions: Vec::new(),
        equity_curve,
        benchmark_equity_curve: Vec::new(),
        period_returns: Vec::new(),
        summary: qrpc_core::BacktestSummary {
            step_count: artifact
                .input_tick_count
                .unwrap_or(artifact.input_bar_count),
            trade_count: artifact.execution_capability_sources.len(),
            total_return_ratio,
            final_equity,
            net_profit,
            win_rate,
            annualized_return: 0.0,
            annualized_volatility: 0.0,
            risk_adjusted: Default::default(),
            trade_analysis: Default::default(),
            drawdown_analysis: Default::default(),
            benchmark_comparison: None,
            skewness: 0.0,
            kurtosis: 0.0,
        },
        final_portfolio: v4_portfolio_from_artifact(artifact),
        v4_artifact: Some(artifact.clone()),
        debug_values: None,
    }
}

fn v4_win_rate_from_equity_curve(equity_curve: &[qrpc_core::BacktestEquityPoint]) -> f64 {
    let mut wins = 0_u64;
    let mut losses = 0_u64;
    for window in equity_curve.windows(2) {
        let prev = window[0].equity;
        let curr = window[1].equity;
        if !prev.is_finite() || !curr.is_finite() {
            continue;
        }
        if curr > prev {
            wins += 1;
        } else if curr < prev {
            losses += 1;
        }
    }
    let decisions = wins + losses;
    if decisions > 0 {
        wins as f64 / decisions as f64
    } else {
        0.0
    }
}

fn v4_equity_curve_from_artifact(
    artifact: &qrpc_core_ir::v4::V4BacktestArtifact,
) -> Vec<qrpc_core::BacktestEquityPoint> {
    let points = artifact
        .final_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.pointer("/simulated_execution/asset_curve"))
        .and_then(Value::as_array)
        .map(|points| {
            points
                .iter()
                .filter_map(|point| {
                    Some(qrpc_core::BacktestEquityPoint {
                        ts_ms: point.get("ts_ms")?.as_u64()?,
                        equity: point.get("portfolio_value")?.as_f64()?,
                        cash_balance: point.get("cash_balance")?.as_f64()?,
                        net_notional: point.get("position_market_value")?.as_f64()?,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    points
}

fn v4_portfolio_from_artifact(
    artifact: &qrpc_core_ir::v4::V4BacktestArtifact,
) -> qrpc_core::PortfolioState {
    let simulated = artifact
        .final_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.get("simulated_execution"));
    let cash = simulated
        .and_then(|value| value.get("cash_balance"))
        .and_then(Value::as_f64)
        .unwrap_or_default();
    let net_notional = simulated
        .and_then(|value| value.get("position_market_value"))
        .and_then(Value::as_f64)
        .unwrap_or_default();
    let mut portfolio = qrpc_core::PortfolioState::new(cash, artifact.ended_at_ms);
    portfolio.total_net_notional = net_notional;
    portfolio.total_gross_notional = net_notional.abs();
    portfolio
}

fn frontend_events_from_v4_backtest_artifact(
    backtest_id: &str,
    artifact: &qrpc_core_ir::v4::V4BacktestArtifact,
    equity_curve: &[qrpc_core::BacktestEquityPoint],
) -> Vec<FrontendRuntimeEvent> {
    let mut events = Vec::new();
    for (index, point) in equity_curve.iter().enumerate() {
        events.push(v4_frontend_event(
            backtest_id,
            format!("{}_v4_portfolio_{}", backtest_id, index),
            "PortfolioUpdated",
            "v4.runtime",
            "v4.execution",
            point.ts_ms,
            "Info",
            format!("v4 portfolio updated equity {:.2}", point.equity),
            json!({
                "cash_balance": point.cash_balance,
                "available_cash_balance": point.cash_balance,
                "frozen_cash_balance": 0.0,
                "total_gross_notional": point.net_notional.abs(),
                "total_net_notional": point.net_notional,
                "total_leverage": 0.0,
                "positions": 0,
                "open_orders": [],
                "equity_estimate": point.equity,
                "trace_id": format!("{}_v4_portfolio_trace_{}", backtest_id, index),
                "projection": {
                    "session_index": index,
                    "cycle_name": "v4",
                    "session_started_at_ms": point.ts_ms
                }
            }),
        ));
    }
    for (index, decision) in artifact.risk_plane_decisions.iter().enumerate() {
        events.push(v4_frontend_event(
            backtest_id,
            format!("{}_v4_risk_{}", backtest_id, index),
            "RiskDecisionProduced",
            "v4.risk_plane",
            decision.target_machine_id.clone(),
            decision.ts_ms,
            if decision.approved { "Info" } else { "Warn" },
            decision.reason.clone(),
            json!({
                "status": if decision.approved { "APPROVED" } else { "REJECTED" },
                "reason_code": if decision.approved { "V4_RISK_APPROVED" } else { "V4_RISK_REJECTED" },
                "decision": decision,
                "trace_id": format!("{}_v4_risk_trace_{}", backtest_id, index),
                "projection": {
                    "session_index": index,
                    "cycle_name": "v4",
                    "session_started_at_ms": decision.ts_ms
                }
            }),
        ));
    }
    for (index, source) in artifact.execution_capability_sources.iter().enumerate() {
        events.push(v4_frontend_event(
            backtest_id,
            format!("{}_v4_execution_capability_{}", backtest_id, index),
            "ExecutionPlanned",
            "v4.execution_capability",
            source.target_machine_id.clone(),
            source.ts_ms,
            if source.accepted { "Info" } else { "Warn" },
            source.reason.clone(),
            json!({
                "orders": if source.accepted { 1 } else { 0 },
                "capability_source": source,
                "trace_id": format!("{}_v4_execution_trace_{}", backtest_id, index),
                "projection": {
                    "session_index": index,
                    "cycle_name": "v4",
                    "session_started_at_ms": source.ts_ms
                }
            }),
        ));
    }
    for (index, point) in artifact.machine_trajectory.iter().enumerate() {
        events.push(v4_frontend_event(
            backtest_id,
            format!("{}_v4_machine_{}", backtest_id, index),
            "V4MachineTrajectoryObserved",
            "v4.machine_graph",
            point.machine_id.clone(),
            point.ts_ms,
            "Info",
            format!("{} reached {}", point.machine_id, point.state_id),
            json!({
                "trajectory": point,
                "trace_id": format!("{}_v4_machine_trace_{}", backtest_id, index),
                "projection": {
                    "session_index": index,
                    "cycle_name": "v4",
                    "session_started_at_ms": point.ts_ms
                }
            }),
        ));
    }
    events.sort_by(|left, right| {
        left.event_time_ms
            .cmp(&right.event_time_ms)
            .then_with(|| left.event_id.cmp(&right.event_id))
    });
    events
}

#[allow(clippy::too_many_arguments)]
fn v4_frontend_event(
    _backtest_id: &str,
    event_id: String,
    event_type: &str,
    source_id: impl Into<String>,
    node_id: impl Into<String>,
    event_time_ms: u64,
    severity: &str,
    summary: impl Into<String>,
    payload: Value,
) -> FrontendRuntimeEvent {
    FrontendRuntimeEvent {
        event_id,
        event_type: event_type.to_string(),
        source_id: source_id.into(),
        node_id: node_id.into(),
        event_time_ms,
        severity: severity.to_string(),
        summary: summary.into(),
        payload,
        envelope: RuntimeEventEnvelope::default(),
    }
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
pub(crate) async fn list_backtests(
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

pub(crate) async fn get_backtest_detail(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(backtest_id): Path<String>,
) -> Result<Json<BacktestDetailResponse>, (StatusCode, String)> {
    let record = load_backtest_record_from_state(&state, &user_id, &backtest_id).await?;
    Ok(Json(backtest_detail_response_from_record(record)))
}

pub(crate) async fn save_backtest_record(
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

pub(crate) async fn discard_backtest_record(
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
pub(crate) async fn list_experiments(
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

pub(crate) async fn get_experiment_detail(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<ExperimentDetailResponse>, (StatusCode, String)> {
    let record = load_experiment_record_from_state(&state, &user_id, &experiment_id).await?;
    Ok(Json(experiment_detail_response_from_record(record)))
}

pub(crate) async fn save_experiment_record(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<ExperimentDetailResponse>, (StatusCode, String)> {
    let mut record = load_experiment_record_from_state(&state, &user_id, &experiment_id).await?;

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

    record.saved = true;
    persist_experiment_record(state.experiment_store_dir.as_ref(), &record)
        .await
        .map_err(io_error)?;
    state
        .experiments
        .write()
        .await
        .insert(auth::scoped_key(&user_id, &experiment_id), record.clone());

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

pub(crate) async fn discard_experiment_record(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<DiscardRuntimeArtifactResponse>, (StatusCode, String)> {
    let record = load_experiment_record_from_state(&state, &user_id, &experiment_id).await?;
    if record.saved {
        return Err((
            StatusCode::CONFLICT,
            format!(
                "experiment `{}` is already saved and cannot be discarded",
                experiment_id
            ),
        ));
    }

    // v1.1.9: 路径遍历防护
    let safe_id = sanitize_storage_path_segment(&experiment_id);
    let path = state.experiment_store_dir.join(format!("{}.json", safe_id));

    let scoped_experiment_id = auth::scoped_key(&user_id, &experiment_id);
    state.experiments.write().await.remove(&scoped_experiment_id);
    if fs::try_exists(&path).await.map_err(io_error)? {
        fs::remove_file(&path).await.map_err(io_error)?;
    }

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

pub(crate) async fn get_backtest_replay(
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

#[cfg(test)]
mod tests {
    use super::*;

    fn point(ts_ms: u64, equity: f64) -> qrpc_core::BacktestEquityPoint {
        qrpc_core::BacktestEquityPoint {
            ts_ms,
            equity,
            cash_balance: equity,
            net_notional: 0.0,
        }
    }

    #[test]
    fn v4_win_rate_counts_up_steps_over_directional_steps() {
        let equity_curve = vec![
            point(0, 100.0),
            point(1, 110.0),
            point(2, 110.0),
            point(3, 90.0),
            point(4, 120.0),
        ];

        assert_eq!(v4_win_rate_from_equity_curve(&equity_curve), 2.0 / 3.0);
    }

    #[test]
    fn v4_equity_curve_empty_artifact_does_not_fabricate_zero_point() {
        let artifact = qrpc_core_ir::v4::V4BacktestArtifact {
            schema_version: "quantpilot/v4-backtest-artifact/v1".to_string(),
            graph_id: "graph_empty".to_string(),
            started_at_ms: 1,
            ended_at_ms: 2,
            replay_mode: "deterministic_bar_replay".to_string(),
            input_bar_count: 0,
            input_tick_count: None,
            symbols: Vec::new(),
            machine_trajectory: Vec::new(),
            risk_plane_decisions: Vec::new(),
            execution_capability_sources: Vec::new(),
            microstructure_metrics: None,
            final_snapshot: None,
        };

        assert!(v4_equity_curve_from_artifact(&artifact).is_empty());
    }
}
