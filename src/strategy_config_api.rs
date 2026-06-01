#[cfg(test)]
use super::*;
#[cfg(test)]
use crate::backend::strategy_config::artifact::{
    build_strategy_config_artifact, STRATEGY_CONFIG_ARTIFACT_SCHEMA,
};
pub(super) use crate::backend::strategy_config::artifact::{
    EvidenceAnchorInput, StrategyConfigArtifactRequest,
};
pub(crate) use crate::backend::strategy_config::preflight::build_strategy_config_preflight_value;
#[cfg(test)]
use crate::backend::strategy_config::preflight::{build_preflight_report, PreflightDecision};

pub(crate) use crate::backend::strategy_config::diff::{
    build_strategy_config_evidence_diff_for_backtests, build_strategy_config_version_diff,
    StrategyConfigDiffReport, StrategyConfigEvidenceDiffReport,
};
#[cfg(test)]
use crate::backend::strategy_config::diff::{
    compare_execution_capability_evidence, compare_machine_trajectory_evidence,
    compare_risk_plane_evidence, StrategyConfigEvidenceDiffStatus,
};
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
