use super::*;
use crate::backend::strategy_config::artifact::{
    ConfigDomainId, ConfigDomainLifecycle, ConfigDomainReadiness, ConfigDomainStatus,
    ConfigSourceRef, EvidenceAnchor, ProposalBinding, ProposalBindingInput, RuntimeBoundarySummary,
    StrategyConfigArtifact, StrategyConfigCapabilitySummary, StrategyConfigFinding,
    StrategyConfigSourceSummary,
};
pub(super) use crate::backend::strategy_config::artifact::{
    EvidenceAnchorInput, StrategyConfigArtifactRequest,
};

const STRATEGY_CONFIG_ARTIFACT_SCHEMA: &str = "quantpilot/v4-strategy-config-artifact/v1";
const STRATEGY_CONFIG_PREFLIGHT_SCHEMA: &str = "quantpilot/v4-strategy-config-preflight/v1";
const STRATEGY_CONFIG_DIFF_SCHEMA: &str = "quantpilot/v4-strategy-config-diff/v1";
const STRATEGY_CONFIG_EVIDENCE_DIFF_SCHEMA: &str = "quantpilot/v4-strategy-config-evidence-diff/v1";

pub(super) fn register_strategy_config_preflight_route(
    router: Router<AppState>,
) -> Router<AppState> {
    router.route(
        "/api/v1/strategy-config/preflight",
        post(preflight_strategy_config),
    )
}

pub(super) fn register_strategy_config_diff_route(router: Router<AppState>) -> Router<AppState> {
    router.route("/api/v1/strategy-config/diff", post(diff_strategy_config))
}

async fn preflight_strategy_config(
    Json(request): Json<StrategyConfigArtifactRequest>,
) -> Result<Json<StrategyConfigPreflightReport>, (StatusCode, String)> {
    let artifact = build_strategy_config_artifact(request, current_time_ms())?;
    Ok(Json(build_preflight_report(artifact)))
}

pub(super) fn build_strategy_config_preflight_value(
    request: StrategyConfigArtifactRequest,
) -> Result<Value, (StatusCode, String)> {
    let artifact = build_strategy_config_artifact(request, current_time_ms())?;
    serde_json::to_value(build_preflight_report(artifact)).map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("strategy config preflight serialization failed: {}", error),
        )
    })
}

async fn diff_strategy_config(
    Json(request): Json<StrategyConfigDiffRequest>,
) -> Result<Json<StrategyConfigDiffReport>, (StatusCode, String)> {
    Ok(Json(build_diff_report(request.left, request.right)))
}

pub(super) fn build_strategy_config_version_diff(
    graph_id: &str,
    left: &GraphVersionEntry,
    left_graph: &Value,
    left_qs_source: Option<String>,
    right: &GraphVersionEntry,
    right_graph: &Value,
    right_qs_source: Option<String>,
) -> Result<StrategyConfigDiffReport, (StatusCode, String)> {
    let left_artifact = build_strategy_config_artifact(
        version_artifact_request(graph_id, left, left_graph, left_qs_source),
        left.updated_at,
    )?;
    let right_artifact = build_strategy_config_artifact(
        version_artifact_request(graph_id, right, right_graph, right_qs_source),
        right.updated_at,
    )?;
    Ok(build_diff_report(left_artifact, right_artifact))
}

