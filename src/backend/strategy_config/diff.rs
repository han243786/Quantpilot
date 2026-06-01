use axum::{http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

use crate::auth;
use crate::backend::strategy_config::artifact::{
    build_strategy_config_artifact, finding, non_empty, version_artifact_request, ConfigDomainId,
    ConfigDomainStatus, StrategyConfigArtifact, StrategyConfigFinding,
};
use crate::{load_backtest_record_from_state, AppState, BacktestRecord, GraphVersionEntry};

pub const MODULE_ID: &str = "backend.strategy_config.diff";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    register_strategy_config_diff_route(router)
}
const STRATEGY_CONFIG_DIFF_SCHEMA: &str = "quantpilot/v4-strategy-config-diff/v1";
const STRATEGY_CONFIG_EVIDENCE_DIFF_SCHEMA: &str = "quantpilot/v4-strategy-config-evidence-diff/v1";

pub(crate) fn register_strategy_config_diff_route(router: Router<AppState>) -> Router<AppState> {
    router.route("/api/v1/strategy-config/diff", post(diff_strategy_config))
}

async fn diff_strategy_config(
    Json(request): Json<StrategyConfigDiffRequest>,
) -> Result<Json<StrategyConfigDiffReport>, (StatusCode, String)> {
    Ok(Json(build_diff_report(request.left, request.right)))
}

pub(crate) fn build_strategy_config_version_diff(
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

pub(crate) async fn build_strategy_config_evidence_diff_for_backtests(
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

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct StrategyConfigDiffRequest {
    pub(crate) left: StrategyConfigArtifact,
    pub(crate) right: StrategyConfigArtifact,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StrategyConfigDiffReport {
    pub(crate) schema_version: String,
    pub(crate) left_artifact_id: String,
    pub(crate) right_artifact_id: String,
    pub(crate) source_digest_changes: Vec<StrategyConfigDigestChange>,
    pub(crate) domain_changes: Vec<StrategyConfigDomainChange>,
    pub(crate) runtime_boundary_changed: bool,
    pub(crate) changed: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StrategyConfigEvidenceDiffStatus {
    Same,
    Different,
    Missing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StrategyConfigEvidenceDiffReport {
    pub(crate) schema_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) left_backtest_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) right_backtest_id: Option<String>,
    pub(crate) status: StrategyConfigEvidenceDiffStatus,
    pub(crate) changed: bool,
    #[serde(default)]
    pub(crate) diagnostics: Vec<StrategyConfigFinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) machine_trajectory: Option<StrategyConfigMachineTrajectoryEvidenceDiff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) risk_plane: Option<StrategyConfigRiskPlaneEvidenceDiff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) execution_capability: Option<StrategyConfigExecutionCapabilityEvidenceDiff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) metrics: Option<StrategyConfigEvidenceMetricsDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StrategyConfigMachineTrajectoryEvidenceDiff {
    pub(crate) status: StrategyConfigEvidenceDiffStatus,
    pub(crate) left_point_count: usize,
    pub(crate) right_point_count: usize,
    #[serde(default)]
    pub(crate) left_visited_states: Vec<String>,
    #[serde(default)]
    pub(crate) right_visited_states: Vec<String>,
    #[serde(default)]
    pub(crate) transition_hit_changes: Vec<StrategyConfigEvidenceCountChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) left_terminal_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) right_terminal_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) first_divergence: Option<StrategyConfigEvidenceFirstDivergence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StrategyConfigRiskPlaneEvidenceDiff {
    pub(crate) status: StrategyConfigEvidenceDiffStatus,
    pub(crate) left_decision_count: usize,
    pub(crate) right_decision_count: usize,
    pub(crate) left_approved_count: usize,
    pub(crate) right_approved_count: usize,
    pub(crate) left_rejected_count: usize,
    pub(crate) right_rejected_count: usize,
    #[serde(default)]
    pub(crate) action_count_changes: Vec<StrategyConfigEvidenceCountChange>,
    #[serde(default)]
    pub(crate) reason_count_changes: Vec<StrategyConfigEvidenceCountChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) first_divergence: Option<StrategyConfigEvidenceFirstDivergence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StrategyConfigExecutionCapabilityEvidenceDiff {
    pub(crate) status: StrategyConfigEvidenceDiffStatus,
    pub(crate) left_source_count: usize,
    pub(crate) right_source_count: usize,
    pub(crate) left_accepted_count: usize,
    pub(crate) right_accepted_count: usize,
    pub(crate) left_rejected_count: usize,
    pub(crate) right_rejected_count: usize,
    #[serde(default)]
    pub(crate) runtime_mode_changes: Vec<StrategyConfigEvidenceCountChange>,
    #[serde(default)]
    pub(crate) capability_kind_changes: Vec<StrategyConfigEvidenceCountChange>,
    #[serde(default)]
    pub(crate) capability_source_changes: Vec<StrategyConfigEvidenceCountChange>,
    #[serde(default)]
    pub(crate) status_changes: Vec<StrategyConfigEvidenceCountChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) first_divergence: Option<StrategyConfigEvidenceFirstDivergence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StrategyConfigEvidenceMetricsDiff {
    pub(crate) status: StrategyConfigEvidenceDiffStatus,
    #[serde(default)]
    pub(crate) fields: Vec<StrategyConfigEvidenceFieldDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StrategyConfigEvidenceCountChange {
    pub(crate) key: String,
    pub(crate) left_count: usize,
    pub(crate) right_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StrategyConfigEvidenceFieldDiff {
    pub(crate) key: String,
    pub(crate) status: StrategyConfigEvidenceDiffStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) left_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) right_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StrategyConfigEvidenceFirstDivergence {
    pub(crate) index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) left: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) right: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StrategyConfigDigestChange {
    pub(crate) field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) after: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StrategyConfigDomainChange {
    pub(crate) domain_id: ConfigDomainId,
    pub(crate) lifecycle_changed: bool,
    pub(crate) readiness_changed: bool,
    pub(crate) source_refs_changed: bool,
    pub(crate) findings_changed: bool,
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

pub(crate) fn compare_machine_trajectory_evidence(
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

pub(crate) fn compare_risk_plane_evidence(
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

pub(crate) fn compare_execution_capability_evidence(
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
