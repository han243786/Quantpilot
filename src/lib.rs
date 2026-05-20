macro_rules! safe_eprintln {
    ($($arg:tt)*) => {{
        // v2.1.0: 尊重 QUANTPILOT_LOG_LEVEL 配置
        if $crate::safe_log::configured_log_level() >= $crate::safe_log::LogLevel::Info {
            eprintln!("{}", $crate::safe_log::sanitize_secrets(&format!($($arg)*)))
        }
    }};
}

mod alert_engine;
mod api_errors;
mod backup;
mod api_test_scenario;
pub mod app_router;
pub mod app_runtime_helpers;
pub mod auth;
mod auth_middleware;
mod credential_api;
pub mod credential_vault;
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
pub mod compile_api;
mod compile_artifact_builders;
mod compile_diagnostics;
mod error_codes;
mod formal_quantscript_authoring_types;
mod frontend_api_types;
mod frontend_runtime_mapping;
mod graph_api;
mod graph_quantscript_api;
mod graph_version_compare;
mod hotswap_api;
pub mod migration_sender;
mod runbook;
pub mod safe_log;
mod runtime;
mod runtime_diagnostics;
mod runtime_event_projection;
pub mod runtime_persistence;
mod runtime_response_mapping;
pub mod runtime_validation;
mod sandbox_verification;
mod snapshot_service;
pub mod storage_lifecycle;
mod test_runner;

