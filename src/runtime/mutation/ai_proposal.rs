use super::*;

#[path = "ai_proposal/approval_persistence.rs"]
mod approval_persistence;
#[path = "ai_proposal/approval_review.rs"]
mod approval_review;
#[path = "ai_proposal/event_lifecycle.rs"]
mod event_lifecycle;
#[path = "ai_proposal/proposal_creation.rs"]
mod proposal_creation;
#[path = "ai_proposal/record_query.rs"]
mod record_query;
#[path = "ai_proposal/sandbox_trigger.rs"]
mod sandbox_trigger;
#[path = "ai_proposal/source_governance_identity.rs"]
mod source_governance_identity;
#[path = "ai_proposal/static_check.rs"]
mod static_check;
#[path = "ai_proposal/status_transition.rs"]
mod status_transition;

use approval_persistence::{load_approval_from_disk, persist_approval};
pub(crate) use approval_review::{
    approve_ai_proposal, claim_ai_proposal_review, get_runtime_approval_detail,
    list_runtime_approvals, reject_ai_proposal,
};
use event_lifecycle::{
    ai_proposal_lifecycle_entry, build_runtime_ai_proposal_event,
    persist_runtime_ai_proposal_transition,
};
pub(crate) use proposal_creation::create_runtime_ai_proposal;
use record_query::load_runtime_ai_proposal_for_user;
pub(crate) use record_query::{get_runtime_ai_proposal_detail, list_runtime_ai_proposals};
use sandbox_trigger::{ensure_ai_proposal_can_be_approved, spawn_ai_proposal_sandbox_verification};
use source_governance_identity::{
    load_runtime_ai_proposal_source_context, runtime_ai_proposal_governance,
    runtime_ai_proposal_record_id,
};
use static_check::{
    ai_proposal_static_check_result, validate_ai_model_identity, validate_hash_identity,
};
use status_transition::{ai_proposal_approved_status, update_ai_proposal_status};

#[cfg(test)]
mod v4_ai_proposal_tests {
    use super::*;

    fn hash(ch: char) -> String {
        format!("sha256:{}", ch.to_string().repeat(64))
    }

    fn v4_target() -> RuntimeParameterMutationTarget {
        RuntimeParameterMutationTarget {
            node_id: "compat.execution".to_string(),
            module_key: "v4.machine.param".to_string(),
            parameter_path: "v4.machine.timeout_ms".to_string(),
        }
    }

    fn v4_binding(
        before_digest: String,
        after_digest: String,
    ) -> RuntimeAiProposalConfigDomainBinding {
        RuntimeAiProposalConfigDomainBinding {
            target_domain: StrategyConfigProposalDomain::StateMachine,
            before_digest,
            after_digest,
            evidence_anchor_ids: vec!["backtest:bt1".to_string()],
        }
    }

    fn sample_ai_proposal(
        status: RuntimeAiProposalStatus,
        binding: Option<RuntimeAiProposalConfigDomainBinding>,
    ) -> RuntimeAiProposalRecord {
        RuntimeAiProposalRecord {
            ai_proposal_id: "ai_proposal_test".to_string(),
            source_kind: RuntimeEvidenceSourceKind::Backtest,
            source_id: "bt1".to_string(),
            graph_id: "graph-v4".to_string(),
            source_evidence: RuntimeAiProposalSourceEvidence {
                source_kind: RuntimeEvidenceSourceKind::Backtest,
                source_id: "bt1".to_string(),
                graph_id: "graph-v4".to_string(),
                event_count: 1,
                evidence_hash: hash('a'),
            },
            target: v4_target(),
            old_value: json!(1),
            new_value: json!(2),
            old_parameter_version: hash('b'),
            proposed_parameter_version: hash('c'),
            status,
            denial_reason: None,
            static_check: RuntimeAiProposalStaticCheckResult {
                status,
                reason_code: "AI_PROPOSAL_STATIC_CHECK_PASSED".to_string(),
                message: "AI proposal candidate passed static validation".to_string(),
                checked_at_ms: 1,
                details: Vec::new(),
            },
            model: RuntimeAiModelIdentity {
                provider: "test".to_string(),
                model: "local".to_string(),
                model_version: "v1".to_string(),
            },
            prompt_hash: hash('d'),
            evidence_hash: hash('a'),
            actor: ActorIdentity {
                actor_id: "ai".to_string(),
                display_name: "AI".to_string(),
            },
            reason: "Tune v4 machine timeout from trajectory evidence".to_string(),
            governance: RuntimeAiProposalGovernance {
                capability_hash: hash('e'),
                deployment_revision: hash('f'),
                strategy_version: "v1".to_string(),
                previous_parameter_version: hash('b'),
                proposed_parameter_version: hash('c'),
                permission_boundary_model_version: "quantpilot/permission-boundary/v1".to_string(),
                ai_write_policy: "proposal_only".to_string(),
            },
            config_domain_binding: binding,
            lifecycle: Vec::new(),
            created_at_ms: 1,
            updated_at_ms: 1,
        }
    }

    #[tokio::test]
    async fn ai_proposal_approval_requires_binding_and_sandbox_report() {
        let root =
            std::env::temp_dir().join(format!("quantpilot_ai_binding_test_{}", current_time_ms()));
        let state = crate::app_runtime_helpers::new_app_state(
            root.join("graphs"),
            root.join("runs"),
            root.join("backtests"),
        );
        let old = hash('b');
        let new = hash('c');
        let unbound = sample_ai_proposal(RuntimeAiProposalStatus::StaticCheckPassed, None);
        let error = ensure_ai_proposal_can_be_approved(&state, &unbound)
            .await
            .unwrap_err();
        assert_eq!(error.0, StatusCode::LOCKED);
        assert!(error.1.contains("strategy_config_ai_binding_required"));

        let bound = sample_ai_proposal(
            RuntimeAiProposalStatus::StaticCheckPassed,
            Some(v4_binding(old, new)),
        );
        let missing_sandbox = ensure_ai_proposal_can_be_approved(&state, &bound)
            .await
            .unwrap_err();
        assert_eq!(missing_sandbox.0, StatusCode::LOCKED);
        assert!(missing_sandbox.1.contains("ai_proposal_sandbox_required"));

        state.sandbox_reports.write().await.insert(
            bound.ai_proposal_id.clone(),
            SandboxVerificationReport {
                proposal_id: bound.ai_proposal_id.clone(),
                sandbox_run_id: "sbx-run-test".to_string(),
                replay_window: ReplayWindow {
                    from_ts: "2026-05-01T00:00:00Z".to_string(),
                    to_ts: "2026-05-02T00:00:00Z".to_string(),
                },
                baseline_metrics: SandboxMetrics::default(),
                candidate_metrics: SandboxMetrics::default(),
                diffs: SandboxMetricsDiff {
                    total_return_ratio: "+0.0000".to_string(),
                    max_drawdown_ratio: "+0.0000".to_string(),
                    sharpe_ratio: "+0.0000".to_string(),
                    win_rate: "+0.0000".to_string(),
                    avg_hold_hours: "+0.0000".to_string(),
                    turnover_ratio: "+0.0000".to_string(),
                    profit_factor: "+0.0000".to_string(),
                    calmar_ratio: "+0.0000".to_string(),
                },
                verdict: SandboxVerdict::CandidateComparable,
                warnings: Vec::new(),
                replay_fidelity: "partial".to_string(),
                generated_at_ms: 1,
            },
        );

        ensure_ai_proposal_can_be_approved(&state, &bound)
            .await
            .unwrap();
        let _ = std::fs::remove_dir_all(root);
    }
}
