mod common;
include!("common/re_exports.rs");

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

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

fn hash(ch: char) -> String {
    format!("sha256:{}", ch.to_string().repeat(64))
}

fn ai_proposal_payload(source_id: &str, request: &Value) -> Value {
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
        "model": {
            "provider": "openai",
            "model": "proposal-model",
            "model_version": "2026-04-29"
        },
        "prompt_hash": hash('a'),
        "evidence_hash": hash('b'),
        "actor": {
            "actor_id": "ai_assistant_1",
            "display_name": "AI Assistant 1"
        },
        "reason": "Reduce max position before a volatile replay window.",
        "capability_context": request["capability_context"].clone()
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_ai_proposal_creates_static_checked_record_and_key_events() {
    let app = common::test_app("api_ai_proposal_creates_candidate");
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
        "/api/runtime/ai-proposals",
        Some(ai_proposal_payload(run_id, &request)),
    )
    .await;
    let ai_proposal_id = proposal["ai_proposal_id"].as_str().unwrap();
    assert_eq!(proposal["status"], "static_check_passed");
    assert_eq!(proposal["source_kind"], "run");
    assert_eq!(proposal["source_id"], run_id);
    assert_eq!(proposal["target"]["module_key"], "builtin.risk.global");
    assert_eq!(proposal["model"]["model"], "proposal-model");
    assert_eq!(proposal["prompt_hash"], hash('a'));
    assert_eq!(proposal["evidence_hash"], hash('b'));
    assert_eq!(proposal["governance"]["ai_write_policy"], "proposal_only");
    assert!(proposal["old_parameter_version"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    assert!(proposal["proposed_parameter_version"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    assert_eq!(proposal["lifecycle"].as_array().unwrap().len(), 2);

    let detail = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/ai-proposals/{ai_proposal_id}"),
        None,
    )
    .await;
    assert_eq!(detail["ai_proposal_id"], proposal["ai_proposal_id"]);

    let list = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/ai-proposals?source_kind=run&source_id={run_id}&status=static_check_passed"),
        None,
    )
    .await;
    assert_eq!(list.as_array().unwrap().len(), 1);

    let run_detail = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/runs/{run_id}"),
        None,
    )
    .await;
    let ai_events: Vec<&Value> = run_detail["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| {
            matches!(
                event["event_type"].as_str(),
                Some("AIProposalCreated" | "AIProposalStaticCheckPassed")
            )
        })
        .collect();
    assert_eq!(ai_events.len(), 2);
    for event in ai_events {
        assert_eq!(
            event["payload"]["ai_proposal_id"],
            proposal["ai_proposal_id"]
        );
        assert_eq!(event["envelope"]["stage"], "system");
        assert_eq!(event["envelope"]["retention_class"], "key");
        assert_eq!(
            event["envelope"]["capability_hash"],
            run_detail["governance"]["capability_hash"]
        );
    }

    let replay = request_json(
        app,
        "GET",
        format!("/api/runtime/runs/{run_id}/replay?key_only=true"),
        None,
    )
    .await;
    let replay_ai_events: Vec<String> = replay["timeline"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|event| event["event_type"].as_str())
        .filter(|event_type| event_type.starts_with("AIProposal"))
        .map(str::to_string)
        .collect();
    assert_eq!(
        replay_ai_events,
        vec!["AIProposalCreated", "AIProposalStaticCheckPassed"]
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_ai_proposal_denies_missing_capability_without_mutation_ledger() {
    let app = common::test_app("api_ai_proposal_denies_missing_capability");
    let request = common::sample_runtime_request();
    let started = request_json(
        app.clone(),
        "POST",
        "/api/runtime/test-run",
        Some(request.clone()),
    )
    .await;
    let run_id = started["run_id"].as_str().unwrap();
    let mut payload = ai_proposal_payload(run_id, &request);
    payload
        .as_object_mut()
        .unwrap()
        .remove("capability_context");

    let status = request_status(
        app.clone(),
        "POST",
        "/api/runtime/ai-proposals",
        Some(payload),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let ai_list = request_json(
        app.clone(),
        "GET",
        format!("/api/runtime/ai-proposals?source_kind=run&source_id={run_id}"),
        None,
    )
    .await;
    assert_eq!(ai_list.as_array().unwrap().len(), 0);
    let mutation_list = request_json(
        app,
        "GET",
        format!("/api/runtime/mutations?source_kind=run&source_id={run_id}"),
        None,
    )
    .await;
    assert_eq!(mutation_list["data"].as_array().unwrap().len(), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_ai_proposal_static_check_failed_candidate_is_auditable() {
    let app = common::test_app("api_ai_proposal_static_check_failed");
    let request = common::sample_runtime_request();
    let started = request_json(
        app.clone(),
        "POST",
        "/api/runtime/test-run",
        Some(request.clone()),
    )
    .await;
    let run_id = started["run_id"].as_str().unwrap();
    let mut payload = ai_proposal_payload(run_id, &request);
    payload["new_value"] = payload["old_value"].clone();

    let proposal = request_json(
        app.clone(),
        "POST",
        "/api/runtime/ai-proposals",
        Some(payload),
    )
    .await;
    assert_eq!(proposal["status"], "static_check_failed");
    assert_eq!(
        proposal["static_check"]["details"][0]["code"],
        "noop_parameter_version"
    );

    let run_detail = request_json(app, "GET", format!("/api/runtime/runs/{run_id}"), None).await;
    let failed_event = run_detail["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["event_type"] == "AIProposalStaticCheckFailed")
        .expect("static check failure should be appended as governed evidence");
    assert_eq!(failed_event["envelope"]["retention_class"], "key");
    assert_eq!(
        failed_event["payload"]["static_check"]["status"],
        "static_check_failed"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_ai_proposal_rejects_missing_contract_fields() {
    let app = common::test_app("api_ai_proposal_rejects_missing_contract_fields");
    let request = common::sample_runtime_request();
    let started = request_json(
        app.clone(),
        "POST",
        "/api/runtime/test-run",
        Some(request.clone()),
    )
    .await;
    let run_id = started["run_id"].as_str().unwrap();
    let mut payload = ai_proposal_payload(run_id, &request);
    payload.as_object_mut().unwrap().remove("prompt_hash");

    let status = request_status(app, "POST", "/api/runtime/ai-proposals", Some(payload)).await;
    assert_ne!(status, StatusCode::OK);
}
