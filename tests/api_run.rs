mod common;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test(flavor = "multi_thread")]
async fn run_endpoints_expose_service_level_contract_for_created_run() {
    let app = common::test_app("api_run_contract");
    let payload = common::sample_runtime_request();

    let response = app
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

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let started: Value = serde_json::from_slice(&body).unwrap();
    let run_id = started["run_id"].as_str().unwrap().to_string();

    assert_eq!(started["graph_id"], "graph_test");
    assert_eq!(started["compile_id"], "compile_test");
    assert_eq!(started["status"], "queued");
    assert!(started["event_count"].as_u64().unwrap() > 0);

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/runs")
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
    let runs: Value = serde_json::from_slice(&list_body).unwrap();
    let items = runs.as_array().unwrap();

    assert!(items.iter().any(|item| item["run_id"] == run_id));

    let detail_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/runs/{run_id}"))
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
    let detail: Value = serde_json::from_slice(&detail_body).unwrap();

    assert_eq!(detail["run_id"], run_id);
    assert_eq!(detail["graph_id"], "graph_test");
    assert_eq!(detail["compile_id"], "compile_test");
    assert_eq!(
        detail["event_count"].as_u64().unwrap(),
        detail["events"].as_array().unwrap().len() as u64
    );
    assert!(!detail["events"].as_array().unwrap().is_empty());
    assert_eq!(
        detail["runtime_diagnostics"]["source"],
        Value::String("runtime_events".to_string())
    );
    assert!(
        !detail["runtime_diagnostics"]["active_nodes"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let selected_node_id = detail["runtime_diagnostics"]["default_selected_node_id"]
        .as_str()
        .unwrap();
    assert!(
        detail["runtime_diagnostics"]["node_details"][selected_node_id]["event_count"]
            .as_u64()
            .unwrap()
            > 0
    );
    let node_details = detail["runtime_diagnostics"]["node_details"]
        .as_object()
        .expect("runtime diagnostics node_details should be an object");
    assert!(node_details.values().any(|node| {
        node["data_quality_rows"]
            .as_array()
            .map(|rows| !rows.is_empty())
            .unwrap_or(false)
    }));
    assert!(node_details.values().any(|node| {
        node["risk_detail_rows"]
            .as_array()
            .map(|rows| !rows.is_empty())
            .unwrap_or(false)
    }));
    assert!(node_details.values().any(|node| {
        node["order_detail_rows"]
            .as_array()
            .map(|rows| !rows.is_empty())
            .unwrap_or(false)
    }));
    let risk_node = node_details
        .values()
        .find(|node| {
            node["risk_detail_rows"]
                .as_array()
                .map(|rows| !rows.is_empty())
                .unwrap_or(false)
        })
        .expect("runtime diagnostics should include a structured risk node");
    assert!(risk_node["explanation_summary"]
        .as_str()
        .map(|value| !value.is_empty())
        .unwrap_or(false));
    assert!(risk_node["explanation_rows"]
        .as_array()
        .map(|rows| rows.iter().any(|row| row["key"] == "explanation_summary"))
        .unwrap_or(false));
    assert!(risk_node["risk_detail_rows"]
        .as_array()
        .map(|rows| {
            rows.iter().any(|row| row["key"] == "limit_triggered" || row["key"] == "status")
                && rows.iter().any(|row| row["key"] == "pre_risk.portfolio_net_exposure_ratio")
                && rows.iter().any(|row| row["key"] == "post_risk.portfolio_net_exposure_ratio")
        })
        .unwrap_or(false));

    let data_quality_node = node_details
        .values()
        .find(|node| {
            node["data_quality_rows"]
                .as_array()
                .map(|rows| !rows.is_empty())
                .unwrap_or(false)
        })
        .expect("runtime diagnostics should include a structured data quality node");
    assert!(data_quality_node["data_quality_rows"]
        .as_array()
        .map(|rows| {
            rows.iter().any(|row| row["key"] == "source_health")
                && rows.iter().any(|row| row["key"] == "freshness_ms")
        })
        .unwrap_or(false));

    let order_node = node_details
        .values()
        .find(|node| {
            node["order_detail_rows"]
                .as_array()
                .map(|rows| !rows.is_empty())
                .unwrap_or(false)
        })
        .expect("runtime diagnostics should include a structured order node");
    assert!(order_node["explanation_summary"]
        .as_str()
        .map(|value| !value.is_empty())
        .unwrap_or(false));
    assert!(order_node["explanation_rows"]
        .as_array()
        .map(|rows| rows.iter().any(|row| row["key"] == "explanation_summary"))
        .unwrap_or(false));
    assert!(order_node["order_detail_rows"]
        .as_array()
        .map(|rows| {
            rows.iter().any(|row| {
                row["key"] == "order_id"
                    || row["key"] == "lifecycle_stage"
                    || row["key"] == "sizing_source"
            })
        })
        .unwrap_or(false));

    let status_response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/runs/{run_id}/status"))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(status_response.status(), StatusCode::OK);

    let status_body = to_bytes(status_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let status: Value = serde_json::from_slice(&status_body).unwrap();

    assert_eq!(status["run_id"], run_id);
    assert_eq!(status["graph_id"], "graph_test");
    assert_eq!(status["compile_id"], "compile_test");
    assert_eq!(status["event_count"], detail["event_count"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn run_endpoint_honors_runtime_targets_for_event_node_mapping() {
    let app = common::test_app("api_run_runtime_targets");
    let mut payload = common::sample_runtime_request();
    payload["runtime_targets"] = serde_json::json!({
        "source_to_node": {
            "data_data_1": "custom_data",
            "intent_intent_1": "custom_intent",
            "agent_agent_1": "custom_agent",
            "risk_risk_1": "custom_risk"
        },
        "runtime_node_id": "custom_runtime",
        "execution_node_id": "custom_execution"
    });

    let response = app
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

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let started: Value = serde_json::from_slice(&body).unwrap();
    let run_id = started["run_id"].as_str().unwrap().to_string();

    let detail_response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/runs/{run_id}"))
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
    let detail: Value = serde_json::from_slice(&detail_body).unwrap();
    let node_ids = detail["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|event| event["node_id"].as_str())
        .collect::<Vec<_>>();

    assert!(node_ids.contains(&"custom_data"));
    assert!(node_ids.contains(&"custom_intent"));
    assert!(node_ids.contains(&"custom_agent"));
    assert!(node_ids.contains(&"custom_risk"));
    assert!(node_ids.contains(&"custom_execution"));
    assert!(node_ids.contains(&"custom_runtime"));
}

#[tokio::test(flavor = "multi_thread")]
async fn run_replay_endpoint_exposes_paginated_ordered_timeline() {
    let app = common::test_app("api_run_replay");
    let payload = common::sample_runtime_request();

    let response = app
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

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let started: Value = serde_json::from_slice(&body).unwrap();
    let run_id = started["run_id"].as_str().unwrap();

    let replay_response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/runs/{run_id}/replay?limit=2"))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(replay_response.status(), StatusCode::OK);

    let replay_body = to_bytes(replay_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let replay: Value = serde_json::from_slice(&replay_body).unwrap();

    assert_eq!(replay["kind"], "run");
    assert_eq!(replay["record_id"], run_id);
    assert_eq!(replay["graph_id"], "graph_test");
    assert_eq!(replay["cursor"], 0);
    assert_eq!(replay["limit"], 2);
    assert!(replay["checkpoints"].as_array().unwrap().len() > 0);
    assert!(replay["events"].as_array().unwrap().len() <= 2);
    let events = replay["events"].as_array().unwrap();
    assert!(!events.is_empty());
    assert_eq!(events[0]["sequence_no"], 1);
    for window in events.windows(2) {
        let left = window[0]["sequence_no"].as_u64().unwrap();
        let right = window[1]["sequence_no"].as_u64().unwrap();
        assert_eq!(right, left + 1);
    }
    if replay["total_events"].as_u64().unwrap() > 2 {
        assert_eq!(replay["next_cursor"], 2);
    }
}
