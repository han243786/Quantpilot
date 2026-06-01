use super::*;
#[cfg(test)]
use crate::backend::strategy_config::artifact::STRATEGY_CONFIG_ARTIFACT_SCHEMA;
pub(super) use crate::backend::strategy_config::artifact::{
    build_strategy_config_artifact, EvidenceAnchorInput, StrategyConfigArtifactRequest,
};
use crate::backend::strategy_config::artifact::{
    finding, non_empty, version_artifact_request, ConfigDomainId, ConfigDomainReadiness,
    ConfigDomainStatus, StrategyConfigArtifact, StrategyConfigFinding,
};

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