pub(super) async fn build_strategy_config_evidence_diff_for_backtests(
    state: &AppState,
    user_id: &auth::UserId,
    graph_id: &str,
    left_backtest_id: Option<&str>,
    right_backtest_id: Option<&str>,
) -> StrategyConfigEvidenceDiffReport {
    let mut diagnostics = Vec::new();
    let left_record =
        load_bound_backtest_for_evidence(state, user_id, graph_id, left_backtest_id, "left")
            .await
            .unwrap_or_else(|finding| {
                diagnostics.push(finding);
                None
            });
    let right_record =
        load_bound_backtest_for_evidence(state, user_id, graph_id, right_backtest_id, "right")
            .await
            .unwrap_or_else(|finding| {
                diagnostics.push(finding);
                None
            });
    build_strategy_config_evidence_diff(
        left_backtest_id.map(str::to_string),
        right_backtest_id.map(str::to_string),
        left_record.as_ref(),
        right_record.as_ref(),
        diagnostics,
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct StrategyConfigPreflightReport {
    pub(super) schema_version: String,
    pub(super) artifact: StrategyConfigArtifact,
    pub(super) decision: PreflightDecision,
    pub(super) can_compile: bool,
    pub(super) can_paper_simulated: bool,
    pub(super) can_backtest: bool,
    pub(super) can_paper_actual_demo: bool,
    pub(super) can_live_execution: bool,
    pub(super) allowed_actions: Vec<String>,
    pub(super) blocked_actions: Vec<PreflightBlockedAction>,
    pub(super) findings: Vec<StrategyConfigFinding>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum PreflightDecision {
    Ready,
    Restricted,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct PreflightBlockedAction {
    pub(super) action: String,
    pub(super) reason: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct StrategyConfigDiffRequest {
    pub(super) left: StrategyConfigArtifact,
    pub(super) right: StrategyConfigArtifact,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct StrategyConfigDiffReport {
    pub(super) schema_version: String,
    pub(super) left_artifact_id: String,
    pub(super) right_artifact_id: String,
    pub(super) source_digest_changes: Vec<StrategyConfigDigestChange>,
    pub(super) domain_changes: Vec<StrategyConfigDomainChange>,
    pub(super) runtime_boundary_changed: bool,
    pub(super) changed: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum StrategyConfigEvidenceDiffStatus {
    Same,
    Different,
    Missing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct StrategyConfigEvidenceDiffReport {
    pub(super) schema_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) left_backtest_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) right_backtest_id: Option<String>,
    pub(super) status: StrategyConfigEvidenceDiffStatus,
    pub(super) changed: bool,
    #[serde(default)]
    pub(super) diagnostics: Vec<StrategyConfigFinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) machine_trajectory: Option<StrategyConfigMachineTrajectoryEvidenceDiff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) risk_plane: Option<StrategyConfigRiskPlaneEvidenceDiff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) execution_capability: Option<StrategyConfigExecutionCapabilityEvidenceDiff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) metrics: Option<StrategyConfigEvidenceMetricsDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct StrategyConfigMachineTrajectoryEvidenceDiff {
    pub(super) status: StrategyConfigEvidenceDiffStatus,
    pub(super) left_point_count: usize,
    pub(super) right_point_count: usize,
    #[serde(default)]
    pub(super) left_visited_states: Vec<String>,
    #[serde(default)]
    pub(super) right_visited_states: Vec<String>,
    #[serde(default)]
    pub(super) transition_hit_changes: Vec<StrategyConfigEvidenceCountChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) left_terminal_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) right_terminal_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) first_divergence: Option<StrategyConfigEvidenceFirstDivergence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct StrategyConfigRiskPlaneEvidenceDiff {
    pub(super) status: StrategyConfigEvidenceDiffStatus,
    pub(super) left_decision_count: usize,
    pub(super) right_decision_count: usize,
    pub(super) left_approved_count: usize,
    pub(super) right_approved_count: usize,
    pub(super) left_rejected_count: usize,
    pub(super) right_rejected_count: usize,
    #[serde(default)]
    pub(super) action_count_changes: Vec<StrategyConfigEvidenceCountChange>,
    #[serde(default)]
    pub(super) reason_count_changes: Vec<StrategyConfigEvidenceCountChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) first_divergence: Option<StrategyConfigEvidenceFirstDivergence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct StrategyConfigExecutionCapabilityEvidenceDiff {
    pub(super) status: StrategyConfigEvidenceDiffStatus,
    pub(super) left_source_count: usize,
    pub(super) right_source_count: usize,
    pub(super) left_accepted_count: usize,
    pub(super) right_accepted_count: usize,
    pub(super) left_rejected_count: usize,
    pub(super) right_rejected_count: usize,
    #[serde(default)]
    pub(super) runtime_mode_changes: Vec<StrategyConfigEvidenceCountChange>,
    #[serde(default)]
    pub(super) capability_kind_changes: Vec<StrategyConfigEvidenceCountChange>,
    #[serde(default)]
    pub(super) capability_source_changes: Vec<StrategyConfigEvidenceCountChange>,
    #[serde(default)]
    pub(super) status_changes: Vec<StrategyConfigEvidenceCountChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) first_divergence: Option<StrategyConfigEvidenceFirstDivergence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct StrategyConfigEvidenceMetricsDiff {
    pub(super) status: StrategyConfigEvidenceDiffStatus,
    #[serde(default)]
    pub(super) fields: Vec<StrategyConfigEvidenceFieldDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct StrategyConfigEvidenceCountChange {
    pub(super) key: String,
    pub(super) left_count: usize,
    pub(super) right_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct StrategyConfigEvidenceFieldDiff {
    pub(super) key: String,
    pub(super) status: StrategyConfigEvidenceDiffStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) left_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) right_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct StrategyConfigEvidenceFirstDivergence {
    pub(super) index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) left: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) right: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct StrategyConfigDigestChange {
    pub(super) field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) after: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct StrategyConfigDomainChange {
    pub(super) domain_id: ConfigDomainId,
    pub(super) lifecycle_changed: bool,
    pub(super) readiness_changed: bool,
    pub(super) source_refs_changed: bool,
    pub(super) findings_changed: bool,
}

pub(super) fn build_strategy_config_artifact(
    request: StrategyConfigArtifactRequest,
    now_ms: u64,
) -> Result<StrategyConfigArtifact, (StatusCode, String)> {
    let strategy_id =
        non_empty(request.strategy_id.clone()).unwrap_or_else(|| "local-strategy".to_string());
    let strategy_version =
        non_empty(request.strategy_version.clone()).unwrap_or_else(|| "local-draft".to_string());
    let source = build_source_summary(&request);
    let capability = build_capability_summary(&request);
    let runtime_boundary = build_runtime_boundary(&request);
    let evidence_anchors = normalize_evidence_anchors(request.evidence_anchors.clone());
    let proposal_bindings = normalize_proposal_bindings(request.proposal_bindings.clone());
    let config_domains = build_config_domains(
        &request,
        &source,
        &capability,
        &runtime_boundary,
        &evidence_anchors,
        &proposal_bindings,
    );
    let artifact_id = format!(
        "strategy_config_{}",
        digest_for_value(&json!({
            "strategy_id": strategy_id,
            "strategy_version": strategy_version,
            "source": source,
            "capability_hash": capability.capability_snapshot_hash,
            "created_at_ms": now_ms,
        }))?
        .trim_start_matches("sha256:")
        .chars()
        .take(16)
        .collect::<String>()
    );

    let mut artifact = StrategyConfigArtifact {
        schema_version: STRATEGY_CONFIG_ARTIFACT_SCHEMA.to_string(),
        artifact_id,
        strategy_id,
        strategy_version,
        created_at_ms: now_ms,
        source,
        capability,
        config_domains,
        runtime_boundary,
        evidence_anchors,
        proposal_bindings,
        artifact_digest: String::new(),
    };
    artifact.artifact_digest = digest_for_value(&artifact_digest_input(&artifact))?;
    Ok(artifact)
}

fn version_artifact_request(
    graph_id: &str,
    version: &GraphVersionEntry,
    graph: &Value,
    qs_source: Option<String>,
) -> StrategyConfigArtifactRequest {
    StrategyConfigArtifactRequest {
        strategy_id: Some(graph_id.to_string()),
        strategy_version: Some(
            version
                .version_label
                .clone()
                .unwrap_or_else(|| version.version_id.clone()),
        ),
        source_mode: Some("graph_version".to_string()),
        graph_json: Some(graph.clone()),
        runtime_config: graph.get("runtime_config").cloned(),
        qs_source: qs_source.and_then(|source| non_empty(Some(source))),
        core_ir: graph
            .get("metadata")
            .and_then(|metadata| metadata.get("artifacts"))
            .and_then(|artifacts| artifacts.get("core_ir"))
            .cloned(),
        v4_graph: graph
            .get("metadata")
            .and_then(|metadata| metadata.get("artifacts"))
            .and_then(|artifacts| artifacts.get("v4_graph"))
            .cloned(),
        capability_snapshot_hash: Some(build_capability_response().schema_hash),
        capability_source: Some("backend_current".to_string()),
        runtime_mode: Some("PaperSimulated".to_string()),
        evidence_anchors: vec![EvidenceAnchorInput {
            anchor_type: "snapshot".to_string(),
            anchor_id: Some(version.version_id.clone()),
            digest: digest_for_value(graph).ok(),
            summary: version
                .version_label
                .clone()
                .or_else(|| version.save_note.clone()),
        }],
        proposal_bindings: Vec::new(),
        required_execution_capability_sources: vec!["runtime_simulated".to_string()],
    }
}

fn build_source_summary(request: &StrategyConfigArtifactRequest) -> StrategyConfigSourceSummary {
    StrategyConfigSourceSummary {
        source_mode: non_empty(request.source_mode.clone())
            .unwrap_or_else(|| infer_source_mode(request)),
        graph_digest: digest_option_value(&request.graph_json),
        runtime_config_digest: digest_option_value(&request.runtime_config),
        qs_digest: request
            .qs_source
            .as_ref()
            .and_then(|source| digest_for_value(&json!({ "qs_source": source })).ok()),
        core_ir_digest: digest_option_value(&request.core_ir),
        v4_graph_digest: digest_option_value(&request.v4_graph),
    }
}

fn build_capability_summary(
    request: &StrategyConfigArtifactRequest,
) -> StrategyConfigCapabilitySummary {
    let response = build_capability_response();
    let capability_snapshot_hash = non_empty(request.capability_snapshot_hash.clone())
        .unwrap_or_else(|| response.schema_hash.clone());
    let capability_snapshot_status = if capability_snapshot_hash == response.schema_hash {
        "current"
    } else if capability_snapshot_hash == "safe-fallback" {
        "safe_fallback"
    } else {
        "stale"
    }
    .to_string();
    StrategyConfigCapabilitySummary {
        capability_snapshot_hash,
        capability_expected_hash: response.schema_hash,
        capability_snapshot_status,
        capability_source: non_empty(request.capability_source.clone())
            .unwrap_or_else(|| "backend_current".to_string()),
        frontend_module_keys: response
            .frontend
            .supported_module_keys
            .into_iter()
            .map(str::to_string)
            .collect(),
        ui_actions: response
            .ui_actions
            .actions
            .into_iter()
            .map(|entry| entry.key.to_string())
            .collect(),
        workspace_surfaces: response
            .workspace
            .surfaces
            .into_iter()
            .map(|entry| entry.key.to_string())
            .collect(),
    }
}

fn build_runtime_boundary(request: &StrategyConfigArtifactRequest) -> RuntimeBoundarySummary {
    let raw_mode = request
        .runtime_mode
        .as_deref()
        .unwrap_or("PaperSimulated")
        .trim()
        .to_ascii_lowercase();
    let legacy_live_label = raw_mode == "live" || raw_mode == "live.okx";
    let mode_label = if matches!(
        raw_mode.as_str(),
        "paperactual" | "paper_actual" | "okx_demo" | "demo" | "live" | "live.okx"
    ) {
        "PaperActual"
    } else {
        "PaperSimulated"
    }
    .to_string();
    let mut execution_capability_sources =
        if request.required_execution_capability_sources.is_empty() {
            vec![if mode_label == "PaperActual" {
                "provider_native".to_string()
            } else {
                "runtime_simulated".to_string()
            }]
        } else {
            request
                .required_execution_capability_sources
                .iter()
                .filter_map(|source| non_empty(Some(source.clone())))
                .collect()
        };
    execution_capability_sources.sort();
    execution_capability_sources.dedup();

    let mut rejection_reasons = Vec::new();
    if legacy_live_label {
        rejection_reasons.push(
            "legacy_live_label_normalized_to_paper_actual_demo; 用户侧不得用 live 描述 OKX demo"
                .to_string(),
        );
    }
    if execution_capability_sources
        .iter()
        .any(|source| source == "unsupported")
    {
        rejection_reasons.push("required execution capability contains unsupported".to_string());
    }

    RuntimeBoundarySummary {
        mode_label: mode_label.clone(),
        provider_order_submission_attached: mode_label == "PaperActual",
        provider_order_submission_allowed: mode_label == "PaperActual"
            && !execution_capability_sources
                .iter()
                .any(|source| source == "unsupported"),
        live_execution_allowed: false,
        execution_capability_sources,
        rejection_reasons,
    }
}

fn normalize_evidence_anchors(inputs: Vec<EvidenceAnchorInput>) -> Vec<EvidenceAnchor> {
    inputs
        .into_iter()
        .enumerate()
        .filter_map(|(index, input)| {
            let anchor_type = input.anchor_type.trim().to_string();
            if anchor_type.is_empty() {
                return None;
            }
            Some(EvidenceAnchor {
                anchor_id: input
                    .anchor_id
                    .and_then(|value| non_empty(Some(value)))
                    .unwrap_or_else(|| format!("{}_{}", anchor_type, index + 1)),
                anchor_type,
                digest: input.digest.and_then(|value| non_empty(Some(value))),
                summary: input.summary.and_then(|value| non_empty(Some(value))),
            })
        })
        .collect()
}

fn normalize_proposal_bindings(inputs: Vec<ProposalBindingInput>) -> Vec<ProposalBinding> {
    inputs
        .into_iter()
        .filter_map(|input| {
            let proposal_id = input.proposal_id.trim().to_string();
            if proposal_id.is_empty() {
                return None;
            }
            Some(ProposalBinding {
                proposal_id,
                target_domain: input.target_domain,
                before_digest: input.before_digest.and_then(|value| non_empty(Some(value))),
                after_digest: input.after_digest.and_then(|value| non_empty(Some(value))),
                evidence_anchor_ids: input.evidence_anchor_ids,
                sandbox_status: input
                    .sandbox_status
                    .and_then(|value| non_empty(Some(value)))
                    .unwrap_or_else(|| "pending".to_string()),
                approval_status: input
                    .approval_status
                    .and_then(|value| non_empty(Some(value)))
                    .unwrap_or_else(|| "pending".to_string()),
            })
        })
        .collect()
}

fn build_config_domains(
    request: &StrategyConfigArtifactRequest,
    source: &StrategyConfigSourceSummary,
    capability: &StrategyConfigCapabilitySummary,
    runtime_boundary: &RuntimeBoundarySummary,
    evidence_anchors: &[EvidenceAnchor],
    proposal_bindings: &[ProposalBinding],
) -> Vec<ConfigDomainStatus> {
    vec![
        market_domain(request, source),
        observation_domain(request, source),
        state_machine_domain(request, source),
        risk_domain(request, source),
        execution_domain(runtime_boundary, capability),
        evidence_domain(evidence_anchors),
        ai_governance_domain(proposal_bindings),
        snapshot_domain(evidence_anchors),
    ]
}

fn market_domain(
    request: &StrategyConfigArtifactRequest,
    source: &StrategyConfigSourceSummary,
) -> ConfigDomainStatus {
    let ready = request.graph_json.is_some() || request.runtime_config.is_some();
    ConfigDomainStatus {
        domain_id: ConfigDomainId::Market,
        lifecycle: ConfigDomainLifecycle::Implemented,
        readiness: if ready {
            ConfigDomainReadiness::Ready
        } else {
            ConfigDomainReadiness::Incomplete
        },
        source_refs: refs_from_pairs([
            ("graph_json", source.graph_digest.clone()),
            ("runtime_config", source.runtime_config_digest.clone()),
        ]),
        findings: if ready {
            Vec::new()
        } else {
            vec![finding(
                "warning",
                "strategy_config_market_incomplete",
                "市场与数据配置缺少 graph_json 或 runtime_config 证据。",
            )]
        },
        primary_action: Some("compile".to_string()),
    }
}

fn observation_domain(
    request: &StrategyConfigArtifactRequest,
    source: &StrategyConfigSourceSummary,
) -> ConfigDomainStatus {
    let ready = request.qs_source.is_some()
        || request.core_ir.is_some()
        || request.v4_graph.is_some()
        || request.graph_json.is_some();
    ConfigDomainStatus {
        domain_id: ConfigDomainId::Observation,
        lifecycle: ConfigDomainLifecycle::Implemented,
        readiness: if ready {
            ConfigDomainReadiness::Ready
        } else {
            ConfigDomainReadiness::Incomplete
        },
        source_refs: refs_from_pairs([
            ("qs_source", source.qs_digest.clone()),
            ("core_ir", source.core_ir_digest.clone()),
            ("v4_graph", source.v4_graph_digest.clone()),
            ("graph_json", source.graph_digest.clone()),
        ]),
        findings: if ready {
            Vec::new()
        } else {
            vec![finding(
                "warning",
                "strategy_config_observation_incomplete",
                "观察与信号配置缺少 QS、Core IR、v4 graph 或策略图证据。",
            )]
        },
        primary_action: Some("compile".to_string()),
    }
}

fn state_machine_domain(
    request: &StrategyConfigArtifactRequest,
    source: &StrategyConfigSourceSummary,
) -> ConfigDomainStatus {
    let source_has_machine = request
        .qs_source
        .as_deref()
        .map(|source| source.contains("machine"))
        .unwrap_or(false);
    let ready = request.v4_graph.is_some() || source_has_machine;
    let mut findings = Vec::new();
    if !ready {
        findings.push(finding(
            "info",
            "strategy_config_state_machine_documentable",
            "当前可说明 v4 状态机模型，但该 artifact 未携带 v4 graph 或 machine QS 证据。",
        ));
    }
    if let Some(v4_graph) = &request.v4_graph {
        if serde_json::from_value::<qrpc_core_ir::v4::V4MachineGraphContract>(v4_graph.clone())
            .is_err()
        {
            findings.push(finding(
                "warning",
                "strategy_config_v4_graph_shape_unverified",
                "v4_graph 未能解析为 V4MachineGraphContract，preflight 将保持受限。",
            ));
        }
    }
    ConfigDomainStatus {
        domain_id: ConfigDomainId::StateMachine,
        lifecycle: if ready {
            ConfigDomainLifecycle::Implemented
        } else {
            ConfigDomainLifecycle::Documentable
        },
        readiness: if ready {
            ConfigDomainReadiness::Ready
        } else {
            ConfigDomainReadiness::Incomplete
        },
        source_refs: refs_from_pairs([
            ("qs_source", source.qs_digest.clone()),
            ("v4_graph", source.v4_graph_digest.clone()),
        ]),
        findings,
        primary_action: Some("start_v4_simulation".to_string()),
    }
}

fn risk_domain(
    request: &StrategyConfigArtifactRequest,
    source: &StrategyConfigSourceSummary,
) -> ConfigDomainStatus {
    let mut findings = Vec::new();
    let mut risk_plane_ready = false;
    if let Some(v4_graph) = &request.v4_graph {
        match serde_json::from_value::<qrpc_core_ir::v4::V4MachineGraphContract>(v4_graph.clone()) {
            Ok(contract) => {
                risk_plane_ready = contract
                    .risk_plane
                    .as_ref()
                    .map(|risk_plane| risk_plane.required && !risk_plane.machine_ids.is_empty())
                    .unwrap_or(false);
                if let Err(errors) = contract.validate_static_contract() {
                    findings.extend(
                        errors
                            .into_iter()
                            .filter(|error| {
                                error.contains("risk_plane")
                                    || error.contains("execution machine")
                                    || error.contains("Risk Plane")
                            })
                            .take(5)
                            .map(|error| {
                                finding(
                                    "warning",
                                    "strategy_config_risk_plane_contract_invalid",
                                    error,
                                )
                            }),
                    );
                    risk_plane_ready = false;
                }
            }
            Err(_) => findings.push(finding(
                "warning",
                "strategy_config_risk_plane_shape_unverified",
                "v4_graph 未能解析为 V4MachineGraphContract，Risk Plane 只能保持受限。",
            )),
        }
    }
    if !risk_plane_ready {
        findings.push(finding(
            "warning",
            "strategy_config_risk_plane_not_attached",
            "未在 artifact 中发现已通过静态契约校验的 risk_plane 证据；执行前必须由后端 preflight 继续核验。",
        ));
    }
    ConfigDomainStatus {
        domain_id: ConfigDomainId::Risk,
        lifecycle: ConfigDomainLifecycle::Implemented,
        readiness: if risk_plane_ready {
            ConfigDomainReadiness::Ready
        } else {
            ConfigDomainReadiness::Restricted
        },
        source_refs: refs_from_pairs([("v4_graph", source.v4_graph_digest.clone())]),
        findings,
        primary_action: Some("preflight".to_string()),
    }
}

fn execution_domain(
    runtime_boundary: &RuntimeBoundarySummary,
    capability: &StrategyConfigCapabilitySummary,
) -> ConfigDomainStatus {
    let blocked = runtime_boundary
        .execution_capability_sources
        .iter()
        .any(|source| source == "unsupported");
    let mut findings = Vec::new();
    if runtime_boundary.mode_label == "PaperActual" {
        findings.push(finding(
            "info",
            "strategy_config_paper_actual_demo_boundary",
            "PaperActual 仅代表 OKX demo / testnet 边界，不代表真实资金自动交易。",
        ));
    }
    findings.extend(runtime_boundary.rejection_reasons.iter().map(|reason| {
        finding(
            if blocked { "error" } else { "warning" },
            "strategy_config_execution_boundary",
            reason.clone(),
        )
    }));
    if capability.capability_snapshot_status != "current" {
        findings.push(finding(
            "warning",
            "strategy_config_stale_capability",
            format!(
                "当前配置使用的能力快照为 {}，后端当前能力快照为 {}，需要重新核验。",
                capability.capability_snapshot_hash, capability.capability_expected_hash
            ),
        ));
    }
    let mut source_refs: Vec<ConfigSourceRef> = runtime_boundary
        .execution_capability_sources
        .iter()
        .map(|source| ConfigSourceRef {
            source_kind: "execution_capability_source".to_string(),
            source_id: source.clone(),
            digest: None,
        })
        .collect();
    source_refs.push(ConfigSourceRef {
        source_kind: "capability_snapshot".to_string(),
        source_id: capability.capability_snapshot_status.clone(),
        digest: Some(capability.capability_snapshot_hash.clone()),
    });
    ConfigDomainStatus {
        domain_id: ConfigDomainId::Execution,
        lifecycle: ConfigDomainLifecycle::Implemented,
        readiness: if blocked {
            ConfigDomainReadiness::Blocked
        } else if capability.capability_snapshot_status != "current" {
            ConfigDomainReadiness::Stale
        } else {
            ConfigDomainReadiness::Restricted
        },
        source_refs,
        findings,
        primary_action: Some("preflight".to_string()),
    }
}

fn evidence_domain(evidence_anchors: &[EvidenceAnchor]) -> ConfigDomainStatus {
    ConfigDomainStatus {
        domain_id: ConfigDomainId::Evidence,
        lifecycle: ConfigDomainLifecycle::Implemented,
        readiness: if evidence_anchors.is_empty() {
            ConfigDomainReadiness::Incomplete
        } else {
            ConfigDomainReadiness::Ready
        },
        source_refs: evidence_anchors
            .iter()
            .map(|anchor| ConfigSourceRef {
                source_kind: "evidence_anchor".to_string(),
                source_id: anchor.anchor_id.clone(),
                digest: anchor.digest.clone(),
            })
            .collect(),
        findings: if evidence_anchors.is_empty() {
            vec![finding(
                "info",
                "strategy_config_evidence_empty",
                "当前 artifact 尚未绑定运行、回测、提案或快照证据。",
            )]
        } else {
            Vec::new()
        },
        primary_action: Some("run_backtest".to_string()),
    }
}

fn ai_governance_domain(proposal_bindings: &[ProposalBinding]) -> ConfigDomainStatus {
    let unbound = proposal_bindings
        .iter()
        .any(|binding| binding.before_digest.is_none() || binding.after_digest.is_none());
    ConfigDomainStatus {
        domain_id: ConfigDomainId::AiGovernance,
        lifecycle: ConfigDomainLifecycle::Implemented,
        readiness: if unbound {
            ConfigDomainReadiness::Restricted
        } else {
            ConfigDomainReadiness::Ready
        },
        source_refs: proposal_bindings
            .iter()
            .map(|binding| ConfigSourceRef {
                source_kind: "proposal_binding".to_string(),
                source_id: binding.proposal_id.clone(),
                digest: binding.after_digest.clone(),
            })
            .collect(),
        findings: if unbound {
            vec![finding(
                "warning",
                "strategy_config_ai_binding_digest_missing",
                "AI 提案必须绑定修改前后 digest，缺失时不能激活。",
            )]
        } else if proposal_bindings.is_empty() {
            vec![finding(
                "info",
                "strategy_config_ai_proposal_only",
                "当前无 AI 提案；治理边界保持 proposal_only。",
            )]
        } else {
            Vec::new()
        },
        primary_action: Some("review_proposals".to_string()),
    }
}

fn snapshot_domain(evidence_anchors: &[EvidenceAnchor]) -> ConfigDomainStatus {
    let snapshots: Vec<_> = evidence_anchors
        .iter()
        .filter(|anchor| anchor.anchor_type == "snapshot")
        .collect();
    ConfigDomainStatus {
        domain_id: ConfigDomainId::Snapshot,
        lifecycle: ConfigDomainLifecycle::Documentable,
        readiness: if snapshots.is_empty() {
            ConfigDomainReadiness::Incomplete
        } else {
            ConfigDomainReadiness::Ready
        },
        source_refs: snapshots
            .into_iter()
            .map(|anchor| ConfigSourceRef {
                source_kind: "snapshot".to_string(),
                source_id: anchor.anchor_id.clone(),
                digest: anchor.digest.clone(),
            })
            .collect(),
        findings: if evidence_anchors
            .iter()
            .any(|anchor| anchor.anchor_type == "snapshot")
        {
            Vec::new()
        } else {
            vec![finding(
                "info",
                "strategy_config_snapshot_not_attached",
                "当前 artifact 尚未绑定快照；现有快照完整性仍使用 canonical JSON SHA-256 摘要校验。",
            )]
        },
        primary_action: Some("create_snapshot".to_string()),
    }
}

fn build_preflight_report(artifact: StrategyConfigArtifact) -> StrategyConfigPreflightReport {
    let has_source = artifact.source.graph_digest.is_some()
        || artifact.source.runtime_config_digest.is_some()
        || artifact.source.qs_digest.is_some()
        || artifact.source.core_ir_digest.is_some()
        || artifact.source.v4_graph_digest.is_some();
    let unsupported_execution = artifact
        .runtime_boundary
        .execution_capability_sources
        .iter()
        .any(|source| source == "unsupported");
    let capability_current = artifact.capability.capability_snapshot_status == "current";
    let ai_binding_incomplete = artifact.proposal_bindings.iter().any(|binding| {
        binding.before_digest.is_none()
            || binding.after_digest.is_none()
            || binding.sandbox_status == "failed"
    });
    let can_compile = has_source && !unsupported_execution;
    let can_paper_simulated = can_compile && capability_current;
    let can_backtest = can_compile && capability_current;
    let can_paper_actual_demo = can_compile
        && capability_current
        && artifact.runtime_boundary.mode_label == "PaperActual"
        && artifact.runtime_boundary.provider_order_submission_allowed;

    let mut findings = artifact
        .config_domains
        .iter()
        .flat_map(|domain| domain.findings.clone())
        .collect::<Vec<_>>();
    let mut blocked_actions = Vec::new();
    if !has_source {
        findings.push(finding(
            "error",
            "strategy_config_source_missing",
            "策略配置缺少 graph、QS、Core IR 或 v4 graph 来源，不能继续运行前核验。",
        ));
        blocked_actions.push(blocked("compile", "缺少策略配置来源"));
    }
    if unsupported_execution {
        blocked_actions.push(blocked(
            "start_runtime",
            "当前策略需要 unsupported 执行能力",
        ));
        blocked_actions.push(blocked(
            "activate_proposal",
            "unsupported 执行能力不能进入激活路径",
        ));
    }
    if !capability_current {
        findings.push(finding(
            "warning",
            "strategy_config_stale_capability",
            "当前策略配置使用的 capability 快照不是后端当前快照，请刷新能力后重新核验。",
        ));
        blocked_actions.push(blocked("start_runtime", "capability 快照不是后端当前快照"));
        blocked_actions.push(blocked("run_backtest", "capability 快照不是后端当前快照"));
        blocked_actions.push(blocked(
            "activate_proposal",
            "capability 快照不是后端当前快照",
        ));
    }
    if ai_binding_incomplete {
        blocked_actions.push(blocked(
            "activate_proposal",
            "AI 提案缺少配置域 digest 或沙箱未通过",
        ));
    }
    blocked_actions.push(blocked(
        "live_execution",
        "live_execution_allowed=false，当前不开放真实资金自动交易",
    ));

    let decision = if !has_source || unsupported_execution {
        PreflightDecision::Blocked
    } else if artifact
        .config_domains
        .iter()
        .any(|domain| domain.readiness == ConfigDomainReadiness::Restricted)
        || !capability_current
        || ai_binding_incomplete
    {
        PreflightDecision::Restricted
    } else {
        PreflightDecision::Ready
    };
    let mut allowed_actions = Vec::new();
    if can_compile {
        allowed_actions.push("compile".to_string());
    }
    if can_paper_simulated {
        allowed_actions.push("start_paper_simulated".to_string());
    }
    if can_backtest {
        allowed_actions.push("run_backtest".to_string());
    }
    if can_paper_actual_demo {
        allowed_actions.push("start_paper_actual_demo".to_string());
    }

    StrategyConfigPreflightReport {
        schema_version: STRATEGY_CONFIG_PREFLIGHT_SCHEMA.to_string(),
        artifact,
        decision,
        can_compile,
        can_paper_simulated,
        can_backtest,
        can_paper_actual_demo,
        can_live_execution: false,
        allowed_actions,
        blocked_actions,
        findings,
    }
}

fn build_diff_report(
    left: StrategyConfigArtifact,
    right: StrategyConfigArtifact,
) -> StrategyConfigDiffReport {
    let source_digest_changes = [
        (
            "graph_digest",
            left.source.graph_digest.clone(),
            right.source.graph_digest.clone(),
        ),
        (
            "runtime_config_digest",
            left.source.runtime_config_digest.clone(),
            right.source.runtime_config_digest.clone(),
        ),
        (
            "qs_digest",
            left.source.qs_digest.clone(),
            right.source.qs_digest.clone(),
        ),
        (
            "core_ir_digest",
            left.source.core_ir_digest.clone(),
            right.source.core_ir_digest.clone(),
        ),
        (
            "v4_graph_digest",
            left.source.v4_graph_digest.clone(),
            right.source.v4_graph_digest.clone(),
        ),
    ]
    .into_iter()
    .filter_map(|(field, before, after)| {
        if before == after {
            None
        } else {
            Some(StrategyConfigDigestChange {
                field: field.to_string(),
                before,
                after,
            })
        }
    })
    .collect::<Vec<_>>();

    let left_domains = domains_by_id(&left.config_domains);
    let right_domains = domains_by_id(&right.config_domains);
    let mut domain_ids = left_domains
        .keys()
        .chain(right_domains.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    let domain_changes = domain_ids
        .iter()
        .filter_map(|domain_id| {
            let left_domain = left_domains.get(domain_id);
            let right_domain = right_domains.get(domain_id);
            let lifecycle_changed =
                left_domain.map(|d| d.lifecycle) != right_domain.map(|d| d.lifecycle);
            let readiness_changed =
                left_domain.map(|d| d.readiness) != right_domain.map(|d| d.readiness);
            let source_refs_changed =
                left_domain.map(|d| &d.source_refs) != right_domain.map(|d| &d.source_refs);
            let findings_changed =
                left_domain.map(|d| &d.findings) != right_domain.map(|d| &d.findings);
            if lifecycle_changed || readiness_changed || source_refs_changed || findings_changed {
                Some(StrategyConfigDomainChange {
                    domain_id: *domain_id,
                    lifecycle_changed,
                    readiness_changed,
                    source_refs_changed,
                    findings_changed,
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    domain_ids.clear();

    let runtime_boundary_changed = left.runtime_boundary.mode_label
        != right.runtime_boundary.mode_label
        || left.runtime_boundary.provider_order_submission_attached
            != right.runtime_boundary.provider_order_submission_attached
        || left.runtime_boundary.provider_order_submission_allowed
            != right.runtime_boundary.provider_order_submission_allowed
        || left.runtime_boundary.live_execution_allowed
            != right.runtime_boundary.live_execution_allowed
        || left.runtime_boundary.execution_capability_sources
            != right.runtime_boundary.execution_capability_sources;

    let changed =
        !source_digest_changes.is_empty() || !domain_changes.is_empty() || runtime_boundary_changed;

    StrategyConfigDiffReport {
        schema_version: STRATEGY_CONFIG_DIFF_SCHEMA.to_string(),
        left_artifact_id: left.artifact_id,
        right_artifact_id: right.artifact_id,
        source_digest_changes,
        domain_changes,
        runtime_boundary_changed,
        changed,
    }
}

async fn load_bound_backtest_for_evidence(
    state: &AppState,
    user_id: &auth::UserId,
    graph_id: &str,
    backtest_id: Option<&str>,
    side: &str,
) -> Result<Option<BacktestRecord>, StrategyConfigFinding> {
    let Some(backtest_id) = backtest_id.and_then(|value| non_empty(Some(value.to_string()))) else {
        return Err(finding(
            "info",
            format!("strategy_config_evidence_{side}_backtest_missing"),
            format!("{side} version has no explicitly bound v4 backtest evidence."),
        ));
    };
    let record = load_backtest_record_from_state(state, user_id, &backtest_id)
        .await
        .map_err(|(_, message)| {
            finding(
                "warning",
                format!("strategy_config_evidence_{side}_backtest_load_failed"),
                format!("{side} backtest `{backtest_id}` could not be loaded: {message}"),
            )
        })?;
    if record.graph_id != graph_id {
        return Err(finding(
            "warning",
            format!("strategy_config_evidence_{side}_backtest_graph_mismatch"),
            format!(
                "{side} backtest `{}` belongs to graph `{}`, expected `{}`.",
                record.backtest_id, record.graph_id, graph_id
            ),
        ));
    }
    if record
        .backtest_artifacts
        .as_ref()
        .and_then(|artifacts| artifacts.v4_artifact.as_ref())
        .is_none()
    {
        return Err(finding(
            "warning",
            format!("strategy_config_evidence_{side}_v4_artifact_missing"),
            format!("{side} backtest `{backtest_id}` has no v4 backtest artifact."),
        ));
    }
    Ok(Some(record))
}

fn build_strategy_config_evidence_diff(
    left_backtest_id: Option<String>,
    right_backtest_id: Option<String>,
    left_record: Option<&BacktestRecord>,
    right_record: Option<&BacktestRecord>,
    diagnostics: Vec<StrategyConfigFinding>,
) -> StrategyConfigEvidenceDiffReport {
    let left_artifact = left_record
        .and_then(|record| record.backtest_artifacts.as_ref())
        .and_then(|artifacts| artifacts.v4_artifact.as_ref());
    let right_artifact = right_record
        .and_then(|record| record.backtest_artifacts.as_ref())
        .and_then(|artifacts| artifacts.v4_artifact.as_ref());
    let left_summary = left_record.map(|record| &record.backtest.summary);
    let right_summary = right_record.map(|record| &record.backtest.summary);

    let mut report = if let (Some(left), Some(right), Some(left_summary), Some(right_summary)) =
        (left_artifact, right_artifact, left_summary, right_summary)
    {
        let machine_trajectory = compare_machine_trajectory_evidence(left, right);
        let risk_plane = compare_risk_plane_evidence(left, right);
        let execution_capability = compare_execution_capability_evidence(left, right);
        let metrics = compare_evidence_metrics(left_summary, right_summary);
        let changed = [
            machine_trajectory.status,
            risk_plane.status,
            execution_capability.status,
            metrics.status,
        ]
        .iter()
        .any(|status| *status == StrategyConfigEvidenceDiffStatus::Different);
        StrategyConfigEvidenceDiffReport {
            schema_version: STRATEGY_CONFIG_EVIDENCE_DIFF_SCHEMA.to_string(),
            left_backtest_id,
            right_backtest_id,
            status: if changed {
                StrategyConfigEvidenceDiffStatus::Different
            } else {
                StrategyConfigEvidenceDiffStatus::Same
            },
            changed,
            diagnostics,
            machine_trajectory: Some(machine_trajectory),
            risk_plane: Some(risk_plane),
            execution_capability: Some(execution_capability),
            metrics: Some(metrics),
        }
    } else {
        StrategyConfigEvidenceDiffReport {
            schema_version: STRATEGY_CONFIG_EVIDENCE_DIFF_SCHEMA.to_string(),
            left_backtest_id,
            right_backtest_id,
            status: StrategyConfigEvidenceDiffStatus::Missing,
            changed: false,
            diagnostics,
            machine_trajectory: None,
            risk_plane: None,
            execution_capability: None,
            metrics: None,
        }
    };
    if report.diagnostics.is_empty() && report.status == StrategyConfigEvidenceDiffStatus::Missing {
        report.diagnostics.push(finding(
            "info",
            "strategy_config_evidence_backtest_required",
            "Both compared versions must explicitly bind v4 backtest evidence before evidence diff can run.",
        ));
    }
    report
}

fn compare_machine_trajectory_evidence(
    left: &qrpc_core_ir::v4::V4BacktestArtifact,
    right: &qrpc_core_ir::v4::V4BacktestArtifact,
) -> StrategyConfigMachineTrajectoryEvidenceDiff {
    let left_signatures = left
        .machine_trajectory
        .iter()
        .map(machine_trajectory_signature)
        .collect::<Vec<_>>();
    let right_signatures = right
        .machine_trajectory
        .iter()
        .map(machine_trajectory_signature)
        .collect::<Vec<_>>();
    let first_divergence = first_divergence(&left_signatures, &right_signatures);
    let left_visited_states = sorted_unique(
        left.machine_trajectory
            .iter()
            .map(|point| format!("{}:{}", point.machine_id, point.state_id)),
    );
    let right_visited_states = sorted_unique(
        right
            .machine_trajectory
            .iter()
            .map(|point| format!("{}:{}", point.machine_id, point.state_id)),
    );
    let transition_hit_changes = diff_count_maps(
        transition_hit_counts(&left.machine_trajectory),
        transition_hit_counts(&right.machine_trajectory),
    );
    let changed = first_divergence.is_some()
        || left_visited_states != right_visited_states
        || !transition_hit_changes.is_empty();
    StrategyConfigMachineTrajectoryEvidenceDiff {
        status: evidence_status(changed),
        left_point_count: left.machine_trajectory.len(),
        right_point_count: right.machine_trajectory.len(),
        left_visited_states,
        right_visited_states,
        transition_hit_changes,
        left_terminal_state: left.machine_trajectory.last().map(machine_terminal_state),
        right_terminal_state: right.machine_trajectory.last().map(machine_terminal_state),
        first_divergence,
    }
}

fn compare_risk_plane_evidence(
    left: &qrpc_core_ir::v4::V4BacktestArtifact,
    right: &qrpc_core_ir::v4::V4BacktestArtifact,
) -> StrategyConfigRiskPlaneEvidenceDiff {
    let left_signatures = left
        .risk_plane_decisions
        .iter()
        .map(risk_decision_signature)
        .collect::<Vec<_>>();
    let right_signatures = right
        .risk_plane_decisions
        .iter()
        .map(risk_decision_signature)
        .collect::<Vec<_>>();
    let first_divergence = first_divergence(&left_signatures, &right_signatures);
    let action_count_changes = diff_count_maps(
        count_by(left.risk_plane_decisions.iter().map(|record| {
            if record.approved {
                "allow".to_string()
            } else {
                "reject".to_string()
            }
        })),
        count_by(right.risk_plane_decisions.iter().map(|record| {
            if record.approved {
                "allow".to_string()
            } else {
                "reject".to_string()
            }
        })),
    );
    let reason_count_changes = diff_count_maps(
        count_by(left.risk_plane_decisions.iter().map(|record| {
            non_empty(Some(record.reason.clone())).unwrap_or_else(|| "unknown".to_string())
        })),
        count_by(right.risk_plane_decisions.iter().map(|record| {
            non_empty(Some(record.reason.clone())).unwrap_or_else(|| "unknown".to_string())
        })),
    );
    let changed = first_divergence.is_some()
        || !action_count_changes.is_empty()
        || !reason_count_changes.is_empty();
    StrategyConfigRiskPlaneEvidenceDiff {
        status: evidence_status(changed),
        left_decision_count: left.risk_plane_decisions.len(),
        right_decision_count: right.risk_plane_decisions.len(),
        left_approved_count: left
            .risk_plane_decisions
            .iter()
            .filter(|record| record.approved)
            .count(),
        right_approved_count: right
            .risk_plane_decisions
            .iter()
            .filter(|record| record.approved)
            .count(),
        left_rejected_count: left
            .risk_plane_decisions
            .iter()
            .filter(|record| !record.approved)
            .count(),
        right_rejected_count: right
            .risk_plane_decisions
            .iter()
            .filter(|record| !record.approved)
            .count(),
        action_count_changes,
        reason_count_changes,
        first_divergence,
    }
}

fn compare_execution_capability_evidence(
    left: &qrpc_core_ir::v4::V4BacktestArtifact,
    right: &qrpc_core_ir::v4::V4BacktestArtifact,
) -> StrategyConfigExecutionCapabilityEvidenceDiff {
    let left_signatures = left
        .execution_capability_sources
        .iter()
        .map(execution_capability_signature)
        .collect::<Vec<_>>();
    let right_signatures = right
        .execution_capability_sources
        .iter()
        .map(execution_capability_signature)
        .collect::<Vec<_>>();
    let first_divergence = first_divergence(&left_signatures, &right_signatures);
    let runtime_mode_changes = diff_count_maps(
        count_by(
            left.execution_capability_sources
                .iter()
                .map(|record| json_label(&record.runtime_mode)),
        ),
        count_by(
            right
                .execution_capability_sources
                .iter()
                .map(|record| json_label(&record.runtime_mode)),
        ),
    );
    let capability_kind_changes = diff_count_maps(
        count_by(
            left.execution_capability_sources
                .iter()
                .map(|record| json_label(&record.capability)),
        ),
        count_by(
            right
                .execution_capability_sources
                .iter()
                .map(|record| json_label(&record.capability)),
        ),
    );
    let capability_source_changes = diff_count_maps(
        count_by(
            left.execution_capability_sources
                .iter()
                .map(|record| json_label(&record.source)),
        ),
        count_by(
            right
                .execution_capability_sources
                .iter()
                .map(|record| json_label(&record.source)),
        ),
    );
    let status_changes = diff_count_maps(
        count_by(
            left.execution_capability_sources
                .iter()
                .map(|record| record.status.clone()),
        ),
        count_by(
            right
                .execution_capability_sources
                .iter()
                .map(|record| record.status.clone()),
        ),
    );
    let changed = first_divergence.is_some()
        || !runtime_mode_changes.is_empty()
        || !capability_kind_changes.is_empty()
        || !capability_source_changes.is_empty()
        || !status_changes.is_empty();
    StrategyConfigExecutionCapabilityEvidenceDiff {
        status: evidence_status(changed),
        left_source_count: left.execution_capability_sources.len(),
        right_source_count: right.execution_capability_sources.len(),
        left_accepted_count: left
            .execution_capability_sources
            .iter()
            .filter(|record| record.accepted)
            .count(),
        right_accepted_count: right
            .execution_capability_sources
            .iter()
            .filter(|record| record.accepted)
            .count(),
        left_rejected_count: left
            .execution_capability_sources
            .iter()
            .filter(|record| !record.accepted)
            .count(),
        right_rejected_count: right
            .execution_capability_sources
            .iter()
            .filter(|record| !record.accepted)
            .count(),
        runtime_mode_changes,
        capability_kind_changes,
        capability_source_changes,
        status_changes,
        first_divergence,
    }
}

fn compare_evidence_metrics(
    left: &qrpc_core::BacktestSummary,
    right: &qrpc_core::BacktestSummary,
) -> StrategyConfigEvidenceMetricsDiff {
    let fields = vec![
        evidence_field("step_count", left.step_count, right.step_count),
        evidence_field("trade_count", left.trade_count, right.trade_count),
        evidence_field(
            "total_return_ratio",
            stable_float(left.total_return_ratio),
            stable_float(right.total_return_ratio),
        ),
        evidence_field(
            "max_drawdown_ratio",
            stable_float(left.drawdown_analysis.max_drawdown_ratio),
            stable_float(right.drawdown_analysis.max_drawdown_ratio),
        ),
        evidence_field(
            "final_equity",
            stable_float(left.final_equity),
            stable_float(right.final_equity),
        ),
        evidence_field(
            "net_profit",
            stable_float(left.net_profit),
            stable_float(right.net_profit),
        ),
        evidence_field(
            "win_rate",
            stable_float(left.win_rate),
            stable_float(right.win_rate),
        ),
    ];
    let changed = fields
        .iter()
        .any(|field| field.status == StrategyConfigEvidenceDiffStatus::Different);
    StrategyConfigEvidenceMetricsDiff {
        status: evidence_status(changed),
        fields,
    }
}

fn evidence_field<T: ToString + PartialEq>(
    key: &str,
    left: T,
    right: T,
) -> StrategyConfigEvidenceFieldDiff {
    let changed = left != right;
    StrategyConfigEvidenceFieldDiff {
        key: key.to_string(),
        status: evidence_status(changed),
        left_value: Some(left.to_string()),
        right_value: Some(right.to_string()),
    }
}

fn stable_float(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.8}")
    } else {
        "nan".to_string()
    }
}

fn evidence_status(changed: bool) -> StrategyConfigEvidenceDiffStatus {
    if changed {
        StrategyConfigEvidenceDiffStatus::Different
    } else {
        StrategyConfigEvidenceDiffStatus::Same
    }
}

fn machine_trajectory_signature(
    point: &qrpc_core_ir::v4::V4BacktestMachineTrajectoryPoint,
) -> String {
    format!(
        "{}:{}:{}:{}",
        point.machine_id,
        point.state_id,
        point.status,
        point.symbol.as_deref().unwrap_or("*")
    )
}

fn machine_terminal_state(point: &qrpc_core_ir::v4::V4BacktestMachineTrajectoryPoint) -> String {
    format!(
        "{}:{}:{}",
        point.machine_id,
        point.state_id,
        point.symbol.as_deref().unwrap_or("*")
    )
}

fn transition_hit_counts(
    points: &[qrpc_core_ir::v4::V4BacktestMachineTrajectoryPoint],
) -> BTreeMap<String, usize> {
    let mut ordered = points.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        left.event_sequence
            .cmp(&right.event_sequence)
            .then_with(|| left.ts_ms.cmp(&right.ts_ms))
    });
    let transitions = ordered.windows(2).filter_map(|pair| {
        let left = pair[0];
        let right = pair[1];
        if left.machine_id != right.machine_id
            || left.symbol != right.symbol
            || left.state_id == right.state_id
        {
            return None;
        }
        Some(format!(
            "{}:{}->{}:{}",
            left.machine_id,
            left.state_id,
            right.state_id,
            left.symbol.as_deref().unwrap_or("*")
        ))
    });
    count_by(transitions)
}

fn risk_decision_signature(record: &qrpc_core_ir::v4::V4BacktestRiskPlaneDecisionRecord) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        record.target_machine_id,
        record.event_type,
        if record.approved { "allow" } else { "reject" },
        record.reason,
        record.symbol.as_deref().unwrap_or("*")
    )
}

fn execution_capability_signature(
    record: &qrpc_core_ir::v4::V4BacktestExecutionCapabilitySourceRecord,
) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}:{}",
        record.target_machine_id,
        json_label(&record.runtime_mode),
        if record.accepted {
            "accepted"
        } else {
            "rejected"
        },
        json_label(&record.capability),
        json_label(&record.source),
        record.status,
        record.symbol.as_deref().unwrap_or("*")
    )
}

