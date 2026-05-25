mod common;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use tower::ServiceExt;

async fn post_json(app: Router, uri: &str, payload: Value) -> (StatusCode, Value) {
    let response = app
        .oneshot(
            Request::builder()
                .uri(uri)
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .expect("request should be buildable"),
        )
        .await
        .expect("request should complete");
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let value = serde_json::from_slice(&body).expect("response should be valid json");
    (status, value)
}

#[tokio::test(flavor = "multi_thread")]
async fn auth_register_error_response_includes_stable_error_code() {
    let app = common::test_app("api_auth_register_error_code");

    let (status, body) = post_json(
        app,
        "/api/auth/register",
        json!({
            "username": "",
            "password": "password123"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "bad_request");
    assert_eq!(body["error_code"], "AUTH_REGISTER_FAILED");
    assert!(body["details"].as_array().unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn auth_login_error_response_includes_stable_error_code() {
    let app = common::test_app("api_auth_login_error_code");

    let (status, body) = post_json(
        app,
        "/api/auth/login",
        json!({
            "username": "missing-user",
            "password": "password123"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], "login_failed");
    assert_eq!(body["error_code"], "AUTH_INVALID_CREDENTIALS");
    assert!(body["details"].as_array().unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn auth_refresh_error_response_includes_stable_error_code() {
    let app = common::test_app("api_auth_refresh_error_code");

    let (status, body) = post_json(
        app,
        "/api/auth/refresh",
        json!({
            "access_token": "not-a-token",
            "refresh_token": "not-a-refresh-token"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], "token_invalid");
    assert_eq!(body["error_code"], "AUTH_TOKEN_INVALID");
    assert!(body["details"].as_array().unwrap().is_empty());
}