use anyhow::{bail, Context};
use async_stream::stream;
use axum::{
    extract::{DefaultBodyLimit, Path, State},
    http::{header, HeaderValue, Method, StatusCode},
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
    IntentKind, OpenOrder, PortfolioState, RiskConfig, RunModeSpec, RunSpec,
    RunSpecRuntimeProtocolInput, RuntimeEvent, RuntimeEventType, RuntimeProtocolCoreConfig,
    SessionOutput, StrategyArtifact, StrategyArtifactSourceKind, StrategyIr, UniverseSnapshot,
    COMPILE_ARTIFACT_V1_VERSION, CORE_IR_ARTIFACT_V1_VERSION, GLOBAL_RISK_PROFILE_ID,
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
use tower_http::cors::CorsLayer;

use api_errors::*;
pub use app_router::*;
pub use app_runtime_helpers::*;
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
use runtime::*;
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
pub struct AppState {
    run_in_progress: Arc<std::sync::atomic::AtomicBool>,
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
    config_generation_history: Arc<tokio::sync::Mutex<Vec<qrpc_runtime::ConfigGenerationEntry>>>,
    credential_vault: Option<Arc<credential_vault::CredentialVault>>,
    // v2.0.0: 多用户认证数据库
    db: Option<Arc<std::sync::Mutex<rusqlite::Connection>>>,
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
            RuntimeParameterMutationStatus::ActivationScheduled => {
                self.mutation_activation_scheduled_count.fetch_add(1, Ordering::Relaxed);
            }
            RuntimeParameterMutationStatus::Activated => {
                self.mutation_activation_applied_count.fetch_add(1, Ordering::Relaxed);
            }
            RuntimeParameterMutationStatus::ActivationFailed => {
                self.mutation_activation_failed_count.fetch_add(1, Ordering::Relaxed);
            }
            RuntimeParameterMutationStatus::SafeWindowDenied => {
                self.mutation_safe_window_denied_count.fetch_add(1, Ordering::Relaxed);
            }
            RuntimeParameterMutationStatus::RollbackScheduled => {
                self.mutation_rollback_scheduled_count.fetch_add(1, Ordering::Relaxed);
            }
            RuntimeParameterMutationStatus::RolledBack => {
                self.mutation_rollback_applied_count.fetch_add(1, Ordering::Relaxed);
            }
            RuntimeParameterMutationStatus::RollbackFailed => {
                self.mutation_rollback_failed_count.fetch_add(1, Ordering::Relaxed);
            }
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
#[allow(dead_code)]
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
    /// v2.3.0: 语言中立错误码
    #[serde(skip_serializing_if = "Option::is_none")]
    error_code: Option<&'static str>,
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

pub async fn run_server() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv(); // 加载 .env 文件 (不存在则静默跳过)
    // v2.2.1: 初始化 tracing 订阅器 (JSON 格式输出到 stderr, 生产环境)
    let log_format = std::env::var("QUANTPILOT_LOG_FORMAT").unwrap_or_else(|_| "compact".to_string());
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .with_writer(std::io::stderr);
    if log_format == "json" {
        subscriber.json().init();
    } else {
        subscriber.compact().init();
    }
    // v1.1.1: 启动时记录 DEV 模式状态
    let is_dev = std::env::var("QUANTPILOT_DEV").unwrap_or_default() == "true";
    if is_dev {
        safe_eprintln!("[启动] DEV 模式已启用 — 瞬态数据 TTL 缩短，强制启动清理");
    }
    // v1.1.11: 全局 panic hook，防止 panic 静默丢失
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("{}", info);
        eprintln!("[panic] {} — 服务将退出", crate::safe_log::sanitize_secrets(&msg));
    }));
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "credential" {
        if let Err(e) = cli_support::handle_credential_command(&args[1..]) {
            safe_eprintln!("错误: {}", e);
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
    let mutation_store_dir = PathBuf::from("storage/mutations");
    let ai_proposal_store_dir = PathBuf::from("storage/ai-proposals");
    // v2.5.0: 并行创建 13 个存储目录, 减少启动等待时间
    let dirs: Vec<_> = [
        &graph_store_dir,
        &run_store_dir,
        &backtest_store_dir,
        &experiment_store_dir,
        &approval_store_dir,
        &sandbox_report_store_dir,
        &alert_store_dir,
        &snapshot_store_dir,
        &chaos_store_dir,
        &audit_store_dir,
        &report_store_dir,
        &mutation_store_dir,
        &ai_proposal_store_dir,
    ].iter().map(|d| d.to_path_buf()).collect();
    let tasks: Vec<_> = dirs.into_iter().map(|dir| {
        tokio::spawn(async move {
            if let Err(e) = fs::create_dir_all(&dir).await {
                safe_eprintln!("[启动] 创建存储目录 {} 失败: {} (服务将继续运行)", dir.display(), e);
            }
        })
    }).collect();
    for task in tasks {
        let _ = task.await;
    }
    if let Err(error) = cleanup_backtest_promotion_work_dirs(&backtest_store_dir).await {
        safe_eprintln!(
            "warning: 清理回测临时目录失败: {}",
            error
        );
    }

    // v2.0.0: 启动时校验市场公钥不是测试向量
    qrpc_runtime::plugin_market::assert_market_public_key_is_production();

    let state = new_app_state(graph_store_dir, run_store_dir, backtest_store_dir);
    // Block 5: 初始化告警规则
    alert_engine::init_alert_rules(&state).await;
    // Block 5: 从磁盘预热持久化数据
    warm_persisted_state(&state).await;
    if let Err(error) =
        cleanup_transient_backtest_records(state.transient_backtest_store_dir.as_ref()).await
    {
        safe_eprintln!(
            "warning: 清理过期回测目录失败: {}",
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
        .filter_map(|s| {
            let trimmed = s.trim();
            // v2.3.3 修复 S0-3: 拒绝通配符 origin 和无效 scheme
            if trimmed == "*" {
                safe_eprintln!("[CORS] 拒绝通配符 origin '*', 请使用明确的 http(s):// 地址");
                return None;
            }
            if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
                safe_eprintln!("[CORS] 拒绝非 http(s) origin: {}", trimmed);
                return None;
            }
            HeaderValue::from_str(trimmed).ok()
        })
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(cors_origins)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    // v2.0.1: HTTP 安全头中间件
    async fn security_headers_middleware(
        request: axum::extract::Request,
        next: axum::middleware::Next,
    ) -> axum::response::Response {
        let mut response = next.run(request).await;
        let headers = response.headers_mut();
        headers.insert(
            axum::http::HeaderName::from_static("x-content-type-options"),
            axum::http::HeaderValue::from_static("nosniff"),
        );
        headers.insert(
            axum::http::HeaderName::from_static("x-frame-options"),
            axum::http::HeaderValue::from_static("DENY"),
        );
        headers.insert(
            axum::http::HeaderName::from_static("referrer-policy"),
            axum::http::HeaderValue::from_static("strict-origin-when-cross-origin"),
        );
        // v2.1.x: 添加 CSP 和 HSTS 安全头
        headers.insert(
            axum::http::HeaderName::from_static("content-security-policy"),
            axum::http::HeaderValue::from_static(
                "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; connect-src 'self'; img-src 'self' data:; base-uri 'self';",
            ),
        );
        headers.insert(
            axum::http::HeaderName::from_static("strict-transport-security"),
            axum::http::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        );
        // v2.4.0: Permissions-Policy 限制浏览器 API 访问
        headers.insert(
            axum::http::HeaderName::from_static("permissions-policy"),
            axum::http::HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
        );
        response
    }

    let app = build_app_router(state.clone())
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB
        .layer(cors)
        .layer(axum::middleware::from_fn(security_headers_middleware))
        .layer(axum::middleware::from_fn(json_rejection_middleware))
        .layer(axum::middleware::from_fn(
            rate_limiter::rate_limit_middleware,
        ))
        .layer(axum::middleware::from_fn(
            auth_middleware::api_key_auth,
        ));

    // Block 5 P1-5 + P3-4: 审批超时 + 观察窗口后台任务
    // v1.0.2: AbortHandle 在进程退出时自动取消后台循环
    let expiry_state = state.clone();
    // v2.3.4: 后台任务 — 每次迭代用 catch_unwind 包裹，防止单次 panic 终止整个循环
    let bg_handle = tokio::spawn(async move {
        let mut tick: u64 = 0;
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            tick += 1;
            let current_tick = tick;
            let state_ref = &expiry_state;
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                let rt = tokio::runtime::Handle::current();
                rt.block_on(async {
                    process_expired_approvals(state_ref).await;
                    check_observation_windows(state_ref).await;
                    if current_tick.is_multiple_of(1440) {
                        backup::backup_permanent_storage().await;
                    }
                    storage_lifecycle::startup_storage_cleanup(std::path::Path::new("storage"));
                });
            }));
            if let Err(e) = result {
                safe_eprintln!(
                    "[后台任务] panic 已恢复: {} (tick {})",
                    e.downcast_ref::<String>().map(|s| s.as_str())
                        .or_else(|| e.downcast_ref::<&str>().copied())
                        .unwrap_or("未知 panic"),
                    current_tick
                );
            }
            if tick.is_multiple_of(10) {
                let _cutoff = current_time_ms().saturating_sub(24 * 3600 * 1000);
                const MAX_CACHED_RECORDS: usize = 500;
                // v2.3.3 P1-11: 扩展淘汰逻辑至全部 11 个 BTreeMap (原仅覆盖 runs/backtests/experiments)
                macro_rules! trim_map {
                    ($map:expr) => {
                        let mut guard = $map.write().await;
                        if guard.len() > MAX_CACHED_RECORDS {
                            let excess = guard.len() - MAX_CACHED_RECORDS;
                            let to_remove: Vec<_> = guard.iter().take(excess).map(|(k, _)| k.clone()).collect();
                            for k in to_remove { guard.remove(&k); }
                        }
                    };
                }
                trim_map!(expiry_state.runs);
                trim_map!(expiry_state.backtests);
                trim_map!(expiry_state.experiments);
                trim_map!(expiry_state.parameter_mutations);
                trim_map!(expiry_state.ai_proposals);
                trim_map!(expiry_state.hotswap_records);
                trim_map!(expiry_state.approval_records);
                trim_map!(expiry_state.sandbox_reports);
                trim_map!(expiry_state.alert_firings);
                trim_map!(expiry_state.snapshots);
                trim_map!(expiry_state.chaos_experiments);
            }
        }
    });

    let port: u16 = match env::var("QUANTPILOT_PORT") {
        Ok(val) => val.parse().unwrap_or_else(|e| {
            safe_eprintln!("[启动] QUANTPILOT_PORT 值 '{}' 无效 ({}), 使用默认 3000", val, e);
            3000
        }),
        Err(_) => 3000,
    };
    if port == 0 {
        anyhow::bail!("端口 0 是保留端口, 请使用 1-65535 范围内的有效端口");
    }
    // v2.0.1: 绑定地址可通过环境变量配置，容器部署需设为 0.0.0.0
    let bind_host = std::env::var("QUANTPILOT_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1".to_string());
    let bind_ip: std::net::Ipv4Addr = bind_host.parse().unwrap_or_else(|_| {
        safe_eprintln!("[启动] QUANTPILOT_BIND_ADDR 无效 ({}), 回退到 127.0.0.1", bind_host);
        [127, 0, 0, 1].into()
    });
    let addr = SocketAddr::from((bind_ip, port));
    println!("QuantPilot v{} API → http://{}", env!("CARGO_PKG_VERSION"), addr);
    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        anyhow::anyhow!("端口 {} 已被占用，请检查是否有其他 QuantPilot 实例在运行: {}", port, e)
    })?;
    // v1.1.11: 优雅关闭 — 监听 ctrl_c 信号
    let shutdown_signal = async {
        let _ = tokio::signal::ctrl_c().await;
        safe_eprintln!("[shutdown] 收到终止信号，正在优雅关闭...");
    };
    #[cfg(unix)]
    let sigterm = {
        let mut sig = tokio::signal::unix::signal(
            tokio::signal::unix::SignalKind::terminate()
        ).expect("无法注册 SIGTERM 处理器");
        async move { sig.recv().await; }
    };
    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        result = axum::serve(listener, app) => { result?; }
        _ = shutdown_signal => {}
        _ = sigterm => {}
    }
    // v2.3.4: 关闭前尝试将内存状态持久化 (runs/backtests/experiments 缓存)
    flush_volatile_state(&state).await;
    bg_handle.abort();
    // v2.1.0: 等待后台任务完成清理 (30s超时防挂起)
    let _ = tokio::time::timeout(std::time::Duration::from_secs(30), bg_handle).await;
    eprintln!("[shutdown] QuantPilot 已停止");
    Ok(())
}

