use crate::*;

pub(super) fn register_chaos_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/chaos/experiments", post(super::create_experiment))
        .route("/api/v1/chaos/experiments", get(super::list_experiments))
        .route(
            "/api/v1/chaos/experiments/:experiment_id",
            get(super::get_experiment),
        )
}
