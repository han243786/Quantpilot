use super::*;

#[path = "ai_proposal/approval_persistence.rs"]
mod approval_persistence;
#[path = "ai_proposal/approval_review.rs"]
mod approval_review;
#[path = "ai_proposal/event_lifecycle.rs"]
mod event_lifecycle;
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

pub(crate) async fn create_runtime_ai_proposal(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Json(request): Json<CreateRuntimeAiProposalRequest>,
) -> Result<Json<RuntimeAiProposalRecord>, (StatusCode, String)> {
    validate_runtime_capability_guard(request.capability_context.as_ref()).map_err(|details| {
        json_bad_request_with_details(
            "ai_proposal_denied",
            "AI proposal candidates require a current capability hash and permission boundary",
            details,
        )
    })?;
    let capability_context = request
        .capability_context
        .as_ref()
        .expect("validated above");
    if capability_context
        .permission_boundary
        .ai_write_policy
        .trim()
        != "proposal_only"
    {
        return Err(json_bad_request(
            "ai_proposal_denied",
            "AI 写入策略必须为 proposal_only 才能创建提案",
        ));
    }
    validate_runtime_parameter_mutation_target(&request.target)?;
    if request.old_value.is_null() {
        return Err(json_bad_request("bad_request", "AI 提案候选需要旧值"));
    }
    if request.new_value.is_null() {
        return Err(json_bad_request("bad_request", "AI 提案候选需要新值"));
    }
    validate_ai_model_identity(&request.model)?;
    validate_hash_identity(
        &request.prompt_hash,
        "prompt_hash",
        "AI proposal candidates",
    )?;
    validate_hash_identity(
        &request.evidence_hash,
        "evidence_hash",
        "AI proposal candidates",
    )?;
    let actor = request
        .actor
        .clone()
        .ok_or_else(|| json_bad_request("bad_request", "AI 提案候选需要指定操作者"))?;
    let actor = normalize_actor_identity(Some(actor));

    let source = load_runtime_ai_proposal_source_context(
        &state,
        &user_id,
        request.source_kind,
        &request.source_id,
    )
    .await?;
    let old_parameter_version =
        canonical_runtime_parameter_version(&request.target, &request.old_value)?;
    let proposed_parameter_version =
        canonical_runtime_parameter_version(&request.target, &request.new_value)?;
    let now_ms = current_time_ms();
    let static_check = ai_proposal_static_check_result(
        &request,
        &old_parameter_version,
        &proposed_parameter_version,
        source.event_count,
        now_ms,
    );
    let status = static_check.status;
    let ai_proposal_id = runtime_ai_proposal_record_id(
        &request,
        now_ms,
        source.event_count,
        &proposed_parameter_version,
    )?;
    let governance = runtime_ai_proposal_governance(
        &source.governance,
        old_parameter_version.clone(),
        proposed_parameter_version.clone(),
    );
    let source_evidence = RuntimeAiProposalSourceEvidence {
        source_kind: request.source_kind,
        source_id: request.source_id.clone(),
        graph_id: source.graph_id.clone(),
        event_count: source.event_count,
        evidence_hash: request.evidence_hash.clone(),
    };
    let mut record = RuntimeAiProposalRecord {
        ai_proposal_id,
        source_kind: request.source_kind,
        source_id: request.source_id.clone(),
        graph_id: source.graph_id.clone(),
        source_evidence,
        target: request.target.clone(),
        old_value: request.old_value.clone(),
        new_value: request.new_value.clone(),
        old_parameter_version,
        proposed_parameter_version,
        status,
        denial_reason: None,
        static_check,
        model: request.model.clone(),
        prompt_hash: request.prompt_hash.clone(),
        evidence_hash: request.evidence_hash.clone(),
        actor,
        reason: request.reason.trim().to_string(),
        governance,
        config_domain_binding: request.config_domain_binding.clone(),
        lifecycle: Vec::new(),
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    };

    let created_event =
        build_runtime_ai_proposal_event(&record, RuntimeAiProposalStatus::Submitted, now_ms);
    let static_event_time_ms = now_ms.saturating_add(1);
    let static_event = build_runtime_ai_proposal_event(&record, status, static_event_time_ms);
    record.lifecycle.push(ai_proposal_lifecycle_entry(
        RuntimeAiProposalStatus::Submitted,
        &created_event,
        source.current_sequence_no + 1,
        "AI proposal candidate submitted",
    ));
    record.lifecycle.push(ai_proposal_lifecycle_entry(
        status,
        &static_event,
        source.current_sequence_no + 2,
        record.static_check.message.clone(),
    ));
    let event_governance =
        governance_with_parameter_version(&source.governance, &record.old_parameter_version);
    if request.source_kind == RuntimeEvidenceSourceKind::Run {
        append_parameter_mutation_events_to_run(
            &state,
            &user_id,
            &request.source_id,
            vec![
                (created_event, event_governance.clone()),
                (static_event, event_governance),
            ],
            None,
        )
        .await?;
    }
    // Block 5 P1-4: 静态校验通过后自动创建审批单并触发沙箱验证
    if status == RuntimeAiProposalStatus::StaticCheckPassed {
        let proposal_id = record.ai_proposal_id.clone();
        // v2.1.0: 原子计数器后缀防毫秒级 approval_id 冲突
        static APPROVAL_COUNTER: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(0);
        let seq = APPROVAL_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let approval_id = format!("apr-{}-{}", now_ms, seq);
        let approval = RuntimeApprovalRecord {
            approval_id: approval_id.clone(),
            proposal_id: proposal_id.clone(),
            approval_level: RuntimeApprovalLevel::L1SingleReviewer,
            review_state: RuntimeApprovalReviewState::Pending,
            chain_stage_impact: vec!["intent".to_string(), "agent".to_string()],
            sandbox_report_url: None,
            rollback_plan: RuntimeRollbackPlan {
                method: "generation_rollback".to_string(),
                target_generation: 0,
                estimated_recovery_ms: 5000,
            },
            created_at_ms: now_ms,
            expires_at_ms: now_ms.saturating_add(24 * 3600 * 1000), // L1: 24h
            reviewers_required: 1,
            reviewers_assigned: vec!["pm-strategy-btc".to_string()],
            reviewers_approved: Vec::new(),
            reviewers_rejected: Vec::new(),
            lifecycle: vec![RuntimeApprovalLifecycleEntry {
                review_state: RuntimeApprovalReviewState::Pending,
                event_id: format!("event_apr_pending_{}", now_ms),
                sequence_no: 1,
                occurred_at_ms: now_ms,
                reason_code: "APPROVAL_CREATED".to_string(),
                message: "审批单自动创建".to_string(),
                actor_id: None,
            }],
        };
        persist_approval(&state.approval_store_dir, &approval)
            .await
            .map_err(io_error)?;
        state
            .approval_records
            .write()
            .await
            .insert(auth::scoped_key(&user_id, &approval_id), approval);

        // GP §9.4: 与 approve 路径保持 approval_records -> ai_proposals 锁顺序。
        persist_runtime_ai_proposal_transition(&state, &user_id, &record).await?;

        spawn_ai_proposal_sandbox_verification(state.clone(), proposal_id.clone());
    }
    if status != RuntimeAiProposalStatus::StaticCheckPassed {
        persist_runtime_ai_proposal_transition(&state, &user_id, &record).await?;
    }

    Ok(Json(record))
}

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
