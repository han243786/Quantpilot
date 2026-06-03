use crate::*;

pub(super) fn register_runbook_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/runbook", get(super::list_scenarios))
        .route("/api/v1/runbook/:scenario_id", get(super::get_scenario))
}
