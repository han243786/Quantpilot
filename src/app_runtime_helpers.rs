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
    let report_store_dir = backtest_store_dir
        .parent()
        .map(|path| path.join("reports"))
        .unwrap_or_else(|| PathBuf::from("storage/reports"));
    let mutation_store_dir = backtest_store_dir
        .parent()
        .map(|path| path.join("mutations"))
        .unwrap_or_else(|| PathBuf::from("storage/mutations"));
    let ai_proposal_store_dir = backtest_store_dir
        .parent()
        .map(|path| path.join("ai-proposals"))
        .unwrap_or_else(|| PathBuf::from("storage/ai-proposals"));
    let transient_backtest_store_dir =
        transient_backtest_store_dir_from_backtest_store_dir(&backtest_store_dir);
    // Block 5 新存储目录
    let approval_store_dir = backtest_store_dir
        .parent()
        .map(|path| path.join("approvals"))
        .unwrap_or_else(|| PathBuf::from("storage/approvals"));
    let sandbox_report_store_dir = backtest_store_dir
        .parent()
        .map(|path| path.join("sandbox-reports"))
        .unwrap_or_else(|| PathBuf::from("storage/sandbox-reports"));
    let alert_store_dir = backtest_store_dir
        .parent()
        .map(|path| path.join("alerts"))
        .unwrap_or_else(|| PathBuf::from("storage/alerts"));
    let snapshot_store_dir = backtest_store_dir
        .parent()
        .map(|path| path.join("snapshots"))
        .unwrap_or_else(|| PathBuf::from("storage/snapshots"));
    let chaos_store_dir = backtest_store_dir
        .parent()
        .map(|path| path.join("chaos"))
        .unwrap_or_else(|| PathBuf::from("storage/chaos"));
    AppState {
        runs: Arc::new(RwLock::new(BTreeMap::new())),
        backtests: Arc::new(RwLock::new(BTreeMap::new())),
        experiments: Arc::new(RwLock::new(BTreeMap::new())),
        parameter_mutations: Arc::new(RwLock::new(BTreeMap::new())),
        ai_proposals: Arc::new(RwLock::new(BTreeMap::new())),
        evidence_metrics: Arc::new(RuntimeEvidenceMetrics::default()),
        graph_store_dir: Arc::new(graph_store_dir),
        run_store_dir: Arc::new(run_store_dir),
        backtest_store_dir: Arc::new(backtest_store_dir),
        experiment_store_dir: Arc::new(experiment_store_dir),
        audit_store_dir: Arc::new(audit_store_dir),
        report_store_dir: Arc::new(report_store_dir),
        mutation_store_dir: Arc::new(mutation_store_dir),
        ai_proposal_store_dir: Arc::new(ai_proposal_store_dir),
        transient_backtest_store_dir: Arc::new(transient_backtest_store_dir),
        transient_backtest_spill_threshold_bytes: DEFAULT_TRANSIENT_BACKTEST_SPILL_THRESHOLD_BYTES,
        hotswap_records: Arc::new(RwLock::new(BTreeMap::new())),
        // Block 5 新字段
        approval_records: Arc::new(RwLock::new(BTreeMap::new())),
        sandbox_reports: Arc::new(RwLock::new(BTreeMap::new())),
        alert_rules: Arc::new(RwLock::new(Vec::new())),
        alert_firings: Arc::new(RwLock::new(BTreeMap::new())),
        snapshots: Arc::new(RwLock::new(BTreeMap::new())),
        chaos_experiments: Arc::new(RwLock::new(BTreeMap::new())),
        approval_store_dir: Arc::new(approval_store_dir),
        sandbox_report_store_dir: Arc::new(sandbox_report_store_dir),
        alert_store_dir: Arc::new(alert_store_dir),
        snapshot_store_dir: Arc::new(snapshot_store_dir),
        chaos_store_dir: Arc::new(chaos_store_dir),
        chaos_mode: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        config_generation: Arc::new(std::sync::atomic::AtomicU64::new(1)),
        config_generation_history: Arc::new(std::sync::Mutex::new(Vec::new())),
        #[cfg(test)]
        test_storage_root: None,
    }
}

fn transient_backtest_store_dir_from_backtest_store_dir(backtest_store_dir: &FsPath) -> PathBuf {
    let parent = backtest_store_dir
        .parent()
        .unwrap_or_else(|| FsPath::new("."));
    let project_root = if parent.file_name().is_some_and(|name| name == "storage") {
        parent
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| FsPath::new("."))
    } else {
        parent
    };

    project_root
        .join(".quantpilot-tmp")
        .join("runtime-artifacts")
        .join("backtests")
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

#[allow(clippy::too_many_arguments)]
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
        RunSpecRuntimeProtocolInput {
            graph_id: request.runtime_config.metadata.graph_id.clone(),
            compile_id: request.runtime_config.metadata.compile_id.clone(),
            run_mode: RunModeSpec::Backtest,
            runtime_mode: request.runtime_config.metadata.mode.clone(),
            protocol_name: compiled.protocol_name.clone(),
            config_hash: compiled.config_hash.clone(),
            core_ir_digest: artifacts.core_ir.digest.clone(),
        },
        &compiled.config,
    );

    run_spec.execution_assumptions = execution_assumptions;
    run_spec.execution_assumption_sources = Some(execution_assumption_sources);
    let snapshot = qrpc_core::MarketDataSnapshotSpec::from_runtime_protocol(
        format!("market_snapshot_{backtest_id}"),
        artifact_replay_source(replay_source),
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

pub(super) fn epoch_ms_to_iso8601(epoch_ms: u64) -> String {
    let secs = (epoch_ms / 1000) as i64;
    let days_since_epoch = secs / 86400;
    let mut remaining = secs % 86400;
    if remaining < 0 {
        remaining += 86400;
    }
    let hours = remaining / 3600;
    remaining %= 3600;
    let minutes = remaining / 60;
    let secs_remaining = remaining % 60;

    let mut year = 1970i64;
    let mut days = days_since_epoch;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let month_days = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1usize;
    for md in month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    let day = days + 1;
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, hours, minutes, secs_remaining)
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
