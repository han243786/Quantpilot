use crate::*;
mod read_routes;
mod route_facade;
mod scenario_catalog;

// ── Runbook: 已知故障场景诊断与恢复手册 ──
// Block 5: 6 类故障场景，含诊断步骤、恢复命令、验证标准

pub(super) fn register_runbook_routes(router: Router<AppState>) -> Router<AppState> {
    route_facade::register_runbook_routes(router)
}

async fn list_scenarios() -> Result<Json<Vec<RunbookScenario>>, (StatusCode, String)> {
    read_routes::list_scenarios().await
}

async fn get_scenario(
    Path(scenario_id): Path<String>,
) -> Result<Json<RunbookScenario>, (StatusCode, String)> {
    read_routes::get_scenario(Path(scenario_id)).await
}

fn build_default_runbook() -> Vec<RunbookScenario> {
    scenario_catalog::build_default_runbook()
}
