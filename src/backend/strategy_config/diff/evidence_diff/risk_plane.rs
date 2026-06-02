use serde::{Deserialize, Serialize};

use super::{
    count_by, diff_count_maps, evidence_status, first_divergence, non_empty,
    StrategyConfigEvidenceCountChange, StrategyConfigEvidenceDiffStatus,
    StrategyConfigEvidenceFirstDivergence,
};

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
