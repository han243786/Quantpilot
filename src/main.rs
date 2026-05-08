mod alert_engine;
mod api_errors;
mod api_test_scenario;
mod app_router;
mod app_runtime_helpers;
mod auth_middleware;
mod credential_api;
mod credential_vault;
mod rate_limiter;
mod backtest_artifacts;
mod backtest_compare;
mod backtest_compare_core;
mod backtest_compare_narrative;
mod backtest_compare_types;
mod capability_api;
mod chaos_experiment;
mod cli_support;
mod collaboration;
mod compile_api;
mod compile_artifact_builders;
mod compile_diagnostics;
mod formal_quantscript_authoring_types;
mod frontend_api_types;
mod frontend_runtime_mapping;
mod graph_api;
mod graph_quantscript_api;
mod graph_version_compare;
mod hotswap_api;
mod runbook;
mod safe_log;
mod runtime_api;
mod runtime_diagnostics;
mod runtime_event_projection;
mod runtime_persistence;
mod runtime_response_mapping;
mod runtime_validation;
mod sandbox_verification;
mod snapshot_service;
mod storage_lifecycle;
mod test_runner;

use anyhow::{bail, Context};
use async_stream::stream;
use axum::{
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderValue, Method, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use qrpc_compiler::{
    compile_runtime_protocol_config, compile_runtime_protocol_config_with_metadata,
    lower_strategy_ir_to_core_ir,
};
use qrpc_core::{
    canonical_json_sha256_digest, declared_indicator_kinds, supported_indicator_kinds, AgentConfig,
    ArtifactDigest, BacktestOutput, BacktestReplaySource as ArtifactBacktestReplaySource,
    BacktestSpec, CompileArtifact, CompileArtifactBundle, CoreIrArtifact, DataKind,
    DataSourceConfig, DatasetSpec, Exchange, ExecutionAssumptionSourceSummary,
    ExecutionAssumptionSpec, ExecutionAssumptionValueSource, IndicatorKind, IntentConfig,
    IntentKind, MarketType, OpenOrder, PortfolioState, RiskConfig, RunModeSpec, RunSpec,
    RunSpecRuntimeProtocolInput, RuntimeEvent, RuntimeEventType, RuntimeProtocolCoreConfig,
    SessionOutput, StrategyArtifact, StrategyArtifactSourceKind, StrategyIr, UniverseSnapshot,
    COMPILE_ARTIFACT_V1_VERSION, CORE_IR_ARTIFACT_V1_VERSION,
    GLOBAL_RISK_PROFILE_DEFAULT_MAX_EXCHANGE_LEVERAGE, GLOBAL_RISK_PROFILE_DEFAULT_MAX_POSITION,
    GLOBAL_RISK_PROFILE_DEFAULT_MAX_TOTAL_LEVERAGE,
    GLOBAL_RISK_PROFILE_DEFAULT_MIN_ACTION_INTERVAL_MS, GLOBAL_RISK_PROFILE_ID,
    PAPER_EXECUTION_PROFILE_DEFAULT_FEE_BPS, PAPER_EXECUTION_PROFILE_DEFAULT_SLIPPAGE_BPS,
    PAPER_EXECUTION_PROFILE_ID, STRATEGY_ARTIFACT_V1_VERSION,
};
use qrpc_core_ir::{CoreMetadata, CoreSourceKind};
use qrpc_runtime::{
    runtime_support_boundary, DeterministicTestMode, FastBacktestSandbox, RealTimeSandbox,
    RuntimeCoordinator, Sandbox, SUPPORTED_RUNTIME_EXECUTION_MODULE_KEYS,
    SUPPORTED_RUNTIME_MODE_KEYS,
};
use quantscript::{
    analyze_script_module as analyze_formal_script_module, extract_formal_instrument_pool_spec,
    lower_script_to_runtime_config_with_context as lower_formal_script_to_runtime_config,
    lower_script_to_typed_hir as lower_formal_script_to_typed_hir,
    parse_quant_script_module as parse_formal_quant_script_module, Expr as FormalExpr,
    InstrumentPoolSelectionKey, InstrumentPoolSourceSpec, InstrumentPoolSpec, InstrumentPoolValue,
    LoweringContext as FormalLoweringContext, ResolveResult as FormalResolveResult,
    ScriptModule as FormalScriptModule, Stmt as FormalStmt,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    convert::Infallible,
    env,
    net::SocketAddr,
    path::{Path as FsPath, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::{fs, sync::RwLock, time::sleep};
use tower_http::cors::{Any, CorsLayer};

use api_errors::*;
use app_router::*;
use app_runtime_helpers::*;
use backtest_artifacts::{
    build_backtest_artifact_views, cleanup_backtest_promotion_work_dirs,
    cleanup_transient_backtest_records, delete_transient_backtest_record,
    is_backtest_promotion_work_dir, load_backtest_record_from_directory,
    load_transient_backtest_record, maybe_spill_transient_backtest_record,
    persist_backtest_artifacts, BacktestArtifactViews, ExecutionAssumptionsModule,
    ExecutionAssumptionsTag, DEFAULT_TRANSIENT_BACKTEST_SPILL_THRESHOLD_BYTES,
};
#[cfg(test)]
use backtest_compare_core::*;
#[cfg(test)]
use backtest_compare_narrative::*;
use backtest_compare_types::*;
use capability_api::*;
use cli_support::*;
use collaboration::*;
use compile_api::*;
use compile_artifact_builders::*;
use compile_diagnostics::*;
use formal_quantscript_authoring_types::*;
use frontend_api_types::*;
use frontend_runtime_mapping::*;
use graph_api::*;
use graph_quantscript_api::*;
use graph_version_compare::*;
use runtime_api::*;
use runtime_diagnostics::*;
use runtime_event_projection::*;
use runtime_persistence::*;
use runtime_response_mapping::*;
use runtime_validation::*;

const RUN_WINDOW_MS: u64 = 5_000;
const SSE_EVENT_DELAY_MS: u64 = 350;
const BACKTEST_DETERMINISTIC_SEED: u64 = 7;
const CAPABILITY_API_VERSION: &str = "quantpilot-capabilities/v1";
const CAPABILITY_SCHEMA_VERSION: &str = "quantpilot/capabilities-schema/v1";
const CAPABILITY_PERMISSION_MODEL_VERSION: &str = "quantpilot/permission-boundary/v1";
const CAPABILITY_VERSIONING_MODEL_VERSION: &str = "quantpilot/versioning-model/v1";
const RUNTIME_GOVERNANCE_SCHEMA_VERSION: &str = "quantpilot/runtime-governance/v1";
const RUNTIME_CHAIN_STAGES: [&str; 6] = ["data", "intent", "agent", "risk", "execution", "fill"];

const DECLARED_FRONTEND_MODULE_KEYS: [&str; 14] = [
    "builtin.data.kline",
    "builtin.data.quote",
    "builtin.intent.double_ma",
    "builtin.intent.ma_deviation",
    "builtin.intent.rsi",
    "builtin.intent.macd",
    "builtin.intent.momentum",
    "builtin.intent.zscore",
    "builtin.intent.spread_observer",
    "builtin.agent.weighted",
    "builtin.agent.arbitrage",
    "builtin.risk.global",
    "builtin.execution.paper",
    "builtin.runtime.control",
];

const SUPPORTED_FRONTEND_MODULE_KEYS: [&str; 14] = [
    "builtin.data.kline",
    "builtin.data.quote",
    "builtin.intent.double_ma",
    "builtin.intent.ma_deviation",
    "builtin.intent.rsi",
    "builtin.intent.macd",
    "builtin.intent.momentum",
    "builtin.intent.zscore",
    "builtin.intent.spread_observer",
    "builtin.agent.weighted",
    "builtin.agent.arbitrage",
    "builtin.risk.global",
    "builtin.execution.paper",
    "builtin.runtime.control",
];

#[derive(Clone)]
struct AppState {
    runs: Arc<RwLock<BTreeMap<String, RunRecord>>>,
    backtests: Arc<RwLock<BTreeMap<String, BacktestRecord>>>,
    experiments: Arc<RwLock<BTreeMap<String, ExperimentRecord>>>,
    parameter_mutations: Arc<RwLock<BTreeMap<String, RuntimeParameterMutationRecord>>>,
    ai_proposals: Arc<RwLock<BTreeMap<String, RuntimeAiProposalRecord>>>,
    evidence_metrics: Arc<RuntimeEvidenceMetrics>,
    graph_store_dir: Arc<PathBuf>,
    run_store_dir: Arc<PathBuf>,
    backtest_store_dir: Arc<PathBuf>,
    experiment_store_dir: Arc<PathBuf>,
    audit_store_dir: Arc<PathBuf>,
    report_store_dir: Arc<PathBuf>,
    mutation_store_dir: Arc<PathBuf>,
    ai_proposal_store_dir: Arc<PathBuf>,
    hotswap_records: Arc<RwLock<BTreeMap<String, HotSwapRecord>>>,
    transient_backtest_store_dir: Arc<PathBuf>,
    transient_backtest_spill_threshold_bytes: u64,
    // Block 5 新增
    approval_records: Arc<RwLock<BTreeMap<String, RuntimeApprovalRecord>>>,
    sandbox_reports: Arc<RwLock<BTreeMap<String, SandboxVerificationReport>>>,
    alert_rules: Arc<RwLock<Vec<AlertRule>>>,
    alert_firings: Arc<RwLock<BTreeMap<String, AlertFiring>>>,
    snapshots: Arc<RwLock<BTreeMap<String, DeploymentSignatureSnapshot>>>,
    chaos_experiments: Arc<RwLock<BTreeMap<String, ChaosExperimentReport>>>,
    approval_store_dir: Arc<PathBuf>,
    sandbox_report_store_dir: Arc<PathBuf>,
    alert_store_dir: Arc<PathBuf>,
    snapshot_store_dir: Arc<PathBuf>,
    chaos_store_dir: Arc<PathBuf>,
    chaos_mode: Arc<std::sync::atomic::AtomicBool>,
    config_generation: Arc<AtomicU64>,
    config_generation_history: Arc<std::sync::Mutex<Vec<qrpc_runtime::ConfigGenerationEntry>>>,
    credential_vault: Option<Arc<credential_vault::CredentialVault>>,
    #[cfg(test)]
    test_storage_root: Option<Arc<TestStorageRoot>>,
}

#[derive(Default)]
struct RuntimeEvidenceMetrics {
    report_generation_count: AtomicU64,
    report_generation_failure_count: AtomicU64,
    report_source_changed_count: AtomicU64,
    replay_page_count: AtomicU64,
    replay_page_latency_total_ms: AtomicU64,
    compact_projection_source_event_count_total: AtomicU64,
    compact_projection_retained_event_count_total: AtomicU64,
    compact_detail_window_required_count: AtomicU64,
    mutation_proposal_created_count: AtomicU64,
    mutation_proposal_rejected_count: AtomicU64,
    mutation_activation_scheduled_count: AtomicU64,
    mutation_activation_applied_count: AtomicU64,
    mutation_activation_failed_count: AtomicU64,
    mutation_activation_latency_total_ms: AtomicU64,
    mutation_safe_window_denied_count: AtomicU64,
    mutation_rollback_attempt_count: AtomicU64,
    mutation_rollback_scheduled_count: AtomicU64,
    mutation_rollback_applied_count: AtomicU64,
    mutation_rollback_failed_count: AtomicU64,
}

impl RuntimeEvidenceMetrics {
    fn record_report_generation(&self, report: &RuntimeEvidenceReportRecord) {
        self.report_generation_count.fetch_add(1, Ordering::Relaxed);
        if report.status != RuntimeReportLifecycleStatus::Ready {
            self.report_generation_failure_count
                .fetch_add(1, Ordering::Relaxed);
        }
        self.compact_projection_source_event_count_total
            .fetch_add(report.source_event_count as u64, Ordering::Relaxed);
        self.compact_projection_retained_event_count_total
            .fetch_add(report.retained_event_count as u64, Ordering::Relaxed);
        if report.retained_event_count == 0
            || report.retained_event_count == report.source_event_count
        {
            self.compact_detail_window_required_count
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    fn record_report_source_changed(&self) {
        self.report_source_changed_count
            .fetch_add(1, Ordering::Relaxed);
        self.report_generation_failure_count
            .fetch_add(1, Ordering::Relaxed);
    }

    fn record_replay_page(&self, latency_ms: u64) {
        self.replay_page_count.fetch_add(1, Ordering::Relaxed);
        self.replay_page_latency_total_ms
            .fetch_add(latency_ms, Ordering::Relaxed);
    }

    fn record_mutation_proposal(&self, status: RuntimeParameterMutationStatus) {
        match status {
            RuntimeParameterMutationStatus::Proposed => {
                self.mutation_proposal_created_count
                    .fetch_add(1, Ordering::Relaxed);
            }
            RuntimeParameterMutationStatus::Rejected => {
                self.mutation_proposal_rejected_count
                    .fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    fn record_mutation_activation_scheduled(&self) {
        self.mutation_activation_scheduled_count
            .fetch_add(1, Ordering::Relaxed);
    }

    fn record_mutation_activation_applied(&self, latency_ms: u64) {
        self.mutation_activation_applied_count
            .fetch_add(1, Ordering::Relaxed);
        self.mutation_activation_latency_total_ms
            .fetch_add(latency_ms, Ordering::Relaxed);
    }

    fn record_mutation_activation_failed(&self) {
        self.mutation_activation_failed_count
            .fetch_add(1, Ordering::Relaxed);
    }

    fn record_mutation_safe_window_denied(&self) {
        self.mutation_safe_window_denied_count
            .fetch_add(1, Ordering::Relaxed);
    }

    fn record_mutation_rollback_attempt(&self) {
        self.mutation_rollback_attempt_count
            .fetch_add(1, Ordering::Relaxed);
    }

    fn record_mutation_rollback_scheduled(&self) {
        self.mutation_rollback_scheduled_count
            .fetch_add(1, Ordering::Relaxed);
    }

    fn record_mutation_rollback_applied(&self) {
        self.mutation_rollback_applied_count
            .fetch_add(1, Ordering::Relaxed);
    }

    fn record_mutation_rollback_failed(&self) {
        self.mutation_rollback_failed_count
            .fetch_add(1, Ordering::Relaxed);
    }

    fn snapshot(&self) -> RuntimeEvidenceMetricsSnapshot {
        let replay_page_count = self.replay_page_count.load(Ordering::Relaxed);
        let replay_page_latency_total_ms =
            self.replay_page_latency_total_ms.load(Ordering::Relaxed);
        let mutation_activation_applied_count = self
            .mutation_activation_applied_count
            .load(Ordering::Relaxed);
        let mutation_activation_latency_total_ms = self
            .mutation_activation_latency_total_ms
            .load(Ordering::Relaxed);
        RuntimeEvidenceMetricsSnapshot {
            report_generation_count: self.report_generation_count.load(Ordering::Relaxed),
            report_generation_failure_count: self
                .report_generation_failure_count
                .load(Ordering::Relaxed),
            report_source_changed_count: self.report_source_changed_count.load(Ordering::Relaxed),
            replay_page_count,
            replay_page_latency_total_ms,
            replay_page_latency_avg_ms: if replay_page_count == 0 {
                0.0
            } else {
                replay_page_latency_total_ms as f64 / replay_page_count as f64
            },
            compact_projection_source_event_count_total: self
                .compact_projection_source_event_count_total
                .load(Ordering::Relaxed),
            compact_projection_retained_event_count_total: self
                .compact_projection_retained_event_count_total
                .load(Ordering::Relaxed),
            compact_detail_window_required_count: self
                .compact_detail_window_required_count
                .load(Ordering::Relaxed),
            mutation_proposal_created_count: self
                .mutation_proposal_created_count
                .load(Ordering::Relaxed),
            mutation_proposal_rejected_count: self
                .mutation_proposal_rejected_count
                .load(Ordering::Relaxed),
            mutation_activation_scheduled_count: self
                .mutation_activation_scheduled_count
                .load(Ordering::Relaxed),
            mutation_activation_applied_count,
            mutation_activation_failed_count: self
                .mutation_activation_failed_count
                .load(Ordering::Relaxed),
            mutation_activation_latency_total_ms,
            mutation_activation_latency_avg_ms: if mutation_activation_applied_count == 0 {
                0.0
            } else {
                mutation_activation_latency_total_ms as f64
                    / mutation_activation_applied_count as f64
            },
            mutation_safe_window_denied_count: self
                .mutation_safe_window_denied_count
                .load(Ordering::Relaxed),
            mutation_rollback_attempt_count: self
                .mutation_rollback_attempt_count
                .load(Ordering::Relaxed),
            mutation_rollback_scheduled_count: self
                .mutation_rollback_scheduled_count
                .load(Ordering::Relaxed),
            mutation_rollback_applied_count: self
                .mutation_rollback_applied_count
                .load(Ordering::Relaxed),
            mutation_rollback_failed_count: self
                .mutation_rollback_failed_count
                .load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
struct TestStorageRoot {
    path: PathBuf,
}

#[cfg(test)]
impl Drop for TestStorageRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct CapabilityResponse {
    api_version: &'static str,
    schema_version: &'static str,
    schema_hash: String,
    chain_stages: Vec<&'static str>,
    strategy_ir: StrategyIrCapabilitySummary,
    runtime: RuntimeCapabilitySummary,
    market_data: MarketDataCapabilitySummary,
    frontend: FrontendCapabilitySummary,
    versioning: CapabilityVersioningSummary,
    permission_boundary: CapabilityPermissionBoundarySummary,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
struct CapabilityVersioningSummary {
    model_version: &'static str,
    strategy_version_source: &'static str,
    parameter_version_policy: &'static str,
    deployment_revision_policy: &'static str,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
struct CapabilityPermissionBoundarySummary {
    model_version: &'static str,
    execution_owner_module: &'static str,
    live_execution_allowed: bool,
    ai_write_policy: AiWritePolicy,
    plugin_network_default: BoundaryAccessPolicy,
    non_execution_order_access: BoundaryAccessPolicy,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum AiWritePolicy {
    ProposalOnly,
    Disabled,
}

impl AiWritePolicy {
    fn as_str(self) -> &'static str {
        match self {
            Self::ProposalOnly => "proposal_only",
            Self::Disabled => "disabled",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum BoundaryAccessPolicy {
    Deny,
    Allow,
}

impl BoundaryAccessPolicy {
    fn as_str(self) -> &'static str {
        match self {
            Self::Deny => "deny",
            Self::Allow => "allow",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum RuntimeEventStage {
    Data,
    Intent,
    Agent,
    Risk,
    Execution,
    Fill,
    System,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum RuntimeEventRetentionClass {
    Key,
    Summary,
    Debug,
}

#[derive(Debug, Serialize)]
struct StrategyIrCapabilitySummary {
    declared_indicator_kinds: Vec<IndicatorKind>,
    supported_indicator_kinds: Vec<IndicatorKind>,
    indicator_support: Vec<IndicatorCapabilityEntry>,
}

#[derive(Debug, Serialize)]
struct RuntimeCapabilitySummary {
    supported_modes: Vec<&'static str>,
    supported_execution_modules: Vec<&'static str>,
    mode_support: Vec<NamedCapabilityEntry>,
    execution_module_support: Vec<NamedCapabilityEntry>,
}

#[derive(Debug, Serialize)]
struct MarketDataCapabilitySummary {
    supported_exchanges: Vec<&'static str>,
    supported_symbols: Vec<&'static str>,
    exchange_support: Vec<NamedCapabilityEntry>,
    symbol_support: Vec<NamedCapabilityEntry>,
}

#[derive(Debug, Serialize)]
struct FrontendCapabilitySummary {
    declared_module_keys: Vec<&'static str>,
    supported_module_keys: Vec<&'static str>,
    unsupported_module_reasons: BTreeMap<&'static str, &'static str>,
    module_support: Vec<ModuleCapabilityEntry>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum CapabilitySupportStatus {
    Supported,
    DeclaredOnly,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
struct IndicatorCapabilityEntry {
    kind: IndicatorKind,
    status: CapabilitySupportStatus,
    reason: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
struct NamedCapabilityEntry {
    key: &'static str,
    status: CapabilitySupportStatus,
    reason: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
struct ModuleCapabilityEntry {
    module_key: &'static str,
    status: CapabilitySupportStatus,
    reason: Option<&'static str>,
}

#[derive(Debug, Serialize)]
struct ApiErrorDetail {
    code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<String>,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    span_label: Option<String>,
    reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct ApiErrorResponse {
    error: &'static str,
    message: String,
    details: Vec<ApiErrorDetail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    partial_artifacts: Option<ApiPartialArtifacts>,
}

#[derive(Debug, Serialize)]
struct ApiPartialArtifacts {
    #[serde(skip_serializing_if = "Option::is_none")]
    quantscript_authoring_view: Option<QuantScriptAuthoringView>,
}

const SUPPORTED_EXCHANGES: [&str; 2] = ["binance", "okx"];
const SUPPORTED_SYMBOLS: [&str; 3] = ["BTCUSDT", "ETHUSDT", "SOLUSDT"];

#[tokio::main]
#[cfg_attr(test, allow(dead_code))]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "credential" {
        if let Err(e) = cli_support::handle_credential_command(&args[1..]) {
            eprintln!("错误: {}", e);
            std::process::exit(1);
        }
        return Ok(());
    }
    match parse_cli_command_from(env::args())? {
        CliCommand::Serve => run_api_server().await,
        CliCommand::PrintHelp => {
            print_cli_usage();
            Ok(())
        }
        CliCommand::StrategyIrValidate { path } => validate_strategy_ir_file(path).await,
    }
}

#[cfg_attr(test, allow(dead_code))]
async fn run_api_server() -> anyhow::Result<()> {
    let graph_store_dir = PathBuf::from("storage/graphs");
    let run_store_dir = PathBuf::from("storage/runs");
    let backtest_store_dir = PathBuf::from("storage/backtests");
    let experiment_store_dir = PathBuf::from("storage/experiments");
    // Block 5 新存储目录
    let approval_store_dir = PathBuf::from("storage/approvals");
    let sandbox_report_store_dir = PathBuf::from("storage/sandbox-reports");
    let alert_store_dir = PathBuf::from("storage/alerts");
    let snapshot_store_dir = PathBuf::from("storage/snapshots");
    let chaos_store_dir = PathBuf::from("storage/chaos");
    let audit_store_dir = PathBuf::from("storage/audit");
    let report_store_dir = PathBuf::from("storage/reports");
    fs::create_dir_all(&graph_store_dir).await?;
    fs::create_dir_all(&run_store_dir).await?;
    fs::create_dir_all(&backtest_store_dir).await?;
    fs::create_dir_all(&experiment_store_dir).await?;
    fs::create_dir_all(&approval_store_dir).await?;
    fs::create_dir_all(&sandbox_report_store_dir).await?;
    fs::create_dir_all(&alert_store_dir).await?;
    fs::create_dir_all(&snapshot_store_dir).await?;
    fs::create_dir_all(&chaos_store_dir).await?;
    fs::create_dir_all(&audit_store_dir).await?;
    fs::create_dir_all(&report_store_dir).await?;
    if let Err(error) = cleanup_backtest_promotion_work_dirs(&backtest_store_dir).await {
        eprintln!(
            "warning: failed to clean stale backtest promotion temp directories: {}",
            error
        );
    }

    let state = new_app_state(graph_store_dir, run_store_dir, backtest_store_dir);
    // Block 5: 初始化告警规则
    alert_engine::init_alert_rules(&state).await;
    // Block 5: 从磁盘预热持久化数据
    warm_persisted_state(&state).await;
    if let Err(error) =
        cleanup_transient_backtest_records(state.transient_backtest_store_dir.as_ref()).await
    {
        eprintln!(
            "warning: failed to clean stale transient backtest directories: {}",
            error
        );
    }

    // 启动时清理过期存储文件和构建工件
    storage_lifecycle::startup_storage_cleanup(std::path::Path::new("storage"));
    storage_lifecycle::cleanup_build_artifacts();

    let cors_origin = env::var("QUANTPILOT_CORS_ORIGIN")
        .unwrap_or_else(|_| "http://127.0.0.1:5173,http://localhost:5173".to_string());
    let cors_origins: Vec<HeaderValue> = cors_origin
        .split(',')
        .filter_map(|s| HeaderValue::from_str(s.trim()).ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(cors_origins)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any);

    let app = build_app_router(state.clone())
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB
        .layer(cors)
        .layer(axum::middleware::from_fn(json_rejection_middleware))
        .layer(axum::middleware::from_fn(
            rate_limiter::rate_limit_middleware,
        ))
        .layer(axum::middleware::from_fn(
            auth_middleware::api_key_auth,
        ));

    // Block 5 P1-5 + P3-4: 审批超时 + 观察窗口后台任务
    let expiry_state = state.clone();
    tokio::spawn(async move {
        let mut tick: u64 = 0;
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            tick += 1;
            process_expired_approvals(&expiry_state).await;
            check_observation_windows(&expiry_state).await;
            // 每小时清理一次过期存储文件
            if tick % 60 == 0 {
                storage_lifecycle::startup_storage_cleanup(std::path::Path::new("storage"));
            }
        }
    });

    let port: u16 = env::var("QUANTPILOT_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("QuantPilot API listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn json_rejection_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let response = next.run(req).await;
    if response.status() == StatusCode::UNPROCESSABLE_ENTITY {
        let body = axum::Json(serde_json::json!({
            "error": "bad_request",
            "message": "请求格式错误: 无法解析 JSON 请求体"
        }));
        return (StatusCode::BAD_REQUEST, body).into_response();
    }
    response
}

async fn warm_persisted_state(state: &AppState) {
    // 从磁盘加载审批记录
    if let Ok(mut entries) = fs::read_dir(state.approval_store_dir.as_ref()).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(data) = fs::read(entry.path()).await {
                if let Ok(record) = serde_json::from_slice::<RuntimeApprovalRecord>(&data) {
                    state
                        .approval_records
                        .write()
                        .await
                        .insert(record.approval_id.clone(), record);
                }
            }
        }
    }
    // 从磁盘加载快照
    if let Ok(mut entries) = fs::read_dir(state.snapshot_store_dir.as_ref()).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(data) = fs::read(entry.path()).await {
                if let Ok(snapshot) = serde_json::from_slice::<DeploymentSignatureSnapshot>(&data) {
                    state
                        .snapshots
                        .write()
                        .await
                        .insert(snapshot.snapshot_id.clone(), snapshot);
                }
            }
        }
    }
    // 从磁盘加载告警 firing 状态
    if let Ok(mut entries) = fs::read_dir(state.alert_store_dir.as_ref()).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(data) = fs::read(entry.path()).await {
                if let Ok(firing) = serde_json::from_slice::<AlertFiring>(&data) {
                    state
                        .alert_firings
                        .write()
                        .await
                        .insert(firing.firing_id.clone(), firing);
                }
            }
        }
    }
    // 从磁盘加载沙箱报告
    if let Ok(mut entries) = fs::read_dir(state.sandbox_report_store_dir.as_ref()).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(data) = fs::read(entry.path()).await {
                if let Ok(report) = serde_json::from_slice::<SandboxVerificationReport>(&data) {
                    state
                        .sandbox_reports
                        .write()
                        .await
                        .insert(report.proposal_id.clone(), report);
                }
            }
        }
    }
    // 从磁盘加载混沌实验
    if let Ok(mut entries) = fs::read_dir(state.chaos_store_dir.as_ref()).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(data) = fs::read(entry.path()).await {
                if let Ok(experiment) = serde_json::from_slice::<ChaosExperimentReport>(&data) {
                    state
                        .chaos_experiments
                        .write()
                        .await
                        .insert(experiment.experiment_id.clone(), experiment);
                }
            }
        }
    }
    eprintln!(
        "[startup] warmed state: {} approvals, {} snapshots, {} alerts, {} sandbox reports, {} chaos experiments",
        state.approval_records.read().await.len(),
        state.snapshots.read().await.len(),
        state.alert_firings.read().await.len(),
        state.sandbox_reports.read().await.len(),
        state.chaos_experiments.read().await.len(),
    );
}

// Block 5 P1-5: 审批超时自动处理
async fn process_expired_approvals(state: &AppState) {
    let now_ms = current_time_ms();
    let mut approvals = state.approval_records.write().await;
    for approval in approvals.values_mut() {
        if approval.review_state == RuntimeApprovalReviewState::Pending
            || approval.review_state == RuntimeApprovalReviewState::UnderReview
        {
            if now_ms > approval.expires_at_ms {
                approval.review_state = RuntimeApprovalReviewState::Expired;
                approval.lifecycle.push(RuntimeApprovalLifecycleEntry {
                    review_state: RuntimeApprovalReviewState::Expired,
                    event_id: format!("event_apr_expired_{}", now_ms),
                    sequence_no: approval.lifecycle.len() as u64 + 1,
                    occurred_at_ms: now_ms,
                    reason_code: "APPROVAL_EXPIRED".to_string(),
                    message: format!(
                        "审批单已过期 (L{}审批, {}h超时)",
                        match approval.approval_level {
                            RuntimeApprovalLevel::L1SingleReviewer => 1,
                            RuntimeApprovalLevel::L2DualReviewer => 2,
                            RuntimeApprovalLevel::L3RiskOwnerReview => 3,
                        },
                        match approval.approval_level {
                            RuntimeApprovalLevel::L1SingleReviewer => 24,
                            RuntimeApprovalLevel::L2DualReviewer => 48,
                            RuntimeApprovalLevel::L3RiskOwnerReview => 72,
                        },
                    ),
                    actor_id: None,
                });
                // 更新对应 AI 提案状态为 Expired
                let mut proposals = state.ai_proposals.write().await;
                if let Some(proposal) = proposals.get_mut(&approval.proposal_id) {
                    proposal.status = RuntimeAiProposalStatus::Expired;
                }
                // 持久化
                let _ = serde_json::to_vec_pretty(&*approval).ok().and_then(|json| {
                    let dir = state.approval_store_dir.to_path_buf();
                    let id = approval.approval_id.clone();
                    tokio::task::spawn_blocking(move || {
                        let _ = std::fs::create_dir_all(&dir);
                        let _ = std::fs::write(dir.join(format!("{}.json", id)), &json);
                    });
                    None::<()>
                });
            }
        }
    }
}

// Block 5 P3-4: 观察窗口检查
async fn check_observation_windows(state: &AppState) {
    let now_ms = current_time_ms();
    let risk_reject = state
        .evidence_metrics
        .mutation_proposal_rejected_count
        .load(Ordering::Relaxed);
    let rollback_count = state
        .evidence_metrics
        .mutation_rollback_attempt_count
        .load(Ordering::Relaxed);

    // 检查最近的 mutation 激活记录，若在观察窗口内且异常，触发告警
    let mutations = state.parameter_mutations.read().await;
    for mutation in mutations.values() {
        if let Some(ref activation) = mutation.activation_state {
            if let Some(deadline_ms) = activation.observation_deadline_ms {
                if now_ms < deadline_ms {
                    // 仍在观察窗口内
                    if risk_reject > 100 || rollback_count > 0 {
                        // 异常检测：触发告警
                        let alert_id = format!(
                            "alert-observation-{}-{}",
                            mutation.proposal_id, now_ms
                        );
                        let firing = AlertFiring {
                            firing_id: alert_id.clone(),
                            rule_name: "hotswap_rollback_occurred".to_string(),
                            severity: AlertSeverity::P2,
                            state: AlertFiringState::Firing,
                            fired_at_ms: now_ms,
                            acknowledged_at_ms: None,
                            resolved_at_ms: None,
                            acknowledged_by: None,
                            detail: format!(
                                "观察窗口异常: mutation {} 激活后风控拒绝率或回滚率超阈值",
                                mutation.proposal_id
                            ),
                        };
                        state
                            .alert_firings
                            .write()
                            .await
                            .insert(alert_id, firing);
                    }
                }
            }
        }
    }
}

fn collect_compile_diagnostics(runtime_config: &FrontendRuntimeConfig) -> Vec<CompileDiagnostic> {
    let data_nodes = runtime_config
        .data_sources
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<BTreeMap<_, _>>();

    runtime_config
        .intent_generators
        .iter()
        .flat_map(|intent| {
            let Some(required_warmup) = required_warmup_bars_for_intent(intent) else {
                return Vec::new();
            };
            intent
                .input_refs
                .iter()
                .filter_map(|input_ref| {
                    let data_node = data_nodes.get(input_ref.source_id.as_str())?;
                    if data_node.module_key != "builtin.data.kline" {
                        return None;
                    }

                    let configured_window = json_usize_field(&data_node.config, "window_size")?;
                    if configured_window >= required_warmup {
                        return None;
                    }

                    Some(CompileDiagnostic {
                        code: "QPWARM001".to_string(),
                        severity: CompileDiagnosticSeverity::Warning,
                        message: format!(
                            "intent `{}` needs at least {} bars, but data node `{}` is configured with {}",
                            intent.name, required_warmup, data_node.name, configured_window
                        ),
                        span_label: None,
                        target: Some(node_field_target(
                            &data_node.id,
                            "window_size",
                            format!("{}.window_size", data_node.name),
                        )),
                        hint: Some(format!(
                            "Increase `{}` window_size to >= {} for `{}`.",
                            data_node.name, required_warmup, intent.name
                        )),
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn collect_runtime_compile_contract_diagnostics(
    runtime_config: &FrontendRuntimeConfig,
) -> Vec<CompileDiagnostic> {
    runtime_config
        .intent_generators
        .iter()
        .flat_map(runtime_intent_contract_diagnostics)
        .collect()
}

fn runtime_intent_contract_diagnostics(intent: &FrontendIntentConfig) -> Vec<CompileDiagnostic> {
    if intent.module_key != "builtin.intent.spread_observer" {
        return Vec::new();
    }
    if !spread_threshold_metadata_present(&intent.config) {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    let label = intent.name.clone();

    let spread_output_code = json_number_field(&intent.config, "spread_output_code")
        .unwrap_or_default()
        .round() as i64;
    if spread_output_code != 1 {
        diagnostics.push(CompileDiagnostic {
            code: "QPSPREAD001".to_string(),
            severity: CompileDiagnosticSeverity::Error,
            message: "spread threshold currently supports only bps output".to_string(),
            span_label: Some("spread threshold output".to_string()),
            target: Some(node_field_target(
                &intent.id,
                "spread_output_code",
                label.clone(),
            )),
            hint: Some(
                "Set `spread_output_code` to the bps code (`1`) for the current graph/runtime spread threshold slice.".to_string(),
            ),
        });
    }

    let tolerance_ms = json_number_field(&intent.config, "max_time_diff_ms").unwrap_or_default();
    if tolerance_ms <= 0.0 {
        diagnostics.push(CompileDiagnostic {
            code: "QPSPREAD002".to_string(),
            severity: CompileDiagnosticSeverity::Error,
            message:
                "spread threshold currently requires a positive max_time_diff_ms tolerance"
                    .to_string(),
            span_label: Some("spread threshold tolerance".to_string()),
            target: Some(node_field_target(
                &intent.id,
                "max_time_diff_ms",
                label.clone(),
            )),
            hint: Some(
                "Set `max_time_diff_ms` to a positive value for the current graph/runtime spread threshold slice.".to_string(),
            ),
        });
    }

    let shape_code = json_number_field(&intent.config, "comparison_shape_code")
        .map(|value| value.round() as i64);
    let op_code =
        json_number_field(&intent.config, "comparison_op_code").map(|value| value.round() as i64);
    let threshold = json_number_field(&intent.config, "comparison_threshold");
    let shape_valid = matches!(shape_code, Some(1));
    let op_valid = matches!(op_code, Some(2 | 3));
    let threshold_valid = threshold.is_some();
    if !shape_valid || !op_valid || !threshold_valid {
        let field = if !shape_valid {
            "comparison_shape_code"
        } else if !op_valid {
            "comparison_op_code"
        } else {
            "comparison_threshold"
        };
        diagnostics.push(CompileDiagnostic {
            code: "QPSPREAD003".to_string(),
            severity: CompileDiagnosticSeverity::Error,
            message: "spread threshold currently supports only one-sided buy shapes using `>` or `>=` with an explicit numeric threshold".to_string(),
            span_label: Some("spread threshold shape".to_string()),
            target: Some(node_field_target(&intent.id, field, label)),
            hint: Some(
                "Provide `comparison_shape_code=1`, `comparison_op_code=2 or 3`, and a numeric `comparison_threshold`.".to_string(),
            ),
        });
    }

    diagnostics
}

fn spread_threshold_metadata_present(config: &Value) -> bool {
    [
        "comparison_shape_code",
        "comparison_op_code",
        "comparison_threshold",
    ]
    .iter()
    .any(|field| config.get(*field).is_some())
}

fn required_warmup_bars_for_intent(intent: &FrontendIntentConfig) -> Option<usize> {
    match intent.module_key.as_str() {
        "builtin.intent.double_ma" => Some(
            json_usize_field(&intent.config, "fast_period")
                .unwrap_or_default()
                .max(json_usize_field(&intent.config, "slow_period").unwrap_or_default()),
        ),
        "builtin.intent.ma_deviation" => Some(
            json_usize_field(&intent.config, "lookback")
                .unwrap_or_default()
                .max(json_usize_field(&intent.config, "baseline_period").unwrap_or_default()),
        ),
        "builtin.intent.rsi" => json_usize_field(&intent.config, "period"),
        "builtin.intent.macd" => Some(
            json_usize_field(&intent.config, "slow_period").unwrap_or_default()
                + json_usize_field(&intent.config, "signal_period").unwrap_or_default(),
        ),
        "builtin.intent.momentum" => json_usize_field(&intent.config, "lookback"),
        "builtin.intent.zscore" => json_usize_field(&intent.config, "window"),
        "builtin.intent.spread_observer" => json_usize_field(&intent.config, "window_size"),
        _ => None,
    }
}

fn json_number_field(value: &Value, key: &str) -> Option<f64> {
    let field = value.get(key)?;
    field
        .as_f64()
        .or_else(|| field.as_u64().map(|item| item as f64))
        .or_else(|| field.as_i64().map(|item| item as f64))
}

fn json_usize_field(value: &Value, key: &str) -> Option<usize> {
    json_number_field(value, key).map(|value| value.max(0.0).round() as usize)
}

fn node_field_target(
    node_id: impl Into<String>,
    field: impl Into<String>,
    label: impl Into<String>,
) -> CompileDiagnosticTarget {
    CompileDiagnosticTarget {
        scope: CompileDiagnosticTargetScope::Node,
        node_id: Some(node_id.into()),
        edge_id: None,
        field: Some(field.into()),
        label: Some(label.into()),
    }
}

#[derive(Debug, Clone)]
struct AuthoringSourceLine {
    line_number: usize,
    raw: String,
    header_kind: Option<QuantScriptAuthoringSectionKind>,
    inferred_kind: Option<QuantScriptAuthoringSectionKind>,
}

#[derive(Debug, Default, Clone)]
struct AuthoringSemanticSymbolIndex {
    data_defined: BTreeSet<String>,
    agent_defined: BTreeSet<String>,
    known_callables: BTreeSet<String>,
}

fn build_formal_quantscript_strategy_metadata(
    source: &str,
    module: &FormalScriptModule,
    resolved: &FormalResolveResult,
    lowering_context: &FormalLoweringContext,
) -> anyhow::Result<BTreeMap<String, Value>> {
    let authoring_view =
        build_quantscript_authoring_view(source, module, resolved, lowering_context)?;
    Ok(BTreeMap::from([(
        "quantscript_authoring_view".to_string(),
        serde_json::to_value(authoring_view)
            .context("序列化 QuantScript 编写视图失败")?,
    )]))
}

fn build_quantscript_authoring_view(
    source: &str,
    module: &FormalScriptModule,
    resolved: &FormalResolveResult,
    lowering_context: &FormalLoweringContext,
) -> anyhow::Result<QuantScriptAuthoringView> {
    let source_hash = format!(
        "sha256:{}",
        canonical_json_sha256_digest(&source)
            .context("计算形式化源码哈希失败")?
            .value
    );
    let symbol_index = collect_authoring_semantic_symbol_index(module, resolved);
    let sections = build_authoring_sections(source, &symbol_index);

    let edges = build_authoring_edges(&sections);
    let pool_pipeline = extract_formal_instrument_pool_spec(module, lowering_context)
        .ok()
        .flatten()
        .map(|instrument_pool| build_authoring_pool_pipeline(&instrument_pool, &sections));

    Ok(QuantScriptAuthoringView {
        kind: "quantscript_authoring_view".to_string(),
        source_hash,
        source_order: vec![
            QuantScriptAuthoringSectionKind::Risk,
            QuantScriptAuthoringSectionKind::Execution,
            QuantScriptAuthoringSectionKind::Data,
            QuantScriptAuthoringSectionKind::Intent,
            QuantScriptAuthoringSectionKind::Agent,
        ],
        pipeline_order: vec![
            QuantScriptAuthoringSectionKind::Data,
            QuantScriptAuthoringSectionKind::Intent,
            QuantScriptAuthoringSectionKind::Agent,
            QuantScriptAuthoringSectionKind::Risk,
            QuantScriptAuthoringSectionKind::Execution,
        ],
        sections,
        edges,
        pool_pipeline,
    })
}

fn build_authoring_sections(
    source: &str,
    symbol_index: &AuthoringSemanticSymbolIndex,
) -> Vec<QuantScriptAuthoringSection> {
    let lines = collect_strategy_source_lines(source, symbol_index);
    if lines.is_empty() {
        return Vec::new();
    }

    let mut sections = if lines.iter().any(|line| line.header_kind.is_some()) {
        build_authoring_sections_from_headers(&lines, symbol_index)
    } else {
        build_authoring_sections_from_inferred_lines(&lines, symbol_index)
    };
    let mut counts = BTreeMap::<QuantScriptAuthoringSectionKind, usize>::new();
    for section in &mut sections {
        let count = counts.entry(section.effective_kind).or_default();
        *count += 1;
        section.id = format!(
            "section:{}:{}",
            authoring_kind_code(section.effective_kind),
            *count
        );
    }
    sections
}

fn collect_strategy_source_lines(
    source: &str,
    symbol_index: &AuthoringSemanticSymbolIndex,
) -> Vec<AuthoringSourceLine> {
    let mut in_strategy = false;
    let mut brace_depth = 0usize;
    let mut lines = Vec::new();

    for (index, raw_line) in source.lines().enumerate() {
        let raw_trimmed = raw_line.trim();
        let code_trimmed = strip_quantscript_comment(raw_line).trim();
        if !in_strategy {
            if code_trimmed.starts_with("fn strategy") {
                in_strategy = true;
                brace_depth = brace_delta(code_trimmed).max(0) as usize;
            }
            continue;
        }

        let delta = brace_delta(code_trimmed);
        let next_depth = if delta.is_negative() {
            brace_depth.saturating_sub(delta.unsigned_abs() as usize)
        } else {
            brace_depth.saturating_add(delta as usize)
        };
        let is_final_closing = brace_depth == 1 && next_depth == 0 && code_trimmed == "}";
        if !is_final_closing {
            let header_kind = authoring_section_header_kind(raw_trimmed);
            if header_kind.is_some() || !code_trimmed.is_empty() {
                lines.push(AuthoringSourceLine {
                    line_number: index + 1,
                    raw: raw_line.to_string(),
                    header_kind,
                    inferred_kind: if header_kind.is_some() {
                        None
                    } else {
                        infer_authoring_line_kind(code_trimmed, symbol_index)
                    },
                });
            }
        }
        brace_depth = next_depth;
        if brace_depth == 0 {
            break;
        }
    }

    lines
}

fn build_authoring_sections_from_headers(
    lines: &[AuthoringSourceLine],
    symbol_index: &AuthoringSemanticSymbolIndex,
) -> Vec<QuantScriptAuthoringSection> {
    let header_positions = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| line.header_kind.map(|kind| (index, kind)))
        .collect::<Vec<_>>();
    let mut sections = Vec::new();

    for (pos, (start_index, declared_kind)) in header_positions.iter().enumerate() {
        let end_index = header_positions
            .get(pos + 1)
            .map(|(next_index, _)| next_index.saturating_sub(1))
            .unwrap_or(lines.len().saturating_sub(1));
        if *start_index > end_index {
            continue;
        }
        sections.push(build_authoring_section(
            &lines[*start_index..=end_index],
            *declared_kind,
            QuantScriptAuthoringSectionOrigin::Authored,
            symbol_index,
        ));
    }

    sections
}

fn build_authoring_sections_from_inferred_lines(
    lines: &[AuthoringSourceLine],
    symbol_index: &AuthoringSemanticSymbolIndex,
) -> Vec<QuantScriptAuthoringSection> {
    let mut sections = Vec::new();
    let mut start = None::<usize>;
    let mut current_kind = None::<QuantScriptAuthoringSectionKind>;

    for (index, line) in lines.iter().enumerate() {
        let Some(kind) = line.inferred_kind else {
            if let (Some(start_index), Some(kind)) = (start, current_kind) {
                sections.push(build_authoring_section(
                    &lines[start_index..index],
                    kind,
                    QuantScriptAuthoringSectionOrigin::Hybrid,
                    symbol_index,
                ));
                start = None;
                current_kind = None;
            }
            continue;
        };
        match current_kind {
            Some(existing_kind) if existing_kind == kind => {}
            Some(existing_kind) => {
                if let Some(start_index) = start {
                    sections.push(build_authoring_section(
                        &lines[start_index..index],
                        existing_kind,
                        QuantScriptAuthoringSectionOrigin::Hybrid,
                        symbol_index,
                    ));
                }
                start = Some(index);
                current_kind = Some(kind);
            }
            None => {
                start = Some(index);
                current_kind = Some(kind);
            }
        }
    }

    if let (Some(start_index), Some(kind)) = (start, current_kind) {
        sections.push(build_authoring_section(
            &lines[start_index..],
            kind,
            QuantScriptAuthoringSectionOrigin::Hybrid,
            symbol_index,
        ));
    }

    sections
}

fn build_authoring_section(
    lines: &[AuthoringSourceLine],
    declared_kind: QuantScriptAuthoringSectionKind,
    origin: QuantScriptAuthoringSectionOrigin,
    symbol_index: &AuthoringSemanticSymbolIndex,
) -> QuantScriptAuthoringSection {
    let effective_kind = effective_authoring_kind(lines, declared_kind);
    let status = if matches!(effective_kind, QuantScriptAuthoringSectionKind::Mixed) {
        QuantScriptAuthoringSectionStatus::Mismatch
    } else if declared_kind == effective_kind
        || matches!(effective_kind, QuantScriptAuthoringSectionKind::Unknown)
    {
        if matches!(origin, QuantScriptAuthoringSectionOrigin::Hybrid)
            && lines.iter().all(|line| line.inferred_kind.is_none())
        {
            QuantScriptAuthoringSectionStatus::Partial
        } else {
            QuantScriptAuthoringSectionStatus::Ok
        }
    } else {
        QuantScriptAuthoringSectionStatus::Mismatch
    };
    let snippet = lines
        .iter()
        .map(|line| line.raw.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let symbols_defined = extract_defined_symbols(&snippet);
    let symbols_used = extract_used_symbols(&snippet, &symbols_defined, symbol_index);

    QuantScriptAuthoringSection {
        id: String::new(),
        declared_kind,
        effective_kind,
        origin,
        status,
        start_line: lines
            .first()
            .map(|line| line.line_number)
            .unwrap_or_default(),
        end_line: lines
            .last()
            .map(|line| line.line_number)
            .unwrap_or_default(),
        snippet,
        symbols_defined,
        symbols_used,
    }
}

fn build_authoring_edges(
    sections: &[QuantScriptAuthoringSection],
) -> Vec<QuantScriptAuthoringEdge> {
    let mut by_kind =
        BTreeMap::<QuantScriptAuthoringSectionKind, &QuantScriptAuthoringSection>::new();
    for section in sections {
        by_kind.entry(section.effective_kind).or_insert(section);
    }
    let mut edges = Vec::new();
    if let (Some(data), Some(intent)) = (
        by_kind.get(&QuantScriptAuthoringSectionKind::Data),
        by_kind.get(&QuantScriptAuthoringSectionKind::Intent),
    ) {
        edges.push(QuantScriptAuthoringEdge {
            from: data.id.clone(),
            to: intent.id.clone(),
            relation: QuantScriptAuthoringEdgeRelation::Dataflow,
            reason: "intent_reads_data".to_string(),
        });
    }
    if let (Some(intent), Some(agent)) = (
        by_kind.get(&QuantScriptAuthoringSectionKind::Intent),
        by_kind.get(&QuantScriptAuthoringSectionKind::Agent),
    ) {
        edges.push(QuantScriptAuthoringEdge {
            from: intent.id.clone(),
            to: agent.id.clone(),
            relation: QuantScriptAuthoringEdgeRelation::DecisionFlow,
            reason: "agent_uses_intent".to_string(),
        });
    }
    if let (Some(agent), Some(risk)) = (
        by_kind.get(&QuantScriptAuthoringSectionKind::Agent),
        by_kind.get(&QuantScriptAuthoringSectionKind::Risk),
    ) {
        edges.push(QuantScriptAuthoringEdge {
            from: agent.id.clone(),
            to: risk.id.clone(),
            relation: QuantScriptAuthoringEdgeRelation::PolicyAttachment,
            reason: "risk_governs_agent".to_string(),
        });
    }
    if let (Some(agent), Some(execution)) = (
        by_kind.get(&QuantScriptAuthoringSectionKind::Agent),
        by_kind.get(&QuantScriptAuthoringSectionKind::Execution),
    ) {
        edges.push(QuantScriptAuthoringEdge {
            from: agent.id.clone(),
            to: execution.id.clone(),
            relation: QuantScriptAuthoringEdgeRelation::ExecutionAttachment,
            reason: "execution_applies_to_agent".to_string(),
        });
    }
    edges
}

fn build_authoring_pool_pipeline(
    instrument_pool: &InstrumentPoolSpec,
    sections: &[QuantScriptAuthoringSection],
) -> QuantScriptAuthoringPoolPipeline {
    let order = vec![
        QuantScriptAuthoringPoolStageKind::Source,
        QuantScriptAuthoringPoolStageKind::Eligibility,
        QuantScriptAuthoringPoolStageKind::Features,
        QuantScriptAuthoringPoolStageKind::Selection,
        QuantScriptAuthoringPoolStageKind::Weighting,
        QuantScriptAuthoringPoolStageKind::Rebalance,
    ];
    let stages = order
        .iter()
        .copied()
        .map(|kind| build_authoring_pool_stage(kind, instrument_pool, sections))
        .collect();
    QuantScriptAuthoringPoolPipeline { order, stages }
}

fn build_authoring_pool_stage(
    kind: QuantScriptAuthoringPoolStageKind,
    instrument_pool: &InstrumentPoolSpec,
    sections: &[QuantScriptAuthoringSection],
) -> QuantScriptAuthoringPoolStage {
    let related_section_ids = authoring_pool_related_section_ids(kind, sections);
    match kind {
        QuantScriptAuthoringPoolStageKind::Source => {
            let (status, summary, details) = match &instrument_pool.source {
                InstrumentPoolSourceSpec::ExplicitSymbols => (
                    QuantScriptAuthoringPoolStageStatus::Present,
                    "explicit symbol source".to_string(),
                    vec!["source=explicit_symbols".to_string()],
                ),
                InstrumentPoolSourceSpec::Universe {
                    exchange,
                    market,
                    quote,
                } => {
                    let mut details = Vec::new();
                    if let Some(value) = exchange {
                        details.push(format!("exchange={value}"));
                    }
                    if let Some(value) = market {
                        details.push(format!("market={value}"));
                    }
                    if let Some(value) = quote {
                        details.push(format!("quote={value}"));
                    }
                    let summary = if details.is_empty() {
                        "universe source".to_string()
                    } else {
                        format!("universe({})", details.join(", "))
                    };
                    (
                        QuantScriptAuthoringPoolStageStatus::Present,
                        summary,
                        details,
                    )
                }
            };
            QuantScriptAuthoringPoolStage {
                kind,
                status,
                summary,
                details,
                related_section_ids,
            }
        }
        QuantScriptAuthoringPoolStageKind::Eligibility => {
            let details = instrument_pool
                .eligibility_rules
                .iter()
                .map(|rule| {
                    format!(
                        "{} {} {}",
                        rule.field,
                        rule.op,
                        authoring_pool_value_label(&rule.value)
                    )
                })
                .collect::<Vec<_>>();
            QuantScriptAuthoringPoolStage {
                kind,
                status: if details.is_empty() {
                    QuantScriptAuthoringPoolStageStatus::Empty
                } else {
                    QuantScriptAuthoringPoolStageStatus::Present
                },
                summary: if details.is_empty() {
                    "no eligibility rules".to_string()
                } else {
                    format!("{} eligibility rule(s)", details.len())
                },
                details,
                related_section_ids,
            }
        }
        QuantScriptAuthoringPoolStageKind::Features => {
            let details = instrument_pool
                .feature_defs
                .iter()
                .map(|feature| format!("{} ({})", feature.name, feature.kind))
                .collect::<Vec<_>>();
            QuantScriptAuthoringPoolStage {
                kind,
                status: if details.is_empty() {
                    QuantScriptAuthoringPoolStageStatus::Empty
                } else {
                    QuantScriptAuthoringPoolStageStatus::Present
                },
                summary: if details.is_empty() {
                    "no derived feature defs yet".to_string()
                } else {
                    format!("{} feature def(s)", details.len())
                },
                details,
                related_section_ids,
            }
        }
        QuantScriptAuthoringPoolStageKind::Selection => {
            let (status, summary, details) = match &instrument_pool.selection_rule {
                Some(rule) => {
                    let mut details = vec![format!("kind={}", rule.kind)];
                    if let Some(key) = &rule.key {
                        details.push(format!("key={}", authoring_pool_selection_key_label(key)));
                    }
                    if let Some(order) = &rule.order {
                        details.push(format!("order={order}"));
                    }
                    if let Some(count) = rule.count {
                        details.push(format!("count={count}"));
                    }
                    (
                        QuantScriptAuthoringPoolStageStatus::Present,
                        authoring_pool_selection_summary(rule),
                        details,
                    )
                }
                None => (
                    QuantScriptAuthoringPoolStageStatus::Empty,
                    "no selection rule".to_string(),
                    Vec::new(),
                ),
            };
            QuantScriptAuthoringPoolStage {
                kind,
                status,
                summary,
                details,
                related_section_ids,
            }
        }
        QuantScriptAuthoringPoolStageKind::Weighting => {
            let (status, summary, details) = match &instrument_pool.weighting_rule {
                Some(rule) => {
                    let mut details = vec![format!("kind={}", rule.kind)];
                    if let Some(method) = &rule.method {
                        details.push(format!("method={method}"));
                    }
                    if let Some(normalize) = &rule.normalize {
                        details.push(format!("normalize={normalize}"));
                    }
                    if !rule.target_weights.is_empty() {
                        details.push(format!(
                            "target_weights=[{}]",
                            rule.target_weights
                                .iter()
                                .map(authoring_pool_number_label)
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                    (
                        QuantScriptAuthoringPoolStageStatus::Present,
                        authoring_pool_weighting_summary(rule),
                        details,
                    )
                }
                None => (
                    QuantScriptAuthoringPoolStageStatus::Empty,
                    "no weighting rule".to_string(),
                    Vec::new(),
                ),
            };
            QuantScriptAuthoringPoolStage {
                kind,
                status,
                summary,
                details,
                related_section_ids,
            }
        }
        QuantScriptAuthoringPoolStageKind::Rebalance => {
            let (status, summary, details) = match &instrument_pool.rebalance_rule {
                Some(rule) => {
                    let details = vec![format!(
                        "every={}",
                        rule.every
                            .as_ref()
                            .map(authoring_rebalance_schedule_label)
                            .unwrap_or("unspecified")
                    )];
                    (
                        QuantScriptAuthoringPoolStageStatus::Present,
                        format!(
                            "rebalance {}",
                            rule.every
                                .as_ref()
                                .map(authoring_rebalance_schedule_label)
                                .unwrap_or("unspecified")
                        ),
                        details,
                    )
                }
                None => (
                    QuantScriptAuthoringPoolStageStatus::Empty,
                    "no rebalance cadence".to_string(),
                    Vec::new(),
                ),
            };
            QuantScriptAuthoringPoolStage {
                kind,
                status,
                summary,
                details,
                related_section_ids,
            }
        }
    }
}

fn authoring_pool_related_section_ids(
    kind: QuantScriptAuthoringPoolStageKind,
    sections: &[QuantScriptAuthoringSection],
) -> Vec<String> {
    sections
        .iter()
        .filter(|section| match kind {
            QuantScriptAuthoringPoolStageKind::Source
            | QuantScriptAuthoringPoolStageKind::Eligibility
            | QuantScriptAuthoringPoolStageKind::Features
            | QuantScriptAuthoringPoolStageKind::Selection => {
                matches!(
                    section.effective_kind,
                    QuantScriptAuthoringSectionKind::Data | QuantScriptAuthoringSectionKind::Mixed
                ) || matches!(section.declared_kind, QuantScriptAuthoringSectionKind::Data)
            }
            QuantScriptAuthoringPoolStageKind::Weighting
            | QuantScriptAuthoringPoolStageKind::Rebalance => {
                matches!(
                    section.effective_kind,
                    QuantScriptAuthoringSectionKind::Agent | QuantScriptAuthoringSectionKind::Mixed
                ) || matches!(
                    section.declared_kind,
                    QuantScriptAuthoringSectionKind::Agent
                )
            }
        })
        .map(|section| section.id.clone())
        .collect()
}

fn authoring_pool_value_label(value: &InstrumentPoolValue) -> String {
    match value {
        InstrumentPoolValue::String(value) => value.clone(),
        InstrumentPoolValue::Number(value) => authoring_pool_number_label(value),
    }
}

fn authoring_pool_number_label(value: &f64) -> String {
    let mut text = value.to_string();
    if text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    text
}

fn authoring_pool_selection_key_label(key: &InstrumentPoolSelectionKey) -> String {
    match key {
        InstrumentPoolSelectionKey::Symbol => "symbol".to_string(),
        InstrumentPoolSelectionKey::MetadataField(field) => format!("metadata.{field}"),
        InstrumentPoolSelectionKey::Feature(feature) => format!("feature.{feature}"),
    }
}

fn authoring_pool_selection_summary(rule: &quantscript::InstrumentPoolSelectionRule) -> String {
    let mut summary = rule.kind.clone();
    if let Some(key) = &rule.key {
        summary.push_str(&format!(" by {}", authoring_pool_selection_key_label(key)));
    }
    if let Some(order) = &rule.order {
        summary.push_str(&format!(" {order}"));
    }
    if let Some(count) = rule.count {
        summary.push_str(&format!(" top {count}"));
    }
    summary
}

fn authoring_pool_weighting_summary(rule: &quantscript::InstrumentPoolWeightingRule) -> String {
    let mut summary = rule.kind.clone();
    if let Some(method) = &rule.method {
        summary.push_str(&format!(" ({method})"));
    }
    if let Some(normalize) = &rule.normalize {
        summary.push_str(&format!(" normalize={normalize}"));
    }
    if !rule.target_weights.is_empty() {
        summary.push_str(&format!(" {} target weight(s)", rule.target_weights.len()));
    }
    summary
}

fn authoring_rebalance_schedule_label(schedule: &qrpc_core::RebalanceSchedule) -> &'static str {
    match schedule {
        qrpc_core::RebalanceSchedule::Every1d => "1d",
        qrpc_core::RebalanceSchedule::Weekly => "weekly",
        qrpc_core::RebalanceSchedule::EverySlow => "slow",
    }
}

fn effective_authoring_kind(
    lines: &[AuthoringSourceLine],
    declared_kind: QuantScriptAuthoringSectionKind,
) -> QuantScriptAuthoringSectionKind {
    let mut seen = lines
        .iter()
        .filter_map(|line| line.inferred_kind)
        .collect::<BTreeSet<_>>();
    if seen.is_empty() {
        return declared_kind;
    }
    if seen.len() == 1 {
        return seen.pop_first().unwrap_or(declared_kind);
    }
    QuantScriptAuthoringSectionKind::Mixed
}

fn authoring_section_header_kind(line: &str) -> Option<QuantScriptAuthoringSectionKind> {
    let lowered = line.trim().to_ascii_lowercase();
    match lowered.as_str() {
        "# risk" => Some(QuantScriptAuthoringSectionKind::Risk),
        "# execution" => Some(QuantScriptAuthoringSectionKind::Execution),
        "# data" => Some(QuantScriptAuthoringSectionKind::Data),
        "# intent" => Some(QuantScriptAuthoringSectionKind::Intent),
        "# agent" => Some(QuantScriptAuthoringSectionKind::Agent),
        _ => None,
    }
}

fn infer_authoring_line_kind(
    line: &str,
    symbol_index: &AuthoringSemanticSymbolIndex,
) -> Option<QuantScriptAuthoringSectionKind> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed == "{" || trimmed == "}" {
        return None;
    }
    if trimmed.contains("risk.profile(") {
        return Some(QuantScriptAuthoringSectionKind::Risk);
    }
    if trimmed.contains("execution.profile(") {
        return Some(QuantScriptAuthoringSectionKind::Execution);
    }
    if contains_any_call_name(
        trimmed,
        &[
            "rebalance",
            "equal_weight",
            "fixed_weights",
            "rank_weight",
            "score_weight",
        ],
    ) {
        return Some(QuantScriptAuthoringSectionKind::Agent);
    }
    if trimmed.contains("emit Intent(")
        || trimmed.starts_with("if ")
        || trimmed.starts_with("else if ")
    {
        return Some(QuantScriptAuthoringSectionKind::Intent);
    }
    if contains_any_call_name(
        trimmed,
        &[
            "fetch",
            "get_data",
            "sma",
            "ema",
            "rsi",
            "macd",
            "momentum",
            "zscore",
            "align_asof",
            "spread",
        ],
    ) {
        return Some(QuantScriptAuthoringSectionKind::Data);
    }
    if let Some(bound) = let_binding_name(trimmed) {
        if symbol_index.data_defined.contains(&bound) {
            return Some(QuantScriptAuthoringSectionKind::Data);
        }
        if symbol_index.agent_defined.contains(&bound) {
            return Some(QuantScriptAuthoringSectionKind::Agent);
        }
    }
    if symbol_index
        .agent_defined
        .iter()
        .any(|symbol| line_mentions_identifier(trimmed, symbol))
    {
        return Some(QuantScriptAuthoringSectionKind::Agent);
    }
    None
}

fn collect_authoring_semantic_symbol_index(
    module: &FormalScriptModule,
    resolved: &FormalResolveResult,
) -> AuthoringSemanticSymbolIndex {
    let mut index = AuthoringSemanticSymbolIndex {
        known_callables: resolved
            .callables
            .keys()
            .cloned()
            .chain(resolved.functions.keys().cloned())
            .collect(),
        ..Default::default()
    };
    let Some(strategy) = module.items.iter().find_map(|item| match item {
        quantscript::Item::Function(function) if function.name == "strategy" => Some(function),
        _ => None,
    }) else {
        return index;
    };

    collect_stmt_symbol_kinds(&strategy.body, &mut index);
    index
}

fn collect_stmt_symbol_kinds(stmts: &[FormalStmt], index: &mut AuthoringSemanticSymbolIndex) {
    for stmt in stmts {
        match stmt {
            FormalStmt::Let { pattern, value, .. } => {
                if expr_contains_data_call(value) {
                    index.data_defined.insert(pattern.clone());
                } else if expr_contains_agent_call(value) {
                    index.agent_defined.insert(pattern.clone());
                }
                collect_expr_symbol_kinds(value, index);
            }
            FormalStmt::Return(Some(value)) | FormalStmt::Expr(value) => {
                collect_expr_symbol_kinds(value, index);
            }
            FormalStmt::Return(None) => {}
            FormalStmt::EmitIntent { args } => {
                for arg in args {
                    collect_expr_symbol_kinds(&arg.value, index);
                }
            }
            FormalStmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                collect_expr_symbol_kinds(condition, index);
                collect_stmt_symbol_kinds(then_branch, index);
                for (branch_condition, branch) in else_if_branches {
                    collect_expr_symbol_kinds(branch_condition, index);
                    collect_stmt_symbol_kinds(branch, index);
                }
                if let Some(branch) = else_branch {
                    collect_stmt_symbol_kinds(branch, index);
                }
            }
            FormalStmt::For { iterable, body, .. } => {
                collect_expr_symbol_kinds(iterable, index);
                collect_stmt_symbol_kinds(body, index);
            }
            FormalStmt::While { condition, body } => {
                collect_expr_symbol_kinds(condition, index);
                collect_stmt_symbol_kinds(body, index);
            }
            FormalStmt::Match { expr, arms } => {
                collect_expr_symbol_kinds(expr, index);
                for arm in arms {
                    match &arm.body {
                        quantscript::MatchArmBody::Statement(stmt) => {
                            collect_stmt_symbol_kinds(std::slice::from_ref(stmt.as_ref()), index);
                        }
                        quantscript::MatchArmBody::Expr(expr) => {
                            collect_expr_symbol_kinds(expr, index);
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::only_used_in_recursion)]
fn collect_expr_symbol_kinds(expr: &FormalExpr, index: &mut AuthoringSemanticSymbolIndex) {
    match expr {
        FormalExpr::Call { callee, args } => {
            collect_expr_symbol_kinds(callee, index);
            for arg in args {
                collect_expr_symbol_kinds(&arg.value, index);
            }
        }
        FormalExpr::Member { object, .. } | FormalExpr::Unary { expr: object, .. } => {
            collect_expr_symbol_kinds(object, index);
        }
        FormalExpr::Index {
            object,
            index: value,
        } => {
            collect_expr_symbol_kinds(object, index);
            collect_expr_symbol_kinds(value, index);
        }
        FormalExpr::Slice { object, start, end } => {
            collect_expr_symbol_kinds(object, index);
            if let Some(start) = start {
                collect_expr_symbol_kinds(start, index);
            }
            if let Some(end) = end {
                collect_expr_symbol_kinds(end, index);
            }
        }
        FormalExpr::Binary { left, right, .. }
        | FormalExpr::Range {
            start: left,
            end: right,
        } => {
            collect_expr_symbol_kinds(left, index);
            collect_expr_symbol_kinds(right, index);
        }
        FormalExpr::Await(inner) | FormalExpr::Try(inner) => {
            collect_expr_symbol_kinds(inner, index)
        }
        FormalExpr::List(items) => {
            for item in items {
                collect_expr_symbol_kinds(item, index);
            }
        }
        FormalExpr::Raw(_)
        | FormalExpr::Identifier(_)
        | FormalExpr::Number(_)
        | FormalExpr::String(_)
        | FormalExpr::Bool(_) => {}
    }
}

fn expr_contains_data_call(expr: &FormalExpr) -> bool {
    expr_contains_named_call(
        expr,
        &[
            "fetch",
            "get_data",
            "sma",
            "ema",
            "rsi",
            "macd",
            "momentum",
            "zscore",
            "align_asof",
            "spread",
        ],
    )
}

fn expr_contains_agent_call(expr: &FormalExpr) -> bool {
    expr_contains_named_call(
        expr,
        &[
            "symbols",
            "universe",
            "filter",
            "sort_by",
            "top",
            "rebalance",
            "equal_weight",
            "fixed_weights",
            "rank_weight",
            "score_weight",
        ],
    )
}

fn expr_contains_named_call(expr: &FormalExpr, names: &[&str]) -> bool {
    match expr {
        FormalExpr::Call { callee, args } => {
            let current = call_name(callee)
                .map(|name| names.contains(&name))
                .unwrap_or(false);
            current
                || expr_contains_named_call(callee, names)
                || args
                    .iter()
                    .any(|arg| expr_contains_named_call(&arg.value, names))
        }
        FormalExpr::Member { object, .. } | FormalExpr::Unary { expr: object, .. } => {
            expr_contains_named_call(object, names)
        }
        FormalExpr::Index { object, index } => {
            expr_contains_named_call(object, names) || expr_contains_named_call(index, names)
        }
        FormalExpr::Slice { object, start, end } => {
            expr_contains_named_call(object, names)
                || start
                    .as_ref()
                    .is_some_and(|expr| expr_contains_named_call(expr, names))
                || end
                    .as_ref()
                    .is_some_and(|expr| expr_contains_named_call(expr, names))
        }
        FormalExpr::Binary { left, right, .. }
        | FormalExpr::Range {
            start: left,
            end: right,
        } => expr_contains_named_call(left, names) || expr_contains_named_call(right, names),
        FormalExpr::Await(inner) | FormalExpr::Try(inner) => expr_contains_named_call(inner, names),
        FormalExpr::List(items) => items
            .iter()
            .any(|item| expr_contains_named_call(item, names)),
        FormalExpr::Raw(_)
        | FormalExpr::Identifier(_)
        | FormalExpr::Number(_)
        | FormalExpr::String(_)
        | FormalExpr::Bool(_) => false,
    }
}

fn call_name(expr: &FormalExpr) -> Option<&str> {
    match expr {
        FormalExpr::Identifier(name) => Some(name.as_str()),
        FormalExpr::Member { field, .. } => Some(field.as_str()),
        _ => None,
    }
}

fn strip_quantscript_comment(line: &str) -> &str {
    let mut in_string = false;
    let mut prev_escape = false;
    for (idx, ch) in line.char_indices() {
        if ch == '"' && !prev_escape {
            in_string = !in_string;
        }
        if ch == '#' && !in_string {
            return &line[..idx];
        }
        prev_escape = ch == '\\' && !prev_escape;
        if ch != '\\' {
            prev_escape = false;
        }
    }
    line
}

fn brace_delta(line: &str) -> isize {
    let opens = line.matches('{').count() as isize;
    let closes = line.matches('}').count() as isize;
    opens - closes
}

fn contains_any_call_name(line: &str, names: &[&str]) -> bool {
    names.iter().any(|name| line.contains(&format!("{name}(")))
}

fn let_binding_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with("let ") {
        return None;
    }
    let rhs = trimmed.trim_start_matches("let ").trim();
    let rhs = rhs.strip_prefix("mut ").unwrap_or(rhs).trim();
    let binding = rhs.split(['=', ':']).next()?.trim();
    if binding.is_empty() {
        return None;
    }
    Some(binding.to_string())
}

fn extract_defined_symbols(snippet: &str) -> Vec<String> {
    let mut defined = BTreeSet::new();
    for line in snippet.lines() {
        if let Some(symbol) = let_binding_name(strip_quantscript_comment(line)) {
            defined.insert(symbol);
        }
    }
    defined.into_iter().collect()
}

fn extract_used_symbols(
    snippet: &str,
    defined_symbols: &[String],
    symbol_index: &AuthoringSemanticSymbolIndex,
) -> Vec<String> {
    let defined = defined_symbols.iter().cloned().collect::<BTreeSet<_>>();
    let reserved = authoring_reserved_identifiers(symbol_index);
    let mut used = BTreeSet::new();
    for identifier in collect_identifiers(snippet) {
        if defined.contains(&identifier) || reserved.contains(&identifier) {
            continue;
        }
        used.insert(identifier);
    }
    used.into_iter().collect()
}

fn authoring_reserved_identifiers(symbol_index: &AuthoringSemanticSymbolIndex) -> BTreeSet<String> {
    let mut reserved = [
        "fn",
        "let",
        "mut",
        "if",
        "else",
        "for",
        "in",
        "while",
        "match",
        "return",
        "emit",
        "Intent",
        "risk",
        "execution",
        "profile",
        "exchange",
        "interval",
        "lookback",
        "window",
        "instrument",
        "quantity",
        "every",
        "weights",
        "method",
        "normalize",
        "fee_bps",
        "slippage_bps",
        "max_position",
        "max_total_leverage",
        "max_exchange_leverage",
        "min_action_interval_ms",
        "true",
        "false",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    reserved.extend(symbol_index.known_callables.iter().cloned());
    reserved
}

fn collect_identifiers(snippet: &str) -> Vec<String> {
    let mut identifiers = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut prev_escape = false;
    for ch in snippet.chars() {
        if ch == '"' && !prev_escape {
            in_string = !in_string;
            if !current.is_empty() {
                identifiers.push(current.clone());
                current.clear();
            }
            prev_escape = false;
            continue;
        }
        if in_string {
            prev_escape = ch == '\\' && !prev_escape;
            if ch != '\\' {
                prev_escape = false;
            }
            continue;
        }
        if current.is_empty() {
            if ch == '_' || ch.is_ascii_alphabetic() {
                current.push(ch);
            }
        } else if ch == '_' || ch.is_ascii_alphanumeric() {
            current.push(ch);
        } else {
            identifiers.push(current.clone());
            current.clear();
        }
    }
    if !current.is_empty() {
        identifiers.push(current);
    }
    identifiers
}

fn line_mentions_identifier(line: &str, identifier: &str) -> bool {
    collect_identifiers(line)
        .into_iter()
        .any(|candidate| candidate == identifier)
}

fn authoring_kind_code(kind: QuantScriptAuthoringSectionKind) -> &'static str {
    match kind {
        QuantScriptAuthoringSectionKind::Risk => "risk",
        QuantScriptAuthoringSectionKind::Execution => "execution",
        QuantScriptAuthoringSectionKind::Data => "data",
        QuantScriptAuthoringSectionKind::Intent => "intent",
        QuantScriptAuthoringSectionKind::Agent => "agent",
        QuantScriptAuthoringSectionKind::Mixed => "mixed",
        QuantScriptAuthoringSectionKind::Unknown => "unknown",
    }
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod cli_tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn test_storage_base(label: &str) -> PathBuf {
        static TEST_STORAGE_COUNTER: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(0);
        let sequence = TEST_STORAGE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "quantpilot-test-{}-{}-{}-{}",
            label,
            std::process::id(),
            current_time_ms(),
            sequence
        ))
    }

    fn test_app_state() -> AppState {
        let base = test_storage_base("api");
        let graph_dir = base.join("graphs");
        let run_dir = base.join("runs");
        let backtest_dir = base.join("backtests");
        std::fs::create_dir_all(&graph_dir).unwrap();
        std::fs::create_dir_all(&run_dir).unwrap();
        std::fs::create_dir_all(&backtest_dir).unwrap();
        test_app_state_from_dirs(base, graph_dir, run_dir, backtest_dir)
    }

    fn test_app_state_from_dirs(
        base: PathBuf,
        graph_dir: PathBuf,
        run_dir: PathBuf,
        backtest_dir: PathBuf,
    ) -> AppState {
        let mut state = new_app_state(graph_dir, run_dir, backtest_dir);
        state.test_storage_root = Some(Arc::new(TestStorageRoot { path: base }));
        state
    }

    #[test]
    fn defaults_to_server_when_no_cli_args_are_provided() {
        let command = parse_cli_command_from(["quantpilot"] as [&str; 1]).unwrap();
        assert_eq!(command, CliCommand::Serve);
    }

    #[test]
    fn parses_strategy_ir_validate_command() {
        let command = parse_cli_command_from([
            "quantpilot",
            "strategy-ir",
            "validate",
            "config/strategy_ir.v0.example.json",
        ])
        .unwrap();
        assert_eq!(
            command,
            CliCommand::StrategyIrValidate {
                path: PathBuf::from("config/strategy_ir.v0.example.json"),
            }
        );
    }

    #[test]
    fn rejects_unknown_cli_command() {
        let err = parse_cli_command_from(["quantpilot", "unknown"]).unwrap_err();
        assert!(err.to_string().contains("不支持的命令"));
    }

    #[test]
    fn parses_strategy_ir_json_with_utf8_bom() {
        let source = concat!(
            "\u{feff}",
            "{",
            "\"ir_version\":\"strategy_ir/v0\",",
            "\"metadata\":{",
            "\"strategy_id\":\"demo\",",
            "\"name\":\"Demo\",",
            "\"summary\":\"Demo strategy\",",
            "\"source\":{\"source_type\":\"manual_paper_analysis\",\"paper_title\":\"Demo\",\"paper_reference\":null}",
            "},",
            "\"signals\":[{\"signal_id\":\"s1\",\"name\":\"Signal\",\"indicator\":{\"kind\":\"rsi\",\"inputs\":[\"close\"],\"params\":{}}}],",
            "\"logic\":{\"entry_rules\":[{\"rule_id\":\"r1\",\"condition\":\"close > open\",\"action\":\"open_long\"}],\"exit_rules\":[],\"position_sizing\":{\"method\":\"fixed_ratio\",\"value\":0.1,\"unit\":\"portfolio_ratio\"},\"rebalance_rule\":null},",
            "\"risk_rules\":{\"max_position_ratio\":0.2,\"stop_loss_ratio\":0.02,\"take_profit_ratio\":null,\"max_drawdown_ratio\":null,\"max_trades_per_day\":null,\"notes\":[]},",
            "\"data_requirements\":[{\"data_id\":\"d1\",\"venue\":\"binance\",\"symbol\":\"BTCUSDT\",\"data_type\":\"kline\",\"granularity\":\"1d\",\"lookback\":100,\"fields\":[\"close\"]}],",
            "\"execution\":{\"venue_type\":\"paper\",\"order_type\":\"market\",\"time_in_force\":null,\"slippage_model\":\"fixed_bps\",\"latency_assumption_ms\":null,\"capital_base\":null},",
            "\"gap_annotations\":[],",
            "\"unknowns\":[]",
            "}"
        );
        let strategy_ir = parse_strategy_ir_json(source).unwrap();
        assert_eq!(strategy_ir.metadata.strategy_id, "demo");
    }

    #[test]
    fn capability_response_distinguishes_supported_and_declared_only_indicator_kinds() {
        let response = build_capability_response();

        assert_eq!(response.api_version, CAPABILITY_API_VERSION);
        assert_eq!(response.schema_version, CAPABILITY_SCHEMA_VERSION);
        assert_eq!(response.chain_stages, RUNTIME_CHAIN_STAGES.to_vec());
        assert!(response.schema_hash.starts_with("sha256:"));
        assert_eq!(
            response.strategy_ir.declared_indicator_kinds,
            declared_indicator_kinds().to_vec()
        );
        assert_eq!(
            response.strategy_ir.supported_indicator_kinds,
            supported_indicator_kinds().to_vec()
        );

        let spread = response
            .strategy_ir
            .indicator_support
            .iter()
            .find(|entry| entry.kind == IndicatorKind::Spread)
            .unwrap();
        assert_eq!(spread.status, CapabilitySupportStatus::Supported);
        assert_eq!(spread.reason, None);

        let custom = response
            .strategy_ir
            .indicator_support
            .iter()
            .find(|entry| entry.kind == IndicatorKind::Custom)
            .unwrap();
        assert_eq!(custom.status, CapabilitySupportStatus::Supported);
        assert_eq!(custom.reason, None);

        let ma_cross = response
            .strategy_ir
            .indicator_support
            .iter()
            .find(|entry| entry.kind == IndicatorKind::MaCross)
            .unwrap();
        assert_eq!(ma_cross.status, CapabilitySupportStatus::Supported);
        assert_eq!(ma_cross.reason, None);
    }

    #[test]
    fn compare_execution_assumptions_modules_reports_missing_when_either_side_absent() {
        let left = Some(ExecutionAssumptionsModule {
            summary: super::backtest_artifacts::ExecutionAssumptionsSummary {
                fee_bps: 10.0,
                slippage_bps: 5.0,
                latency_ms: 0,
                sources: None,
            },
            list_tag: backtest_artifacts::ExecutionAssumptionsTag {
                label: "fee=10 slip=5 lat=0".to_string(),
                sources_label: "fee:na slip:na lat:na".to_string(),
            },
        });

        let compared = compare_execution_assumptions_modules(left, None);
        assert_eq!(compared.status, BacktestCompareStatus::Missing);
        assert!(compared.left.is_some());
        assert!(compared.right.is_none());
        assert_eq!(
            compared.fields,
            BacktestExecutionAssumptionsFieldDiffs {
                fee_bps: BacktestExecutionAssumptionsFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                slippage_bps: BacktestExecutionAssumptionsFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                latency_ms: BacktestExecutionAssumptionsFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                sources: BacktestExecutionAssumptionsFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
            }
        );

        let metrics = compare_metrics_summaries(
            Some(qrpc_core::BacktestSummary {
                step_count: 10,
                trade_count: 2,
                total_return_ratio: 0.1,
                max_drawdown_ratio: 0.02,
                final_equity: 110.0,
                net_profit: 10.0,
                turnover_ratio: 0.4,
                average_trade_notional: 50.0,
                fee_drag_ratio: 0.01,
                win_rate: 0.0,
            }),
            None,
        );
        assert_eq!(metrics.status, BacktestCompareStatus::Missing);
        assert_eq!(
            metrics.fields,
            BacktestMetricsFieldDiffs {
                step_count: BacktestMetricsFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                trade_count: BacktestMetricsFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                total_return_ratio: BacktestMetricsFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                max_drawdown_ratio: BacktestMetricsFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                final_equity: BacktestMetricsFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                net_profit: BacktestMetricsFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                turnover_ratio: BacktestMetricsFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                average_trade_notional: BacktestMetricsFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                fee_drag_ratio: BacktestMetricsFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
            }
        );

        let trade_ledger = compare_trade_ledger_summaries(
            Some(backtest_artifacts::TradeLedgerSummary {
                trade_count: 2,
                buy_fill_count: 0,
                sell_fill_count: 0,
                total_fees_paid: 3.0,
                buy_fees_paid: 0.0,
                sell_fees_paid: 0.0,
                total_filled_notional: 1000.0,
                buy_filled_notional: 0.0,
                sell_filled_notional: 0.0,
                average_fill_price: 0.0,
                average_buy_fill_price: None,
                average_sell_fill_price: None,
                average_fee_per_fill: 1.5,
                average_buy_fee: None,
                average_sell_fee: None,
            }),
            None,
        );
        assert_eq!(trade_ledger.status, BacktestCompareStatus::Missing);
        assert_eq!(
            trade_ledger.fields,
            BacktestTradeLedgerFieldDiffs {
                trade_count: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                buy_fill_count: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                sell_fill_count: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                total_fees_paid: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                buy_fees_paid: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                sell_fees_paid: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                total_filled_notional: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                buy_filled_notional: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                sell_filled_notional: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                average_fill_price: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                average_buy_fill_price: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                average_sell_fill_price: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                average_fee_per_fill: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                average_buy_fee: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                average_sell_fee: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
            }
        );
        let equity_curve = compare_equity_curve_points(
            Some(vec![qrpc_core::BacktestEquityPoint {
                ts_ms: 1_700_000_000_000,
                equity: 110.0,
                cash_balance: 90.0,
                net_notional: 20.0,
            }]),
            None,
        );

        let report_bundle =
            build_compare_report_bundle(&compared, &metrics, &trade_ledger, &equity_curve);
        let narrative = build_report_narrative_compare_block(
            &compared,
            &metrics,
            &trade_ledger,
            &equity_curve,
            &report_bundle,
        );
        assert_eq!(narrative.status, BacktestCompareStatus::Missing);
        assert!(narrative.headline.contains("cannot be fully compared"));
        assert_eq!(
            narrative.bullets,
            vec![
                "Execution assumptions: missing.".to_string(),
                "Metrics summary: missing.".to_string(),
                "Trade ledger summary: missing.".to_string(),
                "Equity curve: missing.".to_string(),
            ]
        );
        assert_eq!(
            narrative.highlights,
            vec![
                "Execution assumptions are unavailable on one or both runs.".to_string(),
                "Metrics summary is unavailable on one or both runs.".to_string(),
                "Trade ledger summary is unavailable on one or both runs.".to_string(),
                "Equity curve is unavailable on one or both runs.".to_string(),
            ]
        );
        assert_eq!(
            narrative.source_explanations,
            vec![
                "Fee source is unavailable on one or both runs.".to_string(),
                "Slippage source is unavailable on one or both runs.".to_string(),
                "Latency source is unavailable on one or both runs.".to_string(),
            ]
        );
        assert_eq!(
            narrative.sections,
            vec![
                BacktestReportNarrativeSection {
                    title: "Execution assumptions".to_string(),
                    status: BacktestCompareStatus::Missing,
                    summary: "Execution assumptions are unavailable on one or both runs."
                        .to_string(),
                    lines: vec![
                        "Status: missing.".to_string(),
                        "Left resolved values: fee_bps=10, slippage_bps=5, latency_ms=0."
                            .to_string(),
                        "Right resolved values: missing.".to_string(),
                    ],
                },
                BacktestReportNarrativeSection {
                    title: "Metrics summary".to_string(),
                    status: BacktestCompareStatus::Missing,
                    summary: "Metrics summary is unavailable on one or both runs."
                        .to_string(),
                    lines: vec![
                        "Status: missing.".to_string(),
                        "Performance drilldown: missing.".to_string(),
                        "Activity drilldown: missing.".to_string(),
                        "Cost drilldown: missing.".to_string(),
                        "Left summary: steps=10, trades=2, return=0.1, drawdown=0.02, final_equity=110, net_profit=10, turnover_ratio=0.4, average_trade_notional=50, fee_drag_ratio=0.01.".to_string(),
                        "Right summary: missing.".to_string(),
                    ],
                },
                BacktestReportNarrativeSection {
                    title: "Trade ledger summary".to_string(),
                    status: BacktestCompareStatus::Missing,
                    summary: "Trade ledger summary is unavailable on one or both runs."
                        .to_string(),
                    lines: vec![
                        "Status: missing.".to_string(),
                        "Left summary: trade_count=2, buy_fill_count=0, sell_fill_count=0, total_fees_paid=3, buy_fees_paid=0, sell_fees_paid=0, total_filled_notional=1000, buy_filled_notional=0, sell_filled_notional=0, average_fill_price=0, average_buy_fill_price=na, average_sell_fill_price=na, average_fee_per_fill=1.5, average_buy_fee=na, average_sell_fee=na.".to_string(),
                        "Right summary: missing.".to_string(),
                    ],
                },
                BacktestReportNarrativeSection {
                    title: "Equity curve".to_string(),
                    status: BacktestCompareStatus::Missing,
                    summary: "Equity curve is unavailable on one or both runs.".to_string(),
                    lines: vec![
                        "Status: missing.".to_string(),
                        "Sample drilldown: missing.".to_string(),
                        "Left summary: point_count=1, started_at_ms=1700000000000, ended_at_ms=1700000000000, first_equity=110, final_equity=110, min_equity=110, max_equity=110.".to_string(),
                        "Right summary: missing.".to_string(),
                    ],
                },
            ]
        );
        let compare_report =
            build_compare_report_view(&metrics, &equity_curve, &narrative, &report_bundle);
        assert_eq!(compare_report.status, BacktestCompareStatus::Missing);
        assert_eq!(
            compare_report.overview,
            BacktestCompareReportOverview {
                bullets: vec![
                    "Execution assumptions: missing.".to_string(),
                    "Metrics summary: missing.".to_string(),
                    "Trade ledger summary: missing.".to_string(),
                    "Equity curve: missing.".to_string(),
                ],
                highlights: vec![
                    "Execution assumptions are unavailable on one or both runs.".to_string(),
                    "Metrics summary is unavailable on one or both runs.".to_string(),
                    "Trade ledger summary is unavailable on one or both runs.".to_string(),
                    "Equity curve is unavailable on one or both runs.".to_string(),
                ],
            }
        );
        assert_eq!(
            compare_report
                .modules
                .execution_assumptions
                .source_explanations,
            vec![
                "Fee source is unavailable on one or both runs.".to_string(),
                "Slippage source is unavailable on one or both runs.".to_string(),
                "Latency source is unavailable on one or both runs.".to_string(),
            ]
        );
        assert_eq!(
            compare_report.modules.metrics.drilldown.performance.status,
            BacktestCompareStatus::Missing
        );
        assert_eq!(
            compare_report.modules.equity_curve.status,
            BacktestCompareStatus::Missing
        );
    }

    #[test]
    fn compare_metrics_summaries_reports_field_level_differences() {
        let compared = compare_metrics_summaries(
            Some(qrpc_core::BacktestSummary {
                step_count: 10,
                trade_count: 2,
                total_return_ratio: 0.1,
                max_drawdown_ratio: 0.02,
                final_equity: 110.0,
                net_profit: 10.0,
                turnover_ratio: 0.4,
                average_trade_notional: 50.0,
                fee_drag_ratio: 0.01,
                win_rate: 0.0,
            }),
            Some(qrpc_core::BacktestSummary {
                step_count: 10,
                trade_count: 3,
                total_return_ratio: 0.08,
                max_drawdown_ratio: 0.02,
                final_equity: 108.0,
                net_profit: 8.0,
                turnover_ratio: 0.5,
                average_trade_notional: 40.0,
                fee_drag_ratio: 0.02,
                win_rate: 0.0,
            }),
        );

        assert_eq!(compared.status, BacktestCompareStatus::Different);
        assert_eq!(
            compared.fields,
            BacktestMetricsFieldDiffs {
                step_count: BacktestMetricsFieldDiff {
                    status: BacktestCompareStatus::Same,
                },
                trade_count: BacktestMetricsFieldDiff {
                    status: BacktestCompareStatus::Different,
                },
                total_return_ratio: BacktestMetricsFieldDiff {
                    status: BacktestCompareStatus::Different,
                },
                max_drawdown_ratio: BacktestMetricsFieldDiff {
                    status: BacktestCompareStatus::Same,
                },
                final_equity: BacktestMetricsFieldDiff {
                    status: BacktestCompareStatus::Different,
                },
                net_profit: BacktestMetricsFieldDiff {
                    status: BacktestCompareStatus::Different,
                },
                turnover_ratio: BacktestMetricsFieldDiff {
                    status: BacktestCompareStatus::Different,
                },
                average_trade_notional: BacktestMetricsFieldDiff {
                    status: BacktestCompareStatus::Different,
                },
                fee_drag_ratio: BacktestMetricsFieldDiff {
                    status: BacktestCompareStatus::Different,
                },
            }
        );
        assert_eq!(
            compared.drilldown,
            BacktestMetricsDrilldownCompare {
                performance: BacktestMetricsDrilldownGroupCompare {
                    status: BacktestCompareStatus::Different,
                    fields: vec![
                        BacktestMetricsDrilldownFieldCompare {
                            key: "total_return_ratio".to_string(),
                            status: BacktestCompareStatus::Different,
                            left_value: Some("0.1".to_string()),
                            right_value: Some("0.08".to_string()),
                        },
                        BacktestMetricsDrilldownFieldCompare {
                            key: "net_profit".to_string(),
                            status: BacktestCompareStatus::Different,
                            left_value: Some("10".to_string()),
                            right_value: Some("8".to_string()),
                        },
                        BacktestMetricsDrilldownFieldCompare {
                            key: "final_equity".to_string(),
                            status: BacktestCompareStatus::Different,
                            left_value: Some("110".to_string()),
                            right_value: Some("108".to_string()),
                        },
                        BacktestMetricsDrilldownFieldCompare {
                            key: "max_drawdown_ratio".to_string(),
                            status: BacktestCompareStatus::Same,
                            left_value: Some("0.02".to_string()),
                            right_value: Some("0.02".to_string()),
                        },
                    ],
                },
                activity: BacktestMetricsDrilldownGroupCompare {
                    status: BacktestCompareStatus::Different,
                    fields: vec![
                        BacktestMetricsDrilldownFieldCompare {
                            key: "step_count".to_string(),
                            status: BacktestCompareStatus::Same,
                            left_value: Some("10".to_string()),
                            right_value: Some("10".to_string()),
                        },
                        BacktestMetricsDrilldownFieldCompare {
                            key: "trade_count".to_string(),
                            status: BacktestCompareStatus::Different,
                            left_value: Some("2".to_string()),
                            right_value: Some("3".to_string()),
                        },
                        BacktestMetricsDrilldownFieldCompare {
                            key: "turnover_ratio".to_string(),
                            status: BacktestCompareStatus::Different,
                            left_value: Some("0.4".to_string()),
                            right_value: Some("0.5".to_string()),
                        },
                        BacktestMetricsDrilldownFieldCompare {
                            key: "average_trade_notional".to_string(),
                            status: BacktestCompareStatus::Different,
                            left_value: Some("50".to_string()),
                            right_value: Some("40".to_string()),
                        },
                    ],
                },
                costs: BacktestMetricsDrilldownGroupCompare {
                    status: BacktestCompareStatus::Different,
                    fields: vec![BacktestMetricsDrilldownFieldCompare {
                        key: "fee_drag_ratio".to_string(),
                        status: BacktestCompareStatus::Different,
                        left_value: Some("0.01".to_string()),
                        right_value: Some("0.02".to_string()),
                    }],
                },
            }
        );
    }

    #[test]
    fn compare_trade_ledger_summaries_reports_field_level_differences() {
        let compared = compare_trade_ledger_summaries(
            Some(backtest_artifacts::TradeLedgerSummary {
                trade_count: 2,
                buy_fill_count: 1,
                sell_fill_count: 1,
                total_fees_paid: 3.0,
                buy_fees_paid: 1.0,
                sell_fees_paid: 2.0,
                total_filled_notional: 1000.0,
                buy_filled_notional: 450.0,
                sell_filled_notional: 550.0,
                average_fill_price: 100.0,
                average_buy_fill_price: Some(95.0),
                average_sell_fill_price: Some(105.0),
                average_fee_per_fill: 1.5,
                average_buy_fee: Some(1.0),
                average_sell_fee: Some(2.0),
            }),
            Some(backtest_artifacts::TradeLedgerSummary {
                trade_count: 2,
                buy_fill_count: 2,
                sell_fill_count: 0,
                total_fees_paid: 4.0,
                buy_fees_paid: 4.0,
                sell_fees_paid: 0.0,
                total_filled_notional: 1250.0,
                buy_filled_notional: 1250.0,
                sell_filled_notional: 0.0,
                average_fill_price: 125.0,
                average_buy_fill_price: Some(125.0),
                average_sell_fill_price: None,
                average_fee_per_fill: 2.0,
                average_buy_fee: Some(2.0),
                average_sell_fee: None,
            }),
        );

        assert_eq!(compared.status, BacktestCompareStatus::Different);
        assert_eq!(
            compared.fields,
            BacktestTradeLedgerFieldDiffs {
                trade_count: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Same,
                },
                buy_fill_count: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Different,
                },
                sell_fill_count: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Different,
                },
                total_fees_paid: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Different,
                },
                buy_fees_paid: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Different,
                },
                sell_fees_paid: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Different,
                },
                total_filled_notional: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Different,
                },
                buy_filled_notional: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Different,
                },
                sell_filled_notional: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Different,
                },
                average_fill_price: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Different,
                },
                average_buy_fill_price: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Different,
                },
                average_sell_fill_price: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
                average_fee_per_fill: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Different,
                },
                average_buy_fee: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Different,
                },
                average_sell_fee: BacktestTradeLedgerFieldDiff {
                    status: BacktestCompareStatus::Missing,
                },
            }
        );
    }

    #[test]
    fn capability_response_keeps_legacy_frontend_fields_and_adds_module_support_entries() {
        let response = build_capability_response();

        assert_eq!(
            response.frontend.declared_module_keys,
            DECLARED_FRONTEND_MODULE_KEYS.to_vec()
        );
        assert_eq!(
            response.frontend.supported_module_keys,
            SUPPORTED_FRONTEND_MODULE_KEYS.to_vec()
        );
        assert!(response.frontend.unsupported_module_reasons.is_empty());

        let arbitrage = response
            .frontend
            .module_support
            .iter()
            .find(|entry| entry.module_key == "builtin.agent.arbitrage")
            .unwrap();
        assert_eq!(arbitrage.status, CapabilitySupportStatus::Supported);
        assert_eq!(arbitrage.reason, None);

        let weighted = response
            .frontend
            .module_support
            .iter()
            .find(|entry| entry.module_key == "builtin.agent.weighted")
            .unwrap();
        assert_eq!(weighted.status, CapabilitySupportStatus::Supported);
        assert_eq!(weighted.reason, None);
    }

    #[test]
    fn capability_response_serializes_new_support_sections() {
        let value = serde_json::to_value(build_capability_response()).unwrap();

        assert_eq!(value["api_version"], CAPABILITY_API_VERSION);
        assert!(value["strategy_ir"]["indicator_support"].is_array());
        assert!(value["runtime"]["mode_support"].is_array());
        assert!(value["market_data"]["exchange_support"].is_array());
        assert!(value["frontend"]["module_support"].is_array());
        assert!(value["frontend"]["supported_module_keys"].is_array());
        assert_eq!(
            value["chain_stages"],
            serde_json::json!(RUNTIME_CHAIN_STAGES)
        );
        assert_eq!(
            value["permission_boundary"]["non_execution_order_access"],
            "deny"
        );
        assert_eq!(
            value["versioning"]["parameter_version_policy"],
            "immutable_generation_pointer"
        );
    }

    #[test]
    fn capability_contract_drives_response_hash_and_runtime_governance() {
        let response = build_capability_response();
        let governance = runtime_governance_snapshot(
            &FrontendMetadata {
                graph_id: "graph_hash_contract".to_string(),
                compile_id: "compile_hash_contract".to_string(),
                name: "Hash Contract".to_string(),
                version: "1.2.3".to_string(),
                mode: "paper".to_string(),
            },
            Some("params_hash_contract"),
        );

        assert_eq!(response.schema_hash, current_capability_hash());
        assert_eq!(response.schema_hash, governance.capability_hash);
        assert!(governance.deployment_revision.starts_with("sha256:"));
    }

    #[test]
    fn capability_contract_hash_changes_when_governed_fields_change() {
        let base = build_capability_contract();
        let base_hash = capability_contract_hash(&base);

        let mut changed_stage = base.clone();
        changed_stage.chain_stages.push("settlement");
        assert_ne!(capability_contract_hash(&changed_stage), base_hash);

        let mut changed_runtime_mode = base.clone();
        changed_runtime_mode.runtime_modes.push("live");
        assert_ne!(capability_contract_hash(&changed_runtime_mode), base_hash);

        let mut changed_module = base.clone();
        changed_module
            .supported_module_keys
            .push("builtin.intent.contract_test");
        assert_ne!(capability_contract_hash(&changed_module), base_hash);

        let mut changed_symbol = base.clone();
        changed_symbol.supported_symbols.push("DOGEUSDT");
        assert_ne!(capability_contract_hash(&changed_symbol), base_hash);

        let mut changed_policy = base;
        changed_policy.permission_boundary = CapabilityPermissionBoundarySummary {
            ai_write_policy: AiWritePolicy::Disabled,
            ..changed_policy.permission_boundary
        };
        assert_ne!(capability_contract_hash(&changed_policy), base_hash);
    }

    #[test]
    fn capability_contract_hash_is_canonical_and_order_stable() {
        let mut left = build_capability_contract();
        left.unsupported_module_reasons = BTreeMap::from([
            ("builtin.intent.contract_b", "beta reason"),
            ("builtin.intent.contract_a", "alpha reason"),
        ]);

        let mut right = build_capability_contract();
        right.unsupported_module_reasons = BTreeMap::from([
            ("builtin.intent.contract_a", "alpha reason"),
            ("builtin.intent.contract_b", "beta reason"),
        ]);

        let left_hash = capability_contract_hash(&left);
        assert!(left_hash.starts_with("sha256:"));
        assert_eq!(left_hash, capability_contract_hash(&right));
    }

    #[tokio::test]
    async fn capabilities_endpoint_returns_capability_response_over_router() {
        let graph_app = build_app_router(test_app_state());

        let response = graph_app
            .oneshot(
                Request::builder()
                    .uri("/api/capabilities")
                    .method("GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["api_version"], CAPABILITY_API_VERSION);
        assert!(value["schema_hash"]
            .as_str()
            .unwrap()
            .starts_with("sha256:"));
        assert_eq!(
            value["permission_boundary"]["ai_write_policy"],
            serde_json::json!("proposal_only")
        );
        assert!(value["strategy_ir"]["indicator_support"].is_array());
        assert!(value["frontend"]["module_support"].is_array());
        assert_eq!(
            value["market_data"]["supported_symbols"],
            serde_json::json!(["BTCUSDT", "ETHUSDT", "SOLUSDT"])
        );
        assert_eq!(
            value["frontend"]["unsupported_module_reasons"],
            serde_json::json!({})
        );
    }

    #[test]
    fn capability_fixture_matches_backend_response_snapshot() {
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../frontend/src/test/fixtures/capabilities/backend-capabilities-v1.json"
        ))
        .unwrap();
        let actual = serde_json::to_value(build_capability_response()).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    #[ignore]
    fn export_capability_fixture_json_snapshot() {
        let json = serde_json::to_vec_pretty(&build_capability_response()).unwrap();
        let encoded = json
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>();

        println!("__CAPABILITY_FIXTURE_START__");
        println!("{}", encoded);
        println!("__CAPABILITY_FIXTURE_END__");
    }

    fn sample_compile_request_json() -> serde_json::Value {
        serde_json::json!({
            "capability_context": serde_json::to_value(current_capability_context()).unwrap(),
            "runtime_config": {
                "metadata": {
                    "graph_id": "graph_test",
                    "compile_id": "compile_test",
                    "name": "Test Graph",
                    "version": "1.0.0",
                    "mode": "paper"
                },
                "data_sources": [
                    {
                        "id": "data_data_1",
                        "module_key": "builtin.data.kline",
                        "name": "Data",
                        "config": {
                            "exchange": "binance",
                            "instrument": "BTCUSDT",
                            "timeframe": "1d",
                            "window_size": 200
                        }
                    }
                ],
                "intent_generators": [
                    {
                        "id": "intent_intent_1",
                        "module_key": "builtin.intent.double_ma",
                        "name": "Intent",
                        "config": {
                            "fast_period": 20,
                            "slow_period": 50,
                            "entry_ratio": 0.2
                        },
                        "input_refs": [
                            {
                                "source_id": "data_data_1",
                                "source_port": "market_data_out",
                                "target_port": "data_input"
                            }
                        ]
                    }
                ],
                "agents": [
                    {
                        "id": "agent_agent_1",
                        "module_key": "builtin.agent.weighted",
                        "name": "Agent",
                        "config": {
                            "decision_threshold": 0.05,
                            "max_quantity_ratio": 0.2
                        },
                        "intent_refs": ["intent_intent_1"]
                    }
                ],
                "risk_controls": [
                    {
                        "id": "risk_risk_1",
                        "module_key": "builtin.risk.global",
                        "name": "Risk",
                        "config": {
                            "profile_id": "global",
                            "max_position": 0.2,
                            "max_total_leverage": 3.0,
                            "max_exchange_leverage": 3.0,
                            "min_action_interval_ms": 100
                        },
                        "agent_refs": ["agent_agent_1"]
                    }
                ],
                "executions": [
                    {
                        "id": "execution_execution_1",
                        "module_key": "builtin.execution.paper",
                        "name": "Execution",
                        "config": {
                            "profile_id": "paper",
                            "mode": "paper",
                            "slippage_bps": 5
                        },
                        "risk_ref": "risk_risk_1"
                    }
                ],
                "runtime_control": {
                    "id": "runtime_runtime_1",
                    "module_key": "builtin.runtime.control",
                    "name": "Runtime",
                    "config": {
                        "mode": "paper"
                    }
                }
            },
            "graph_json": {
                "metadata": { "graph_id": "graph_test", "name": "Test Graph", "version": "1.0.0" },
                "nodes": [
                    { "id": "data_data_1", "type": "data", "module_key": "builtin.data.kline", "name": "Data", "config": { "exchange": "binance", "instrument": "BTCUSDT", "timeframe": "1d", "window_size": 200 } },
                    { "id": "intent_intent_1", "type": "intent", "module_key": "builtin.intent.double_ma", "name": "Intent", "config": { "fast_period": 20, "slow_period": 50, "entry_ratio": 0.2 } },
                    { "id": "agent_agent_1", "type": "agent", "module_key": "builtin.agent.weighted", "name": "Agent", "config": { "decision_threshold": 0.05, "max_quantity_ratio": 0.2 } },
                    { "id": "risk_risk_1", "type": "risk", "module_key": "builtin.risk.global", "name": "Risk", "config": { "profile_id": "global", "max_position": 0.2, "max_total_leverage": 3.0, "max_exchange_leverage": 3.0, "min_action_interval_ms": 100 } },
                    { "id": "execution_execution_1", "type": "execution", "module_key": "builtin.execution.paper", "name": "Execution", "config": { "profile_id": "paper", "mode": "paper", "slippage_bps": 5 } },
                    { "id": "runtime_runtime_1", "type": "runtime", "module_key": "builtin.runtime.control", "name": "Runtime", "config": { "mode": "paper" } }
                ],
                "edges": [
                    { "source_node_id": "data_data_1", "source_port": "market_data_out", "target_node_id": "intent_intent_1", "target_port": "data_input" },
                    { "source_node_id": "intent_intent_1", "source_port": "intent_out", "target_node_id": "agent_agent_1", "target_port": "intent_input" },
                    { "source_node_id": "agent_agent_1", "source_port": "agent_out", "target_node_id": "risk_risk_1", "target_port": "agent_input" },
                    { "source_node_id": "risk_risk_1", "source_port": "risk_out", "target_node_id": "execution_execution_1", "target_port": "risk_input" }
                ]
            }
        })
    }

    fn sample_strategy_ir_compile_request_json() -> serde_json::Value {
        serde_json::json!({
            "graph_id": "strategy_graph_test",
            "compile_id": "strategy_compile_test",
            "strategy_ir": {
                "ir_version": "strategy_ir/v0",
                "metadata": {
                    "strategy_id": "restricted_custom_v1",
                    "name": "Restricted Custom",
                    "summary": "Custom signal lowered into Core IR.",
                    "source": {
                        "source_type": "manual_paper_analysis",
                        "paper_title": "Restricted Custom",
                        "paper_reference": null
                    },
                    "authors": ["QuantPilot"],
                    "tags": ["custom"]
                },
                "signals": [
                    {
                        "signal_id": "custom_signal",
                        "name": "Custom Signal",
                        "indicator": {
                            "kind": "custom",
                            "inputs": ["btc_1d"],
                            "params": {
                                "custom_expr": {
                                    "schema_version": "quantpilot/custom-expr/v1",
                                    "signal_kind": "long",
                                    "predicate": {
                                        "left": {
                                            "kind": "window_agg",
                                            "data_id": "btc_1d",
                                            "field": "close",
                                            "window_size": 3,
                                            "agg": "mean"
                                        },
                                        "op": "gt",
                                        "right": {
                                            "kind": "number",
                                            "value": 100.0
                                        }
                                    },
                                    "strength": {
                                        "kind": "binary",
                                        "left": {
                                            "kind": "input",
                                            "data_id": "btc_1d",
                                            "field": "close"
                                        },
                                        "op": "sub",
                                        "right": {
                                            "kind": "number",
                                            "value": 95.0
                                        }
                                    },
                                    "confidence": 0.9
                                }
                            }
                        },
                        "transforms": []
                    }
                ],
                "logic": {
                    "entry_rules": [
                        {
                            "rule_id": "entry_rule",
                            "condition": "custom_signal > 0",
                            "action": "open_long"
                        }
                    ],
                    "exit_rules": [],
                    "position_sizing": {
                        "method": "fixed_ratio",
                        "value": 0.2,
                        "unit": "portfolio_ratio"
                    },
                    "rebalance_rule": null
                },
                "risk_rules": {
                    "max_position_ratio": 0.2,
                    "stop_loss_ratio": 0.05,
                    "take_profit_ratio": null,
                    "max_drawdown_ratio": null,
                    "max_trades_per_day": null,
                    "notes": []
                },
                "data_requirements": [
                    {
                        "data_id": "btc_1d",
                        "venue": "binance",
                        "symbol": "BTCUSDT",
                        "data_type": "kline",
                        "granularity": "1d",
                        "lookback": 200,
                        "fields": ["close"]
                    }
                ],
                "execution": {
                    "venue_type": "paper",
                    "order_type": "market",
                    "time_in_force": null,
                    "slippage_model": "fixed_bps",
                    "latency_assumption_ms": null,
                    "capital_base": null
                },
                "gap_annotations": [],
                "unknowns": []
            }
        })
    }

    fn sample_spread_compile_request_json() -> serde_json::Value {
        serde_json::json!({
            "runtime_config": {
                "metadata": {
                    "graph_id": "graph_spread_test",
                    "compile_id": "compile_spread_test",
                    "name": "Spread Test Graph",
                    "version": "1.0.0",
                    "mode": "paper"
                },
                "data_sources": [
                    {
                        "id": "data_binance_quote",
                        "module_key": "builtin.data.quote",
                        "name": "Binance Quote",
                        "config": {
                            "exchange": "binance",
                            "instrument": "BTCUSDT"
                        }
                    },
                    {
                        "id": "data_okx_quote",
                        "module_key": "builtin.data.quote",
                        "name": "OKX Quote",
                        "config": {
                            "exchange": "okx",
                            "instrument": "BTCUSDT"
                        }
                    }
                ],
                "intent_generators": [
                    {
                        "id": "intent_spread_1",
                        "module_key": "builtin.intent.spread_observer",
                        "name": "Spread Observer",
                        "config": {
                            "max_time_diff_ms": 5000,
                            "field_code": 0,
                            "align_direction_code": 0,
                            "resample_period_ms": 60000,
                            "resample_agg_code": 0,
                            "window_size": 3,
                            "window_agg_code": 1,
                            "spread_output_code": 1
                        },
                        "input_refs": [
                            {
                                "source_id": "data_binance_quote",
                                "source_port": "market_data_out",
                                "target_port": "data_input"
                            },
                            {
                                "source_id": "data_okx_quote",
                                "source_port": "market_data_out",
                                "target_port": "data_input"
                            }
                        ]
                    }
                ],
                "agents": [
                    {
                        "id": "agent_arb_1",
                        "module_key": "builtin.agent.arbitrage",
                        "name": "Arbitrage Agent",
                        "config": {
                            "spread_trigger_bps": 30,
                            "max_quantity_ratio": 0.2
                        },
                        "intent_refs": ["intent_spread_1"]
                    }
                ],
                "risk_controls": [
                    {
                        "id": "risk_risk_1",
                        "module_key": "builtin.risk.global",
                        "name": "Risk",
                        "config": {
                            "profile_id": "global",
                            "max_position": 0.2,
                            "max_total_leverage": 3.0,
                            "max_exchange_leverage": 3.0,
                            "min_action_interval_ms": 100
                        },
                        "agent_refs": ["agent_arb_1"]
                    }
                ],
                "executions": [
                    {
                        "id": "execution_execution_1",
                        "module_key": "builtin.execution.paper",
                        "name": "Execution",
                        "config": {
                            "profile_id": "paper",
                            "mode": "paper",
                            "slippage_bps": 5
                        },
                        "risk_ref": "risk_risk_1"
                    }
                ],
                "runtime_control": {
                    "id": "runtime_runtime_1",
                    "module_key": "builtin.runtime.control",
                    "name": "Runtime",
                    "config": {
                        "mode": "paper"
                    }
                }
            }
        })
    }

    async fn compile_formal_quantscript_for_test(
        source: &str,
        compile_id: &str,
    ) -> serde_json::Value {
        compile_formal_quantscript_for_test_with_universe_snapshot(source, compile_id, None).await
    }

    async fn compile_formal_quantscript_for_test_with_universe_snapshot(
        source: &str,
        compile_id: &str,
        universe_snapshot: Option<serde_json::Value>,
    ) -> serde_json::Value {
        let graph_app = build_app_router(test_app_state());
        let mut payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": compile_id,
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": source,
        });
        if let Some(snapshot) = universe_snapshot {
            payload["universe_snapshot"] = snapshot;
        }

        let response = graph_app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        serde_json::from_slice(&body).unwrap()
    }

    fn sample_formal_universe_snapshot_json() -> serde_json::Value {
        serde_json::json!({
            "snapshot_id": "authoring_view_pool_pipeline_snapshot",
            "as_of_ms": 1_710_000_000_000u64,
            "assets": [
                {
                    "symbol": "BTCUSDT",
                    "exchange": "Binance",
                    "market_type": "Spot",
                    "quote": "USDT",
                    "market_cap": 1_500_000_000_000.0,
                    "volume_24h": 40_000_000_000.0,
                    "listed_at_ms": 1_500_000_000_000u64,
                    "enabled": true
                },
                {
                    "symbol": "ETHUSDT",
                    "exchange": "Binance",
                    "market_type": "Spot",
                    "quote": "USDT",
                    "market_cap": 500_000_000_000.0,
                    "volume_24h": 18_000_000_000.0,
                    "listed_at_ms": 1_510_000_000_000u64,
                    "enabled": true
                },
                {
                    "symbol": "SOLUSDT",
                    "exchange": "Binance",
                    "market_type": "Spot",
                    "quote": "USDT",
                    "market_cap": 120_000_000_000.0,
                    "volume_24h": 4_000_000_000.0,
                    "listed_at_ms": 1_520_000_000_000u64,
                    "enabled": true
                }
            ]
        })
    }

    async fn compile_formal_quantscript_error_for_test(
        source: &str,
        compile_id: &str,
    ) -> serde_json::Value {
        compile_formal_quantscript_error_for_test_with_universe_snapshot(source, compile_id, None)
            .await
    }

    async fn compile_formal_quantscript_error_for_test_with_universe_snapshot(
        source: &str,
        compile_id: &str,
        universe_snapshot: Option<serde_json::Value>,
    ) -> serde_json::Value {
        let graph_app = build_app_router(test_app_state());
        let mut payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": compile_id,
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": source,
        });
        if let Some(snapshot) = universe_snapshot {
            payload["universe_snapshot"] = snapshot;
        }

        let response = graph_app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "{}",
            String::from_utf8_lossy(&body)
        );
        serde_json::from_slice(&body).unwrap()
    }

    fn formal_compile_authoring_view(value: &serde_json::Value) -> serde_json::Value {
        value["artifacts"]["strategy"]["metadata"]["quantscript_authoring_view"].clone()
    }

    fn formal_compile_partial_authoring_view(value: &serde_json::Value) -> serde_json::Value {
        value["partial_artifacts"]["quantscript_authoring_view"].clone()
    }

    async fn compile_runtime_graph_for_test(
        module_key: &str,
        config: serde_json::Value,
        compile_id: &str,
    ) -> serde_json::Value {
        let graph_app = build_app_router(test_app_state());
        let mut payload = sample_compile_request_json();
        payload["compile_id"] = serde_json::Value::String(compile_id.to_string());
        payload["runtime_config"]["intent_generators"][0]["module_key"] =
            serde_json::Value::String(module_key.to_string());
        payload["runtime_config"]["intent_generators"][0]["config"] = config;

        let response = graph_app
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        serde_json::from_slice(&body).unwrap()
    }

    async fn compile_runtime_spread_graph_for_test(
        config: serde_json::Value,
        compile_id: &str,
    ) -> serde_json::Value {
        let graph_app = build_app_router(test_app_state());
        let mut payload = sample_spread_compile_request_json();
        payload["compile_id"] = serde_json::Value::String(compile_id.to_string());
        payload["runtime_config"]["intent_generators"][0]["config"] = config;

        let response = graph_app
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        serde_json::from_slice(&body).unwrap()
    }

    async fn compile_runtime_spread_graph_error_for_test(
        config: serde_json::Value,
        compile_id: &str,
    ) -> serde_json::Value {
        let graph_app = build_app_router(test_app_state());
        let mut payload = sample_spread_compile_request_json();
        payload["compile_id"] = serde_json::Value::String(compile_id.to_string());
        payload["runtime_config"]["intent_generators"][0]["config"] = config;

        let response = graph_app
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "{}",
            String::from_utf8_lossy(&body)
        );
        serde_json::from_slice(&body).unwrap()
    }

    async fn compile_strategy_ir_for_test(
        signal_id: &str,
        signal_name: &str,
        indicator_kind: &str,
        indicator_params: serde_json::Value,
        condition: &str,
        compile_id: &str,
    ) -> serde_json::Value {
        let graph_app = build_app_router(test_app_state());
        let mut payload = sample_strategy_ir_compile_request_json();
        payload["compile_id"] = serde_json::Value::String(compile_id.to_string());
        payload["strategy_ir"]["signals"][0]["signal_id"] =
            serde_json::Value::String(signal_id.to_string());
        payload["strategy_ir"]["signals"][0]["name"] =
            serde_json::Value::String(signal_name.to_string());
        payload["strategy_ir"]["signals"][0]["indicator"]["kind"] =
            serde_json::Value::String(indicator_kind.to_string());
        payload["strategy_ir"]["signals"][0]["indicator"]["params"] = indicator_params;
        payload["strategy_ir"]["logic"]["entry_rules"][0]["condition"] =
            serde_json::Value::String(condition.to_string());
        payload["strategy_ir"]["logic"]["entry_rules"][0]["action"] =
            serde_json::Value::String("open_long".to_string());

        let response = graph_app
            .oneshot(
                Request::builder()
                    .uri("/api/strategy-ir/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        serde_json::from_slice(&body).unwrap()
    }

    async fn compile_strategy_ir_spread_for_test(
        indicator_params: serde_json::Value,
        condition: &str,
        compile_id: &str,
    ) -> serde_json::Value {
        let graph_app = build_app_router(test_app_state());
        let mut payload = sample_strategy_ir_compile_request_json();
        payload["compile_id"] = serde_json::Value::String(compile_id.to_string());
        payload["strategy_ir"]["signals"][0]["signal_id"] =
            serde_json::Value::String("spread_signal".to_string());
        payload["strategy_ir"]["signals"][0]["name"] =
            serde_json::Value::String("Spread Signal".to_string());
        payload["strategy_ir"]["signals"][0]["indicator"]["kind"] =
            serde_json::Value::String("spread".to_string());
        payload["strategy_ir"]["signals"][0]["indicator"]["inputs"] =
            serde_json::json!(["binance_btc_quote", "okx_btc_quote"]);
        payload["strategy_ir"]["signals"][0]["indicator"]["params"] = indicator_params;
        payload["strategy_ir"]["logic"]["entry_rules"][0]["condition"] =
            serde_json::Value::String(condition.to_string());
        payload["strategy_ir"]["logic"]["entry_rules"][0]["action"] =
            serde_json::Value::String("open_long".to_string());
        payload["strategy_ir"]["data_requirements"] = serde_json::json!([
            {
                "data_id": "binance_btc_quote",
                "venue": "binance",
                "symbol": "BTCUSDT",
                "data_type": "quote",
                "granularity": "1m",
                "lookback": 200,
                "fields": ["bid", "ask", "mid"]
            },
            {
                "data_id": "okx_btc_quote",
                "venue": "okx",
                "symbol": "BTCUSDT",
                "data_type": "quote",
                "granularity": "1m",
                "lookback": 200,
                "fields": ["bid", "ask", "mid"]
            }
        ]);

        let response = graph_app
            .oneshot(
                Request::builder()
                    .uri("/api/strategy-ir/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        serde_json::from_slice(&body).unwrap()
    }

    async fn compile_strategy_ir_spread_error_for_test(
        indicator_params: serde_json::Value,
        condition: &str,
        compile_id: &str,
    ) -> serde_json::Value {
        let graph_app = build_app_router(test_app_state());
        let mut payload = sample_strategy_ir_compile_request_json();
        payload["compile_id"] = serde_json::Value::String(compile_id.to_string());
        payload["strategy_ir"]["signals"][0]["signal_id"] =
            serde_json::Value::String("spread_signal".to_string());
        payload["strategy_ir"]["signals"][0]["name"] =
            serde_json::Value::String("Spread Signal".to_string());
        payload["strategy_ir"]["signals"][0]["indicator"]["kind"] =
            serde_json::Value::String("spread".to_string());
        payload["strategy_ir"]["signals"][0]["indicator"]["inputs"] =
            serde_json::json!(["binance_btc_quote", "okx_btc_quote"]);
        payload["strategy_ir"]["signals"][0]["indicator"]["params"] = indicator_params;
        payload["strategy_ir"]["logic"]["entry_rules"][0]["condition"] =
            serde_json::Value::String(condition.to_string());
        payload["strategy_ir"]["logic"]["entry_rules"][0]["action"] =
            serde_json::Value::String("open_long".to_string());
        payload["strategy_ir"]["data_requirements"] = serde_json::json!([
            {
                "data_id": "binance_btc_quote",
                "venue": "binance",
                "symbol": "BTCUSDT",
                "data_type": "quote",
                "granularity": "1m",
                "lookback": 200,
                "fields": ["bid", "ask", "mid"]
            },
            {
                "data_id": "okx_btc_quote",
                "venue": "okx",
                "symbol": "BTCUSDT",
                "data_type": "quote",
                "granularity": "1m",
                "lookback": 200,
                "fields": ["bid", "ask", "mid"]
            }
        ]);

        let response = graph_app
            .oneshot(
                Request::builder()
                    .uri("/api/strategy-ir/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "{}",
            String::from_utf8_lossy(&body)
        );
        serde_json::from_slice(&body).unwrap()
    }

    fn formal_compile_golden_view(value: &serde_json::Value) -> serde_json::Value {
        let indicator_kinds = value["core_ir"]["indicators"]
            .as_array()
            .unwrap()
            .iter()
            .map(|indicator| indicator["kind"].clone())
            .collect::<Vec<_>>();
        let agent_policy_kinds = value["core_ir"]["agent_policies"]
            .as_array()
            .unwrap()
            .iter()
            .map(|policy| policy["kind"].clone())
            .collect::<Vec<_>>();

        serde_json::json!({
            "core_source_kind": value["core_ir"]["metadata"]["source_kind"].clone(),
            "data_bindings": value["core_ir"]["data_bindings"].clone(),
            "indicator_kinds": indicator_kinds,
            "signal_rules": value["core_ir"]["signal_rules"].clone(),
            "agent_policy_kinds": agent_policy_kinds,
            "runtime_projection": {
                "data_modules": value["runtime_config"]["data_sources"].as_array().unwrap().iter().map(|node| node["module_key"].clone()).collect::<Vec<_>>(),
                "intent_modules": value["runtime_config"]["intent_generators"].as_array().unwrap().iter().map(|node| node["module_key"].clone()).collect::<Vec<_>>(),
                "agent_modules": value["runtime_config"]["agents"].as_array().unwrap().iter().map(|node| node["module_key"].clone()).collect::<Vec<_>>(),
                "risk_modules": value["runtime_config"]["risk_controls"].as_array().unwrap().iter().map(|node| node["module_key"].clone()).collect::<Vec<_>>(),
                "execution_modules": value["runtime_config"]["executions"].as_array().unwrap().iter().map(|node| node["module_key"].clone()).collect::<Vec<_>>(),
                "runtime_module": value["runtime_config"]["runtime_control"]["module_key"].clone(),
            }
        })
    }

    fn formal_compile_error_golden_view(value: &serde_json::Value) -> serde_json::Value {
        let first_detail = value["details"].as_array().unwrap().first().unwrap();

        serde_json::json!({
            "error": value["error"].clone(),
            "detail": {
                "code": first_detail["code"].clone(),
                "message": first_detail["message"].clone(),
                "reason": first_detail
                    .get("reason")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
                "span_label": first_detail
                    .get("span_label")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            }
        })
    }

    fn formal_compile_error_details_golden_view(value: &serde_json::Value) -> serde_json::Value {
        let details = value["details"]
            .as_array()
            .unwrap()
            .iter()
            .map(|detail| {
                serde_json::json!({
                    "code": detail["code"].clone(),
                    "message": detail["message"].clone(),
                    "span_label": detail
                        .get("span_label")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                })
            })
            .collect::<Vec<_>>();

        serde_json::json!({
            "error": value["error"].clone(),
            "details": details,
        })
    }

    fn expected_formal_spread_rejection_golden_view() -> serde_json::Value {
        serde_json::json!({
            "error": "quantscript_lowering_failed",
            "detail": {
                "code": "QPQSLOW001",
                "message": "QPQSLOW001 不支持的条件下发 Intent 下层转换: 条件必须映射到支持的指标或价差意图",
                "reason": "将条件下发重写为支持的指标或价差意图，或保留下发为无条件。",
                "span_label": serde_json::Value::Null,
            }
        })
    }

    fn canonical_condition_for_entry_equivalence(value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Array(items) => serde_json::Value::Array(
                items
                    .iter()
                    .map(canonical_condition_for_entry_equivalence)
                    .collect(),
            ),
            serde_json::Value::Object(map) => {
                if map.get("kind").and_then(|kind| kind.as_str()) == Some("ref") {
                    serde_json::json!({
                        "kind": "ref",
                        "name": "__ref__"
                    })
                } else {
                    let mut normalized = serde_json::Map::new();
                    for (key, child) in map {
                        if key == "data_id" {
                            normalized.insert(
                                key.clone(),
                                serde_json::Value::String("__data__".to_string()),
                            );
                            continue;
                        }
                        normalized.insert(
                            key.clone(),
                            canonical_condition_for_entry_equivalence(child),
                        );
                    }
                    serde_json::Value::Object(normalized)
                }
            }
            _ => value.clone(),
        }
    }

    fn core_ir_entry_equivalence_view(core_ir: &serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "indicator_kind": core_ir["indicators"][0]["kind"].clone(),
            "condition": canonical_condition_for_entry_equivalence(&core_ir["signal_rules"][0]["condition"]),
        })
    }

    fn core_ir_risk_profile_equivalence_view(core_ir: &serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "max_position_ratio": core_ir["risk_policies"][0]["max_position_ratio"].clone(),
            "max_total_leverage": core_ir["risk_policies"][0]["max_total_leverage"].clone(),
            "max_exchange_leverage": core_ir["risk_policies"][0]["max_exchange_leverage"].clone(),
            "min_action_interval_ms": core_ir["risk_policies"][0]["min_action_interval_ms"].clone(),
        })
    }

    fn core_ir_execution_profile_equivalence_view(
        core_ir: &serde_json::Value,
    ) -> serde_json::Value {
        serde_json::json!({
            "venue_kind": core_ir["execution"]["venue_kind"].clone(),
            "taker_fee_bps": core_ir["execution"]["taker_fee_bps"].clone(),
            "slippage_bps": core_ir["execution"]["slippage_bps"].clone(),
        })
    }

    fn graph_value_from_runtime_config(
        runtime_config: &serde_json::Value,
        formal_source: &str,
    ) -> serde_json::Value {
        let mut nodes = Vec::<serde_json::Value>::new();
        let mut edges = Vec::<serde_json::Value>::new();

        for node in runtime_config["data_sources"].as_array().unwrap() {
            nodes.push(serde_json::json!({
                "id": node["id"].clone(),
                "type": "data",
                "module_key": node["module_key"].clone(),
                "name": node["name"].clone(),
                "config": node["config"].clone(),
            }));
        }

        for node in runtime_config["intent_generators"].as_array().unwrap() {
            nodes.push(serde_json::json!({
                "id": node["id"].clone(),
                "type": "intent",
                "module_key": node["module_key"].clone(),
                "name": node["name"].clone(),
                "config": node["config"].clone(),
            }));

            for input_ref in node["input_refs"].as_array().unwrap() {
                edges.push(serde_json::json!({
                    "source_node_id": input_ref["source_id"].clone(),
                    "source_port": input_ref["source_port"].clone(),
                    "target_node_id": node["id"].clone(),
                    "target_port": input_ref["target_port"].clone(),
                }));
            }
        }

        for node in runtime_config["agents"].as_array().unwrap() {
            nodes.push(serde_json::json!({
                "id": node["id"].clone(),
                "type": "agent",
                "module_key": node["module_key"].clone(),
                "name": node["name"].clone(),
                "config": node["config"].clone(),
            }));

            for intent_ref in node["intent_refs"].as_array().unwrap() {
                edges.push(serde_json::json!({
                    "source_node_id": intent_ref.clone(),
                    "source_port": "intent_out",
                    "target_node_id": node["id"].clone(),
                    "target_port": "intent_input",
                }));
            }
        }

        for node in runtime_config["risk_controls"].as_array().unwrap() {
            nodes.push(serde_json::json!({
                "id": node["id"].clone(),
                "type": "risk",
                "module_key": node["module_key"].clone(),
                "name": node["name"].clone(),
                "config": node["config"].clone(),
            }));

            for agent_ref in node["agent_refs"].as_array().unwrap() {
                edges.push(serde_json::json!({
                    "source_node_id": agent_ref.clone(),
                    "source_port": "agent_out",
                    "target_node_id": node["id"].clone(),
                    "target_port": "agent_input",
                }));
            }
        }

        for node in runtime_config["executions"].as_array().unwrap() {
            nodes.push(serde_json::json!({
                "id": node["id"].clone(),
                "type": "execution",
                "module_key": node["module_key"].clone(),
                "name": node["name"].clone(),
                "config": node["config"].clone(),
            }));

            if !node["risk_ref"].is_null() {
                edges.push(serde_json::json!({
                    "source_node_id": node["risk_ref"].clone(),
                    "source_port": "risk_out",
                    "target_node_id": node["id"].clone(),
                    "target_port": "risk_input",
                }));
            }
        }

        let runtime_node = &runtime_config["runtime_control"];
        nodes.push(serde_json::json!({
            "id": runtime_node["id"].clone(),
            "type": "runtime",
            "module_key": runtime_node["module_key"].clone(),
            "name": runtime_node["name"].clone(),
            "config": runtime_node["config"].clone(),
        }));

        if let Some(execution_node) = runtime_config["executions"].as_array().unwrap().first() {
            edges.push(serde_json::json!({
                "source_node_id": execution_node["id"].clone(),
                "source_port": "execution_out",
                "target_node_id": runtime_node["id"].clone(),
                "target_port": "execution_input",
            }));
        }

        serde_json::json!({
            "metadata": {
                "graph_id": runtime_config["metadata"]["graph_id"].clone(),
                "name": runtime_config["metadata"]["name"].clone(),
                "version": runtime_config["metadata"]["version"].clone(),
                "artifacts": {
                    "quantscript": {
                        "formal_source": formal_source,
                    }
                }
            },
            "nodes": nodes,
            "edges": edges,
        })
    }

    fn api_error_detail_by_code<'a>(
        value: &'a serde_json::Value,
        code: &str,
    ) -> &'a serde_json::Value {
        value["details"]
            .as_array()
            .unwrap()
            .iter()
            .find(|detail| detail["code"] == code)
            .unwrap_or_else(|| panic!("missing api error detail for code {code}"))
    }

    #[test]
    fn attach_quantscript_artifacts_preserves_node_source_targets() {
        let mut graph = serde_json::json!({
            "metadata": {
                "graph_id": "graph_test",
                "name": "Test Graph",
                "version": "1.0.0"
            },
            "nodes": [
                {
                    "id": "data_feed",
                    "type": "data",
                    "module_key": "builtin.data.kline",
                    "name": "Price Feed",
                    "config": {
                        "window_size": 20,
                        "timeframe": "1d"
                    }
                }
            ],
            "edges": []
        });

        attach_quantscript_artifacts(
            &mut graph,
            "strategy_graph graph_test {\n}",
            1,
            std::path::Path::new("storage/graphs/graph_test.qs"),
        );

        let quantscript = &graph["metadata"]["artifacts"]["quantscript"];
        assert!(quantscript["node_sources"]["data_feed"].is_string());
        assert_eq!(
            quantscript["label_targets"]["Price Feed.window_size"]["node_id"],
            "data_feed"
        );
        assert_eq!(
            quantscript["label_targets"]["Price Feed.window_size"]["field"],
            "window_size"
        );
        assert_eq!(
            quantscript["runtime_targets"]["source_to_node"]["data_data_feed"],
            "data_feed"
        );
    }

    #[test]
    fn attach_quantscript_artifacts_preserves_formal_source() {
        let mut graph = serde_json::json!({
            "metadata": {
                "graph_id": "graph_test",
                "name": "Test Graph",
                "version": "1.0.0",
                "artifacts": {
                    "quantscript": {
                        "formal_source": "fn strategy() {\n    emit Intent(\"BUY\", instrument=\"BTCUSDT\", quantity=1.0)\n}"
                    }
                }
            },
            "nodes": [],
            "edges": []
        });

        attach_quantscript_artifacts(
            &mut graph,
            "strategy_graph graph_test {\n}",
            1,
            std::path::Path::new("storage/graphs/graph_test.qs"),
        );

        assert_eq!(
            graph["metadata"]["artifacts"]["quantscript"]["formal_source"],
            "fn strategy() {\n    emit Intent(\"BUY\", instrument=\"BTCUSDT\", quantity=1.0)\n}"
        );
    }

    #[tokio::test]
    async fn compile_endpoint_accepts_spread_arbitrage_modules_and_lowers_spread_indicator() {
        let graph_app = build_app_router(test_app_state());

        let response = graph_app
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(sample_spread_compile_request_json().to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            value["artifacts"]["core_ir"]["core_ir"]["indicators"][0]["kind"],
            "spread"
        );
        assert_eq!(
            value["artifacts"]["core_ir"]["core_ir"]["indicators"][0]["spread_spec"]["output"],
            "bps"
        );
        assert_eq!(
            value["artifacts"]["core_ir"]["core_ir"]["agent_policies"][0]["kind"],
            "cross_venue_arbitrage"
        );
    }

    #[tokio::test]
    async fn compile_endpoint_roundtrips_data_request_controls() {
        let graph_app = build_app_router(test_app_state());
        let mut payload = sample_compile_request_json();
        payload["runtime_config"]["data_sources"][0]["config"]["ping_enabled"] =
            serde_json::Value::Bool(true);
        payload["runtime_config"]["data_sources"][0]["config"]["request_interval_ms"] =
            serde_json::Value::from(2_500_u64);

        let response = graph_app
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            value["artifacts"]["core_ir"]["core_ir"]["data_bindings"][0]["source_hints"]
                ["ping_enabled"],
            "true"
        );
        assert_eq!(
            value["artifacts"]["core_ir"]["core_ir"]["data_bindings"][0]["source_hints"]
                ["request_interval_ms"],
            "2500"
        );
    }

    #[tokio::test]
    async fn compile_endpoint_lowers_graph_spread_bps_to_structured_threshold_condition() {
        let value = compile_runtime_spread_graph_for_test(
            serde_json::json!({
                "max_time_diff_ms": 5000,
                "field_code": 0,
                "align_direction_code": 0,
                "resample_period_ms": 0,
                "resample_agg_code": 0,
                "window_size": 1,
                "window_agg_code": 1,
                "spread_output_code": 1,
                "comparison_shape_code": 1,
                "comparison_op_code": 2,
                "comparison_threshold": 5.0
            }),
            "compile_graph_spread_threshold",
        )
        .await;

        assert_eq!(
            value["artifacts"]["core_ir"]["core_ir"]["indicators"][0]["kind"],
            "spread"
        );
        assert_eq!(
            value["artifacts"]["core_ir"]["core_ir"]["signal_rules"][0]["condition"],
            serde_json::json!({
                "kind": "compare",
                "left": {
                    "kind": "ref",
                    "name": "intent_spread_1"
                },
                "op": "gt",
                "right": {
                    "kind": "number",
                    "value": 5.0
                }
            })
        );
    }

    #[tokio::test]
    async fn compile_endpoint_rejects_graph_spread_threshold_with_non_bps_output() {
        let value = compile_runtime_spread_graph_error_for_test(
            serde_json::json!({
                "max_time_diff_ms": 5000,
                "field_code": 0,
                "align_direction_code": 0,
                "resample_period_ms": 0,
                "resample_agg_code": 0,
                "window_size": 1,
                "window_agg_code": 1,
                "spread_output_code": 0,
                "comparison_shape_code": 1,
                "comparison_op_code": 2,
                "comparison_threshold": 5.0
            }),
            "compile_graph_spread_non_bps_reject",
        )
        .await;

        assert_eq!(value["error"], "runtime_compile_failed");
        assert_eq!(value["details"][0]["code"], "QPSPREAD001");
    }

    #[tokio::test]
    async fn compile_endpoint_rejects_graph_spread_threshold_with_non_positive_tolerance() {
        let value = compile_runtime_spread_graph_error_for_test(
            serde_json::json!({
                "max_time_diff_ms": 0,
                "field_code": 0,
                "align_direction_code": 0,
                "resample_period_ms": 0,
                "resample_agg_code": 0,
                "window_size": 1,
                "window_agg_code": 1,
                "spread_output_code": 1,
                "comparison_shape_code": 1,
                "comparison_op_code": 2,
                "comparison_threshold": 5.0
            }),
            "compile_graph_spread_bad_tolerance_reject",
        )
        .await;

        assert_eq!(value["error"], "runtime_compile_failed");
        assert_eq!(value["details"][0]["code"], "QPSPREAD002");
    }

    #[tokio::test]
    async fn compile_endpoint_rejects_graph_spread_threshold_with_non_one_sided_shape() {
        let value = compile_runtime_spread_graph_error_for_test(
            serde_json::json!({
                "max_time_diff_ms": 5000,
                "field_code": 0,
                "align_direction_code": 0,
                "resample_period_ms": 0,
                "resample_agg_code": 0,
                "window_size": 1,
                "window_agg_code": 1,
                "spread_output_code": 1,
                "comparison_shape_code": 2,
                "comparison_op_code": 0,
                "comparison_threshold": 5.0
            }),
            "compile_graph_spread_bad_shape_reject",
        )
        .await;

        assert_eq!(value["error"], "runtime_compile_failed");
        assert_eq!(value["details"][0]["code"], "QPSPREAD003");
    }

    #[tokio::test]
    async fn compile_endpoint_rejects_unsupported_runtime_mode_with_structured_error() {
        let app = build_app_router(test_app_state());
        let mut payload = sample_compile_request_json();
        payload["runtime_config"]["metadata"]["mode"] =
            serde_json::Value::String("live".to_string());

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["error"], "capability_gated");
        assert_eq!(value["details"][0]["code"], "unsupported_runtime_mode");
        assert_eq!(value["details"][0]["target"], "metadata.mode");
    }

    #[tokio::test]
    async fn compile_endpoint_returns_warmup_diagnostics() {
        let app = build_app_router(test_app_state());
        let mut payload = sample_compile_request_json();
        payload["runtime_config"]["data_sources"][0]["config"]["window_size"] =
            serde_json::Value::from(20);
        payload["runtime_config"]["intent_generators"][0]["config"]["slow_period"] =
            serde_json::Value::from(50);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["diagnostics"][0]["code"], "QPWARM001");
        assert_eq!(value["diagnostics"][0]["severity"], "warning");
        assert_eq!(value["diagnostics"][0]["target"]["scope"], "node");
        assert_eq!(value["diagnostics"][0]["target"]["node_id"], "data_data_1");
        assert_eq!(value["diagnostics"][0]["target"]["field"], "window_size");
    }

    #[tokio::test]
    async fn compile_endpoint_returns_artifact_bundle() {
        let app = build_app_router(test_app_state());

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(sample_compile_request_json().to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            value["artifacts"]["strategy"]["schema_version"],
            "quantpilot/strategy-artifact/v1"
        );
        assert_eq!(
            value["artifacts"]["compile"]["schema_version"],
            "quantpilot/compile-artifact/v1"
        );
        assert_eq!(
            value["artifacts"]["core_ir"]["schema_version"],
            "quantpilot/core-ir-artifact/v1"
        );
        assert_eq!(
            value["artifacts"]["compile"]["strategy_artifact_id"],
            value["artifacts"]["strategy"]["artifact_id"]
        );
        assert_eq!(
            value["artifacts"]["compile"]["core_ir_artifact_id"],
            value["artifacts"]["core_ir"]["artifact_id"]
        );
        assert_eq!(
            value["artifacts"]["compile"]["config_hash"],
            value["config_hash"]
        );
    }

    #[tokio::test]
    async fn compile_endpoint_lowers_graph_momentum_to_structured_threshold_condition() {
        let app = build_app_router(test_app_state());
        let mut payload = sample_compile_request_json();
        payload["runtime_config"]["intent_generators"][0]["module_key"] =
            serde_json::Value::String("builtin.intent.momentum".to_string());
        payload["runtime_config"]["intent_generators"][0]["config"] = serde_json::json!({
            "lookback": 20,
            "threshold_ratio": 0.03
        });

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            value["artifacts"]["core_ir"]["core_ir"]["indicators"][0]["kind"],
            "momentum"
        );
        assert_eq!(
            value["artifacts"]["core_ir"]["core_ir"]["signal_rules"][0]["condition"],
            serde_json::json!({
                "kind": "compare",
                "left": {
                    "kind": "ref",
                    "name": "intent_intent_1"
                },
                "op": "gt",
                "right": {
                    "kind": "number",
                    "value": 0.03
                }
            })
        );
    }

    #[tokio::test]
    async fn compile_endpoint_lowers_graph_rsi_to_structured_threshold_condition() {
        let app = build_app_router(test_app_state());
        let mut payload = sample_compile_request_json();
        payload["runtime_config"]["intent_generators"][0]["module_key"] =
            serde_json::Value::String("builtin.intent.rsi".to_string());
        payload["runtime_config"]["intent_generators"][0]["config"] = serde_json::json!({
            "period": 14,
            "oversold_threshold": 25.0,
            "overbought_threshold": 70.0
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            value["artifacts"]["core_ir"]["core_ir"]["indicators"][0]["kind"],
            "rsi"
        );
        assert_eq!(
            value["artifacts"]["core_ir"]["core_ir"]["signal_rules"][0]["condition"],
            serde_json::json!({
                "kind": "compare",
                "left": {
                    "kind": "ref",
                    "name": "intent_intent_1"
                },
                "op": "lt",
                "right": {
                    "kind": "number",
                    "value": 25.0
                }
            })
        );
    }

    #[tokio::test]
    async fn compile_endpoint_lowers_graph_zscore_to_structured_threshold_condition() {
        let app = build_app_router(test_app_state());
        let mut payload = sample_compile_request_json();
        payload["runtime_config"]["intent_generators"][0]["module_key"] =
            serde_json::Value::String("builtin.intent.zscore".to_string());
        payload["runtime_config"]["intent_generators"][0]["config"] = serde_json::json!({
            "window": 20,
            "entry_z": 2.0
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            value["artifacts"]["core_ir"]["core_ir"]["indicators"][0]["kind"],
            "z_score"
        );
        assert_eq!(
            value["artifacts"]["core_ir"]["core_ir"]["signal_rules"][0]["condition"],
            serde_json::json!({
                "kind": "compare",
                "left": {
                    "kind": "ref",
                    "name": "intent_intent_1"
                },
                "op": "lt",
                "right": {
                    "kind": "number",
                    "value": -2.0
                }
            })
        );
    }

    #[tokio::test]
    async fn strategy_ir_compile_endpoint_accepts_restricted_custom_and_lowers_to_core_ir() {
        let app = build_app_router(test_app_state());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/strategy-ir/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        sample_strategy_ir_compile_request_json().to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["graph_id"], "strategy_graph_test");
        assert_eq!(value["compile_id"], "strategy_compile_test");
        assert_eq!(value["compilable"], true);
        assert_eq!(value["core_ir"]["indicators"][0]["kind"], "custom");
        assert_eq!(
            value["core_ir"]["indicators"][0]["custom_expr"]["schema_version"],
            "quantpilot/custom-expr/v1"
        );
        assert_eq!(value["diagnostics"], serde_json::json!([]));
    }

    #[tokio::test]
    async fn strategy_ir_compile_endpoint_lowers_rsi_to_structured_threshold_condition() {
        let app = build_app_router(test_app_state());
        let mut payload = sample_strategy_ir_compile_request_json();
        payload["strategy_ir"]["signals"][0]["signal_id"] =
            serde_json::Value::String("rsi_signal".to_string());
        payload["strategy_ir"]["signals"][0]["name"] =
            serde_json::Value::String("RSI Signal".to_string());
        payload["strategy_ir"]["signals"][0]["indicator"]["kind"] =
            serde_json::Value::String("rsi".to_string());
        payload["strategy_ir"]["signals"][0]["indicator"]["params"] = serde_json::json!({
            "period": 14
        });
        payload["strategy_ir"]["logic"]["entry_rules"][0]["condition"] =
            serde_json::Value::String("rsi_signal < 25".to_string());
        payload["strategy_ir"]["logic"]["entry_rules"][0]["action"] =
            serde_json::Value::String("open_long".to_string());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/strategy-ir/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["core_ir"]["indicators"][0]["kind"], "rsi");
        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"],
            serde_json::json!({
                "kind": "compare",
                "left": {
                    "kind": "ref",
                    "name": "rsi_signal"
                },
                "op": "lt",
                "right": {
                    "kind": "number",
                    "value": 25.0
                }
            })
        );
    }

    #[tokio::test]
    async fn strategy_ir_compile_endpoint_lowers_ma_cross_to_structured_series_compare() {
        let app = build_app_router(test_app_state());
        let mut payload = sample_strategy_ir_compile_request_json();
        payload["strategy_ir"]["signals"][0]["signal_id"] =
            serde_json::Value::String("ma_cross".to_string());
        payload["strategy_ir"]["signals"][0]["name"] =
            serde_json::Value::String("MA Cross".to_string());
        payload["strategy_ir"]["signals"][0]["indicator"]["kind"] =
            serde_json::Value::String("ma_cross".to_string());
        payload["strategy_ir"]["signals"][0]["indicator"]["params"] = serde_json::json!({
            "fast": 20,
            "slow": 50
        });
        payload["strategy_ir"]["logic"]["entry_rules"][0]["condition"] =
            serde_json::Value::String("ma_cross > 0".to_string());
        payload["strategy_ir"]["logic"]["entry_rules"][0]["action"] =
            serde_json::Value::String("open_long".to_string());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/strategy-ir/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["core_ir"]["indicators"][0]["kind"], "ma_cross");
        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"],
            serde_json::json!({
                "kind": "compare",
                "left": {
                    "kind": "series",
                    "expr": {
                        "kind": "window_agg",
                        "input": {
                            "kind": "data_field",
                            "data_id": "btc_1d",
                            "field": "close"
                        },
                        "window_size": 20,
                        "agg": "mean"
                    }
                },
                "op": "gt",
                "right": {
                    "kind": "series",
                    "expr": {
                        "kind": "window_agg",
                        "input": {
                            "kind": "data_field",
                            "data_id": "btc_1d",
                            "field": "close"
                        },
                        "window_size": 50,
                        "agg": "mean"
                    }
                }
            })
        );
    }

    #[tokio::test]
    async fn strategy_ir_compile_endpoint_lowers_momentum_to_structured_threshold_condition() {
        let app = build_app_router(test_app_state());
        let mut payload = sample_strategy_ir_compile_request_json();
        payload["strategy_ir"]["signals"][0]["signal_id"] =
            serde_json::Value::String("momentum_signal".to_string());
        payload["strategy_ir"]["signals"][0]["name"] =
            serde_json::Value::String("Momentum Signal".to_string());
        payload["strategy_ir"]["signals"][0]["indicator"]["kind"] =
            serde_json::Value::String("momentum".to_string());
        payload["strategy_ir"]["signals"][0]["indicator"]["params"] = serde_json::json!({
            "lookback": 20
        });
        payload["strategy_ir"]["logic"]["entry_rules"][0]["condition"] =
            serde_json::Value::String("momentum_signal > 0.03".to_string());
        payload["strategy_ir"]["logic"]["entry_rules"][0]["action"] =
            serde_json::Value::String("open_long".to_string());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/strategy-ir/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["core_ir"]["indicators"][0]["kind"], "momentum");
        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"],
            serde_json::json!({
                "kind": "compare",
                "left": {
                    "kind": "ref",
                    "name": "momentum_signal"
                },
                "op": "gt",
                "right": {
                    "kind": "number",
                    "value": 0.03
                }
            })
        );
    }

    #[tokio::test]
    async fn strategy_ir_compile_endpoint_lowers_zscore_to_structured_threshold_condition() {
        let app = build_app_router(test_app_state());
        let mut payload = sample_strategy_ir_compile_request_json();
        payload["strategy_ir"]["signals"][0]["signal_id"] =
            serde_json::Value::String("zscore_signal".to_string());
        payload["strategy_ir"]["signals"][0]["name"] =
            serde_json::Value::String("ZScore Signal".to_string());
        payload["strategy_ir"]["signals"][0]["indicator"]["kind"] =
            serde_json::Value::String("z_score".to_string());
        payload["strategy_ir"]["signals"][0]["indicator"]["params"] = serde_json::json!({
            "window": 20
        });
        payload["strategy_ir"]["logic"]["entry_rules"][0]["condition"] =
            serde_json::Value::String("zscore_signal < -2".to_string());
        payload["strategy_ir"]["logic"]["entry_rules"][0]["action"] =
            serde_json::Value::String("open_long".to_string());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/strategy-ir/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["core_ir"]["indicators"][0]["kind"], "z_score");
        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"],
            serde_json::json!({
                "kind": "compare",
                "left": {
                    "kind": "ref",
                    "name": "zscore_signal"
                },
                "op": "lt",
                "right": {
                    "kind": "number",
                    "value": -2.0
                }
            })
        );
    }

    #[tokio::test]
    async fn strategy_ir_compile_endpoint_lowers_spread_to_structured_threshold_condition() {
        let value = compile_strategy_ir_spread_for_test(
            serde_json::json!({
                "align_direction_code": 0,
                "max_time_diff_ms": 5000,
                "spread_output_code": 1
            }),
            "spread_signal > 5",
            "compile_strategy_ir_spread_threshold",
        )
        .await;

        assert_eq!(value["core_ir"]["indicators"][0]["kind"], "spread");
        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"],
            serde_json::json!({
                "kind": "compare",
                "left": {
                    "kind": "ref",
                    "name": "spread_signal"
                },
                "op": "gt",
                "right": {
                    "kind": "number",
                    "value": 5.0
                }
            })
        );
    }

    #[tokio::test]
    async fn strategy_ir_compile_endpoint_rejects_spread_with_non_bps_output() {
        let value = compile_strategy_ir_spread_error_for_test(
            serde_json::json!({
                "align_direction_code": 0,
                "max_time_diff_ms": 5000,
                "spread_output_code": 0
            }),
            "spread_signal > 5",
            "compile_strategy_ir_spread_non_bps_reject",
        )
        .await;

        assert_eq!(value["error"], "strategy_ir_compile_failed");
        assert_eq!(value["details"][0]["code"], "QPSTRATSPREAD001");
    }

    #[tokio::test]
    async fn strategy_ir_compile_endpoint_rejects_spread_with_non_positive_tolerance() {
        let value = compile_strategy_ir_spread_error_for_test(
            serde_json::json!({
                "align_direction_code": 0,
                "max_time_diff_ms": 0,
                "spread_output_code": 1
            }),
            "spread_signal > 5",
            "compile_strategy_ir_spread_bad_tolerance_reject",
        )
        .await;

        assert_eq!(value["error"], "strategy_ir_compile_failed");
        assert_eq!(value["details"][0]["code"], "QPSTRATSPREAD002");
    }

    #[tokio::test]
    async fn strategy_ir_compile_endpoint_rejects_spread_with_non_one_sided_shape() {
        let value = compile_strategy_ir_spread_error_for_test(
            serde_json::json!({
                "align_direction_code": 0,
                "max_time_diff_ms": 5000,
                "spread_output_code": 1
            }),
            "spread_signal < -5",
            "compile_strategy_ir_spread_bad_shape_reject",
        )
        .await;

        assert_eq!(value["error"], "strategy_ir_compile_failed");
        assert_eq!(value["details"][0]["code"], "QPSTRATSPREAD003");
    }

    #[tokio::test]
    async fn spread_bps_condition_lowers_equivalently_across_graph_and_strategy_ir() {
        let graph = compile_runtime_spread_graph_for_test(
            serde_json::json!({
                "max_time_diff_ms": 5000,
                "field_code": 0,
                "align_direction_code": 0,
                "resample_period_ms": 0,
                "resample_agg_code": 0,
                "window_size": 1,
                "window_agg_code": 1,
                "spread_output_code": 1,
                "comparison_shape_code": 1,
                "comparison_op_code": 2,
                "comparison_threshold": 5.0
            }),
            "compile_graph_spread_equivalence",
        )
        .await;
        let strategy = compile_strategy_ir_spread_for_test(
            serde_json::json!({
                "align_direction_code": 0,
                "max_time_diff_ms": 5000,
                "spread_output_code": 1
            }),
            "spread_signal > 5",
            "compile_strategy_ir_spread_equivalence",
        )
        .await;

        let graph_view = core_ir_entry_equivalence_view(&graph["artifacts"]["core_ir"]["core_ir"]);
        let strategy_view = core_ir_entry_equivalence_view(&strategy["core_ir"]);

        assert_eq!(graph_view, strategy_view);
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_lowers_admitted_spread_to_structured_threshold_condition(
    ) {
        let value = compile_formal_quantscript_for_test(
            r#"
fn strategy() {
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let left_aligned = align_asof(field(left, name="bid"), direction="backward", tolerance_ms=5000)
    let right_aligned = align_asof(field(right, name="ask"), direction="backward", tolerance_ms=5000)
    let spread_signal = spread(left_aligned, right_aligned, output="bps")
    if spread_signal > 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_spread_threshold",
        )
        .await;

        assert_eq!(value["core_ir"]["indicators"][0]["kind"], "spread");
        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"],
            serde_json::json!({
                "kind": "compare",
                "left": {
                    "kind": "ref",
                    "name": "intent_btcusdt_spread"
                },
                "op": "gt",
                "right": {
                    "kind": "number",
                    "value": 5.0
                }
            })
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_rejects_spread_with_non_bps_output() {
        let value = compile_formal_quantscript_error_for_test(
            r#"
fn strategy() {
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let left_aligned = align_asof(field(left, name="bid"), direction="backward", tolerance_ms=5000)
    let right_aligned = align_asof(field(right, name="ask"), direction="backward", tolerance_ms=5000)
    let spread_signal = spread(left_aligned, right_aligned, output="ratio")
    if spread_signal > 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_spread_non_bps",
        )
        .await;

        assert_eq!(value["error"], "quantscript_lowering_failed");
        assert_eq!(
            api_error_detail_by_code(&value, "QPQSLOW001")["code"],
            "QPQSLOW001"
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_rejects_spread_without_explicit_align_asof() {
        let value = compile_formal_quantscript_error_for_test(
            r#"
fn strategy() {
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let spread_signal = spread(field(left, name="bid"), field(right, name="ask"), output="bps")
    if spread_signal > 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_spread_missing_align_asof",
        )
        .await;

        assert_eq!(value["error"], "quantscript_lowering_failed");
        assert_eq!(
            api_error_detail_by_code(&value, "QPQSLOW001")["code"],
            "QPQSLOW001"
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_rejects_spread_with_non_positive_tolerance() {
        let value = compile_formal_quantscript_error_for_test(
            r#"
fn strategy() {
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let left_aligned = align_asof(field(left, name="bid"), direction="backward", tolerance_ms=0)
    let right_aligned = align_asof(field(right, name="ask"), direction="backward", tolerance_ms=0)
    let spread_signal = spread(left_aligned, right_aligned, output="bps")
    if spread_signal > 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_spread_non_positive_tolerance",
        )
        .await;

        assert_eq!(value["error"], "quantscript_lowering_failed");
        assert_eq!(
            api_error_detail_by_code(&value, "QPQSLOW001")["code"],
            "QPQSLOW001"
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_rejects_spread_with_non_one_sided_shape() {
        let value = compile_formal_quantscript_error_for_test(
            r#"
fn strategy() {
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let left_aligned = align_asof(field(left, name="bid"), direction="backward", tolerance_ms=5000)
    let right_aligned = align_asof(field(right, name="ask"), direction="backward", tolerance_ms=5000)
    let spread_signal = spread(left_aligned, right_aligned, output="bps")
    if spread_signal < 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_spread_non_one_sided",
        )
        .await;

        assert_eq!(value["error"], "quantscript_lowering_failed");
        assert_eq!(
            api_error_detail_by_code(&value, "QPQSLOW001")["code"],
            "QPQSLOW001"
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_matches_spread_non_bps_diagnostic_golden_view() {
        let value = compile_formal_quantscript_error_for_test(
            r#"
fn strategy() {
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let left_aligned = align_asof(field(left, name="bid"), direction="backward", tolerance_ms=5000)
    let right_aligned = align_asof(field(right, name="ask"), direction="backward", tolerance_ms=5000)
    let spread_signal = spread(left_aligned, right_aligned, output="ratio")
    if spread_signal > 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_spread_non_bps_golden",
        )
        .await;

        assert_eq!(
            formal_compile_error_golden_view(&value),
            expected_formal_spread_rejection_golden_view()
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_matches_spread_missing_align_diagnostic_golden_view(
    ) {
        let value = compile_formal_quantscript_error_for_test(
            r#"
fn strategy() {
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let spread_signal = spread(field(left, name="bid"), field(right, name="ask"), output="bps")
    if spread_signal > 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_spread_missing_align_golden",
        )
        .await;

        assert_eq!(
            formal_compile_error_golden_view(&value),
            expected_formal_spread_rejection_golden_view()
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_matches_spread_non_positive_tolerance_diagnostic_golden_view(
    ) {
        let value = compile_formal_quantscript_error_for_test(
            r#"
fn strategy() {
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let left_aligned = align_asof(field(left, name="bid"), direction="backward", tolerance_ms=0)
    let right_aligned = align_asof(field(right, name="ask"), direction="backward", tolerance_ms=0)
    let spread_signal = spread(left_aligned, right_aligned, output="bps")
    if spread_signal > 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_spread_non_positive_tolerance_golden",
        )
        .await;

        assert_eq!(
            formal_compile_error_golden_view(&value),
            expected_formal_spread_rejection_golden_view()
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_matches_spread_non_one_sided_diagnostic_golden_view(
    ) {
        let value = compile_formal_quantscript_error_for_test(
            r#"
fn strategy() {
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let left_aligned = align_asof(field(left, name="bid"), direction="backward", tolerance_ms=5000)
    let right_aligned = align_asof(field(right, name="ask"), direction="backward", tolerance_ms=5000)
    let spread_signal = spread(left_aligned, right_aligned, output="bps")
    if spread_signal < 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_spread_non_one_sided_golden",
        )
        .await;

        assert_eq!(
            formal_compile_error_golden_view(&value),
            expected_formal_spread_rejection_golden_view()
        );
    }

    #[tokio::test]
    async fn spread_bps_condition_lowers_equivalently_across_formal_graph_and_strategy_ir() {
        let formal = compile_formal_quantscript_for_test(
            r#"
fn strategy() {
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let left_aligned = align_asof(field(left, name="bid"), direction="backward", tolerance_ms=5000)
    let right_aligned = align_asof(field(right, name="ask"), direction="backward", tolerance_ms=5000)
    let spread_signal = spread(left_aligned, right_aligned, output="bps")
    if spread_signal > 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_spread_equivalence",
        )
        .await;
        let graph = compile_runtime_spread_graph_for_test(
            serde_json::json!({
                "max_time_diff_ms": 5000,
                "field_code": 0,
                "align_direction_code": 0,
                "resample_period_ms": 0,
                "resample_agg_code": 0,
                "window_size": 1,
                "window_agg_code": 1,
                "spread_output_code": 1,
                "comparison_shape_code": 1,
                "comparison_op_code": 2,
                "comparison_threshold": 5.0
            }),
            "compile_graph_spread_formal_equivalence",
        )
        .await;
        let strategy = compile_strategy_ir_spread_for_test(
            serde_json::json!({
                "align_direction_code": 0,
                "max_time_diff_ms": 5000,
                "spread_output_code": 1
            }),
            "spread_signal > 5",
            "compile_strategy_ir_spread_formal_equivalence",
        )
        .await;

        let formal_view = core_ir_entry_equivalence_view(&formal["core_ir"]);
        let graph_view = core_ir_entry_equivalence_view(&graph["artifacts"]["core_ir"]["core_ir"]);
        let strategy_view = core_ir_entry_equivalence_view(&strategy["core_ir"]);

        assert_eq!(formal_view, graph_view);
        assert_eq!(formal_view, strategy_view);
    }

    #[tokio::test]
    async fn one_sided_rsi_condition_lowers_equivalently_across_formal_graph_and_strategy_ir() {
        let formal = compile_formal_quantscript_for_test(
            r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let r = rsi(data_feed, 14)
    if r < 25 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_rsi_equivalence",
        )
        .await;
        let graph = compile_runtime_graph_for_test(
            "builtin.intent.rsi",
            serde_json::json!({
                "period": 14,
                "oversold_threshold": 25.0,
                "overbought_threshold": 70.0
            }),
            "compile_graph_rsi_equivalence",
        )
        .await;
        let strategy = compile_strategy_ir_for_test(
            "rsi_signal",
            "RSI Signal",
            "rsi",
            serde_json::json!({ "period": 14 }),
            "rsi_signal < 25",
            "compile_strategy_ir_rsi_equivalence",
        )
        .await;

        let formal_view = core_ir_entry_equivalence_view(&formal["core_ir"]);
        let graph_view = core_ir_entry_equivalence_view(&graph["artifacts"]["core_ir"]["core_ir"]);
        let strategy_view = core_ir_entry_equivalence_view(&strategy["core_ir"]);

        assert_eq!(formal_view, graph_view);
        assert_eq!(formal_view, strategy_view);
    }

    #[tokio::test]
    async fn one_sided_momentum_condition_lowers_equivalently_across_formal_graph_and_strategy_ir()
    {
        let formal = compile_formal_quantscript_for_test(
            r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let m = momentum(data_feed, 20)
    if m > 0.03 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_momentum_equivalence",
        )
        .await;
        let graph = compile_runtime_graph_for_test(
            "builtin.intent.momentum",
            serde_json::json!({
                "lookback": 20,
                "threshold_ratio": 0.03
            }),
            "compile_graph_momentum_equivalence",
        )
        .await;
        let strategy = compile_strategy_ir_for_test(
            "momentum_signal",
            "Momentum Signal",
            "momentum",
            serde_json::json!({ "lookback": 20 }),
            "momentum_signal > 0.03",
            "compile_strategy_ir_momentum_equivalence",
        )
        .await;

        let formal_view = core_ir_entry_equivalence_view(&formal["core_ir"]);
        let graph_view = core_ir_entry_equivalence_view(&graph["artifacts"]["core_ir"]["core_ir"]);
        let strategy_view = core_ir_entry_equivalence_view(&strategy["core_ir"]);

        assert_eq!(formal_view, graph_view);
        assert_eq!(formal_view, strategy_view);
    }

    #[tokio::test]
    async fn one_sided_zscore_condition_lowers_equivalently_across_formal_graph_and_strategy_ir() {
        let formal = compile_formal_quantscript_for_test(
            r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let z = zscore(data_feed, 20)
    if z < -2 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_zscore_equivalence",
        )
        .await;
        let graph = compile_runtime_graph_for_test(
            "builtin.intent.zscore",
            serde_json::json!({
                "window": 20,
                "entry_z": 2.0
            }),
            "compile_graph_zscore_equivalence",
        )
        .await;
        let strategy = compile_strategy_ir_for_test(
            "zscore_signal",
            "ZScore Signal",
            "z_score",
            serde_json::json!({ "window": 20 }),
            "zscore_signal < -2",
            "compile_strategy_ir_zscore_equivalence",
        )
        .await;

        let formal_view = core_ir_entry_equivalence_view(&formal["core_ir"]);
        let graph_view = core_ir_entry_equivalence_view(&graph["artifacts"]["core_ir"]["core_ir"]);
        let strategy_view = core_ir_entry_equivalence_view(&strategy["core_ir"]);

        assert_eq!(formal_view, graph_view);
        assert_eq!(formal_view, strategy_view);
    }

    #[tokio::test]
    async fn direct_ma_condition_lowers_equivalently_across_formal_graph_and_strategy_ir() {
        let formal = compile_formal_quantscript_for_test(
            r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let fast = sma(data_feed, 20)
    let slow = sma(data_feed, 100)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_direct_ma_equivalence",
        )
        .await;
        let graph = compile_runtime_graph_for_test(
            "builtin.intent.double_ma",
            serde_json::json!({
                "fast_period": 20,
                "slow_period": 100,
                "entry_ratio": 1.0
            }),
            "compile_graph_direct_ma_equivalence",
        )
        .await;
        let strategy = compile_strategy_ir_for_test(
            "ma_cross",
            "MA Cross",
            "ma_cross",
            serde_json::json!({
                "fast": 20,
                "slow": 100
            }),
            "ma_cross > 0",
            "compile_strategy_ir_direct_ma_equivalence",
        )
        .await;

        let formal_view = core_ir_entry_equivalence_view(&formal["core_ir"]);
        let graph_view = core_ir_entry_equivalence_view(&graph["artifacts"]["core_ir"]["core_ir"]);
        let strategy_view = core_ir_entry_equivalence_view(&strategy["core_ir"]);

        assert_eq!(formal_view["indicator_kind"], graph_view["indicator_kind"]);
        assert_eq!(formal_view["indicator_kind"], strategy_view["indicator_kind"]);
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_lowers_global_risk_profile_to_runtime_global_risk_node(
    ) {
        let value = compile_formal_quantscript_for_test(
            r#"
fn strategy() {
    risk.profile("global", max_position=0.35, max_total_leverage=4.0, max_exchange_leverage=5.0, min_action_interval_ms=250)
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let r = rsi(data_feed, 14)
    if r < 25 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_risk_profile_global",
        )
        .await;

        assert_eq!(
            value["runtime_config"]["risk_controls"][0]["module_key"],
            "builtin.risk.global"
        );
        assert_eq!(
            value["runtime_config"]["risk_controls"][0]["config"]["profile_id"],
            "global"
        );
        assert_eq!(
            value["core_ir"]["risk_policies"][0]["max_position_ratio"],
            serde_json::json!(0.35)
        );
        assert_eq!(
            value["core_ir"]["risk_policies"][0]["max_total_leverage"],
            serde_json::json!(4.0)
        );
        assert_eq!(
            value["core_ir"]["risk_policies"][0]["max_exchange_leverage"],
            serde_json::json!(5.0)
        );
        assert_eq!(
            value["core_ir"]["risk_policies"][0]["min_action_interval_ms"],
            serde_json::json!(250)
        );
    }

    #[tokio::test]
    async fn global_risk_profile_lowers_equivalently_across_formal_graph_and_strategy_ir() {
        let formal = compile_formal_quantscript_for_test(
            r#"
fn strategy() {
    risk.profile("global", max_position=0.35, max_total_leverage=4.0, max_exchange_leverage=5.0, min_action_interval_ms=250)
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let m = momentum(data_feed, 20)
    if m > 0.03 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_risk_profile_equivalence",
        )
        .await;

        let graph_app = build_app_router(test_app_state());
        let mut graph_payload = sample_compile_request_json();
        graph_payload["compile_id"] =
            serde_json::Value::String("compile_graph_risk_profile_equivalence".to_string());
        graph_payload["runtime_config"]["intent_generators"][0]["module_key"] =
            serde_json::Value::String("builtin.intent.momentum".to_string());
        graph_payload["runtime_config"]["intent_generators"][0]["config"] = serde_json::json!({
            "lookback": 20,
            "threshold": 0.03
        });
        graph_payload["runtime_config"]["risk_controls"][0]["config"] = serde_json::json!({
            "profile_id": "global",
            "max_position": 0.35,
            "max_total_leverage": 4.0,
            "max_exchange_leverage": 5.0,
            "min_action_interval_ms": 250
        });

        let graph_response = graph_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(graph_payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(graph_response.status(), StatusCode::OK);
        let graph_body = to_bytes(graph_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let graph: serde_json::Value = serde_json::from_slice(&graph_body).unwrap();

        let mut strategy_payload = sample_strategy_ir_compile_request_json();
        strategy_payload["compile_id"] =
            serde_json::Value::String("compile_strategy_ir_risk_profile_equivalence".to_string());
        strategy_payload["strategy_ir"]["signals"][0]["signal_id"] =
            serde_json::Value::String("momentum_signal".to_string());
        strategy_payload["strategy_ir"]["signals"][0]["name"] =
            serde_json::Value::String("Momentum Signal".to_string());
        strategy_payload["strategy_ir"]["signals"][0]["indicator"]["kind"] =
            serde_json::Value::String("momentum".to_string());
        strategy_payload["strategy_ir"]["signals"][0]["indicator"]["params"] = serde_json::json!({
            "lookback": 20
        });
        strategy_payload["strategy_ir"]["logic"]["entry_rules"][0]["condition"] =
            serde_json::Value::String("momentum_signal > 0.03".to_string());
        strategy_payload["strategy_ir"]["logic"]["entry_rules"][0]["action"] =
            serde_json::Value::String("open_long".to_string());
        strategy_payload["strategy_ir"]["risk_profile"] = serde_json::json!({
            "profile_id": "global",
            "max_position": 0.35,
            "max_total_leverage": 4.0,
            "max_exchange_leverage": 5.0,
            "min_action_interval_ms": 250
        });

        let strategy_app = build_app_router(test_app_state());
        let strategy_response = strategy_app
            .oneshot(
                Request::builder()
                    .uri("/api/strategy-ir/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(strategy_payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(strategy_response.status(), StatusCode::OK);
        let strategy_body = to_bytes(strategy_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let strategy: serde_json::Value = serde_json::from_slice(&strategy_body).unwrap();

        let formal_view = core_ir_risk_profile_equivalence_view(&formal["core_ir"]);
        let graph_view = core_ir_risk_profile_equivalence_view(&graph["core_ir"]);
        let strategy_view = core_ir_risk_profile_equivalence_view(&strategy["core_ir"]);

        assert_eq!(
            formal["runtime_config"]["risk_controls"][0]["config"]["profile_id"],
            "global"
        );
        assert_eq!(
            graph["runtime_config"]["risk_controls"][0]["config"]["profile_id"],
            "global"
        );
        assert_eq!(formal_view, graph_view);
        assert_eq!(formal_view, strategy_view);
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_lowers_paper_execution_profile_to_runtime_execution_node(
    ) {
        let value = compile_formal_quantscript_for_test(
            r#"
fn strategy() {
    execution.profile("paper", fee_bps=12.5, slippage_bps=7.5)
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let m = momentum(data_feed, 20)
    if m > 0.03 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_execution_profile_paper",
        )
        .await;

        assert_eq!(
            value["runtime_config"]["executions"][0]["module_key"],
            "builtin.execution.paper"
        );
        assert_eq!(
            value["runtime_config"]["executions"][0]["config"]["profile_id"],
            "paper"
        );
        assert_eq!(
            value["runtime_config"]["executions"][0]["config"]["fee_bps"],
            serde_json::json!(12.5)
        );
        assert_eq!(
            value["runtime_config"]["executions"][0]["config"]["slippage_bps"],
            serde_json::json!(7.5)
        );
        assert_eq!(value["core_ir"]["execution"]["venue_kind"], "paper");
        assert_eq!(
            value["core_ir"]["execution"]["taker_fee_bps"],
            serde_json::json!(12.5)
        );
        assert_eq!(
            value["core_ir"]["execution"]["slippage_bps"],
            serde_json::json!(7.5)
        );
    }

    #[tokio::test]
    async fn paper_execution_profile_lowers_equivalently_across_formal_graph_and_strategy_ir() {
        let formal = compile_formal_quantscript_for_test(
            r#"
fn strategy() {
    execution.profile("paper", fee_bps=12.5, slippage_bps=7.5)
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let m = momentum(data_feed, 20)
    if m > 0.03 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_execution_profile_equivalence",
        )
        .await;

        let graph_app = build_app_router(test_app_state());
        let mut graph_payload = sample_compile_request_json();
        graph_payload["compile_id"] =
            serde_json::Value::String("compile_graph_execution_profile_equivalence".to_string());
        graph_payload["runtime_config"]["intent_generators"][0]["module_key"] =
            serde_json::Value::String("builtin.intent.momentum".to_string());
        graph_payload["runtime_config"]["intent_generators"][0]["config"] = serde_json::json!({
            "lookback": 20,
            "threshold": 0.03
        });
        graph_payload["runtime_config"]["executions"][0]["config"] = serde_json::json!({
            "profile_id": "paper",
            "mode": "paper",
            "fee_bps": 12.5,
            "slippage_bps": 7.5
        });

        let graph_response = graph_app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(graph_payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(graph_response.status(), StatusCode::OK);
        let graph_body = to_bytes(graph_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let graph: serde_json::Value = serde_json::from_slice(&graph_body).unwrap();

        let mut strategy_payload = sample_strategy_ir_compile_request_json();
        strategy_payload["compile_id"] = serde_json::Value::String(
            "compile_strategy_ir_execution_profile_equivalence".to_string(),
        );
        strategy_payload["strategy_ir"]["signals"][0]["signal_id"] =
            serde_json::Value::String("momentum_signal".to_string());
        strategy_payload["strategy_ir"]["signals"][0]["name"] =
            serde_json::Value::String("Momentum Signal".to_string());
        strategy_payload["strategy_ir"]["signals"][0]["indicator"]["kind"] =
            serde_json::Value::String("momentum".to_string());
        strategy_payload["strategy_ir"]["signals"][0]["indicator"]["params"] =
            serde_json::json!({ "lookback": 20 });
        strategy_payload["strategy_ir"]["logic"]["entry_rules"][0]["condition"] =
            serde_json::Value::String("momentum_signal > 0.03".to_string());
        strategy_payload["strategy_ir"]["logic"]["entry_rules"][0]["action"] =
            serde_json::Value::String("open_long".to_string());
        strategy_payload["strategy_ir"]["execution_profile"] = serde_json::json!({
            "profile_id": "paper",
            "fee_bps": 12.5,
            "slippage_bps": 7.5
        });

        let strategy_response = graph_app
            .oneshot(
                Request::builder()
                    .uri("/api/strategy-ir/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(strategy_payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(strategy_response.status(), StatusCode::OK);
        let strategy_body = to_bytes(strategy_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let strategy: serde_json::Value = serde_json::from_slice(&strategy_body).unwrap();

        let formal_view = core_ir_execution_profile_equivalence_view(&formal["core_ir"]);
        let graph_view = core_ir_execution_profile_equivalence_view(&graph["core_ir"]);
        let strategy_view = core_ir_execution_profile_equivalence_view(&strategy["core_ir"]);

        assert_eq!(
            formal["runtime_config"]["executions"][0]["config"]["profile_id"],
            "paper"
        );
        assert_eq!(
            graph["runtime_config"]["executions"][0]["config"]["profile_id"],
            "paper"
        );
        assert_eq!(formal_view, graph_view);
        assert_eq!(formal_view, strategy_view);
    }

    #[tokio::test]
    async fn strategy_ir_compile_endpoint_returns_structured_custom_diagnostics() {
        let app = build_app_router(test_app_state());
        let mut payload = sample_strategy_ir_compile_request_json();
        payload["strategy_ir"]["signals"][0]["indicator"]["params"]["custom_expr"]["predicate"]
            ["left"]["data_id"] = serde_json::Value::String("other_data".to_string());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/strategy-ir/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["error"], "strategy_ir_compile_failed");
        assert_eq!(value["details"][0]["code"], "CUSTOM006");
        assert_eq!(
            value["details"][0]["target"],
            "params.custom_expr"
        );
        assert!(value["details"][0]["message"]
            .as_str()
            .unwrap()
            .contains("未声明的输入"));
        assert!(value["details"][0]["reason"]
            .as_str()
            .unwrap()
            .contains("Custom indicators are restricted"));
    }

    #[tokio::test]
    async fn backtest_endpoint_returns_output_artifacts() {
        let app = build_app_router(test_app_state());
        let mut payload = sample_compile_request_json();
        payload["backtest_options"] = serde_json::json!({
            "replay_source": "deterministic_mock"
        });

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/backtest")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            value["backtest_artifacts"]["manifest"]["backtest_spec"]["schema_version"],
            "quantpilot/backtest-spec/v1"
        );
        assert_eq!(
            value["backtest_artifacts"]["manifest"]["backtest_spec"]["replay_source"],
            "deterministic_mock"
        );
        assert_eq!(
            value["backtest_artifacts"]["manifest"]["backtest_spec"]["run_spec"]["config_hash"],
            value["config_hash"]
        );
        assert_eq!(
            value["backtest_artifacts"]["manifest"]["compile_artifacts"]["compile"]["config_hash"],
            value["config_hash"]
        );
        assert_eq!(
            value["backtest_artifacts"]["manifest"]["compile_artifacts"]["core_ir"]["digest"]
                ["value"],
            value["backtest_artifacts"]["manifest"]["backtest_spec"]["run_spec"]["core_ir_digest"]
                ["value"]
        );
        assert!(value["backtest_artifacts"]["metrics"]["summary"].is_object());
        assert!(value["backtest_artifacts"]["manifest"]["output_artifacts"].is_array());
    }

    #[tokio::test]
    async fn runtime_run_is_persisted_only_after_save() {
        let base = test_storage_base("run-save-gate");
        let graph_dir = base.join("graphs");
        let run_dir = base.join("runs");
        let backtest_dir = base.join("backtests");
        std::fs::create_dir_all(&graph_dir).unwrap();
        std::fs::create_dir_all(&run_dir).unwrap();
        std::fs::create_dir_all(&backtest_dir).unwrap();
        let app = build_app_router(test_app_state_from_dirs(
            base,
            graph_dir,
            run_dir.clone(),
            backtest_dir,
        ));
        let payload = sample_compile_request_json();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/test-run")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let run_id = created["run_id"].as_str().unwrap().to_string();
        let run_path = run_dir.join(format!("{run_id}.json"));

        assert!(!run_path.exists());

        let save_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/runtime/runs/{run_id}/save"))
                    .method("POST")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(save_response.status(), StatusCode::OK);
        assert!(run_path.exists());
    }

    #[tokio::test]
    async fn runtime_run_can_be_discarded_only_before_save() {
        let base = test_storage_base("run-discard");
        let graph_dir = base.join("graphs");
        let run_dir = base.join("runs");
        let backtest_dir = base.join("backtests");
        std::fs::create_dir_all(&graph_dir).unwrap();
        std::fs::create_dir_all(&run_dir).unwrap();
        std::fs::create_dir_all(&backtest_dir).unwrap();
        let app = build_app_router(test_app_state_from_dirs(
            base,
            graph_dir,
            run_dir.clone(),
            backtest_dir,
        ));
        let payload = sample_compile_request_json();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/test-run")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let started: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let run_id = started["run_id"].as_str().unwrap().to_string();

        let discard_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/runtime/runs/{run_id}"))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(discard_response.status(), StatusCode::OK);

        let detail_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/runtime/runs/{run_id}"))
                    .method("GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(detail_response.status(), StatusCode::NOT_FOUND);

        let saved_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/test-run")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let saved_body = to_bytes(saved_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let saved: serde_json::Value = serde_json::from_slice(&saved_body).unwrap();
        let saved_run_id = saved["run_id"].as_str().unwrap().to_string();
        let save_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/runtime/runs/{saved_run_id}/save"))
                    .method("POST")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(save_response.status(), StatusCode::OK);

        let discard_saved_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/runtime/runs/{saved_run_id}"))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(discard_saved_response.status(), StatusCode::CONFLICT);
        assert!(run_dir.join(format!("{saved_run_id}.json")).exists());
    }

    #[tokio::test]
    async fn backtest_detail_can_be_reloaded_from_artifact_directory() {
        let base = test_storage_base("backtest-artifacts");
        let graph_dir = base.join("graphs");
        let run_dir = base.join("runs");
        let backtest_dir = base.join("backtests");
        std::fs::create_dir_all(&graph_dir).unwrap();
        std::fs::create_dir_all(&run_dir).unwrap();
        std::fs::create_dir_all(&backtest_dir).unwrap();

        let app = build_app_router(test_app_state_from_dirs(
            base,
            graph_dir.clone(),
            run_dir.clone(),
            backtest_dir.clone(),
        ));
        let mut payload = sample_compile_request_json();
        payload["backtest_options"] = serde_json::json!({
            "replay_source": "deterministic_mock"
        });

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/backtest")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let backtest_id = created["backtest_id"].as_str().unwrap().to_string();

        assert!(!backtest_dir.join(&backtest_id).exists());

        let save_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/runtime/backtests/{backtest_id}/save"))
                    .method("POST")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(save_response.status(), StatusCode::OK);

        assert!(backtest_dir
            .join(&backtest_id)
            .join("manifest.json")
            .exists());
        assert!(backtest_dir
            .join(&backtest_id)
            .join("trade_ledger.json")
            .exists());
        assert!(backtest_dir
            .join(&backtest_id)
            .join("equity_curve.json")
            .exists());
        assert!(std::fs::read_dir(&backtest_dir)
            .unwrap()
            .all(|entry| !is_backtest_promotion_work_dir(&entry.unwrap().path())));

        let second_save_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/runtime/backtests/{backtest_id}/save"))
                    .method("POST")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(second_save_response.status(), StatusCode::OK);
        assert!(backtest_dir
            .join(&backtest_id)
            .join("manifest.json")
            .exists());
        assert!(std::fs::read_dir(&backtest_dir)
            .unwrap()
            .all(|entry| !is_backtest_promotion_work_dir(&entry.unwrap().path())));

        let fresh_app = build_app_router(new_app_state(graph_dir, run_dir, backtest_dir));
        let detail_response = fresh_app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/runtime/backtests/{backtest_id}"))
                    .method("GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(detail_response.status(), StatusCode::OK);
        let detail_body = to_bytes(detail_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let detail: serde_json::Value = serde_json::from_slice(&detail_body).unwrap();

        assert_eq!(detail["backtest_id"], backtest_id);
        assert!(detail["backtest_artifacts"]["trade_ledger"]["trades"].is_array());
        assert!(detail["backtest_artifacts"]["equity_curve"]["points"].is_array());
        assert_eq!(
            detail["backtest_artifacts"]["metrics"]["summary"]["final_equity"],
            created["backtest_artifacts"]["metrics"]["summary"]["final_equity"]
        );
    }

    #[tokio::test]
    async fn large_transient_backtest_spills_to_temp_until_save() {
        let base = test_storage_base("backtest-transient-spill");
        let graph_dir = base.join("graphs");
        let run_dir = base.join("runs");
        let backtest_dir = base.join("backtests");
        std::fs::create_dir_all(&graph_dir).unwrap();
        std::fs::create_dir_all(&run_dir).unwrap();
        std::fs::create_dir_all(&backtest_dir).unwrap();

        let mut state = test_app_state_from_dirs(base, graph_dir, run_dir, backtest_dir.clone());
        state.transient_backtest_spill_threshold_bytes = 1;
        let transient_dir = state.transient_backtest_store_dir.as_ref().clone();
        let app = build_app_router(state);
        let mut payload = sample_compile_request_json();
        payload["backtest_options"] = serde_json::json!({
            "replay_source": "deterministic_mock"
        });

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/backtest")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let backtest_id = created["backtest_id"].as_str().unwrap().to_string();

        assert!(!backtest_dir.join(&backtest_id).exists());
        assert!(std::fs::read_dir(&transient_dir)
            .unwrap()
            .any(|entry| entry.unwrap().path().is_dir()));

        let detail_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/runtime/backtests/{backtest_id}"))
                    .method("GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(detail_response.status(), StatusCode::OK);
        let detail_body = to_bytes(detail_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let detail: serde_json::Value = serde_json::from_slice(&detail_body).unwrap();
        let mut expected_loaded_governance =
            created["backtest_artifacts"]["manifest"]["governance"].clone();
        expected_loaded_governance["governance_source"] =
            serde_json::Value::String("loaded_manifest".to_string());
        assert_eq!(detail["governance"], expected_loaded_governance);
        assert_eq!(
            detail["backtest_artifacts"]["manifest"]["governance"],
            detail["governance"]
        );

        let save_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/runtime/backtests/{backtest_id}/save"))
                    .method("POST")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(save_response.status(), StatusCode::OK);
        let save_body = to_bytes(save_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let saved: serde_json::Value = serde_json::from_slice(&save_body).unwrap();
        assert_eq!(saved["governance"], detail["governance"]);
        assert_eq!(
            saved["backtest_artifacts"]["manifest"]["governance"],
            detail["governance"]
        );
        assert!(backtest_dir
            .join(&backtest_id)
            .join("manifest.json")
            .exists());
        assert!(std::fs::read_dir(&transient_dir)
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(true));
    }

    #[tokio::test]
    async fn backtest_can_be_discarded_only_before_save() {
        let base = test_storage_base("backtest-discard");
        let graph_dir = base.join("graphs");
        let run_dir = base.join("runs");
        let backtest_dir = base.join("backtests");
        std::fs::create_dir_all(&graph_dir).unwrap();
        std::fs::create_dir_all(&run_dir).unwrap();
        std::fs::create_dir_all(&backtest_dir).unwrap();

        let mut state = test_app_state_from_dirs(base, graph_dir, run_dir, backtest_dir.clone());
        state.transient_backtest_spill_threshold_bytes = 1;
        let transient_dir = state.transient_backtest_store_dir.as_ref().clone();
        let app = build_app_router(state);
        let mut payload = sample_compile_request_json();
        payload["backtest_options"] = serde_json::json!({
            "replay_source": "deterministic_mock"
        });

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/backtest")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let backtest_id = created["backtest_id"].as_str().unwrap().to_string();

        let discard_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/runtime/backtests/{backtest_id}"))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(discard_response.status(), StatusCode::OK);
        assert!(std::fs::read_dir(&transient_dir)
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(true));

        let detail_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/runtime/backtests/{backtest_id}"))
                    .method("GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(detail_response.status(), StatusCode::NOT_FOUND);

        let saved_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/runtime/backtest")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let saved_body = to_bytes(saved_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let saved: serde_json::Value = serde_json::from_slice(&saved_body).unwrap();
        let saved_backtest_id = saved["backtest_id"].as_str().unwrap().to_string();
        let save_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/runtime/backtests/{saved_backtest_id}/save"))
                    .method("POST")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(save_response.status(), StatusCode::OK);

        let discard_saved_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/runtime/backtests/{saved_backtest_id}"))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(discard_saved_response.status(), StatusCode::CONFLICT);
        assert!(backtest_dir
            .join(&saved_backtest_id)
            .join("manifest.json")
            .exists());
    }

    #[tokio::test]
    async fn graphs_endpoint_lists_saved_graph_files_only() {
        let base = test_storage_base("graph-index");
        let graph_dir = base.join("graphs");
        let run_dir = base.join("runs");
        let backtest_dir = base.join("backtests");
        std::fs::create_dir_all(&graph_dir).unwrap();
        std::fs::create_dir_all(&run_dir).unwrap();
        std::fs::create_dir_all(&backtest_dir).unwrap();

        std::fs::write(
            graph_dir.join("alpha_strategy.json"),
            serde_json::json!({
                "metadata": {
                    "graph_id": "alpha_strategy",
                    "name": "Alpha strategy",
                    "updated_at": 1710000000000u64,
                    "artifacts": {
                        "quantscript": {
                            "saved_path": graph_dir.join("alpha_strategy.qs").to_string_lossy().to_string()
                        }
                    }
                }
            })
            .to_string(),
        )
        .unwrap();
        std::fs::write(
            graph_dir.join("alpha_strategy.qs"),
            "strategy_graph alpha_strategy {}",
        )
        .unwrap();
        std::fs::write(
            graph_dir.join("beta_strategy.json"),
            serde_json::json!({
                "metadata": {
                    "graph_id": "beta_strategy",
                    "name": "Beta strategy",
                    "updated_at": 1710000200000u64,
                    "artifacts": {
                        "quantscript": {
                            "saved_path": graph_dir.join("beta_strategy.qs").to_string_lossy().to_string()
                        }
                    }
                }
            })
            .to_string(),
        )
        .unwrap();
        std::fs::write(
            graph_dir.join("beta_strategy.qs"),
            "strategy_graph beta_strategy {}",
        )
        .unwrap();
        std::fs::write(
            graph_dir.join("latest.json"),
            serde_json::json!({
                "metadata": {
                    "graph_id": "latest_shadow",
                    "name": "Latest shadow",
                    "updated_at": 1710000300000u64
                }
            })
            .to_string(),
        )
        .unwrap();

        let app = build_app_router(test_app_state_from_dirs(
            base,
            graph_dir,
            run_dir,
            backtest_dir,
        ));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/graphs")
                    .method("GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value.as_array().unwrap().len(), 2);
        assert_eq!(value[0]["graph_id"], "beta_strategy");
        assert_eq!(value[1]["graph_id"], "alpha_strategy");
        assert!(value[0]["path"]
            .as_str()
            .unwrap()
            .ends_with("beta_strategy.qs"));
        assert!(value[1]["path"]
            .as_str()
            .unwrap()
            .ends_with("alpha_strategy.qs"));
        assert!(value
            .as_array()
            .unwrap()
            .iter()
            .all(|entry| entry["graph_id"] != "latest_shadow"));
    }

    #[tokio::test]
    async fn delete_graph_endpoint_removes_strategy_files_and_refreshes_latest() {
        let base = test_storage_base("graph-delete");
        let graph_dir = base.join("graphs");
        let run_dir = base.join("runs");
        let backtest_dir = base.join("backtests");
        std::fs::create_dir_all(&graph_dir).unwrap();
        std::fs::create_dir_all(&run_dir).unwrap();
        std::fs::create_dir_all(&backtest_dir).unwrap();
        std::fs::create_dir_all(graph_dir.join("versions").join("alpha_strategy")).unwrap();

        let alpha = serde_json::json!({
            "metadata": {
                "graph_id": "alpha_strategy",
                "name": "Alpha strategy",
                "updated_at": 1710000000000u64,
                "collaboration": {
                    "owner": {
                        "actor_id": "previous_operator",
                        "display_name": "Previous operator"
                    }
                }
            },
            "nodes": [],
            "edges": []
        });
        let beta = serde_json::json!({
            "metadata": {
                "graph_id": "beta_strategy",
                "name": "Beta strategy",
                "updated_at": 1710000200000u64
            },
            "nodes": [],
            "edges": []
        });
        std::fs::write(graph_dir.join("alpha_strategy.json"), alpha.to_string()).unwrap();
        std::fs::write(graph_dir.join("alpha_strategy.qs"), "strategy alpha() {}").unwrap();
        std::fs::write(
            graph_dir
                .join("versions")
                .join("alpha_strategy")
                .join("1710000000000.json"),
            alpha.to_string(),
        )
        .unwrap();
        std::fs::write(graph_dir.join("beta_strategy.json"), beta.to_string()).unwrap();
        std::fs::write(graph_dir.join("beta_strategy.qs"), "strategy beta() {}").unwrap();
        std::fs::write(graph_dir.join("latest.json"), alpha.to_string()).unwrap();

        let app = build_app_router(test_app_state_from_dirs(
            base,
            graph_dir.clone(),
            run_dir,
            backtest_dir,
        ));
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/graphs/alpha_strategy")
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(!graph_dir.join("alpha_strategy.json").exists());
        assert!(!graph_dir.join("alpha_strategy.qs").exists());
        assert!(!graph_dir.join("versions").join("alpha_strategy").exists());
        assert!(graph_dir.join("beta_strategy.json").exists());

        let latest: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(graph_dir.join("latest.json")).unwrap())
                .unwrap();
        assert_eq!(latest["metadata"]["graph_id"], "beta_strategy");

        let list_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/graphs")
                    .method("GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(list_response.status(), StatusCode::OK);
        let body = to_bytes(list_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value.as_array().unwrap().len(), 1);
        assert_eq!(value[0]["graph_id"], "beta_strategy");
    }

    #[tokio::test]
    async fn delete_graph_endpoint_returns_not_found_for_missing_graph() {
        let base = test_storage_base("graph-delete-missing");
        let graph_dir = base.join("graphs");
        let run_dir = base.join("runs");
        let backtest_dir = base.join("backtests");
        std::fs::create_dir_all(&graph_dir).unwrap();
        std::fs::create_dir_all(&run_dir).unwrap();
        std::fs::create_dir_all(&backtest_dir).unwrap();

        let app = build_app_router(test_app_state_from_dirs(
            base,
            graph_dir,
            run_dir,
            backtest_dir,
        ));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/graphs/missing_strategy")
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn graph_version_endpoints_list_load_and_restore_versions() {
        let base = test_storage_base("graph-versions");
        let graph_dir = base.join("graphs");
        let run_dir = base.join("runs");
        let backtest_dir = base.join("backtests");
        std::fs::create_dir_all(&graph_dir).unwrap();
        std::fs::create_dir_all(&run_dir).unwrap();
        std::fs::create_dir_all(&backtest_dir).unwrap();

        let app = build_app_router(test_app_state_from_dirs(
            base,
            graph_dir.clone(),
            run_dir,
            backtest_dir,
        ));
        let graph_v1 = serde_json::json!({
            "metadata": {
                "graph_id": "versioned_strategy",
                "name": "Versioned Strategy V1",
                "version": "1.0.0"
            },
            "nodes": [
                {
                    "id": "data_feed",
                    "type": "data",
                    "module_key": "builtin.data.kline",
                    "name": "Price Feed",
                    "config": {
                        "window_size": 20
                    }
                }
            ],
            "edges": []
        });
        let graph_v2 = serde_json::json!({
            "metadata": {
                "graph_id": "versioned_strategy",
                "name": "Versioned Strategy V2",
                "version": "1.0.0"
            },
            "nodes": [
                {
                    "id": "data_feed",
                    "type": "data",
                    "module_key": "builtin.data.kline",
                    "name": "Price Feed",
                    "config": {
                        "window_size": 55
                    }
                }
            ],
            "edges": []
        });

        let save_v1 = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/graphs/save")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "graph": graph_v1 }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(save_v1.status(), StatusCode::OK);
        let save_v1_body = to_bytes(save_v1.into_body(), usize::MAX).await.unwrap();
        let save_v1_value: serde_json::Value = serde_json::from_slice(&save_v1_body).unwrap();
        let version_v1 = save_v1_value["version_id"].as_str().unwrap().to_string();

        sleep(Duration::from_millis(5)).await;

        let save_v2 = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/graphs/save")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "graph": graph_v2 }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(save_v2.status(), StatusCode::OK);
        let save_v2_body = to_bytes(save_v2.into_body(), usize::MAX).await.unwrap();
        let save_v2_value: serde_json::Value = serde_json::from_slice(&save_v2_body).unwrap();
        let version_v2 = save_v2_value["version_id"].as_str().unwrap().to_string();
        assert_ne!(version_v1, version_v2);

        let versions_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/graphs/versioned_strategy/versions")
                    .method("GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(versions_response.status(), StatusCode::OK);
        let versions_body = to_bytes(versions_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let versions: serde_json::Value = serde_json::from_slice(&versions_body).unwrap();
        assert_eq!(versions.as_array().unwrap().len(), 2);
        assert_eq!(versions[0]["version_id"], version_v2);
        assert_eq!(versions[0]["is_latest"], true);
        assert_eq!(versions[1]["version_id"], version_v1);
        assert!(versions[0]["path"]
            .as_str()
            .unwrap()
            .contains("\\versions\\"));

        let old_version_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/graphs/versioned_strategy/versions/{version_v1}"
                    ))
                    .method("GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(old_version_response.status(), StatusCode::OK);
        let old_version_body = to_bytes(old_version_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let old_version: serde_json::Value = serde_json::from_slice(&old_version_body).unwrap();
        assert_eq!(old_version["metadata"]["name"], "Versioned Strategy V1");
        assert_eq!(old_version["nodes"][0]["config"]["window_size"], 20);

        sleep(Duration::from_millis(5)).await;

        let restore_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/graphs/versioned_strategy/versions/{version_v1}/restore"
                    ))
                    .method("POST")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(restore_response.status(), StatusCode::OK);

        let latest_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/graphs/versioned_strategy")
                    .method("GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(latest_response.status(), StatusCode::OK);
        let latest_body = to_bytes(latest_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let latest: serde_json::Value = serde_json::from_slice(&latest_body).unwrap();
        assert_eq!(latest["metadata"]["name"], "Versioned Strategy V1");
        assert_eq!(latest["nodes"][0]["config"]["window_size"], 20);

        let versions_after_restore_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/graphs/versioned_strategy/versions")
                    .method("GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(versions_after_restore_response.status(), StatusCode::OK);
        let versions_after_restore_body =
            to_bytes(versions_after_restore_response.into_body(), usize::MAX)
                .await
                .unwrap();
        let versions_after_restore: serde_json::Value =
            serde_json::from_slice(&versions_after_restore_body).unwrap();
        assert_eq!(versions_after_restore.as_array().unwrap().len(), 3);
        assert_eq!(versions_after_restore[0]["is_latest"], true);
    }

    #[tokio::test]
    async fn reveal_graph_endpoint_returns_not_found_for_missing_graph() {
        let base = test_storage_base("graph-reveal");
        let graph_dir = base.join("graphs");
        let run_dir = base.join("runs");
        let backtest_dir = base.join("backtests");
        std::fs::create_dir_all(&graph_dir).unwrap();
        std::fs::create_dir_all(&run_dir).unwrap();
        std::fs::create_dir_all(&backtest_dir).unwrap();

        let app = build_app_router(test_app_state_from_dirs(
            base,
            graph_dir,
            run_dir,
            backtest_dir,
        ));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/graphs/missing_strategy/reveal")
                    .method("POST")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn reveal_graph_path_prefers_existing_quantscript_and_returns_absolute_path() {
        let base = test_storage_base("graph-reveal-path");
        let graph_dir = base.join("graphs");
        std::fs::create_dir_all(&graph_dir).unwrap();
        let graph_path = graph_dir.join("alpha_strategy.json");
        let quantscript_path = graph_dir.join("alpha_strategy.qs");
        std::fs::write(&graph_path, "{}").unwrap();
        std::fs::write(&quantscript_path, "strategy alpha() {}").unwrap();

        let graph = serde_json::json!({
            "metadata": {
                "artifacts": {
                    "quantscript": {
                        "saved_path": quantscript_path.to_string_lossy()
                    }
                }
            }
        });
        let reveal_path = graph_api::resolve_graph_reveal_path_from_value(&graph, &graph_path)
            .await
            .unwrap();

        assert!(reveal_path.is_absolute());
        assert_eq!(
            reveal_path,
            std::fs::canonicalize(&quantscript_path).unwrap()
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_returns_success() {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_test",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "runtime_targets": {
                "source_to_node": {
                    "data_data_feed": "data_feed",
                    "intent_intent_rsi": "intent_rsi",
                    "agent_script_main": "agent_main",
                    "risk_script_global": "risk_main"
                },
                "runtime_node_id": "runtime_main",
                "execution_node_id": "execution_main"
            },
            "source": r#"
fn strategy() {
    let data_data_feed_series = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let intent_intent_rsi_signal = rsi(data_data_feed_series, 14)
    if intent_intent_rsi_signal < 30 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else if intent_intent_rsi_signal > 70 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["graph_id"], "graph_test");
        assert_eq!(value["compile_id"], "compile_formal_test");
        assert_eq!(value["compilable"], true);
        assert_eq!(value["counts"]["data_sources"], 1);
        assert_eq!(
            value["core_ir"]["metadata"]["source_kind"],
            "formal_quant_script"
        );
        assert_eq!(value["core_ir"]["metadata"]["strategy_id"], "graph_test");
        assert_eq!(value["core_ir"]["metadata"]["name"], "Test Graph");
        assert_eq!(
            value["runtime_config"]["metadata"]["compile_id"],
            "compile_formal_test"
        );
        assert_eq!(
            value["runtime_config"]["data_sources"][0]["id"],
            "data_feed"
        );
        assert_eq!(
            value["runtime_config"]["intent_generators"][0]["id"],
            "intent_rsi"
        );
        assert_eq!(
            value["runtime_targets"]["source_to_node"]["data_data_feed"],
            "data_feed"
        );
        assert_eq!(
            value["runtime_targets"]["source_to_node"]["intent_intent_rsi"],
            "intent_rsi"
        );
        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"]["kind"],
            "raw_text"
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_matches_direct_ma_compare_golden_view() {
        let value = compile_formal_quantscript_for_test(
            r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let fast = sma(data_feed, 20)
    let slow = sma(data_feed, 100)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_ma_golden",
        )
        .await;

        assert_eq!(
            formal_compile_golden_view(&value),
            serde_json::json!({
                "core_source_kind": "formal_quant_script",
                "data_bindings": [
                    {
                        "data_id": "script_okx_btcusdt_1d",
                        "kind": "kline_series",
                        "source_hints": {
                            "exchange": "okx",
                            "symbol": "BTCUSDT",
                            "timeframe": "1d"
                        }
                    }
                ],
                "indicator_kinds": ["ma_cross"],
                "signal_rules": [
                    {
                        "signal_id": "intent_btcusdt_ma_entry_signal",
                        "signal_kind": "long",
                        "indicator_id": "intent_btcusdt_ma_entry",
                        "condition": {
                            "kind": "raw_text",
                            "source": "ma_cross(fast=20, slow=100, entry_ratio=0.2)"
                        }
                    }
                ],
                "agent_policy_kinds": ["weighted_signals"],
                "runtime_projection": {
                    "data_modules": ["builtin.data.kline"],
                    "intent_modules": ["builtin.intent.double_ma"],
                    "agent_modules": ["builtin.agent.weighted"],
                    "risk_modules": ["builtin.risk.global"],
                    "execution_modules": ["builtin.execution.paper"],
                    "runtime_module": "builtin.runtime.control"
                }
            })
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_matches_one_sided_rsi_golden_view() {
        let value = compile_formal_quantscript_for_test(
            r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let r = rsi(data_feed, 14)
    if r < 25 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_rsi_golden",
        )
        .await;

        assert_eq!(
            formal_compile_golden_view(&value),
            serde_json::json!({
                "core_source_kind": "formal_quant_script",
                "data_bindings": [
                    {
                        "data_id": "script_okx_btcusdt_1d",
                        "kind": "kline_series",
                        "source_hints": {
                            "exchange": "okx",
                            "symbol": "BTCUSDT",
                            "timeframe": "1d"
                        }
                    }
                ],
                "indicator_kinds": ["rsi"],
                "signal_rules": [
                    {
                        "signal_id": "intent_btcusdt_rsi_signal",
                        "signal_kind": "long",
                        "indicator_id": "intent_btcusdt_rsi",
                        "condition": {
                            "kind": "compare",
                            "left": {
                                "kind": "ref",
                                "name": "intent_btcusdt_rsi"
                            },
                            "op": "lt",
                            "right": {
                                "kind": "number",
                                "value": 25.0
                            }
                        }
                    }
                ],
                "agent_policy_kinds": ["weighted_signals"],
                "runtime_projection": {
                    "data_modules": ["builtin.data.kline"],
                    "intent_modules": ["builtin.intent.rsi"],
                    "agent_modules": ["builtin.agent.weighted"],
                    "risk_modules": ["builtin.risk.global"],
                    "execution_modules": ["builtin.execution.paper"],
                    "runtime_module": "builtin.runtime.control"
                }
            })
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_matches_one_sided_momentum_golden_view() {
        let value = compile_formal_quantscript_for_test(
            r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let m = momentum(data_feed, 20)
    if m > 0.03 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_momentum_golden",
        )
        .await;

        assert_eq!(
            formal_compile_golden_view(&value),
            serde_json::json!({
                "core_source_kind": "formal_quant_script",
                "data_bindings": [
                    {
                        "data_id": "script_okx_btcusdt_1d",
                        "kind": "kline_series",
                        "source_hints": {
                            "exchange": "okx",
                            "symbol": "BTCUSDT",
                            "timeframe": "1d"
                        }
                    }
                ],
                "indicator_kinds": ["momentum"],
                "signal_rules": [
                    {
                        "signal_id": "intent_btcusdt_momentum_signal",
                        "signal_kind": "long",
                        "indicator_id": "intent_btcusdt_momentum",
                        "condition": {
                            "kind": "compare",
                            "left": {
                                "kind": "ref",
                                "name": "intent_btcusdt_momentum"
                            },
                            "op": "gt",
                            "right": {
                                "kind": "number",
                                "value": 0.03
                            }
                        }
                    }
                ],
                "agent_policy_kinds": ["weighted_signals"],
                "runtime_projection": {
                    "data_modules": ["builtin.data.kline"],
                    "intent_modules": ["builtin.intent.momentum"],
                    "agent_modules": ["builtin.agent.weighted"],
                    "risk_modules": ["builtin.risk.global"],
                    "execution_modules": ["builtin.execution.paper"],
                    "runtime_module": "builtin.runtime.control"
                }
            })
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_matches_one_sided_zscore_golden_view() {
        let value = compile_formal_quantscript_for_test(
            r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let z = zscore(data_feed, 20)
    if z < -2 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_zscore_golden",
        )
        .await;

        assert_eq!(
            formal_compile_golden_view(&value),
            serde_json::json!({
                "core_source_kind": "formal_quant_script",
                "data_bindings": [
                    {
                        "data_id": "script_okx_btcusdt_1d",
                        "kind": "kline_series",
                        "source_hints": {
                            "exchange": "okx",
                            "symbol": "BTCUSDT",
                            "timeframe": "1d"
                        }
                    }
                ],
                "indicator_kinds": ["z_score"],
                "signal_rules": [
                    {
                        "signal_id": "intent_btcusdt_zscore_signal",
                        "signal_kind": "long",
                        "indicator_id": "intent_btcusdt_zscore",
                        "condition": {
                            "kind": "compare",
                            "left": {
                                "kind": "ref",
                                "name": "intent_btcusdt_zscore"
                            },
                            "op": "lt",
                            "right": {
                                "kind": "number",
                                "value": -2.0
                            }
                        }
                    }
                ],
                "agent_policy_kinds": ["weighted_signals"],
                "runtime_projection": {
                    "data_modules": ["builtin.data.kline"],
                    "intent_modules": ["builtin.intent.zscore"],
                    "agent_modules": ["builtin.agent.weighted"],
                    "risk_modules": ["builtin.risk.global"],
                    "execution_modules": ["builtin.execution.paper"],
                    "runtime_module": "builtin.runtime.control"
                }
            })
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_matches_non_trunk_import_alias_diagnostic_golden_view(
    ) {
        let value = compile_formal_quantscript_error_for_test(
            r#"
import data as market_data

fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=50)?
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#,
            "compile_formal_import_alias_golden",
        )
        .await;

        assert_eq!(
            formal_compile_error_golden_view(&value),
            serde_json::json!({
                "error": "quantscript_compile_failed",
                "detail": {
                    "code": "QS0608",
                    "message": "形式化 QuantScript 不支持简单的 `import foo as bar`；请使用 `from module import name as alias`",
                    "reason": serde_json::Value::Null,
                    "span_label": "data as market_data",
                }
            })
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_matches_universe_helper_input_diagnostic_golden_view(
    ) {
        let value = compile_formal_quantscript_error_for_test(
            r#"
fn strategy() {
    let selected = top(1, 2)
    rebalance(equal_weight(selected), every="1d")
}
"#,
            "compile_formal_universe_input_golden",
        )
        .await;

        assert_eq!(
            formal_compile_error_golden_view(&value),
            serde_json::json!({
                "error": "quantscript_compile_failed",
                "detail": {
                    "code": "QS0610",
                    "message": "策略函数必须包含至少一个 fetch() 调用来获取市场数据",
                    "reason": serde_json::Value::Null,
                    "span_label": "strategy",
                }
            })
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_matches_multi_detail_non_trunk_diagnostic_golden_view(
    ) {
        let value = compile_formal_quantscript_error_for_test(
            r#"
import data as market_data

fn helper(series) {
    return helper(series)
}

async fn strategy() {
    let closes = await fetch("BTCUSDT", interval="1d", lookback=50)?
    let unsafe_try = sma(closes, 20)?
    let mut out = []
    out.push(1)
    if fetch("BTCUSDT", interval="1d", lookback=20).retryable() {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    for value in closes[20..] {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    while closes[0] > 0 {
        match closes[0] {
            _ => emit Intent("BUY", instrument="BTCUSDT", quantity=1.0),
        }
    }
}
"#,
            "compile_formal_non_trunk_golden",
        )
        .await;

        assert_eq!(
            formal_compile_error_details_golden_view(&value),
            serde_json::json!({
                "error": "quantscript_compile_failed",
                "details": [
                    {
                        "code": "QS0608",
                        "message": "形式化 QuantScript 不支持简单的 `import foo as bar`；请使用 `from module import name as alias`",
                        "span_label": "data as market_data",
                    },
                    {
                        "code": "QS0605",
                        "message": "形式化 QuantScript 不支持可执行主干中的递归辅助调用",
                        "span_label": "helper",
                    },
                    {
                        "code": "QS0601",
                        "message": "形式化 QuantScript 不支持可执行主干中的异步函数",
                        "span_label": "strategy",
                    },
                    {
                        "code": "QS0602",
                        "message": "形式化 QuantScript 不支持可执行主干中的 await 表达式",
                        "span_label": "await",
                    },
                    {
                        "code": "QS0607",
                        "message": "形式化 QuantScript 在可执行主干中仅支持对 fetch 类数据源表达式使用后缀 `?`",
                        "span_label": "?",
                    },
                    {
                        "code": "QS0609",
                        "message": "形式化 QuantScript 不支持可执行主干中使用 `.push(...)` 构建可变列表",
                        "span_label": ".push",
                    },
                    {
                        "code": "QS0610",
                        "message": "形式化 QuantScript 不支持可执行主干中的 `.ok()` / `.retryable()` 辅助方法",
                        "span_label": "retryable",
                    },
                    {
                        "code": "QS0603",
                        "message": "形式化 QuantScript 不支持可执行主干中的 while 循环",
                        "span_label": "strategy",
                    },
                    {
                        "code": "QS0604",
                        "message": "形式化 QuantScript 不支持可执行主干中的 match 语句",
                        "span_label": "strategy",
                    },
                    {
                        "code": "QS0606",
                        "message": "形式化 QuantScript 在可执行主干中仅支持对 Universe 的 for 循环",
                        "span_label": "for",
                    }
                ],
            })
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_matches_multi_detail_lowering_diagnostic_golden_view(
    ) {
        let value = compile_formal_quantscript_error_for_test(
            r#"
fn strategy() {
    emit Intent("HOLD", instrument="BTCUSDT", quantity=1.0)
    emit Intent("", instrument="BTCUSDT", quantity=1.0)
}
"#,
            "compile_formal_lowering_multi_golden",
        )
        .await;

        assert_eq!(
            formal_compile_error_details_golden_view(&value),
            serde_json::json!({
                "error": "quantscript_compile_failed",
                "details": [
                    {
                        "code": "QS0610",
                        "message": "策略函数必须包含至少一个 fetch() 调用来获取市场数据",
                        "span_label": "strategy",
                    }
                ],
            })
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_matches_mixed_boundary_diagnostic_golden_view() {
        let value = compile_formal_quantscript_error_for_test(
            r#"
import data as market_data

fn strategy() {
    emit Intent("HOLD", instrument="BTCUSDT", quantity=1.0)
    emit Intent("", instrument="BTCUSDT", quantity=1.0)
}
"#,
            "compile_formal_mixed_boundary_golden",
        )
        .await;

        assert_eq!(
            formal_compile_error_details_golden_view(&value),
            serde_json::json!({
                "error": "quantscript_compile_failed",
                "details": [
                    {
                        "code": "QS0608",
                        "message": "形式化 QuantScript 不支持简单的 `import foo as bar`；请使用 `from module import name as alias`",
                        "span_label": "data as market_data",
                    },
                    {
                        "code": "QS0610",
                        "message": "策略函数必须包含至少一个 fetch() 调用来获取市场数据",
                        "span_label": "strategy",
                    }
                ],
            })
        );
    }

    #[tokio::test]
    async fn formal_quantscript_text_to_core_ir_to_graph_round_trip_sample() {
        let source = r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let fast = sma(data_feed, 20)
    let slow = sma(data_feed, 100)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#;
        let value =
            compile_formal_quantscript_for_test(source, "compile_formal_round_trip_sample").await;

        let graph = graph_value_from_runtime_config(&value["runtime_config"], source);
        let generated = generate_quantscript_from_graph_value(&graph).unwrap();
        let reparsed = parse_graph_quantscript_source(&generated).unwrap();
        let regenerated = generate_quantscript_from_graph_value(&reparsed).unwrap();
        let module_keys: Vec<&str> = reparsed["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|node| node["module_key"].as_str())
            .collect();

        assert_eq!(
            value["core_ir"]["metadata"]["source_kind"],
            "formal_quant_script"
        );
        assert!(generated.starts_with("strategy_graph graph_test {"));
        assert!(regenerated.starts_with("strategy_graph graph_test {"));
        assert_eq!(reparsed["metadata"]["graph_id"], "graph_test");
        assert_eq!(reparsed["nodes"].as_array().unwrap().len(), 6);
        assert_eq!(reparsed["edges"].as_array().unwrap().len(), 5);
        assert!(module_keys.contains(&"builtin.data.kline"));
        assert!(module_keys.contains(&"builtin.intent.double_ma"));
        assert!(module_keys.contains(&"builtin.agent.weighted"));
        assert!(module_keys.contains(&"builtin.risk.global"));
        assert!(module_keys.contains(&"builtin.execution.paper"));
        assert!(module_keys.contains(&"builtin.runtime.control"));
    }

    #[tokio::test]
    async fn formal_quantscript_text_to_core_ir_to_graph_round_trip_momentum_sample() {
        let source = r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let m = momentum(data_feed, 20)
    if m > 0.03 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#;
        let value = compile_formal_quantscript_for_test(
            source,
            "compile_formal_round_trip_momentum_sample",
        )
        .await;

        let graph = graph_value_from_runtime_config(&value["runtime_config"], source);
        let generated = generate_quantscript_from_graph_value(&graph).unwrap();
        let reparsed = parse_graph_quantscript_source(&generated).unwrap();
        let regenerated = generate_quantscript_from_graph_value(&reparsed).unwrap();
        let module_keys: Vec<&str> = reparsed["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|node| node["module_key"].as_str())
            .collect();

        assert_eq!(
            value["core_ir"]["metadata"]["source_kind"],
            "formal_quant_script"
        );
        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"]["left"]["name"],
            "intent_btcusdt_momentum"
        );
        assert!(generated.starts_with("strategy_graph graph_test {"));
        assert!(regenerated.starts_with("strategy_graph graph_test {"));
        assert_eq!(reparsed["metadata"]["graph_id"], "graph_test");
        assert_eq!(reparsed["nodes"].as_array().unwrap().len(), 6);
        assert_eq!(reparsed["edges"].as_array().unwrap().len(), 5);
        assert!(module_keys.contains(&"builtin.data.kline"));
        assert!(module_keys.contains(&"builtin.intent.momentum"));
        assert!(module_keys.contains(&"builtin.agent.weighted"));
        assert!(module_keys.contains(&"builtin.risk.global"));
        assert!(module_keys.contains(&"builtin.execution.paper"));
        assert!(module_keys.contains(&"builtin.runtime.control"));
    }

    #[tokio::test]
    async fn formal_quantscript_text_to_core_ir_to_graph_round_trip_rsi_sample() {
        let source = r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let r = rsi(data_feed, 14)
    if r < 25 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#;
        let value =
            compile_formal_quantscript_for_test(source, "compile_formal_round_trip_rsi_sample")
                .await;

        let graph = graph_value_from_runtime_config(&value["runtime_config"], source);
        let generated = generate_quantscript_from_graph_value(&graph).unwrap();
        let reparsed = parse_graph_quantscript_source(&generated).unwrap();
        let regenerated = generate_quantscript_from_graph_value(&reparsed).unwrap();
        let module_keys: Vec<&str> = reparsed["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|node| node["module_key"].as_str())
            .collect();

        assert_eq!(
            value["core_ir"]["metadata"]["source_kind"],
            "formal_quant_script"
        );
        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"]["left"]["name"],
            "intent_btcusdt_rsi"
        );
        assert!(generated.starts_with("strategy_graph graph_test {"));
        assert!(regenerated.starts_with("strategy_graph graph_test {"));
        assert_eq!(reparsed["metadata"]["graph_id"], "graph_test");
        assert_eq!(reparsed["nodes"].as_array().unwrap().len(), 6);
        assert_eq!(reparsed["edges"].as_array().unwrap().len(), 5);
        assert!(module_keys.contains(&"builtin.data.kline"));
        assert!(module_keys.contains(&"builtin.intent.rsi"));
        assert!(module_keys.contains(&"builtin.agent.weighted"));
        assert!(module_keys.contains(&"builtin.risk.global"));
        assert!(module_keys.contains(&"builtin.execution.paper"));
        assert!(module_keys.contains(&"builtin.runtime.control"));
    }

    #[tokio::test]
    async fn formal_quantscript_text_to_core_ir_to_graph_round_trip_zscore_sample() {
        let source = r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let z = zscore(data_feed, 20)
    if z < -2 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#;
        let value =
            compile_formal_quantscript_for_test(source, "compile_formal_round_trip_zscore_sample")
                .await;

        let graph = graph_value_from_runtime_config(&value["runtime_config"], source);
        let generated = generate_quantscript_from_graph_value(&graph).unwrap();
        let reparsed = parse_graph_quantscript_source(&generated).unwrap();
        let regenerated = generate_quantscript_from_graph_value(&reparsed).unwrap();
        let module_keys: Vec<&str> = reparsed["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|node| node["module_key"].as_str())
            .collect();

        assert_eq!(
            value["core_ir"]["metadata"]["source_kind"],
            "formal_quant_script"
        );
        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"]["left"]["name"],
            "intent_btcusdt_zscore"
        );
        assert!(generated.starts_with("strategy_graph graph_test {"));
        assert!(regenerated.starts_with("strategy_graph graph_test {"));
        assert_eq!(reparsed["metadata"]["graph_id"], "graph_test");
        assert_eq!(reparsed["nodes"].as_array().unwrap().len(), 6);
        assert_eq!(reparsed["edges"].as_array().unwrap().len(), 5);
        assert!(module_keys.contains(&"builtin.data.kline"));
        assert!(module_keys.contains(&"builtin.intent.zscore"));
        assert!(module_keys.contains(&"builtin.agent.weighted"));
        assert!(module_keys.contains(&"builtin.risk.global"));
        assert!(module_keys.contains(&"builtin.execution.paper"));
        assert!(module_keys.contains(&"builtin.runtime.control"));
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_lowers_direct_ma_compare_to_structured_core_ir_condition(
    ) {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_ma_compare",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let fast = sma(data_feed, 20)
    let slow = sma(data_feed, 100)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"]["kind"],
            "raw_text"
        );
        assert!(
            value["core_ir"]["signal_rules"][0]["condition"]["source"]
                .as_str()
                .unwrap_or("")
                .contains("ma_cross")
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_lowers_one_sided_rsi_compare_to_structured_core_ir_condition(
    ) {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_rsi_compare",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let r = rsi(data_feed, 14)
    if r < 25 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"]["kind"],
            "compare"
        );
        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"]["left"]["kind"],
            "ref"
        );
        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"]["left"]["name"],
            "intent_btcusdt_rsi"
        );
        assert_eq!(value["core_ir"]["signal_rules"][0]["condition"]["op"], "lt");
        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"]["right"]["kind"],
            "number"
        );
        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"]["right"]["value"],
            25.0
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_lowers_one_sided_momentum_compare_to_structured_core_ir_condition(
    ) {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_momentum_compare",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let m = momentum(data_feed, 20)
    if m > 0.03 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"]["kind"],
            "compare"
        );
        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"]["left"]["name"],
            "intent_btcusdt_momentum"
        );
        assert_eq!(value["core_ir"]["signal_rules"][0]["condition"]["op"], "gt");
        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"]["right"]["value"],
            0.03
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_keeps_dual_sided_momentum_compare_on_raw_text_path(
    ) {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_momentum_dual",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let m = momentum(data_feed, 20)
    if m > 0.03 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else if m < -0.03 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"]["kind"],
            "raw_text"
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_lowers_one_sided_zscore_compare_to_structured_core_ir_condition(
    ) {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_zscore_compare",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let z = zscore(data_feed, 20)
    if z < -2 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"]["kind"],
            "compare"
        );
        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"]["left"]["name"],
            "intent_btcusdt_zscore"
        );
        assert_eq!(value["core_ir"]["signal_rules"][0]["condition"]["op"], "lt");
        assert_eq!(
            value["core_ir"]["signal_rules"][0]["condition"]["right"]["value"],
            -2.0
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_lowers_equal_weight_rebalance_helper() {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_rebalance",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": r#"
fn strategy() {
    let base = symbols(["BTCUSDT", "ETHUSDT"])
    rebalance(equal_weight(base), every="1d")
    for s in base {
        let closes = fetch(s, exchange="binance", interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)
        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["compilable"], true);
        assert_eq!(
            value["artifacts"]["core_ir"]["core_ir"]["agent_policies"][0]["kind"],
            "portfolio_rebalance"
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_emits_authoring_view_for_headered_sections() {
        let value = compile_formal_quantscript_for_test(
            r#"fn strategy() {
    # risk
    risk.profile("global", max_position=0.35)
    # execution
    execution.profile("paper", fee_bps=12.5, slippage_bps=7.5)
    # data
    let closes = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let fast = sma(closes, 20)
    let slow = sma(closes, 100)
    # intent
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    # agent
    let base = symbols(["BTCUSDT", "ETHUSDT"])
    rebalance(equal_weight(base), every="1d")
}
"#,
            "compile_formal_authoring_view_headers",
        )
        .await;

        let authoring_view = formal_compile_authoring_view(&value);
        assert_eq!(authoring_view["kind"], "quantscript_authoring_view");
        assert_eq!(
            authoring_view["source_order"],
            serde_json::json!(["risk", "execution", "data", "intent", "agent"])
        );
        assert_eq!(
            authoring_view["pipeline_order"],
            serde_json::json!(["data", "intent", "agent", "risk", "execution"])
        );
        assert_eq!(
            authoring_view["sections"]
                .as_array()
                .unwrap()
                .iter()
                .map(|section| section["effective_kind"].clone())
                .collect::<Vec<_>>(),
            vec![
                serde_json::json!("risk"),
                serde_json::json!("execution"),
                serde_json::json!("data"),
                serde_json::json!("intent"),
                serde_json::json!("agent"),
            ]
        );
        assert_eq!(
            authoring_view["edges"]
                .as_array()
                .unwrap()
                .iter()
                .map(|edge| edge["reason"].clone())
                .collect::<Vec<_>>(),
            vec![
                serde_json::json!("intent_reads_data"),
                serde_json::json!("agent_uses_intent"),
                serde_json::json!("risk_governs_agent"),
                serde_json::json!("execution_applies_to_agent"),
            ]
        );
        assert_eq!(
            authoring_view["sections"][0]["start_line"],
            serde_json::json!(2)
        );
        assert_eq!(
            authoring_view["sections"][0]["end_line"],
            serde_json::json!(3)
        );
        assert!(authoring_view["sections"][0]["snippet"]
            .as_str()
            .unwrap()
            .contains("# risk"));
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_emits_authoring_view_without_explicit_headers() {
        let value = compile_formal_quantscript_for_test(
            r#"fn strategy() {
    execution.profile("paper", fee_bps=12.5, slippage_bps=7.5)
    let closes = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let m = momentum(closes, 20)
    if m > 0.03 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_authoring_view_inferred",
        )
        .await;

        let authoring_view = formal_compile_authoring_view(&value);
        assert_eq!(
            authoring_view["sections"]
                .as_array()
                .unwrap()
                .iter()
                .map(|section| {
                    serde_json::json!({
                        "declared": section["declared_kind"].clone(),
                        "effective": section["effective_kind"].clone(),
                        "origin": section["origin"].clone(),
                    })
                })
                .collect::<Vec<_>>(),
            vec![
                serde_json::json!({"declared": "execution", "effective": "execution", "origin": "hybrid"}),
                serde_json::json!({"declared": "data", "effective": "data", "origin": "hybrid"}),
                serde_json::json!({"declared": "intent", "effective": "intent", "origin": "hybrid"}),
            ]
        );
        assert_eq!(
            authoring_view["edges"]
                .as_array()
                .unwrap()
                .iter()
                .map(|edge| edge["relation"].clone())
                .collect::<Vec<_>>(),
            vec![serde_json::json!("dataflow")]
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_emits_partial_authoring_view_on_semantic_failure()
    {
        let value = compile_formal_quantscript_error_for_test(
            r#"fn strategy() {
    # risk
    risk.profile("global", max_position=0.35)
    # data
    let closes = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let fast = sma(closes, 20)
    # intent
    if fast > threshold {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_partial_authoring_semantic_failure",
        )
        .await;

        let authoring_view = formal_compile_partial_authoring_view(&value);
        assert_eq!(authoring_view["kind"], "quantscript_authoring_view");
        assert_eq!(
            authoring_view["sections"]
                .as_array()
                .unwrap()
                .iter()
                .map(|section| section["effective_kind"].clone())
                .collect::<Vec<_>>(),
            vec![
                serde_json::json!("risk"),
                serde_json::json!("data"),
                serde_json::json!("intent"),
            ]
        );
        assert_eq!(authoring_view["pool_pipeline"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_emits_pool_pipeline_in_authoring_view() {
        let value = compile_formal_quantscript_for_test_with_universe_snapshot(
            r#"fn strategy() {
    # data
    let closes = fetch("BTCUSDT", exchange="binance", interval="1d", lookback=30)?
    let base = universe(exchange="binance", market="spot", quote="USDT")
    let liquid = filter(base, min_volume_24h=1000000000, min_listing_age_days=180)
    let leaders = top(sort_by(liquid, key="market_cap", order="desc"), 2)

    # intent
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)

    # agent
    rebalance(rank_weight(leaders, method="linear"), every="weekly")
}
"#,
            "compile_formal_authoring_view_pool_pipeline",
            Some(sample_formal_universe_snapshot_json()),
        )
        .await;

        let authoring_view = formal_compile_authoring_view(&value);
        assert_eq!(
            authoring_view["pool_pipeline"]["order"],
            serde_json::json!([
                "source",
                "eligibility",
                "features",
                "selection",
                "weighting",
                "rebalance"
            ])
        );
        assert_eq!(
            authoring_view["pool_pipeline"]["stages"]
                .as_array()
                .unwrap()
                .iter()
                .map(|stage| stage["kind"].clone())
                .collect::<Vec<_>>(),
            vec![
                serde_json::json!("source"),
                serde_json::json!("eligibility"),
                serde_json::json!("features"),
                serde_json::json!("selection"),
                serde_json::json!("weighting"),
                serde_json::json!("rebalance"),
            ]
        );
        assert_eq!(
            authoring_view["pool_pipeline"]["stages"][0]["summary"],
            serde_json::json!("universe(exchange=binance, market=spot, quote=USDT)")
        );
        assert_eq!(
            authoring_view["pool_pipeline"]["stages"][1]["details"],
            serde_json::json!(["volume_24h >= 1000000000", "listing_age_days >= 180"])
        );
        assert_eq!(
            authoring_view["pool_pipeline"]["stages"][2]["status"],
            serde_json::json!("empty")
        );
        assert_eq!(
            authoring_view["pool_pipeline"]["stages"][3]["summary"],
            serde_json::json!("ordered_top_n by metadata.market_cap desc top 2")
        );
        assert_eq!(
            authoring_view["pool_pipeline"]["stages"][4]["summary"],
            serde_json::json!("rank_weight (linear)")
        );
        assert_eq!(
            authoring_view["pool_pipeline"]["stages"][5]["summary"],
            serde_json::json!("rebalance weekly")
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_emits_partial_pool_pipeline_on_lowering_failure() {
        let value = compile_formal_quantscript_error_for_test_with_universe_snapshot(
            r#"fn strategy() {
    # data
    let base = universe(exchange="binance", market="spot", quote="USDT")
    let liquid = filter(base, min_volume_24h=1000000000, min_listing_age_days=180)
    let ranked = top(sort_by(liquid, key="factor_score", order="desc"), 3)

    # intent
    for s in ranked {
        let closes = fetch(s, exchange="binance", interval="1d", lookback=30)?
        let signal = momentum(closes, 20)
        if signal > 0.03 {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }

    # agent
    rebalance(rank_weight(ranked, method="linear"), every="weekly")
}
"#,
            "compile_formal_partial_authoring_lowering_failure",
            Some(sample_formal_universe_snapshot_json()),
        )
        .await;

        assert_eq!(value["error"], "quantscript_lowering_failed");
        assert_eq!(
            api_error_detail_by_code(&value, "QPQSLOW011")["code"],
            "QPQSLOW011"
        );

        let authoring_view = formal_compile_partial_authoring_view(&value);
        assert_eq!(authoring_view["kind"], "quantscript_authoring_view");
        assert_eq!(
            authoring_view["pool_pipeline"]["stages"][3]["summary"],
            serde_json::json!("ordered_top_n by feature.factor_score desc top 3")
        );
        assert!(
            !authoring_view["pool_pipeline"]["stages"][3]["related_section_ids"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        let effective_kinds = authoring_view["sections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|section| section["effective_kind"].as_str().unwrap().to_string())
            .collect::<Vec<_>>();
        assert!(effective_kinds.len() >= 2);
        assert!(
            effective_kinds.iter().any(|kind| kind == "agent")
                || effective_kinds.iter().any(|kind| kind == "mixed")
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_returns_structured_diagnostics() {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_test",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=20)?
    let slow = closes[50..].sum() / 50
    if closes.last() > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["error"], "quantscript_compile_failed");
        assert_eq!(value["details"][0]["code"], "QS0501");
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_rejects_non_trunk_control_flow_constructs_early() {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_non_trunk",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": r#"
import data as market_data

fn helper(series) {
    return helper(series)
}

async fn strategy() {
    let closes = await fetch("BTCUSDT", interval="1d", lookback=50)?
    let unsafe_try = sma(closes, 20)?
    let mut out = []
    out.push(1)
    if fetch("BTCUSDT", interval="1d", lookback=20).retryable() {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    for value in closes[20..] {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    while closes[0] > 0 {
        match closes[0] {
            _ => emit Intent("BUY", instrument="BTCUSDT", quantity=1.0),
        }
    }
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["error"], "quantscript_compile_failed");
        assert!(value["details"]
            .as_array()
            .unwrap()
            .iter()
            .any(|detail| detail["code"] == "QS0601"));
        assert!(value["details"]
            .as_array()
            .unwrap()
            .iter()
            .any(|detail| detail["code"] == "QS0602"));
        assert!(value["details"]
            .as_array()
            .unwrap()
            .iter()
            .any(|detail| detail["code"] == "QS0603"));
        assert!(value["details"]
            .as_array()
            .unwrap()
            .iter()
            .any(|detail| detail["code"] == "QS0604"));
        assert!(value["details"]
            .as_array()
            .unwrap()
            .iter()
            .any(|detail| detail["code"] == "QS0605"));
        assert!(value["details"]
            .as_array()
            .unwrap()
            .iter()
            .any(|detail| detail["code"] == "QS0606"));
        assert!(value["details"]
            .as_array()
            .unwrap()
            .iter()
            .any(|detail| detail["code"] == "QS0607"));
        assert!(value["details"]
            .as_array()
            .unwrap()
            .iter()
            .any(|detail| detail["code"] == "QS0608"));
        assert!(value["details"]
            .as_array()
            .unwrap()
            .iter()
            .any(|detail| detail["code"] == "QS0609"));
        assert!(value["details"]
            .as_array()
            .unwrap()
            .iter()
            .any(|detail| detail["code"] == "QS0610"));
        assert!(api_error_detail_by_code(&value, "QS0610")["message"]
            .as_str()
            .unwrap()
            .contains(".ok()"));
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_missing_fetch(
    ) {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_missing_fetch",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": r#"
fn strategy() {
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["error"], "quantscript_compile_failed");
        assert_eq!(value["details"][0]["code"], "QS0610");
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_unsupported_emit_action(
    ) {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_bad_action",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=20)?
    emit Intent("HOLD", instrument="BTCUSDT", quantity=1.0)
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["error"], "quantscript_lowering_failed");
        assert_eq!(value["details"][0]["code"], "QPQSLOW004");
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_malformed_spread_helper(
    ) {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_bad_spread_helper",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": r#"
fn strategy() {
    let left = fetch("BTCUSDT", interval="1d", lookback=20)?
    let right = fetch("ETHUSDT", interval="1d", lookback=20)?
    if spread(left) > 0.0 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["error"], "quantscript_lowering_failed");
        assert_eq!(value["details"][0]["code"], "QPQSLOW001");
        assert_eq!(
            value["details"][0]["reason"],
            "将条件下发重写为支持的指标或价差意图，或保留下发为无条件。"
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_missing_indicator_source(
    ) {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_missing_indicator_source",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=20)?
    let r = rsi(1, 14)
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["error"], "quantscript_compile_failed");
        assert_eq!(value["details"][0]["code"], "QS0007");
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_missing_macd_source(
    ) {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_missing_macd_source",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=20)?
    let m = macd()
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["error"], "quantscript_lowering_failed");
        assert_eq!(value["details"][0]["code"], "QPQSLOW022");
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_missing_momentum_source(
    ) {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_missing_momentum_source",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=20)?
    let m = momentum()
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["error"], "quantscript_lowering_failed");
        assert_eq!(value["details"][0]["code"], "QPQSLOW022");
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_missing_zscore_source(
    ) {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_missing_zscore_source",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=20)?
    let z = zscore()
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["error"], "quantscript_lowering_failed");
        assert_eq!(value["details"][0]["code"], "QPQSLOW022");
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_non_positive_indicator_window(
    ) {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_non_positive_indicator_window",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=20)?
    let fast = sma(closes, 0)
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["error"], "quantscript_lowering_failed");
        assert_eq!(value["details"][0]["code"], "QPQSLOW023");
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_missing_indicator_window(
    ) {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_missing_indicator_window",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=20)?
    let fast = sma(closes)
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["error"], "quantscript_lowering_failed");
        assert_eq!(value["details"][0]["code"], "QPQSLOW023");
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_invalid_moving_average_source(
    ) {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_invalid_moving_average_source",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=20)?
    let fast = sma(1, 20)
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["error"], "quantscript_compile_failed");
        assert_eq!(value["details"][0]["code"], "QS0007");
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_missing_moving_average_source(
    ) {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_missing_moving_average_source",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=20)?
    let fast = sma()
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["error"], "quantscript_lowering_failed");
        assert_eq!(value["details"][0]["code"], "QPQSLOW024");
        assert_eq!(
            value["details"][0]["reason"],
            "将 fetch/get_data 序列传入移动平均辅助函数，或对 ema(...) 传入可识别的 MACD 线。"
        );
    }

    #[tokio::test]
    async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_invalid_universe_helper_input(
    ) {
        let app = build_app_router(test_app_state());
        let payload = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_formal_invalid_universe_input",
            "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
            "source": r#"
fn strategy() {
    let selected = top(1, 2)
    rebalance(equal_weight(selected), every="1d")
}
"#
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/quantscript/formal/compile")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["error"], "quantscript_compile_failed");
        assert_eq!(value["details"][0]["code"], "QS0610");
    }
}
