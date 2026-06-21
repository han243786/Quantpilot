use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct GraphListEntry {
    pub(super) graph_id: String,
    pub(super) name: String,
    pub(super) updated_at: u64,
    pub(super) path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct DeleteGraphResponse {
    pub(super) graph_id: String,
    pub(super) deleted: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct GraphVersionEntry {
    pub(super) graph_id: String,
    pub(super) version_id: String,
    pub(super) name: String,
    pub(super) updated_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) version_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) save_note: Option<String>,
    pub(super) node_count: usize,
    pub(super) edge_count: usize,
    pub(super) path: String,
    pub(super) quantscript_path: String,
    pub(super) is_latest: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ActorIdentity {
    pub(super) actor_id: String,
    pub(super) display_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
pub(super) struct GraphCollaborationMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) owner: Option<ActorIdentity>,
    #[serde(default)]
    pub(super) editors: Vec<ActorIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) last_saved_by: Option<ActorIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) last_run_actor: Option<ActorIdentity>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum GraphAuditAction {
    GraphSaved,
    GraphRestored,
    RunCreated,
    BacktestCreated,
    ExperimentCreated,
    GraphDeleted, // v1.1.9
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct GraphAuditEntry {
    pub(super) audit_id: String,
    pub(super) graph_id: String,
    pub(super) action: GraphAuditAction,
    pub(super) created_at_ms: u64,
    pub(super) actor: ActorIdentity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) target_id: Option<String>,
    pub(super) summary: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum GraphVersionDiffStatus {
    Same,
    Different,
    Added,
    Removed,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct GraphVersionDiffRow {
    pub(super) key: String,
    pub(super) label: String,
    pub(super) status: GraphVersionDiffStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) left_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) right_value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct GraphVersionCollectionDiff {
    pub(super) left_count: usize,
    pub(super) right_count: usize,
    #[serde(default)]
    pub(super) added_ids: Vec<String>,
    #[serde(default)]
    pub(super) removed_ids: Vec<String>,
    #[serde(default)]
    pub(super) changed_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct GraphVersionConfigDiffEntry {
    pub(super) node_id: String,
    pub(super) node_name: String,
    pub(super) field_path: String,
    pub(super) status: GraphVersionDiffStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) left_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) right_value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct GraphVersionCompareResponse {
    pub(super) graph_id: String,
    pub(super) left: GraphVersionEntry,
    pub(super) right: GraphVersionEntry,
    #[serde(default)]
    pub(super) metadata_rows: Vec<GraphVersionDiffRow>,
    pub(super) node_diff: GraphVersionCollectionDiff,
    pub(super) edge_diff: GraphVersionCollectionDiff,
    #[serde(default)]
    pub(super) config_diffs: Vec<GraphVersionConfigDiffEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) strategy_config_diff: Option<StrategyConfigDiffReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) strategy_config_evidence_diff: Option<StrategyConfigEvidenceDiffReport>,
    pub(super) has_changes: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RevealGraphResponse {
    pub(super) graph_id: String,
    pub(super) path: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FrontendRunRequest {
    #[serde(default)]
    pub(super) actor: Option<ActorIdentity>,
    #[serde(default)]
    pub(super) capability_context: Option<FrontendCapabilityContext>,
    pub(super) runtime_config: FrontendRuntimeConfig,
    #[serde(default)]
    pub(super) graph_json: Option<Value>,
    #[serde(default)]
    pub(super) backtest_options: FrontendBacktestOptions,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct FrontendRuntimeConfig {
    pub(super) metadata: FrontendMetadata,
    pub(super) data_sources: Vec<FrontendNodeConfig>,
    pub(super) intent_generators: Vec<FrontendIntentConfig>,
    pub(super) agents: Vec<FrontendAgentConfig>,
    pub(super) risk_controls: Vec<FrontendRiskConfig>,
    pub(super) executions: Vec<FrontendExecutionConfig>,
    pub(super) runtime_control: Option<FrontendNodeConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
// 注：未加 deny_unknown_fields，因前端发送 phantom `volatility` 字段需兼容
pub(super) struct FrontendBacktestOptions {
    pub(super) replay_source: Option<FrontendBacktestReplaySource>,
    pub(super) replay_mode: Option<String>,
    pub(super) execution_assumptions: Option<FrontendExecutionAssumptionOverrides>,
    pub(super) runtime_kind: Option<String>,
    #[serde(default)]
    pub(super) symbols: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub(super) struct FrontendExecutionAssumptionOverrides {
    pub(super) fee_bps: Option<f64>,
    pub(super) slippage_bps: Option<f64>,
    pub(super) latency_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub(super) struct FrontendExecutionAssumptionSweepGrid {
    #[serde(default)]
    pub(super) fee_bps: Vec<f64>,
    #[serde(default)]
    pub(super) slippage_bps: Vec<f64>,
    #[serde(default)]
    pub(super) latency_ms: Vec<u64>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct FrontendExperimentRequest {
    #[serde(default)]
    pub(super) experiment_name: Option<String>,
    #[serde(default)]
    pub(super) actor: Option<ActorIdentity>,
    #[serde(default)]
    pub(super) capability_context: Option<FrontendCapabilityContext>,
    pub(super) runtime_config: FrontendRuntimeConfig,
    #[serde(default)]
    pub(super) graph_json: Option<Value>,
    #[serde(default)]
    pub(super) backtest_options: FrontendBacktestOptions,
    #[serde(default)]
    pub(super) parameter_grid: FrontendExecutionAssumptionSweepGrid,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum FrontendBacktestReplaySource {
    HistoricalReplay,
    DeterministicMock,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct FrontendCapabilityContext {
    pub(super) schema_hash: String,
    pub(super) permission_boundary: PermissionBoundarySnapshot,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct FrontendMetadata {
    pub(super) graph_id: String,
    pub(super) compile_id: String,
    pub(super) name: String,
    pub(super) version: String,
    pub(super) mode: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct FrontendNodeConfig {
    pub(super) id: String,
    pub(super) module_key: String,
    pub(super) name: String,
    pub(super) config: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct FrontendIntentConfig {
    pub(super) id: String,
    pub(super) module_key: String,
    pub(super) name: String,
    pub(super) config: Value,
    pub(super) input_refs: Vec<FrontendInputRef>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct FrontendAgentConfig {
    pub(super) id: String,
    pub(super) module_key: String,
    pub(super) name: String,
    pub(super) config: Value,
    pub(super) intent_refs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct FrontendRiskConfig {
    pub(super) id: String,
    pub(super) module_key: String,
    pub(super) name: String,
    pub(super) config: Value,
    pub(super) agent_refs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct FrontendExecutionConfig {
    pub(super) id: String,
    pub(super) module_key: String,
    pub(super) name: String,
    pub(super) config: Value,
    pub(super) risk_ref: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct FrontendInputRef {
    pub(super) source_id: String,
    pub(super) source_port: String,
    pub(super) target_port: String,
}

#[derive(Debug, Serialize)]
pub(super) struct BacktestRunResponse {
    pub(super) backtest_id: String,
    pub(super) graph_id: String,
    pub(super) compile_id: String,
    pub(super) protocol_name: String,
    pub(super) config_hash: String,
    pub(super) event_count: usize,
    pub(super) account: AccountSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) execution_assumptions: Option<ExecutionAssumptionsModule>,
    pub(super) backtest_artifacts: BacktestArtifactViews,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct FrontendRuntimeEvent {
    pub(super) event_id: String,
    pub(super) event_type: String,
    pub(super) source_id: String,
    pub(super) node_id: String,
    pub(super) event_time_ms: u64,
    pub(super) severity: String,
    pub(super) summary: String,
    pub(super) payload: Value,
    #[serde(default)]
    pub(super) envelope: RuntimeEventEnvelope,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct RuntimeEventEnvelope {
    pub(super) event_id: String,
    pub(super) event_type: String,
    pub(super) stage: RuntimeEventStage,
    pub(super) run_id: String,
    pub(super) sequence_no: u64,
    pub(super) occurred_at_ms: u64,
    pub(super) ingested_at_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) trace_id: Option<String>,
    pub(super) module_key: String,
    pub(super) strategy_version: String,
    pub(super) parameter_version: String,
    pub(super) deployment_revision: String,
    pub(super) capability_hash: String,
    pub(super) mode: String,
    pub(super) severity: String,
    pub(super) retention_class: RuntimeEventRetentionClass,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) reason_code: Option<String>,
    pub(super) payload_version: u16,
}

impl Default for RuntimeEventEnvelope {
    fn default() -> Self {
        Self {
            event_id: String::new(),
            event_type: String::new(),
            stage: RuntimeEventStage::System,
            run_id: String::new(),
            sequence_no: 0,
            occurred_at_ms: 0,
            ingested_at_ms: 0,
            trace_id: None,
            module_key: String::new(),
            strategy_version: "unknown".to_string(),
            parameter_version: "unknown".to_string(),
            deployment_revision: "unknown".to_string(),
            capability_hash: "unknown".to_string(),
            mode: "unknown".to_string(),
            severity: "Info".to_string(),
            retention_class: RuntimeEventRetentionClass::Summary,
            reason_code: None,
            payload_version: 1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct PermissionBoundarySnapshot {
    pub(super) model_version: String,
    pub(super) execution_owner_module: String,
    pub(super) live_execution_allowed: bool,
    pub(super) ai_write_policy: String,
    pub(super) plugin_network_default: String,
    pub(super) non_execution_order_access: String,
}

impl Default for PermissionBoundarySnapshot {
    fn default() -> Self {
        Self {
            model_version: "quantpilot/permission-boundary/v1".to_string(),
            execution_owner_module: "builtin.execution.paper".to_string(),
            live_execution_allowed: false,
            ai_write_policy: "proposal_only".to_string(),
            plugin_network_default: "deny".to_string(),
            non_execution_order_access: "deny".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct RuntimeGovernanceSnapshot {
    pub(super) schema_version: String,
    #[serde(default = "default_governance_source")]
    pub(super) governance_source: String,
    pub(super) capability_hash: String,
    pub(super) strategy_version: String,
    pub(super) parameter_version: String,
    pub(super) deployment_revision: String,
    pub(super) permission_boundary: PermissionBoundarySnapshot,
}

fn default_governance_source() -> String {
    "legacy_default".to_string()
}

impl Default for RuntimeGovernanceSnapshot {
    fn default() -> Self {
        Self {
            schema_version: "quantpilot/runtime-governance/v1".to_string(),
            governance_source: default_governance_source(),
            capability_hash: "unknown".to_string(),
            strategy_version: "unknown".to_string(),
            parameter_version: "unknown".to_string(),
            deployment_revision: "unknown".to_string(),
            permission_boundary: PermissionBoundarySnapshot::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeTimelineGovernanceIdentity {
    pub(super) capability_hash: String,
    pub(super) deployment_revision: String,
    pub(super) strategy_version: String,
    pub(super) parameter_version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum RuntimeTimelineCompactability {
    Retain,
    Summarize,
    DropCandidate,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeTimelineItem {
    pub(super) timeline_item_version: u16,
    pub(super) event_id: String,
    pub(super) event_type: String,
    pub(super) sequence_no: u64,
    pub(super) occurred_at_ms: u64,
    pub(super) ingested_at_ms: u64,
    pub(super) stage: RuntimeEventStage,
    pub(super) retention_class: RuntimeEventRetentionClass,
    pub(super) severity: String,
    pub(super) module_key: String,
    pub(super) node_id: String,
    pub(super) summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) reason_code: Option<String>,
    pub(super) governance: RuntimeTimelineGovernanceIdentity,
    pub(super) payload_version: u16,
    pub(super) compactability: RuntimeTimelineCompactability,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeRetainedKeyEventIndex {
    pub(super) index_version: u16,
    pub(super) policy_version: String,
    pub(super) source_event_count: usize,
    pub(super) retained_event_count: usize,
    pub(super) key_event_count: usize,
    pub(super) system_event_count: usize,
    #[serde(default)]
    pub(super) entries: Vec<RuntimeTimelineItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeCompactEvidenceProjection {
    pub(super) projection_version: u16,
    pub(super) policy_version: String,
    pub(super) source_event_count: usize,
    pub(super) retained_event_count: usize,
    pub(super) dropped_event_count: usize,
    #[serde(default)]
    pub(super) dropped_by_retention: std::collections::BTreeMap<String, usize>,
    #[serde(default)]
    pub(super) dropped_by_stage: std::collections::BTreeMap<String, usize>,
    pub(super) key_event_count: usize,
    pub(super) system_event_count: usize,
    pub(super) governance: RuntimeTimelineGovernanceIdentity,
    #[serde(default)]
    pub(super) entries: Vec<RuntimeTimelineItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct OpenOrderSummary {
    pub(super) order_id: String,
    pub(super) side: String,
    pub(super) remaining_qty: f64,
    pub(super) limit_price: Option<f64>,
    pub(super) reserved_cash: f64,
    pub(super) reserved_qty: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct AccountSummary {
    #[serde(default)]
    pub(super) equity_estimate: f64,
    pub(super) cash_balance: f64,
    pub(super) available_cash_balance: f64,
    pub(super) frozen_cash_balance: f64,
    pub(super) total_leverage: f64,
    pub(super) total_gross_notional: f64,
    pub(super) total_net_notional: f64,
    pub(super) positions: usize,
    pub(super) open_order_count: usize,
    pub(super) open_orders: Vec<OpenOrderSummary>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct RunRecord {
    pub(super) run_id: String,
    pub(super) graph_id: String,
    pub(super) compile_id: String,
    pub(super) created_at_ms: u64,
    pub(super) events: Vec<FrontendRuntimeEvent>,
    pub(super) account: AccountSummary,
    pub(super) session: SessionOutput,
    #[serde(default)]
    pub(super) governance: RuntimeGovernanceSnapshot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) actor: Option<ActorIdentity>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct BacktestRecord {
    pub(super) backtest_id: String,
    pub(super) graph_id: String,
    pub(super) compile_id: String,
    pub(super) created_at_ms: u64,
    pub(super) protocol_name: String,
    pub(super) config_hash: String,
    pub(super) account: AccountSummary,
    pub(super) events: Vec<FrontendRuntimeEvent>,
    pub(super) backtest: BacktestOutput,
    #[serde(default)]
    pub(super) backtest_spec: Option<BacktestSpec>,
    #[serde(default)]
    pub(super) artifacts: Option<CompileArtifactBundle>,
    #[serde(default)]
    pub(super) backtest_artifacts: Option<BacktestArtifactViews>,
    #[serde(default)]
    pub(super) governance: RuntimeGovernanceSnapshot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) actor: Option<ActorIdentity>,
    /// v1.1.1: 非关键文件加载失败时为 true，UI 中标记"部分数据不可用"
    #[serde(default)]
    pub(super) degraded: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct RunStatusResponse {
    pub(super) run_id: String,
    pub(super) graph_id: String,
    pub(super) compile_id: String,
    pub(super) event_count: usize,
    pub(super) account: AccountSummary,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SaveGraphRequest {
    pub(super) graph: Value,
    #[serde(default)]
    pub(super) version_label: Option<String>,
    #[serde(default)]
    pub(super) save_note: Option<String>,
    #[serde(default)]
    pub(super) actor: Option<ActorIdentity>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct GraphMutationActorRequest {
    #[serde(default)]
    pub(super) actor: Option<ActorIdentity>,
}

#[derive(Debug, Serialize)]
pub(super) struct SaveGraphResponse {
    pub(super) graph_id: String,
    pub(super) version_id: String,
    pub(super) saved_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) version_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) save_note: Option<String>,
    pub(super) path: String,
    pub(super) quantscript_path: String,
    pub(super) collaboration: GraphCollaborationMetadata,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ParseGraphQuantScriptRequest {
    pub(super) source: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CompileRuntimeRequest {
    #[serde(default, rename = "compile_id")]
    pub(super) _compile_id: Option<String>,
    #[serde(default, rename = "capability_context")]
    pub(super) _capability_context: Option<Value>,
    pub(super) runtime_config: FrontendRuntimeConfig,
    #[serde(default, rename = "graph_json")]
    pub(super) _graph_json: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CompileStrategyIrRequest {
    pub(super) graph_id: String,
    pub(super) compile_id: String,
    pub(super) strategy_ir: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub(super) struct CompileRuntimeTargets {
    #[serde(default)]
    pub(super) source_to_node: BTreeMap<String, String>,
    #[serde(default)]
    pub(super) runtime_node_id: Option<String>,
    #[serde(default)]
    pub(super) execution_node_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CompileFormalQuantScriptRequest {
    pub(super) graph_id: String,
    pub(super) compile_id: String,
    pub(super) source: String,
    pub(super) runtime_template: FrontendRuntimeConfig,
    #[serde(default)]
    pub(super) universe_snapshot: Option<UniverseSnapshot>,
    #[serde(default)]
    pub(super) runtime_targets: CompileRuntimeTargets,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct CompileCounts {
    pub(super) data_sources: usize,
    pub(super) intent_generators: usize,
    pub(super) agents: usize,
    pub(super) risk_controls: usize,
    pub(super) executions: usize,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(super) enum CompileDiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum CompileDiagnosticTargetScope {
    Graph,
    Node,
}

#[derive(Debug, Serialize, Clone)]
pub(super) struct CompileDiagnosticTarget {
    pub(super) scope: CompileDiagnosticTargetScope,
    pub(super) node_id: Option<String>,
    pub(super) edge_id: Option<String>,
    pub(super) field: Option<String>,
    pub(super) label: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub(super) struct CompileDiagnostic {
    pub(super) code: String,
    pub(super) severity: CompileDiagnosticSeverity,
    pub(super) message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) span_label: Option<String>,
    pub(super) target: Option<CompileDiagnosticTarget>,
    pub(super) hint: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct CompileRuntimeResponse {
    pub(super) graph_id: String,
    pub(super) compile_id: String,
    pub(super) compilable: bool,
    pub(super) protocol_name: String,
    pub(super) config_hash: String,
    pub(super) core_ir: qrpc_core_ir::CoreStrategyIr,
    pub(super) artifacts: CompileArtifactBundle,
    pub(super) counts: CompileCounts,
    pub(super) diagnostics: Vec<CompileDiagnostic>,
    pub(super) runtime_config: FrontendRuntimeConfig,
    pub(super) runtime_targets: CompileRuntimeTargets,
}

#[derive(Debug, Serialize)]
pub(super) struct CompileStrategyIrResponse {
    pub(super) graph_id: String,
    pub(super) compile_id: String,
    pub(super) compilable: bool,
    pub(super) core_ir: qrpc_core_ir::CoreStrategyIr,
    pub(super) diagnostics: Vec<CompileDiagnostic>,
}

#[derive(Debug, Serialize)]
pub(super) struct RunListItem {
    pub(super) run_id: String,
    pub(super) graph_id: String,
    pub(super) compile_id: String,
    pub(super) created_at_ms: u64,
    pub(super) event_count: usize,
    pub(super) account: AccountSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) actor: Option<ActorIdentity>,
}

#[derive(Debug, Serialize)]
pub(super) struct RunDetailResponse {
    pub(super) run_id: String,
    pub(super) graph_id: String,
    pub(super) compile_id: String,
    pub(super) created_at_ms: u64,
    pub(super) event_count: usize,
    pub(super) account: AccountSummary,
    pub(super) events: Vec<FrontendRuntimeEvent>,
    #[serde(default)]
    pub(super) timeline: Vec<RuntimeTimelineItem>,
    pub(super) retained_key_event_index: RuntimeRetainedKeyEventIndex,
    pub(super) compact_evidence: RuntimeCompactEvidenceProjection,
    pub(super) runtime_diagnostics: RuntimeDiagnosticsPayload,
    pub(super) session: SessionOutput,
    pub(super) governance: RuntimeGovernanceSnapshot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) actor: Option<ActorIdentity>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct RuntimeDiagnosticsFieldRow {
    pub(super) key: String,
    pub(super) label: String,
    pub(super) value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct RuntimeDiagnosticsEventSummary {
    pub(super) event_id: String,
    pub(super) event_type: String,
    pub(super) label: String,
    pub(super) summary: String,
    pub(super) tone: String,
    pub(super) severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) event_time_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct RuntimeDiagnosticsNodeSummary {
    pub(super) node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) latest_event_type: Option<String>,
    pub(super) latest_event_label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) latest_event_time_ms: Option<u64>,
    pub(super) event_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct RuntimeDiagnosticsNodeDetail {
    pub(super) node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) latest_event: Option<RuntimeDiagnosticsEventSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) explanation_summary: Option<String>,
    #[serde(default)]
    pub(super) latest_input_rows: Vec<RuntimeDiagnosticsFieldRow>,
    #[serde(default)]
    pub(super) latest_output_rows: Vec<RuntimeDiagnosticsFieldRow>,
    #[serde(default)]
    pub(super) explanation_rows: Vec<RuntimeDiagnosticsFieldRow>,
    #[serde(default)]
    pub(super) data_quality_rows: Vec<RuntimeDiagnosticsFieldRow>,
    #[serde(default)]
    pub(super) risk_detail_rows: Vec<RuntimeDiagnosticsFieldRow>,
    #[serde(default)]
    pub(super) order_detail_rows: Vec<RuntimeDiagnosticsFieldRow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) latest_notice: Option<RuntimeDiagnosticsEventSummary>,
    #[serde(default)]
    pub(super) recent_events: Vec<RuntimeDiagnosticsEventSummary>,
    pub(super) event_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct RuntimeDiagnosticsPayload {
    pub(super) source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) default_selected_node_id: Option<String>,
    #[serde(default)]
    pub(super) active_nodes: Vec<RuntimeDiagnosticsNodeSummary>,
    #[serde(default)]
    pub(super) node_details: BTreeMap<String, RuntimeDiagnosticsNodeDetail>,
}

#[derive(Debug, Serialize)]
pub(super) struct BacktestFilterMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) replay_source: Option<ArtifactBacktestReplaySource>,
    #[serde(default)]
    pub(super) dataset_labels: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) execution_assumptions_tag: Option<ExecutionAssumptionsTag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) started_at_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) ended_at_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub(super) struct BacktestListItem {
    pub(super) backtest_id: String,
    pub(super) graph_id: String,
    pub(super) compile_id: String,
    pub(super) created_at_ms: u64,
    pub(super) protocol_name: String,
    pub(super) config_hash: String,
    pub(super) event_count: usize,
    pub(super) account: AccountSummary,
    pub(super) summary: qrpc_core::BacktestSummary,
    pub(super) filters: BacktestFilterMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) actor: Option<ActorIdentity>,
}

#[derive(Debug, Serialize)]
pub(super) struct BacktestDetailResponse {
    pub(super) backtest_id: String,
    pub(super) graph_id: String,
    pub(super) compile_id: String,
    pub(super) created_at_ms: u64,
    pub(super) protocol_name: String,
    pub(super) config_hash: String,
    pub(super) event_count: usize,
    pub(super) account: AccountSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) execution_assumptions: Option<ExecutionAssumptionsModule>,
    pub(super) runtime_diagnostics: RuntimeDiagnosticsPayload,
    #[serde(default)]
    pub(super) timeline: Vec<RuntimeTimelineItem>,
    pub(super) retained_key_event_index: RuntimeRetainedKeyEventIndex,
    pub(super) compact_evidence: RuntimeCompactEvidenceProjection,
    pub(super) backtest_artifacts: BacktestArtifactViews,
    pub(super) governance: RuntimeGovernanceSnapshot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) actor: Option<ActorIdentity>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct ExperimentVariantSummary {
    pub(super) variant_id: String,
    pub(super) backtest_id: String,
    pub(super) created_at_ms: u64,
    pub(super) fee_bps: f64,
    pub(super) slippage_bps: f64,
    pub(super) latency_ms: u64,
    pub(super) summary: qrpc_core::BacktestSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) execution_assumptions_tag: Option<ExecutionAssumptionsTag>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct ExperimentDefinitionSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) experiment_name: Option<String>,
    pub(super) replay_source: FrontendBacktestReplaySource,
    #[serde(default)]
    pub(super) base_execution_assumptions: FrontendExecutionAssumptionOverrides,
    pub(super) parameter_grid: FrontendExecutionAssumptionSweepGrid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct ExperimentRecord {
    pub(super) experiment_id: String,
    pub(super) graph_id: String,
    pub(super) compile_id: String,
    pub(super) created_at_ms: u64,
    #[serde(default)]
    pub(super) saved: bool,
    pub(super) definition: ExperimentDefinitionSummary,
    #[serde(default)]
    pub(super) variants: Vec<ExperimentVariantSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) actor: Option<ActorIdentity>,
}

#[derive(Debug, Serialize)]
pub(super) struct ExperimentListItem {
    pub(super) experiment_id: String,
    pub(super) graph_id: String,
    pub(super) compile_id: String,
    pub(super) created_at_ms: u64,
    pub(super) saved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) experiment_name: Option<String>,
    pub(super) replay_source: FrontendBacktestReplaySource,
    pub(super) variant_count: usize,
    #[serde(default)]
    pub(super) sweep_axes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) best_backtest_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) best_total_return_ratio: Option<f64>,
}

#[derive(Debug, Serialize)]
pub(super) struct ExperimentDetailResponse {
    pub(super) experiment_id: String,
    pub(super) graph_id: String,
    pub(super) compile_id: String,
    pub(super) created_at_ms: u64,
    pub(super) saved: bool,
    pub(super) definition: ExperimentDefinitionSummary,
    #[serde(default)]
    pub(super) variants: Vec<ExperimentVariantSummary>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum RuntimeReplayRecordKind {
    Run,
    Backtest,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct RuntimeReplayEventItem {
    pub(super) sequence_no: usize,
    pub(super) event: FrontendRuntimeEvent,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub(super) struct RuntimeReplayFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) stage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) severity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) retention_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) module_key: Option<String>,
    #[serde(default)]
    pub(super) key_only: bool,
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeReplayOptions {
    pub(super) cursor: usize,
    pub(super) limit: usize,
    pub(super) sequence_cursor: Option<u64>,
    pub(super) filters: RuntimeReplayFilters,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub(super) struct RuntimeEvidenceMetricsSnapshot {
    pub(super) report_generation_count: u64,
    pub(super) report_generation_failure_count: u64,
    pub(super) report_source_changed_count: u64,
    pub(super) replay_page_count: u64,
    pub(super) replay_page_latency_total_ms: u64,
    pub(super) replay_page_latency_avg_ms: f64,
    pub(super) compact_projection_source_event_count_total: u64,
    pub(super) compact_projection_retained_event_count_total: u64,
    pub(super) compact_detail_window_required_count: u64,
    pub(super) mutation_proposal_created_count: u64,
    pub(super) mutation_proposal_rejected_count: u64,
    pub(super) mutation_activation_scheduled_count: u64,
    pub(super) mutation_activation_applied_count: u64,
    pub(super) mutation_activation_failed_count: u64,
    pub(super) mutation_activation_latency_total_ms: u64,
    pub(super) mutation_activation_latency_avg_ms: f64,
    pub(super) mutation_safe_window_denied_count: u64,
    pub(super) mutation_rollback_attempt_count: u64,
    pub(super) mutation_rollback_scheduled_count: u64,
    pub(super) mutation_rollback_applied_count: u64,
    pub(super) mutation_rollback_failed_count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct RuntimeEvidenceReportStatusCounts {
    pub(super) requested: usize,
    pub(super) generating: usize,
    pub(super) ready: usize,
    pub(super) failed: usize,
    pub(super) expired: usize,
    pub(super) source_changed: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct RuntimeEvidenceCleanupPolicy {
    pub(super) policy_version: String,
    pub(super) transient_generation_ttl_ms: u64,
    pub(super) protects_persisted_report_records: bool,
    pub(super) transient_output_prefixes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct RuntimeEvidenceHealthResponse {
    pub(super) status: String,
    pub(super) metrics: RuntimeEvidenceMetricsSnapshot,
    pub(super) persisted_report_count: usize,
    pub(super) report_status_counts: RuntimeEvidenceReportStatusCounts,
    pub(super) cleanup_policy: RuntimeEvidenceCleanupPolicy,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct RuntimeEvidenceCleanupRequest {
    pub(super) max_age_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct RuntimeEvidenceCleanupResponse {
    pub(super) policy: RuntimeEvidenceCleanupPolicy,
    pub(super) removed_transient_generation_outputs: usize,
    pub(super) retained_report_records: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct RuntimeReplayCheckpoint {
    pub(super) cursor: usize,
    pub(super) sequence_cursor: u64,
    pub(super) label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) event_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) event_time_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum RuntimeEvidenceSourceKind {
    Run,
    Backtest,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum RuntimeParameterMutationStatus {
    Proposed,
    Rejected,
    ActivationScheduled,
    Activated,
    ActivationFailed,
    SafeWindowDenied,
    RollbackScheduled,
    RolledBack,
    RollbackFailed,
}

impl RuntimeParameterMutationStatus {
    #[allow(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            Self::Proposed => "已提案，等待激活",
            Self::Rejected => "已拒绝（参数无变化）",
            Self::ActivationScheduled => "激活已排期，等待安全窗口",
            Self::Activated => "已激活，参数已生效",
            Self::ActivationFailed => "激活失败",
            Self::SafeWindowDenied => "安全窗口拒绝（运行时活跃中）",
            Self::RollbackScheduled => "回滚已排期",
            Self::RolledBack => "已回滚至先前版本",
            Self::RollbackFailed => "回滚失败",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeParameterMutationTarget {
    pub(super) node_id: String,
    pub(super) module_key: String,
    pub(super) parameter_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeParameterMutationBoundary {
    pub(super) requested: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) resolved_sequence_no: Option<u64>,
}

impl Default for RuntimeParameterMutationBoundary {
    fn default() -> Self {
        Self {
            requested: "next_cycle_start".to_string(),
            resolved_sequence_no: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeParameterMutationGovernance {
    pub(super) capability_hash: String,
    pub(super) deployment_revision: String,
    pub(super) strategy_version: String,
    pub(super) previous_parameter_version: String,
    pub(super) proposed_parameter_version: String,
    pub(super) permission_boundary_model_version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeParameterMutationActivationState {
    pub(super) requested_boundary: RuntimeParameterMutationBoundary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) resolved_sequence_no: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) scheduled_at_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) activated_at_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) active_parameter_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) failure_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) observation_deadline_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeParameterMutationSafeWindowSnapshot {
    pub(super) policy_version: String,
    pub(super) runtime_status: String,
    pub(super) open_order_count: u64,
    pub(super) outstanding_risk_violation: bool,
    pub(super) data_freshness_ms: u64,
    pub(super) portfolio_exposure_bps: i64,
    pub(super) cooldown_remaining_ms: u64,
}

impl Default for RuntimeParameterMutationSafeWindowSnapshot {
    fn default() -> Self {
        Self {
            policy_version: "quantpilot/mutation-safe-window/v1".to_string(),
            runtime_status: "paused".to_string(),
            open_order_count: 0,
            outstanding_risk_violation: false,
            data_freshness_ms: 0,
            portfolio_exposure_bps: 0,
            cooldown_remaining_ms: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeParameterMutationSafeWindowState {
    pub(super) status: String,
    pub(super) policy_version: String,
    pub(super) allowed: bool,
    pub(super) reason_code: String,
    pub(super) message: String,
    pub(super) retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) retry_after_ms: Option<u64>,
    pub(super) snapshot: RuntimeParameterMutationSafeWindowSnapshot,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeParameterMutationLifecycleEntry {
    pub(super) status: RuntimeParameterMutationStatus,
    pub(super) event_id: String,
    pub(super) sequence_no: u64,
    pub(super) occurred_at_ms: u64,
    pub(super) reason_code: String,
    pub(super) message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct RuntimeParameterMutationRecord {
    pub(super) proposal_id: String,
    pub(super) source_kind: RuntimeEvidenceSourceKind,
    pub(super) source_id: String,
    pub(super) graph_id: String,
    pub(super) target: RuntimeParameterMutationTarget,
    pub(super) old_value: Value,
    pub(super) new_value: Value,
    pub(super) old_parameter_version: String,
    pub(super) proposed_parameter_version: String,
    pub(super) status: RuntimeParameterMutationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) rejection_reason: Option<String>,
    pub(super) activation_boundary: RuntimeParameterMutationBoundary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) activation_state: Option<RuntimeParameterMutationActivationState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) safe_window_state: Option<RuntimeParameterMutationSafeWindowState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) rollback_of: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) rollback_target_parameter_version: Option<String>,
    pub(super) actor: ActorIdentity,
    pub(super) reason: String,
    pub(super) governance: RuntimeParameterMutationGovernance,
    #[serde(default)]
    pub(super) lifecycle: Vec<RuntimeParameterMutationLifecycleEntry>,
    pub(super) created_at_ms: u64,
    pub(super) updated_at_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct CreateRuntimeParameterMutationRequest {
    pub(super) source_kind: RuntimeEvidenceSourceKind,
    pub(super) source_id: String,
    pub(super) target: RuntimeParameterMutationTarget,
    pub(super) old_value: Value,
    pub(super) new_value: Value,
    #[serde(default)]
    pub(super) activation_boundary: RuntimeParameterMutationBoundary,
    #[serde(default)]
    pub(super) actor: Option<ActorIdentity>,
    pub(super) reason: String,
    #[serde(default)]
    pub(super) capability_context: Option<FrontendCapabilityContext>,
    #[serde(default)]
    pub(super) safe_window_context: Option<RuntimeParameterMutationSafeWindowSnapshot>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum RuntimeAiProposalStatus {
    Draft,
    Submitted,
    StaticCheckFailed,
    StaticCheckPassed,
    Approved, // v1.2.1: 审批通过，与StaticCheckPassed区分
    Denied,
    Expired,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeAiModelIdentity {
    pub(super) provider: String,
    pub(super) model: String,
    pub(super) model_version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeAiProposalSourceEvidence {
    pub(super) source_kind: RuntimeEvidenceSourceKind,
    pub(super) source_id: String,
    pub(super) graph_id: String,
    pub(super) event_count: usize,
    pub(super) evidence_hash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeAiProposalGovernance {
    pub(super) capability_hash: String,
    pub(super) deployment_revision: String,
    pub(super) strategy_version: String,
    pub(super) previous_parameter_version: String,
    pub(super) proposed_parameter_version: String,
    pub(super) permission_boundary_model_version: String,
    pub(super) ai_write_policy: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum StrategyConfigProposalDomain {
    Market,
    Observation,
    StateMachine,
    Risk,
    Execution,
    AiGovernance,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeAiProposalConfigDomainBinding {
    pub(super) target_domain: StrategyConfigProposalDomain,
    pub(super) before_digest: String,
    pub(super) after_digest: String,
    #[serde(default)]
    pub(super) evidence_anchor_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeAiProposalStaticCheckDetail {
    pub(super) code: String,
    pub(super) target: String,
    pub(super) message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeAiProposalStaticCheckResult {
    pub(super) status: RuntimeAiProposalStatus,
    pub(super) reason_code: String,
    pub(super) message: String,
    pub(super) checked_at_ms: u64,
    #[serde(default)]
    pub(super) details: Vec<RuntimeAiProposalStaticCheckDetail>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeAiProposalLifecycleEntry {
    pub(super) status: RuntimeAiProposalStatus,
    pub(super) event_id: String,
    pub(super) sequence_no: u64,
    pub(super) occurred_at_ms: u64,
    pub(super) reason_code: String,
    pub(super) message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct RuntimeAiProposalRecord {
    pub(super) ai_proposal_id: String,
    pub(super) source_kind: RuntimeEvidenceSourceKind,
    pub(super) source_id: String,
    pub(super) graph_id: String,
    pub(super) source_evidence: RuntimeAiProposalSourceEvidence,
    pub(super) target: RuntimeParameterMutationTarget,
    pub(super) old_value: Value,
    pub(super) new_value: Value,
    pub(super) old_parameter_version: String,
    pub(super) proposed_parameter_version: String,
    pub(super) status: RuntimeAiProposalStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) denial_reason: Option<String>,
    pub(super) static_check: RuntimeAiProposalStaticCheckResult,
    pub(super) model: RuntimeAiModelIdentity,
    pub(super) prompt_hash: String,
    pub(super) evidence_hash: String,
    pub(super) actor: ActorIdentity,
    pub(super) reason: String,
    pub(super) governance: RuntimeAiProposalGovernance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) config_domain_binding: Option<RuntimeAiProposalConfigDomainBinding>,
    #[serde(default)]
    pub(super) lifecycle: Vec<RuntimeAiProposalLifecycleEntry>,
    pub(super) created_at_ms: u64,
    pub(super) updated_at_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct CreateRuntimeAiProposalRequest {
    pub(super) source_kind: RuntimeEvidenceSourceKind,
    pub(super) source_id: String,
    pub(super) target: RuntimeParameterMutationTarget,
    pub(super) old_value: Value,
    pub(super) new_value: Value,
    pub(super) model: RuntimeAiModelIdentity,
    pub(super) prompt_hash: String,
    pub(super) evidence_hash: String,
    #[serde(default)]
    pub(super) actor: Option<ActorIdentity>,
    pub(super) reason: String,
    #[serde(default)]
    pub(super) capability_context: Option<FrontendCapabilityContext>,
    #[serde(default)]
    pub(super) config_domain_binding: Option<RuntimeAiProposalConfigDomainBinding>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct CreateContractRepairApprovalRequest {
    pub(super) status: String,
    pub(super) payload_kind: String,
    pub(super) request_id: String,
    pub(super) target_path: String,
    pub(super) target_kind: String,
    #[serde(default)]
    pub(super) changed_fields: Vec<String>,
    #[serde(default)]
    pub(super) patch_payload: std::collections::BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub(super) contract_source_ref: ContractRepairApprovalContractSourceRef,
    pub(super) mutation_enabled: bool,
    pub(super) review_required: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
pub(super) struct ContractRepairApprovalContractSourceRef {
    #[serde(default)]
    pub(super) source_kind: String,
    #[serde(default)]
    pub(super) source_id: String,
    #[serde(default)]
    pub(super) version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) artifact_digest: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct ContractRepairApprovalReviewIntentRequest {
    pub(super) action: String,
    pub(super) reviewer_id: String,
    pub(super) reason: String,
    pub(super) review_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalBlockedResponse {
    pub(super) status: String,
    pub(super) request_id: String,
    pub(super) route_status: String,
    pub(super) payload_kind: String,
    pub(super) target_path: String,
    pub(super) target_kind: String,
    #[serde(default)]
    pub(super) changed_fields: Vec<String>,
    #[serde(default)]
    pub(super) patch_payload: std::collections::BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub(super) contract_source_ref: ContractRepairApprovalContractSourceRef,
    pub(super) mutation_enabled: bool,
    pub(super) review_required: bool,
    #[serde(default)]
    pub(super) blocked_reasons: Vec<String>,
    pub(super) approval_record_preview: ContractRepairApprovalRecordPreview,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalReviewBlockedResponse {
    pub(super) status: String,
    pub(super) approval_id: String,
    pub(super) route_status: String,
    pub(super) action: String,
    pub(super) reviewer_id: String,
    pub(super) review_enabled: bool,
    pub(super) persistence_enabled: bool,
    pub(super) mutation_enabled: bool,
    pub(super) execution_gate: ContractRepairApprovalReviewExecutionGate,
    pub(super) execution_plan_preview: ContractRepairApprovalReviewExecutionPlanPreview,
    pub(super) approve_execution_gate: ContractRepairApprovalApproveExecutionGate,
    pub(super) approve_execution_transaction_dry_run:
        ContractRepairApprovalApproveExecutionTransactionDryRun,
    pub(super) approve_execution_admission_gate:
        ContractRepairApprovalApproveExecutionAdmissionGate,
    pub(super) approve_execution_transaction_runner_enablement_dry_run:
        ContractRepairApprovalApproveExecutionTransactionRunnerEnablementDryRun,
    pub(super) approve_execution_transaction_runner_dry_run:
        ContractRepairApprovalApproveExecutionTransactionRunnerDryRun,
    pub(super) approve_execution_recovery_marker_write_dry_run:
        ContractRepairApprovalApproveExecutionRecoveryMarkerWriteDryRun,
    pub(super) approve_execution_recovery_marker_idempotency_precheck:
        ContractRepairApprovalApproveExecutionRecoveryMarkerIdempotencyPrecheck,
    pub(super) approve_execution_recovery_marker_persistence_gate:
        ContractRepairApprovalApproveExecutionRecoveryMarkerPersistenceGate,
    pub(super) approve_execution_recovery_marker_persistence_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRecoveryMarkerPersistenceReadinessDryRun,
    pub(super) approve_execution_transaction_commit_gate:
        ContractRepairApprovalApproveExecutionTransactionCommitGate,
    pub(super) approve_execution_transaction_commit_readiness_dry_run:
        ContractRepairApprovalApproveExecutionTransactionCommitReadinessDryRun,
    pub(super) approve_execution_atomic_side_effects_gate:
        ContractRepairApprovalApproveExecutionAtomicSideEffectsGate,
    pub(super) approve_execution_atomic_side_effects_enablement_dry_run:
        ContractRepairApprovalApproveExecutionAtomicSideEffectsEnablementDryRun,
    pub(super) approve_execution_atomic_side_effects_readiness_dry_run:
        ContractRepairApprovalApproveExecutionAtomicSideEffectsReadinessDryRun,
    pub(super) approve_execution_runner_attempt_enablement_dry_run:
        ContractRepairApprovalApproveExecutionRunnerAttemptEnablementDryRun,
    pub(super) approve_execution_runner_attempt:
        ContractRepairApprovalApproveExecutionRunnerAttempt,
    pub(super) approve_execution_runner_attempt_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerAttemptReadinessDryRun,
    pub(super) approve_execution_runner_execution_enablement_dry_run:
        ContractRepairApprovalApproveExecutionRunnerExecutionEnablementDryRun,
    pub(super) approve_execution_runner_execution_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerExecutionReadinessDryRun,
    pub(super) approve_execution_runner_outcome:
        ContractRepairApprovalApproveExecutionRunnerOutcome,
    pub(super) approve_execution_runner_route_dispatch_enablement_dry_run:
        ContractRepairApprovalApproveExecutionRunnerRouteDispatchEnablementDryRun,
    pub(super) approve_execution_runner_dispatch_gate:
        ContractRepairApprovalApproveExecutionRunnerDispatchGate,
    pub(super) approve_execution_runner_dispatch_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerDispatchReadinessDryRun,
    pub(super) approve_execution_runner_handoff:
        ContractRepairApprovalApproveExecutionRunnerHandoff,
    pub(super) approve_execution_runner_handoff_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerHandoffReadinessDryRun,
    pub(super) approve_execution_runner_call_enablement_dry_run:
        ContractRepairApprovalApproveExecutionRunnerCallEnablementDryRun,
    pub(super) approve_execution_runner_call_dry_run:
        ContractRepairApprovalApproveExecutionRunnerCallDryRun,
    pub(super) approve_execution_runner_call_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerCallReadinessDryRun,
    pub(super) approve_execution_runner_body_enablement_dry_run:
        ContractRepairApprovalApproveExecutionRunnerBodyEnablementDryRun,
    pub(super) approve_execution_runner_call_body_dry_run:
        ContractRepairApprovalApproveExecutionRunnerCallBodyDryRun,
    pub(super) approve_execution_runner_body_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerBodyReadinessDryRun,
    pub(super) approve_execution_runner_phase_execution_enablement_dry_run:
        ContractRepairApprovalApproveExecutionRunnerPhaseExecutionEnablementDryRun,
    pub(super) approve_execution_runner_body_phase_sequence_dry_run:
        ContractRepairApprovalApproveExecutionRunnerBodyPhaseSequenceDryRun,
    pub(super) approve_execution_runner_phases_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerPhasesReadinessDryRun,
    pub(super) approve_execution_runner_lifecycle_phase_enablement_dry_run:
        ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseEnablementDryRun,
    pub(super) approve_execution_runner_lifecycle_phase_dry_run:
        ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseDryRun,
    pub(super) approve_execution_runner_lifecycle_phase_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseReadinessDryRun,
    pub(super) approve_execution_runner_source_mutation_phase_enablement_dry_run:
        ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseEnablementDryRun,
    pub(super) approve_execution_runner_source_mutation_phase_dry_run:
        ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseDryRun,
    pub(super) approve_execution_runner_source_mutation_phase_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseReadinessDryRun,
    pub(super) approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run:
        ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseEnablementDryRun,
    pub(super) approve_execution_runner_recovery_marker_cleanup_phase_dry_run:
        ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseDryRun,
    pub(super) approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseReadinessDryRun,
    pub(super) approve_execution_runner_transaction_commit_phase_enablement_dry_run:
        ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseEnablementDryRun,
    pub(super) approve_execution_runner_transaction_commit_phase_dry_run:
        ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseDryRun,
    pub(super) approve_execution_runner_transaction_commit_phase_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseReadinessDryRun,
    pub(super) approve_execution_runner_rollback_execution_phase_enablement_dry_run:
        ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseEnablementDryRun,
    pub(super) approve_execution_runner_rollback_execution_phase_dry_run:
        ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseDryRun,
    pub(super) approve_execution_runner_rollback_execution_phase_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseReadinessDryRun,
    pub(super) approve_execution_runner_activation_enablement_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationEnablementDryRun,
    pub(super) approve_execution_runner_activation_enablement_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationEnablementReadinessDryRun,
    pub(super) approve_execution_runner_activation_path_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationPathDryRun,
    pub(super) approve_execution_runner_activation_path_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationPathReadinessDryRun,
    pub(super) approve_execution_runner_activation_execution_plan_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationExecutionPlanDryRun,
    pub(super) approve_execution_runner_activation_execution_plan_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationExecutionPlanReadinessDryRun,
    pub(super) approve_execution_runner_activation_switch_transaction_proof_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationSwitchTransactionProofDryRun,
    pub(super) approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationSwitchTransactionProofReadinessDryRun,
    pub(super) approve_execution_runner_activation_switch_write_transaction_enablement_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionEnablementDryRun,
    pub(super) approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionEnablementReadinessDryRun,
    pub(super) approve_execution_runner_activation_switch_write_transaction_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionDryRun,
    pub(super) approve_execution_runner_activation_switch_write_transaction_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionReadinessDryRun,
    pub(super) approve_execution_runner_activation_transaction_admission_gate_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationTransactionAdmissionGateDryRun,
    pub(super) approve_execution_runner_activation_transaction_admission_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationTransactionAdmissionReadinessDryRun,
    pub(super) approve_execution_runner_activation_admission_handoff_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationAdmissionHandoffDryRun,
    pub(super) approve_execution_runner_activation_admission_handoff_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationAdmissionHandoffReadinessDryRun,
    pub(super) approve_execution_runner_activation_handoff_enablement_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationHandoffEnablementDryRun,
    pub(super) approve_execution_runner_activation_handoff_enablement_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationHandoffEnablementReadinessDryRun,
    pub(super) approve_execution_runner_activation_handoff_attempt_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationHandoffAttemptDryRun,
    pub(super) approve_execution_runner_activation_handoff_attempt_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationHandoffAttemptReadinessDryRun,
    pub(super) approve_execution_runner_activation_post_handoff_attempt_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationPostHandoffAttemptDryRun,
    pub(super) approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationPostHandoffAttemptReadinessDryRun,
    pub(super) approve_execution_runner_activation_success_admission_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationSuccessAdmissionDryRun,
    pub(super) approve_execution_runner_activation_success_admission_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationSuccessAdmissionReadinessDryRun,
    pub(super) approve_execution_runner_activation_success_return_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationSuccessReturnDryRun,
    pub(super) approve_execution_runner_activation_success_return_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerActivationSuccessReturnReadinessDryRun,
    pub(super) approve_execution_runner_route_success_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerRouteSuccessReadinessDryRun,
    pub(super) approve_execution_runner_route_status_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerRouteStatusReadinessDryRun,
    pub(super) approve_execution_formal_review_execution_readiness_dry_run:
        ContractRepairApprovalApproveExecutionFormalReviewExecutionReadinessDryRun,
    pub(super) approve_execution_final_atomic_readiness_dry_run:
        ContractRepairApprovalApproveExecutionFinalAtomicReadinessDryRun,
    pub(super) approve_execution_final_atomic_execution_plan_dry_run:
        ContractRepairApprovalApproveExecutionFinalAtomicExecutionPlanDryRun,
    pub(super) approve_execution_final_atomic_admission_gate_dry_run:
        ContractRepairApprovalApproveExecutionFinalAtomicAdmissionGateDryRun,
    pub(super) approve_execution_final_execution_entry_dry_run:
        ContractRepairApprovalApproveExecutionFinalExecutionEntryDryRun,
    pub(super) approve_execution_final_execution_switch_readiness_dry_run:
        ContractRepairApprovalApproveExecutionFinalExecutionSwitchReadinessDryRun,
    pub(super) approve_execution_final_execution_rollback_readiness_dry_run:
        ContractRepairApprovalApproveExecutionFinalExecutionRollbackReadinessDryRun,
    pub(super) approve_execution_final_execution_replay_plan_dry_run:
        ContractRepairApprovalApproveExecutionFinalExecutionReplayPlanDryRun,
    pub(super) approve_execution_final_execution_replay_executor_dry_run:
        ContractRepairApprovalApproveExecutionFinalExecutionReplayExecutorDryRun,
    pub(super) approve_execution_final_execution_replay_executor_routing_dry_run:
        ContractRepairApprovalApproveExecutionFinalExecutionReplayExecutorRoutingDryRun,
    pub(super) approve_execution_final_execution_routed_write_handoff_dry_run:
        ContractRepairApprovalApproveExecutionFinalExecutionRoutedWriteHandoffDryRun,
    pub(super) approve_execution_final_execution_routed_handler_plan_dry_run:
        ContractRepairApprovalApproveExecutionFinalExecutionRoutedHandlerPlanDryRun,
    pub(super) approve_execution_final_execution_routed_execution_attempt_dry_run:
        ContractRepairApprovalApproveExecutionFinalExecutionRoutedExecutionAttemptDryRun,
    pub(super) approve_execution_final_execution_ordered_handler_execution_confirmation_dry_run:
        ContractRepairApprovalApproveExecutionFinalExecutionOrderedHandlerExecutionConfirmationDryRun,
    pub(super) approve_execution_final_execution_routed_route_success_release_dry_run:
        ContractRepairApprovalApproveExecutionFinalExecutionRoutedRouteSuccessReleaseDryRun,
    pub(super) approve_execution_final_execution_durable_writeback_bundle_dry_run:
        ContractRepairApprovalApproveExecutionFinalExecutionDurableWritebackBundleDryRun,
    pub(super) approve_execution_durable_disk_application_execution:
        ContractRepairApprovalApproveExecutionDurableDiskApplicationExecution,
    pub(super) approve_execution_decision_lock_summary_dry_run:
        ContractRepairApprovalApproveExecutionDecisionLockSummaryDryRun,
    pub(super) approve_execution_runner_control_readiness_dry_run:
        ContractRepairApprovalApproveExecutionRunnerControlReadinessDryRun,
    pub(super) approve_execution_runner_enablement_plan_dry_run:
        ContractRepairApprovalApproveExecutionRunnerEnablementPlanDryRun,
    pub(super) persistence_plan_preview: ContractRepairApprovalPersistencePlanPreview,
    pub(super) persistence_path_preview: ContractRepairApprovalPersistencePathPreview,
    pub(super) record_snapshot_preview: ContractRepairApprovalRecordSnapshotPreview,
    pub(super) storage_readiness_gate: ContractRepairApprovalStorageReadinessGate,
    pub(super) storage_dry_run_preview: ContractRepairApprovalStorageDryRunPreview,
    pub(super) idempotency_precheck: ContractRepairApprovalIdempotencyPrecheck,
    pub(super) reviewer_authorization_precheck: ContractRepairApprovalReviewerAuthorizationPrecheck,
    pub(super) review_transition_dry_run: ContractRepairApprovalReviewTransitionDryRun,
    pub(super) record_write_dry_run: ContractRepairApprovalRecordWriteDryRun,
    pub(super) lifecycle_event_dry_run: ContractRepairApprovalLifecycleEventDryRun,
    pub(super) lifecycle_entry_append_dry_run: ContractRepairApprovalLifecycleEntryAppendDryRun,
    pub(super) lifecycle_emission_enablement_gate:
        ContractRepairApprovalLifecycleEmissionEnablementGate,
    pub(super) approve_execution_lifecycle_effects_readiness_dry_run:
        ContractRepairApprovalApproveExecutionLifecycleEffectsReadinessDryRun,
    pub(super) approve_execution_lifecycle_event_emission_enablement_dry_run:
        ContractRepairApprovalApproveExecutionLifecycleEventEmissionEnablementDryRun,
    pub(super) approve_execution_lifecycle_entry_append_enablement_dry_run:
        ContractRepairApprovalApproveExecutionLifecycleEntryAppendEnablementDryRun,
    pub(super) contract_writeback_dry_run: ContractRepairApprovalContractWritebackDryRun,
    pub(super) contract_mutation_enablement_gate:
        ContractRepairApprovalContractMutationEnablementGate,
    pub(super) approve_execution_contract_mutation_readiness_dry_run:
        ContractRepairApprovalApproveExecutionContractMutationReadinessDryRun,
    #[serde(default)]
    pub(super) blocked_reasons: Vec<String>,
    pub(super) approval_record_preview: ContractRepairApprovalRecordPreview,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalReviewExecutionGate {
    pub(super) status: String,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalReviewExecutionPlanPreview {
    pub(super) status: String,
    pub(super) execution_enabled: bool,
    pub(super) action: String,
    pub(super) target_review_state: RuntimeApprovalReviewState,
    pub(super) would_persist_approval_record: bool,
    pub(super) would_mutate_contract: bool,
    pub(super) would_emit_lifecycle_event: bool,
    #[serde(default)]
    pub(super) side_effects: Vec<String>,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionGate {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) target_review_state: RuntimeApprovalReviewState,
    pub(super) approval_persistence_ready: bool,
    pub(super) transition_ready: bool,
    pub(super) lifecycle_emission_ready: bool,
    pub(super) lifecycle_append_ready: bool,
    pub(super) contract_writeback_ready: bool,
    pub(super) mutation_api_enabled: bool,
    pub(super) approve_execution_ready: bool,
    pub(super) would_execute: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionTransactionDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) transaction_plan_ready: bool,
    pub(super) execution_ready: bool,
    pub(super) approved_transition_ready: bool,
    pub(super) record_write_ready: bool,
    pub(super) lifecycle_emission_ready: bool,
    pub(super) lifecycle_append_ready: bool,
    pub(super) contract_writeback_ready: bool,
    pub(super) lifecycle_emission_enabled: bool,
    pub(super) mutation_api_enabled: bool,
    pub(super) would_transition_review: bool,
    pub(super) would_write_approval_record: bool,
    pub(super) would_emit_lifecycle_event: bool,
    pub(super) would_append_lifecycle_entry: bool,
    pub(super) would_write_contract_source: bool,
    pub(super) would_execute_transaction: bool,
    #[serde(default)]
    pub(super) step_order: Vec<String>,
    #[serde(default)]
    pub(super) atomicity_scope: Vec<String>,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionAdmissionGate {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) transaction_plan_ready: bool,
    pub(super) transaction_execution_ready: bool,
    pub(super) atomicity_scope_ready: bool,
    pub(super) transaction_runner_enabled: bool,
    pub(super) admission_ready: bool,
    pub(super) partial_execution_allowed: bool,
    pub(super) would_start_transaction: bool,
    pub(super) would_persist_any_side_effect: bool,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionTransactionRunnerEnablementDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) switch_name: String,
    pub(super) approve_action: bool,
    pub(super) transaction_plan_ready: bool,
    pub(super) transaction_execution_ready: bool,
    pub(super) atomicity_scope_ready: bool,
    pub(super) lifecycle_emission_enabled: bool,
    pub(super) contract_mutation_api_enabled: bool,
    pub(super) transaction_runner_enabled: bool,
    pub(super) enablement_prerequisites_ready: bool,
    pub(super) runner_enablement_ready: bool,
    pub(super) would_enable_runner: bool,
    pub(super) would_start_runner: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionTransactionRunnerDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) runner_enabled: bool,
    pub(super) admission_ready: bool,
    pub(super) transaction_plan_ready: bool,
    pub(super) commit_barrier_ready: bool,
    pub(super) recovery_marker_ready: bool,
    pub(super) rollback_plan_ready: bool,
    pub(super) commit_ready: bool,
    pub(super) would_start_runner: bool,
    pub(super) would_write_recovery_marker: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_rollback_on_error: bool,
    #[serde(default)]
    pub(super) phase_order: Vec<String>,
    #[serde(default)]
    pub(super) rollback_order: Vec<String>,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRecoveryMarkerWriteDryRun {
    pub(super) status: String,
    pub(super) approval_id: String,
    pub(super) marker_kind: String,
    pub(super) store_kind: String,
    pub(super) marker_key: String,
    pub(super) path_segment: String,
    pub(super) file_name: String,
    pub(super) payload_ready: bool,
    pub(super) path_ready: bool,
    pub(super) storage_ready: bool,
    pub(super) runner_ready: bool,
    pub(super) write_ready: bool,
    pub(super) atomic_write_required: bool,
    pub(super) would_write_marker: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) payload_fields: Vec<String>,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRecoveryMarkerIdempotencyPrecheck {
    pub(super) status: String,
    pub(super) marker_key: String,
    pub(super) file_name: String,
    pub(super) store_lookup_enabled: bool,
    pub(super) existing_marker_found: bool,
    pub(super) conflict_detected: bool,
    pub(super) marker_write_ready: bool,
    pub(super) safe_to_write_marker: bool,
    pub(super) would_write_marker: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRecoveryMarkerPersistenceGate {
    pub(super) status: String,
    pub(super) marker_key: String,
    pub(super) file_name: String,
    pub(super) marker_persistence_plan_ready: bool,
    pub(super) marker_write_ready: bool,
    pub(super) idempotency_checked: bool,
    pub(super) no_existing_marker_conflict: bool,
    pub(super) runner_ready: bool,
    pub(super) marker_persistence_enabled: bool,
    pub(super) persistence_ready: bool,
    pub(super) would_persist_marker: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRecoveryMarkerPersistenceReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) marker_key: String,
    pub(super) file_name: String,
    pub(super) marker_persistence_plan_ready: bool,
    pub(super) marker_write_ready: bool,
    pub(super) idempotency_checked: bool,
    pub(super) no_existing_marker_conflict: bool,
    pub(super) runner_ready: bool,
    pub(super) marker_persistence_enabled: bool,
    pub(super) recovery_marker_persistence_ready: bool,
    pub(super) would_persist_marker: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_transaction_commit: bool,
    pub(super) would_unblock_atomic_side_effects: bool,
    #[serde(default)]
    pub(super) inherited_recovery_marker_persistence_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionTransactionCommitGate {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) runner_plan_ready: bool,
    pub(super) runner_enabled: bool,
    pub(super) admission_ready: bool,
    pub(super) commit_barrier_ready: bool,
    pub(super) rollback_plan_ready: bool,
    pub(super) recovery_marker_persistence_plan_ready: bool,
    pub(super) recovery_marker_persistence_ready: bool,
    pub(super) commit_gate_enabled: bool,
    pub(super) commit_ready: bool,
    pub(super) would_start_runner: bool,
    pub(super) would_persist_recovery_marker: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionTransactionCommitReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) runner_plan_ready: bool,
    pub(super) runner_enabled: bool,
    pub(super) admission_ready: bool,
    pub(super) commit_barrier_ready: bool,
    pub(super) rollback_plan_ready: bool,
    pub(super) recovery_marker_persistence_plan_ready: bool,
    pub(super) recovery_marker_persistence_ready: bool,
    pub(super) commit_gate_enabled: bool,
    pub(super) transaction_commit_ready: bool,
    pub(super) would_start_runner: bool,
    pub(super) would_persist_recovery_marker: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_atomic_side_effects: bool,
    #[serde(default)]
    pub(super) inherited_transaction_commit_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionAtomicSideEffectsGate {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) lifecycle_plan_ready: bool,
    pub(super) contract_mutation_plan_ready: bool,
    pub(super) recovery_marker_persistence_plan_ready: bool,
    pub(super) transaction_commit_plan_ready: bool,
    pub(super) atomic_side_effects_plan_ready: bool,
    pub(super) lifecycle_effects_ready: bool,
    pub(super) contract_mutation_ready: bool,
    pub(super) recovery_marker_persistence_ready: bool,
    pub(super) transaction_commit_ready: bool,
    pub(super) atomic_side_effects_enabled: bool,
    pub(super) atomic_side_effects_ready: bool,
    pub(super) would_emit_lifecycle: bool,
    pub(super) would_mutate_contract: bool,
    pub(super) would_persist_recovery_marker: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionAtomicSideEffectsEnablementDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) switch_name: String,
    pub(super) lifecycle_plan_ready: bool,
    pub(super) contract_mutation_plan_ready: bool,
    pub(super) recovery_marker_persistence_plan_ready: bool,
    pub(super) transaction_commit_plan_ready: bool,
    pub(super) atomic_side_effects_plan_ready: bool,
    pub(super) lifecycle_effects_ready: bool,
    pub(super) contract_mutation_ready: bool,
    pub(super) recovery_marker_persistence_ready: bool,
    pub(super) transaction_commit_ready: bool,
    pub(super) atomic_side_effects_enabled: bool,
    pub(super) enablement_prerequisites_ready: bool,
    pub(super) atomic_side_effects_enablement_ready: bool,
    pub(super) would_enable_atomic_side_effects: bool,
    pub(super) would_emit_lifecycle: bool,
    pub(super) would_mutate_contract: bool,
    pub(super) would_persist_recovery_marker: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_atomic_side_effects_readiness: bool,
    pub(super) would_unblock_runner_attempt_readiness: bool,
    #[serde(default)]
    pub(super) inherited_atomic_side_effects_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionAtomicSideEffectsReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) lifecycle_plan_ready: bool,
    pub(super) contract_mutation_plan_ready: bool,
    pub(super) recovery_marker_persistence_plan_ready: bool,
    pub(super) transaction_commit_plan_ready: bool,
    pub(super) atomic_side_effects_plan_ready: bool,
    pub(super) lifecycle_effects_ready: bool,
    pub(super) contract_mutation_ready: bool,
    pub(super) recovery_marker_persistence_ready: bool,
    pub(super) transaction_commit_ready: bool,
    pub(super) atomic_side_effects_enabled: bool,
    pub(super) atomic_side_effects_ready: bool,
    pub(super) would_emit_lifecycle: bool,
    pub(super) would_mutate_contract: bool,
    pub(super) would_persist_recovery_marker: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_runner_attempt_readiness: bool,
    #[serde(default)]
    pub(super) inherited_atomic_side_effects_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerAttemptEnablementDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) switch_name: String,
    pub(super) attempt_requested: bool,
    pub(super) atomic_side_effects_plan_ready: bool,
    pub(super) atomic_side_effects_ready: bool,
    pub(super) transaction_runner_enablement_ready: bool,
    pub(super) runner_attempt_enabled: bool,
    pub(super) enablement_prerequisites_ready: bool,
    pub(super) runner_attempt_enablement_ready: bool,
    pub(super) would_enable_runner_attempt: bool,
    pub(super) would_start_runner: bool,
    pub(super) would_persist_any_side_effect: bool,
    pub(super) would_rollback_on_error: bool,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerAttempt {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) attempt_requested: bool,
    pub(super) atomic_side_effects_plan_ready: bool,
    pub(super) atomic_side_effects_ready: bool,
    pub(super) runner_attempt_enabled: bool,
    pub(super) runner_attempt_ready: bool,
    pub(super) would_start_runner: bool,
    pub(super) would_persist_any_side_effect: bool,
    pub(super) would_rollback_on_error: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerAttemptReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) attempt_requested: bool,
    pub(super) atomic_side_effects_plan_ready: bool,
    pub(super) atomic_side_effects_ready: bool,
    pub(super) transaction_runner_enablement_ready: bool,
    pub(super) runner_attempt_enabled: bool,
    pub(super) runner_attempt_enablement_ready: bool,
    pub(super) runner_attempt_ready: bool,
    pub(super) would_enable_runner_attempt: bool,
    pub(super) would_start_runner: bool,
    pub(super) would_persist_any_side_effect: bool,
    pub(super) would_rollback_on_error: bool,
    pub(super) would_unblock_control_readiness: bool,
    #[serde(default)]
    pub(super) inherited_runner_attempt_blockers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_attempt_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerExecutionEnablementDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) switch_name: String,
    pub(super) branch_selected: bool,
    pub(super) runner_attempt_ready: bool,
    pub(super) atomic_side_effects_ready: bool,
    pub(super) runner_attempt_enablement_ready: bool,
    pub(super) runner_execution_enabled: bool,
    pub(super) enablement_prerequisites_ready: bool,
    pub(super) runner_execution_enablement_ready: bool,
    pub(super) would_enable_runner_execution: bool,
    pub(super) would_start_runner: bool,
    pub(super) would_persist_any_side_effect: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_rollback_on_error: bool,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerExecutionReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) branch_selected: bool,
    pub(super) runner_attempt_ready: bool,
    pub(super) atomic_side_effects_ready: bool,
    pub(super) runner_execution_enabled: bool,
    pub(super) runner_execution_enablement_ready: bool,
    pub(super) runner_execution_ready: bool,
    pub(super) would_enable_runner_execution: bool,
    pub(super) would_start_runner: bool,
    pub(super) would_persist_any_side_effect: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_rollback_on_error: bool,
    pub(super) would_unblock_route_dispatch: bool,
    pub(super) would_unblock_control_readiness: bool,
    #[serde(default)]
    pub(super) inherited_runner_outcome_blockers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_execution_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerOutcome {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) branch_selected: bool,
    pub(super) runner_attempt_ready: bool,
    pub(super) atomic_side_effects_ready: bool,
    pub(super) runner_execution_enabled: bool,
    pub(super) runner_execution_ready: bool,
    pub(super) would_start_runner: bool,
    pub(super) would_persist_any_side_effect: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_rollback_on_error: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerRouteDispatchEnablementDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) switch_name: String,
    pub(super) branch_selected: bool,
    pub(super) runner_execution_ready: bool,
    pub(super) runner_execution_enablement_ready: bool,
    pub(super) route_dispatch_enabled: bool,
    pub(super) enablement_prerequisites_ready: bool,
    pub(super) route_dispatch_enablement_ready: bool,
    pub(super) would_enable_route_dispatch: bool,
    pub(super) would_enter_runner_branch: bool,
    pub(super) would_return_success: bool,
    pub(super) would_persist_any_side_effect: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerDispatchGate {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) branch_selected: bool,
    pub(super) runner_execution_ready: bool,
    pub(super) route_dispatch_enabled: bool,
    pub(super) dispatch_ready: bool,
    pub(super) would_enter_runner_branch: bool,
    pub(super) would_return_success: bool,
    pub(super) would_persist_any_side_effect: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerDispatchReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) branch_selected: bool,
    pub(super) runner_execution_ready: bool,
    pub(super) route_dispatch_enabled: bool,
    pub(super) route_dispatch_enablement_ready: bool,
    pub(super) dispatch_ready: bool,
    pub(super) would_enter_runner_branch: bool,
    pub(super) would_return_success: bool,
    pub(super) would_persist_any_side_effect: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_handoff: bool,
    pub(super) would_unblock_control_readiness: bool,
    #[serde(default)]
    pub(super) inherited_dispatch_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_route_dispatch_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerHandoff {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) dispatch_ready: bool,
    pub(super) route_dispatch_enabled: bool,
    pub(super) handoff_ready: bool,
    pub(super) expected_http_status: u16,
    pub(super) expected_route_status: String,
    pub(super) would_call_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_persist_any_side_effect: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerHandoffReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) dispatch_ready: bool,
    pub(super) route_dispatch_enabled: bool,
    pub(super) dispatch_readiness_ready: bool,
    pub(super) handoff_ready: bool,
    pub(super) expected_http_status: u16,
    pub(super) expected_route_status: String,
    pub(super) would_call_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_persist_any_side_effect: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_call: bool,
    pub(super) would_unblock_control_readiness: bool,
    #[serde(default)]
    pub(super) inherited_handoff_blockers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_dispatch_readiness_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerCallEnablementDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) switch_name: String,
    pub(super) handoff_ready: bool,
    pub(super) route_dispatch_enablement_ready: bool,
    pub(super) runner_call_enabled: bool,
    pub(super) enablement_prerequisites_ready: bool,
    pub(super) runner_call_enablement_ready: bool,
    pub(super) would_enable_runner_call: bool,
    pub(super) would_call_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_persist_any_side_effect: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_rollback_on_error: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerCallDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) handoff_ready: bool,
    pub(super) runner_call_enabled: bool,
    pub(super) call_ready: bool,
    pub(super) expected_runner_result: String,
    pub(super) would_call_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_persist_any_side_effect: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_rollback_on_error: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerCallReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) handoff_ready: bool,
    pub(super) runner_call_enabled: bool,
    pub(super) runner_call_enablement_ready: bool,
    pub(super) call_ready: bool,
    pub(super) expected_runner_result: String,
    pub(super) would_call_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_persist_any_side_effect: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_rollback_on_error: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_body: bool,
    pub(super) would_unblock_control_readiness: bool,
    #[serde(default)]
    pub(super) inherited_call_blockers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_call_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerBodyEnablementDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) switch_name: String,
    pub(super) call_ready: bool,
    pub(super) side_effect_bundle_ready: bool,
    pub(super) atomic_side_effects_ready: bool,
    pub(super) runner_body_enabled: bool,
    pub(super) enablement_prerequisites_ready: bool,
    pub(super) runner_body_enablement_ready: bool,
    pub(super) would_enable_runner_body: bool,
    pub(super) would_enter_body: bool,
    pub(super) would_emit_lifecycle: bool,
    pub(super) would_mutate_contract: bool,
    pub(super) would_persist_recovery_marker: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_rollback_on_error: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerCallBodyDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) runner_entrypoint: String,
    pub(super) call_ready: bool,
    pub(super) side_effect_bundle_ready: bool,
    pub(super) atomic_side_effects_ready: bool,
    pub(super) runner_body_enabled: bool,
    pub(super) body_ready: bool,
    pub(super) would_enter_body: bool,
    pub(super) would_emit_lifecycle: bool,
    pub(super) would_mutate_contract: bool,
    pub(super) would_persist_recovery_marker: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_rollback_on_error: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerBodyReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) runner_entrypoint: String,
    pub(super) call_ready: bool,
    pub(super) side_effect_bundle_ready: bool,
    pub(super) atomic_side_effects_ready: bool,
    pub(super) runner_body_enabled: bool,
    pub(super) runner_body_enablement_ready: bool,
    pub(super) body_ready: bool,
    pub(super) would_enter_body: bool,
    pub(super) would_emit_lifecycle: bool,
    pub(super) would_mutate_contract: bool,
    pub(super) would_persist_recovery_marker: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_rollback_on_error: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_phase_sequence: bool,
    pub(super) would_unblock_control_readiness: bool,
    #[serde(default)]
    pub(super) inherited_body_blockers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_body_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerPhaseExecutionEnablementDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) switch_name: String,
    pub(super) body_ready: bool,
    pub(super) phase_sequence_ready: bool,
    pub(super) rollback_sequence_ready: bool,
    pub(super) phase_execution_enabled: bool,
    pub(super) enablement_prerequisites_ready: bool,
    pub(super) phase_execution_enablement_ready: bool,
    #[serde(default)]
    pub(super) phase_order: Vec<String>,
    #[serde(default)]
    pub(super) rollback_order: Vec<String>,
    pub(super) would_enable_phase_execution: bool,
    pub(super) would_execute_phase_sequence: bool,
    pub(super) would_execute_rollback_sequence: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerBodyPhaseSequenceDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) body_ready: bool,
    pub(super) phase_sequence_ready: bool,
    pub(super) rollback_sequence_ready: bool,
    pub(super) phase_execution_enabled: bool,
    pub(super) phases_ready: bool,
    #[serde(default)]
    pub(super) phase_order: Vec<String>,
    #[serde(default)]
    pub(super) rollback_order: Vec<String>,
    pub(super) would_execute_phase_sequence: bool,
    pub(super) would_execute_rollback_sequence: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerPhasesReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) body_ready: bool,
    pub(super) phase_sequence_ready: bool,
    pub(super) rollback_sequence_ready: bool,
    pub(super) phase_execution_enabled: bool,
    pub(super) phase_execution_enablement_ready: bool,
    pub(super) phases_ready: bool,
    #[serde(default)]
    pub(super) phase_order: Vec<String>,
    #[serde(default)]
    pub(super) rollback_order: Vec<String>,
    pub(super) would_execute_phase_sequence: bool,
    pub(super) would_execute_rollback_sequence: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_lifecycle_phase: bool,
    pub(super) would_unblock_control_readiness: bool,
    #[serde(default)]
    pub(super) inherited_phase_sequence_blockers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_phase_execution_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseEnablementDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) event_id: String,
    pub(super) switch_name: String,
    pub(super) phase_sequence_ready: bool,
    pub(super) phase_execution_enablement_ready: bool,
    pub(super) lifecycle_phase_present: bool,
    pub(super) lifecycle_plan_ready: bool,
    pub(super) lifecycle_effects_ready: bool,
    pub(super) lifecycle_phase_enabled: bool,
    pub(super) enablement_prerequisites_ready: bool,
    pub(super) lifecycle_phase_enablement_ready: bool,
    pub(super) would_enable_lifecycle_phase: bool,
    pub(super) would_emit_lifecycle: bool,
    pub(super) would_append_lifecycle: bool,
    pub(super) would_touch_lifecycle_log: bool,
    pub(super) would_continue_to_next_phase: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) event_id: String,
    pub(super) phase_sequence_ready: bool,
    pub(super) lifecycle_phase_present: bool,
    pub(super) lifecycle_plan_ready: bool,
    pub(super) lifecycle_effects_ready: bool,
    pub(super) lifecycle_phase_enabled: bool,
    pub(super) lifecycle_phase_ready: bool,
    pub(super) would_emit_lifecycle: bool,
    pub(super) would_append_lifecycle: bool,
    pub(super) would_touch_lifecycle_log: bool,
    pub(super) would_continue_to_next_phase: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) event_id: String,
    pub(super) gate_name: String,
    pub(super) phase_sequence_ready: bool,
    pub(super) phase_execution_enablement_ready: bool,
    pub(super) lifecycle_phase_present: bool,
    pub(super) lifecycle_plan_ready: bool,
    pub(super) lifecycle_effects_ready: bool,
    pub(super) lifecycle_phase_enabled: bool,
    pub(super) lifecycle_phase_enablement_ready: bool,
    pub(super) lifecycle_phase_ready: bool,
    pub(super) would_emit_lifecycle: bool,
    pub(super) would_append_lifecycle: bool,
    pub(super) would_touch_lifecycle_log: bool,
    pub(super) would_continue_to_next_phase: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_source_mutation_phase: bool,
    pub(super) would_unblock_control_readiness: bool,
    #[serde(default)]
    pub(super) inherited_lifecycle_phase_blockers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_lifecycle_phase_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseEnablementDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) target_path: String,
    pub(super) target_kind: String,
    pub(super) source_path: String,
    pub(super) switch_name: String,
    pub(super) phase_sequence_ready: bool,
    pub(super) lifecycle_phase_enablement_ready: bool,
    pub(super) lifecycle_phase_ready: bool,
    pub(super) source_mutation_phase_present: bool,
    pub(super) writeback_plan_ready: bool,
    pub(super) source_write_ready: bool,
    pub(super) lifecycle_effects_ready: bool,
    pub(super) contract_mutation_api_enabled: bool,
    pub(super) contract_mutation_ready: bool,
    pub(super) source_mutation_phase_enabled: bool,
    pub(super) enablement_prerequisites_ready: bool,
    pub(super) source_mutation_phase_enablement_ready: bool,
    pub(super) would_enable_source_mutation_phase: bool,
    pub(super) would_mutate_contract: bool,
    pub(super) would_write_contract_source: bool,
    pub(super) would_continue_to_next_phase: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) target_path: String,
    pub(super) target_kind: String,
    pub(super) source_path: String,
    pub(super) phase_sequence_ready: bool,
    pub(super) lifecycle_phase_ready: bool,
    pub(super) source_mutation_phase_present: bool,
    pub(super) writeback_plan_ready: bool,
    pub(super) source_write_ready: bool,
    pub(super) lifecycle_effects_ready: bool,
    pub(super) contract_mutation_api_enabled: bool,
    pub(super) contract_mutation_ready: bool,
    pub(super) source_mutation_phase_enabled: bool,
    pub(super) source_mutation_phase_ready: bool,
    pub(super) would_mutate_contract: bool,
    pub(super) would_write_contract_source: bool,
    pub(super) would_continue_to_next_phase: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) target_path: String,
    pub(super) target_kind: String,
    pub(super) source_path: String,
    pub(super) gate_name: String,
    pub(super) phase_sequence_ready: bool,
    pub(super) lifecycle_phase_enablement_ready: bool,
    pub(super) lifecycle_phase_ready: bool,
    pub(super) source_mutation_phase_present: bool,
    pub(super) writeback_plan_ready: bool,
    pub(super) source_write_ready: bool,
    pub(super) lifecycle_effects_ready: bool,
    pub(super) contract_mutation_api_enabled: bool,
    pub(super) contract_mutation_ready: bool,
    pub(super) source_mutation_phase_enabled: bool,
    pub(super) source_mutation_phase_enablement_ready: bool,
    pub(super) source_mutation_phase_ready: bool,
    pub(super) would_mutate_contract: bool,
    pub(super) would_write_contract_source: bool,
    pub(super) would_continue_to_next_phase: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_recovery_marker_cleanup: bool,
    pub(super) would_unblock_control_readiness: bool,
    #[serde(default)]
    pub(super) inherited_source_mutation_phase_blockers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_source_mutation_phase_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseEnablementDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) marker_key: String,
    pub(super) file_name: String,
    pub(super) switch_name: String,
    pub(super) phase_sequence_ready: bool,
    pub(super) source_mutation_phase_enablement_ready: bool,
    pub(super) source_mutation_phase_ready: bool,
    pub(super) cleanup_phase_present: bool,
    pub(super) marker_persistence_plan_ready: bool,
    pub(super) marker_persistence_ready: bool,
    pub(super) cleanup_phase_enabled: bool,
    pub(super) enablement_prerequisites_ready: bool,
    pub(super) cleanup_phase_enablement_ready: bool,
    pub(super) would_enable_cleanup_phase: bool,
    pub(super) would_clear_recovery_marker: bool,
    pub(super) would_continue_to_commit: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) marker_key: String,
    pub(super) file_name: String,
    pub(super) phase_sequence_ready: bool,
    pub(super) source_mutation_phase_ready: bool,
    pub(super) cleanup_phase_present: bool,
    pub(super) marker_persistence_plan_ready: bool,
    pub(super) marker_persistence_ready: bool,
    pub(super) cleanup_phase_enabled: bool,
    pub(super) cleanup_phase_ready: bool,
    pub(super) would_clear_recovery_marker: bool,
    pub(super) would_continue_to_commit: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseReadinessDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) marker_key: String,
    pub(super) file_name: String,
    pub(super) gate_name: String,
    pub(super) phase_sequence_ready: bool,
    pub(super) source_mutation_phase_enablement_ready: bool,
    pub(super) source_mutation_phase_ready: bool,
    pub(super) cleanup_phase_present: bool,
    pub(super) marker_persistence_plan_ready: bool,
    pub(super) marker_persistence_ready: bool,
    pub(super) cleanup_phase_enabled: bool,
    pub(super) cleanup_phase_enablement_ready: bool,
    pub(super) cleanup_phase_ready: bool,
    pub(super) would_clear_recovery_marker: bool,
    pub(super) would_continue_to_commit: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_transaction_commit: bool,
    pub(super) would_unblock_control_readiness: bool,
    #[serde(default)]
    pub(super) inherited_cleanup_phase_blockers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_cleanup_phase_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseEnablementDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) switch_name: String,
    pub(super) phase_sequence_ready: bool,
    pub(super) cleanup_phase_enablement_ready: bool,
    pub(super) cleanup_phase_ready: bool,
    pub(super) runner_plan_ready: bool,
    pub(super) runner_enabled: bool,
    pub(super) admission_ready: bool,
    pub(super) commit_barrier_ready: bool,
    pub(super) rollback_plan_ready: bool,
    pub(super) recovery_marker_persistence_plan_ready: bool,
    pub(super) recovery_marker_persistence_ready: bool,
    pub(super) commit_gate_enabled: bool,
    pub(super) commit_ready: bool,
    pub(super) commit_phase_enabled: bool,
    pub(super) enablement_prerequisites_ready: bool,
    pub(super) commit_phase_enablement_ready: bool,
    pub(super) would_enable_commit_phase: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) phase_sequence_ready: bool,
    pub(super) cleanup_phase_ready: bool,
    pub(super) runner_plan_ready: bool,
    pub(super) runner_enabled: bool,
    pub(super) admission_ready: bool,
    pub(super) commit_barrier_ready: bool,
    pub(super) rollback_plan_ready: bool,
    pub(super) recovery_marker_persistence_plan_ready: bool,
    pub(super) recovery_marker_persistence_ready: bool,
    pub(super) commit_gate_enabled: bool,
    pub(super) commit_ready: bool,
    pub(super) commit_phase_enabled: bool,
    pub(super) commit_phase_ready: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseReadinessDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) phase_sequence_ready: bool,
    pub(super) cleanup_phase_enablement_ready: bool,
    pub(super) cleanup_phase_ready: bool,
    pub(super) runner_plan_ready: bool,
    pub(super) runner_enabled: bool,
    pub(super) admission_ready: bool,
    pub(super) commit_barrier_ready: bool,
    pub(super) rollback_plan_ready: bool,
    pub(super) recovery_marker_persistence_plan_ready: bool,
    pub(super) recovery_marker_persistence_ready: bool,
    pub(super) commit_gate_enabled: bool,
    pub(super) commit_ready: bool,
    pub(super) commit_phase_enabled: bool,
    pub(super) commit_phase_enablement_ready: bool,
    pub(super) commit_phase_ready: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_rollback_execution: bool,
    pub(super) would_unblock_control_readiness: bool,
    #[serde(default)]
    pub(super) inherited_commit_phase_blockers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_commit_phase_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseEnablementDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) switch_name: String,
    pub(super) phase_sequence_ready: bool,
    pub(super) rollback_sequence_ready: bool,
    pub(super) commit_phase_enablement_ready: bool,
    pub(super) commit_phase_ready: bool,
    pub(super) rollback_phase_present: bool,
    pub(super) rollback_plan_ready: bool,
    pub(super) rollback_execution_enabled: bool,
    pub(super) enablement_prerequisites_ready: bool,
    pub(super) rollback_execution_enablement_ready: bool,
    #[serde(default)]
    pub(super) rollback_order: Vec<String>,
    pub(super) would_enable_rollback_execution: bool,
    pub(super) would_restore_contract_source: bool,
    pub(super) would_restore_approval_record: bool,
    pub(super) would_mark_recovery_marker_rolled_back: bool,
    pub(super) would_rollback_on_error: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) phase_sequence_ready: bool,
    pub(super) rollback_sequence_ready: bool,
    pub(super) commit_phase_ready: bool,
    pub(super) rollback_phase_present: bool,
    pub(super) rollback_plan_ready: bool,
    pub(super) rollback_execution_enabled: bool,
    pub(super) rollback_execution_ready: bool,
    #[serde(default)]
    pub(super) rollback_order: Vec<String>,
    pub(super) would_restore_contract_source: bool,
    pub(super) would_restore_approval_record: bool,
    pub(super) would_mark_recovery_marker_rolled_back: bool,
    pub(super) would_rollback_on_error: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseReadinessDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) phase_sequence_ready: bool,
    pub(super) rollback_sequence_ready: bool,
    pub(super) commit_phase_enablement_ready: bool,
    pub(super) commit_phase_ready: bool,
    pub(super) rollback_phase_present: bool,
    pub(super) rollback_plan_ready: bool,
    pub(super) rollback_execution_enabled: bool,
    pub(super) rollback_execution_enablement_ready: bool,
    pub(super) rollback_execution_ready: bool,
    #[serde(default)]
    pub(super) rollback_order: Vec<String>,
    pub(super) would_restore_contract_source: bool,
    pub(super) would_restore_approval_record: bool,
    pub(super) would_mark_recovery_marker_rolled_back: bool,
    pub(super) would_rollback_on_error: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_runner_activation: bool,
    pub(super) would_unblock_control_readiness: bool,
    #[serde(default)]
    pub(super) inherited_rollback_execution_blockers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_rollback_execution_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationEnablementDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) switch_name: String,
    pub(super) structural_plan_ready: bool,
    pub(super) runner_control_ready: bool,
    pub(super) phase_chain_ready: bool,
    pub(super) rollback_chain_ready: bool,
    pub(super) side_effect_enablement_ready: bool,
    pub(super) rollback_execution_enablement_ready: bool,
    pub(super) prior_enablements_ready: bool,
    pub(super) runner_activation_enabled: bool,
    pub(super) enablement_prerequisites_ready: bool,
    pub(super) runner_activation_enablement_ready: bool,
    pub(super) would_enable_runner_activation: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) required_enablements: Vec<String>,
    #[serde(default)]
    pub(super) passed_enablements: Vec<String>,
    #[serde(default)]
    pub(super) blocked_enablements: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationEnablementReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) switch_name: String,
    pub(super) structural_plan_ready: bool,
    pub(super) runner_control_ready: bool,
    pub(super) phase_chain_ready: bool,
    pub(super) rollback_chain_ready: bool,
    pub(super) side_effect_enablement_ready: bool,
    pub(super) rollback_execution_enablement_ready: bool,
    pub(super) prior_enablements_ready: bool,
    pub(super) runner_activation_enabled: bool,
    pub(super) enablement_prerequisites_ready: bool,
    pub(super) runner_activation_enablement_ready: bool,
    pub(super) would_enable_runner_activation: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_activation_path: bool,
    #[serde(default)]
    pub(super) required_enablements: Vec<String>,
    #[serde(default)]
    pub(super) passed_enablements: Vec<String>,
    #[serde(default)]
    pub(super) blocked_enablements: Vec<String>,
    #[serde(default)]
    pub(super) inherited_activation_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationPathDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) path_name: String,
    pub(super) structural_plan_ready: bool,
    pub(super) runner_control_ready: bool,
    pub(super) phase_chain_ready: bool,
    pub(super) rollback_chain_ready: bool,
    pub(super) side_effect_enablement_ready: bool,
    pub(super) activation_enablement_ready: bool,
    pub(super) prior_enablements_ready: bool,
    pub(super) atomic_activation_required: bool,
    pub(super) activation_path_ready: bool,
    #[serde(default)]
    pub(super) activation_steps: Vec<String>,
    pub(super) would_enable_any_switch: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) required_enablements: Vec<String>,
    #[serde(default)]
    pub(super) passed_enablements: Vec<String>,
    #[serde(default)]
    pub(super) blocked_enablements: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationPathReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) path_name: String,
    pub(super) structural_plan_ready: bool,
    pub(super) runner_control_ready: bool,
    pub(super) phase_chain_ready: bool,
    pub(super) rollback_chain_ready: bool,
    pub(super) side_effect_enablement_ready: bool,
    pub(super) activation_enablement_ready: bool,
    pub(super) activation_enablement_readiness_ready: bool,
    pub(super) prior_enablements_ready: bool,
    pub(super) atomic_activation_required: bool,
    pub(super) activation_path_ready: bool,
    pub(super) activation_step_count: usize,
    #[serde(default)]
    pub(super) activation_steps: Vec<String>,
    pub(super) would_enable_any_switch: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_activation_execution_plan: bool,
    #[serde(default)]
    pub(super) required_enablements: Vec<String>,
    #[serde(default)]
    pub(super) passed_enablements: Vec<String>,
    #[serde(default)]
    pub(super) blocked_enablements: Vec<String>,
    #[serde(default)]
    pub(super) inherited_path_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_activation_enablement_readiness_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationExecutionPlanDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) plan_name: String,
    pub(super) path_name: String,
    pub(super) activation_path_ready: bool,
    pub(super) atomic_activation_required: bool,
    pub(super) write_set_ready: bool,
    pub(super) all_or_nothing_guard_ready: bool,
    pub(super) rollback_boundary_ready: bool,
    pub(super) no_partial_activation_allowed: bool,
    pub(super) activation_execution_plan_ready: bool,
    pub(super) activation_step_count: usize,
    #[serde(default)]
    pub(super) atomic_write_set: Vec<String>,
    pub(super) would_enable_any_switch: bool,
    pub(super) would_persist_activation_switches: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) required_enablements: Vec<String>,
    #[serde(default)]
    pub(super) passed_enablements: Vec<String>,
    #[serde(default)]
    pub(super) blocked_enablements: Vec<String>,
    #[serde(default)]
    pub(super) inherited_path_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationExecutionPlanReadinessDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) plan_name: String,
    pub(super) path_name: String,
    pub(super) activation_path_ready: bool,
    pub(super) activation_path_readiness_ready: bool,
    pub(super) atomic_activation_required: bool,
    pub(super) write_set_ready: bool,
    pub(super) all_or_nothing_guard_ready: bool,
    pub(super) rollback_boundary_ready: bool,
    pub(super) no_partial_activation_allowed: bool,
    pub(super) activation_execution_plan_ready: bool,
    pub(super) activation_step_count: usize,
    #[serde(default)]
    pub(super) atomic_write_set: Vec<String>,
    pub(super) would_enable_any_switch: bool,
    pub(super) would_persist_activation_switches: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_switch_transaction_proof: bool,
    #[serde(default)]
    pub(super) required_enablements: Vec<String>,
    #[serde(default)]
    pub(super) passed_enablements: Vec<String>,
    #[serde(default)]
    pub(super) blocked_enablements: Vec<String>,
    #[serde(default)]
    pub(super) inherited_execution_plan_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_path_readiness_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationSwitchTransactionProofDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) transaction_name: String,
    pub(super) plan_name: String,
    pub(super) activation_execution_plan_ready: bool,
    pub(super) write_set_ready: bool,
    pub(super) all_or_nothing_guard_ready: bool,
    pub(super) failure_probe_coverage_ready: bool,
    pub(super) rollback_action_coverage_ready: bool,
    pub(super) partial_state_proof_ready: bool,
    pub(super) transaction_failure_proof_ready: bool,
    pub(super) atomic_write_set_count: usize,
    pub(super) simulated_failure_point_count: usize,
    pub(super) rollback_action_count: usize,
    pub(super) partial_enabled_after_failure_count: usize,
    #[serde(default)]
    pub(super) atomic_write_set: Vec<String>,
    #[serde(default)]
    pub(super) simulated_failure_points: Vec<String>,
    #[serde(default)]
    pub(super) rollback_actions: Vec<String>,
    pub(super) would_write_switches: bool,
    pub(super) would_persist_partial_state: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_execution_plan_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationSwitchTransactionProofReadinessDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) transaction_name: String,
    pub(super) plan_name: String,
    pub(super) activation_execution_plan_ready: bool,
    pub(super) activation_execution_plan_readiness_ready: bool,
    pub(super) write_set_ready: bool,
    pub(super) all_or_nothing_guard_ready: bool,
    pub(super) failure_probe_coverage_ready: bool,
    pub(super) rollback_action_coverage_ready: bool,
    pub(super) partial_state_proof_ready: bool,
    pub(super) transaction_failure_proof_ready: bool,
    pub(super) atomic_write_set_count: usize,
    pub(super) simulated_failure_point_count: usize,
    pub(super) rollback_action_count: usize,
    pub(super) partial_enabled_after_failure_count: usize,
    #[serde(default)]
    pub(super) atomic_write_set: Vec<String>,
    #[serde(default)]
    pub(super) simulated_failure_points: Vec<String>,
    #[serde(default)]
    pub(super) rollback_actions: Vec<String>,
    pub(super) would_write_switches: bool,
    pub(super) would_persist_partial_state: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_switch_write_transaction_enablement: bool,
    #[serde(default)]
    pub(super) inherited_transaction_proof_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_execution_plan_readiness_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionEnablementDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) switch_name: String,
    pub(super) transaction_name: String,
    pub(super) transaction_failure_proof_ready: bool,
    pub(super) write_set_ready: bool,
    pub(super) all_or_nothing_guard_ready: bool,
    pub(super) failure_probe_coverage_ready: bool,
    pub(super) rollback_action_coverage_ready: bool,
    pub(super) partial_state_proof_ready: bool,
    pub(super) partial_enabled_after_failure_count: usize,
    pub(super) transaction_enabled: bool,
    pub(super) enablement_prerequisites_ready: bool,
    pub(super) switch_write_transaction_enablement_ready: bool,
    #[serde(default)]
    pub(super) atomic_write_set: Vec<String>,
    #[serde(default)]
    pub(super) rollback_actions: Vec<String>,
    pub(super) would_enable_transaction: bool,
    pub(super) would_write_switches: bool,
    pub(super) would_persist_partial_state: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_transaction_proof_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionEnablementReadinessDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) switch_name: String,
    pub(super) transaction_name: String,
    pub(super) transaction_failure_proof_ready: bool,
    pub(super) transaction_failure_proof_readiness_ready: bool,
    pub(super) write_set_ready: bool,
    pub(super) all_or_nothing_guard_ready: bool,
    pub(super) failure_probe_coverage_ready: bool,
    pub(super) rollback_action_coverage_ready: bool,
    pub(super) partial_state_proof_ready: bool,
    pub(super) partial_enabled_after_failure_count: usize,
    pub(super) transaction_enabled: bool,
    pub(super) enablement_prerequisites_ready: bool,
    pub(super) switch_write_transaction_enablement_ready: bool,
    #[serde(default)]
    pub(super) atomic_write_set: Vec<String>,
    #[serde(default)]
    pub(super) rollback_actions: Vec<String>,
    pub(super) would_enable_transaction: bool,
    pub(super) would_write_switches: bool,
    pub(super) would_persist_partial_state: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_switch_write_transaction: bool,
    #[serde(default)]
    pub(super) inherited_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_transaction_proof_readiness_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) transaction_name: String,
    pub(super) transaction_failure_proof_ready: bool,
    pub(super) write_set_ready: bool,
    pub(super) all_or_nothing_guard_ready: bool,
    pub(super) failure_probe_coverage_ready: bool,
    pub(super) rollback_action_coverage_ready: bool,
    pub(super) partial_state_proof_ready: bool,
    pub(super) transaction_enabled: bool,
    pub(super) transaction_prerequisites_ready: bool,
    pub(super) switch_write_transaction_ready: bool,
    pub(super) partial_enabled_after_failure_count: usize,
    #[serde(default)]
    pub(super) activation_switch_write_order: Vec<String>,
    #[serde(default)]
    pub(super) atomic_write_set: Vec<String>,
    #[serde(default)]
    pub(super) rollback_actions: Vec<String>,
    pub(super) would_write_switches: bool,
    pub(super) would_persist_partial_state: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_transaction_proof_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionReadinessDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) transaction_name: String,
    pub(super) transaction_failure_proof_ready: bool,
    pub(super) switch_write_transaction_enablement_readiness_ready: bool,
    pub(super) write_set_ready: bool,
    pub(super) all_or_nothing_guard_ready: bool,
    pub(super) failure_probe_coverage_ready: bool,
    pub(super) rollback_action_coverage_ready: bool,
    pub(super) partial_state_proof_ready: bool,
    pub(super) transaction_enabled: bool,
    pub(super) transaction_prerequisites_ready: bool,
    pub(super) switch_write_transaction_ready: bool,
    pub(super) partial_enabled_after_failure_count: usize,
    #[serde(default)]
    pub(super) activation_switch_write_order: Vec<String>,
    #[serde(default)]
    pub(super) atomic_write_set: Vec<String>,
    #[serde(default)]
    pub(super) rollback_actions: Vec<String>,
    pub(super) would_write_switches: bool,
    pub(super) would_persist_partial_state: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_activation_transaction_admission: bool,
    #[serde(default)]
    pub(super) inherited_switch_write_transaction_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_enablement_readiness_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationTransactionAdmissionGateDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) transaction_name: String,
    pub(super) activation_path_ready: bool,
    pub(super) activation_execution_plan_ready: bool,
    pub(super) transaction_failure_proof_ready: bool,
    pub(super) switch_write_transaction_enablement_ready: bool,
    pub(super) switch_write_transaction_ready: bool,
    pub(super) transaction_enabled: bool,
    pub(super) transaction_shape_ready: bool,
    pub(super) activation_transaction_admission_ready: bool,
    pub(super) partial_enabled_after_failure_count: usize,
    #[serde(default)]
    pub(super) activation_switch_write_order: Vec<String>,
    #[serde(default)]
    pub(super) atomic_write_set: Vec<String>,
    #[serde(default)]
    pub(super) rollback_actions: Vec<String>,
    pub(super) would_admit_transaction: bool,
    pub(super) would_write_switches: bool,
    pub(super) would_persist_partial_state: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_path_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_execution_plan_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_transaction_proof_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_transaction_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_switch_write_transaction_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationTransactionAdmissionReadinessDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) source_gate_name: String,
    pub(super) transaction_name: String,
    pub(super) activation_path_ready: bool,
    pub(super) activation_execution_plan_ready: bool,
    pub(super) transaction_failure_proof_ready: bool,
    pub(super) switch_write_transaction_enablement_ready: bool,
    pub(super) switch_write_transaction_ready: bool,
    pub(super) switch_write_transaction_readiness_ready: bool,
    pub(super) transaction_enabled: bool,
    pub(super) transaction_shape_ready: bool,
    pub(super) activation_transaction_admission_ready: bool,
    pub(super) partial_enabled_after_failure_count: usize,
    #[serde(default)]
    pub(super) activation_switch_write_order: Vec<String>,
    #[serde(default)]
    pub(super) atomic_write_set: Vec<String>,
    #[serde(default)]
    pub(super) rollback_actions: Vec<String>,
    pub(super) would_admit_transaction: bool,
    pub(super) would_write_switches: bool,
    pub(super) would_persist_partial_state: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_activation_admission_handoff: bool,
    #[serde(default)]
    pub(super) inherited_admission_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_switch_write_transaction_readiness_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationAdmissionHandoffDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) handoff_name: String,
    pub(super) source_gate_name: String,
    pub(super) structural_plan_ready: bool,
    pub(super) runner_control_ready: bool,
    pub(super) phase_chain_ready: bool,
    pub(super) rollback_chain_ready: bool,
    pub(super) side_effect_enablement_ready: bool,
    pub(super) activation_admission_required: bool,
    pub(super) activation_transaction_admission_ready: bool,
    pub(super) runner_activation_enabled: bool,
    pub(super) activation_handoff_prerequisites_ready: bool,
    pub(super) runner_activation_handoff_ready: bool,
    #[serde(default)]
    pub(super) activation_switch_write_order: Vec<String>,
    pub(super) would_handoff_to_runner: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_activation_admission_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_runner_enablement_blocked_by: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationAdmissionHandoffReadinessDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) handoff_name: String,
    pub(super) source_gate_name: String,
    pub(super) structural_plan_ready: bool,
    pub(super) runner_control_ready: bool,
    pub(super) phase_chain_ready: bool,
    pub(super) rollback_chain_ready: bool,
    pub(super) side_effect_enablement_ready: bool,
    pub(super) activation_admission_required: bool,
    pub(super) activation_transaction_admission_ready: bool,
    pub(super) activation_transaction_admission_readiness_ready: bool,
    pub(super) runner_activation_enabled: bool,
    pub(super) activation_handoff_prerequisites_ready: bool,
    pub(super) runner_activation_handoff_ready: bool,
    #[serde(default)]
    pub(super) activation_switch_write_order: Vec<String>,
    pub(super) would_handoff_to_runner: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_handoff_enablement: bool,
    #[serde(default)]
    pub(super) inherited_handoff_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_admission_readiness_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_runner_enablement_blocked_by: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationHandoffEnablementDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) switch_name: String,
    pub(super) handoff_name: String,
    pub(super) structural_plan_ready: bool,
    pub(super) runner_control_ready: bool,
    pub(super) phase_chain_ready: bool,
    pub(super) rollback_chain_ready: bool,
    pub(super) side_effect_enablement_ready: bool,
    pub(super) activation_admission_required: bool,
    pub(super) activation_transaction_admission_ready: bool,
    pub(super) activation_handoff_prerequisites_ready: bool,
    pub(super) runner_activation_enabled: bool,
    pub(super) activation_handoff_enablement_ready: bool,
    #[serde(default)]
    pub(super) activation_switch_write_order: Vec<String>,
    pub(super) would_enable_runner_activation: bool,
    pub(super) would_handoff_to_runner: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_handoff_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationHandoffEnablementReadinessDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) switch_name: String,
    pub(super) handoff_name: String,
    pub(super) structural_plan_ready: bool,
    pub(super) runner_control_ready: bool,
    pub(super) phase_chain_ready: bool,
    pub(super) rollback_chain_ready: bool,
    pub(super) side_effect_enablement_ready: bool,
    pub(super) activation_admission_required: bool,
    pub(super) activation_transaction_admission_ready: bool,
    pub(super) activation_admission_handoff_readiness_ready: bool,
    pub(super) activation_handoff_prerequisites_ready: bool,
    pub(super) runner_activation_enabled: bool,
    pub(super) activation_handoff_enablement_ready: bool,
    #[serde(default)]
    pub(super) activation_switch_write_order: Vec<String>,
    pub(super) would_unblock_handoff_attempt: bool,
    pub(super) would_enable_runner_activation: bool,
    pub(super) would_handoff_to_runner: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_handoff_readiness_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationHandoffAttemptDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) attempt_name: String,
    pub(super) source_handoff_name: String,
    pub(super) source_switch_name: String,
    pub(super) structural_plan_ready: bool,
    pub(super) runner_control_ready: bool,
    pub(super) phase_chain_ready: bool,
    pub(super) rollback_chain_ready: bool,
    pub(super) side_effect_enablement_ready: bool,
    pub(super) activation_admission_required: bool,
    pub(super) activation_transaction_admission_ready: bool,
    pub(super) activation_handoff_prerequisites_ready: bool,
    pub(super) runner_activation_enabled: bool,
    pub(super) activation_handoff_enablement_ready: bool,
    pub(super) handoff_attempt_ready: bool,
    #[serde(default)]
    pub(super) activation_switch_write_order: Vec<String>,
    pub(super) would_start_handoff: bool,
    pub(super) would_handoff_to_runner: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_handoff_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationHandoffAttemptReadinessDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) attempt_name: String,
    pub(super) source_handoff_name: String,
    pub(super) source_switch_name: String,
    pub(super) structural_plan_ready: bool,
    pub(super) runner_control_ready: bool,
    pub(super) phase_chain_ready: bool,
    pub(super) rollback_chain_ready: bool,
    pub(super) side_effect_enablement_ready: bool,
    pub(super) activation_admission_required: bool,
    pub(super) activation_transaction_admission_ready: bool,
    pub(super) activation_handoff_prerequisites_ready: bool,
    pub(super) runner_activation_enabled: bool,
    pub(super) activation_handoff_enablement_ready: bool,
    pub(super) activation_handoff_enablement_readiness_ready: bool,
    pub(super) handoff_attempt_ready: bool,
    #[serde(default)]
    pub(super) activation_switch_write_order: Vec<String>,
    pub(super) would_unblock_post_handoff_attempt: bool,
    pub(super) would_start_handoff: bool,
    pub(super) would_handoff_to_runner: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_attempt_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_enablement_readiness_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_handoff_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationPostHandoffAttemptDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) attempt_name: String,
    pub(super) source_attempt_name: String,
    pub(super) source_handoff_name: String,
    pub(super) source_switch_name: String,
    pub(super) structural_plan_ready: bool,
    pub(super) runner_control_ready: bool,
    pub(super) phase_chain_ready: bool,
    pub(super) rollback_chain_ready: bool,
    pub(super) side_effect_enablement_ready: bool,
    pub(super) activation_admission_required: bool,
    pub(super) activation_transaction_admission_ready: bool,
    pub(super) activation_handoff_prerequisites_ready: bool,
    pub(super) runner_activation_enabled: bool,
    pub(super) activation_handoff_enablement_ready: bool,
    pub(super) handoff_attempt_ready: bool,
    pub(super) activation_attempt_ready: bool,
    #[serde(default)]
    pub(super) activation_switch_write_order: Vec<String>,
    pub(super) would_attempt_activation: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_handoff_attempt_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_handoff_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationPostHandoffAttemptReadinessDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) attempt_name: String,
    pub(super) source_attempt_name: String,
    pub(super) source_handoff_name: String,
    pub(super) source_switch_name: String,
    pub(super) structural_plan_ready: bool,
    pub(super) runner_control_ready: bool,
    pub(super) phase_chain_ready: bool,
    pub(super) rollback_chain_ready: bool,
    pub(super) side_effect_enablement_ready: bool,
    pub(super) activation_admission_required: bool,
    pub(super) activation_transaction_admission_ready: bool,
    pub(super) activation_handoff_prerequisites_ready: bool,
    pub(super) runner_activation_enabled: bool,
    pub(super) activation_handoff_enablement_ready: bool,
    pub(super) handoff_attempt_ready: bool,
    pub(super) handoff_attempt_readiness_ready: bool,
    pub(super) activation_attempt_ready: bool,
    pub(super) activation_post_handoff_attempt_ready: bool,
    #[serde(default)]
    pub(super) activation_switch_write_order: Vec<String>,
    pub(super) would_unblock_success_admission: bool,
    pub(super) would_attempt_activation: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_post_handoff_attempt_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_handoff_attempt_readiness_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_handoff_attempt_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_handoff_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationSuccessAdmissionDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) admission_name: String,
    pub(super) source_attempt_name: String,
    pub(super) source_handoff_attempt_name: String,
    pub(super) source_handoff_name: String,
    pub(super) source_switch_name: String,
    pub(super) structural_plan_ready: bool,
    pub(super) runner_control_ready: bool,
    pub(super) phase_chain_ready: bool,
    pub(super) rollback_chain_ready: bool,
    pub(super) side_effect_enablement_ready: bool,
    pub(super) activation_admission_required: bool,
    pub(super) activation_transaction_admission_ready: bool,
    pub(super) activation_handoff_prerequisites_ready: bool,
    pub(super) runner_activation_enabled: bool,
    pub(super) activation_handoff_enablement_ready: bool,
    pub(super) handoff_attempt_ready: bool,
    pub(super) activation_attempt_ready: bool,
    pub(super) activation_post_handoff_attempt_ready: bool,
    pub(super) activation_success_admission_ready: bool,
    #[serde(default)]
    pub(super) activation_switch_write_order: Vec<String>,
    pub(super) would_admit_success: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_post_handoff_attempt_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_handoff_attempt_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_handoff_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationSuccessAdmissionReadinessDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) admission_name: String,
    pub(super) source_attempt_name: String,
    pub(super) source_handoff_attempt_name: String,
    pub(super) source_handoff_name: String,
    pub(super) source_switch_name: String,
    pub(super) structural_plan_ready: bool,
    pub(super) runner_control_ready: bool,
    pub(super) phase_chain_ready: bool,
    pub(super) rollback_chain_ready: bool,
    pub(super) side_effect_enablement_ready: bool,
    pub(super) activation_admission_required: bool,
    pub(super) activation_transaction_admission_ready: bool,
    pub(super) activation_handoff_prerequisites_ready: bool,
    pub(super) runner_activation_enabled: bool,
    pub(super) activation_handoff_enablement_ready: bool,
    pub(super) handoff_attempt_ready: bool,
    pub(super) activation_attempt_ready: bool,
    pub(super) activation_post_handoff_attempt_ready: bool,
    pub(super) activation_post_handoff_attempt_readiness_ready: bool,
    pub(super) activation_success_admission_ready: bool,
    #[serde(default)]
    pub(super) activation_switch_write_order: Vec<String>,
    pub(super) would_unblock_success_return: bool,
    pub(super) would_admit_success: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_success_admission_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_post_handoff_attempt_readiness_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_post_handoff_attempt_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_handoff_attempt_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_handoff_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationSuccessReturnDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) return_name: String,
    pub(super) source_admission_name: String,
    pub(super) source_attempt_name: String,
    pub(super) source_handoff_attempt_name: String,
    pub(super) source_handoff_name: String,
    pub(super) source_switch_name: String,
    pub(super) structural_plan_ready: bool,
    pub(super) runner_control_ready: bool,
    pub(super) phase_chain_ready: bool,
    pub(super) rollback_chain_ready: bool,
    pub(super) side_effect_enablement_ready: bool,
    pub(super) activation_admission_required: bool,
    pub(super) activation_transaction_admission_ready: bool,
    pub(super) activation_handoff_prerequisites_ready: bool,
    pub(super) runner_activation_enabled: bool,
    pub(super) activation_handoff_enablement_ready: bool,
    pub(super) handoff_attempt_ready: bool,
    pub(super) activation_attempt_ready: bool,
    pub(super) activation_post_handoff_attempt_ready: bool,
    pub(super) activation_success_admission_ready: bool,
    pub(super) activation_success_return_ready: bool,
    #[serde(default)]
    pub(super) activation_switch_write_order: Vec<String>,
    pub(super) would_admit_success: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_success_admission_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_post_handoff_attempt_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_handoff_attempt_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_handoff_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerActivationSuccessReturnReadinessDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) return_name: String,
    pub(super) source_admission_name: String,
    pub(super) source_attempt_name: String,
    pub(super) source_handoff_attempt_name: String,
    pub(super) source_handoff_name: String,
    pub(super) source_switch_name: String,
    pub(super) structural_plan_ready: bool,
    pub(super) runner_control_ready: bool,
    pub(super) phase_chain_ready: bool,
    pub(super) rollback_chain_ready: bool,
    pub(super) side_effect_enablement_ready: bool,
    pub(super) activation_admission_required: bool,
    pub(super) activation_transaction_admission_ready: bool,
    pub(super) activation_handoff_prerequisites_ready: bool,
    pub(super) runner_activation_enabled: bool,
    pub(super) activation_handoff_enablement_ready: bool,
    pub(super) handoff_attempt_ready: bool,
    pub(super) activation_attempt_ready: bool,
    pub(super) activation_post_handoff_attempt_ready: bool,
    pub(super) activation_success_admission_ready: bool,
    pub(super) activation_success_admission_readiness_ready: bool,
    pub(super) activation_success_return_ready: bool,
    #[serde(default)]
    pub(super) activation_switch_write_order: Vec<String>,
    pub(super) would_unblock_route_success: bool,
    pub(super) would_admit_success: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_success_return_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_success_admission_readiness_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_success_admission_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_post_handoff_attempt_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_handoff_attempt_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_handoff_enablement_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerRouteSuccessReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) route_status_name: String,
    pub(super) source_return_name: String,
    pub(super) source_enablement_plan_name: String,
    pub(super) structural_plan_ready: bool,
    pub(super) runner_control_ready: bool,
    pub(super) phase_chain_ready: bool,
    pub(super) rollback_chain_ready: bool,
    pub(super) side_effect_enablement_ready: bool,
    pub(super) runner_activation_enabled: bool,
    pub(super) runner_activation_ready: bool,
    pub(super) activation_success_return_ready: bool,
    pub(super) activation_success_return_readiness_ready: bool,
    pub(super) enablement_plan_success_ready: bool,
    pub(super) route_success_ready: bool,
    pub(super) would_set_route_success: bool,
    pub(super) would_mark_review_approve_executed: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_success_return_readiness_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_success_return_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_enablement_plan_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_enablement_plan_blocked_enablements: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerRouteStatusReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) current_response_status: String,
    pub(super) current_route_status: String,
    pub(super) target_response_status: String,
    pub(super) target_route_status: String,
    pub(super) expected_http_status: u16,
    pub(super) decision_execution_preflight_requested: bool,
    pub(super) review_execution_enabled: bool,
    pub(super) approve_runner_success: bool,
    pub(super) route_success_ready: bool,
    pub(super) route_status_ready: bool,
    pub(super) would_set_response_status: bool,
    pub(super) would_set_route_status: bool,
    pub(super) would_return_http_ok: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_route_success_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_blocked_reasons: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionFormalReviewExecutionReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) decision_execution_preflight_requested: bool,
    pub(super) review_request_enabled: bool,
    pub(super) review_execution_gate_clear: bool,
    pub(super) approve_execution_ready: bool,
    pub(super) route_success_ready: bool,
    pub(super) formal_approve_review_execution_enabled: bool,
    pub(super) review_execution_enabled: bool,
    pub(super) formal_review_execution_ready: bool,
    pub(super) would_execute_decision: bool,
    pub(super) would_persist_approval_record: bool,
    pub(super) would_mutate_contract: bool,
    pub(super) would_emit_lifecycle_event: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_return_http_ok: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_execution_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_approve_execution_blockers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_route_success_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_route_status_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_blocked_reasons: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionFinalAtomicReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) decision_execution_preflight_requested: bool,
    pub(super) review_execution_enabled: bool,
    pub(super) record_write_ready: bool,
    pub(super) lifecycle_effects_ready: bool,
    pub(super) contract_mutation_ready: bool,
    pub(super) recovery_marker_persistence_ready: bool,
    pub(super) transaction_commit_ready: bool,
    pub(super) route_success_ready: bool,
    pub(super) formal_review_execution_ready: bool,
    pub(super) final_atomic_execution_ready: bool,
    pub(super) would_execute_decision: bool,
    pub(super) would_persist_approval_record: bool,
    pub(super) would_emit_lifecycle_event: bool,
    pub(super) would_mutate_contract: bool,
    pub(super) would_persist_recovery_marker: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_return_http_ok: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_record_write_blockers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_lifecycle_effects_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_contract_mutation_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_recovery_marker_persistence_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_transaction_commit_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_route_success_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_formal_review_execution_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionFinalAtomicExecutionPlanDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) plan_name: String,
    pub(super) execution_order_ready: bool,
    pub(super) rollback_order_ready: bool,
    pub(super) transaction_runner_ready: bool,
    pub(super) cleanup_phase_ready: bool,
    pub(super) transaction_commit_phase_ready: bool,
    pub(super) rollback_execution_ready: bool,
    pub(super) final_atomic_readiness_ready: bool,
    pub(super) formal_review_execution_ready: bool,
    pub(super) review_execution_enabled: bool,
    pub(super) final_atomic_execution_plan_ready: bool,
    pub(super) partial_execution_allowed: bool,
    pub(super) recovery_marker_required: bool,
    pub(super) commit_barrier_ready: bool,
    pub(super) rollback_plan_ready: bool,
    pub(super) would_start_atomic_execution: bool,
    pub(super) would_persist_approval_record: bool,
    pub(super) would_mutate_contract: bool,
    pub(super) would_persist_recovery_marker: bool,
    pub(super) would_clear_recovery_marker: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_rollback_on_error: bool,
    pub(super) would_return_http_ok: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) execution_order: Vec<String>,
    #[serde(default)]
    pub(super) rollback_order: Vec<String>,
    #[serde(default)]
    pub(super) inherited_runner_blockers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_cleanup_phase_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_transaction_commit_phase_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_rollback_execution_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_final_atomic_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionFinalAtomicAdmissionGateDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) execution_plan_structural_ready: bool,
    pub(super) execution_order_ready: bool,
    pub(super) rollback_order_ready: bool,
    pub(super) final_atomic_execution_plan_ready: bool,
    pub(super) final_atomic_readiness_ready: bool,
    pub(super) formal_review_execution_ready: bool,
    pub(super) review_execution_enabled: bool,
    pub(super) partial_execution_allowed: bool,
    pub(super) recovery_marker_required: bool,
    pub(super) commit_barrier_ready: bool,
    pub(super) rollback_plan_ready: bool,
    pub(super) admission_ready: bool,
    pub(super) would_enter_final_execution: bool,
    pub(super) would_execute_decision: bool,
    pub(super) would_return_http_ok: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_execution_plan_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_execution_order: Vec<String>,
    #[serde(default)]
    pub(super) inherited_rollback_order: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionFinalExecutionEntryDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) entry_name: String,
    pub(super) admission_ready: bool,
    pub(super) review_execution_enabled: bool,
    pub(super) approve_final_execution_enabled: bool,
    pub(super) route_status_ready: bool,
    pub(super) rollback_order_ready: bool,
    pub(super) no_partial_execution_ready: bool,
    pub(super) final_execution_entry_ready: bool,
    pub(super) would_enter_final_execution: bool,
    pub(super) would_execute_decision: bool,
    pub(super) would_persist_approval_record: bool,
    pub(super) would_mutate_contract: bool,
    pub(super) would_persist_recovery_marker: bool,
    pub(super) would_clear_recovery_marker: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_return_http_ok: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_admission_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_execution_order: Vec<String>,
    #[serde(default)]
    pub(super) inherited_rollback_order: Vec<String>,
    #[serde(default)]
    pub(super) inherited_route_status_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_blocked_reasons: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionFinalExecutionSwitchReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) switch_name: String,
    pub(super) final_execution_entry_ready: bool,
    pub(super) approve_final_execution_enabled: bool,
    pub(super) record_write_ready: bool,
    pub(super) contract_mutation_ready: bool,
    pub(super) recovery_marker_persistence_ready: bool,
    pub(super) cleanup_phase_ready: bool,
    pub(super) transaction_commit_phase_ready: bool,
    pub(super) route_status_ready: bool,
    pub(super) rollback_order_ready: bool,
    pub(super) no_partial_execution_ready: bool,
    pub(super) final_execution_switch_ready: bool,
    pub(super) final_execution_switch_enabled: bool,
    pub(super) side_effect_replay_required: bool,
    pub(super) would_enable_final_execution: bool,
    pub(super) would_persist_approval_record: bool,
    pub(super) would_mutate_contract: bool,
    pub(super) would_persist_recovery_marker: bool,
    pub(super) would_clear_recovery_marker: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_return_http_ok: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) replay_order: Vec<String>,
    #[serde(default)]
    pub(super) inherited_final_entry_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionFinalExecutionRollbackReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) final_execution_switch_ready: bool,
    pub(super) approve_final_execution_enabled: bool,
    pub(super) rollback_execution_ready: bool,
    pub(super) rollback_plan_ready: bool,
    pub(super) rollback_order_ready: bool,
    pub(super) failure_window_covered: bool,
    pub(super) final_execution_rollback_ready: bool,
    pub(super) would_rollback_on_error: bool,
    pub(super) would_restore_contract_source: bool,
    pub(super) would_restore_approval_record: bool,
    pub(super) would_mark_recovery_marker_rolled_back: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) rollback_order: Vec<String>,
    #[serde(default)]
    pub(super) inherited_switch_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_rollback_execution_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionFinalExecutionReplayPlanDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) plan_name: String,
    pub(super) final_execution_switch_ready: bool,
    pub(super) final_execution_rollback_ready: bool,
    pub(super) approve_final_execution_enabled: bool,
    pub(super) replay_order_ready: bool,
    pub(super) rollback_order_ready: bool,
    pub(super) replay_plan_ready: bool,
    pub(super) replay_enabled: bool,
    pub(super) side_effect_replay_required: bool,
    pub(super) would_replay_side_effects: bool,
    pub(super) would_enter_final_execution: bool,
    pub(super) would_return_http_ok: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) replay_order: Vec<String>,
    #[serde(default)]
    pub(super) rollback_order: Vec<String>,
    #[serde(default)]
    pub(super) required_replay_phases: Vec<String>,
    #[serde(default)]
    pub(super) missing_replay_phases: Vec<String>,
    #[serde(default)]
    pub(super) inherited_switch_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_rollback_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionFinalExecutionReplayExecutorDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) executor_name: String,
    pub(super) replay_plan_ready: bool,
    pub(super) final_execution_rollback_ready: bool,
    pub(super) approve_final_execution_enabled: bool,
    pub(super) replay_executor_ready: bool,
    pub(super) replay_executor_admitted: bool,
    pub(super) replay_executor_order_ready: bool,
    pub(super) would_start_replay_executor: bool,
    pub(super) would_write_recovery_marker: bool,
    pub(super) would_persist_approval_record: bool,
    pub(super) would_emit_lifecycle_event: bool,
    pub(super) would_append_lifecycle_entry: bool,
    pub(super) would_write_contract_source: bool,
    pub(super) would_clear_recovery_marker: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_dispatch_route_success: bool,
    pub(super) would_return_http_ok: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) expected_replay_order: Vec<String>,
    #[serde(default)]
    pub(super) missing_executor_replay_phases: Vec<String>,
    #[serde(default)]
    pub(super) unexpected_executor_replay_phases: Vec<String>,
    #[serde(default)]
    pub(super) replay_order: Vec<String>,
    #[serde(default)]
    pub(super) rollback_order: Vec<String>,
    #[serde(default)]
    pub(super) inherited_replay_plan_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionFinalExecutionReplayExecutorRoutingDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) routing_name: String,
    pub(super) replay_executor_ready: bool,
    pub(super) replay_executor_order_ready: bool,
    pub(super) approve_final_execution_enabled: bool,
    pub(super) handler_routing_ready: bool,
    pub(super) replay_executor_admitted: bool,
    pub(super) routing_admitted: bool,
    pub(super) executor_routing_required: bool,
    pub(super) would_route_through_executor: bool,
    pub(super) would_start_replay_executor: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) expected_replay_order: Vec<String>,
    #[serde(default)]
    pub(super) handler_phases: Vec<String>,
    #[serde(default)]
    pub(super) handler_routes: Vec<String>,
    #[serde(default)]
    pub(super) missing_handler_phases: Vec<String>,
    #[serde(default)]
    pub(super) unexpected_handler_phases: Vec<String>,
    #[serde(default)]
    pub(super) inherited_executor_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionFinalExecutionRoutedWriteHandoffDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) handoff_name: String,
    pub(super) approve_final_execution_enabled: bool,
    pub(super) routed_write_handoff_enabled: bool,
    pub(super) legacy_inline_final_writes_enabled: bool,
    pub(super) replay_executor_ready: bool,
    pub(super) replay_executor_order_ready: bool,
    pub(super) handler_routing_ready: bool,
    pub(super) routing_ready: bool,
    pub(super) legacy_inline_final_writes_blocked: bool,
    pub(super) routed_write_handoff_ready: bool,
    pub(super) routed_write_handoff_admitted: bool,
    pub(super) would_route_writes_through_executor: bool,
    pub(super) would_start_replay_executor: bool,
    pub(super) would_write_recovery_marker: bool,
    pub(super) would_persist_approval_record: bool,
    pub(super) would_emit_lifecycle_event: bool,
    pub(super) would_append_lifecycle_entry: bool,
    pub(super) would_write_contract_source: bool,
    pub(super) would_clear_recovery_marker: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_dispatch_route_success: bool,
    pub(super) would_return_http_ok: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) handoff_phases: Vec<String>,
    #[serde(default)]
    pub(super) handler_routes: Vec<String>,
    #[serde(default)]
    pub(super) inline_final_write_gates: Vec<String>,
    #[serde(default)]
    pub(super) missing_handoff_phases: Vec<String>,
    #[serde(default)]
    pub(super) unexpected_handoff_phases: Vec<String>,
    #[serde(default)]
    pub(super) inherited_routing_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionFinalExecutionRoutedHandlerPlanDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) plan_name: String,
    pub(super) routed_write_handoff_ready: bool,
    pub(super) routed_write_handoff_admitted: bool,
    pub(super) routing_ready: bool,
    pub(super) handler_execution_plan_ready: bool,
    pub(super) handler_count: usize,
    pub(super) ready_handler_count: usize,
    pub(super) blocked_handler_count: usize,
    pub(super) would_execute_handlers: bool,
    pub(super) would_start_replay_executor: bool,
    pub(super) would_write_recovery_marker: bool,
    pub(super) would_transition_review: bool,
    pub(super) would_persist_approval_record: bool,
    pub(super) would_emit_lifecycle_event: bool,
    pub(super) would_append_lifecycle_entry: bool,
    pub(super) would_write_contract_source: bool,
    pub(super) would_clear_recovery_marker: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_dispatch_route_success: bool,
    pub(super) would_return_http_ok: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) handler_phases: Vec<String>,
    #[serde(default)]
    pub(super) handler_routes: Vec<String>,
    #[serde(default)]
    pub(super) handler_readiness: Vec<String>,
    #[serde(default)]
    pub(super) ready_handlers: Vec<String>,
    #[serde(default)]
    pub(super) blocked_handlers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_handoff_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionFinalExecutionRoutedExecutionAttemptDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) attempt_name: String,
    pub(super) handler_execution_plan_ready: bool,
    pub(super) routed_write_handoff_admitted: bool,
    pub(super) execution_attempt_ready: bool,
    pub(super) execution_attempt_admitted: bool,
    pub(super) execution_attempt_blocked_reason: String,
    pub(super) handler_count: usize,
    pub(super) ready_handler_count: usize,
    pub(super) blocked_handler_count: usize,
    pub(super) would_execute_handlers: bool,
    pub(super) would_start_replay_executor: bool,
    pub(super) would_write_recovery_marker: bool,
    pub(super) would_transition_review: bool,
    pub(super) would_persist_approval_record: bool,
    pub(super) would_emit_lifecycle_event: bool,
    pub(super) would_append_lifecycle_entry: bool,
    pub(super) would_write_contract_source: bool,
    pub(super) would_clear_recovery_marker: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_dispatch_route_success: bool,
    pub(super) would_return_http_ok: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) execution_order: Vec<String>,
    #[serde(default)]
    pub(super) handler_readiness: Vec<String>,
    #[serde(default)]
    pub(super) inherited_handler_plan_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionFinalExecutionOrderedHandlerExecutionConfirmationDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) confirmation_name: String,
    pub(super) execution_attempt_admitted: bool,
    pub(super) execution_order_ready: bool,
    pub(super) handler_readiness_ready: bool,
    pub(super) handler_count: usize,
    pub(super) ready_handler_count: usize,
    pub(super) blocked_handler_count: usize,
    pub(super) rollback_confirmation_ready: bool,
    pub(super) no_partial_write_guard_ready: bool,
    pub(super) ordered_handler_execution_ready: bool,
    pub(super) ordered_handler_execution_connection_preflight_ready: bool,
    pub(super) ordered_handler_execution_connection_enabled: bool,
    pub(super) ordered_handler_execution_connected: bool,
    pub(super) ordered_handler_execution_confirmed: bool,
    pub(super) ordered_handler_execution_dry_run_ready: bool,
    pub(super) ordered_handler_execution_dry_run_complete: bool,
    pub(super) dry_run_handler_execution_count: usize,
    pub(super) dry_run_handler_execution_effects_blocked: bool,
    pub(super) confirmation_blocked_reason: String,
    pub(super) would_execute_handlers: bool,
    pub(super) would_start_replay_executor: bool,
    pub(super) would_write_recovery_marker: bool,
    pub(super) would_transition_review: bool,
    pub(super) would_persist_approval_record: bool,
    pub(super) would_emit_lifecycle_event: bool,
    pub(super) would_append_lifecycle_entry: bool,
    pub(super) would_write_contract_source: bool,
    pub(super) would_clear_recovery_marker: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_dispatch_route_success: bool,
    pub(super) would_return_http_ok: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) execution_order: Vec<String>,
    #[serde(default)]
    pub(super) handler_readiness: Vec<String>,
    #[serde(default)]
    pub(super) dry_run_handler_execution_order: Vec<String>,
    #[serde(default)]
    pub(super) dry_run_handler_execution_receipts: Vec<String>,
    #[serde(default)]
    pub(super) unconfirmed_handlers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_attempt_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionFinalExecutionRoutedRouteSuccessReleaseDryRun
{
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) release_name: String,
    pub(super) execution_attempt_admitted: bool,
    pub(super) route_success_phase_ready: bool,
    pub(super) legacy_inline_final_writes_enabled: bool,
    pub(super) legacy_inline_final_writes_blocked: bool,
    pub(super) response_status_transition_ready: bool,
    pub(super) route_status_transition_ready: bool,
    pub(super) ordered_handler_execution_required: bool,
    pub(super) ordered_handler_execution_confirmed: bool,
    pub(super) response_status_connection_ready: bool,
    pub(super) response_status_connection_blocked_reason: String,
    pub(super) routed_route_success_release_ready: bool,
    pub(super) routed_route_success_release_connected: bool,
    pub(super) routed_route_success_release_admitted: bool,
    pub(super) routed_route_success_release_application_ready: bool,
    pub(super) routed_route_success_release_application_enabled: bool,
    pub(super) routed_route_success_release_applied: bool,
    pub(super) response_status_application_blocked_reason: String,
    pub(super) current_response_status: String,
    pub(super) current_route_status: String,
    pub(super) target_response_status: String,
    pub(super) target_route_status: String,
    pub(super) would_release_route_success: bool,
    pub(super) would_set_response_status: bool,
    pub(super) would_set_route_status: bool,
    pub(super) would_return_http_ok: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_attempt_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_ordered_handler_execution_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) execution_order: Vec<String>,
    #[serde(default)]
    pub(super) handler_readiness: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionFinalExecutionDurableWritebackBundleDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) bundle_name: String,
    pub(super) response_application_success: bool,
    pub(super) approval_record_write_ready: bool,
    pub(super) approval_record_persistence_enabled: bool,
    pub(super) contract_source_write_ready: bool,
    pub(super) contract_source_write_enabled: bool,
    pub(super) recovery_marker_persistence_ready: bool,
    pub(super) transaction_commit_ready: bool,
    pub(super) recovery_marker_cleanup_ready: bool,
    pub(super) rollback_ready: bool,
    pub(super) no_partial_write_guard_ready: bool,
    pub(super) durable_writeback_bundle_ready: bool,
    pub(super) durable_writeback_bundle_enabled: bool,
    pub(super) durable_writeback_bundle_admitted: bool,
    pub(super) durable_writeback_bundle_execution_enabled: bool,
    pub(super) durable_writeback_bundle_execution_admitted: bool,
    pub(super) durable_writeback_bundle_disk_application_enabled: bool,
    pub(super) durable_writeback_bundle_disk_application_admitted: bool,
    pub(super) durable_writeback_bundle_execution_preflight_ready: bool,
    pub(super) dry_run_execution_ready: bool,
    pub(super) dry_run_execution_complete: bool,
    pub(super) dry_run_execution_count: usize,
    pub(super) dry_run_execution_effects_blocked: bool,
    pub(super) rollback_dry_run_ready: bool,
    pub(super) rollback_dry_run_complete: bool,
    pub(super) rollback_dry_run_count: usize,
    pub(super) rollback_dry_run_effects_blocked: bool,
    pub(super) rollback_coverage_ready: bool,
    pub(super) durable_execution_rollback_barrier_ready: bool,
    pub(super) disk_application_plan_ready: bool,
    pub(super) disk_application_plan_complete: bool,
    pub(super) disk_application_effect_count: usize,
    pub(super) disk_application_effects_blocked: bool,
    pub(super) approval_record_disk_write_planned: bool,
    pub(super) contract_source_disk_write_planned: bool,
    pub(super) recovery_marker_disk_write_planned: bool,
    pub(super) recovery_marker_cleanup_planned: bool,
    pub(super) transaction_commit_planned: bool,
    pub(super) approval_record_file_name: String,
    pub(super) recovery_marker_file_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) contract_source_digest_before: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) contract_source_digest_after: Option<String>,
    pub(super) disk_application_transaction_proof_ready: bool,
    pub(super) disk_application_atomic_write_set_ready: bool,
    pub(super) disk_application_all_or_nothing_guard_ready: bool,
    pub(super) disk_application_failure_probe_coverage_ready: bool,
    pub(super) disk_application_rollback_action_coverage_ready: bool,
    pub(super) disk_application_cleanup_recovery_action_ready: bool,
    pub(super) disk_application_commit_terminal_verification_ready: bool,
    pub(super) disk_application_terminal_recovery_proof_ready: bool,
    pub(super) disk_application_partial_state_proof_ready: bool,
    pub(super) disk_application_transaction_failure_proof_ready: bool,
    pub(super) disk_application_executor_name: String,
    pub(super) disk_application_executor_ready: bool,
    pub(super) disk_application_executor_admitted: bool,
    pub(super) disk_application_executor_effects_blocked: bool,
    pub(super) disk_application_executor_blocked_reason: String,
    pub(super) disk_application_endpoint_helper_name: String,
    pub(super) disk_application_endpoint_helper_wired: bool,
    pub(super) disk_application_endpoint_helper_admission_ready: bool,
    pub(super) disk_application_endpoint_helper_admitted: bool,
    pub(super) disk_application_endpoint_helper_execution_connected: bool,
    pub(super) disk_application_endpoint_helper_execution_blocked: bool,
    pub(super) disk_application_endpoint_helper_would_execute: bool,
    pub(super) disk_application_endpoint_helper_blocked_reason: String,
    pub(super) disk_application_endpoint_helper_execution_blocked_reason: String,
    pub(super) disk_application_executor_handler_count: usize,
    pub(super) disk_application_executor_ready_handler_count: usize,
    pub(super) disk_application_executor_blocked_handler_count: usize,
    pub(super) disk_application_atomic_write_set_count: usize,
    pub(super) disk_application_simulated_failure_point_count: usize,
    pub(super) disk_application_rollback_action_count: usize,
    pub(super) disk_application_partial_enabled_after_failure_count: usize,
    pub(super) bundle_blocked_reason: String,
    pub(super) would_persist_approval_record: bool,
    pub(super) would_write_contract_source: bool,
    pub(super) would_persist_recovery_marker: bool,
    pub(super) would_clear_recovery_marker: bool,
    pub(super) would_commit_transaction: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) execution_order: Vec<String>,
    #[serde(default)]
    pub(super) rollback_order: Vec<String>,
    #[serde(default)]
    pub(super) dry_run_durable_execution_order: Vec<String>,
    #[serde(default)]
    pub(super) dry_run_durable_execution_receipts: Vec<String>,
    #[serde(default)]
    pub(super) dry_run_rollback_order: Vec<String>,
    #[serde(default)]
    pub(super) dry_run_rollback_receipts: Vec<String>,
    #[serde(default)]
    pub(super) rollback_coverage_pairs: Vec<String>,
    #[serde(default)]
    pub(super) uncovered_durable_phases: Vec<String>,
    #[serde(default)]
    pub(super) disk_application_plan_receipts: Vec<String>,
    #[serde(default)]
    pub(super) disk_application_atomic_write_set: Vec<String>,
    #[serde(default)]
    pub(super) disk_application_simulated_failure_points: Vec<String>,
    #[serde(default)]
    pub(super) disk_application_rollback_actions: Vec<String>,
    #[serde(default)]
    pub(super) disk_application_executor_handler_routes: Vec<String>,
    #[serde(default)]
    pub(super) disk_application_executor_ready_handlers: Vec<String>,
    #[serde(default)]
    pub(super) disk_application_executor_blocked_handlers: Vec<String>,
    #[serde(default)]
    pub(super) disk_application_endpoint_helper_required_inputs: Vec<String>,
    #[serde(default)]
    pub(super) disk_application_endpoint_helper_ready_inputs: Vec<String>,
    #[serde(default)]
    pub(super) disk_application_endpoint_helper_blocked_inputs: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionDurableDiskApplicationExecution {
    pub(super) status: String,
    pub(super) executor_name: String,
    pub(super) endpoint_helper_execution_connected: bool,
    pub(super) endpoint_helper_would_execute: bool,
    pub(super) executor_admitted: bool,
    pub(super) recovery_marker_written: bool,
    pub(super) approval_record_persisted: bool,
    pub(super) contract_source_written: bool,
    pub(super) recovery_marker_cleared: bool,
    pub(super) transaction_committed: bool,
    pub(super) rollback_executed: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) execution_receipts: Vec<String>,
    #[serde(default)]
    pub(super) rollback_receipts: Vec<String>,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionDecisionLockSummaryDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) response_status: String,
    pub(super) route_status: String,
    pub(super) target_response_status: String,
    pub(super) target_route_status: String,
    pub(super) expected_http_status: u16,
    pub(super) decision_execution_preflight_requested: bool,
    pub(super) review_execution_enabled: bool,
    pub(super) approve_runner_success: bool,
    pub(super) routed_route_success_release_applied: bool,
    pub(super) final_response_application_success: bool,
    pub(super) route_status_ready: bool,
    pub(super) final_execution_entry_ready: bool,
    pub(super) approve_final_execution_enabled: bool,
    pub(super) final_execution_switch_enabled: bool,
    pub(super) final_execution_locked: bool,
    pub(super) primary_blocked_reason: String,
    pub(super) blocked_reason_count: usize,
    pub(super) would_execute_decision: bool,
    pub(super) would_mutate_contract: bool,
    pub(super) would_return_http_ok: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_route_status_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_final_execution_entry_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) inherited_blocked_reasons: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerControlReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) runner_attempt_ready: bool,
    pub(super) runner_execution_ready: bool,
    pub(super) dispatch_ready: bool,
    pub(super) call_ready: bool,
    pub(super) body_ready: bool,
    pub(super) phases_ready: bool,
    pub(super) runner_control_ready: bool,
    pub(super) would_unblock_activation_control: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_runner_attempt_blockers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_runner_outcome_blockers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_dispatch_blockers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_call_blockers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_body_blockers: Vec<String>,
    #[serde(default)]
    pub(super) inherited_phase_sequence_blockers: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionRunnerEnablementPlanDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) structural_plan_ready: bool,
    pub(super) runner_control_ready: bool,
    pub(super) phase_chain_ready: bool,
    pub(super) rollback_chain_ready: bool,
    pub(super) side_effect_enablement_ready: bool,
    pub(super) runner_activation_enabled: bool,
    pub(super) runner_activation_ready: bool,
    pub(super) would_activate_runner: bool,
    pub(super) would_return_success: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) required_enablements: Vec<String>,
    #[serde(default)]
    pub(super) passed_enablements: Vec<String>,
    #[serde(default)]
    pub(super) blocked_enablements: Vec<String>,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalPersistencePlanPreview {
    pub(super) status: String,
    pub(super) persistence_enabled: bool,
    pub(super) would_write_record: bool,
    pub(super) store_kind: String,
    pub(super) record_kind: String,
    pub(super) record_key: String,
    pub(super) idempotency_key: String,
    pub(super) record_source_kind: String,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalPersistencePathPreview {
    pub(super) status: String,
    pub(super) store_kind: String,
    pub(super) record_key: String,
    pub(super) path_segment: String,
    pub(super) file_name: String,
    pub(super) atomic_write_required: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalRecordPreview {
    pub(super) status: String,
    pub(super) approval_id: String,
    pub(super) idempotency_key: String,
    pub(super) target_path: String,
    pub(super) target_kind: String,
    #[serde(default)]
    pub(super) changed_fields: Vec<String>,
    #[serde(default)]
    pub(super) patch_payload: std::collections::BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub(super) contract_source_ref: ContractRepairApprovalContractSourceRef,
    pub(super) review_state: RuntimeApprovalReviewState,
    pub(super) reviewers_required: u8,
    pub(super) transient_review_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) transient_review_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) transient_reviewer_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) transient_review_reason: Option<String>,
    #[serde(default)]
    pub(super) lifecycle: Vec<ContractRepairApprovalLifecycleEntryPreview>,
    pub(super) would_persist: bool,
    pub(super) persistence_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalLifecycleEntryPreview {
    pub(super) event_id: String,
    pub(super) review_state: RuntimeApprovalReviewState,
    pub(super) sequence_no: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalRecordSnapshotPreview {
    pub(super) status: String,
    pub(super) approval_id: String,
    pub(super) record_kind: String,
    pub(super) target_path: String,
    pub(super) target_kind: String,
    #[serde(default)]
    pub(super) changed_fields: Vec<String>,
    #[serde(default)]
    pub(super) patch_payload: std::collections::BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub(super) contract_source_ref: ContractRepairApprovalContractSourceRef,
    pub(super) review_state: RuntimeApprovalReviewState,
    pub(super) reviewer_id: String,
    pub(super) review_reason: String,
    pub(super) idempotency_key: String,
    pub(super) persistence_enabled: bool,
    pub(super) would_write_record: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalStorageReadinessGate {
    pub(super) status: String,
    pub(super) persistence_enabled: bool,
    pub(super) store_ready: bool,
    pub(super) schema_ready: bool,
    pub(super) idempotency_ready: bool,
    pub(super) snapshot_ready: bool,
    #[serde(default)]
    pub(super) ready_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalStorageDryRunPreview {
    pub(super) status: String,
    pub(super) adapter_kind: String,
    pub(super) store_kind: String,
    pub(super) record_key: String,
    pub(super) would_write: bool,
    pub(super) accepted_by_adapter: bool,
    pub(super) readiness_status: String,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalIdempotencyPrecheck {
    pub(super) status: String,
    pub(super) idempotency_key: String,
    pub(super) candidate_record_key: String,
    pub(super) store_lookup_enabled: bool,
    pub(super) existing_record_found: bool,
    pub(super) conflict_detected: bool,
    pub(super) safe_to_write: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalReviewerAuthorizationPrecheck {
    pub(super) status: String,
    pub(super) policy_version: String,
    pub(super) required_role: String,
    pub(super) grant_source: String,
    pub(super) reviewer_id: String,
    pub(super) auth_subject: String,
    pub(super) identity_format_valid: bool,
    pub(super) identity_matches_auth_subject: bool,
    pub(super) role_policy_available: bool,
    pub(super) authorized: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalReviewTransitionDryRun {
    pub(super) status: String,
    pub(super) approval_id: String,
    pub(super) from_review_state: RuntimeApprovalReviewState,
    pub(super) target_review_state: RuntimeApprovalReviewState,
    pub(super) reviewer_id: String,
    pub(super) reason_code: String,
    pub(super) sequence_no_preview: u64,
    pub(super) transition_ready: bool,
    pub(super) would_transition: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalRecordWriteDryRun {
    pub(super) status: String,
    pub(super) adapter_kind: String,
    pub(super) store_kind: String,
    pub(super) record_key: String,
    pub(super) file_name: String,
    pub(super) transition_ready: bool,
    pub(super) storage_ready: bool,
    pub(super) schema_ready: bool,
    pub(super) idempotency_ready: bool,
    pub(super) snapshot_ready: bool,
    pub(super) idempotency_precheck_passed: bool,
    pub(super) write_ready: bool,
    pub(super) would_write: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalLifecycleEventDryRun {
    pub(super) status: String,
    pub(super) event_id: String,
    pub(super) event_kind: String,
    pub(super) target_review_state: RuntimeApprovalReviewState,
    pub(super) actor_id: String,
    pub(super) reason_code: String,
    pub(super) sequence_no: u64,
    pub(super) transition_ready: bool,
    pub(super) event_payload_ready: bool,
    pub(super) emission_ready: bool,
    pub(super) would_emit: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalLifecycleEntryAppendDryRun {
    pub(super) status: String,
    pub(super) event_id: String,
    pub(super) review_state: RuntimeApprovalReviewState,
    pub(super) sequence_no: u64,
    pub(super) entry_ready: bool,
    pub(super) emission_ready: bool,
    pub(super) append_ready: bool,
    pub(super) would_append: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalLifecycleEmissionEnablementGate {
    pub(super) status: String,
    pub(super) event_id: String,
    pub(super) transition_ready: bool,
    pub(super) event_payload_ready: bool,
    pub(super) emission_ready: bool,
    pub(super) entry_append_ready: bool,
    pub(super) lifecycle_emission_plan_ready: bool,
    pub(super) lifecycle_event_emission_enabled: bool,
    pub(super) lifecycle_entry_append_enabled: bool,
    pub(super) lifecycle_effects_ready: bool,
    pub(super) would_emit: bool,
    pub(super) would_append: bool,
    pub(super) would_touch_lifecycle_log: bool,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionLifecycleEffectsReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) event_id: String,
    pub(super) transition_ready: bool,
    pub(super) event_payload_ready: bool,
    pub(super) emission_ready: bool,
    pub(super) entry_append_ready: bool,
    pub(super) lifecycle_emission_plan_ready: bool,
    pub(super) lifecycle_event_emission_enabled: bool,
    pub(super) lifecycle_entry_append_enabled: bool,
    pub(super) lifecycle_effects_ready: bool,
    pub(super) would_emit: bool,
    pub(super) would_append: bool,
    pub(super) would_touch_lifecycle_log: bool,
    pub(super) would_unblock_atomic_side_effects: bool,
    #[serde(default)]
    pub(super) inherited_lifecycle_emission_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionLifecycleEventEmissionEnablementDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) switch_name: String,
    pub(super) event_id: String,
    pub(super) transition_ready: bool,
    pub(super) event_payload_ready: bool,
    pub(super) emission_ready: bool,
    pub(super) entry_append_ready: bool,
    pub(super) lifecycle_emission_plan_ready: bool,
    pub(super) lifecycle_event_emission_enabled: bool,
    pub(super) lifecycle_event_emission_enablement_ready: bool,
    pub(super) would_enable_lifecycle_event_emission: bool,
    pub(super) would_emit: bool,
    pub(super) would_append: bool,
    pub(super) would_touch_lifecycle_log: bool,
    pub(super) would_unblock_lifecycle_effects: bool,
    #[serde(default)]
    pub(super) inherited_lifecycle_emission_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionLifecycleEntryAppendEnablementDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) switch_name: String,
    pub(super) event_id: String,
    pub(super) transition_ready: bool,
    pub(super) event_payload_ready: bool,
    pub(super) emission_ready: bool,
    pub(super) entry_append_ready: bool,
    pub(super) lifecycle_emission_plan_ready: bool,
    pub(super) lifecycle_entry_append_enabled: bool,
    pub(super) lifecycle_entry_append_enablement_ready: bool,
    pub(super) would_enable_lifecycle_entry_append: bool,
    pub(super) would_emit: bool,
    pub(super) would_append: bool,
    pub(super) would_touch_lifecycle_log: bool,
    pub(super) would_unblock_lifecycle_effects: bool,
    #[serde(default)]
    pub(super) inherited_lifecycle_emission_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalContractWritebackDryRun {
    pub(super) status: String,
    pub(super) patch_kind: String,
    pub(super) target_path: String,
    pub(super) target_kind: String,
    #[serde(default)]
    pub(super) changed_fields: Vec<String>,
    #[serde(default)]
    pub(super) patch_payload: std::collections::BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub(super) contract_source_ref: ContractRepairApprovalContractSourceRef,
    pub(super) source_resolution: ContractRepairApprovalContractSourceResolutionDryRun,
    pub(super) patch_plan: ContractRepairApprovalContractPatchPlanPreview,
    pub(super) patch_apply_dry_run: ContractRepairApprovalContractPatchApplyDryRun,
    pub(super) source_write_dry_run: ContractRepairApprovalContractSourceWriteDryRun,
    pub(super) eligible_after_approval: bool,
    pub(super) patch_ready: bool,
    pub(super) patch_payload_ready: bool,
    pub(super) contract_source_ready: bool,
    pub(super) transition_ready: bool,
    pub(super) lifecycle_append_ready: bool,
    pub(super) writeback_ready: bool,
    #[serde(default)]
    pub(super) missing_patch_fields: Vec<String>,
    #[serde(default)]
    pub(super) missing_contract_source_fields: Vec<String>,
    pub(super) would_mutate_contract: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalContractMutationEnablementGate {
    pub(super) status: String,
    pub(super) target_path: String,
    pub(super) target_kind: String,
    pub(super) source_path: String,
    pub(super) writeback_plan_ready: bool,
    pub(super) source_write_ready: bool,
    pub(super) lifecycle_effects_ready: bool,
    pub(super) contract_mutation_api_enabled: bool,
    pub(super) mutation_ready: bool,
    pub(super) would_mutate_contract: bool,
    pub(super) would_write_source: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalApproveExecutionContractMutationReadinessDryRun {
    pub(super) status: String,
    pub(super) action: String,
    pub(super) approval_id: String,
    pub(super) gate_name: String,
    pub(super) target_path: String,
    pub(super) target_kind: String,
    pub(super) source_path: String,
    pub(super) writeback_plan_ready: bool,
    pub(super) source_write_ready: bool,
    pub(super) lifecycle_effects_ready: bool,
    pub(super) contract_mutation_api_enabled: bool,
    pub(super) contract_mutation_ready: bool,
    pub(super) would_mutate_contract: bool,
    pub(super) would_write_source: bool,
    pub(super) would_touch_disk: bool,
    pub(super) would_unblock_atomic_side_effects: bool,
    #[serde(default)]
    pub(super) inherited_contract_mutation_blocked_gates: Vec<String>,
    #[serde(default)]
    pub(super) required_gates: Vec<String>,
    #[serde(default)]
    pub(super) passed_gates: Vec<String>,
    #[serde(default)]
    pub(super) blocked_gates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalContractSourceResolutionDryRun {
    pub(super) status: String,
    pub(super) source_kind: String,
    pub(super) source_id: String,
    pub(super) version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) artifact_digest: Option<String>,
    pub(super) source_path: String,
    pub(super) source_kind_supported: bool,
    pub(super) source_exists: bool,
    pub(super) source_id_match: bool,
    pub(super) version_match: bool,
    pub(super) artifact_digest_match: bool,
    pub(super) contract_shape_ready: bool,
    pub(super) resolved: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalContractPatchPlanPreview {
    pub(super) status: String,
    pub(super) plan_kind: String,
    pub(super) target_kind: String,
    pub(super) target_path: String,
    pub(super) contract_patch_ready: bool,
    pub(super) evidence_sample_patch: bool,
    #[serde(default)]
    pub(super) operations: Vec<ContractRepairApprovalContractPatchOperationPreview>,
    #[serde(default)]
    pub(super) unsupported_fields: Vec<String>,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalContractPatchOperationPreview {
    pub(super) operation: String,
    pub(super) domain: String,
    pub(super) selector: String,
    pub(super) field_name: String,
    pub(super) value: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalContractPatchApplyDryRun {
    pub(super) status: String,
    pub(super) target_kind: String,
    pub(super) target_path: String,
    pub(super) source_resolved: bool,
    pub(super) patch_plan_ready: bool,
    pub(super) apply_ready: bool,
    pub(super) operation_count: usize,
    pub(super) applied_operation_count: usize,
    #[serde(default)]
    pub(super) applied_selectors: Vec<String>,
    pub(super) would_persist_source: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalContractSourceWriteDryRun {
    pub(super) status: String,
    pub(super) source_path: String,
    pub(super) temp_file_name: String,
    pub(super) source_resolved: bool,
    pub(super) patch_apply_ready: bool,
    pub(super) write_ready: bool,
    pub(super) atomic_write_required: bool,
    pub(super) operation_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) source_digest_before: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) source_digest_after: Option<String>,
    pub(super) would_write_source: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) blocked_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalReadModelResponse {
    pub(super) status: String,
    pub(super) route_status: String,
    pub(super) record_source_status: String,
    pub(super) record_source_kind: String,
    pub(super) persisted_record_count: usize,
    pub(super) preview_record_count: usize,
    pub(super) persistence_enabled: bool,
    pub(super) mutation_enabled: bool,
    pub(super) decision_lock_summary: ContractRepairApprovalReadModelDecisionLockSummary,
    #[serde(default)]
    pub(super) records: Vec<ContractRepairApprovalRecordPreview>,
    #[serde(default)]
    pub(super) blocked_reasons: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalDetailReadModelResponse {
    pub(super) status: String,
    pub(super) approval_id: String,
    pub(super) route_status: String,
    pub(super) record_source_status: String,
    pub(super) record_source_kind: String,
    pub(super) persisted_record_count: usize,
    pub(super) preview_record_count: usize,
    pub(super) persistence_enabled: bool,
    pub(super) mutation_enabled: bool,
    pub(super) decision_lock_summary: ContractRepairApprovalReadModelDecisionLockSummary,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) record: Option<ContractRepairApprovalRecordPreview>,
    #[serde(default)]
    pub(super) blocked_reasons: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct ContractRepairApprovalReadModelDecisionLockSummary {
    pub(super) status: String,
    pub(super) target_action: String,
    pub(super) response_status: String,
    pub(super) route_status: String,
    pub(super) target_response_status: String,
    pub(super) target_route_status: String,
    pub(super) expected_http_status: u16,
    pub(super) persisted_record_count: usize,
    pub(super) preview_record_count: usize,
    pub(super) persistence_enabled: bool,
    pub(super) mutation_enabled: bool,
    pub(super) final_execution_locked: bool,
    pub(super) primary_blocked_reason: String,
    pub(super) blocked_reason_count: usize,
    pub(super) would_execute_decision: bool,
    pub(super) would_mutate_contract: bool,
    pub(super) would_return_http_ok: bool,
    pub(super) would_touch_disk: bool,
    #[serde(default)]
    pub(super) inherited_blocked_reasons: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct ActivateRuntimeParameterMutationRequest {
    #[serde(default)]
    pub(super) activation_boundary: Option<RuntimeParameterMutationBoundary>,
    #[serde(default)]
    pub(super) actor: Option<ActorIdentity>,
    #[serde(default)]
    pub(super) capability_context: Option<FrontendCapabilityContext>,
    #[serde(default)]
    pub(super) safe_window_context: Option<RuntimeParameterMutationSafeWindowSnapshot>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct RollbackRuntimeParameterMutationRequest {
    #[serde(default)]
    pub(super) target_parameter_version: Option<String>,
    #[serde(default)]
    pub(super) activation_boundary: Option<RuntimeParameterMutationBoundary>,
    #[serde(default)]
    pub(super) actor: Option<ActorIdentity>,
    #[serde(default)]
    pub(super) capability_context: Option<FrontendCapabilityContext>,
    #[serde(default)]
    pub(super) safe_window_context: Option<RuntimeParameterMutationSafeWindowSnapshot>,
    #[serde(default)]
    pub(super) reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum RuntimeReportLifecycleStatus {
    Requested,
    Generating,
    Ready,
    Failed,
    Expired,
    SourceChanged,
}

impl RuntimeReportLifecycleStatus {
    #[allow(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            Self::Requested => "报告已请求生成",
            Self::Generating => "报告生成中",
            Self::Ready => "报告就绪，可以查看",
            Self::Failed => "报告生成失败",
            Self::Expired => "报告已过期",
            Self::SourceChanged => "源数据已变更，报告可能过时",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeReportSourceSequenceRange {
    pub(super) from: u64,
    pub(super) to: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeReportArtifactMetadata {
    pub(super) kind: String,
    pub(super) artifact_id: String,
    pub(super) file_name: String,
    pub(super) content_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeEvidenceReportSection {
    pub(super) section_id: String,
    pub(super) title: String,
    pub(super) summary: String,
    pub(super) evidence_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeReportFailureMetadata {
    pub(super) reason_code: String,
    pub(super) message: String,
    pub(super) retry_eligible: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeReportLoadingStrategy {
    pub(super) primary_source: String,
    pub(super) source_event_count: usize,
    pub(super) retained_event_count: usize,
    pub(super) requires_detail_window: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeEvidenceReportArtifact {
    pub(super) schema_version: String,
    pub(super) report_id: String,
    pub(super) source_kind: RuntimeEvidenceSourceKind,
    pub(super) source_id: String,
    pub(super) graph_id: String,
    pub(super) status: RuntimeReportLifecycleStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) source_sequence_range: Option<RuntimeReportSourceSequenceRange>,
    pub(super) source_event_count: usize,
    pub(super) retained_event_count: usize,
    pub(super) mutation_lifecycle_event_count: usize,
    pub(super) governance: RuntimeTimelineGovernanceIdentity,
    pub(super) generation_policy: String,
    pub(super) evidence_digest: String,
    pub(super) loading_strategy: RuntimeReportLoadingStrategy,
    #[serde(default)]
    pub(super) sections: Vec<RuntimeEvidenceReportSection>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct RuntimeEvidenceReportRecord {
    pub(super) report_id: String,
    pub(super) source_kind: RuntimeEvidenceSourceKind,
    pub(super) source_id: String,
    pub(super) graph_id: String,
    pub(super) status: RuntimeReportLifecycleStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) source_sequence_range: Option<RuntimeReportSourceSequenceRange>,
    pub(super) source_event_count: usize,
    pub(super) retained_event_count: usize,
    pub(super) mutation_lifecycle_event_count: usize,
    pub(super) governance: RuntimeTimelineGovernanceIdentity,
    pub(super) generation_policy: String,
    #[serde(default)]
    pub(super) artifacts: Vec<RuntimeReportArtifactMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) failure_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) failure: Option<RuntimeReportFailureMetadata>,
    pub(super) created_at_ms: u64,
    pub(super) updated_at_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct CreateRuntimeReportRequest {
    pub(super) source_kind: RuntimeEvidenceSourceKind,
    pub(super) source_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) generation_policy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct RuntimeReplayResponse {
    pub(super) kind: RuntimeReplayRecordKind,
    pub(super) record_id: String,
    pub(super) graph_id: String,
    pub(super) source_event_count: usize,
    pub(super) total_events: usize,
    pub(super) cursor: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) sequence_cursor: Option<u64>,
    pub(super) limit: usize,
    pub(super) window_end: usize,
    pub(super) fill_event_count: usize,
    pub(super) account: AccountSummary,
    #[serde(default)]
    pub(super) filters: RuntimeReplayFilters,
    #[serde(default)]
    pub(super) checkpoints: Vec<RuntimeReplayCheckpoint>,
    #[serde(default)]
    pub(super) events: Vec<RuntimeReplayEventItem>,
    #[serde(default)]
    pub(super) timeline: Vec<RuntimeTimelineItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) previous_cursor: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) next_cursor: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) previous_sequence_cursor: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) next_sequence_cursor: Option<u64>,
}

// ── Hot‑swap API types ──

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct HotSwapModuleTargetDto {
    pub(super) module_key: String,
    pub(super) candidate_config: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct SubmitHotSwapRequest {
    pub(super) module_targets: Vec<HotSwapModuleTargetDto>,
    pub(super) reason: String,
    pub(super) deployment_revision: String,
    #[serde(default = "default_safe_window_timeout_ms")]
    pub(super) safe_window_timeout_ms: u64,
    #[serde(default = "default_observation_window_ms")]
    pub(super) observation_window_ms: u64,
    #[serde(default = "default_shadow_replay_window_ms")]
    pub(super) shadow_replay_window_ms: u64,
}

fn default_safe_window_timeout_ms() -> u64 {
    30_000
}
fn default_observation_window_ms() -> u64 {
    60_000
}
fn default_shadow_replay_window_ms() -> u64 {
    120_000
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct HotSwapResponse {
    pub(super) hotswap_id: String,
    pub(super) success: bool,
    pub(super) new_deployment_revision: Option<String>,
    pub(super) rollback_reason: Option<String>,
    pub(super) final_step: String,
    pub(super) elapsed_ms: u64,
    pub(super) event_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct HotSwapStatusResponse {
    pub(super) hotswap_id: String,
    pub(super) status: String,
    pub(super) step: String,
    pub(super) started_at_ms: u64,
    pub(super) events: Vec<FrontendRuntimeEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct HotSwapRecord {
    pub(super) hotswap_id: String,
    pub(super) status: String,
    pub(super) step: String,
    pub(super) request: SubmitHotSwapRequest,
    pub(super) started_at_ms: u64,
    pub(super) completed_at_ms: Option<u64>,
    pub(super) success: Option<bool>,
    pub(super) rollback_reason: Option<String>,
    #[serde(default)]
    pub(super) events: Vec<FrontendRuntimeEvent>,
}

// ── Block 5: 审批流引擎类型 ──

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum RuntimeApprovalLevel {
    L1SingleReviewer,
    L2DualReviewer,
    L3RiskOwnerReview,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum RuntimeApprovalReviewState {
    Pending,
    UnderReview,
    Approved,
    Rejected,
    Expired,
    // v1.2.1: 以下变体为 V2 调度-激活-观察生命周期预留，当前未使用
    Scheduled,
    Activated,
    Observing,
    Completed,
    RolledBack,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeRollbackPlan {
    pub(super) method: String,
    pub(super) target_generation: u64,
    pub(super) estimated_recovery_ms: u64,
}

impl Default for RuntimeRollbackPlan {
    fn default() -> Self {
        Self {
            method: "generation_rollback".to_string(),
            target_generation: 0,
            estimated_recovery_ms: 5000,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RuntimeApprovalLifecycleEntry {
    pub(super) review_state: RuntimeApprovalReviewState,
    pub(super) event_id: String,
    pub(super) sequence_no: u64,
    pub(super) occurred_at_ms: u64,
    pub(super) reason_code: String,
    pub(super) message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) actor_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct RuntimeApprovalSourceRunnerEvidence {
    #[serde(default)]
    pub(super) source_kind: String,
    #[serde(default)]
    pub(super) source_id: String,
    pub(super) status: String,
    pub(super) execution_status: String,
    pub(super) digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) artifact_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) artifact_file_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) artifact_digest: Option<String>,
    #[serde(default)]
    pub(super) artifact_ref_verified: bool,
    #[serde(default)]
    pub(super) proposal_evidence_hash_matches: bool,
    pub(super) event_count: usize,
    pub(super) risk_decision_count: usize,
    pub(super) matches_original: bool,
    #[serde(default)]
    pub(super) missing_inputs: Vec<String>,
    pub(super) verified: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct RuntimeApprovalRecord {
    pub(super) approval_id: String,
    pub(super) proposal_id: String,
    pub(super) approval_level: RuntimeApprovalLevel,
    pub(super) review_state: RuntimeApprovalReviewState,
    #[serde(default)]
    pub(super) chain_stage_impact: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) sandbox_report_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) source_runner_evidence: Option<RuntimeApprovalSourceRunnerEvidence>,
    #[serde(default)]
    pub(super) rollback_plan: RuntimeRollbackPlan,
    pub(super) created_at_ms: u64,
    pub(super) expires_at_ms: u64,
    pub(super) reviewers_required: u8,
    #[serde(default)]
    pub(super) reviewers_assigned: Vec<String>,
    #[serde(default)]
    pub(super) reviewers_approved: Vec<String>,
    #[serde(default)]
    pub(super) reviewers_rejected: Vec<String>,
    #[serde(default)]
    pub(super) lifecycle: Vec<RuntimeApprovalLifecycleEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct ApprovalActionRequest {
    pub(super) actor_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) comment: Option<String>,
}

// ── Block 5: 沙箱验证服务类型 ──

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum SandboxVerdict {
    CandidateOutperformsBaseline,
    CandidateComparable,
    CandidateUnderperforms,
    ReplayFidelityPartial,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct ReplayWindow {
    pub(super) from_ts: String,
    pub(super) to_ts: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct SandboxMetrics {
    pub(super) total_return_ratio: f64,
    pub(super) max_drawdown_ratio: f64,
    pub(super) sharpe_ratio: f64,
    pub(super) win_rate: f64,
    pub(super) avg_hold_hours: f64,
    pub(super) turnover_ratio: f64,
    pub(super) profit_factor: f64,
    pub(super) calmar_ratio: f64,
}

impl Default for SandboxMetrics {
    fn default() -> Self {
        Self {
            total_return_ratio: 0.0,
            max_drawdown_ratio: 0.0,
            sharpe_ratio: 0.0,
            win_rate: 0.0,
            avg_hold_hours: 0.0,
            turnover_ratio: 0.0,
            profit_factor: 0.0,
            calmar_ratio: 0.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct SandboxMetricsDiff {
    pub(super) total_return_ratio: String,
    pub(super) max_drawdown_ratio: String,
    pub(super) sharpe_ratio: String,
    pub(super) win_rate: String,
    pub(super) avg_hold_hours: String,
    pub(super) turnover_ratio: String,
    pub(super) profit_factor: String,
    pub(super) calmar_ratio: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum SandboxReplayDiffGateStatus {
    Passed,
    Blocked,
    Missing,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct SandboxReplayDiffGate {
    pub(super) status: SandboxReplayDiffGateStatus,
    pub(super) source: String,
    pub(super) reason_code: String,
    #[serde(default)]
    pub(super) evidence_anchor_ids: Vec<String>,
    #[serde(default)]
    pub(super) source_evidence_hash_bound: bool,
    #[serde(default)]
    pub(super) proposal_evidence_hash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct SandboxVerificationReport {
    pub(super) proposal_id: String,
    pub(super) sandbox_run_id: String,
    pub(super) replay_window: ReplayWindow,
    pub(super) baseline_metrics: SandboxMetrics,
    pub(super) candidate_metrics: SandboxMetrics,
    pub(super) diffs: SandboxMetricsDiff,
    pub(super) verdict: SandboxVerdict,
    #[serde(default)]
    pub(super) warnings: Vec<String>,
    pub(super) replay_fidelity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) productization_replay_diff: Option<SandboxReplayDiffGate>,
    pub(super) generated_at_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct RequestSandboxVerificationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) backtest_id: Option<String>,
    pub(super) proposal_id: String,
}

// ── Block 5: 告警规则引擎类型 ──

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum AlertSeverity {
    P1,
    P2,
    P3,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum AlertFiringState {
    Firing,
    Acknowledged,
    Resolved,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct AlertRule {
    pub(super) rule_name: String,
    pub(super) description: String,
    pub(super) trigger_condition: String,
    pub(super) severity: AlertSeverity,
    pub(super) action: String,
    pub(super) enabled: bool,
    /// v3.5.0 §9.3: 自动恢复条件 (满足后触发 AlertResolved)
    #[serde(default)]
    pub(super) resolve_condition: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct AlertFiring {
    pub(super) firing_id: String,
    pub(super) rule_name: String,
    pub(super) severity: AlertSeverity,
    pub(super) state: AlertFiringState,
    pub(super) fired_at_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) acknowledged_at_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) resolved_at_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) acknowledged_by: Option<String>,
    pub(super) detail: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct AlertListResponse {
    pub(super) firings: Vec<AlertFiring>,
    pub(super) rules: Vec<AlertRule>,
}

// ── Block 5: 签名快照类型 ──

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct EventSliceBounds {
    pub(super) from_event_id: String,
    pub(super) to_event_id: String,
    pub(super) from_sequence: u64,
    pub(super) to_sequence: u64,
    pub(super) event_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct DeploymentSignatureSnapshot {
    pub(super) snapshot_id: String,
    pub(super) deployment_revision: String,
    pub(super) capability_hash: String,
    pub(super) strategy_version: String,
    pub(super) parameter_version: String,
    pub(super) core_ir_digest: String,
    pub(super) event_slice_bounds: EventSliceBounds,
    pub(super) created_at_ms: u64,
    pub(super) signature: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct RestoreSnapshotRequest {
    pub(super) actor_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) reason: Option<String>,
}

// ── Block 5: 运营报表类型 ──

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct OpsDailyReportSummary {
    pub(super) total_runs: usize,
    pub(super) active_runs: usize,
    pub(super) total_events_24h: u64,
    pub(super) avg_event_rate_per_sec: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct OpsDataHealth {
    pub(super) sources_healthy: usize,
    pub(super) sources_degraded: usize,
    pub(super) p95_freshness_ms: u64,
    pub(super) gap_events_24h: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct OpsRuntimeHealth {
    pub(super) total_executions: u64,
    pub(super) execution_success_rate: f64,
    pub(super) risk_reject_rate: f64,
    pub(super) avg_decision_latency_p95_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct OpsAlertsSummary {
    pub(super) total_fired: usize,
    pub(super) p1_fired: usize,
    pub(super) p2_fired: usize,
    pub(super) p3_fired: usize,
    pub(super) acknowledged: usize,
    pub(super) resolved: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct OpsDegradationEvent {
    pub(super) timestamp: String,
    pub(super) trigger: String,
    pub(super) action: String,
    pub(super) recovery_timestamp: String,
    pub(super) duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct OpsStorage {
    pub(super) hot_layer_usage_ratio: f64,
    pub(super) warm_layer_total_mb: u64,
    pub(super) cold_layer_total_mb: u64,
    pub(super) disk_watermark_ratio: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct OpsDailyReport {
    pub(super) report_type: String,
    pub(super) report_date: String,
    pub(super) generated_at: String,
    pub(super) summary: OpsDailyReportSummary,
    pub(super) data_health: OpsDataHealth,
    pub(super) runtime_health: OpsRuntimeHealth,
    pub(super) alerts_24h: OpsAlertsSummary,
    #[serde(default)]
    pub(super) degradation_events: Vec<OpsDegradationEvent>,
    pub(super) storage: OpsStorage,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct AuditWeeklyReport {
    pub(super) report_type: String,
    pub(super) week_start: String,
    pub(super) week_end: String,
    pub(super) generated_at: String,
    pub(super) total_approvals: usize,
    pub(super) approved_count: usize,
    pub(super) rejected_count: usize,
    pub(super) expired_count: usize,
    pub(super) ai_proposals_total: usize,
    pub(super) ai_proposals_approved: usize,
    pub(super) parameter_changes: usize,
    pub(super) rollback_events: usize,
    pub(super) hotswap_events: usize,
    #[serde(default)]
    pub(super) notable_incidents: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct ResearchMonthlyReport {
    pub(super) report_type: String,
    pub(super) month: String,
    pub(super) generated_at: String,
    pub(super) strategy_performance: Vec<StrategyPerformanceSummary>,
    pub(super) ai_proposal_effectiveness: AiProposalEffectivenessSummary,
    pub(super) capacity_trend: CapacityTrend,
    pub(super) cost_analysis: CostAnalysisSummary,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct StrategyPerformanceSummary {
    pub(super) strategy_id: String,
    pub(super) total_return: f64,
    pub(super) max_drawdown: f64,
    pub(super) sharpe_ratio: f64,
    pub(super) win_rate: f64,
    pub(super) total_trades: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct AiProposalEffectivenessSummary {
    pub(super) total_proposals: usize,
    pub(super) approved: usize,
    pub(super) improved_performance: usize,
    pub(super) no_significant_change: usize,
    pub(super) degraded_performance: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct CapacityTrend {
    pub(super) max_concurrent_runs: usize,
    pub(super) avg_runs_per_day: f64,
    pub(super) peak_events_per_second: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct CostAnalysisSummary {
    pub(super) total_storage_mb: u64,
    pub(super) hot_storage_mb: u64,
    pub(super) warm_storage_mb: u64,
    pub(super) cold_storage_mb: u64,
    pub(super) estimated_monthly_cost_usd: f64,
}

// ── Block 5: Runbook 类型 ──

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct RunbookDiagnosticStep {
    pub(super) step_number: u8,
    pub(super) description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) api_call: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) expected: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct RunbookRecoveryStep {
    pub(super) step_number: u8,
    pub(super) condition: String,
    pub(super) action: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct RunbookScenario {
    pub(super) scenario_id: String,
    pub(super) name: String,
    pub(super) symptoms: Vec<String>,
    pub(super) severity: AlertSeverity,
    pub(super) diagnostic_steps: Vec<RunbookDiagnosticStep>,
    pub(super) recovery_steps: Vec<RunbookRecoveryStep>,
    pub(super) verification: String,
}

// ── Block 5: 混沌实验类型 ──

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum ChaosExperimentType {
    DataLatencyInjection,
    EventLossInjection,
    DiskPressureInjection,
    ClockSkewInjection,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct ChaosInjectionSpec {
    pub(super) target: String,
    pub(super) parameter: String,
    pub(super) value: f64,
    pub(super) duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct ChaosSteadyStateMetrics {
    pub(super) data_freshness_p95_ms: f64,
    pub(super) execution_planned_rate_per_min: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct ChaosExperimentReport {
    pub(super) experiment_id: String,
    pub(super) experiment_type: ChaosExperimentType,
    pub(super) executed_at: String,
    pub(super) injection: ChaosInjectionSpec,
    pub(super) steady_state_metrics_before: ChaosSteadyStateMetrics,
    pub(super) steady_state_metrics_during: ChaosSteadyStateMetrics,
    pub(super) steady_state_metrics_after: ChaosSteadyStateMetrics,
    #[serde(default)]
    pub(super) alerts_triggered: Vec<String>,
    #[serde(default)]
    pub(super) degradation_actions: Vec<String>,
    pub(super) recovery_duration_ms: u64,
    pub(super) passed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct CreateChaosExperimentRequest {
    pub(super) experiment_type: ChaosExperimentType,
    pub(super) injection: ChaosInjectionSpec,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) notes: Option<String>,
}

// ── v1.0.5 分页 ──

// v2.1.x: 自定义 Deserialize 以返回更具体的参数范围错误 (S0-3 修复)
#[derive(Debug, Default)]
pub struct PaginationQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl<'de> serde::Deserialize<'de> for PaginationQuery {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let raw = serde_json::Value::deserialize(deserializer)?;
        let mut query = PaginationQuery::default();

        if let Some(val) = raw.get("limit") {
            if val.is_null() { /* skip null */
            } else if let Some(n) = val.as_u64() {
                query.limit = Some(n.min(100) as u32);
            } else if let Some(n) = val.as_i64() {
                if n < 0 {
                    return Err(Error::custom("分页参数 limit 必须为非负整数 (0-100)"));
                }
                query.limit = Some((n as u64).min(100) as u32);
            } else {
                return Err(Error::custom("分页参数 limit 必须为非负整数 (0-100)"));
            }
        }

        if let Some(val) = raw.get("offset") {
            if val.is_null() { /* skip null */
            } else if let Some(n) = val.as_u64() {
                query.offset = Some(n.min(10_000) as u32);
            } else if let Some(n) = val.as_i64() {
                if n < 0 {
                    return Err(Error::custom("分页参数 offset 必须为非负整数 (0-10000)"));
                }
                query.offset = Some((n as u64).min(10_000) as u32);
            } else {
                return Err(Error::custom("分页参数 offset 必须为非负整数 (0-10000)"));
            }
        }

        Ok(query)
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: usize,
    pub limit: u32,
    pub offset: u32,
}

pub fn paginate<T: Serialize>(mut items: Vec<T>, query: PaginationQuery) -> PaginatedResponse<T> {
    let total = items.len();
    let limit = query.limit.unwrap_or(20).min(100) as usize;
    let offset = query.offset.unwrap_or(0).min(10_000) as usize; // v1.2.3: 分页上限防DoS
    if offset > 0 {
        items = items.into_iter().skip(offset).collect();
    }
    items.truncate(limit);
    PaginatedResponse {
        data: items,
        total,
        limit: limit as u32,
        offset: offset as u32,
    }
}

// ── v1.0.6 统一 API 错误响应 ──

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct ApiErrorResponse {
    pub error: &'static str,
    pub message: String,
    /// v2.3.0: 语言中立错误码, 前端可映射本地化文本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<&'static str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<ApiErrorDetail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_artifacts: Option<ApiPartialArtifacts>,
}
