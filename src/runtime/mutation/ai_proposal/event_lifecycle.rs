use crate::{
    auth, io_error, persist_runtime_ai_proposal_record, AppState, FrontendRuntimeEvent,
    RuntimeAiProposalLifecycleEntry, RuntimeAiProposalRecord, RuntimeAiProposalStatus,
    RuntimeEventEnvelope,
};
use axum::http::StatusCode;
use serde_json::json;

fn ai_proposal_event_contract(status: RuntimeAiProposalStatus) -> (&'static str, &'static str) {
    match status {
        RuntimeAiProposalStatus::Submitted | RuntimeAiProposalStatus::Draft => {
            ("AIProposalCreated", "AI_PROPOSAL_CREATED")
        }
        RuntimeAiProposalStatus::Denied => ("AIProposalDenied", "AI_PROPOSAL_DENIED"),
        RuntimeAiProposalStatus::StaticCheckPassed => (
            "AIProposalStaticCheckPassed",
            "AI_PROPOSAL_STATIC_CHECK_PASSED",
        ),
        RuntimeAiProposalStatus::StaticCheckFailed => (
            "AIProposalStaticCheckFailed",
            "AI_PROPOSAL_STATIC_CHECK_FAILED",
        ),
        RuntimeAiProposalStatus::Expired => ("AIProposalDenied", "AI_PROPOSAL_EXPIRED"),
        RuntimeAiProposalStatus::Approved => ("AIProposalApproved", "AI_PROPOSAL_APPROVED"),
    }
}

pub(super) fn build_runtime_ai_proposal_event(
    record: &RuntimeAiProposalRecord,
    status: RuntimeAiProposalStatus,
    event_time_ms: u64,
) -> FrontendRuntimeEvent {
    let (event_type, reason_code) = ai_proposal_event_contract(status);
    FrontendRuntimeEvent {
        event_id: format!(
            "event_{}_{}_{}",
            record.ai_proposal_id, reason_code, event_time_ms
        ),
        event_type: event_type.to_string(),
        source_id: record.target.module_key.clone(),
        node_id: record.target.node_id.clone(),
        event_time_ms,
        severity: match status {
            RuntimeAiProposalStatus::Denied | RuntimeAiProposalStatus::StaticCheckFailed => {
                "Warn".to_string()
            }
            _ => "Info".to_string(),
        },
        summary: match status {
            RuntimeAiProposalStatus::Submitted | RuntimeAiProposalStatus::Draft => format!(
                "AI proposal candidate created for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeAiProposalStatus::Denied => format!(
                "AI proposal candidate denied for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeAiProposalStatus::StaticCheckPassed => format!(
                "AI proposal static check passed for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeAiProposalStatus::StaticCheckFailed => format!(
                "AI proposal static check failed for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeAiProposalStatus::Expired => format!(
                "AI proposal expired for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeAiProposalStatus::Approved => format!(
                "AI proposal approved for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
        },
        payload: json!({
            "ai_proposal_id": &record.ai_proposal_id,
            "status": status,
            "reason_code": reason_code,
            "source_kind": record.source_kind,
            "source_id": &record.source_id,
            "graph_id": &record.graph_id,
            "source_evidence": &record.source_evidence,
            "target": &record.target,
            "old_parameter_version": &record.old_parameter_version,
            "proposed_parameter_version": &record.proposed_parameter_version,
            "denial_reason": &record.denial_reason,
            "static_check": &record.static_check,
            "model": &record.model,
            "prompt_hash": &record.prompt_hash,
            "evidence_hash": &record.evidence_hash,
            "actor": &record.actor,
            "reason": &record.reason,
            "governance": &record.governance,
            "config_domain_binding": &record.config_domain_binding,
        }),
        envelope: RuntimeEventEnvelope::default(),
    }
}

pub(super) fn ai_proposal_lifecycle_entry(
    status: RuntimeAiProposalStatus,
    event: &FrontendRuntimeEvent,
    sequence_no: u64,
    message: impl Into<String>,
) -> RuntimeAiProposalLifecycleEntry {
    let (_, reason_code) = ai_proposal_event_contract(status);
    RuntimeAiProposalLifecycleEntry {
        status,
        event_id: event.event_id.clone(),
        sequence_no,
        occurred_at_ms: event.event_time_ms,
        reason_code: reason_code.to_string(),
        message: message.into(),
    }
}

pub(super) async fn persist_runtime_ai_proposal_transition(
    state: &AppState,
    user_id: &auth::UserId,
    record: &RuntimeAiProposalRecord,
) -> Result<(), (StatusCode, String)> {
    persist_runtime_ai_proposal_record(state.ai_proposal_store_dir.as_ref(), record)
        .await
        .map_err(io_error)?;
    state.ai_proposals.write().await.insert(
        auth::scoped_key(user_id, &record.ai_proposal_id),
        record.clone(),
    );
    Ok(())
}