/// v2.3.4: 关闭时将内存缓存中尚未持久化的记录写入磁盘
async fn flush_volatile_state(_state: &AppState) {
    // 大部分运行时记录在创建时已持久化，此处作为关闭安全网
    let _ = crate::storage_lifecycle::startup_storage_cleanup(std::path::Path::new("storage"));
    safe_eprintln!("[shutdown] 内存状态已刷盘");
}

async fn json_rejection_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let response = next.run(req).await;
    // 覆盖 Axum 默认的 JSON 解析错误 (422/400/415), 统一返回中文 JSON
    let status = response.status();
    if status == StatusCode::UNPROCESSABLE_ENTITY
        || status == StatusCode::BAD_REQUEST
        || status == StatusCode::UNSUPPORTED_MEDIA_TYPE
    {
        let body = axum::Json(serde_json::json!({
            "error": "bad_request",
            "message": "请求格式错误: 请使用 Content-Type: application/json 并确保请求体为有效 JSON"
        }));
        return (StatusCode::BAD_REQUEST, body).into_response();
    }
    response
}

// v2.1.0: 启动时恢复因崩溃残留的 .bak 文件
async fn recover_stale_bak_files(graph_store_dir: &FsPath) {
    let Ok(mut entries) = fs::read_dir(graph_store_dir).await else {
        return;
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("bak") {
            if let Some(stem) = path.file_stem() {
                let json_path = path.with_file_name(stem);
                if !fs::try_exists(&json_path).await.unwrap_or(true) {
                    safe_eprintln!("[startup] 恢复残留 bak 文件: {}", path.display());
                    let _ = fs::rename(&path, &json_path).await;
                } else {
                    // 主文件已存在，bak 是残留，安全删除
                    let _ = fs::remove_file(&path).await;
                }
            }
        }
    }
}

