use crate::*;
mod scenario_catalog;

// ── Runbook: 已知故障场景诊断与恢复手册 ──
// Block 5: 6 类故障场景，含诊断步骤、恢复命令、验证标准

pub(super) fn register_runbook_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/runbook", get(list_scenarios))
        .route("/api/v1/runbook/:scenario_id", get(get_scenario))
}

fn build_default_runbook() -> Vec<RunbookScenario> {
    scenario_catalog::build_default_runbook()
}

async fn list_scenarios() -> Result<Json<Vec<RunbookScenario>>, (StatusCode, String)> {
    Ok(Json(build_default_runbook()))
}

async fn get_scenario(
    Path(scenario_id): Path<String>,
) -> Result<Json<RunbookScenario>, (StatusCode, String)> {
    let scenarios = build_default_runbook();
    if let Some(scenario) = scenarios.into_iter().find(|s| s.scenario_id == scenario_id) {
        return Ok(Json(scenario));
    }
    Err(json_bad_request(
        "not_found",
        format!("故障场景 '{}' 不存在", scenario_id),
    ))
}
