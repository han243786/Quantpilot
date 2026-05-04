mod common;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use std::fs;
use tower::ServiceExt;

fn assert_complete_event_envelopes(events: &[Value], record_id: &str, governance: &Value) {
    for (index, event) in events.iter().enumerate() {
        let envelope = &event["envelope"];
        assert_eq!(envelope["event_id"], event["event_id"]);
        assert_eq!(envelope["event_type"], event["event_type"]);
        assert_eq!(envelope["run_id"], record_id);
        assert_eq!(envelope["sequence_no"], Value::from(index as u64 + 1));
        assert_eq!(envelope["occurred_at_ms"], event["event_time_ms"]);
        assert_eq!(envelope["capability_hash"], governance["capability_hash"]);
        assert_eq!(
            envelope["deployment_revision"],
            governance["deployment_revision"]
        );

        for key in [
            "event_id",
            "event_type",
            "run_id",
            "stage",
            "strategy_version",
            "parameter_version",
            "deployment_revision",
            "capability_hash",
            "mode",
            "severity",
            "retention_class",
        ] {
            assert!(
                envelope[key]
                    .as_str()
                    .map(|value| !value.trim().is_empty())
                    .unwrap_or(false),
                "event {} has empty envelope field {key}",
                event["event_id"]
            );
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_write_rejects_missing_capability_context_without_creating_run() {
    let (app, _dirs) = common::test_app_with_dirs("api_run_capability_guard");
    let mut payload = common::sample_runtime_request();
    payload
        .as_object_mut()
        .unwrap()
        .remove("capability_context");

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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(error["error"], "capability_boundary_violation");
    assert_eq!(error["details"][0]["code"], "missing_capability_context");

    let list_response = app
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
    assert!(runs.as_array().unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn run_endpoints_expose_service_level_contract_for_created_run() {
    let (app, dirs) = common::test_app_with_dirs("api_run_contract");
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

    let save_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/runs/{run_id}/save"))
                .method("POST")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(save_response.status(), StatusCode::OK);
    let save_body = to_bytes(save_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let saved: Value = serde_json::from_slice(&save_body).unwrap();

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
    let detail_events = detail["events"].as_array().unwrap();
    let detail_timeline = detail["timeline"].as_array().unwrap();
    let retained_index = &detail["retained_key_event_index"];
    let retained_entries = retained_index["entries"].as_array().unwrap();
    let compact_evidence = &detail["compact_evidence"];
    let compact_entries = compact_evidence["entries"].as_array().unwrap();

    assert_eq!(detail["run_id"], run_id);
    assert_eq!(detail["graph_id"], "graph_test");
    assert_eq!(detail["compile_id"], "compile_test");
    assert_eq!(
        detail["event_count"].as_u64().unwrap(),
        detail_events.len() as u64
    );
    assert!(!detail_events.is_empty());
    assert_eq!(detail_timeline.len(), detail_events.len());
    assert_eq!(retained_index["index_version"], 1);
    assert_eq!(
        retained_index["policy_version"],
        "quantpilot/key-event-index/v1"
    );
    assert_eq!(retained_index["source_event_count"], detail["event_count"]);
    assert_eq!(
        retained_index["retained_event_count"].as_u64().unwrap() as usize,
        retained_entries.len()
    );
    assert_eq!(compact_evidence["projection_version"], 1);
    assert_eq!(
        compact_evidence["policy_version"],
        "quantpilot/evidence-compaction/v1"
    );
    assert_eq!(
        compact_evidence["source_event_count"],
        detail["event_count"]
    );
    assert_eq!(
        compact_evidence["retained_event_count"],
        retained_index["retained_event_count"]
    );
    assert_eq!(
        compact_evidence["dropped_event_count"].as_u64().unwrap()
            + compact_evidence["retained_event_count"].as_u64().unwrap(),
        detail["event_count"].as_u64().unwrap()
    );
    assert_eq!(compact_entries, retained_entries);
    assert_eq!(
        compact_evidence["governance"]["capability_hash"],
        detail["governance"]["capability_hash"]
    );
    assert!(!retained_entries.is_empty());
    for entry in retained_entries {
        let event_type = entry["event_type"].as_str().unwrap();
        assert!(
            entry["retention_class"] == "key"
                || matches!(
                    event_type,
                    "CapabilitySnapshotTaken" | "SecurityViolationDetected"
                ),
            "unexpected retained key index entry {event_type}"
        );
        assert!(detail_timeline
            .iter()
            .any(|item| item["event_id"] == entry["event_id"]));
    }
    assert_eq!(saved["governance"], detail["governance"]);
    assert_complete_event_envelopes(detail_events, &run_id, &detail["governance"]);
    assert_eq!(
        detail["governance"]["schema_version"],
        Value::String("quantpilot/runtime-governance/v1".to_string())
    );
    assert_eq!(detail["governance"]["governance_source"], "current_runtime");
    assert_eq!(
        detail["governance"]["permission_boundary"]["non_execution_order_access"],
        Value::String("deny".to_string())
    );
    assert_eq!(
        detail["governance"]["permission_boundary"]["live_execution_allowed"],
        Value::Bool(false)
    );
    let first_event = &detail_events[0];
    assert_eq!(first_event["event_type"], "CapabilitySnapshotTaken");
    assert_eq!(
        first_event["payload"]["capability_hash"],
        detail["governance"]["capability_hash"]
    );
    assert_eq!(
        first_event["payload"]["schema_version"],
        Value::String("quantpilot/capabilities-schema/v1".to_string())
    );
    assert_eq!(
        first_event["payload"]["permission_boundary_model_version"],
        detail["governance"]["permission_boundary"]["model_version"]
    );
    assert_eq!(
        first_event["payload"]["runtime_mode"],
        Value::String("paper".to_string())
    );
    assert_eq!(
        first_event["envelope"]["run_id"],
        Value::String(run_id.clone())
    );
    assert_eq!(first_event["envelope"]["sequence_no"], Value::from(1));
    assert_eq!(first_event["envelope"]["event_id"], first_event["event_id"]);
    assert_eq!(
        first_event["envelope"]["capability_hash"],
        detail["governance"]["capability_hash"]
    );
    assert_eq!(
        first_event["envelope"]["deployment_revision"],
        detail["governance"]["deployment_revision"]
    );
    assert_eq!(first_event["envelope"]["stage"], "system");
    assert_eq!(first_event["envelope"]["retention_class"], "key");
    assert_eq!(detail_timeline[0]["event_id"], first_event["event_id"]);
    assert_eq!(detail_timeline[0]["event_type"], first_event["event_type"]);
    assert_eq!(
        detail_timeline[0]["sequence_no"],
        first_event["envelope"]["sequence_no"]
    );
    assert_eq!(
        detail_timeline[0]["stage"],
        first_event["envelope"]["stage"]
    );
    assert_eq!(
        detail_timeline[0]["retention_class"],
        first_event["envelope"]["retention_class"]
    );
    assert_eq!(
        detail_timeline[0]["governance"]["capability_hash"],
        detail["governance"]["capability_hash"]
    );
    assert_eq!(
        detail["runtime_diagnostics"]["source"],
        Value::String("runtime_events".to_string())
    );
    assert!(!detail["runtime_diagnostics"]["active_nodes"]
        .as_array()
        .unwrap()
        .is_empty());
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
            rows.iter()
                .any(|row| row["key"] == "limit_triggered" || row["key"] == "status")
                && rows
                    .iter()
                    .any(|row| row["key"] == "pre_risk.portfolio_net_exposure_ratio")
                && rows
                    .iter()
                    .any(|row| row["key"] == "post_risk.portfolio_net_exposure_ratio")
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

    let reloaded_app = common::test_app_from_dirs(dirs);
    let reloaded_response = reloaded_app
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/runs/{run_id}"))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reloaded_response.status(), StatusCode::OK);
    let reloaded_body = to_bytes(reloaded_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let reloaded: Value = serde_json::from_slice(&reloaded_body).unwrap();
    let mut expected_loaded_governance = detail["governance"].clone();
    expected_loaded_governance["governance_source"] = Value::String("loaded_manifest".to_string());
    assert_eq!(reloaded["governance"], expected_loaded_governance);
    assert_eq!(reloaded["events"], detail["events"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_report_records_persist_governed_run_evidence_metadata() {
    let (app, dirs) = common::test_app_with_dirs("api_run_report_metadata");
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

    let report_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/reports")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "source_kind": "run",
                        "source_id": run_id,
                        "generation_policy": "quantpilot/report-policy/test"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(report_response.status(), StatusCode::OK);
    let report_body = to_bytes(report_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let report: Value = serde_json::from_slice(&report_body).unwrap();
    let report_id = report["report_id"].as_str().unwrap().to_string();

    assert_eq!(report["source_kind"], "run");
    assert_eq!(report["source_id"], run_id);
    assert_eq!(report["status"], "ready");
    assert_eq!(report["generation_policy"], "quantpilot/report-policy/test");
    assert!(report["source_sequence_range"]["from"].as_u64().unwrap() >= 1);
    assert!(
        report["source_sequence_range"]["to"].as_u64().unwrap()
            >= report["source_sequence_range"]["from"].as_u64().unwrap()
    );
    assert!(report["governance"]["capability_hash"]
        .as_str()
        .map(|value| !value.is_empty() && value != "unknown")
        .unwrap_or(false));
    assert_eq!(report["artifacts"][0]["kind"], "metadata");
    assert!(report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|artifact| artifact["kind"] == "evidence_report"));
    assert!(report.get("events").is_none());
    assert!(report.get("entries").is_none());

    let duplicate_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/reports")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "source_kind": "run",
                        "source_id": run_id,
                        "generation_policy": "quantpilot/report-policy/test"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(duplicate_response.status(), StatusCode::OK);
    let duplicate_body = to_bytes(duplicate_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let duplicate: Value = serde_json::from_slice(&duplicate_body).unwrap();
    assert_eq!(duplicate["report_id"], report["report_id"]);
    assert_eq!(duplicate["created_at_ms"], report["created_at_ms"]);
    assert_eq!(duplicate["artifacts"], report["artifacts"]);

    let export_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/reports/{report_id}/export"))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(export_response.status(), StatusCode::OK);
    let export_body = to_bytes(export_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let export: Value = serde_json::from_slice(&export_body).unwrap();
    assert_eq!(
        export["schema_version"],
        "quantpilot/evidence-report-artifact/v1"
    );
    assert_eq!(export["report_id"], report["report_id"]);
    assert_eq!(
        export["source_sequence_range"],
        report["source_sequence_range"]
    );
    assert_eq!(export["governance"], report["governance"]);
    assert!(export["evidence_digest"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    assert_eq!(
        export["loading_strategy"]["primary_source"],
        "compact_evidence"
    );
    assert!(export.get("events").is_none());
    assert!(export.get("entries").is_none());

    let save_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/runs/{run_id}/save"))
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
                .uri("/api/runtime/reports")
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
    let reports: Value = serde_json::from_slice(&list_body).unwrap();
    assert!(reports
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["report_id"] == report_id));

    let reloaded_app = common::test_app_from_dirs(dirs);
    let reloaded_response = reloaded_app
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/reports/{report_id}"))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reloaded_response.status(), StatusCode::OK);
    let reloaded_body = to_bytes(reloaded_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let reloaded: Value = serde_json::from_slice(&reloaded_body).unwrap();
    assert_eq!(reloaded["report_id"], report["report_id"]);
    assert_eq!(reloaded["source_id"], report["source_id"]);
    assert_eq!(
        reloaded["source_sequence_range"],
        report["source_sequence_range"]
    );
    assert_eq!(reloaded["governance"], report["governance"]);
    assert_eq!(reloaded["artifacts"], report["artifacts"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_report_detail_marks_ready_report_when_source_evidence_changes() {
    let (app, dirs) = common::test_app_with_dirs("api_run_report_source_changed");
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

    let save_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/runs/{run_id}/save"))
                .method("POST")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(save_response.status(), StatusCode::OK);

    let report_response = app
        .oneshot(
            Request::builder()
                .uri("/api/runtime/reports")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "source_kind": "run",
                        "source_id": run_id
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(report_response.status(), StatusCode::OK);
    let report_body = to_bytes(report_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let report: Value = serde_json::from_slice(&report_body).unwrap();
    let report_id = report["report_id"].as_str().unwrap().to_string();

    let run_path = dirs.run_store_dir.join(format!("{run_id}.json"));
    let mut stored: Value = serde_json::from_str(&fs::read_to_string(&run_path).unwrap()).unwrap();
    stored["events"].as_array_mut().unwrap().pop();
    fs::write(&run_path, serde_json::to_string_pretty(&stored).unwrap()).unwrap();

    let reloaded_app = common::test_app_from_dirs(dirs);
    let changed_response = reloaded_app
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/reports/{report_id}"))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(changed_response.status(), StatusCode::OK);
    let changed_body = to_bytes(changed_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let changed: Value = serde_json::from_slice(&changed_body).unwrap();
    assert_eq!(changed["status"], "source_changed");
    assert_eq!(changed["failure"]["reason_code"], "source_changed");
    assert_eq!(changed["failure"]["retry_eligible"], true);
    assert!(changed["artifacts"].as_array().unwrap().is_empty());
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
        .clone()
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

    assert_eq!(replay["kind"], "run");
    assert_eq!(replay["record_id"], run_id);
    assert_eq!(replay["graph_id"], "graph_test");
    assert_eq!(replay["cursor"], 0);
    assert_eq!(replay["sequence_cursor"], 1);
    assert_eq!(replay["limit"], 2);
    assert_eq!(replay["source_event_count"], detail["event_count"]);
    assert!(!replay["checkpoints"].as_array().unwrap().is_empty());
    assert!(replay["events"].as_array().unwrap().len() <= 2);
    let events = replay["events"].as_array().unwrap();
    let timeline = replay["timeline"].as_array().unwrap();
    let detail_timeline = detail["timeline"].as_array().unwrap();
    assert!(!events.is_empty());
    assert_eq!(timeline.len(), events.len());
    assert_eq!(events[0]["sequence_no"], 1);
    assert_eq!(events[0]["event"]["event_type"], "CapabilitySnapshotTaken");
    for (index, item) in timeline.iter().enumerate() {
        assert_eq!(item, &detail_timeline[index]);
        assert_eq!(item["event_id"], events[index]["event"]["event_id"]);
        assert_eq!(item["sequence_no"], events[index]["sequence_no"]);
    }
    let replay_events = events
        .iter()
        .map(|item| item["event"].clone())
        .collect::<Vec<_>>();
    assert_complete_event_envelopes(&replay_events, run_id, &detail["governance"]);
    for window in events.windows(2) {
        let left = window[0]["sequence_no"].as_u64().unwrap();
        let right = window[1]["sequence_no"].as_u64().unwrap();
        assert_eq!(right, left + 1);
    }
    if replay["total_events"].as_u64().unwrap() > 2 {
        assert_eq!(replay["next_cursor"], 2);
        assert_eq!(replay["next_sequence_cursor"], 3);
    }

    let key_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/runtime/runs/{run_id}/replay?limit=20&key_only=true"
                ))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(key_response.status(), StatusCode::OK);
    let key_body = to_bytes(key_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let key_replay: Value = serde_json::from_slice(&key_body).unwrap();
    assert_eq!(key_replay["filters"]["key_only"], true);
    let key_timeline = key_replay["timeline"].as_array().unwrap();
    assert!(!key_timeline.is_empty());
    for item in key_timeline {
        assert_eq!(item["retention_class"], "key");
    }
    for window in key_timeline.windows(2) {
        let left = window[0]["sequence_no"].as_u64().unwrap();
        let right = window[1]["sequence_no"].as_u64().unwrap();
        assert!(right > left);
    }

    let invalid_response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/runtime/runs/{run_id}/replay?sequence_cursor=999999999"
                ))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid_response.status(), StatusCode::BAD_REQUEST);
    let invalid_body = to_bytes(invalid_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let invalid: Value = serde_json::from_slice(&invalid_body).unwrap();
    assert_eq!(invalid["error"], "bad_replay_cursor");
}

#[tokio::test(flavor = "multi_thread")]
async fn legacy_run_record_without_governance_loads_with_safe_defaults() {
    let (app, dirs) = common::test_app_with_dirs("api_run_legacy_governance");
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

    let save_response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/runs/{run_id}/save"))
                .method("POST")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(save_response.status(), StatusCode::OK);

    let run_path = dirs.run_store_dir.join(format!("{run_id}.json"));
    let mut legacy: Value = serde_json::from_str(&fs::read_to_string(&run_path).unwrap()).unwrap();
    legacy.as_object_mut().unwrap().remove("governance");
    for event in legacy["events"].as_array_mut().unwrap() {
        event.as_object_mut().unwrap().remove("envelope");
    }
    fs::write(&run_path, serde_json::to_string_pretty(&legacy).unwrap()).unwrap();

    let reloaded_app = common::test_app_from_dirs(dirs);
    let detail_response = reloaded_app
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
    assert_eq!(detail["governance"]["capability_hash"], "unknown");
    assert_eq!(detail["governance"]["governance_source"], "legacy_default");
    assert_eq!(
        detail["governance"]["permission_boundary"]["live_execution_allowed"],
        false
    );
    assert_complete_event_envelopes(
        detail["events"].as_array().unwrap(),
        &run_id,
        &detail["governance"],
    );
}
