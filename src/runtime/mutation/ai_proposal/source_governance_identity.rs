use crate::{
    auth, canonical_json_sha256_digest, internal_error, load_backtest_record_from_state,
    load_run_record_from_state, AppState, CreateRuntimeAiProposalRequest,
    RuntimeAiProposalGovernance, RuntimeEvidenceSourceKind, RuntimeGovernanceSnapshot,
};
use axum::http::StatusCode;
use serde_json::json;

pub(super) struct RuntimeAiProposalSourceContext {
    pub(super) graph_id: String,
    pub(super) event_count: usize,
    pub(super) current_sequence_no: u64,
    pub(super) governance: RuntimeGovernanceSnapshot,
}

pub(super) async fn load_runtime_ai_proposal_source_context(
    state: &AppState,
    user_id: &auth::UserId,
    source_kind: RuntimeEvidenceSourceKind,
    source_id: &str,
) -> Result<RuntimeAiProposalSourceContext, (StatusCode, String)> {
    match source_kind {
        RuntimeEvidenceSourceKind::Run => {
            let source = load_run_record_from_state(state, user_id, source_id).await?;
            let current_sequence_no = source
                .events
                .last()
                .map(|event| event.envelope.sequence_no)
                .unwrap_or(source.events.len() as u64);
            Ok(RuntimeAiProposalSourceContext {
                graph_id: source.graph_id,
                event_count: source.events.len(),
                current_sequence_no,
                governance: source.governance,
            })
        }
        RuntimeEvidenceSourceKind::Backtest => {
            let source = load_backtest_record_from_state(state, user_id, source_id).await?;
            let current_sequence_no = source
                .events
                .last()
                .map(|event| event.envelope.sequence_no)
                .unwrap_or(source.events.len() as u64);
            Ok(RuntimeAiProposalSourceContext {
                graph_id: source.graph_id,
                event_count: source.events.len(),
                current_sequence_no,
                governance: source.governance,
            })
        }
    }
}

pub(super) fn runtime_ai_proposal_governance(
    source_governance: &RuntimeGovernanceSnapshot,
    old_parameter_version: String,
    proposed_parameter_version: String,
) -> RuntimeAiProposalGovernance {
    RuntimeAiProposalGovernance {
        capability_hash: source_governance.capability_hash.clone(),
        deployment_revision: source_governance.deployment_revision.clone(),
        strategy_version: source_governance.strategy_version.clone(),
        previous_parameter_version: old_parameter_version,
        proposed_parameter_version,
        permission_boundary_model_version: source_governance
            .permission_boundary
            .model_version
            .clone(),
        ai_write_policy: source_governance
            .permission_boundary
            .ai_write_policy
            .clone(),
    }
}

pub(super) fn runtime_ai_proposal_record_id(
    request: &CreateRuntimeAiProposalRequest,
    created_at_ms: u64,
    source_event_count: usize,
    proposed_parameter_version: &str,
) -> Result<String, (StatusCode, String)> {
    let digest = canonical_json_sha256_digest(&json!({
        "created_at_ms": created_at_ms,
        "source_event_count": source_event_count,
        "source_kind": request.source_kind,
        "source_id": &request.source_id,
        "target": &request.target,
        "model": &request.model,
        "prompt_hash": &request.prompt_hash,
        "evidence_hash": &request.evidence_hash,
        "proposed_parameter_version": proposed_parameter_version,
    }))
    .map_err(|error| internal_error(anyhow::anyhow!(error)))?;
    Ok(format!(
        "ai_proposal_{}_{}",
        created_at_ms,
        &digest.value[..12]
    ))
}
