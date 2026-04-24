mod common;

use axum::{
    body::{to_bytes, Body},
    http::{header::CONTENT_TYPE, Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test(flavor = "multi_thread")]
async fn run_events_endpoint_streams_sse_frames_for_completed_run() {
    let app = common::test_app("api_sse_contract");
    let payload = common::sample_runtime_request();

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/test-run")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::OK);

    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let started: Value = serde_json::from_slice(&create_body).unwrap();
    let run_id = started["run_id"].as_str().unwrap().to_string();

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/runs/{run_id}/events"))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(content_type.starts_with("text/event-stream"));

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let stream_text = String::from_utf8(body.to_vec()).unwrap();

    assert!(stream_text.contains("event: run_started"));
    assert!(stream_text.contains("event: runtime_event"));
    assert!(stream_text.contains("event: account"));
    assert!(stream_text.contains("event: run_completed"));
    assert!(stream_text.contains(&format!("\"run_id\":\"{run_id}\"")));
}