fn first_divergence(
    left: &[String],
    right: &[String],
) -> Option<StrategyConfigEvidenceFirstDivergence> {
    let max_len = left.len().max(right.len());
    (0..max_len).find_map(|index| {
        let left_value = left.get(index).cloned();
        let right_value = right.get(index).cloned();
        (left_value != right_value).then_some(StrategyConfigEvidenceFirstDivergence {
            index,
            left: left_value,
            right: right_value,
        })
    })
}

fn sorted_unique(values: impl Iterator<Item = String>) -> Vec<String> {
    values.collect::<BTreeSet<_>>().into_iter().collect()
}

fn count_by(values: impl Iterator<Item = String>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value).or_insert(0) += 1;
    }
    counts
}

fn diff_count_maps(
    left: BTreeMap<String, usize>,
    right: BTreeMap<String, usize>,
) -> Vec<StrategyConfigEvidenceCountChange> {
    left.keys()
        .chain(right.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter_map(|key| {
            let left_count = left.get(&key).copied().unwrap_or(0);
            let right_count = right.get(&key).copied().unwrap_or(0);
            (left_count != right_count).then_some(StrategyConfigEvidenceCountChange {
                key,
                left_count,
                right_count,
            })
        })
        .collect()
}

fn json_label(value: &impl Serialize) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| match value {
            Value::String(value) => Some(value),
            other => Some(other.to_string()),
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn domains_by_id(domains: &[ConfigDomainStatus]) -> BTreeMap<ConfigDomainId, &ConfigDomainStatus> {
    domains
        .iter()
        .map(|domain| (domain.domain_id, domain))
        .collect()
}

fn refs_from_pairs<const N: usize>(pairs: [(&str, Option<String>); N]) -> Vec<ConfigSourceRef> {
    pairs
        .into_iter()
        .filter_map(|(source_kind, digest)| {
            digest.map(|value| ConfigSourceRef {
                source_kind: source_kind.to_string(),
                source_id: source_kind.to_string(),
                digest: Some(value),
            })
        })
        .collect()
}

fn artifact_digest_input(artifact: &StrategyConfigArtifact) -> Value {
    json!({
        "schema_version": artifact.schema_version,
        "artifact_id": artifact.artifact_id,
        "strategy_id": artifact.strategy_id,
        "strategy_version": artifact.strategy_version,
        "created_at_ms": artifact.created_at_ms,
        "source": artifact.source,
        "capability": artifact.capability,
        "config_domains": artifact.config_domains,
        "runtime_boundary": artifact.runtime_boundary,
        "evidence_anchors": artifact.evidence_anchors,
        "proposal_bindings": artifact.proposal_bindings,
    })
}

fn digest_option_value(value: &Option<Value>) -> Option<String> {
    value
        .as_ref()
        .and_then(|value| digest_for_value(value).ok())
}

fn digest_for_value(value: &impl Serialize) -> Result<String, (StatusCode, String)> {
    canonical_json_sha256_digest(value)
        .map(|digest| format!("sha256:{}", digest.value))
        .map_err(|error| {
            json_bad_request(
                "strategy_config_digest_failed",
                format!("策略配置摘要计算失败: {error}"),
            )
        })
}

fn infer_source_mode(request: &StrategyConfigArtifactRequest) -> String {
    if request.v4_graph.is_some() {
        "v4_qs_handoff".to_string()
    } else if request.qs_source.is_some() {
        "formal_qs".to_string()
    } else {
        "strategy_graph".to_string()
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn finding(
    severity: impl Into<String>,
    code: impl Into<String>,
    message: impl Into<String>,
) -> StrategyConfigFinding {
    StrategyConfigFinding {
        severity: severity.into(),
        code: code.into(),
        message: message.into(),
    }
}

fn blocked(action: impl Into<String>, reason: impl Into<String>) -> PreflightBlockedAction {
    PreflightBlockedAction {
        action: action.into(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> StrategyConfigArtifactRequest {
        StrategyConfigArtifactRequest {
            strategy_id: Some("s1".to_string()),
            strategy_version: Some("v1".to_string()),
            source_mode: None,
            graph_json: Some(json!({"nodes": [{"id": "data"}]})),
            runtime_config: Some(json!({"metadata": {"graph_id": "s1"}})),
            qs_source: Some("v4_strategy demo { machine observe {} }".to_string()),
            core_ir: None,
            v4_graph: Some(json!({"graph_id": "g1", "risk_plane": {"required": true}})),
            capability_snapshot_hash: Some(current_capability_hash().to_string()),
            capability_source: Some("test".to_string()),
            runtime_mode: Some("PaperSimulated".to_string()),
            evidence_anchors: vec![EvidenceAnchorInput {
                anchor_type: "backtest".to_string(),
                anchor_id: Some("bt1".to_string()),
                digest: Some("sha256:abc".to_string()),
                summary: None,
            }],
            proposal_bindings: Vec::new(),
            required_execution_capability_sources: Vec::new(),
        }
    }

    #[test]
    fn artifact_digest_is_populated_and_keeps_paper_boundary() {
        let artifact = build_strategy_config_artifact(sample_request(), 100).unwrap();

        assert_eq!(artifact.schema_version, STRATEGY_CONFIG_ARTIFACT_SCHEMA);
        assert!(artifact.artifact_digest.starts_with("sha256:"));
        assert_eq!(artifact.runtime_boundary.mode_label, "PaperSimulated");
        assert!(!artifact.runtime_boundary.live_execution_allowed);
    }

    #[test]
    fn preflight_blocks_unsupported_execution() {
        let mut request = sample_request();
        request.required_execution_capability_sources = vec!["unsupported".to_string()];
        let report = build_preflight_report(build_strategy_config_artifact(request, 100).unwrap());

        assert_eq!(report.decision, PreflightDecision::Blocked);
        assert!(!report.can_paper_simulated);
        assert!(report
            .blocked_actions
            .iter()
            .any(|action| action.action == "start_runtime"));
    }

    #[test]
    fn preflight_restricts_stale_capability_snapshot() {
        let mut request = sample_request();
        request.capability_snapshot_hash = Some(format!("sha256:{}", "0".repeat(64)));
        let report = build_preflight_report(build_strategy_config_artifact(request, 100).unwrap());

        assert_eq!(report.decision, PreflightDecision::Restricted);
        assert!(report.can_compile);
        assert!(!report.can_paper_simulated);
        assert!(!report.can_backtest);
        assert_eq!(
            report.artifact.capability.capability_snapshot_status,
            "stale"
        );
        assert!(report
            .findings
            .iter()
            .any(|finding| finding.code == "strategy_config_stale_capability"));
        assert!(report
            .blocked_actions
            .iter()
            .any(|action| action.action == "start_runtime"));
    }

    #[test]
    fn legacy_live_label_is_normalized_to_paper_actual_demo() {
        let mut request = sample_request();
        request.runtime_mode = Some("live".to_string());
        let artifact = build_strategy_config_artifact(request, 100).unwrap();

        assert_eq!(artifact.runtime_boundary.mode_label, "PaperActual");
        assert!(!artifact.runtime_boundary.live_execution_allowed);
        assert!(artifact
            .runtime_boundary
            .rejection_reasons
            .iter()
            .any(|reason| reason.contains("legacy_live_label")));
    }

    fn v4_evidence_artifact(
        state_id: &str,
        approved: bool,
        accepted: bool,
    ) -> qrpc_core_ir::v4::V4BacktestArtifact {
        qrpc_core_ir::v4::V4BacktestArtifact {
            schema_version: "quantpilot/v4-backtest-artifact/v1".to_string(),
            graph_id: "g1".to_string(),
            started_at_ms: 1,
            ended_at_ms: 2,
            replay_mode: "deterministic_bar_replay".to_string(),
            input_bar_count: 1,
            input_tick_count: None,
            symbols: vec!["BTCUSDT".to_string()],
            machine_trajectory: vec![
                qrpc_core_ir::v4::V4BacktestMachineTrajectoryPoint {
                    ts_ms: 1,
                    event_sequence: 1,
                    machine_id: "machine".to_string(),
                    template: qrpc_core_ir::v4::MachineTemplateKind::Observation,
                    state_id: "observe".to_string(),
                    status: "ok".to_string(),
                    symbol: Some("BTCUSDT".to_string()),
                },
                qrpc_core_ir::v4::V4BacktestMachineTrajectoryPoint {
                    ts_ms: 2,
                    event_sequence: 2,
                    machine_id: "machine".to_string(),
                    template: qrpc_core_ir::v4::MachineTemplateKind::Execution,
                    state_id: state_id.to_string(),
                    status: "ok".to_string(),
                    symbol: Some("BTCUSDT".to_string()),
                },
            ],
            risk_plane_decisions: vec![qrpc_core_ir::v4::V4BacktestRiskPlaneDecisionRecord {
                decision_id: "risk_1".to_string(),
                target_machine_id: "machine".to_string(),
                source_machine_id: "risk".to_string(),
                event_type: "bar".to_string(),
                approved,
                reason: if approved { "approved" } else { "risk_limit" }.to_string(),
                ts_ms: 2,
                sequence: 2,
                symbol: Some("BTCUSDT".to_string()),
            }],
            execution_capability_sources: vec![
                qrpc_core_ir::v4::V4BacktestExecutionCapabilitySourceRecord {
                    decision_id: "exec_1".to_string(),
                    target_machine_id: "machine".to_string(),
                    venue_id: "paper-local".to_string(),
                    runtime_mode: qrpc_core_ir::v4::RuntimeTradingMode::PaperSimulated,
                    accepted,
                    reason: if accepted { "accepted" } else { "unsupported" }.to_string(),
                    capability: qrpc_core_ir::v4::ExecutionCapabilityKind::Market,
                    source: if accepted {
                        qrpc_core_ir::v4::CapabilitySupportSource::RuntimeSimulated
                    } else {
                        qrpc_core_ir::v4::CapabilitySupportSource::Unsupported
                    },
                    status: if accepted { "accepted" } else { "rejected" }.to_string(),
                    ts_ms: 2,
                    sequence: 2,
                    symbol: Some("BTCUSDT".to_string()),
                },
            ],
            microstructure_metrics: None,
            final_snapshot: None,
        }
    }

    #[test]
    fn evidence_diff_detects_v4_behavior_changes() {
        let left = v4_evidence_artifact("trade", true, true);
        let right = v4_evidence_artifact("blocked", false, false);

        let trajectory = compare_machine_trajectory_evidence(&left, &right);
        let risk = compare_risk_plane_evidence(&left, &right);
        let execution = compare_execution_capability_evidence(&left, &right);

        assert_eq!(
            trajectory.status,
            StrategyConfigEvidenceDiffStatus::Different
        );
        assert!(trajectory.first_divergence.is_some());
        assert_eq!(risk.right_rejected_count, 1);
        assert!(risk
            .reason_count_changes
            .iter()
            .any(|change| change.key == "risk_limit"));
        assert_eq!(
            execution.status,
            StrategyConfigEvidenceDiffStatus::Different
        );
        assert!(execution
            .capability_source_changes
            .iter()
            .any(|change| change.key == "unsupported"));
    }
}