async fn warm_persisted_state(state: &AppState) {
    // v2.1.0: 启动时恢复残留的 .bak 文件（上次保存崩溃残留）
    recover_stale_bak_files(state.graph_store_dir.as_ref()).await;
    // 从磁盘加载审批记录
    if let Ok(mut entries) = fs::read_dir(state.approval_store_dir.as_ref()).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            // v2.0.1: 仅加载 .json 文件, 跳过 .tmp/.bak 等残留
            if entry.path().extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(data) = fs::read(entry.path()).await {
                if let Ok(record) = serde_json::from_slice::<RuntimeApprovalRecord>(&data) {
                    let key = auth::scoped_key(&auth::UserId(0), &record.approval_id);
                    state
                        .approval_records
                        .write()
                        .await
                        .insert(key, record);
                } else {
                    safe_eprintln!("[startup] 跳过不可读的审批记录: {}", entry.path().display());
                }
            }
        }
    }
    // 从磁盘加载快照
    if let Ok(mut entries) = fs::read_dir(state.snapshot_store_dir.as_ref()).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(data) = fs::read(entry.path()).await {
                if let Ok(snapshot) = serde_json::from_slice::<DeploymentSignatureSnapshot>(&data) {
                    let key = auth::scoped_key(&auth::UserId(0), &snapshot.snapshot_id);
                    state
                        .snapshots
                        .write()
                        .await
                        .insert(key, snapshot);
                }
            }
        }
    }
    // 从磁盘加载告警 firing 状态
    if let Ok(mut entries) = fs::read_dir(state.alert_store_dir.as_ref()).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(data) = fs::read(entry.path()).await {
                if let Ok(firing) = serde_json::from_slice::<AlertFiring>(&data) {
                    let key = auth::scoped_key(&auth::UserId(0), &firing.firing_id);
                    state
                        .alert_firings
                        .write()
                        .await
                        .insert(key, firing);
                }
            }
        }
    }
    // 从磁盘加载沙箱报告
    if let Ok(mut entries) = fs::read_dir(state.sandbox_report_store_dir.as_ref()).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(data) = fs::read(entry.path()).await {
                if let Ok(report) = serde_json::from_slice::<SandboxVerificationReport>(&data) {
                    let key = auth::scoped_key(&auth::UserId(0), &report.proposal_id);
                    state
                        .sandbox_reports
                        .write()
                        .await
                        .insert(key, report);
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
    safe_eprintln!(
        "[startup] 已预热状态: {} 审批单, {} 快照, {} 告警, {} 沙箱报告, {} 混沌实验",
        state.approval_records.read().await.len(),
        state.snapshots.read().await.len(),
        state.alert_firings.read().await.len(),
        state.sandbox_reports.read().await.len(),
        state.chaos_experiments.read().await.len(),
    );
}

// Block 5 P1-5: 审批超时自动处理
// v2.3.3 修复 S0-1: 拆分为两阶段避免嵌套写锁死锁
// 阶段1: 在 approval_records 写锁内收集过期变更, 释放锁后批量持久化
// 阶段2: 在 ai_proposals 写锁内更新关联提案状态
async fn process_expired_approvals(state: &AppState) {
    let now_ms = current_time_ms();
    // 阶段1: 收集过期审批并标记, 记录需更新的 proposal_id 列表
    let expired_proposal_ids: Vec<String> = {
        let mut approvals = state.approval_records.write().await;
        let mut ids = Vec::new();
        for approval in approvals.values_mut() {
            if (approval.review_state == RuntimeApprovalReviewState::Pending
                || approval.review_state == RuntimeApprovalReviewState::UnderReview)
                && now_ms > approval.expires_at_ms {
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
                    ids.push(approval.proposal_id.clone());
                    // 持久化在锁内完成, 但使用克隆数据避免长时间持锁
                    if let Some(json) = serde_json::to_vec_pretty(&*approval).ok() {
                        let dir = state.approval_store_dir.to_path_buf();
                        let id = approval.approval_id.clone();
                        let approval_dir = dir.clone();
                        let approval_id = id.clone();
                        let approval_json = json.clone();
                        // spawn 到后台执行 I/O, 不持锁等待
                        tokio::spawn(async move {
                            let _ = tokio::fs::create_dir_all(&approval_dir).await;
                            let tmp = approval_dir.join(format!("{}.json.tmp", approval_id));
                            let final_path = approval_dir.join(format!("{}.json", approval_id));
                            let _ = tokio::fs::write(&tmp, &approval_json).await;
                            let _ = tokio::fs::rename(&tmp, &final_path).await;
                        });
                    }
                }
        }
        ids
    }; // approval_records 写锁在此释放
    // 阶段2: 在 ai_proposals 写锁内更新提案状态 (独立锁, 无嵌套)
    if !expired_proposal_ids.is_empty() {
        let mut proposals = state.ai_proposals.write().await;
        for proposal_id in &expired_proposal_ids {
            if let Some(proposal) = proposals.get_mut(proposal_id) {
                proposal.status = RuntimeAiProposalStatus::Expired;
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
    if !tolerance_ms.is_finite() || tolerance_ms <= 0.0 {
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
mod tests_backend;