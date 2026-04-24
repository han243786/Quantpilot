use super::*;

pub(super) fn new_app_state(
    graph_store_dir: PathBuf,
    run_store_dir: PathBuf,
    backtest_store_dir: PathBuf,
) -> AppState {
    let experiment_store_dir = backtest_store_dir
        .parent()
        .map(|path| path.join("experiments"))
        .unwrap_or_else(|| PathBuf::from("storage/experiments"));
    let audit_store_dir = backtest_store_dir
        .parent()
        .map(|path| path.join("audit"))
        .unwrap_or_else(|| PathBuf::from("storage/audit"));
    AppState {
        runs: Arc::new(RwLock::new(BTreeMap::new())),
        backtests: Arc::new(RwLock::new(BTreeMap::new())),
        experiments: Arc::new(RwLock::new(BTreeMap::new())),
        graph_store_dir: Arc::new(graph_store_dir),
        run_store_dir: Arc::new(run_store_dir),
        backtest_store_dir: Arc::new(backtest_store_dir),
        experiment_store_dir: Arc::new(experiment_store_dir),
        audit_store_dir: Arc::new(audit_store_dir),
    }
}

pub(super) async fn health() -> impl IntoResponse {
    Json(HealthResponse { status: "ok" })
}

pub(super) fn artifact_replay_source(
    source: FrontendBacktestReplaySource,
) -> ArtifactBacktestReplaySource {
    match source {
        FrontendBacktestReplaySource::HistoricalReplay => {
            ArtifactBacktestReplaySource::HistoricalReplay
        }
        FrontendBacktestReplaySource::DeterministicMock => {
            ArtifactBacktestReplaySource::DeterministicMock
        }
    }
}

pub(super) fn build_backtest_spec(
    backtest_id: &str,
    replay_source: FrontendBacktestReplaySource,
    request: &FrontendRunRequest,
    compiled: &qrpc_core::CompiledRuntimeProtocol,
    artifacts: &CompileArtifactBundle,
    requested_at_ms: u64,
    execution_assumptions: ExecutionAssumptionSpec,
    execution_assumption_sources: ExecutionAssumptionSourceSummary,
) -> BacktestSpec {
    let mut run_spec = RunSpec::from_runtime_protocol(
        request.runtime_config.metadata.graph_id.clone(),
        request.runtime_config.metadata.compile_id.clone(),
        RunModeSpec::Backtest,
        request.runtime_config.metadata.mode.clone(),
        compiled.protocol_name.clone(),
        compiled.config_hash.clone(),
        &compiled.config,
        artifacts.core_ir.digest.clone(),
    );

    run_spec.execution_assumptions = execution_assumptions;
    run_spec.execution_assumption_sources = Some(execution_assumption_sources);
    let snapshot = qrpc_core::MarketDataSnapshotSpec::from_runtime_protocol(
        format!("market_snapshot_{backtest_id}"),
        artifact_replay_source(replay_source.clone()),
        requested_at_ms,
        &compiled.config,
    );

    BacktestSpec::new(
        backtest_id.to_string(),
        artifact_replay_source(replay_source),
        requested_at_ms,
        run_spec,
        snapshot,
    )
}

pub(super) fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_millis() as u64
}
