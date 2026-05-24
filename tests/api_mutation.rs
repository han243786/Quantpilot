mod common;
include!("common/re_exports.rs");

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
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

fn first_lifecycle_entry(record: &Value) -> &Value {
    record["lifecycle"]
        .as_array()
        .and_then(|items| items.first())
        .expect("mutation lifecycle should contain at least one entry")
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

async fn request_status(
    app: axum::Router,
    method: &str,
    uri: impl Into<String>,
    body: Option<Value>,
) -> StatusCode {
    let request = Request::builder()
        .uri(uri.into())
        .method(method)
        .header("content-type", "application/json")
        .body(Body::from(
            body.map(|value| value.to_string()).unwrap_or_default(),
        ))
        .unwrap();
    app.oneshot(request).await.unwrap().status()
}

fn mutation_payload(source_id: &str, request: &Value) -> Value {
    json!({
        "source_kind": "run",
        "source_id": source_id,
        "target": {
            "node_id": "risk_risk_1",
            "module_key": "builtin.risk.global",
            "parameter_path": "max_position"
        },
        "old_value": 0.2,
        "new_value": 0.15,
        "activation_boundary": {
            "requested": "next_cycle_start"
        },
        "actor": {
            "actor_id": "operator_1",
            "display_name": "Operator 1"
        },
        "reason": "Reduce max position before a volatile replay window.",
        "capability_context": request["capability_context"].clone()
    })
}

fn build_mutation_contract_snapshot(
    proposal: &Value,
    activated: &Value,
    safe_denied: &Value,
    rollback: &Value,
    run_detail: &Value,
    replay: &Value,
    report: &Value,
    export: &Value,
    health: &Value,
) -> Value {
    let mutation_events: Vec<String> = run_detail["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|event| event["event_type"].as_str())
        .filter(|event_type| event_type.starts_with("ParameterMutation"))
        .map(|event_type| event_type.to_string())
        .collect();
    let replay_mutation_events: Vec<String> = replay["timeline"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["event_type"].as_str())
        .filter(|event_type| event_type.starts_with("ParameterMutation"))
        .map(|event_type| event_type.to_string())
        .collect();
    let report_sections: Vec<String> = export["sections"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|section| section["section_id"].as_str())
        .map(|section_id| section_id.to_string())
        .collect();

    json!({
        "proposal_record_fields": field_names(proposal),
        "activated_record_fields": field_names(activated),
        "safe_denied_record_fields": field_names(safe_denied),
        "rollback_record_fields": field_names(rollback),
        "target_fields": field_names(&proposal["target"]),
        "governance_fields": field_names(&proposal["governance"]),
        "activation_state_fields": field_names(&activated["activation_state"]),
        "safe_window_state_fields": field_names(&safe_denied["safe_window_state"]),
        "safe_window_snapshot_fields": field_names(&safe_denied["safe_window_state"]["snapshot"]),
        "lifecycle_entry_fields": field_names(first_lifecycle_entry(activated)),
        "health_metrics_fields": field_names(&health["metrics"]),
        "stable_values": {
            "proposal_status": proposal["status"],
            "activated_status": activated["status"],
            "safe_denied_status": safe_denied["status"],
            "rollback_status": rollback["status"],
            "safe_window_policy_version": safe_denied["safe_window_state"]["policy_version"],
            "safe_window_denied_reason": safe_denied["safe_window_state"]["reason_code"],
            "report_mutation_lifecycle_event_count": report["mutation_lifecycle_event_count"],
            "export_mutation_lifecycle_event_count": export["mutation_lifecycle_event_count"]
        },
        "mutation_event_types": mutation_events,
        "replay_mutation_event_types": replay_mutation_events,
        "report_sections": report_sections
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_parameter_mutation_requires_capability_context_without_ledger() {
    let app = common::test_app("api_mutation_requires_capability");
    let request = common::sample_runtime_request();
    let started = request_json(
        app.clone(),
        "POST",
        "/api/runtime/test-run",
        Some(request.clone()),
    )
    .await;
    let run_id = started["run_id"].as_str().unwrap();
    let mut payload = mutation_payload(run_id, &request);
    payload
        .as_object_mut()
        .unwrap()
        .remove("capability_context");

    let status = request_status(app.clone(), "POST", "/api/runtime/mutations", Some(payload)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let list = request_json(
        app,
        "GET",
        format!("/api/runtime/mutations?source_kind=run&source_id={run_id}"),
        None,
    )
    .await;
    assert_eq!(list["data"].as_array().unwrap().len(), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_parameter_mutation_creates_persisted_proposal_and_key_event() {
    let (app, dirs) = common::test_app_with_dirs("api_mutation_creates_proposal");
    let request = common::sample_runtime_request();
    let started = request_json(
        app.clone(),
        "POST",
        "/api/runtime/test-run",
        Some(request.clone()),
    )
    .await;
    let run_id = started["run_id"].as_str().unwrap();

    let proposal = request_json(
        app.clone(),
        "POST",
        "/api/runtime/mutations",
        Some(mutation_payload(run_id, &request)),
    )
    .await;
    let proposal_id = proposal["proposal_id"].as_str().unwrap();
    assert_eq!(proposal["status"], "proposed");
    assert_eq!(proposal["source_kind"], "run");
    assert_eq!(proposal["source_id"], run_id);
    assert_eq!(proposal["target"]["module_key"], "builtin.risk.global");
    assert!(proposal["old_parameter_version"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    assert!(proposal["proposed_parameter_version"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    assert_ne!(
        proposal["old_parameter_version"],
        proposal["proposed_parameter_version"]
    );
    assert_eq!(
        proposal["governance"]["permission_boundary_model_version"],
        "quantpilot/permission-boundary/v1"
    );

    let detail = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/mutations/{proposal_id}"),
        None,
    )
    .await;
    assert_eq!(detail["proposal_id"], proposal["proposal_id"]);

    let run_detail = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/runs/{run_id}"),
        None,
    )
    .await;
    let mutation_event = run_detail["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["event_type"] == "ParameterMutationProposed")
        .expect("mutation proposal event should be appended to run evidence");
    assert_eq!(
        mutation_event["payload"]["proposal_id"],
        proposal["proposal_id"]
    );
    assert_eq!(mutation_event["envelope"]["stage"], "system");
    assert_eq!(mutation_event["envelope"]["retention_class"], "key");
    assert_eq!(
        mutation_event["envelope"]["capability_hash"],
        run_detail["governance"]["capability_hash"]
    );

    let list = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/mutations?source_kind=run&source_id={run_id}"),
        None,
    )
    .await;
    assert_eq!(list["data"].as_array().unwrap().len(), 1);

    let reloaded = common::test_app_from_dirs(dirs);
    let reloaded_detail = request_json(
        reloaded,
        "GET",
        format!("/api/runtime/mutations/{proposal_id}"),
        None,
    )
    .await;
    assert_eq!(reloaded_detail["proposal_id"], proposal["proposal_id"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_parameter_mutation_rejects_noop_with_rejection_event() {
    let app = common::test_app("api_mutation_rejects_noop");
    let request = common::sample_runtime_request();
    let started = request_json(
        app.clone(),
        "POST",
        "/api/runtime/test-run",
        Some(request.clone()),
    )
    .await;
    let run_id = started["run_id"].as_str().unwrap();
    let mut payload = mutation_payload(run_id, &request);
    payload["new_value"] = payload["old_value"].clone();

    let proposal = request_json(app.clone(), "POST", "/api/runtime/mutations", Some(payload)).await;
    assert_eq!(proposal["status"], "rejected");
    assert!(proposal["rejection_reason"]
        .as_str()
        .unwrap()
        .contains("相同的规范参数版本"));

    let run_detail = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/runs/{run_id}"),
        None,
    )
    .await;
    assert!(run_detail["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|event| event["event_type"] == "ParameterMutationRejected"
            && event["envelope"]["retention_class"] == "key"));
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_parameter_mutation_parameter_versions_are_canonical() {
    let app = common::test_app("api_mutation_canonical_versions");
    let request = common::sample_runtime_request();
    let started = request_json(
        app.clone(),
        "POST",
        "/api/runtime/test-run",
        Some(request.clone()),
    )
    .await;
    let run_id = started["run_id"].as_str().unwrap();
    let mut first_payload = mutation_payload(run_id, &request);
    first_payload["old_value"] = json!({
        "max_position": 0.2,
        "min_action_interval_ms": 100
    });
    first_payload["new_value"] = json!({
        "max_position": 0.15,
        "min_action_interval_ms": 100
    });
    let mut second_payload = mutation_payload(run_id, &request);
    second_payload["old_value"] = json!({
        "min_action_interval_ms": 100,
        "max_position": 0.2
    });
    second_payload["new_value"] = json!({
        "min_action_interval_ms": 100,
        "max_position": 0.15
    });

    let first = request_json(
        app.clone(),
        "POST",
        "/api/runtime/mutations",
        Some(first_payload),
    )
    .await;
    let second = request_json(app, "POST", "/api/runtime/mutations", Some(second_payload)).await;

    assert_eq!(
        first["old_parameter_version"],
        second["old_parameter_version"]
    );
    assert_eq!(
        first["proposed_parameter_version"],
        second["proposed_parameter_version"]
    );
    assert_ne!(
        first["old_parameter_version"],
        first["proposed_parameter_version"]
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_parameter_mutation_activation_uses_explicit_boundary_versions_and_reports() {
    let app = common::test_app("api_mutation_activation_boundary");
    let request = common::sample_runtime_request();
    let started = request_json(
        app.clone(),
        "POST",
        "/api/runtime/test-run",
        Some(request.clone()),
    )
    .await;
    let run_id = started["run_id"].as_str().unwrap();
    let before_activation = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/runs/{run_id}"),
        None,
    )
    .await;
    let startup_parameter_version = before_activation["governance"]["parameter_version"]
        .as_str()
        .unwrap()
        .to_string();
    let proposal = request_json(
        app.clone(),
        "POST",
        "/api/runtime/mutations",
        Some(mutation_payload(run_id, &request)),
    )
    .await;
    let proposal_id = proposal["proposal_id"].as_str().unwrap();
    let proposed_parameter_version = proposal["proposed_parameter_version"].as_str().unwrap();

    let activated = request_json(
        app.clone(),
        "POST",
        format!("/api/runtime/mutations/{proposal_id}/activate"),
        Some(json!({
            "capability_context": request["capability_context"],
            "activation_boundary": {
                "requested": "next_cycle_start"
            }
        })),
    )
    .await;
    assert_eq!(activated["status"], "activated");
    assert_eq!(
        activated["activation_state"]["active_parameter_version"],
        proposal["proposed_parameter_version"]
    );
    assert_eq!(activated["lifecycle"].as_array().unwrap().len(), 2);

    let run_detail = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/runs/{run_id}"),
        None,
    )
    .await;
    assert_eq!(
        run_detail["governance"]["parameter_version"],
        proposal["proposed_parameter_version"]
    );
    let events = run_detail["events"].as_array().unwrap();
    let scheduled = events
        .iter()
        .find(|event| event["event_type"] == "ParameterMutationActivationScheduled")
        .expect("activation schedule event should be retained");
    let activated_event = events
        .iter()
        .find(|event| event["event_type"] == "ParameterMutationActivated")
        .expect("activation applied event should be retained");
    assert_eq!(
        scheduled["envelope"]["parameter_version"],
        proposal["old_parameter_version"]
    );
    assert_eq!(
        activated_event["envelope"]["parameter_version"],
        proposed_parameter_version
    );
    assert!(
        scheduled["envelope"]["sequence_no"].as_u64().unwrap()
            < activated_event["envelope"]["sequence_no"].as_u64().unwrap()
    );
    assert_ne!(
        scheduled["envelope"]["parameter_version"],
        startup_parameter_version
    );

    let replay = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/runs/{run_id}/replay?retention_class=key&limit=50"),
        None,
    )
    .await;
    assert!(replay["timeline"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["event_type"] == "ParameterMutationActivated"
            && item["governance"]["parameter_version"] == proposed_parameter_version));

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
    assert_eq!(report["mutation_lifecycle_event_count"], 3);
    assert_eq!(
        report["governance"]["parameter_version"],
        proposal["proposed_parameter_version"]
    );
    let report_id = report["report_id"].as_str().unwrap();
    let export = request_json(
        app,
        "GET",
        format!("/api/runtime/reports/{report_id}/export"),
        None,
    )
    .await;
    assert_eq!(export["mutation_lifecycle_event_count"], 3);
    assert!(export["sections"]
        .as_array()
        .unwrap()
        .iter()
        .any(|section| section["section_id"] == "mutation_lifecycle"));
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_parameter_mutation_manual_pause_stays_pending() {
    let app = common::test_app("api_mutation_manual_pause_pending");
    let request = common::sample_runtime_request();
    let started = request_json(
        app.clone(),
        "POST",
        "/api/runtime/test-run",
        Some(request.clone()),
    )
    .await;
    let run_id = started["run_id"].as_str().unwrap();
    let proposal = request_json(
        app.clone(),
        "POST",
        "/api/runtime/mutations",
        Some(mutation_payload(run_id, &request)),
    )
    .await;
    let proposal_id = proposal["proposal_id"].as_str().unwrap();

    let scheduled = request_json(
        app.clone(),
        "POST",
        format!("/api/runtime/mutations/{proposal_id}/activate"),
        Some(json!({
            "capability_context": request["capability_context"],
            "activation_boundary": {
                "requested": "manual_pause"
            }
        })),
    )
    .await;
    assert_eq!(scheduled["status"], "activation_scheduled");
    assert_eq!(
        scheduled["activation_state"]["resolved_sequence_no"],
        Value::Null
    );
    assert_eq!(
        scheduled["activation_state"]["active_parameter_version"],
        Value::Null
    );

    let run_detail = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/runs/{run_id}"),
        None,
    )
    .await;
    assert_ne!(
        run_detail["governance"]["parameter_version"],
        proposal["proposed_parameter_version"]
    );
    assert!(run_detail["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|event| event["event_type"] == "ParameterMutationActivationScheduled"));
    assert!(!run_detail["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|event| event["event_type"] == "ParameterMutationActivated"));
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_parameter_mutation_safe_window_denial_is_audited_without_activation() {
    let app = common::test_app("api_mutation_safe_window_denied");
    let request = common::sample_runtime_request();
    let started = request_json(
        app.clone(),
        "POST",
        "/api/runtime/test-run",
        Some(request.clone()),
    )
    .await;
    let run_id = started["run_id"].as_str().unwrap();
    let proposal = request_json(
        app.clone(),
        "POST",
        "/api/runtime/mutations",
        Some(mutation_payload(run_id, &request)),
    )
    .await;
    let proposal_id = proposal["proposal_id"].as_str().unwrap();

    let status = request_status(
        app.clone(),
        "POST",
        format!("/api/runtime/mutations/{proposal_id}/activate"),
        Some(json!({
            "capability_context": request["capability_context"],
            "activation_boundary": {
                "requested": "next_cycle_start"
            },
            "safe_window_context": {
                "policy_version": "quantpilot/mutation-safe-window/v1",
                "runtime_status": "running",
                "open_order_count": 2,
                "outstanding_risk_violation": false,
                "data_freshness_ms": 100,
                "portfolio_exposure_bps": 100,
                "cooldown_remaining_ms": 0
            }
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let detail = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/mutations/{proposal_id}"),
        None,
    )
    .await;
    assert_eq!(detail["status"], "safe_window_denied");
    assert_eq!(
        detail["safe_window_state"]["reason_code"],
        "SAFE_WINDOW_RUNTIME_ACTIVE"
    );

    let run_detail = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/runs/{run_id}"),
        None,
    )
    .await;
    assert_ne!(
        run_detail["governance"]["parameter_version"],
        proposal["proposed_parameter_version"]
    );
    assert!(run_detail["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(
            |event| event["event_type"] == "ParameterMutationSafeWindowDenied"
                && event["envelope"]["retention_class"] == "key"
        ));

    let health = request_json(app, "GET", "/api/runtime/evidence/health", None).await;
    assert_eq!(health["metrics"]["mutation_safe_window_denied_count"], 1);
    assert_eq!(health["metrics"]["mutation_activation_applied_count"], 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_parameter_mutation_rolls_back_to_ledger_backed_prior_version() {
    let app = common::test_app("api_mutation_rollback");
    let request = common::sample_runtime_request();
    let started = request_json(
        app.clone(),
        "POST",
        "/api/runtime/test-run",
        Some(request.clone()),
    )
    .await;
    let run_id = started["run_id"].as_str().unwrap();
    let proposal = request_json(
        app.clone(),
        "POST",
        "/api/runtime/mutations",
        Some(mutation_payload(run_id, &request)),
    )
    .await;
    let proposal_id = proposal["proposal_id"].as_str().unwrap();
    let activated = request_json(
        app.clone(),
        "POST",
        format!("/api/runtime/mutations/{proposal_id}/activate"),
        Some(json!({
            "capability_context": request["capability_context"],
            "activation_boundary": {
                "requested": "next_cycle_start"
            }
        })),
    )
    .await;
    assert_eq!(activated["status"], "activated");

    let unknown_status = request_status(
        app.clone(),
        "POST",
        format!("/api/runtime/mutations/{proposal_id}/rollback"),
        Some(json!({
            "capability_context": request["capability_context"],
            "target_parameter_version": "sha256:missing",
            "activation_boundary": {
                "requested": "next_cycle_start"
            }
        })),
    )
    .await;
    assert_eq!(unknown_status, StatusCode::BAD_REQUEST);

    let rollback = request_json(
        app.clone(),
        "POST",
        format!("/api/runtime/mutations/{proposal_id}/rollback"),
        Some(json!({
            "capability_context": request["capability_context"],
            "target_parameter_version": proposal["old_parameter_version"],
            "activation_boundary": {
                "requested": "next_cycle_start"
            },
            "reason": "Rollback after validation window."
        })),
    )
    .await;
    assert_eq!(rollback["status"], "rolled_back");
    assert_eq!(rollback["rollback_of"], proposal["proposal_id"]);
    assert_eq!(
        rollback["activation_state"]["active_parameter_version"],
        proposal["old_parameter_version"]
    );

    let run_detail = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/runs/{run_id}"),
        None,
    )
    .await;
    assert_eq!(
        run_detail["governance"]["parameter_version"],
        proposal["old_parameter_version"]
    );
    assert!(run_detail["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|event| event["event_type"] == "ParameterMutationRolledBack"
            && event["payload"]["rollback_of"] == proposal["proposal_id"]));

    let health = request_json(app, "GET", "/api/runtime/evidence/health", None).await;
    assert_eq!(health["metrics"]["mutation_proposal_created_count"], 1);
    assert_eq!(health["metrics"]["mutation_activation_scheduled_count"], 1);
    assert_eq!(health["metrics"]["mutation_activation_applied_count"], 1);
    assert_eq!(health["metrics"]["mutation_rollback_attempt_count"], 2);
    assert_eq!(health["metrics"]["mutation_rollback_scheduled_count"], 1);
    assert_eq!(health["metrics"]["mutation_rollback_applied_count"], 1);
    assert!(
        health["metrics"]["mutation_activation_latency_avg_ms"]
            .as_f64()
            .unwrap()
            >= 1.0
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_parameter_mutation_contract_snapshot_matches_fixture() {
    let app = common::test_app("api_mutation_contract_snapshot");
    let request = common::sample_runtime_request();
    let started = request_json(
        app.clone(),
        "POST",
        "/api/runtime/test-run",
        Some(request.clone()),
    )
    .await;
    let run_id = started["run_id"].as_str().unwrap();

    let safe_proposal = request_json(
        app.clone(),
        "POST",
        "/api/runtime/mutations",
        Some(mutation_payload(run_id, &request)),
    )
    .await;
    let safe_proposal_id = safe_proposal["proposal_id"].as_str().unwrap();
    let status = request_status(
        app.clone(),
        "POST",
        format!("/api/runtime/mutations/{safe_proposal_id}/activate"),
        Some(json!({
            "capability_context": request["capability_context"],
            "safe_window_context": {
                "policy_version": "quantpilot/mutation-safe-window/v1",
                "runtime_status": "paused",
                "open_order_count": 1,
                "outstanding_risk_violation": false,
                "data_freshness_ms": 100,
                "portfolio_exposure_bps": 100,
                "cooldown_remaining_ms": 0
            }
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let safe_denied = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/mutations/{safe_proposal_id}"),
        None,
    )
    .await;

    let active_proposal = request_json(
        app.clone(),
        "POST",
        "/api/runtime/mutations",
        Some(mutation_payload(run_id, &request)),
    )
    .await;
    let active_proposal_id = active_proposal["proposal_id"].as_str().unwrap();
    let activated = request_json(
        app.clone(),
        "POST",
        format!("/api/runtime/mutations/{active_proposal_id}/activate"),
        Some(json!({
            "capability_context": request["capability_context"],
            "activation_boundary": {
                "requested": "next_cycle_start"
            }
        })),
    )
    .await;
    let rollback = request_json(
        app.clone(),
        "POST",
        format!("/api/runtime/mutations/{active_proposal_id}/rollback"),
        Some(json!({
            "capability_context": request["capability_context"],
            "target_parameter_version": active_proposal["old_parameter_version"],
            "activation_boundary": {
                "requested": "next_cycle_start"
            }
        })),
    )
    .await;
    let run_detail = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/runs/{run_id}"),
        None,
    )
    .await;
    let replay = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/runs/{run_id}/replay?retention_class=key&limit=50"),
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
        app.clone(),
        "GET",
        format!("/api/runtime/reports/{report_id}/export"),
        None,
    )
    .await;
    let health = request_json(app, "GET", "/api/runtime/evidence/health", None).await;

    let actual = build_mutation_contract_snapshot(
        &active_proposal,
        &activated,
        &safe_denied,
        &rollback,
        &run_detail,
        &replay,
        &report,
        &export,
        &health,
    );
    let expected: Value = serde_json::from_str(include_str!(
        "fixtures/runtime/mutation_contract_snapshot.json"
    ))
    .unwrap();
    assert_eq!(
        actual,
        expected,
        "mutation contract snapshot drifted; update the fixture only with an intentional contract change"
    );
}
