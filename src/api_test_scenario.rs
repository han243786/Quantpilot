use super::*;
use axum::{routing::post, Json, Router};
use serde::Deserialize;

use super::test_runner::{TestReport, TestRunner, TestRunnerContext};

#[derive(Debug, Deserialize)]
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
            "source is empty — provide a valid quantscript strategy with @test directives".to_string(),
        ));
    }
    let ctx = TestRunnerContext::from_source(&request.source)
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, format!("parse error: {e}")))?;

    let mut runner = TestRunner::new();
    let report = runner
        .execute(&ctx)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("execution error: {e}"),
            )
        })?;

    Ok(Json(report))
}
