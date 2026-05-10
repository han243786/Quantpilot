mod common;
include!("common/re_exports.rs");

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test(flavor = "multi_thread")]
async fn experiment_endpoints_expose_parameter_grid_and_variant_summaries() {
    let app = common::test_app("api_experiments_contract");
    let mut request = common::sample_runtime_request();
    request["backtest_options"] = serde_json::json!({
        "replay_source": "deterministic_mock"
    });
    request["experiment_name"] = Value::String("Execution assumptions sweep".to_string());
    request["parameter_grid"] = serde_json::json!({
        "fee_bps": [5.0, 15.0],
        "slippage_bps": [5.0],
        "latency_ms": [0, 200]
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/experiments/backtest-sweep")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let detail: Value = serde_json::from_slice(&body).unwrap();
    let experiment_id = detail["experiment_id"].as_str().unwrap().to_string();
    assert_eq!(
        detail["definition"]["experiment_name"],
        "Execution assumptions sweep"
    );
    assert_eq!(detail["definition"]["replay_source"], "deterministic_mock");
    assert_eq!(detail["variants"].as_array().unwrap().len(), 4);
    assert!(detail["variants"]
        .as_array()
        .unwrap()
        .iter()
        .all(|variant| variant["backtest_id"].as_str().is_some()));

    let save_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/experiments/{experiment_id}/save"))
                .method("POST")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(save_response.status(), StatusCode::OK);

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/experiments")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let list: Value = serde_json::from_slice(&list_body).unwrap();
    let items = list.as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["experiment_id"], experiment_id);
    assert_eq!(items[0]["variant_count"], 4);
    assert!(items[0]["sweep_axes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "fee_bps"));
    assert!(items[0]["sweep_axes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "latency_ms"));
    assert!(items[0]["best_backtest_id"].as_str().is_some());

    let detail_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/experiments/{experiment_id}"))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail_response.status(), StatusCode::OK);
    let detail_body = to_bytes(detail_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let loaded_detail: Value = serde_json::from_slice(&detail_body).unwrap();
    assert_eq!(loaded_detail["experiment_id"], experiment_id);
    assert_eq!(loaded_detail["variants"].as_array().unwrap().len(), 4);
}

#[tokio::test(flavor = "multi_thread")]
async fn experiment_preview_can_be_discarded_before_save_only() {
    let app = common::test_app("api_experiments_discard");
    let mut request = common::sample_runtime_request();
    request["backtest_options"] = serde_json::json!({
        "replay_source": "deterministic_mock"
    });
    request["parameter_grid"] = serde_json::json!({
        "fee_bps": [5.0],
        "slippage_bps": [5.0],
        "latency_ms": [0]
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/experiments/backtest-sweep")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let detail: Value = serde_json::from_slice(&body).unwrap();
    let experiment_id = detail["experiment_id"].as_str().unwrap().to_string();
    let variant_backtest_id = detail["variants"][0]["backtest_id"]
        .as_str()
        .unwrap()
        .to_string();

    let discard_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/experiments/{experiment_id}"))
                .method("DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(discard_response.status(), StatusCode::OK);

    let detail_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/experiments/{experiment_id}"))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail_response.status(), StatusCode::NOT_FOUND);

    let variant_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/backtests/{variant_backtest_id}"))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(variant_response.status(), StatusCode::NOT_FOUND);

    let second_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/experiments/backtest-sweep")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second_response.status(), StatusCode::OK);
    let second_body = to_bytes(second_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let second_detail: Value = serde_json::from_slice(&second_body).unwrap();
    let saved_experiment_id = second_detail["experiment_id"].as_str().unwrap().to_string();

    let save_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/runtime/experiments/{saved_experiment_id}/save"
                ))
                .method("POST")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(save_response.status(), StatusCode::OK);

    let saved_discard_response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/experiments/{saved_experiment_id}"))
                .method("DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(saved_discard_response.status(), StatusCode::CONFLICT);
}
