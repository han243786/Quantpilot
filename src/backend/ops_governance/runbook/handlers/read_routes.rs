use crate::*;

pub(super) async fn list_scenarios() -> Result<Json<Vec<RunbookScenario>>, (StatusCode, String)> {
    Ok(Json(super::build_default_runbook()))
}

pub(super) async fn get_scenario(
    Path(scenario_id): Path<String>,
) -> Result<Json<RunbookScenario>, (StatusCode, String)> {
    let scenarios = super::build_default_runbook();
    if let Some(scenario) = scenarios.into_iter().find(|s| s.scenario_id == scenario_id) {
        return Ok(Json(scenario));
    }
    Err(json_bad_request(
        "not_found",
        format!("故障场景 '{}' 不存在", scenario_id),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn list_scenarios_returns_default_catalog() {
        let Json(scenarios) = list_scenarios().await.expect("list scenarios");

        assert_eq!(scenarios.len(), 6);
    }

    #[tokio::test]
    async fn get_scenario_returns_matching_scenario() {
        let Json(scenario) = get_scenario(Path("data_source_unavailable".to_string()))
            .await
            .expect("get known scenario");

        assert_eq!(scenario.scenario_id, "data_source_unavailable");
    }

    #[tokio::test]
    async fn get_scenario_returns_not_found_for_unknown_id() {
        let (status, body) = get_scenario(Path("missing".to_string()))
            .await
            .expect_err("unknown scenario should fail");

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body.contains("\"error\":\"not_found\""));
    }
}
