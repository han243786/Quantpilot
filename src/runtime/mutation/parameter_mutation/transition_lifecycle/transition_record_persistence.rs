use super::mutation_event_contract;
use crate::{
    auth, io_error, persist_runtime_parameter_mutation_record, AppState, FrontendRuntimeEvent,
    RuntimeParameterMutationLifecycleEntry, RuntimeParameterMutationRecord,
    RuntimeParameterMutationStatus,
};
use axum::http::StatusCode;

pub(super) fn mutation_lifecycle_entry(
    status: RuntimeParameterMutationStatus,
    event: &FrontendRuntimeEvent,
    sequence_no: u64,
    message: impl Into<String>,
) -> RuntimeParameterMutationLifecycleEntry {
    let (_, reason_code) = mutation_event_contract(status);
    RuntimeParameterMutationLifecycleEntry {
        status,
        event_id: event.event_id.clone(),
        sequence_no,
        occurred_at_ms: event.event_time_ms,
        reason_code: reason_code.to_string(),
        message: message.into(),
    }
}

pub(super) async fn persist_runtime_parameter_mutation_transition(
    state: &AppState,
    user_id: &auth::UserId,
    record: &RuntimeParameterMutationRecord,
) -> Result<(), (StatusCode, String)> {
    persist_runtime_parameter_mutation_record(state.mutation_store_dir.as_ref(), record)
        .await
        .map_err(io_error)?;
    state.parameter_mutations.write().await.insert(
        auth::scoped_key(user_id, &record.proposal_id),
        record.clone(),
    );
    Ok(())
}
