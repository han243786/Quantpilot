use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct GraphListEntry {
    pub(super) graph_id: String,
    pub(super) name: String,
    pub(super) updated_at: u64,
    pub(super) path: String,
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
    pub(super) has_changes: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct RevealGraphResponse {
    pub(super) graph_id: String,
    pub(super) path: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct FrontendRunRequest {
    #[serde(default)]
    pub(super) actor: Option<ActorIdentity>,
    pub(super) runtime_config: FrontendRuntimeConfig,
    #[serde(default)]
    pub(super) runtime_targets: CompileRuntimeTargets,
    #[serde(default)]
    pub(super) backtest_options: FrontendBacktestOptions,
}

impl FrontendRunRequest {
    pub(super) fn backtest_replay_source(&self) -> FrontendBacktestReplaySource {
        self.backtest_options
            .replay_source
            .unwrap_or(FrontendBacktestReplaySource::HistoricalReplay)
    }
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
pub(super) struct FrontendBacktestOptions {
    pub(super) replay_source: Option<FrontendBacktestReplaySource>,
    pub(super) execution_assumptions: Option<FrontendExecutionAssumptionOverrides>,
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
pub(super) struct FrontendExperimentRequest {
    #[serde(default)]
    pub(super) experiment_name: Option<String>,
    #[serde(default)]
    pub(super) actor: Option<ActorIdentity>,
    pub(super) runtime_config: FrontendRuntimeConfig,
    #[serde(default)]
    pub(super) runtime_targets: CompileRuntimeTargets,
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
pub(super) struct RunStartResponse {
    pub(super) run_id: String,
    pub(super) graph_id: String,
    pub(super) compile_id: String,
    pub(super) event_count: usize,
    pub(super) status: &'static str,
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
pub(super) struct RunRecord {
    pub(super) run_id: String,
    pub(super) graph_id: String,
    pub(super) compile_id: String,
    pub(super) created_at_ms: u64,
    pub(super) events: Vec<FrontendRuntimeEvent>,
    pub(super) account: AccountSummary,
    pub(super) session: SessionOutput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) actor: Option<ActorIdentity>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) actor: Option<ActorIdentity>,
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
pub(super) struct ParseGraphQuantScriptRequest {
    pub(super) source: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct CompileRuntimeRequest {
    pub(super) runtime_config: FrontendRuntimeConfig,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Serialize)]
pub(super) struct CompileCounts {
    pub(super) data_sources: usize,
    pub(super) intent_generators: usize,
    pub(super) agents: usize,
    pub(super) risk_controls: usize,
    pub(super) executions: usize,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(super) enum CompileDiagnosticSeverity {
    Error,
    Warning,
    Info,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum CompileDiagnosticTargetScope {
    Graph,
    Node,
    Edge,
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

#[derive(Debug, Serialize)]
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
    pub(super) runtime_diagnostics: RuntimeDiagnosticsPayload,
    pub(super) session: SessionOutput,
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
    pub(super) backtest_artifacts: BacktestArtifactViews,
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
pub(super) struct ExperimentRecord {
    pub(super) experiment_id: String,
    pub(super) graph_id: String,
    pub(super) compile_id: String,
    pub(super) created_at_ms: u64,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct RuntimeReplayCheckpoint {
    pub(super) cursor: usize,
    pub(super) label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) event_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) event_time_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct RuntimeReplayResponse {
    pub(super) kind: RuntimeReplayRecordKind,
    pub(super) record_id: String,
    pub(super) graph_id: String,
    pub(super) total_events: usize,
    pub(super) cursor: usize,
    pub(super) limit: usize,
    pub(super) window_end: usize,
    pub(super) fill_event_count: usize,
    pub(super) account: AccountSummary,
    #[serde(default)]
    pub(super) checkpoints: Vec<RuntimeReplayCheckpoint>,
    #[serde(default)]
    pub(super) events: Vec<RuntimeReplayEventItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) previous_cursor: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) next_cursor: Option<usize>,
}
