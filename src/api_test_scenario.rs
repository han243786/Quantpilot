use super::*;
use axum::{routing::post, Json, Router};
use serde::Deserialize;

use super::test_runner::{TestReport, TestRunner, TestRunnerContext};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunScenarioRequest {
    pub source: String,
}

pub(super) fn register_test_scenario_routes(router: Router<AppState>) -> Router<AppState> {
    router.route("/api/test/scenario/run", post(run_test_scenario))
}

async fn run_test_scenario(
    Json(request): Json<RunScenarioRequest>,
) -> Result<Json<TestReport>, (axum::http::StatusCode, String)> {
    if request.source.trim().is_empty() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "源码为空 —— 请提供包含 @test 指令的有效 QuantScript 策略".to_string(),
        ));
    }
    let ctx = TestRunnerContext::from_source(&request.source).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            format!("解析错误: {e}"),
        )
    })?;

    let mut runner = TestRunner::new();
    let report = runner.execute(&ctx).await.map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("执行错误: {e}"),
        )
    })?;

    Ok(Json(report))
}
