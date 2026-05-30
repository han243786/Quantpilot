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
async fn v1_ops_health_endpoints_return_minimal_contracts() {
    let app = common::test_app("api_v1_ops_health_smoke");

    let merge = get_json(app.clone(), "/api/v1/merge/records").await;
    assert!(merge["records"].is_array());
    assert!(merge["total_conflicts"].is_number());
    assert!(merge["total_suppressed"].is_number());

    let generations = get_json(app.clone(), "/api/v1/runtime/generations").await;
    assert!(generations["current_generation"].is_number());
    assert!(generations["history"].is_array());

    let storage = get_json(app, "/api/v1/storage/health").await;
    assert!(storage["total_storage_mb"].is_number());
    assert!(storage["layers"].is_array());
    assert!(storage["hot_layer_usage_ratio"].is_number());
    assert!(storage["disk_watermark_ratio"].is_number());
    assert!(storage["archive_enabled"].is_boolean());
    assert!(
        storage["layers"]
            .as_array()
            .map(|layers| layers.iter().any(|layer| layer["name"] == "runs"))
            .unwrap_or(false),
        "storage layers should include runs"
    );
}
