use super::*;

#[path = "transition_lifecycle/activation_flow.rs"]
mod activation_flow;
#[path = "transition_lifecycle/activation_snapshot_side_effect.rs"]
mod activation_snapshot_side_effect;
#[path = "transition_lifecycle/boundary_safety.rs"]
mod boundary_safety;
#[path = "transition_lifecycle/rollback_flow.rs"]
mod rollback_flow;
#[path = "transition_lifecycle/transition_record_persistence.rs"]
mod transition_record_persistence;

pub(crate) use activation_flow::activate_runtime_parameter_mutation;
pub(crate) use rollback_flow::rollback_runtime_parameter_mutation;

use activation_snapshot_side_effect::auto_snapshot_on_activation;
use boundary_safety::{
    evaluate_runtime_parameter_mutation_safe_window, resolve_runtime_parameter_mutation_boundary,
};
use transition_record_persistence::{
    mutation_lifecycle_entry, persist_runtime_parameter_mutation_transition,
};

pub(super) fn validate_runtime_parameter_mutation_boundary(
    boundary: &RuntimeParameterMutationBoundary,
) -> Result<(), (StatusCode, String)> {
    boundary_safety::validate_runtime_parameter_mutation_boundary(boundary)
}

fn runtime_parameter_mutation_rollback_record_id(
    source_id: &str,
    rollback_of: &str,
    target: &RuntimeParameterMutationTarget,
    created_at_ms: u64,
    source_event_count: usize,
    proposed_parameter_version: &str,
) -> Result<String, (StatusCode, String)> {
    let digest = canonical_json_sha256_digest(&json!({
        "created_at_ms": created_at_ms,
        "rollback_of": rollback_of,
        "source_event_count": source_event_count,
        "source_id": source_id,
        "target": target,
        "proposed_parameter_version": proposed_parameter_version,
    }))
    .map_err(|error| internal_error(anyhow::anyhow!(error)))?;
    Ok(format!(
        "parameter_rollback_{}_{}",
        created_at_ms,
        &digest.value[..12]
    ))
}
