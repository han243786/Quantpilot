use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::auth;
use crate::backend::strategy_config::artifact::{finding, non_empty, StrategyConfigFinding};
use crate::{load_backtest_record_from_state, AppState, BacktestRecord};

const STRATEGY_CONFIG_EVIDENCE_DIFF_SCHEMA: &str = "quantpilot/v4-strategy-config-evidence-diff/v1";

pub mod machine_trajectory;
pub(crate) use machine_trajectory::{
    compare_machine_trajectory_evidence, StrategyConfigMachineTrajectoryEvidenceDiff,
};
pub mod risk_plane;
pub(crate) use risk_plane::{compare_risk_plane_evidence, StrategyConfigRiskPlaneEvidenceDiff};
pub mod execution_capability;
pub(crate) use execution_capability::{
    compare_execution_capability_evidence, StrategyConfigExecutionCapabilityEvidenceDiff,
};
pub mod metrics;
pub(crate) use metrics::{compare_evidence_metrics, StrategyConfigEvidenceMetricsDiff};

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
pub(crate) struct StrategyConfigEvidenceCountChange {
    pub(crate) key: String,
    pub(crate) left_count: usize,
    pub(crate) right_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StrategyConfigEvidenceFirstDivergence {
    pub(crate) index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) left: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) right: Option<String>,
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

fn evidence_status(changed: bool) -> StrategyConfigEvidenceDiffStatus {
    if changed {
        StrategyConfigEvidenceDiffStatus::Different
    } else {
        StrategyConfigEvidenceDiffStatus::Same
    }
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
