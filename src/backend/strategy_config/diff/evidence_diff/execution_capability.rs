use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    count_by, diff_count_maps, evidence_status, first_divergence,
    StrategyConfigEvidenceCountChange, StrategyConfigEvidenceDiffStatus,
    StrategyConfigEvidenceFirstDivergence,
};

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

fn json_label(value: &impl Serialize) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| match value {
            Value::String(value) => Some(value),
            other => Some(other.to_string()),
        })
        .unwrap_or_else(|| "unknown".to_string())
}
