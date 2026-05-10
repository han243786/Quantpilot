mod common;
include!("common/re_exports.rs");

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use std::{fs, thread, time::Duration};
use tower::ServiceExt;

fn field_names(value: &Value) -> Vec<String> {
    let mut fields: Vec<String> = value
        .as_object()
        .expect("snapshot source should be a JSON object")
        .keys()
        .cloned()
        .collect();
    fields.sort();
    fields
}

fn first_array_item<'a>(value: &'a Value, field: &str) -> &'a Value {
    value[field]
        .as_array()
        .and_then(|items| items.first())
        .unwrap_or_else(|| panic!("{field} should contain at least one item"))
}

async fn request_json(
    app: axum::Router,
    method: &str,
    uri: impl Into<String>,
    body: Option<Value>,
) -> Value {
    let request = Request::builder()
        .uri(uri.into())
        .method(method)
        .header("content-type", "application/json")
        .body(Body::from(
            body.map(|value| value.to_string()).unwrap_or_default(),
        ))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn build_evidence_contract_snapshot(
    detail: &Value,
    replay: &Value,
    report: &Value,
    export: &Value,
) -> Value {
    let timeline_item = first_array_item(detail, "timeline");
    let compact = &detail["compact_evidence"];
    let compact_entry = first_array_item(compact, "entries");
    let retained_key_index = &detail["retained_key_event_index"];
    let replay_timeline_item = first_array_item(replay, "timeline");
    let replay_event_item = first_array_item(replay, "events");
    let report_artifact = first_array_item(report, "artifacts");
    let export_section = first_array_item(export, "sections");

    json!({
        "timeline_item_fields": field_names(timeline_item),
        "timeline_governance_fields": field_names(&timeline_item["governance"]),
        "replay_window_fields": field_names(replay),
        "replay_filters_fields": field_names(&replay["filters"]),
        "replay_event_wrapper_fields": field_names(replay_event_item),
        "replay_timeline_item_fields": field_names(replay_timeline_item),
        "retained_key_event_index_fields": field_names(retained_key_index),
        "compact_evidence_fields": field_names(compact),
        "compact_entry_fields": field_names(compact_entry),
        "report_record_fields": field_names(report),
        "report_artifact_metadata_fields": field_names(report_artifact),
        "report_export_fields": field_names(export),
        "report_export_loading_strategy_fields": field_names(&export["loading_strategy"]),
        "report_export_section_fields": field_names(export_section),
        "stable_values": {
            "timeline_item_version": timeline_item["timeline_item_version"],
            "replay_kind": replay["kind"],
            "compact_policy_version": compact["policy_version"],
            "key_index_policy_version": retained_key_index["policy_version"],
            "report_status": report["status"],
            "report_source_kind": report["source_kind"],
            "report_export_schema_version": export["schema_version"],
            "report_loading_primary_source": export["loading_strategy"]["primary_source"]
        }
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_evidence_contract_snapshot_matches_fixture() {
    let app = common::test_app("api_evidence_contract_snapshot");
    let payload = common::sample_runtime_request();

    let started = request_json(app.clone(), "POST", "/api/runtime/test-run", Some(payload)).await;
    let run_id = started["run_id"].as_str().unwrap();

    let detail = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/runs/{run_id}"),
        None,
    )
    .await;
    let replay = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/runs/{run_id}/replay?sequence_cursor=1&limit=2"),
        None,
    )
    .await;
    let report = request_json(
        app.clone(),
        "POST",
        "/api/runtime/reports",
        Some(json!({
            "source_kind": "run",
            "source_id": run_id
        })),
    )
    .await;
    let report_id = report["report_id"].as_str().unwrap();
    let export = request_json(
        app,
        "GET",
        format!("/api/runtime/reports/{report_id}/export"),
        None,
    )
    .await;

    let actual = build_evidence_contract_snapshot(&detail, &replay, &report, &export);
    let expected: Value = serde_json::from_str(include_str!(
        "fixtures/runtime/evidence_contract_snapshot.json"
    ))
    .unwrap();
    assert_eq!(
        actual,
        expected,
        "evidence contract snapshot drifted; update the fixture only with an intentional contract change"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_evidence_health_tracks_metrics_and_cleanup_preserves_reports() {
    let (app, dirs) = common::test_app_with_dirs("api_evidence_health_cleanup");
    let payload = common::sample_runtime_request();

    let started = request_json(app.clone(), "POST", "/api/runtime/test-run", Some(payload)).await;
    let run_id = started["run_id"].as_str().unwrap();

    let _replay = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/runs/{run_id}/replay?sequence_cursor=1&limit=2"),
        None,
    )
    .await;
    let report = request_json(
        app.clone(),
        "POST",
        "/api/runtime/reports",
        Some(json!({
            "source_kind": "run",
            "source_id": run_id
        })),
    )
    .await;
    let report_id = report["report_id"].as_str().unwrap();

    let health = request_json(app.clone(), "GET", "/api/runtime/evidence/health", None).await;
    assert_eq!(health["status"], "ok");
    assert_eq!(health["metrics"]["report_generation_count"], 1);
    assert_eq!(health["metrics"]["report_generation_failure_count"], 0);
    assert_eq!(health["metrics"]["replay_page_count"], 1);
    assert!(
        health["metrics"]["compact_projection_source_event_count_total"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert_eq!(health["persisted_report_count"], 1);
    assert_eq!(health["report_status_counts"]["ready"], 1);
    assert_eq!(
        health["cleanup_policy"]["policy_version"],
        "quantpilot/evidence-cleanup/v1"
    );
    assert_eq!(
        health["cleanup_policy"]["protects_persisted_report_records"],
        true
    );

    let report_store_dir = dirs.backtest_store_dir.parent().unwrap().join("reports");
    let transient_dir = report_store_dir.join("report-generation-tmp-orphan");
    let transient_file = report_store_dir.join("report-generation-partial-orphan");
    fs::create_dir_all(&transient_dir).unwrap();
    fs::write(transient_dir.join("partial.json"), "{}").unwrap();
    fs::write(&transient_file, "{}").unwrap();
    thread::sleep(Duration::from_millis(5));

    let cleanup = request_json(
        app.clone(),
        "POST",
        "/api/runtime/evidence/cleanup",
        Some(json!({ "max_age_ms": 0 })),
    )
    .await;
    assert_eq!(cleanup["removed_transient_generation_outputs"], 2);
    assert_eq!(cleanup["retained_report_records"], 1);
    assert!(!transient_dir.exists());
    assert!(!transient_file.exists());

    let reloaded_report = request_json(
        app,
        "GET",
        format!("/api/runtime/reports/{report_id}"),
        None,
    )
    .await;
    assert_eq!(reloaded_report["report_id"], report["report_id"]);
    assert_eq!(reloaded_report["status"], "ready");
}
