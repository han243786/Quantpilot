mod common;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

async fn get_json(app: axum::Router, uri: &str) -> Value {
    let response = app
        .oneshot(
            Request::builder()
                .uri(uri)
                .method("GET")
                .body(Body::empty())
                .expect("request should be buildable"),
        )
        .await
        .expect("request should complete");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    serde_json::from_slice(&body).expect("response should be valid json")
}

#[tokio::test(flavor = "multi_thread")]
async fn v1_report_endpoints_return_minimal_contracts() {
    let app = common::test_app("api_v1_reports_smoke");

    let ops = get_json(app.clone(), "/api/v1/reports/ops/daily").await;
    assert_eq!(ops["report_type"], "ops");
    assert!(ops["generated_at"].as_str().is_some());
    assert!(ops["summary"].is_object());
    assert!(ops["data_health"].is_object());
    assert!(ops["runtime_health"].is_object());

    let audit = get_json(app.clone(), "/api/v1/reports/audit/weekly").await;
    assert_eq!(audit["report_type"], "audit");
    assert!(audit["generated_at"].as_str().is_some());
    assert!(audit["total_approvals"].is_number());
    assert!(audit["notable_incidents"].is_array());

    let research = get_json(app, "/api/v1/reports/research/monthly").await;
    assert_eq!(research["report_type"], "research");
    assert!(research["generated_at"].as_str().is_some());
    assert!(research["strategy_performance"].is_array());
    assert!(research["ai_proposal_effectiveness"].is_object());
}
