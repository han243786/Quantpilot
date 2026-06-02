use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::{
    count_by, diff_count_maps, evidence_status, first_divergence, sorted_unique,
    StrategyConfigEvidenceCountChange, StrategyConfigEvidenceDiffStatus,
    StrategyConfigEvidenceFirstDivergence,
};

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
