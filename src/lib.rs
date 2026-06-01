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
mod api_test_scenario;
pub mod app_router;
pub mod app_runtime_helpers;
pub mod auth;
mod auth_middleware;
pub mod backend;
mod backtest_artifacts;
mod backtest_compare;
mod backtest_compare_core;
mod backtest_compare_narrative;
mod backtest_compare_types;
mod backup;
mod capability_api;
mod chaos_experiment;
mod cli_support;
mod collaboration;
pub mod compile_api;
mod compile_artifact_builders;
mod compile_diagnostics;
mod credential_api;
pub mod credential_vault;
mod error_codes;
mod formal_quantscript_authoring_types;
mod frontend_api_types;
mod frontend_runtime_mapping;
mod graph_api;
mod graph_version_compare;
mod hotswap_api;
pub mod migration_sender;
mod rate_limiter;
mod runbook;
mod runtime;
mod runtime_diagnostics;
mod runtime_event_projection;
pub mod runtime_persistence;
mod runtime_response_mapping;
pub mod runtime_validation;
pub mod safe_log;
mod sandbox_verification;
mod snapshot_service;
pub mod storage_lifecycle;
mod strategy_config_api;
pub mod system;
mod test_runner;

use anyhow::{bail, Context};
use async_stream::stream;
use axum::{
    extract::{Path, State},
    http::StatusCode,
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
    path::{Path as FsPath, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{fs, sync::RwLock, time::sleep};

use api_errors::*;
pub use app_router::*;
pub use app_runtime_helpers::*;
pub(crate) use backend::graph_compile::quantscript_graph::{
    attach_quantscript_artifacts, build_compile_runtime_targets_from_graph,
    convert_graph_json_to_script_module, generate_quantscript_from_graph_value,
    parse_graph_quantscript_source,
};
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
#[cfg(test)]
use cli_support::*;
use collaboration::*;
use compile_api::*;
use compile_artifact_builders::*;
use compile_diagnostics::*;
use formal_quantscript_authoring_types::*;
use frontend_api_types::*;
use frontend_runtime_mapping::*;
use graph_version_compare::*;
use runtime_diagnostics::*;
use runtime_event_projection::*;
use runtime_persistence::*;
use runtime_response_mapping::*;
use runtime_validation::*;
use strategy_config_api::*;
pub use system::entry::backend_process::run_server;

const RUN_WINDOW_MS: u64 = 5_000;
const SSE_EVENT_DELAY_MS: u64 = 350;
const BACKTEST_DETERMINISTIC_SEED: u64 = 7;
const CAPABILITY_API_VERSION: &str = "quantpilot-capabilities/v1";
const CAPABILITY_SCHEMA_VERSION: &str = "quantpilot/capabilities-schema/v1";
const CAPABILITY_PERMISSION_MODEL_VERSION: &str = "quantpilot/permission-boundary/v1";
const CAPABILITY_VERSIONING_MODEL_VERSION: &str = "quantpilot/versioning-model/v1";
const RUNTIME_GOVERNANCE_SCHEMA_VERSION: &str = "quantpilot/runtime-governance/v1";
const RUNTIME_CHAIN_STAGES: [&str; 6] = ["data", "intent", "agent", "risk", "execution", "fill"];

const DECLARED_FRONTEND_MODULE_KEYS: [&str; 16] = [
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
    "v4.machine.param",
    "v4.transition.guard",
];

const SUPPORTED_FRONTEND_MODULE_KEYS: [&str; 16] = [
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
    "v4.machine.param",
    "v4.transition.guard",
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
                self.mutation_activation_scheduled_count
                    .fetch_add(1, Ordering::Relaxed);
            }
            RuntimeParameterMutationStatus::Activated => {
                self.mutation_activation_applied_count
                    .fetch_add(1, Ordering::Relaxed);
            }
            RuntimeParameterMutationStatus::ActivationFailed => {
                self.mutation_activation_failed_count
                    .fetch_add(1, Ordering::Relaxed);
            }
            RuntimeParameterMutationStatus::SafeWindowDenied => {
                self.mutation_safe_window_denied_count
                    .fetch_add(1, Ordering::Relaxed);
            }
            RuntimeParameterMutationStatus::RollbackScheduled => {
                self.mutation_rollback_scheduled_count
                    .fetch_add(1, Ordering::Relaxed);
            }
            RuntimeParameterMutationStatus::RolledBack => {
                self.mutation_rollback_applied_count
                    .fetch_add(1, Ordering::Relaxed);
            }
            RuntimeParameterMutationStatus::RollbackFailed => {
                self.mutation_rollback_failed_count
                    .fetch_add(1, Ordering::Relaxed);
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
    workspace: WorkspaceCapabilitySummary,
    ui_actions: UiActionCapabilitySummary,
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

#[derive(Debug, Serialize)]
struct WorkspaceCapabilitySummary {
    surfaces: Vec<UiCapabilityEntry>,
}

#[derive(Debug, Serialize)]
struct UiActionCapabilitySummary {
    actions: Vec<UiCapabilityEntry>,
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

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
struct UiCapabilityEntry {
    key: &'static str,
    status: CapabilitySupportStatus,
    reason: Option<&'static str>,
    source: &'static str,
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
        serde_json::to_value(authoring_view).context("序列化 QuantScript 编写视图失败")?,
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
