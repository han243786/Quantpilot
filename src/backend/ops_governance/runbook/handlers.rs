use crate::*;
mod read_routes;
mod scenario_catalog;

// ── Runbook: 已知故障场景诊断与恢复手册 ──
// Block 5: 6 类故障场景，含诊断步骤、恢复命令、验证标准

pub(super) fn register_runbook_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/runbook", get(read_routes::list_scenarios))
        .route(
            "/api/v1/runbook/:scenario_id",
            get(read_routes::get_scenario),
        )
}

fn build_default_runbook() -> Vec<RunbookScenario> {
    scenario_catalog::build_default_runbook()
}
