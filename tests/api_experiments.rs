mod common;
include!("common/re_exports.rs");

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

const VALID_V4_EXPERIMENT_QS: &str = r#"
v4_strategy strategy.v4.experiment {
  venue paper-local
  mode paper_simulated
  require capability market

  machine data.market observation priority 8000 {
    state idle initial
    state ready
    state_group active idle ready
    memory last_signal_at: time nullable
    on market.tick from idle to ready emit bar_closed write last_signal_at
  }

  machine risk.guard decision priority 9500 {
    state idle initial
    state ready
    state_group active idle ready
    memory last_signal_at: time nullable
    on bar_closed from idle to ready emit risk.approved write last_signal_at
  }

  machine execution.router execution priority 4000 {
    state idle initial
    state ready
    state_group active idle ready
    memory last_signal_at: time nullable
    on risk.approved from idle to ready write last_signal_at
  }

  edge data.market -> risk.guard on bar_closed
  edge risk.guard -> execution.router on risk.approved
  risk_plane risk.guard priority 9000
}
"#;

fn paginated_items(value: &Value) -> &[Value] {
    value
        .get("data")
        .and_then(Value::as_array)
        .or_else(|| value.as_array())
        .expect("response should contain an item array")
}

#[tokio::test(flavor = "multi_thread")]
async fn experiment_endpoints_expose_parameter_grid_and_variant_summaries() {
    let app = common::test_app("api_experiments_contract");
    let mut request = common::sample_runtime_request();
    request["backtest_options"] = serde_json::json!({
        "replay_source": "deterministic_mock",
        "runtime_kind": "v4",
        "symbols": ["BTCUSDT"]
    });
    request["graph_json"]["metadata"]["runtime_kind"] = Value::String("v4".to_string());
    request["graph_json"]["metadata"]["artifacts"] = serde_json::json!({
        "quantscript": {
            "formal_source": VALID_V4_EXPERIMENT_QS
        },
        "v4_symbols": ["BTCUSDT"]
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

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let detail: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(status, StatusCode::OK, "{detail}");
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
    let items = paginated_items(&list);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["experiment_id"], experiment_id);
    assert_eq!(items[0]["saved"], true);
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
    assert_eq!(loaded_detail["saved"], true);
    assert_eq!(loaded_detail["variants"].as_array().unwrap().len(), 4);
}

#[tokio::test(flavor = "multi_thread")]
async fn experiment_sweep_rejects_parameter_grid_above_variant_limit() {
    let app = common::test_app("api_experiments_variant_limit");
    let mut request = common::sample_runtime_request();
    request["backtest_options"] = serde_json::json!({
        "replay_source": "deterministic_mock",
        "runtime_kind": "v4",
        "symbols": ["BTCUSDT"]
    });
    request["graph_json"]["metadata"]["runtime_kind"] = Value::String("v4".to_string());
    request["graph_json"]["metadata"]["artifacts"] = serde_json::json!({
        "quantscript": {
            "formal_source": VALID_V4_EXPERIMENT_QS
        },
        "v4_symbols": ["BTCUSDT"]
    });
    request["parameter_grid"] = serde_json::json!({
        "fee_bps": [1.0, 2.0, 3.0, 4.0],
        "slippage_bps": [1.0, 2.0, 3.0],
        "latency_ms": [0, 100, 200]
    });

    let response = app
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let payload: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload["error"], "bad_request");
    let message = payload["message"]
        .as_str()
        .expect("error response should include message");
    assert!(message.contains("参数扫描展开为 36 个变体"));
    assert!(message.contains("超出当前限制 27"));
}
