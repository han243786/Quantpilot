use axum::{
    routing::{get, post},
    Router,
};

use crate::{runtime as runtime_handlers, AppState};

pub const MODULE_ID: &str = "backend.runtime.routes.mutation";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/runtime/mutations",
            get(runtime_handlers::list_runtime_parameter_mutations)
                .post(runtime_handlers::create_runtime_parameter_mutation),
        )
        .route(
            "/api/runtime/mutations/:proposal_id",
            get(runtime_handlers::get_runtime_parameter_mutation_detail),
        )
        .route(
            "/api/runtime/mutations/:proposal_id/activate",
            post(runtime_handlers::activate_runtime_parameter_mutation),
        )
        .route(
            "/api/runtime/mutations/:proposal_id/rollback",
            post(runtime_handlers::rollback_runtime_parameter_mutation),
        )
        .route(
            "/api/runtime/ai-proposals",
            get(runtime_handlers::list_runtime_ai_proposals)
                .post(runtime_handlers::create_runtime_ai_proposal),
        )
        .route(
            "/api/runtime/ai-proposals/:ai_proposal_id",
            get(runtime_handlers::get_runtime_ai_proposal_detail),
        )
        .route(
            "/api/v1/ai/approvals",
            get(runtime_handlers::list_runtime_approvals),
        )
        .route(
            "/api/v1/ai/approvals/:approval_id",
            get(runtime_handlers::get_runtime_approval_detail),
        )
        .route(
            "/api/v1/ai/proposals/:proposal_id/approve",
            post(runtime_handlers::approve_ai_proposal),
        )
        .route(
            "/api/v1/ai/proposals/:proposal_id/reject",
            post(runtime_handlers::reject_ai_proposal),
        )
        .route(
            "/api/v1/ai/proposals/:proposal_id/claim",
            post(runtime_handlers::claim_ai_proposal_review),
        )
}
