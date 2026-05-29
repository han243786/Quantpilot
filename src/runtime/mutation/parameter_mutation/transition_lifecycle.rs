use super::*;

#[path = "transition_lifecycle/activation_flow.rs"]
mod activation_flow;
#[path = "transition_lifecycle/activation_snapshot_side_effect.rs"]
mod activation_snapshot_side_effect;
#[path = "transition_lifecycle/boundary_safety.rs"]
mod boundary_safety;
#[path = "transition_lifecycle/rollback_flow.rs"]
mod rollback_flow;
#[path = "transition_lifecycle/rollback_record_identity.rs"]
mod rollback_record_identity;
#[path = "transition_lifecycle/transition_record_persistence.rs"]
mod transition_record_persistence;

pub(crate) use activation_flow::activate_runtime_parameter_mutation;
pub(crate) use rollback_flow::rollback_runtime_parameter_mutation;

use activation_snapshot_side_effect::auto_snapshot_on_activation;
use boundary_safety::{
    evaluate_runtime_parameter_mutation_safe_window, resolve_runtime_parameter_mutation_boundary,
};
use rollback_record_identity::runtime_parameter_mutation_rollback_record_id;
use transition_record_persistence::{
    mutation_lifecycle_entry, persist_runtime_parameter_mutation_transition,
};

pub(super) fn validate_runtime_parameter_mutation_boundary(
    boundary: &RuntimeParameterMutationBoundary,
) -> Result<(), (StatusCode, String)> {
    boundary_safety::validate_runtime_parameter_mutation_boundary(boundary)
}
