use super::{
    approval_persistence::persist_approval,
    event_lifecycle::{
        ai_proposal_lifecycle_entry, build_runtime_ai_proposal_event,
        persist_runtime_ai_proposal_transition,
    },
    sandbox_trigger::spawn_ai_proposal_sandbox_verification,
    source_governance_identity::{
        load_runtime_ai_proposal_source_context, runtime_ai_proposal_governance,
        runtime_ai_proposal_record_id,
    },
    static_check::{
        ai_proposal_static_check_result, validate_ai_model_identity, validate_hash_identity,
    },
};
use crate::{
    auth, current_time_ms, io_error, json_bad_request, json_bad_request_with_details,
    normalize_actor_identity,
    runtime::{
        append_parameter_mutation_events_to_run, canonical_runtime_parameter_version,
        governance_with_parameter_version, validate_runtime_parameter_mutation_target,
    },
    validate_runtime_capability_guard, AppState, CreateRuntimeAiProposalRequest,
    RuntimeAiProposalRecord, RuntimeAiProposalSourceEvidence, RuntimeAiProposalStatus,
    RuntimeApprovalLevel, RuntimeApprovalLifecycleEntry, RuntimeApprovalRecord,
    RuntimeApprovalReviewState, RuntimeEvidenceSourceKind, RuntimeRollbackPlan,
};
use axum::{extract::State, http::StatusCode, Json};

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
            source_runner_evidence: None,
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
