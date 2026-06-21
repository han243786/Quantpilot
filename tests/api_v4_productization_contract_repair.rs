mod common;
include!("common/re_exports.rs");

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use std::sync::OnceLock;
use tower::ServiceExt;

const APPROVE_LIVE_ROUTE_ENV: &str = "QUANTPILOT_CONTRACT_REPAIR_APPROVE_LIVE_ROUTE";

static APPROVE_LIVE_ROUTE_ENV_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

fn approve_live_route_env_lock() -> &'static tokio::sync::Mutex<()> {
    APPROVE_LIVE_ROUTE_ENV_LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

struct TestEnvRestore {
    key: &'static str,
    old_value: Option<String>,
}

impl TestEnvRestore {
    fn set(key: &'static str, value: &str) -> Self {
        let old_value = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self { key, old_value }
    }
}

impl Drop for TestEnvRestore {
    fn drop(&mut self) {
        match &self.old_value {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}

async fn post_contract_repair_request(payload: Value) -> (StatusCode, Value) {
    let app = common::test_app("api_v4_productization_contract_repair");
    post_contract_repair_request_with_app(app, payload).await
}

async fn post_contract_repair_request_with_app(app: Router, payload: Value) -> (StatusCode, Value) {
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/runtime/v4/productization/contract-repairs/approval-requests")
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

async fn get_contract_repair_path(path: &str) -> (StatusCode, Value) {
    let app = common::test_app("api_v4_productization_contract_repair_read");
    get_contract_repair_path_with_app(app, path).await
}

async fn get_contract_repair_path_with_app(app: Router, path: &str) -> (StatusCode, Value) {
    let response = app
        .oneshot(
            Request::builder()
                .uri(path)
                .method("GET")
                .body(Body::empty())
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

async fn post_contract_repair_path_with_app(
    app: Router,
    path: &str,
    payload: Value,
) -> (StatusCode, Value) {
    let _guard = approve_live_route_env_lock().lock().await;
    post_contract_repair_path_with_app_unlocked(app, path, payload).await
}

async fn post_contract_repair_path_with_app_with_approve_live_route_env(
    app: Router,
    path: &str,
    payload: Value,
    configured_value: &str,
) -> (StatusCode, Value) {
    let _guard = approve_live_route_env_lock().lock().await;
    let _restore = TestEnvRestore::set(APPROVE_LIVE_ROUTE_ENV, configured_value);
    post_contract_repair_path_with_app_unlocked(app, path, payload).await
}

async fn post_contract_repair_path_with_app_unlocked(
    app: Router,
    path: &str,
    payload: Value,
) -> (StatusCode, Value) {
    let response = app
        .oneshot(
            Request::builder()
                .uri(path)
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

fn write_contract_source_fixture(
    dirs: &common::TestAppDirs,
    source_id: &str,
    version: &str,
    artifact_digest: &str,
) {
    let source_path = dirs.graph_store_dir.join(format!("{source_id}.json"));
    let source = json!({
        "schema_version": "quantpilot/machine-graph-contract/v1",
        "graph_id": source_id,
        "machines": [
            {
                "machine_id": "decision.dual_ma",
                "memory": []
            }
        ],
        "event_catalog": {
            "events": []
        },
        "metadata": {
            "graph_version": version,
            "artifact_hash": artifact_digest
        }
    });
    std::fs::write(source_path, source.to_string()).expect("contract source fixture should write");
}

#[tokio::test(flavor = "multi_thread")]
async fn contract_repair_approval_request_route_is_present_but_locked() {
    let (status, body) = post_contract_repair_request(json!({
        "status": "body_preview_only",
        "payload_kind": "v4_contract_repair_approval_request",
        "request_id": "approval-request:repair-draft:event_catalog/observation.bar_closed/fast_ma/bar-002",
        "target_path": "event_catalog/observation.bar_closed/fast_ma/bar-002",
        "target_kind": "event_instance_payload",
        "changed_fields": ["payload_value"],
        "patch_payload": {"payload_value": "101.25"},
        "contract_source_ref": {
            "source_kind": "v4_machine_graph_contract",
            "source_id": "graph:dual-ma",
            "version": "v4-test",
            "artifact_digest": "sha256:test-contract"
        },
        "mutation_enabled": false,
        "review_required": true
    }))
    .await;

    assert_eq!(status, StatusCode::LOCKED, "{body}");
    assert_eq!(body["status"], "blocked");
    assert_eq!(body["route_status"], "contract_mutation_api_disabled");
    assert_eq!(body["payload_kind"], "v4_contract_repair_approval_request");
    assert_eq!(body["target_kind"], "event_instance_payload");
    assert_eq!(body["changed_fields"], json!(["payload_value"]));
    assert_eq!(body["patch_payload"]["payload_value"], "101.25");
    assert_eq!(
        body["contract_source_ref"]["source_kind"],
        "v4_machine_graph_contract"
    );
    assert_eq!(body["contract_source_ref"]["source_id"], "graph:dual-ma");
    assert_eq!(body["contract_source_ref"]["version"], "v4-test");
    assert_eq!(
        body["approval_record_preview"]["patch_payload"]["payload_value"],
        "101.25"
    );
    assert_eq!(
        body["approval_record_preview"]["contract_source_ref"]["artifact_digest"],
        "sha256:test-contract"
    );
    assert_eq!(body["mutation_enabled"], false);
    assert_eq!(body["review_required"], true);
    assert_eq!(
        body["approval_record_preview"]["status"],
        "approval_request_persisted"
    );
    assert!(body["approval_record_preview"]["approval_id"]
        .as_str()
        .expect("approval id should be a string")
        .starts_with("contract-repair-apr-"));
    assert!(body["approval_record_preview"]["idempotency_key"]
        .as_str()
        .expect("idempotency key should be a string")
        .starts_with("sha256:"));
    assert_eq!(body["approval_record_preview"]["review_state"], "pending");
    assert_eq!(body["approval_record_preview"]["reviewers_required"], 1);
    assert_eq!(
        body["approval_record_preview"]["transient_review_status"],
        "not_claimed"
    );
    assert_eq!(body["approval_record_preview"]["would_persist"], true);
    assert_eq!(body["approval_record_preview"]["persistence_enabled"], true);
    assert!(body["blocked_reasons"]
        .as_array()
        .expect("blocked reasons should be an array")
        .contains(&json!("contract_mutation_api_disabled")));
}

#[tokio::test(flavor = "multi_thread")]
async fn contract_repair_approval_request_rejects_empty_changed_fields() {
    let (status, body) = post_contract_repair_request(json!({
        "status": "body_preview_only",
        "payload_kind": "v4_contract_repair_approval_request",
        "request_id": "approval-request:repair-draft:memory_schema/foo",
        "target_path": "memory_schema/foo",
        "target_kind": "memory_field",
        "changed_fields": [],
        "mutation_enabled": false,
        "review_required": true
    }))
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["error"], "contract_repair_changed_fields_missing");
}

#[tokio::test(flavor = "multi_thread")]
async fn contract_repair_approval_list_read_model_is_present_but_disabled() {
    let (status, body) = get_contract_repair_path(
        "/api/runtime/v4/productization/contract-repairs/approval-requests",
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["status"], "read_model_disabled");
    assert_eq!(body["route_status"], "approval_persistence_not_enabled");
    assert_eq!(body["record_source_status"], "record_source_disabled");
    assert_eq!(body["record_source_kind"], "none");
    assert_eq!(body["persisted_record_count"], 0);
    assert_eq!(body["preview_record_count"], 0);
    assert_eq!(body["persistence_enabled"], false);
    assert_eq!(body["mutation_enabled"], false);
    assert_eq!(body["records"], json!([]));
    assert!(body["blocked_reasons"]
        .as_array()
        .expect("blocked reasons should be an array")
        .contains(&json!("approval_persistence_not_enabled")));
    assert_eq!(
        body["decision_lock_summary"]["status"],
        "read_model_decision_lock_summary"
    );
    assert_eq!(body["decision_lock_summary"]["target_action"], "approve");
    assert_eq!(
        body["decision_lock_summary"]["response_status"],
        "read_model_disabled"
    );
    assert_eq!(
        body["decision_lock_summary"]["route_status"],
        "approval_persistence_not_enabled"
    );
    assert_eq!(
        body["decision_lock_summary"]["target_response_status"],
        "review_approve_executed"
    );
    assert_eq!(body["decision_lock_summary"]["expected_http_status"], 423);
    assert_eq!(
        body["decision_lock_summary"]["primary_blocked_reason"],
        "approval_persistence_not_enabled"
    );
    assert_eq!(
        body["decision_lock_summary"]["final_execution_locked"],
        true
    );
    assert_eq!(
        body["decision_lock_summary"]["would_execute_decision"],
        false
    );
    assert_eq!(
        body["decision_lock_summary"]["would_mutate_contract"],
        false
    );
    assert_eq!(body["decision_lock_summary"]["would_return_http_ok"], false);
    assert_eq!(body["decision_lock_summary"]["would_touch_disk"], false);
    assert!(body["decision_lock_summary"]["inherited_blocked_reasons"]
        .as_array()
        .expect("read model decision lock reasons should be an array")
        .contains(&json!("approval_persistence_not_enabled")));
}

#[tokio::test(flavor = "multi_thread")]
async fn contract_repair_approval_detail_read_model_is_present_but_disabled() {
    let (status, body) = get_contract_repair_path(
        "/api/runtime/v4/productization/contract-repairs/approval-requests/contract-repair-apr-preview",
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["status"], "detail_read_model_disabled");
    assert_eq!(body["approval_id"], "contract-repair-apr-preview");
    assert_eq!(body["route_status"], "approval_persistence_not_enabled");
    assert_eq!(body["record_source_status"], "record_source_disabled");
    assert_eq!(body["record_source_kind"], "none");
    assert_eq!(body["persisted_record_count"], 0);
    assert_eq!(body["preview_record_count"], 0);
    assert_eq!(body["persistence_enabled"], false);
    assert_eq!(body["mutation_enabled"], false);
    assert!(body.get("record").is_none());
    assert!(body["blocked_reasons"]
        .as_array()
        .expect("blocked reasons should be an array")
        .contains(&json!("approval_persistence_not_enabled")));
    assert_eq!(
        body["decision_lock_summary"]["status"],
        "read_model_decision_lock_summary"
    );
    assert_eq!(
        body["decision_lock_summary"]["response_status"],
        "detail_read_model_disabled"
    );
    assert_eq!(
        body["decision_lock_summary"]["route_status"],
        "approval_persistence_not_enabled"
    );
    assert_eq!(
        body["decision_lock_summary"]["primary_blocked_reason"],
        "approval_persistence_not_enabled"
    );
    assert_eq!(
        body["decision_lock_summary"]["final_execution_locked"],
        true
    );
    assert_eq!(
        body["decision_lock_summary"]["would_execute_decision"],
        false
    );
    assert_eq!(body["decision_lock_summary"]["would_touch_disk"], false);
}

#[tokio::test(flavor = "multi_thread")]
async fn contract_repair_approval_locked_post_populates_transient_preview_read_model() {
    let (app, dirs) = common::test_app_with_dirs("api_v4_productization_contract_repair_durable");
    let (post_status, post_body) = post_contract_repair_request_with_app(app.clone(), json!({
        "status": "body_preview_only",
        "payload_kind": "v4_contract_repair_approval_request",
        "request_id": "approval-request:repair-draft:memory_schema/decision.dual_ma/last_signal_at",
        "target_path": "memory_schema/decision.dual_ma/last_signal_at",
        "target_kind": "memory_field",
        "changed_fields": ["type_name"],
        "mutation_enabled": false,
        "review_required": true
    }))
    .await;
    let approval_id = post_body["approval_record_preview"]["approval_id"]
        .as_str()
        .expect("approval id should be present")
        .to_string();

    assert_eq!(post_status, StatusCode::LOCKED, "{post_body}");
    assert_eq!(
        post_body["approval_record_preview"]["status"],
        "approval_request_persisted"
    );
    assert_eq!(
        post_body["approval_record_preview"]["transient_review_status"],
        "not_claimed"
    );
    assert_eq!(
        post_body["approval_record_preview"]["persistence_enabled"],
        true
    );

    let (list_status, list_body) = get_contract_repair_path_with_app(
        app.clone(),
        "/api/runtime/v4/productization/contract-repairs/approval-requests",
    )
    .await;

    assert_eq!(list_status, StatusCode::OK, "{list_body}");
    assert_eq!(list_body["status"], "persisted_read_model");
    assert_eq!(list_body["route_status"], "approval_persistence_enabled");
    assert_eq!(list_body["record_source_status"], "durable_records_ready");
    assert_eq!(
        list_body["record_source_kind"],
        "contract_repair_approval_records"
    );
    assert_eq!(list_body["persisted_record_count"], 1);
    assert_eq!(list_body["preview_record_count"], 0);
    assert_eq!(list_body["persistence_enabled"], true);
    assert_eq!(list_body["mutation_enabled"], false);
    assert_eq!(list_body["records"][0]["approval_id"], approval_id);
    assert_eq!(
        list_body["decision_lock_summary"]["response_status"],
        "persisted_read_model"
    );
    assert_eq!(
        list_body["decision_lock_summary"]["route_status"],
        "approval_persistence_enabled"
    );
    assert_eq!(
        list_body["decision_lock_summary"]["primary_blocked_reason"],
        "contract_mutation_api_disabled"
    );
    assert_eq!(
        list_body["decision_lock_summary"]["persisted_record_count"],
        1
    );
    assert_eq!(
        list_body["decision_lock_summary"]["preview_record_count"],
        0
    );
    assert_eq!(
        list_body["decision_lock_summary"]["persistence_enabled"],
        true
    );
    assert_eq!(
        list_body["decision_lock_summary"]["mutation_enabled"],
        false
    );
    assert_eq!(
        list_body["decision_lock_summary"]["final_execution_locked"],
        true
    );
    assert_eq!(
        list_body["decision_lock_summary"]["would_mutate_contract"],
        false
    );

    let (detail_status, detail_body) = get_contract_repair_path_with_app(
        app,
        &format!("/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}"),
    )
    .await;

    assert_eq!(detail_status, StatusCode::OK, "{detail_body}");
    assert_eq!(detail_body["status"], "detail_persisted_record");
    assert_eq!(detail_body["record_source_status"], "durable_records_ready");
    assert_eq!(
        detail_body["record_source_kind"],
        "contract_repair_approval_records"
    );
    assert_eq!(detail_body["persisted_record_count"], 1);
    assert_eq!(detail_body["preview_record_count"], 0);
    assert_eq!(detail_body["persistence_enabled"], true);
    assert_eq!(detail_body["mutation_enabled"], false);
    assert_eq!(detail_body["record"]["approval_id"], approval_id);
    assert_eq!(
        detail_body["decision_lock_summary"]["response_status"],
        "detail_persisted_record"
    );
    assert_eq!(
        detail_body["decision_lock_summary"]["primary_blocked_reason"],
        "contract_mutation_api_disabled"
    );
    assert_eq!(
        detail_body["decision_lock_summary"]["final_execution_locked"],
        true
    );

    let restarted_app = common::test_app_from_dirs(dirs);
    let (restarted_detail_status, restarted_detail_body) = get_contract_repair_path_with_app(
        restarted_app.clone(),
        &format!("/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}"),
    )
    .await;

    assert_eq!(
        restarted_detail_status,
        StatusCode::OK,
        "{restarted_detail_body}"
    );
    assert_eq!(restarted_detail_body["status"], "detail_persisted_record");
    assert_eq!(
        restarted_detail_body["record_source_kind"],
        "contract_repair_approval_records"
    );
    assert_eq!(restarted_detail_body["record"]["approval_id"], approval_id);
    assert_eq!(restarted_detail_body["record"]["persistence_enabled"], true);
}

#[tokio::test(flavor = "multi_thread")]
async fn contract_repair_approval_review_intent_is_present_but_locked() {
    let (app, dirs) =
        common::test_app_with_dirs("api_v4_productization_contract_repair_review_intent");
    let (post_status, post_body) = post_contract_repair_request_with_app(app.clone(), json!({
        "status": "body_preview_only",
        "payload_kind": "v4_contract_repair_approval_request",
        "request_id": "approval-request:repair-draft:event_catalog/observation.bar_closed/fast_ma/bar-002",
        "target_path": "event_catalog/observation.bar_closed/fast_ma/bar-002",
        "target_kind": "event_instance_payload",
        "changed_fields": ["payload_value"],
        "mutation_enabled": false,
        "review_required": true
    }))
    .await;
    let approval_id = post_body["approval_record_preview"]["approval_id"]
        .as_str()
        .expect("approval id should be present")
        .to_string();

    assert_eq!(post_status, StatusCode::LOCKED, "{post_body}");

    let (review_status, review_body) = post_contract_repair_path_with_app(
        app.clone(),
        &format!(
            "/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}/review"
        ),
        json!({
            "action": "claim",
            "reviewer_id": "reviewer-a",
            "reason": "inspect transient preview",
            "review_enabled": false
        }),
    )
    .await;

    assert_eq!(review_status, StatusCode::LOCKED, "{review_body}");
    assert_eq!(review_body["status"], "blocked");
    assert_eq!(review_body["route_status"], "review_workflow_disabled");
    assert_eq!(review_body["approval_id"], approval_id);
    assert_eq!(review_body["action"], "claim");
    assert_eq!(review_body["reviewer_id"], "reviewer-a");
    assert_eq!(review_body["review_enabled"], false);
    assert_eq!(review_body["persistence_enabled"], false);
    assert_eq!(review_body["mutation_enabled"], false);
    assert_eq!(review_body["execution_gate"]["status"], "blocked");
    assert!(review_body["execution_gate"]["passed_gates"]
        .as_array()
        .expect("passed gates should be an array")
        .contains(&json!("transient_preview_exists")));
    assert!(review_body["execution_gate"]["passed_gates"]
        .as_array()
        .expect("passed gates should be an array")
        .contains(&json!("review_intent_valid")));
    assert!(review_body["execution_gate"]["passed_gates"]
        .as_array()
        .expect("passed gates should be an array")
        .contains(&json!("reviewer_identity_present")));
    assert!(review_body["execution_gate"]["passed_gates"]
        .as_array()
        .expect("passed gates should be an array")
        .contains(&json!("contract_repair_approval_store_ready")));
    assert!(review_body["execution_gate"]["passed_gates"]
        .as_array()
        .expect("passed gates should be an array")
        .contains(&json!("idempotency_precheck_passed")));
    assert!(review_body["execution_gate"]["blocked_gates"]
        .as_array()
        .expect("blocked gates should be an array")
        .contains(&json!("review_workflow_enabled")));
    assert!(review_body["execution_gate"]["blocked_gates"]
        .as_array()
        .expect("blocked gates should be an array")
        .contains(&json!("reviewer_identity_format_valid")));
    assert!(review_body["execution_gate"]["blocked_gates"]
        .as_array()
        .expect("blocked gates should be an array")
        .contains(&json!("reviewer_identity_matches_auth_subject")));
    assert!(review_body["execution_gate"]["blocked_gates"]
        .as_array()
        .expect("blocked gates should be an array")
        .contains(&json!("formal_reviewer_authorized")));
    assert_eq!(
        review_body["reviewer_authorization_precheck"]["status"],
        "authorization_precheck_denied"
    );
    assert_eq!(
        review_body["reviewer_authorization_precheck"]["policy_version"],
        "quantpilot/contract-repair-reviewer-role-policy/v1"
    );
    assert_eq!(
        review_body["reviewer_authorization_precheck"]["required_role"],
        "contract_repair_reviewer"
    );
    assert_eq!(
        review_body["reviewer_authorization_precheck"]["grant_source"],
        "not_configured"
    );
    assert_eq!(
        review_body["reviewer_authorization_precheck"]["reviewer_id"],
        "reviewer-a"
    );
    assert_eq!(
        review_body["reviewer_authorization_precheck"]["auth_subject"],
        "user:0"
    );
    assert_eq!(
        review_body["reviewer_authorization_precheck"]["identity_format_valid"],
        false
    );
    assert_eq!(
        review_body["reviewer_authorization_precheck"]["identity_matches_auth_subject"],
        false
    );
    assert_eq!(
        review_body["reviewer_authorization_precheck"]["role_policy_available"],
        true
    );
    assert_eq!(
        review_body["reviewer_authorization_precheck"]["authorized"],
        false
    );
    assert!(review_body["reviewer_authorization_precheck"]["blocked_by"]
        .as_array()
        .expect("authorization precheck blocked_by should be an array")
        .contains(&json!("formal_reviewer_role_grant_missing")));
    assert_eq!(
        review_body["execution_plan_preview"]["status"],
        "execution_plan_preview_only"
    );
    assert_eq!(
        review_body["execution_plan_preview"]["execution_enabled"],
        false
    );
    assert_eq!(review_body["execution_plan_preview"]["action"], "claim");
    assert_eq!(
        review_body["execution_plan_preview"]["target_review_state"],
        "under_review"
    );
    assert_eq!(
        review_body["execution_plan_preview"]["would_persist_approval_record"],
        false
    );
    assert_eq!(
        review_body["execution_plan_preview"]["would_mutate_contract"],
        false
    );
    assert_eq!(
        review_body["execution_plan_preview"]["would_emit_lifecycle_event"],
        false
    );
    assert!(review_body["execution_plan_preview"]["blocked_by"]
        .as_array()
        .expect("blocked by should be an array")
        .contains(&json!("approval_persistence_enabled")));
    assert_eq!(
        review_body["persistence_plan_preview"]["status"],
        "persistence_plan_preview_only"
    );
    assert_eq!(
        review_body["persistence_plan_preview"]["persistence_enabled"],
        false
    );
    assert_eq!(
        review_body["persistence_plan_preview"]["would_write_record"],
        false
    );
    assert_eq!(
        review_body["persistence_plan_preview"]["store_kind"],
        "contract_repair_approval_records"
    );
    assert_eq!(
        review_body["persistence_plan_preview"]["record_key"],
        approval_id
    );
    assert_eq!(
        review_body["persistence_plan_preview"]["idempotency_key"],
        post_body["approval_record_preview"]["idempotency_key"]
    );
    assert!(review_body["persistence_plan_preview"]["blocked_by"]
        .as_array()
        .expect("persistence blocked by should be an array")
        .contains(&json!("approval_persistence_enabled")));
    assert_eq!(
        review_body["persistence_path_preview"]["status"],
        "persistence_path_preview_only"
    );
    assert_eq!(
        review_body["persistence_path_preview"]["store_kind"],
        "contract_repair_approval_records"
    );
    assert_eq!(
        review_body["persistence_path_preview"]["record_key"],
        approval_id
    );
    assert_eq!(
        review_body["persistence_path_preview"]["path_segment"],
        approval_id
    );
    assert_eq!(
        review_body["persistence_path_preview"]["file_name"],
        format!("{approval_id}.json")
    );
    assert_eq!(
        review_body["persistence_path_preview"]["atomic_write_required"],
        true
    );
    assert_eq!(
        review_body["persistence_path_preview"]["would_touch_disk"],
        false
    );
    assert!(review_body["persistence_path_preview"]["blocked_by"]
        .as_array()
        .expect("persistence path blocked by should be an array")
        .contains(&json!("contract_repair_approval_store_ready")));
    assert_eq!(
        review_body["record_snapshot_preview"]["status"],
        "record_snapshot_preview_only"
    );
    assert_eq!(
        review_body["record_snapshot_preview"]["approval_id"],
        approval_id
    );
    assert_eq!(
        review_body["record_snapshot_preview"]["record_kind"],
        "contract_repair_approval"
    );
    assert_eq!(
        review_body["record_snapshot_preview"]["target_path"],
        "event_catalog/observation.bar_closed/fast_ma/bar-002"
    );
    assert_eq!(
        review_body["record_snapshot_preview"]["changed_fields"],
        json!(["payload_value"])
    );
    assert_eq!(
        review_body["record_snapshot_preview"]["review_state"],
        "under_review"
    );
    assert_eq!(
        review_body["record_snapshot_preview"]["reviewer_id"],
        "reviewer-a"
    );
    assert_eq!(
        review_body["record_snapshot_preview"]["persistence_enabled"],
        false
    );
    assert_eq!(
        review_body["record_snapshot_preview"]["would_write_record"],
        false
    );
    assert_eq!(review_body["storage_readiness_gate"]["status"], "blocked");
    assert_eq!(
        review_body["storage_readiness_gate"]["persistence_enabled"],
        false
    );
    assert_eq!(review_body["storage_readiness_gate"]["store_ready"], true);
    assert_eq!(review_body["storage_readiness_gate"]["schema_ready"], true);
    assert_eq!(
        review_body["storage_readiness_gate"]["idempotency_ready"],
        true
    );
    assert_eq!(
        review_body["storage_readiness_gate"]["snapshot_ready"],
        true
    );
    assert!(review_body["storage_readiness_gate"]["ready_gates"]
        .as_array()
        .expect("ready gates should be an array")
        .contains(&json!("persistence_path_preview_ready")));
    assert!(review_body["storage_readiness_gate"]["ready_gates"]
        .as_array()
        .expect("ready gates should be an array")
        .contains(&json!("record_snapshot_preview_ready")));
    assert!(review_body["storage_readiness_gate"]["ready_gates"]
        .as_array()
        .expect("ready gates should be an array")
        .contains(&json!("contract_repair_approval_store_ready")));
    assert!(review_body["storage_readiness_gate"]["blocked_gates"]
        .as_array()
        .expect("blocked gates should be an array")
        .contains(&json!("approval_persistence_enabled")));
    assert!(!review_body["storage_readiness_gate"]["blocked_gates"]
        .as_array()
        .expect("blocked gates should be an array")
        .contains(&json!("contract_repair_approval_store_ready")));
    assert_eq!(
        review_body["storage_dry_run_preview"]["status"],
        "dry_run_blocked"
    );
    assert_eq!(
        review_body["storage_dry_run_preview"]["adapter_kind"],
        "contract_repair_approval_store_adapter"
    );
    assert_eq!(
        review_body["storage_dry_run_preview"]["record_key"],
        approval_id
    );
    assert_eq!(
        review_body["storage_dry_run_preview"]["accepted_by_adapter"],
        true
    );
    assert_eq!(review_body["storage_dry_run_preview"]["would_write"], false);
    assert!(review_body["storage_dry_run_preview"]["blocked_by"]
        .as_array()
        .expect("dry run blocked by should be an array")
        .contains(&json!("approval_persistence_enabled")));
    assert!(!review_body["storage_dry_run_preview"]["blocked_by"]
        .as_array()
        .expect("dry run blocked by should be an array")
        .contains(&json!("contract_repair_approval_store_ready")));
    assert_eq!(
        review_body["idempotency_precheck"]["status"],
        "precheck_checked_blocked"
    );
    assert_eq!(
        review_body["idempotency_precheck"]["idempotency_key"],
        post_body["approval_record_preview"]["idempotency_key"]
    );
    assert_eq!(
        review_body["idempotency_precheck"]["candidate_record_key"],
        approval_id
    );
    assert_eq!(
        review_body["idempotency_precheck"]["store_lookup_enabled"],
        true
    );
    assert_eq!(
        review_body["idempotency_precheck"]["existing_record_found"],
        true
    );
    assert_eq!(
        review_body["idempotency_precheck"]["conflict_detected"],
        false
    );
    assert_eq!(review_body["idempotency_precheck"]["safe_to_write"], false);
    assert!(review_body["idempotency_precheck"]["blocked_by"]
        .as_array()
        .expect("idempotency precheck blocked by should be an array")
        .contains(&json!("approval_persistence_enabled")));
    assert_eq!(
        review_body["review_transition_dry_run"]["status"],
        "transition_dry_run_blocked"
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["approval_id"],
        approval_id
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["from_review_state"],
        "pending"
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["target_review_state"],
        "under_review"
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["transition_ready"],
        false
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["would_transition"],
        false
    );
    assert!(review_body["review_transition_dry_run"]["blocked_by"]
        .as_array()
        .expect("transition dry-run blocked_by should be an array")
        .contains(&json!("formal_reviewer_authorized")));
    assert_eq!(
        review_body["record_write_dry_run"]["status"],
        "record_write_dry_run_blocked"
    );
    assert_eq!(
        review_body["record_write_dry_run"]["adapter_kind"],
        "contract_repair_approval_record_writer"
    );
    assert_eq!(
        review_body["record_write_dry_run"]["store_kind"],
        "contract_repair_approval_records"
    );
    assert_eq!(
        review_body["record_write_dry_run"]["record_key"],
        approval_id
    );
    assert_eq!(
        review_body["record_write_dry_run"]["file_name"],
        format!("{approval_id}.json")
    );
    assert_eq!(
        review_body["record_write_dry_run"]["transition_ready"],
        false
    );
    assert_eq!(review_body["record_write_dry_run"]["storage_ready"], true);
    assert_eq!(review_body["record_write_dry_run"]["schema_ready"], true);
    assert_eq!(
        review_body["record_write_dry_run"]["idempotency_ready"],
        true
    );
    assert_eq!(review_body["record_write_dry_run"]["snapshot_ready"], true);
    assert_eq!(
        review_body["record_write_dry_run"]["idempotency_precheck_passed"],
        true
    );
    assert_eq!(review_body["record_write_dry_run"]["write_ready"], false);
    assert_eq!(review_body["record_write_dry_run"]["would_write"], false);
    assert!(review_body["record_write_dry_run"]["blocked_by"]
        .as_array()
        .expect("record write dry-run blocked_by should be an array")
        .contains(&json!("review_transition_ready")));
    assert!(review_body["record_write_dry_run"]["blocked_by"]
        .as_array()
        .expect("record write dry-run blocked_by should be an array")
        .contains(&json!("approval_persistence_enabled")));
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["status"],
        "lifecycle_dry_run_blocked"
    );
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["event_id"],
        format!("contract-repair-review-claim:{approval_id}")
    );
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["event_kind"],
        "contract_repair_approval_review"
    );
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["target_review_state"],
        "under_review"
    );
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["actor_id"],
        "reviewer-a"
    );
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["reason_code"],
        "review_transition_preview_only"
    );
    assert_eq!(review_body["lifecycle_event_dry_run"]["sequence_no"], 1);
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["transition_ready"],
        false
    );
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["event_payload_ready"],
        true
    );
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["emission_ready"],
        false
    );
    assert_eq!(review_body["lifecycle_event_dry_run"]["would_emit"], false);
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["status"],
        "lifecycle_entry_append_blocked"
    );
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["entry_ready"],
        true
    );
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["append_ready"],
        false
    );
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["would_append"],
        false
    );
    assert!(review_body["lifecycle_event_dry_run"]["blocked_by"]
        .as_array()
        .expect("lifecycle dry-run blocked by should be an array")
        .contains(&json!("formal_reviewer_authorized")));
    assert_eq!(
        review_body["contract_writeback_dry_run"]["status"],
        "contract_writeback_dry_run_blocked"
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["patch_kind"],
        "event_payload_instance_patch"
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["target_path"],
        "event_catalog/observation.bar_closed/fast_ma/bar-002"
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["target_kind"],
        "event_instance_payload"
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["changed_fields"],
        json!(["payload_value"])
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["patch_payload_ready"],
        false
    );
    assert!(
        review_body["contract_writeback_dry_run"]["missing_patch_fields"]
            .as_array()
            .expect("missing patch fields should be an array")
            .contains(&json!("payload_value"))
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["contract_source_ready"],
        false
    );
    assert!(
        review_body["contract_writeback_dry_run"]["missing_contract_source_fields"]
            .as_array()
            .expect("missing contract source fields should be an array")
            .contains(&json!("source_id"))
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["eligible_after_approval"],
        false
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["lifecycle_append_ready"],
        false
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["would_mutate_contract"],
        false
    );
    assert!(review_body["contract_writeback_dry_run"]["blocked_by"]
        .as_array()
        .expect("contract writeback dry-run blocked by should be an array")
        .contains(&json!("contract_patch_payload_ready")));
    assert!(review_body["contract_writeback_dry_run"]["blocked_by"]
        .as_array()
        .expect("contract writeback dry-run blocked by should be an array")
        .contains(&json!("contract_source_ref_ready")));
    assert!(review_body["contract_writeback_dry_run"]["blocked_by"]
        .as_array()
        .expect("contract writeback dry-run blocked by should be an array")
        .contains(&json!("lifecycle_entry_append_ready")));
    assert!(review_body["contract_writeback_dry_run"]["blocked_by"]
        .as_array()
        .expect("contract writeback dry-run blocked by should be an array")
        .contains(&json!("contract_mutation_api_enabled")));
    assert_eq!(
        review_body["approval_record_preview"]["review_state"],
        "pending"
    );
    assert_eq!(
        review_body["approval_record_preview"]["transient_review_status"],
        "claim_intent_recorded"
    );
    assert_eq!(
        review_body["approval_record_preview"]["transient_review_action"],
        "claim"
    );
    assert_eq!(
        review_body["approval_record_preview"]["transient_reviewer_id"],
        "reviewer-a"
    );
    assert!(review_body["blocked_reasons"]
        .as_array()
        .expect("blocked reasons should be an array")
        .contains(&json!("review_workflow_disabled")));

    let (detail_status, detail_body) = get_contract_repair_path_with_app(
        app,
        &format!("/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}"),
    )
    .await;

    assert_eq!(detail_status, StatusCode::OK, "{detail_body}");
    assert_eq!(detail_body["record"]["approval_id"], approval_id);
    assert_eq!(detail_body["record"]["review_state"], "pending");
    assert_eq!(
        detail_body["record"]["transient_review_status"],
        "claim_intent_recorded"
    );
    assert_eq!(detail_body["record"]["transient_reviewer_id"], "reviewer-a");
    assert_eq!(detail_body["persistence_enabled"], true);
    assert_eq!(detail_body["mutation_enabled"], false);

    let restarted_app = common::test_app_from_dirs(dirs);
    let (restarted_detail_status, restarted_detail_body) = get_contract_repair_path_with_app(
        restarted_app.clone(),
        &format!("/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}"),
    )
    .await;

    assert_eq!(
        restarted_detail_status,
        StatusCode::OK,
        "{restarted_detail_body}"
    );
    assert_eq!(
        restarted_detail_body["record"]["transient_review_status"],
        "claim_intent_recorded"
    );
    assert_eq!(
        restarted_detail_body["record"]["transient_review_action"],
        "claim"
    );
    assert_eq!(
        restarted_detail_body["record"]["transient_reviewer_id"],
        "reviewer-a"
    );
    assert_eq!(restarted_detail_body["record"]["review_state"], "pending");
    assert_eq!(restarted_detail_body["persistence_enabled"], true);
    assert_eq!(restarted_detail_body["mutation_enabled"], false);

    let (restarted_list_status, restarted_list_body) = get_contract_repair_path_with_app(
        restarted_app,
        "/api/runtime/v4/productization/contract-repairs/approval-requests",
    )
    .await;

    assert_eq!(
        restarted_list_status,
        StatusCode::OK,
        "{restarted_list_body}"
    );
    assert_eq!(restarted_list_body["status"], "persisted_read_model");
    assert_eq!(restarted_list_body["persisted_record_count"], 1);
    assert_eq!(
        restarted_list_body["records"][0]["transient_review_status"],
        "claim_intent_recorded"
    );
    assert_eq!(
        restarted_list_body["records"][0]["transient_review_action"],
        "claim"
    );
    assert_eq!(
        restarted_list_body["records"][0]["transient_reviewer_id"],
        "reviewer-a"
    );
    assert_eq!(restarted_list_body["records"][0]["review_state"], "pending");
}

#[tokio::test(flavor = "multi_thread")]
async fn contract_repair_approval_decision_intents_record_transient_markers_only() {
    let (app, dirs) =
        common::test_app_with_dirs("api_v4_productization_contract_repair_decision_intents");
    let (post_status, post_body) = post_contract_repair_request_with_app(app.clone(), json!({
        "status": "body_preview_only",
        "payload_kind": "v4_contract_repair_approval_request",
        "request_id": "approval-request:repair-draft:event_catalog/observation.bar_closed/slow_ma/bar-003",
        "target_path": "event_catalog/observation.bar_closed/slow_ma/bar-003",
        "target_kind": "event_instance_payload",
        "changed_fields": ["payload_value"],
        "mutation_enabled": false,
        "review_required": true
    }))
    .await;
    let approval_id = post_body["approval_record_preview"]["approval_id"]
        .as_str()
        .expect("approval id should be present")
        .to_string();

    assert_eq!(post_status, StatusCode::LOCKED, "{post_body}");

    let (approve_status, approve_body) = post_contract_repair_path_with_app(
        app.clone(),
        &format!(
            "/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}/review"
        ),
        json!({
            "action": "approve",
            "reviewer_id": "reviewer-a",
            "reason": "preview looks safe",
            "review_enabled": false
        }),
    )
    .await;

    assert_eq!(approve_status, StatusCode::LOCKED, "{approve_body}");
    assert_eq!(approve_body["route_status"], "review_workflow_disabled");
    assert_eq!(approve_body["action"], "approve");
    assert_eq!(
        approve_body["approval_record_preview"]["review_state"],
        "pending"
    );
    assert_eq!(
        approve_body["approval_record_preview"]["transient_review_status"],
        "approve_intent_recorded"
    );
    assert_eq!(
        approve_body["approval_record_preview"]["transient_review_action"],
        "approve"
    );
    assert_eq!(approve_body["persistence_enabled"], false);
    assert_eq!(approve_body["mutation_enabled"], false);
    assert_eq!(approve_body["execution_gate"]["status"], "blocked");
    assert!(approve_body["execution_gate"]["passed_gates"]
        .as_array()
        .expect("passed gates should be an array")
        .contains(&json!("transient_preview_exists")));
    assert!(approve_body["execution_gate"]["passed_gates"]
        .as_array()
        .expect("passed gates should be an array")
        .contains(&json!("review_intent_valid")));
    assert!(approve_body["execution_gate"]["passed_gates"]
        .as_array()
        .expect("passed gates should be an array")
        .contains(&json!("reviewer_identity_present")));
    assert!(approve_body["execution_gate"]["passed_gates"]
        .as_array()
        .expect("passed gates should be an array")
        .contains(&json!("contract_repair_approval_store_ready")));
    assert!(approve_body["execution_gate"]["passed_gates"]
        .as_array()
        .expect("passed gates should be an array")
        .contains(&json!("idempotency_precheck_passed")));
    assert!(approve_body["execution_gate"]["blocked_gates"]
        .as_array()
        .expect("blocked gates should be an array")
        .contains(&json!("reviewer_identity_format_valid")));
    assert!(approve_body["execution_gate"]["blocked_gates"]
        .as_array()
        .expect("blocked gates should be an array")
        .contains(&json!("reviewer_identity_matches_auth_subject")));
    assert!(approve_body["execution_gate"]["blocked_gates"]
        .as_array()
        .expect("blocked gates should be an array")
        .contains(&json!("contract_mutation_api_enabled")));
    assert_eq!(
        approve_body["reviewer_authorization_precheck"]["authorized"],
        false
    );
    assert!(
        approve_body["reviewer_authorization_precheck"]["blocked_by"]
            .as_array()
            .expect("authorization precheck blocked_by should be an array")
            .contains(&json!("reviewer_identity_format_valid"))
    );
    assert_eq!(
        approve_body["execution_plan_preview"]["target_review_state"],
        "approved"
    );
    assert_eq!(
        approve_body["execution_plan_preview"]["would_mutate_contract"],
        false
    );
    assert_eq!(
        approve_body["persistence_plan_preview"]["record_kind"],
        "contract_repair_approval"
    );
    assert_eq!(
        approve_body["persistence_plan_preview"]["record_source_kind"],
        "transient_preview_cache"
    );
    assert_eq!(
        approve_body["record_snapshot_preview"]["review_state"],
        "approved"
    );
    assert_eq!(
        approve_body["record_snapshot_preview"]["would_write_record"],
        false
    );
    assert_eq!(approve_body["storage_readiness_gate"]["schema_ready"], true);
    assert_eq!(approve_body["storage_readiness_gate"]["store_ready"], true);
    assert!(approve_body["storage_readiness_gate"]["ready_gates"]
        .as_array()
        .expect("ready gates should be an array")
        .contains(&json!("contract_repair_approval_store_ready")));
    assert!(!approve_body["storage_readiness_gate"]["blocked_gates"]
        .as_array()
        .expect("blocked gates should be an array")
        .contains(&json!("contract_repair_approval_store_ready")));
    assert_eq!(
        approve_body["storage_dry_run_preview"]["would_write"],
        false
    );
    assert_eq!(
        approve_body["idempotency_precheck"]["store_lookup_enabled"],
        true
    );
    assert_eq!(
        approve_body["idempotency_precheck"]["existing_record_found"],
        true
    );
    assert_eq!(approve_body["idempotency_precheck"]["safe_to_write"], false);
    assert_eq!(
        approve_body["lifecycle_event_dry_run"]["target_review_state"],
        "approved"
    );
    assert_eq!(approve_body["lifecycle_event_dry_run"]["would_emit"], false);
    assert_eq!(
        approve_body["contract_writeback_dry_run"]["eligible_after_approval"],
        true
    );
    assert_eq!(
        approve_body["contract_writeback_dry_run"]["would_mutate_contract"],
        false
    );

    let (reject_status, reject_body) = post_contract_repair_path_with_app(
        app.clone(),
        &format!(
            "/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}/review"
        ),
        json!({
            "action": "reject",
            "reviewer_id": "reviewer-b",
            "reason": "needs stronger evidence",
            "review_enabled": false
        }),
    )
    .await;

    assert_eq!(reject_status, StatusCode::LOCKED, "{reject_body}");
    assert_eq!(reject_body["action"], "reject");
    assert_eq!(
        reject_body["approval_record_preview"]["review_state"],
        "pending"
    );
    assert_eq!(
        reject_body["approval_record_preview"]["transient_review_status"],
        "reject_intent_recorded"
    );
    assert_eq!(
        reject_body["approval_record_preview"]["transient_review_action"],
        "reject"
    );
    assert_eq!(
        reject_body["approval_record_preview"]["transient_reviewer_id"],
        "reviewer-b"
    );

    let (detail_status, detail_body) = get_contract_repair_path_with_app(
        app,
        &format!("/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}"),
    )
    .await;

    assert_eq!(detail_status, StatusCode::OK, "{detail_body}");
    assert_eq!(detail_body["record"]["review_state"], "pending");
    assert_eq!(
        detail_body["record"]["transient_review_status"],
        "reject_intent_recorded"
    );
    assert_eq!(detail_body["record"]["transient_review_action"], "reject");
    assert_eq!(detail_body["record"]["transient_reviewer_id"], "reviewer-b");
    assert_eq!(detail_body["persistence_enabled"], true);
    assert_eq!(detail_body["mutation_enabled"], false);

    let restarted_app = common::test_app_from_dirs(dirs);
    let (restarted_detail_status, restarted_detail_body) = get_contract_repair_path_with_app(
        restarted_app,
        &format!("/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}"),
    )
    .await;

    assert_eq!(
        restarted_detail_status,
        StatusCode::OK,
        "{restarted_detail_body}"
    );
    assert_eq!(
        restarted_detail_body["record"]["transient_review_status"],
        "reject_intent_recorded"
    );
    assert_eq!(
        restarted_detail_body["record"]["transient_review_action"],
        "reject"
    );
    assert_eq!(
        restarted_detail_body["record"]["transient_reviewer_id"],
        "reviewer-b"
    );
    assert_eq!(
        restarted_detail_body["record"]["transient_review_reason"],
        "needs stronger evidence"
    );
    assert_eq!(restarted_detail_body["record"]["review_state"], "pending");
    assert_eq!(restarted_detail_body["persistence_enabled"], true);
    assert_eq!(restarted_detail_body["mutation_enabled"], false);
}

#[tokio::test(flavor = "multi_thread")]
async fn contract_repair_approval_authorized_reviewer_precheck_still_keeps_review_locked() {
    let (app, dirs) =
        common::test_app_with_dirs("api_v4_productization_contract_repair_reviewer_grant");
    let grants_path = dirs
        .backtest_store_dir
        .parent()
        .expect("test backtest store should have a storage parent")
        .join("contract-repair-reviewer-grants.json");
    std::fs::write(
        &grants_path,
        r#"{"policy_version":"quantpilot/contract-repair-reviewer-role-policy/v1","grants":[{"subject":"user:0","role":"contract_repair_reviewer"}]}"#,
    )
    .expect("reviewer grant file should be writable");
    let (post_status, post_body) = post_contract_repair_request_with_app(
        app.clone(),
        json!({
            "status": "body_preview_only",
            "payload_kind": "v4_contract_repair_approval_request",
            "request_id": "approval-request:repair-draft:memory_schema/decision.dual_ma/reviewer-grant",
            "target_path": "memory_schema/decision.dual_ma/reviewer-grant",
            "target_kind": "memory_field",
            "changed_fields": ["type_name"],
            "mutation_enabled": false,
            "review_required": true
        }),
    )
    .await;
    let approval_id = post_body["approval_record_preview"]["approval_id"]
        .as_str()
        .expect("approval id should be present")
        .to_string();

    assert_eq!(post_status, StatusCode::LOCKED, "{post_body}");

    let (review_status, review_body) = post_contract_repair_path_with_app(
        app,
        &format!(
            "/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}/review"
        ),
        json!({
            "action": "claim",
            "reviewer_id": "user:0",
            "reason": "claim with explicit reviewer grant",
            "review_enabled": false
        }),
    )
    .await;

    assert_eq!(review_status, StatusCode::LOCKED, "{review_body}");
    assert_eq!(review_body["route_status"], "review_workflow_disabled");
    assert_eq!(
        review_body["reviewer_authorization_precheck"]["status"],
        "authorization_precheck_authorized"
    );
    assert_eq!(
        review_body["reviewer_authorization_precheck"]["policy_version"],
        "quantpilot/contract-repair-reviewer-role-policy/v1"
    );
    assert_eq!(
        review_body["reviewer_authorization_precheck"]["required_role"],
        "contract_repair_reviewer"
    );
    assert_eq!(
        review_body["reviewer_authorization_precheck"]["grant_source"],
        "file:contract-repair-reviewer-grants.json"
    );
    assert_eq!(
        review_body["reviewer_authorization_precheck"]["authorized"],
        true
    );
    assert!(review_body["reviewer_authorization_precheck"]["blocked_by"]
        .as_array()
        .expect("authorization blockers should be an array")
        .is_empty());
    assert!(review_body["execution_gate"]["passed_gates"]
        .as_array()
        .expect("passed gates should be an array")
        .contains(&json!("formal_reviewer_authorized")));
    assert!(review_body["execution_gate"]["blocked_gates"]
        .as_array()
        .expect("blocked gates should be an array")
        .contains(&json!("review_workflow_enabled")));
    assert!(review_body["execution_gate"]["blocked_gates"]
        .as_array()
        .expect("blocked gates should be an array")
        .contains(&json!("approval_persistence_enabled")));
    assert!(!review_body["execution_gate"]["blocked_gates"]
        .as_array()
        .expect("blocked gates should be an array")
        .contains(&json!("contract_mutation_api_enabled")));
    assert!(!review_body["execution_gate"]["blocked_gates"]
        .as_array()
        .expect("blocked gates should be an array")
        .contains(&json!("formal_reviewer_authorized")));
    assert_eq!(
        review_body["execution_plan_preview"]["execution_enabled"],
        false
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["status"],
        "transition_dry_run_ready_blocked"
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["from_review_state"],
        "pending"
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["target_review_state"],
        "under_review"
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["reviewer_id"],
        "user:0"
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["sequence_no_preview"],
        1
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["transition_ready"],
        true
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["would_transition"],
        false
    );
    assert!(!review_body["review_transition_dry_run"]["blocked_by"]
        .as_array()
        .expect("transition dry-run blocked_by should be an array")
        .contains(&json!("formal_reviewer_authorized")));
    assert!(review_body["review_transition_dry_run"]["blocked_by"]
        .as_array()
        .expect("transition dry-run blocked_by should be an array")
        .contains(&json!("review_workflow_enabled")));
    assert_eq!(
        review_body["record_write_dry_run"]["status"],
        "record_write_dry_run_ready_blocked"
    );
    assert_eq!(
        review_body["record_write_dry_run"]["transition_ready"],
        true
    );
    assert_eq!(review_body["record_write_dry_run"]["storage_ready"], true);
    assert_eq!(
        review_body["record_write_dry_run"]["idempotency_precheck_passed"],
        true
    );
    assert_eq!(review_body["record_write_dry_run"]["write_ready"], true);
    assert_eq!(review_body["record_write_dry_run"]["would_write"], false);
    assert!(review_body["record_write_dry_run"]["blocked_by"]
        .as_array()
        .expect("record write dry-run blocked_by should be an array")
        .contains(&json!("approval_persistence_enabled")));
    assert!(!review_body["record_write_dry_run"]["blocked_by"]
        .as_array()
        .expect("record write dry-run blocked_by should be an array")
        .contains(&json!("review_transition_ready")));
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["status"],
        "lifecycle_dry_run_emission_ready_blocked"
    );
    assert_eq!(review_body["lifecycle_event_dry_run"]["sequence_no"], 1);
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["transition_ready"],
        true
    );
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["event_payload_ready"],
        true
    );
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["emission_ready"],
        true
    );
    assert_eq!(review_body["lifecycle_event_dry_run"]["would_emit"], false);
    assert!(!review_body["lifecycle_event_dry_run"]["blocked_by"]
        .as_array()
        .expect("lifecycle dry-run blocked_by should be an array")
        .contains(&json!("formal_reviewer_authorized")));
    assert!(review_body["lifecycle_event_dry_run"]["blocked_by"]
        .as_array()
        .expect("lifecycle dry-run blocked_by should be an array")
        .contains(&json!("review_workflow_enabled")));
    assert_eq!(
        review_body["contract_writeback_dry_run"]["would_mutate_contract"],
        false
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn contract_repair_approval_approve_execution_preflight_stays_locked() {
    let (app, dirs) =
        common::test_app_with_dirs("api_v4_productization_contract_repair_approve_preflight");
    let grants_path = dirs
        .backtest_store_dir
        .parent()
        .expect("test backtest store should have a storage parent")
        .join("contract-repair-reviewer-grants.json");
    std::fs::write(
        &grants_path,
        r#"{"policy_version":"quantpilot/contract-repair-reviewer-role-policy/v1","grants":[{"subject":"user:0","role":"contract_repair_reviewer"}]}"#,
    )
    .expect("reviewer grant file should be writable");
    write_contract_source_fixture(&dirs, "graph-dual-ma", "v4-test", "sha256:test-contract");
    let (post_status, post_body) = post_contract_repair_request_with_app(
        app.clone(),
        json!({
            "status": "body_preview_only",
            "payload_kind": "v4_contract_repair_approval_request",
            "request_id": "approval-request:repair-draft:memory_schema/decision.dual_ma/approve-preflight/last_signal_at",
            "target_path": "memory_schema/decision.dual_ma/approve-preflight/last_signal_at",
            "target_kind": "memory_field",
            "changed_fields": ["type_name"],
            "patch_payload": {"type_name": "time?"},
            "contract_source_ref": {
                "source_kind": "v4_machine_graph_contract",
                "source_id": "graph-dual-ma",
                "version": "v4-test",
                "artifact_digest": "sha256:test-contract"
            },
            "mutation_enabled": false,
            "review_required": true
        }),
    )
    .await;
    let approval_id = post_body["approval_record_preview"]["approval_id"]
        .as_str()
        .expect("approval id should be present")
        .to_string();

    assert_eq!(post_status, StatusCode::LOCKED, "{post_body}");

    let (review_status, review_body) = post_contract_repair_path_with_app(
        app.clone(),
        &format!(
            "/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}/review"
        ),
        json!({
            "action": "approve",
            "reviewer_id": "user:0",
            "reason": "approve with explicit reviewer grant",
            "review_enabled": true
        }),
    )
    .await;

    assert_eq!(review_status, StatusCode::LOCKED, "{review_body}");
    assert_eq!(review_body["status"], "review_decision_execution_blocked");
    assert_eq!(
        review_body["route_status"],
        "review_decision_execution_blocked"
    );
    assert_eq!(review_body["action"], "approve");
    assert_eq!(review_body["review_enabled"], false);
    assert_eq!(review_body["persistence_enabled"], false);
    assert_eq!(review_body["mutation_enabled"], false);
    assert!(review_body["blocked_reasons"]
        .as_array()
        .expect("blocked reasons should be an array")
        .contains(&json!("approve_execution_not_enabled")));
    assert_eq!(
        review_body["reviewer_authorization_precheck"]["status"],
        "authorization_precheck_authorized"
    );
    assert!(review_body["execution_gate"]["passed_gates"]
        .as_array()
        .expect("passed gates should be an array")
        .contains(&json!("formal_reviewer_authorized")));
    assert!(review_body["execution_gate"]["passed_gates"]
        .as_array()
        .expect("passed gates should be an array")
        .contains(&json!("review_workflow_enabled")));
    assert!(review_body["execution_gate"]["passed_gates"]
        .as_array()
        .expect("passed gates should be an array")
        .contains(&json!("approval_persistence_enabled")));
    assert!(!review_body["execution_gate"]["blocked_gates"]
        .as_array()
        .expect("blocked gates should be an array")
        .contains(&json!("review_workflow_enabled")));
    assert!(!review_body["execution_gate"]["blocked_gates"]
        .as_array()
        .expect("blocked gates should be an array")
        .contains(&json!("approval_persistence_enabled")));
    assert!(review_body["execution_gate"]["blocked_gates"]
        .as_array()
        .expect("blocked gates should be an array")
        .contains(&json!("lifecycle_event_emission_enabled")));
    assert!(review_body["execution_gate"]["blocked_gates"]
        .as_array()
        .expect("blocked gates should be an array")
        .contains(&json!("contract_mutation_api_enabled")));
    assert_eq!(
        review_body["execution_plan_preview"]["target_review_state"],
        "approved"
    );
    assert_eq!(
        review_body["execution_plan_preview"]["execution_enabled"],
        false
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["status"],
        "transition_dry_run_ready_blocked"
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["target_review_state"],
        "approved"
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["would_transition"],
        false
    );
    assert_eq!(
        review_body["record_write_dry_run"]["status"],
        "record_write_dry_run_ready_blocked"
    );
    assert_eq!(review_body["record_write_dry_run"]["write_ready"], true);
    assert_eq!(review_body["record_write_dry_run"]["would_write"], false);
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["status"],
        "lifecycle_dry_run_emission_ready_blocked"
    );
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["event_payload_ready"],
        true
    );
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["emission_ready"],
        true
    );
    assert_eq!(review_body["lifecycle_event_dry_run"]["would_emit"], false);
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["status"],
        "lifecycle_entry_append_ready_blocked"
    );
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["would_append"],
        false
    );
    assert!(review_body["lifecycle_event_dry_run"]["blocked_by"]
        .as_array()
        .expect("lifecycle blockers should be an array")
        .contains(&json!("lifecycle_event_emission_enabled")));
    assert!(!review_body["lifecycle_event_dry_run"]["blocked_by"]
        .as_array()
        .expect("lifecycle blockers should be an array")
        .contains(&json!("review_workflow_enabled")));
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["status"],
        "lifecycle_entry_append_ready_blocked"
    );
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["entry_ready"],
        true
    );
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["emission_ready"],
        true
    );
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["append_ready"],
        true
    );
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["would_append"],
        false
    );
    assert!(review_body["lifecycle_entry_append_dry_run"]["blocked_by"]
        .as_array()
        .expect("lifecycle append blockers should be an array")
        .contains(&json!("lifecycle_event_emission_enabled")));
    assert_eq!(
        review_body["lifecycle_emission_enablement_gate"]["status"],
        "lifecycle_emission_enablement_ready_blocked"
    );
    assert_eq!(
        review_body["lifecycle_emission_enablement_gate"]["lifecycle_emission_plan_ready"],
        true
    );
    assert_eq!(
        review_body["lifecycle_emission_enablement_gate"]["transition_ready"],
        true
    );
    assert_eq!(
        review_body["lifecycle_emission_enablement_gate"]["event_payload_ready"],
        true
    );
    assert_eq!(
        review_body["lifecycle_emission_enablement_gate"]["entry_append_ready"],
        true
    );
    assert_eq!(
        review_body["lifecycle_emission_enablement_gate"]["lifecycle_event_emission_enabled"],
        false
    );
    assert_eq!(
        review_body["lifecycle_emission_enablement_gate"]["lifecycle_entry_append_enabled"],
        false
    );
    assert_eq!(
        review_body["lifecycle_emission_enablement_gate"]["lifecycle_effects_ready"],
        false
    );
    assert_eq!(
        review_body["lifecycle_emission_enablement_gate"]["would_emit"],
        false
    );
    assert_eq!(
        review_body["lifecycle_emission_enablement_gate"]["would_append"],
        false
    );
    assert_eq!(
        review_body["lifecycle_emission_enablement_gate"]["would_touch_lifecycle_log"],
        false
    );
    assert!(
        review_body["lifecycle_emission_enablement_gate"]["passed_gates"]
            .as_array()
            .expect("lifecycle enablement passed gates should be an array")
            .contains(&json!("lifecycle_entry_append_ready"))
    );
    assert!(
        review_body["lifecycle_emission_enablement_gate"]["blocked_gates"]
            .as_array()
            .expect("lifecycle enablement blocked gates should be an array")
            .contains(&json!("lifecycle_event_emission_enabled"))
    );
    assert!(
        review_body["lifecycle_emission_enablement_gate"]["blocked_gates"]
            .as_array()
            .expect("lifecycle enablement blocked gates should be an array")
            .contains(&json!("lifecycle_entry_append_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]["status"],
        "approve_execution_lifecycle_effects_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]["gate_name"],
        "lifecycle_effects_ready"
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]
            ["lifecycle_emission_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]["transition_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]["event_payload_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]["emission_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]["entry_append_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]
            ["lifecycle_event_emission_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]
            ["lifecycle_entry_append_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]
            ["lifecycle_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]["would_emit"],
        false
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]["would_append"],
        false
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]
            ["would_touch_lifecycle_log"],
        false
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]
            ["would_unblock_atomic_side_effects"],
        false
    );
    assert!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]
            ["inherited_lifecycle_emission_blocked_gates"]
            .as_array()
            .expect("lifecycle effects readiness inherited blockers should be an array")
            .contains(&json!("lifecycle_event_emission_enabled"))
    );
    assert!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]
            ["inherited_lifecycle_emission_blocked_gates"]
            .as_array()
            .expect("lifecycle effects readiness inherited blockers should be an array")
            .contains(&json!("lifecycle_entry_append_enabled"))
    );
    assert!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]["passed_gates"]
            .as_array()
            .expect("lifecycle effects readiness passed gates should be an array")
            .contains(&json!("lifecycle_entry_append_ready"))
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]["required_gates"]
            .as_array()
            .expect("lifecycle effects readiness required gates should be an array")
            .len(),
        6
    );
    assert!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("lifecycle effects readiness blockers should be an array")
            .contains(&json!("lifecycle_event_emission_enabled"))
    );
    assert!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("lifecycle effects readiness blockers should be an array")
            .contains(&json!("lifecycle_entry_append_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_event_emission_enablement_dry_run"]["status"],
        "approve_execution_lifecycle_event_emission_enablement_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_event_emission_enablement_dry_run"]["switch_name"],
        "lifecycle_event_emission_enabled"
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_event_emission_enablement_dry_run"]
            ["lifecycle_emission_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_event_emission_enablement_dry_run"]
            ["lifecycle_event_emission_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_event_emission_enablement_dry_run"]
            ["lifecycle_event_emission_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_event_emission_enablement_dry_run"]
            ["would_enable_lifecycle_event_emission"],
        false
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_event_emission_enablement_dry_run"]["would_emit"],
        false
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_event_emission_enablement_dry_run"]
            ["would_append"],
        false
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_event_emission_enablement_dry_run"]
            ["would_touch_lifecycle_log"],
        false
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_event_emission_enablement_dry_run"]
            ["would_unblock_lifecycle_effects"],
        false
    );
    assert!(
        review_body["approve_execution_lifecycle_event_emission_enablement_dry_run"]
            ["inherited_lifecycle_emission_blocked_gates"]
            .as_array()
            .expect("lifecycle event emission inherited blockers should be an array")
            .contains(&json!("lifecycle_event_emission_enabled"))
    );
    assert!(
        review_body["approve_execution_lifecycle_event_emission_enablement_dry_run"]
            ["inherited_lifecycle_emission_blocked_gates"]
            .as_array()
            .expect("lifecycle event emission inherited blockers should be an array")
            .contains(&json!("lifecycle_entry_append_enabled"))
    );
    assert!(
        review_body["approve_execution_lifecycle_event_emission_enablement_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("lifecycle event emission passed gates should be an array")
            .contains(&json!("lifecycle_emission_plan_ready"))
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_event_emission_enablement_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("lifecycle event emission required gates should be an array")
            .len(),
        2
    );
    assert!(
        review_body["approve_execution_lifecycle_event_emission_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("lifecycle event emission blockers should be an array")
            .contains(&json!("lifecycle_event_emission_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_entry_append_enablement_dry_run"]["status"],
        "approve_execution_lifecycle_entry_append_enablement_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_entry_append_enablement_dry_run"]["switch_name"],
        "lifecycle_entry_append_enabled"
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_entry_append_enablement_dry_run"]
            ["lifecycle_emission_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_entry_append_enablement_dry_run"]
            ["lifecycle_entry_append_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_entry_append_enablement_dry_run"]
            ["lifecycle_entry_append_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_entry_append_enablement_dry_run"]
            ["would_enable_lifecycle_entry_append"],
        false
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_entry_append_enablement_dry_run"]["would_emit"],
        false
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_entry_append_enablement_dry_run"]["would_append"],
        false
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_entry_append_enablement_dry_run"]
            ["would_touch_lifecycle_log"],
        false
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_entry_append_enablement_dry_run"]
            ["would_unblock_lifecycle_effects"],
        false
    );
    assert!(
        review_body["approve_execution_lifecycle_entry_append_enablement_dry_run"]
            ["inherited_lifecycle_emission_blocked_gates"]
            .as_array()
            .expect("lifecycle entry append inherited blockers should be an array")
            .contains(&json!("lifecycle_event_emission_enabled"))
    );
    assert!(
        review_body["approve_execution_lifecycle_entry_append_enablement_dry_run"]
            ["inherited_lifecycle_emission_blocked_gates"]
            .as_array()
            .expect("lifecycle entry append inherited blockers should be an array")
            .contains(&json!("lifecycle_entry_append_enabled"))
    );
    assert!(
        review_body["approve_execution_lifecycle_entry_append_enablement_dry_run"]["passed_gates"]
            .as_array()
            .expect("lifecycle entry append passed gates should be an array")
            .contains(&json!("lifecycle_emission_plan_ready"))
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_entry_append_enablement_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("lifecycle entry append required gates should be an array")
            .len(),
        2
    );
    assert!(
        review_body["approve_execution_lifecycle_entry_append_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("lifecycle entry append blockers should be an array")
            .contains(&json!("lifecycle_entry_append_enabled"))
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["eligible_after_approval"],
        true
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["status"],
        "contract_writeback_dry_run_ready_blocked"
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["patch_ready"],
        true
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["patch_payload_ready"],
        true
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["patch_payload"]["type_name"],
        "time?"
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["patch_plan"]["status"],
        "contract_patch_plan_ready"
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["patch_plan"]["contract_patch_ready"],
        true
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["patch_plan"]["operations"][0]["selector"],
        "machines[machine_id=decision.dual_ma].memory[name=last_signal_at]"
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["patch_plan"]["operations"][0]["field_name"],
        "type_name"
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["patch_apply_dry_run"]["status"],
        "contract_patch_apply_ready_blocked"
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["patch_apply_dry_run"]["apply_ready"],
        true
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["patch_apply_dry_run"]["operation_count"],
        1
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["patch_apply_dry_run"]["applied_operation_count"],
        1
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["patch_apply_dry_run"]["applied_selectors"][0],
        "machines[machine_id=decision.dual_ma].memory[name=last_signal_at]"
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["patch_apply_dry_run"]["would_persist_source"],
        false
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["source_write_dry_run"]["status"],
        "contract_source_write_ready_blocked"
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["source_write_dry_run"]["write_ready"],
        true
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["source_write_dry_run"]["atomic_write_required"],
        true
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["source_write_dry_run"]["operation_count"],
        1
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["source_write_dry_run"]["would_write_source"],
        false
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["source_write_dry_run"]["would_touch_disk"],
        false
    );
    assert!(
        review_body["contract_writeback_dry_run"]["source_write_dry_run"]["source_digest_before"]
            .as_str()
            .expect("source digest before should be present")
            .starts_with("sha256:")
    );
    assert!(
        review_body["contract_writeback_dry_run"]["source_write_dry_run"]["source_digest_after"]
            .as_str()
            .expect("source digest after should be present")
            .starts_with("sha256:")
    );
    assert_ne!(
        review_body["contract_writeback_dry_run"]["source_write_dry_run"]["source_digest_before"],
        review_body["contract_writeback_dry_run"]["source_write_dry_run"]["source_digest_after"]
    );
    assert!(
        review_body["contract_writeback_dry_run"]["missing_patch_fields"]
            .as_array()
            .expect("missing patch fields should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["contract_source_ready"],
        true
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["contract_source_ref"]["source_id"],
        "graph-dual-ma"
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["source_resolution"]["status"],
        "contract_source_resolved"
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["source_resolution"]["resolved"],
        true
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["source_resolution"]["source_exists"],
        true
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["source_resolution"]["version_match"],
        true
    );
    assert!(
        review_body["contract_writeback_dry_run"]["missing_contract_source_fields"]
            .as_array()
            .expect("missing contract source fields should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["transition_ready"],
        true
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["lifecycle_append_ready"],
        true
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["writeback_ready"],
        true
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["would_mutate_contract"],
        false
    );
    assert!(review_body["contract_writeback_dry_run"]["blocked_by"]
        .as_array()
        .expect("contract writeback blockers should be an array")
        .contains(&json!("lifecycle_event_emission_enabled")));
    assert!(review_body["contract_writeback_dry_run"]["blocked_by"]
        .as_array()
        .expect("contract writeback blockers should be an array")
        .contains(&json!("contract_mutation_api_enabled")));
    assert_eq!(
        review_body["contract_mutation_enablement_gate"]["status"],
        "contract_mutation_enablement_ready_blocked"
    );
    assert_eq!(
        review_body["contract_mutation_enablement_gate"]["target_kind"],
        "memory_field"
    );
    assert_eq!(
        review_body["contract_mutation_enablement_gate"]["writeback_plan_ready"],
        true
    );
    assert_eq!(
        review_body["contract_mutation_enablement_gate"]["source_write_ready"],
        true
    );
    assert_eq!(
        review_body["contract_mutation_enablement_gate"]["lifecycle_effects_ready"],
        false
    );
    assert_eq!(
        review_body["contract_mutation_enablement_gate"]["contract_mutation_api_enabled"],
        false
    );
    assert_eq!(
        review_body["contract_mutation_enablement_gate"]["mutation_ready"],
        false
    );
    assert_eq!(
        review_body["contract_mutation_enablement_gate"]["would_mutate_contract"],
        false
    );
    assert_eq!(
        review_body["contract_mutation_enablement_gate"]["would_write_source"],
        false
    );
    assert_eq!(
        review_body["contract_mutation_enablement_gate"]["would_touch_disk"],
        false
    );
    assert!(
        review_body["contract_mutation_enablement_gate"]["passed_gates"]
            .as_array()
            .expect("contract mutation passed gates should be an array")
            .contains(&json!("contract_source_write_ready"))
    );
    assert!(
        review_body["contract_mutation_enablement_gate"]["blocked_gates"]
            .as_array()
            .expect("contract mutation blocked gates should be an array")
            .contains(&json!("lifecycle_effects_ready"))
    );
    assert!(
        review_body["contract_mutation_enablement_gate"]["blocked_gates"]
            .as_array()
            .expect("contract mutation blocked gates should be an array")
            .contains(&json!("contract_mutation_api_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]["status"],
        "approve_execution_contract_mutation_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]["gate_name"],
        "contract_mutation_ready"
    );
    assert_eq!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]
            ["writeback_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]["source_write_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]
            ["lifecycle_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]
            ["contract_mutation_api_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]
            ["contract_mutation_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]
            ["would_mutate_contract"],
        false
    );
    assert_eq!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]["would_write_source"],
        false
    );
    assert_eq!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]
            ["would_unblock_atomic_side_effects"],
        false
    );
    assert!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]
            ["inherited_contract_mutation_blocked_gates"]
            .as_array()
            .expect("contract mutation readiness inherited blockers should be an array")
            .contains(&json!("contract_mutation_api_enabled"))
    );
    assert!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]["passed_gates"]
            .as_array()
            .expect("contract mutation readiness passed gates should be an array")
            .contains(&json!("contract_source_write_ready"))
    );
    assert_eq!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]["required_gates"]
            .as_array()
            .expect("contract mutation readiness required gates should be an array")
            .len(),
        4
    );
    assert!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("contract mutation readiness blockers should be an array")
            .contains(&json!("lifecycle_effects_ready"))
    );
    assert!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("contract mutation readiness blockers should be an array")
            .contains(&json!("contract_mutation_api_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_gate"]["status"],
        "approve_execution_ready_blocked"
    );
    assert_eq!(review_body["approve_execution_gate"]["action"], "approve");
    assert_eq!(
        review_body["approve_execution_gate"]["target_review_state"],
        "approved"
    );
    assert_eq!(
        review_body["approve_execution_gate"]["approval_persistence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_gate"]["transition_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_gate"]["lifecycle_emission_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_gate"]["lifecycle_append_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_gate"]["contract_writeback_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_gate"]["mutation_api_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_gate"]["approve_execution_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_gate"]["would_execute"],
        false
    );
    assert!(review_body["approve_execution_gate"]["blocked_by"]
        .as_array()
        .expect("approve execution gate blockers should be an array")
        .contains(&json!("lifecycle_event_emission_enabled")));
    assert!(review_body["approve_execution_gate"]["blocked_by"]
        .as_array()
        .expect("approve execution gate blockers should be an array")
        .contains(&json!("contract_mutation_api_enabled")));
    assert_eq!(
        review_body["approve_execution_transaction_dry_run"]["status"],
        "approve_execution_transaction_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_transaction_dry_run"]["approval_id"],
        approval_id
    );
    assert_eq!(
        review_body["approve_execution_transaction_dry_run"]["transaction_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_dry_run"]["execution_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_dry_run"]["approved_transition_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_dry_run"]["record_write_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_dry_run"]["lifecycle_emission_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_dry_run"]["lifecycle_append_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_dry_run"]["contract_writeback_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_dry_run"]["would_execute_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_dry_run"]["would_write_contract_source"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_dry_run"]["step_order"],
        json!([
            "transition_review_state",
            "persist_approval_record",
            "emit_lifecycle_event",
            "append_lifecycle_entry",
            "write_contract_source"
        ])
    );
    assert!(
        review_body["approve_execution_transaction_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve transaction blockers should be an array")
            .contains(&json!("lifecycle_event_emission_enabled"))
    );
    assert!(
        review_body["approve_execution_transaction_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve transaction blockers should be an array")
            .contains(&json!("contract_mutation_api_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_admission_gate"]["status"],
        "approve_execution_admission_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_admission_gate"]["approval_id"],
        approval_id
    );
    assert_eq!(
        review_body["approve_execution_admission_gate"]["transaction_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_admission_gate"]["transaction_execution_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_admission_gate"]["atomicity_scope_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_admission_gate"]["transaction_runner_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_admission_gate"]["admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_admission_gate"]["partial_execution_allowed"],
        false
    );
    assert_eq!(
        review_body["approve_execution_admission_gate"]["would_start_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_admission_gate"]["would_persist_any_side_effect"],
        false
    );
    assert!(
        review_body["approve_execution_admission_gate"]["blocked_gates"]
            .as_array()
            .expect("approve admission blockers should be an array")
            .contains(&json!("lifecycle_event_emission_enabled"))
    );
    assert!(
        review_body["approve_execution_admission_gate"]["blocked_gates"]
            .as_array()
            .expect("approve admission blockers should be an array")
            .contains(&json!("contract_mutation_api_enabled"))
    );
    assert!(
        review_body["approve_execution_admission_gate"]["blocked_gates"]
            .as_array()
            .expect("approve admission blockers should be an array")
            .contains(&json!("approve_execution_transaction_runner_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]["status"],
        "approve_execution_transaction_runner_enablement_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]["switch_name"],
        "approve_execution_transaction_runner_enabled"
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]["approve_action"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]
            ["transaction_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]
            ["transaction_execution_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]
            ["atomicity_scope_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]
            ["lifecycle_emission_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]
            ["contract_mutation_api_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]
            ["transaction_runner_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]
            ["enablement_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]
            ["runner_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]
            ["would_enable_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]
            ["would_start_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]["required_gates"]
            .as_array()
            .expect("runner enablement required gates should be an array")
            .len(),
        7
    );
    assert!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner enablement blockers should be an array")
            .contains(&json!("transaction_execution_ready"))
    );
    assert!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner enablement blockers should be an array")
            .contains(&json!("lifecycle_event_emission_enabled"))
    );
    assert!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner enablement blockers should be an array")
            .contains(&json!("approve_execution_transaction_runner_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_dry_run"]["status"],
        "approve_execution_transaction_runner_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_dry_run"]["approval_id"],
        approval_id
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_dry_run"]["runner_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_dry_run"]["admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_dry_run"]["transaction_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_dry_run"]["commit_barrier_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_dry_run"]["recovery_marker_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_dry_run"]["rollback_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_dry_run"]["commit_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_dry_run"]["would_start_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_dry_run"]["would_write_recovery_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_dry_run"]["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_dry_run"]["phase_order"],
        json!([
            "write_recovery_marker",
            "transition_review_state",
            "persist_approval_record",
            "emit_lifecycle_event",
            "append_lifecycle_entry",
            "write_contract_source",
            "clear_recovery_marker"
        ])
    );
    assert!(
        review_body["approve_execution_transaction_runner_dry_run"]["rollback_order"]
            .as_array()
            .expect("rollback order should be an array")
            .contains(&json!("restore_contract_source"))
    );
    assert!(
        review_body["approve_execution_transaction_runner_dry_run"]["blocked_by"]
            .as_array()
            .expect("runner blockers should be an array")
            .contains(&json!("approve_execution_admission_ready"))
    );
    assert!(
        review_body["approve_execution_transaction_runner_dry_run"]["blocked_by"]
            .as_array()
            .expect("runner blockers should be an array")
            .contains(&json!("approve_execution_transaction_runner_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_write_dry_run"]["status"],
        "approve_execution_recovery_marker_write_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_write_dry_run"]["approval_id"],
        approval_id
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_write_dry_run"]["marker_kind"],
        "approve_execution_recovery_marker"
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_write_dry_run"]["store_kind"],
        "contract_repair_approval_records"
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_write_dry_run"]["payload_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_write_dry_run"]["path_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_write_dry_run"]["storage_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_write_dry_run"]["runner_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_write_dry_run"]["write_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_write_dry_run"]["atomic_write_required"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_write_dry_run"]["would_write_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_write_dry_run"]["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_recovery_marker_write_dry_run"]["file_name"]
            .as_str()
            .expect("marker file name should be present")
            .ends_with(".json")
    );
    assert!(
        review_body["approve_execution_recovery_marker_write_dry_run"]["payload_fields"]
            .as_array()
            .expect("marker payload fields should be an array")
            .contains(&json!("rollback_order"))
    );
    assert!(
        review_body["approve_execution_recovery_marker_write_dry_run"]["blocked_by"]
            .as_array()
            .expect("marker blockers should be an array")
            .contains(&json!("approve_execution_transaction_runner_ready"))
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_idempotency_precheck"]["status"],
        "approve_execution_recovery_marker_idempotency_checked_blocked"
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_idempotency_precheck"]
            ["store_lookup_enabled"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_idempotency_precheck"]
            ["existing_marker_found"],
        false
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_idempotency_precheck"]["conflict_detected"],
        false
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_idempotency_precheck"]["marker_write_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_idempotency_precheck"]
            ["safe_to_write_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_idempotency_precheck"]["would_write_marker"],
        false
    );
    assert!(
        review_body["approve_execution_recovery_marker_idempotency_precheck"]["blocked_by"]
            .as_array()
            .expect("marker idempotency blockers should be an array")
            .contains(&json!("approve_execution_transaction_runner_ready"))
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_gate"]["status"],
        "approve_execution_recovery_marker_persistence_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_gate"]
            ["marker_persistence_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_gate"]["marker_write_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_gate"]["idempotency_checked"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_gate"]
            ["no_existing_marker_conflict"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_gate"]["runner_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_gate"]
            ["marker_persistence_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_gate"]["persistence_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_gate"]["would_persist_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_gate"]["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_recovery_marker_persistence_gate"]["passed_gates"]
            .as_array()
            .expect("marker persistence passed gates should be an array")
            .contains(&json!("recovery_marker_idempotency_checked"))
    );
    assert!(
        review_body["approve_execution_recovery_marker_persistence_gate"]["blocked_gates"]
            .as_array()
            .expect("marker persistence blocked gates should be an array")
            .contains(&json!("approve_execution_transaction_runner_ready"))
    );
    assert!(
        review_body["approve_execution_recovery_marker_persistence_gate"]["blocked_gates"]
            .as_array()
            .expect("marker persistence blocked gates should be an array")
            .contains(&json!("approve_recovery_marker_persistence_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]["status"],
        "approve_execution_recovery_marker_persistence_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]["gate_name"],
        "recovery_marker_persistence_ready"
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["marker_persistence_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["marker_write_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["idempotency_checked"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["no_existing_marker_conflict"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["runner_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["marker_persistence_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["recovery_marker_persistence_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["would_persist_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["would_unblock_transaction_commit"],
        false
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["would_unblock_atomic_side_effects"],
        false
    );
    assert!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["inherited_recovery_marker_persistence_blocked_gates"]
            .as_array()
            .expect("marker persistence readiness inherited blockers should be an array")
            .contains(&json!("approve_execution_transaction_runner_ready"))
    );
    assert!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["inherited_recovery_marker_persistence_blocked_gates"]
            .as_array()
            .expect("marker persistence readiness inherited blockers should be an array")
            .contains(&json!("approve_recovery_marker_persistence_enabled"))
    );
    assert!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("marker persistence readiness passed gates should be an array")
            .contains(&json!("recovery_marker_idempotency_checked"))
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("marker persistence readiness required gates should be an array")
            .len(),
        5
    );
    assert!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("marker persistence readiness blockers should be an array")
            .contains(&json!("approve_execution_transaction_runner_ready"))
    );
    assert!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("marker persistence readiness blockers should be an array")
            .contains(&json!("approve_recovery_marker_persistence_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_gate"]["status"],
        "approve_execution_transaction_commit_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_gate"]["runner_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_gate"]["runner_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_gate"]["admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_gate"]["commit_barrier_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_gate"]["rollback_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_gate"]
            ["recovery_marker_persistence_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_gate"]
            ["recovery_marker_persistence_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_gate"]["commit_gate_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_gate"]["commit_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_gate"]["would_start_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_gate"]["would_persist_recovery_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_gate"]["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_gate"]["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_transaction_commit_gate"]["passed_gates"]
            .as_array()
            .expect("transaction commit passed gates should be an array")
            .contains(&json!("recovery_marker_persistence_plan_ready"))
    );
    assert!(
        review_body["approve_execution_transaction_commit_gate"]["blocked_gates"]
            .as_array()
            .expect("transaction commit blocked gates should be an array")
            .contains(&json!("recovery_marker_persistence_ready"))
    );
    assert!(
        review_body["approve_execution_transaction_commit_gate"]["blocked_gates"]
            .as_array()
            .expect("transaction commit blocked gates should be an array")
            .contains(&json!("approve_execution_transaction_commit_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]["status"],
        "approve_execution_transaction_commit_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]["gate_name"],
        "approve_execution_transaction_commit_ready"
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]["runner_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]["runner_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]["admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]
            ["commit_barrier_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]
            ["rollback_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]
            ["recovery_marker_persistence_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]
            ["recovery_marker_persistence_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]
            ["commit_gate_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]
            ["transaction_commit_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]["would_start_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]
            ["would_persist_recovery_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]
            ["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]
            ["would_unblock_atomic_side_effects"],
        false
    );
    assert!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]
            ["inherited_transaction_commit_blocked_gates"]
            .as_array()
            .expect("transaction commit readiness inherited blockers should be an array")
            .contains(&json!("recovery_marker_persistence_ready"))
    );
    assert!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]
            ["inherited_transaction_commit_blocked_gates"]
            .as_array()
            .expect("transaction commit readiness inherited blockers should be an array")
            .contains(&json!("approve_execution_transaction_commit_enabled"))
    );
    assert!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]["passed_gates"]
            .as_array()
            .expect("transaction commit readiness passed gates should be an array")
            .contains(&json!("recovery_marker_persistence_plan_ready"))
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]["required_gates"]
            .as_array()
            .expect("transaction commit readiness required gates should be an array")
            .len(),
        7
    );
    assert!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("transaction commit readiness blockers should be an array")
            .contains(&json!("recovery_marker_persistence_ready"))
    );
    assert!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("transaction commit readiness blockers should be an array")
            .contains(&json!("approve_execution_transaction_commit_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["status"],
        "approve_execution_atomic_side_effects_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["atomic_side_effects_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["lifecycle_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["contract_mutation_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]
            ["recovery_marker_persistence_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["transaction_commit_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["lifecycle_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["contract_mutation_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]
            ["recovery_marker_persistence_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["transaction_commit_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["atomic_side_effects_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["atomic_side_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["would_emit_lifecycle"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["would_mutate_contract"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["would_persist_recovery_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_atomic_side_effects_gate"]["passed_gates"]
            .as_array()
            .expect("atomic side effects passed gates should be an array")
            .contains(&json!("contract_mutation_plan_ready"))
    );
    assert!(
        review_body["approve_execution_atomic_side_effects_gate"]["blocked_gates"]
            .as_array()
            .expect("atomic side effects blocked gates should be an array")
            .contains(&json!("contract_mutation_ready"))
    );
    assert!(
        review_body["approve_execution_atomic_side_effects_gate"]["blocked_gates"]
            .as_array()
            .expect("atomic side effects blocked gates should be an array")
            .contains(&json!("approve_execution_atomic_side_effects_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]["status"],
        "approve_execution_atomic_side_effects_enablement_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]["switch_name"],
        "approve_execution_atomic_side_effects_enabled"
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["atomic_side_effects_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["lifecycle_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["contract_mutation_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["recovery_marker_persistence_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["transaction_commit_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["atomic_side_effects_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["enablement_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["atomic_side_effects_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["would_enable_atomic_side_effects"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["would_emit_lifecycle"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["would_mutate_contract"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["would_persist_recovery_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["would_unblock_atomic_side_effects_readiness"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["would_unblock_runner_attempt_readiness"],
        false
    );
    assert!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["inherited_atomic_side_effects_blocked_gates"]
            .as_array()
            .expect("atomic side effects enablement inherited blockers should be an array")
            .contains(&json!("contract_mutation_ready"))
    );
    assert!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["inherited_atomic_side_effects_blocked_gates"]
            .as_array()
            .expect("atomic side effects enablement inherited blockers should be an array")
            .contains(&json!("approve_execution_atomic_side_effects_enabled"))
    );
    assert!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]["passed_gates"]
            .as_array()
            .expect("atomic side effects enablement passed gates should be an array")
            .contains(&json!("approve_execution_atomic_side_effects_plan_ready"))
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]["required_gates"]
            .as_array()
            .expect("atomic side effects enablement required gates should be an array")
            .len(),
        6
    );
    assert!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("atomic side effects enablement blockers should be an array")
            .contains(&json!("lifecycle_effects_ready"))
    );
    assert!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("atomic side effects enablement blockers should be an array")
            .contains(&json!("approve_execution_atomic_side_effects_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]["status"],
        "approve_execution_atomic_side_effects_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]["gate_name"],
        "approve_execution_atomic_side_effects_ready"
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]
            ["atomic_side_effects_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]
            ["lifecycle_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]
            ["contract_mutation_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]
            ["recovery_marker_persistence_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]
            ["transaction_commit_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]
            ["atomic_side_effects_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]
            ["atomic_side_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]
            ["would_emit_lifecycle"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]
            ["would_mutate_contract"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]
            ["would_persist_recovery_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]
            ["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]
            ["would_unblock_runner_attempt_readiness"],
        false
    );
    assert!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]
            ["inherited_atomic_side_effects_blocked_gates"]
            .as_array()
            .expect("atomic side effects readiness inherited blockers should be an array")
            .contains(&json!("contract_mutation_ready"))
    );
    assert!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]
            ["inherited_atomic_side_effects_blocked_gates"]
            .as_array()
            .expect("atomic side effects readiness inherited blockers should be an array")
            .contains(&json!("approve_execution_atomic_side_effects_enabled"))
    );
    assert!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]["passed_gates"]
            .as_array()
            .expect("atomic side effects readiness passed gates should be an array")
            .contains(&json!("approve_execution_atomic_side_effects_plan_ready"))
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]["required_gates"]
            .as_array()
            .expect("atomic side effects readiness required gates should be an array")
            .len(),
        6
    );
    assert!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("atomic side effects readiness blockers should be an array")
            .contains(&json!("lifecycle_effects_ready"))
    );
    assert!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("atomic side effects readiness blockers should be an array")
            .contains(&json!("approve_execution_atomic_side_effects_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]["status"],
        "approve_execution_runner_attempt_enablement_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]["switch_name"],
        "approve_execution_runner_attempt_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]["attempt_requested"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]
            ["atomic_side_effects_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]
            ["atomic_side_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]
            ["transaction_runner_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]
            ["runner_attempt_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]
            ["enablement_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]
            ["runner_attempt_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]
            ["would_enable_runner_attempt"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]["would_start_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]
            ["would_persist_any_side_effect"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]
            ["would_rollback_on_error"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]["required_gates"]
            .as_array()
            .expect("runner attempt enablement required gates should be an array")
            .len(),
        5
    );
    assert!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner attempt enablement blockers should be an array")
            .contains(&json!("approve_execution_atomic_side_effects_ready"))
    );
    assert!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner attempt enablement blockers should be an array")
            .contains(&json!(
                "approve_execution_transaction_runner_enablement_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner attempt enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_attempt_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt"]["status"],
        "approve_execution_runner_attempt_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt"]["attempt_requested"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt"]["atomic_side_effects_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt"]["atomic_side_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt"]["runner_attempt_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt"]["runner_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt"]["would_start_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt"]["would_persist_any_side_effect"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt"]["would_rollback_on_error"],
        false
    );
    assert!(
        review_body["approve_execution_runner_attempt"]["blocked_by"]
            .as_array()
            .expect("approve runner attempt blockers should be an array")
            .contains(&json!("approve_execution_atomic_side_effects_ready"))
    );
    assert!(
        review_body["approve_execution_runner_attempt"]["blocked_by"]
            .as_array()
            .expect("approve runner attempt blockers should be an array")
            .contains(&json!("approve_execution_runner_attempt_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]["status"],
        "approve_execution_runner_attempt_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]["gate_name"],
        "approve_execution_runner_attempt_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]["attempt_requested"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]
            ["atomic_side_effects_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]
            ["atomic_side_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]
            ["transaction_runner_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]["runner_attempt_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]
            ["runner_attempt_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]["runner_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]
            ["would_enable_runner_attempt"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]["would_start_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]
            ["would_persist_any_side_effect"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]
            ["would_rollback_on_error"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]
            ["would_unblock_control_readiness"],
        false
    );
    assert!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]
            ["inherited_runner_attempt_blockers"]
            .as_array()
            .expect("runner attempt readiness inherited blockers should be an array")
            .contains(&json!("approve_execution_atomic_side_effects_ready"))
    );
    assert!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]
            ["inherited_runner_attempt_blockers"]
            .as_array()
            .expect("runner attempt readiness inherited blockers should be an array")
            .contains(&json!("approve_execution_runner_attempt_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]
            ["inherited_attempt_enablement_blocked_gates"]
            .as_array()
            .expect("runner attempt readiness inherited enablement blockers should be an array")
            .contains(&json!(
                "approve_execution_transaction_runner_enablement_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]
            ["inherited_attempt_enablement_blocked_gates"]
            .as_array()
            .expect("runner attempt readiness inherited enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_attempt_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]["required_gates"]
            .as_array()
            .expect("runner attempt readiness required gates should be an array")
            .len(),
        3
    );
    assert!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner attempt readiness blockers should be an array")
            .contains(&json!("approve_execution_atomic_side_effects_ready"))
    );
    assert!(
        review_body["approve_execution_runner_attempt_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner attempt readiness blockers should be an array")
            .contains(&json!("approve_execution_runner_attempt_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_enablement_dry_run"]["status"],
        "approve_execution_runner_execution_enablement_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_enablement_dry_run"]["switch_name"],
        "approve_execution_runner_execution_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_enablement_dry_run"]["branch_selected"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_enablement_dry_run"]
            ["runner_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_enablement_dry_run"]
            ["atomic_side_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_enablement_dry_run"]
            ["runner_attempt_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_enablement_dry_run"]
            ["runner_execution_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_enablement_dry_run"]
            ["enablement_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_enablement_dry_run"]
            ["runner_execution_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_enablement_dry_run"]
            ["would_enable_runner_execution"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_enablement_dry_run"]["would_start_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_enablement_dry_run"]
            ["would_persist_any_side_effect"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_enablement_dry_run"]
            ["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_enablement_dry_run"]
            ["would_rollback_on_error"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_enablement_dry_run"]["required_gates"]
            .as_array()
            .expect("runner execution enablement required gates should be an array")
            .len(),
        5
    );
    assert!(
        review_body["approve_execution_runner_execution_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner execution enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_attempt_ready"))
    );
    assert!(
        review_body["approve_execution_runner_execution_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner execution enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_attempt_enablement_ready"))
    );
    assert!(
        review_body["approve_execution_runner_execution_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner execution enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_execution_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]["status"],
        "approve_execution_runner_execution_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]["gate_name"],
        "approve_execution_runner_execution_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]["branch_selected"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]["runner_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]
            ["atomic_side_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]
            ["runner_execution_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]
            ["runner_execution_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]
            ["runner_execution_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]
            ["would_enable_runner_execution"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]["would_start_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]
            ["would_persist_any_side_effect"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]
            ["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]
            ["would_rollback_on_error"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]
            ["would_unblock_route_dispatch"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]
            ["would_unblock_control_readiness"],
        false
    );
    assert!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]
            ["inherited_runner_outcome_blockers"]
            .as_array()
            .expect("runner execution readiness inherited outcome blockers should be an array")
            .contains(&json!("approve_execution_runner_attempt_ready"))
    );
    assert!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]
            ["inherited_runner_outcome_blockers"]
            .as_array()
            .expect("runner execution readiness inherited outcome blockers should be an array")
            .contains(&json!("approve_execution_runner_execution_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]
            ["inherited_execution_enablement_blocked_gates"]
            .as_array()
            .expect("runner execution readiness inherited switch blockers should be an array")
            .contains(&json!("approve_execution_runner_attempt_enablement_ready"))
    );
    assert!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]
            ["inherited_execution_enablement_blocked_gates"]
            .as_array()
            .expect("runner execution readiness inherited switch blockers should be an array")
            .contains(&json!("approve_execution_runner_execution_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]["required_gates"]
            .as_array()
            .expect("runner execution readiness required gates should be an array")
            .len(),
        4
    );
    assert!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner execution readiness blockers should be an array")
            .contains(&json!("approve_execution_runner_attempt_ready"))
    );
    assert!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner execution readiness blockers should be an array")
            .contains(&json!("approve_execution_atomic_side_effects_ready"))
    );
    assert!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner execution readiness blockers should be an array")
            .contains(&json!("approve_execution_runner_execution_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_outcome"]["status"],
        "approve_execution_runner_outcome_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_outcome"]["branch_selected"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_outcome"]["runner_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_outcome"]["atomic_side_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_outcome"]["runner_execution_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_outcome"]["runner_execution_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_outcome"]["would_start_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_outcome"]["would_persist_any_side_effect"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_outcome"]["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_outcome"]["would_rollback_on_error"],
        false
    );
    assert!(
        review_body["approve_execution_runner_outcome"]["blocked_by"]
            .as_array()
            .expect("approve runner outcome blockers should be an array")
            .contains(&json!("approve_execution_runner_attempt_ready"))
    );
    assert!(
        review_body["approve_execution_runner_outcome"]["blocked_by"]
            .as_array()
            .expect("approve runner outcome blockers should be an array")
            .contains(&json!("approve_execution_runner_execution_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_route_dispatch_enablement_dry_run"]["status"],
        "approve_execution_runner_route_dispatch_enablement_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_route_dispatch_enablement_dry_run"]["switch_name"],
        "approve_execution_runner_route_dispatch_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_route_dispatch_enablement_dry_run"]
            ["branch_selected"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_route_dispatch_enablement_dry_run"]
            ["runner_execution_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_dispatch_enablement_dry_run"]
            ["runner_execution_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_dispatch_enablement_dry_run"]
            ["route_dispatch_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_dispatch_enablement_dry_run"]
            ["enablement_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_dispatch_enablement_dry_run"]
            ["route_dispatch_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_dispatch_enablement_dry_run"]
            ["would_enable_route_dispatch"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_dispatch_enablement_dry_run"]
            ["would_enter_runner_branch"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_dispatch_enablement_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_dispatch_enablement_dry_run"]
            ["would_persist_any_side_effect"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_dispatch_enablement_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_dispatch_enablement_dry_run"]["required_gates"]
            .as_array()
            .expect("runner route dispatch enablement required gates should be an array")
            .len(),
        4
    );
    assert!(
        review_body["approve_execution_runner_route_dispatch_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner route dispatch enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_execution_ready"))
    );
    assert!(
        review_body["approve_execution_runner_route_dispatch_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner route dispatch enablement blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_execution_enablement_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_route_dispatch_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner route dispatch enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_route_dispatch_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_gate"]["status"],
        "approve_execution_runner_dispatch_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_gate"]["branch_selected"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_gate"]["runner_execution_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_gate"]["route_dispatch_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_gate"]["dispatch_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_gate"]["would_enter_runner_branch"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_gate"]["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_gate"]["would_persist_any_side_effect"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_gate"]["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_dispatch_gate"]["passed_gates"]
            .as_array()
            .expect("approve runner dispatch passed gates should be an array")
            .contains(&json!("approve_execution_runner_branch_selected"))
    );
    assert!(
        review_body["approve_execution_runner_dispatch_gate"]["blocked_gates"]
            .as_array()
            .expect("approve runner dispatch blocked gates should be an array")
            .contains(&json!("approve_execution_runner_execution_ready"))
    );
    assert!(
        review_body["approve_execution_runner_dispatch_gate"]["blocked_gates"]
            .as_array()
            .expect("approve runner dispatch blocked gates should be an array")
            .contains(&json!("approve_execution_runner_route_dispatch_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]["status"],
        "approve_execution_runner_dispatch_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]["gate_name"],
        "approve_execution_runner_dispatch_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]["branch_selected"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]
            ["runner_execution_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]
            ["route_dispatch_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]
            ["route_dispatch_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]["dispatch_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]
            ["would_enter_runner_branch"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]
            ["would_persist_any_side_effect"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]["would_unblock_handoff"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]
            ["would_unblock_control_readiness"],
        false
    );
    assert!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]
            ["inherited_dispatch_blocked_gates"]
            .as_array()
            .expect("runner dispatch readiness inherited dispatch blockers should be an array")
            .contains(&json!("approve_execution_runner_execution_ready"))
    );
    assert!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]
            ["inherited_dispatch_blocked_gates"]
            .as_array()
            .expect("runner dispatch readiness inherited dispatch blockers should be an array")
            .contains(&json!("approve_execution_runner_route_dispatch_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]
            ["inherited_route_dispatch_enablement_blocked_gates"]
            .as_array()
            .expect("runner dispatch readiness inherited route blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_execution_enablement_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]
            ["inherited_route_dispatch_enablement_blocked_gates"]
            .as_array()
            .expect("runner dispatch readiness inherited route blockers should be an array")
            .contains(&json!("approve_execution_runner_route_dispatch_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]["required_gates"]
            .as_array()
            .expect("runner dispatch readiness required gates should be an array")
            .len(),
        3
    );
    assert!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]["passed_gates"]
            .as_array()
            .expect("runner dispatch readiness passed gates should be an array")
            .contains(&json!("approve_execution_runner_branch_selected"))
    );
    assert!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner dispatch readiness blockers should be an array")
            .contains(&json!("approve_execution_runner_execution_ready"))
    );
    assert!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner dispatch readiness blockers should be an array")
            .contains(&json!("approve_execution_runner_route_dispatch_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff"]["status"],
        "approve_execution_runner_handoff_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff"]["dispatch_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff"]["route_dispatch_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff"]["handoff_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff"]["expected_http_status"],
        423
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff"]["expected_route_status"],
        "review_decision_execution_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff"]["would_call_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff"]["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff"]["would_persist_any_side_effect"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff"]["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_handoff"]["blocked_by"]
            .as_array()
            .expect("approve runner handoff blockers should be an array")
            .contains(&json!("approve_execution_runner_dispatch_ready"))
    );
    assert!(
        review_body["approve_execution_runner_handoff"]["blocked_by"]
            .as_array()
            .expect("approve runner handoff blockers should be an array")
            .contains(&json!("approve_execution_runner_route_dispatch_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]["status"],
        "approve_execution_runner_handoff_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]["gate_name"],
        "approve_execution_runner_handoff_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]["dispatch_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]["route_dispatch_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]
            ["dispatch_readiness_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]["handoff_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]["expected_http_status"],
        423
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]["expected_route_status"],
        "review_decision_execution_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]["would_call_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]
            ["would_persist_any_side_effect"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]["would_unblock_call"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]
            ["would_unblock_control_readiness"],
        false
    );
    assert!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]
            ["inherited_handoff_blockers"]
            .as_array()
            .expect("runner handoff readiness inherited handoff blockers should be an array")
            .contains(&json!("approve_execution_runner_dispatch_ready"))
    );
    assert!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]
            ["inherited_handoff_blockers"]
            .as_array()
            .expect("runner handoff readiness inherited handoff blockers should be an array")
            .contains(&json!("approve_execution_runner_route_dispatch_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]
            ["inherited_dispatch_readiness_blocked_gates"]
            .as_array()
            .expect("runner handoff readiness inherited dispatch blockers should be an array")
            .contains(&json!("approve_execution_runner_execution_ready"))
    );
    assert!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]
            ["inherited_dispatch_readiness_blocked_gates"]
            .as_array()
            .expect("runner handoff readiness inherited dispatch blockers should be an array")
            .contains(&json!("approve_execution_runner_route_dispatch_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]["required_gates"]
            .as_array()
            .expect("runner handoff readiness required gates should be an array")
            .len(),
        2
    );
    assert!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner handoff readiness blockers should be an array")
            .contains(&json!("approve_execution_runner_dispatch_ready"))
    );
    assert!(
        review_body["approve_execution_runner_handoff_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner handoff readiness blockers should be an array")
            .contains(&json!("approve_execution_runner_route_dispatch_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_call_enablement_dry_run"]["status"],
        "approve_execution_runner_call_enablement_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_call_enablement_dry_run"]["switch_name"],
        "approve_execution_runner_call_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_call_enablement_dry_run"]["handoff_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_enablement_dry_run"]
            ["route_dispatch_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_enablement_dry_run"]["runner_call_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_enablement_dry_run"]
            ["enablement_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_enablement_dry_run"]
            ["runner_call_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_enablement_dry_run"]["would_enable_runner_call"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_enablement_dry_run"]["would_call_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_enablement_dry_run"]["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_enablement_dry_run"]
            ["would_persist_any_side_effect"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_enablement_dry_run"]["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_enablement_dry_run"]["would_rollback_on_error"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_enablement_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_enablement_dry_run"]["required_gates"]
            .as_array()
            .expect("runner call enablement required gates should be an array")
            .len(),
        3
    );
    assert!(
        review_body["approve_execution_runner_call_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner call enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_handoff_ready"))
    );
    assert!(
        review_body["approve_execution_runner_call_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner call enablement blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_route_dispatch_enablement_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_call_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner call enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_call_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_call_dry_run"]["status"],
        "approve_execution_runner_call_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_call_dry_run"]["handoff_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_dry_run"]["runner_call_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_dry_run"]["call_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_dry_run"]["expected_runner_result"],
        "not_invoked"
    );
    assert_eq!(
        review_body["approve_execution_runner_call_dry_run"]["would_call_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_dry_run"]["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_dry_run"]["would_persist_any_side_effect"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_dry_run"]["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_dry_run"]["would_rollback_on_error"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_dry_run"]["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_call_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner call blockers should be an array")
            .contains(&json!("approve_execution_runner_handoff_ready"))
    );
    assert!(
        review_body["approve_execution_runner_call_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner call blockers should be an array")
            .contains(&json!("approve_execution_runner_call_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_call_readiness_dry_run"]["status"],
        "approve_execution_runner_call_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_call_readiness_dry_run"]["gate_name"],
        "approve_execution_runner_call_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_call_readiness_dry_run"]["handoff_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_readiness_dry_run"]["runner_call_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_readiness_dry_run"]
            ["runner_call_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_readiness_dry_run"]["call_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_readiness_dry_run"]["expected_runner_result"],
        "not_invoked"
    );
    assert_eq!(
        review_body["approve_execution_runner_call_readiness_dry_run"]["would_call_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_readiness_dry_run"]["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_readiness_dry_run"]
            ["would_persist_any_side_effect"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_readiness_dry_run"]["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_readiness_dry_run"]["would_rollback_on_error"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_readiness_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_readiness_dry_run"]["would_unblock_body"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_readiness_dry_run"]
            ["would_unblock_control_readiness"],
        false
    );
    assert!(
        review_body["approve_execution_runner_call_readiness_dry_run"]["inherited_call_blockers"]
            .as_array()
            .expect("runner call readiness inherited call blockers should be an array")
            .contains(&json!("approve_execution_runner_handoff_ready"))
    );
    assert!(
        review_body["approve_execution_runner_call_readiness_dry_run"]["inherited_call_blockers"]
            .as_array()
            .expect("runner call readiness inherited call blockers should be an array")
            .contains(&json!("approve_execution_runner_call_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_call_readiness_dry_run"]
            ["inherited_call_enablement_blocked_gates"]
            .as_array()
            .expect("runner call readiness inherited switch blockers should be an array")
            .contains(&json!("approve_execution_runner_handoff_ready"))
    );
    assert!(
        review_body["approve_execution_runner_call_readiness_dry_run"]
            ["inherited_call_enablement_blocked_gates"]
            .as_array()
            .expect("runner call readiness inherited switch blockers should be an array")
            .contains(&json!("approve_execution_runner_call_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_call_readiness_dry_run"]["required_gates"]
            .as_array()
            .expect("runner call readiness required gates should be an array")
            .len(),
        2
    );
    assert!(
        review_body["approve_execution_runner_call_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner call readiness blockers should be an array")
            .contains(&json!("approve_execution_runner_handoff_ready"))
    );
    assert!(
        review_body["approve_execution_runner_call_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner call readiness blockers should be an array")
            .contains(&json!("approve_execution_runner_call_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_body_enablement_dry_run"]["status"],
        "approve_execution_runner_body_enablement_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_body_enablement_dry_run"]["switch_name"],
        "approve_execution_runner_body_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_body_enablement_dry_run"]["call_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_enablement_dry_run"]["side_effect_bundle_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_body_enablement_dry_run"]
            ["atomic_side_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_enablement_dry_run"]["runner_body_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_enablement_dry_run"]
            ["enablement_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_enablement_dry_run"]
            ["runner_body_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_enablement_dry_run"]["would_enable_runner_body"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_enablement_dry_run"]["would_enter_body"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_enablement_dry_run"]["would_emit_lifecycle"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_enablement_dry_run"]["would_mutate_contract"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_enablement_dry_run"]
            ["would_persist_recovery_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_enablement_dry_run"]["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_enablement_dry_run"]["would_rollback_on_error"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_enablement_dry_run"]["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_enablement_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_enablement_dry_run"]["required_gates"]
            .as_array()
            .expect("runner body enablement gates should be an array")
            .len(),
        4
    );
    assert!(
        review_body["approve_execution_runner_body_enablement_dry_run"]["passed_gates"]
            .as_array()
            .expect("runner body enablement passed gates should be an array")
            .contains(&json!("approve_execution_side_effect_bundle_ready"))
    );
    assert!(
        review_body["approve_execution_runner_body_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner body enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_call_ready"))
    );
    assert!(
        review_body["approve_execution_runner_body_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner body enablement blockers should be an array")
            .contains(&json!("approve_execution_atomic_side_effects_ready"))
    );
    assert!(
        review_body["approve_execution_runner_body_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner body enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_body_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_call_body_dry_run"]["status"],
        "approve_execution_runner_call_body_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_call_body_dry_run"]["runner_entrypoint"],
        "contract_repair_approval_approve_execution_runner"
    );
    assert_eq!(
        review_body["approve_execution_runner_call_body_dry_run"]["call_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_body_dry_run"]["side_effect_bundle_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_call_body_dry_run"]["atomic_side_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_body_dry_run"]["runner_body_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_body_dry_run"]["body_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_body_dry_run"]["would_enter_body"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_body_dry_run"]["would_emit_lifecycle"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_body_dry_run"]["would_mutate_contract"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_body_dry_run"]["would_persist_recovery_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_body_dry_run"]["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_body_dry_run"]["would_rollback_on_error"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_body_dry_run"]["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_call_body_dry_run"]["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_call_body_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner body blockers should be an array")
            .contains(&json!("approve_execution_runner_call_ready"))
    );
    assert!(
        review_body["approve_execution_runner_call_body_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner body blockers should be an array")
            .contains(&json!("approve_execution_runner_body_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["status"],
        "approve_execution_runner_body_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["gate_name"],
        "approve_execution_runner_body_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["runner_entrypoint"],
        "contract_repair_approval_approve_execution_runner"
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["call_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["side_effect_bundle_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["atomic_side_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["runner_body_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]
            ["runner_body_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["body_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["would_enter_body"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["would_emit_lifecycle"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["would_mutate_contract"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]
            ["would_persist_recovery_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["would_rollback_on_error"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]
            ["would_unblock_phase_sequence"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]
            ["would_unblock_control_readiness"],
        false
    );
    assert!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["inherited_body_blockers"]
            .as_array()
            .expect("runner body readiness inherited body blockers should be an array")
            .contains(&json!("approve_execution_runner_call_ready"))
    );
    assert!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["inherited_body_blockers"]
            .as_array()
            .expect("runner body readiness inherited body blockers should be an array")
            .contains(&json!("approve_execution_runner_body_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_body_readiness_dry_run"]
            ["inherited_body_enablement_blocked_gates"]
            .as_array()
            .expect("runner body readiness inherited switch blockers should be an array")
            .contains(&json!("approve_execution_runner_call_ready"))
    );
    assert!(
        review_body["approve_execution_runner_body_readiness_dry_run"]
            ["inherited_body_enablement_blocked_gates"]
            .as_array()
            .expect("runner body readiness inherited switch blockers should be an array")
            .contains(&json!("approve_execution_runner_body_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["required_gates"]
            .as_array()
            .expect("runner body readiness required gates should be an array")
            .len(),
        4
    );
    assert!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["passed_gates"]
            .as_array()
            .expect("runner body readiness passed gates should be an array")
            .contains(&json!("approve_execution_side_effect_bundle_ready"))
    );
    assert!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner body readiness blockers should be an array")
            .contains(&json!("approve_execution_runner_call_ready"))
    );
    assert!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner body readiness blockers should be an array")
            .contains(&json!("approve_execution_atomic_side_effects_ready"))
    );
    assert!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner body readiness blockers should be an array")
            .contains(&json!("approve_execution_runner_body_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]["status"],
        "approve_execution_runner_phase_execution_enablement_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]["switch_name"],
        "approve_execution_runner_phase_execution_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]["body_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]
            ["phase_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]
            ["rollback_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]
            ["phase_execution_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]
            ["enablement_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]
            ["phase_execution_enablement_ready"],
        false
    );
    assert!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]["phase_order"]
            .as_array()
            .expect("phase execution enablement phase order should be an array")
            .contains(&json!("emit_lifecycle_event"))
    );
    assert!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]["phase_order"]
            .as_array()
            .expect("phase execution enablement phase order should be an array")
            .contains(&json!("write_contract_source"))
    );
    assert!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]
            ["rollback_order"]
            .as_array()
            .expect("phase execution enablement rollback order should be an array")
            .contains(&json!("restore_contract_source"))
    );
    assert_eq!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]
            ["would_enable_phase_execution"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]
            ["would_execute_phase_sequence"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]
            ["would_execute_rollback_sequence"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("phase execution enablement gates should be an array")
            .len(),
        4
    );
    assert!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]["passed_gates"]
            .as_array()
            .expect("phase execution enablement passed gates should be an array")
            .contains(&json!("approve_execution_runner_phase_sequence_ready"))
    );
    assert!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]["passed_gates"]
            .as_array()
            .expect("phase execution enablement passed gates should be an array")
            .contains(&json!("approve_execution_runner_rollback_sequence_ready"))
    );
    assert!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("phase execution enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_body_ready"))
    );
    assert!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("phase execution enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_phase_execution_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_body_phase_sequence_dry_run"]["status"],
        "approve_execution_runner_body_phase_sequence_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_body_phase_sequence_dry_run"]["body_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_phase_sequence_dry_run"]["phase_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_body_phase_sequence_dry_run"]
            ["rollback_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_body_phase_sequence_dry_run"]
            ["phase_execution_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_phase_sequence_dry_run"]["phases_ready"],
        false
    );
    assert!(
        review_body["approve_execution_runner_body_phase_sequence_dry_run"]["phase_order"]
            .as_array()
            .expect("approve runner phase order should be an array")
            .contains(&json!("emit_lifecycle_event"))
    );
    assert!(
        review_body["approve_execution_runner_body_phase_sequence_dry_run"]["phase_order"]
            .as_array()
            .expect("approve runner phase order should be an array")
            .contains(&json!("write_contract_source"))
    );
    assert!(
        review_body["approve_execution_runner_body_phase_sequence_dry_run"]["rollback_order"]
            .as_array()
            .expect("approve runner rollback order should be an array")
            .contains(&json!("restore_contract_source"))
    );
    assert_eq!(
        review_body["approve_execution_runner_body_phase_sequence_dry_run"]
            ["would_execute_phase_sequence"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_phase_sequence_dry_run"]
            ["would_execute_rollback_sequence"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_phase_sequence_dry_run"]["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_body_phase_sequence_dry_run"]["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_body_phase_sequence_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner phases blockers should be an array")
            .contains(&json!("approve_execution_runner_body_ready"))
    );
    assert!(
        review_body["approve_execution_runner_body_phase_sequence_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner phases blockers should be an array")
            .contains(&json!("approve_execution_runner_phase_execution_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]["status"],
        "approve_execution_runner_phases_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]["gate_name"],
        "approve_execution_runner_phases_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]["body_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]["phase_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]["rollback_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]["phase_execution_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]
            ["phase_execution_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]["phases_ready"],
        false
    );
    assert!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]["phase_order"]
            .as_array()
            .expect("phases readiness phase order should be an array")
            .contains(&json!("emit_lifecycle_event"))
    );
    assert!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]["phase_order"]
            .as_array()
            .expect("phases readiness phase order should be an array")
            .contains(&json!("write_contract_source"))
    );
    assert!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]["rollback_order"]
            .as_array()
            .expect("phases readiness rollback order should be an array")
            .contains(&json!("restore_contract_source"))
    );
    assert_eq!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]
            ["would_execute_phase_sequence"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]
            ["would_execute_rollback_sequence"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]
            ["would_unblock_lifecycle_phase"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]
            ["would_unblock_control_readiness"],
        false
    );
    assert!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]
            ["inherited_phase_sequence_blockers"]
            .as_array()
            .expect("phases readiness inherited phase blockers should be an array")
            .contains(&json!("approve_execution_runner_body_ready"))
    );
    assert!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]
            ["inherited_phase_sequence_blockers"]
            .as_array()
            .expect("phases readiness inherited phase blockers should be an array")
            .contains(&json!("approve_execution_runner_phase_execution_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]
            ["inherited_phase_execution_enablement_blocked_gates"]
            .as_array()
            .expect("phases readiness inherited switch blockers should be an array")
            .contains(&json!("approve_execution_runner_body_ready"))
    );
    assert!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]
            ["inherited_phase_execution_enablement_blocked_gates"]
            .as_array()
            .expect("phases readiness inherited switch blockers should be an array")
            .contains(&json!("approve_execution_runner_phase_execution_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]["required_gates"]
            .as_array()
            .expect("phases readiness required gates should be an array")
            .len(),
        4
    );
    assert!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]["passed_gates"]
            .as_array()
            .expect("phases readiness passed gates should be an array")
            .contains(&json!("approve_execution_runner_phase_sequence_ready"))
    );
    assert!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]["passed_gates"]
            .as_array()
            .expect("phases readiness passed gates should be an array")
            .contains(&json!("approve_execution_runner_rollback_sequence_ready"))
    );
    assert!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("phases readiness blockers should be an array")
            .contains(&json!("approve_execution_runner_body_ready"))
    );
    assert!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("phases readiness blockers should be an array")
            .contains(&json!("approve_execution_runner_phase_execution_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]["status"],
        "approve_execution_runner_lifecycle_phase_enablement_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]["switch_name"],
        "approve_execution_runner_lifecycle_phase_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]
            ["phase_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]
            ["phase_execution_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]
            ["lifecycle_phase_present"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]
            ["lifecycle_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]
            ["lifecycle_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]
            ["lifecycle_phase_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]
            ["enablement_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]
            ["lifecycle_phase_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]
            ["would_enable_lifecycle_phase"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]
            ["would_emit_lifecycle"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]
            ["would_append_lifecycle"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]
            ["would_touch_lifecycle_log"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]
            ["would_continue_to_next_phase"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("lifecycle phase enablement gates should be an array")
            .len(),
        6
    );
    assert!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]["passed_gates"]
            .as_array()
            .expect("lifecycle phase enablement passed gates should be an array")
            .contains(&json!("approve_execution_runner_phase_sequence_ready"))
    );
    assert!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]["passed_gates"]
            .as_array()
            .expect("lifecycle phase enablement passed gates should be an array")
            .contains(&json!("approve_execution_runner_lifecycle_phase_present"))
    );
    assert!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]["passed_gates"]
            .as_array()
            .expect("lifecycle phase enablement passed gates should be an array")
            .contains(&json!("lifecycle_emission_plan_ready"))
    );
    assert!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("lifecycle phase enablement blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_phase_execution_enablement_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("lifecycle phase enablement blockers should be an array")
            .contains(&json!("lifecycle_effects_ready"))
    );
    assert!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("lifecycle phase enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_lifecycle_phase_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_dry_run"]["status"],
        "approve_execution_runner_lifecycle_phase_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_dry_run"]["phase_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_dry_run"]["lifecycle_phase_present"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_dry_run"]["lifecycle_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_dry_run"]["lifecycle_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_dry_run"]["lifecycle_phase_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_dry_run"]["lifecycle_phase_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_dry_run"]["would_emit_lifecycle"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_dry_run"]["would_append_lifecycle"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_dry_run"]
            ["would_touch_lifecycle_log"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_dry_run"]
            ["would_continue_to_next_phase"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_dry_run"]["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_dry_run"]["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_lifecycle_phase_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner lifecycle blockers should be an array")
            .contains(&json!("lifecycle_effects_ready"))
    );
    assert!(
        review_body["approve_execution_runner_lifecycle_phase_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner lifecycle blockers should be an array")
            .contains(&json!("approve_execution_runner_lifecycle_phase_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]["status"],
        "approve_execution_runner_lifecycle_phase_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]["gate_name"],
        "approve_execution_runner_lifecycle_phase_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["phase_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["phase_execution_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["lifecycle_phase_present"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["lifecycle_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["lifecycle_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["lifecycle_phase_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["lifecycle_phase_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["lifecycle_phase_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["would_emit_lifecycle"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["would_append_lifecycle"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["would_touch_lifecycle_log"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["would_continue_to_next_phase"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["would_unblock_source_mutation_phase"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["would_unblock_control_readiness"],
        false
    );
    assert!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["inherited_lifecycle_phase_blockers"]
            .as_array()
            .expect("lifecycle readiness inherited phase blockers should be an array")
            .contains(&json!("lifecycle_effects_ready"))
    );
    assert!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["inherited_lifecycle_phase_blockers"]
            .as_array()
            .expect("lifecycle readiness inherited phase blockers should be an array")
            .contains(&json!("approve_execution_runner_lifecycle_phase_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["inherited_lifecycle_phase_enablement_blocked_gates"]
            .as_array()
            .expect("lifecycle readiness inherited switch blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_phase_execution_enablement_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["inherited_lifecycle_phase_enablement_blocked_gates"]
            .as_array()
            .expect("lifecycle readiness inherited switch blockers should be an array")
            .contains(&json!("approve_execution_runner_lifecycle_phase_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]["required_gates"]
            .as_array()
            .expect("lifecycle readiness required gates should be an array")
            .len(),
        5
    );
    assert!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]["passed_gates"]
            .as_array()
            .expect("lifecycle readiness passed gates should be an array")
            .contains(&json!("approve_execution_runner_phase_sequence_ready"))
    );
    assert!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]["passed_gates"]
            .as_array()
            .expect("lifecycle readiness passed gates should be an array")
            .contains(&json!("approve_execution_runner_lifecycle_phase_present"))
    );
    assert!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]["passed_gates"]
            .as_array()
            .expect("lifecycle readiness passed gates should be an array")
            .contains(&json!("lifecycle_emission_plan_ready"))
    );
    assert!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("lifecycle readiness blockers should be an array")
            .contains(&json!("lifecycle_effects_ready"))
    );
    assert!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("lifecycle readiness blockers should be an array")
            .contains(&json!("approve_execution_runner_lifecycle_phase_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]["status"],
        "approve_execution_runner_source_mutation_phase_enablement_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["switch_name"],
        "approve_execution_runner_source_mutation_phase_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["target_kind"],
        "memory_field"
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["phase_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["lifecycle_phase_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["lifecycle_phase_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["source_mutation_phase_present"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["writeback_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["source_write_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["lifecycle_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["contract_mutation_api_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["contract_mutation_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["source_mutation_phase_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["enablement_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["source_mutation_phase_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["would_enable_source_mutation_phase"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["would_mutate_contract"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["would_write_contract_source"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["would_continue_to_next_phase"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("source mutation enablement gates should be an array")
            .len(),
        10
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("source mutation enablement passed gates should be an array")
            .contains(&json!("approve_execution_runner_phase_sequence_ready"))
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("source mutation enablement passed gates should be an array")
            .contains(&json!(
                "approve_execution_runner_source_mutation_phase_present"
            ))
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("source mutation enablement passed gates should be an array")
            .contains(&json!("contract_writeback_ready"))
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("source mutation enablement passed gates should be an array")
            .contains(&json!("contract_source_write_ready"))
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("source mutation enablement blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_lifecycle_phase_enablement_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("source mutation enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_lifecycle_phase_ready"))
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("source mutation enablement blockers should be an array")
            .contains(&json!("lifecycle_effects_ready"))
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("source mutation enablement blockers should be an array")
            .contains(&json!("contract_mutation_api_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("source mutation enablement blockers should be an array")
            .contains(&json!("contract_mutation_ready"))
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("source mutation enablement blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_source_mutation_phase_enabled"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]["status"],
        "approve_execution_runner_source_mutation_phase_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]["target_kind"],
        "memory_field"
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]
            ["phase_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]
            ["lifecycle_phase_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]
            ["source_mutation_phase_present"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]
            ["writeback_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]["source_write_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]
            ["lifecycle_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]
            ["contract_mutation_api_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]
            ["contract_mutation_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]
            ["source_mutation_phase_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]
            ["source_mutation_phase_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]
            ["would_mutate_contract"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]
            ["would_write_contract_source"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]
            ["would_continue_to_next_phase"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner source mutation blockers should be an array")
            .contains(&json!("approve_execution_runner_lifecycle_phase_ready"))
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner source mutation blockers should be an array")
            .contains(&json!("contract_mutation_api_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner source mutation blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_source_mutation_phase_enabled"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]["status"],
        "approve_execution_runner_source_mutation_phase_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["gate_name"],
        "approve_execution_runner_source_mutation_phase_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["target_kind"],
        "memory_field"
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["phase_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["lifecycle_phase_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["lifecycle_phase_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["source_mutation_phase_present"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["writeback_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["source_write_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["lifecycle_effects_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["contract_mutation_api_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["contract_mutation_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["source_mutation_phase_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["source_mutation_phase_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["source_mutation_phase_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["would_mutate_contract"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["would_write_contract_source"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["would_continue_to_next_phase"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["would_unblock_recovery_marker_cleanup"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["would_unblock_control_readiness"],
        false
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["inherited_source_mutation_phase_blockers"]
            .as_array()
            .expect("source readiness inherited phase blockers should be an array")
            .contains(&json!("approve_execution_runner_lifecycle_phase_ready"))
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["inherited_source_mutation_phase_blockers"]
            .as_array()
            .expect("source readiness inherited phase blockers should be an array")
            .contains(&json!("contract_mutation_api_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["inherited_source_mutation_phase_enablement_blocked_gates"]
            .as_array()
            .expect("source readiness inherited switch blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_source_mutation_phase_enabled"
            ))
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["inherited_source_mutation_phase_enablement_blocked_gates"]
            .as_array()
            .expect("source readiness inherited switch blockers should be an array")
            .contains(&json!("contract_mutation_ready"))
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("source readiness required gates should be an array")
            .len(),
        8
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("source readiness passed gates should be an array")
            .contains(&json!(
                "approve_execution_runner_source_mutation_phase_present"
            ))
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("source readiness passed gates should be an array")
            .contains(&json!("contract_source_write_ready"))
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("source readiness blockers should be an array")
            .contains(&json!("approve_execution_runner_lifecycle_phase_ready"))
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("source readiness blockers should be an array")
            .contains(&json!("contract_mutation_api_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("source readiness blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_source_mutation_phase_enabled"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
            ["status"],
        "approve_execution_runner_recovery_marker_cleanup_phase_enablement_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
            ["switch_name"],
        "approve_execution_runner_recovery_marker_cleanup_phase_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
            ["phase_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
            ["source_mutation_phase_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
            ["source_mutation_phase_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
            ["cleanup_phase_present"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
            ["marker_persistence_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
            ["marker_persistence_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
            ["cleanup_phase_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
            ["enablement_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
            ["cleanup_phase_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
            ["would_enable_cleanup_phase"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
            ["would_clear_recovery_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
            ["would_continue_to_commit"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("cleanup enablement gates should be an array")
            .len(),
        7
    );
    assert!(review_body
        ["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
        ["passed_gates"]
        .as_array()
        .expect("cleanup enablement passed gates should be an array")
        .contains(&json!("approve_execution_runner_phase_sequence_ready")));
    assert!(review_body
        ["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
        ["passed_gates"]
        .as_array()
        .expect("cleanup enablement passed gates should be an array")
        .contains(&json!(
            "approve_execution_runner_recovery_marker_cleanup_phase_present"
        )));
    assert!(review_body
        ["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
        ["passed_gates"]
        .as_array()
        .expect("cleanup enablement passed gates should be an array")
        .contains(&json!("recovery_marker_persistence_plan_ready")));
    assert!(review_body
        ["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
        ["blocked_gates"]
        .as_array()
        .expect("cleanup enablement blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_source_mutation_phase_enablement_ready"
        )));
    assert!(review_body
        ["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
        ["blocked_gates"]
        .as_array()
        .expect("cleanup enablement blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_source_mutation_phase_ready"
        )));
    assert!(review_body
        ["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
        ["blocked_gates"]
        .as_array()
        .expect("cleanup enablement blockers should be an array")
        .contains(&json!("recovery_marker_persistence_ready")));
    assert!(review_body
        ["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
        ["blocked_gates"]
        .as_array()
        .expect("cleanup enablement blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_recovery_marker_cleanup_phase_enabled"
        )));
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]["status"],
        "approve_execution_runner_recovery_marker_cleanup_phase_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]
            ["phase_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]
            ["source_mutation_phase_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]
            ["cleanup_phase_present"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]
            ["marker_persistence_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]
            ["marker_persistence_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]
            ["cleanup_phase_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]
            ["cleanup_phase_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]
            ["would_clear_recovery_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]
            ["would_continue_to_commit"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner cleanup blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_source_mutation_phase_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner cleanup blockers should be an array")
            .contains(&json!("recovery_marker_persistence_ready"))
    );
    assert!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner cleanup blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_recovery_marker_cleanup_phase_enabled"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["status"],
        "approve_execution_runner_recovery_marker_cleanup_phase_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["gate_name"],
        "approve_execution_runner_recovery_marker_cleanup_phase_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["phase_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["source_mutation_phase_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["source_mutation_phase_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["cleanup_phase_present"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["marker_persistence_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["marker_persistence_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["cleanup_phase_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["cleanup_phase_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["cleanup_phase_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["would_clear_recovery_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["would_continue_to_commit"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["would_unblock_transaction_commit"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["would_unblock_control_readiness"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("cleanup readiness gates should be an array")
            .len(),
        6
    );
    assert!(review_body
        ["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
        ["passed_gates"]
        .as_array()
        .expect("cleanup readiness passed gates should be an array")
        .contains(&json!("approve_execution_runner_phase_sequence_ready")));
    assert!(review_body
        ["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
        ["passed_gates"]
        .as_array()
        .expect("cleanup readiness passed gates should be an array")
        .contains(&json!(
            "approve_execution_runner_recovery_marker_cleanup_phase_present"
        )));
    assert!(review_body
        ["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
        ["passed_gates"]
        .as_array()
        .expect("cleanup readiness passed gates should be an array")
        .contains(&json!("recovery_marker_persistence_plan_ready")));
    assert!(review_body
        ["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
        ["blocked_gates"]
        .as_array()
        .expect("cleanup readiness blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_source_mutation_phase_ready"
        )));
    assert!(review_body
        ["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
        ["blocked_gates"]
        .as_array()
        .expect("cleanup readiness blockers should be an array")
        .contains(&json!("recovery_marker_persistence_ready")));
    assert!(review_body
        ["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
        ["blocked_gates"]
        .as_array()
        .expect("cleanup readiness blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_recovery_marker_cleanup_phase_enabled"
        )));
    assert!(review_body
        ["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
        ["inherited_cleanup_phase_blockers"]
        .as_array()
        .expect("cleanup readiness inherited blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_source_mutation_phase_ready"
        )));
    assert!(review_body
        ["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
        ["inherited_cleanup_phase_enablement_blocked_gates"]
        .as_array()
        .expect("cleanup readiness inherited switch blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_source_mutation_phase_enablement_ready"
        )));
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["status"],
        "approve_execution_runner_transaction_commit_phase_enablement_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["switch_name"],
        "approve_execution_runner_transaction_commit_phase_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["phase_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["cleanup_phase_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["cleanup_phase_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["runner_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["runner_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["commit_barrier_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["rollback_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["recovery_marker_persistence_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["recovery_marker_persistence_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["commit_gate_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["commit_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["commit_phase_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["enablement_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["commit_phase_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["would_enable_commit_phase"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("commit phase enablement gates should be an array")
            .len(),
        13
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("commit phase enablement passed gates should be an array")
            .contains(&json!("approve_execution_runner_phase_sequence_ready"))
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("commit phase enablement passed gates should be an array")
            .contains(&json!("approve_execution_transaction_commit_plan_ready"))
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("commit phase enablement passed gates should be an array")
            .contains(&json!("commit_barrier_ready"))
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("commit phase enablement passed gates should be an array")
            .contains(&json!("rollback_plan_ready"))
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("commit phase enablement passed gates should be an array")
            .contains(&json!("recovery_marker_persistence_plan_ready"))
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("commit phase enablement blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_recovery_marker_cleanup_phase_enablement_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("commit phase enablement blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_recovery_marker_cleanup_phase_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("commit phase enablement blockers should be an array")
            .contains(&json!("approve_execution_transaction_runner_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("commit phase enablement blockers should be an array")
            .contains(&json!("approve_execution_admission_ready"))
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("commit phase enablement blockers should be an array")
            .contains(&json!("recovery_marker_persistence_ready"))
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("commit phase enablement blockers should be an array")
            .contains(&json!("approve_execution_transaction_commit_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("commit phase enablement blockers should be an array")
            .contains(&json!("approve_execution_transaction_commit_ready"))
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("commit phase enablement blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_transaction_commit_phase_enabled"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]["status"],
        "approve_execution_runner_transaction_commit_phase_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]
            ["phase_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]
            ["cleanup_phase_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]
            ["runner_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]["runner_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]["admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]
            ["commit_barrier_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]
            ["rollback_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]
            ["recovery_marker_persistence_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]
            ["recovery_marker_persistence_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]
            ["commit_gate_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]["commit_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]
            ["commit_phase_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]
            ["commit_phase_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]
            ["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner commit blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_recovery_marker_cleanup_phase_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner commit blockers should be an array")
            .contains(&json!("approve_execution_transaction_commit_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner commit blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_transaction_commit_phase_enabled"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["status"],
        "approve_execution_runner_transaction_commit_phase_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["gate_name"],
        "approve_execution_runner_transaction_commit_phase_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["phase_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["cleanup_phase_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["cleanup_phase_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["runner_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["runner_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["commit_barrier_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["rollback_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["recovery_marker_persistence_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["recovery_marker_persistence_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["commit_gate_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["commit_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["commit_phase_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["commit_phase_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["commit_phase_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["would_unblock_rollback_execution"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["would_unblock_control_readiness"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("commit readiness gates should be an array")
            .len(),
        4
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("commit readiness passed gates should be an array")
            .contains(&json!("approve_execution_runner_phase_sequence_ready"))
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("commit readiness blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_recovery_marker_cleanup_phase_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("commit readiness blockers should be an array")
            .contains(&json!("approve_execution_transaction_commit_ready"))
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("commit readiness blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_transaction_commit_phase_enabled"
            ))
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["inherited_commit_phase_blockers"]
            .as_array()
            .expect("commit readiness inherited blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_recovery_marker_cleanup_phase_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["inherited_commit_phase_enablement_blocked_gates"]
            .as_array()
            .expect("commit readiness inherited switch blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_recovery_marker_cleanup_phase_enablement_ready"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["status"],
        "approve_execution_runner_rollback_execution_phase_enablement_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["switch_name"],
        "approve_execution_runner_rollback_execution_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["phase_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["rollback_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["commit_phase_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["commit_phase_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["rollback_phase_present"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["rollback_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["rollback_execution_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["enablement_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["rollback_execution_enablement_ready"],
        false
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["rollback_order"]
            .as_array()
            .expect("rollback enablement phase order should be an array")
            .contains(&json!("restore_contract_source"))
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["rollback_order"]
            .as_array()
            .expect("rollback enablement phase order should be an array")
            .contains(&json!("mark_recovery_marker_rolled_back"))
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["would_enable_rollback_execution"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["would_restore_contract_source"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["would_restore_approval_record"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["would_mark_recovery_marker_rolled_back"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["would_rollback_on_error"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("rollback enablement gates should be an array")
            .len(),
        7
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("rollback enablement passed gates should be an array")
            .contains(&json!("approve_execution_runner_phase_sequence_ready"))
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("rollback enablement passed gates should be an array")
            .contains(&json!("approve_execution_runner_rollback_sequence_ready"))
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("rollback enablement passed gates should be an array")
            .contains(&json!("approve_execution_runner_rollback_phase_present"))
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("rollback enablement passed gates should be an array")
            .contains(&json!("rollback_plan_ready"))
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("rollback enablement blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_transaction_commit_phase_enablement_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("rollback enablement blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_transaction_commit_phase_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("rollback enablement blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_rollback_execution_enabled"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]["status"],
        "approve_execution_runner_rollback_execution_phase_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]
            ["phase_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]
            ["rollback_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]
            ["commit_phase_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]
            ["rollback_phase_present"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]
            ["rollback_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]
            ["rollback_execution_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]
            ["rollback_execution_ready"],
        false
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]["rollback_order"]
            .as_array()
            .expect("approve runner rollback phase order should be an array")
            .contains(&json!("restore_contract_source"))
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]["rollback_order"]
            .as_array()
            .expect("approve runner rollback phase order should be an array")
            .contains(&json!("mark_recovery_marker_rolled_back"))
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]
            ["would_restore_contract_source"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]
            ["would_restore_approval_record"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]
            ["would_mark_recovery_marker_rolled_back"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]
            ["would_rollback_on_error"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner rollback blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_transaction_commit_phase_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner rollback blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_rollback_execution_enabled"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["status"],
        "approve_execution_runner_rollback_execution_phase_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["gate_name"],
        "approve_execution_runner_rollback_execution_phase_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["phase_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["rollback_sequence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["commit_phase_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["commit_phase_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["rollback_phase_present"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["rollback_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["rollback_execution_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["rollback_execution_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["rollback_execution_ready"],
        false
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["rollback_order"]
            .as_array()
            .expect("rollback readiness phase order should be an array")
            .contains(&json!("restore_contract_source"))
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["rollback_order"]
            .as_array()
            .expect("rollback readiness phase order should be an array")
            .contains(&json!("mark_recovery_marker_rolled_back"))
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["would_restore_contract_source"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["would_restore_approval_record"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["would_mark_recovery_marker_rolled_back"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["would_rollback_on_error"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["would_unblock_runner_activation"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["would_unblock_control_readiness"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("rollback readiness gates should be an array")
            .len(),
        6
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("rollback readiness passed gates should be an array")
            .contains(&json!("approve_execution_runner_phase_sequence_ready"))
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("rollback readiness passed gates should be an array")
            .contains(&json!("approve_execution_runner_rollback_sequence_ready"))
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("rollback readiness passed gates should be an array")
            .contains(&json!("approve_execution_runner_rollback_phase_present"))
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("rollback readiness passed gates should be an array")
            .contains(&json!("rollback_plan_ready"))
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("rollback readiness blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_transaction_commit_phase_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("rollback readiness blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_rollback_execution_enabled"
            ))
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["inherited_rollback_execution_blockers"]
            .as_array()
            .expect("rollback readiness inherited blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_transaction_commit_phase_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["inherited_rollback_execution_enablement_blocked_gates"]
            .as_array()
            .expect("rollback readiness inherited switch blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_transaction_commit_phase_enablement_ready"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]["status"],
        "approve_execution_runner_activation_enablement_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]["switch_name"],
        "approve_execution_runner_activation_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]
            ["structural_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]
            ["runner_control_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]["phase_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]
            ["rollback_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]
            ["side_effect_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]
            ["rollback_execution_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]
            ["prior_enablements_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]
            ["runner_activation_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]
            ["enablement_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]
            ["runner_activation_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]
            ["would_enable_runner_activation"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]
            ["required_enablements"]
            .as_array()
            .expect("activation enablement switches should be an array")
            .len(),
        15
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]["required_gates"]
            .as_array()
            .expect("activation enablement gates should be an array")
            .len(),
        8
    );
    assert!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]["passed_gates"]
            .as_array()
            .expect("activation enablement passed gates should be an array")
            .contains(&json!("approve_execution_runner_structural_plan_ready"))
    );
    assert!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("activation enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_control_ready"))
    );
    assert!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("activation enablement blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_rollback_execution_phase_enablement_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("activation enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_prior_enablements_ready"))
    );
    assert!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("activation enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]
            ["blocked_enablements"]
            .as_array()
            .expect("activation enablement blocked switches should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_readiness_dry_run"]["status"],
        "approve_execution_runner_activation_enablement_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_readiness_dry_run"]
            ["gate_name"],
        "approve_execution_runner_activation_enablement_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_readiness_dry_run"]
            ["switch_name"],
        "approve_execution_runner_activation_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_readiness_dry_run"]
            ["structural_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_readiness_dry_run"]
            ["runner_control_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_readiness_dry_run"]
            ["rollback_execution_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_readiness_dry_run"]
            ["prior_enablements_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_readiness_dry_run"]
            ["runner_activation_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_readiness_dry_run"]
            ["enablement_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_readiness_dry_run"]
            ["runner_activation_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_readiness_dry_run"]
            ["would_enable_runner_activation"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_readiness_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_readiness_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_readiness_dry_run"]
            ["would_unblock_activation_path"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_readiness_dry_run"]
            ["required_enablements"]
            .as_array()
            .expect("activation enablement readiness switches should be an array")
            .len(),
        15
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_readiness_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("activation enablement readiness gates should be an array")
            .len(),
        8
    );
    assert!(
        review_body["approve_execution_runner_activation_enablement_readiness_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("activation enablement readiness passed gates should be an array")
            .contains(&json!("approve_execution_runner_structural_plan_ready"))
    );
    assert!(
        review_body["approve_execution_runner_activation_enablement_readiness_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("activation enablement readiness blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_activation_enablement_readiness_dry_run"]
            ["inherited_activation_enablement_blocked_gates"]
            .as_array()
            .expect("activation enablement readiness inherited blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_dry_run"]["status"],
        "approve_execution_runner_activation_path_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_dry_run"]["path_name"],
        "approve_execution_runner_atomic_activation_path"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_dry_run"]["structural_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_dry_run"]["runner_control_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_dry_run"]["phase_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_dry_run"]["rollback_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_dry_run"]
            ["side_effect_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_dry_run"]
            ["activation_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_dry_run"]["prior_enablements_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_dry_run"]
            ["atomic_activation_required"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_dry_run"]["activation_path_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_dry_run"]["would_enable_any_switch"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_dry_run"]["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_dry_run"]["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_dry_run"]["activation_steps"]
            .as_array()
            .expect("activation path steps should be an array")
            .len(),
        15
    );
    assert!(
        review_body["approve_execution_runner_activation_path_dry_run"]["activation_steps"]
            .as_array()
            .expect("activation path steps should be an array")
            .contains(&json!("approve_execution_transaction_runner_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_activation_path_dry_run"]["activation_steps"]
            .as_array()
            .expect("activation path steps should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_dry_run"]["required_enablements"]
            .as_array()
            .expect("activation path required switches should be an array")
            .len(),
        15
    );
    assert!(
        review_body["approve_execution_runner_activation_path_dry_run"]["blocked_enablements"]
            .as_array()
            .expect("activation path blocked switches should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_dry_run"]["required_gates"]
            .as_array()
            .expect("activation path gates should be an array")
            .len(),
        8
    );
    assert!(
        review_body["approve_execution_runner_activation_path_dry_run"]["passed_gates"]
            .as_array()
            .expect("activation path passed gates should be an array")
            .contains(&json!("approve_execution_runner_structural_plan_ready"))
    );
    assert!(
        review_body["approve_execution_runner_activation_path_dry_run"]["passed_gates"]
            .as_array()
            .expect("activation path passed gates should be an array")
            .contains(&json!(
                "approve_execution_runner_atomic_activation_required"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_path_dry_run"]["blocked_gates"]
            .as_array()
            .expect("activation path blockers should be an array")
            .contains(&json!("approve_execution_runner_control_ready"))
    );
    assert!(
        review_body["approve_execution_runner_activation_path_dry_run"]["blocked_gates"]
            .as_array()
            .expect("activation path blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_enablement_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_path_dry_run"]["blocked_gates"]
            .as_array()
            .expect("activation path blockers should be an array")
            .contains(&json!("approve_execution_runner_prior_enablements_ready"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_readiness_dry_run"]["status"],
        "approve_execution_runner_activation_path_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_readiness_dry_run"]["gate_name"],
        "approve_execution_runner_activation_path_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_readiness_dry_run"]["path_name"],
        "approve_execution_runner_atomic_activation_path"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_readiness_dry_run"]
            ["structural_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_readiness_dry_run"]
            ["activation_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_readiness_dry_run"]
            ["activation_enablement_readiness_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_readiness_dry_run"]
            ["prior_enablements_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_readiness_dry_run"]
            ["atomic_activation_required"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_readiness_dry_run"]
            ["activation_path_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_readiness_dry_run"]
            ["activation_step_count"],
        15
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_readiness_dry_run"]
            ["would_enable_any_switch"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_readiness_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_readiness_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_readiness_dry_run"]
            ["would_unblock_activation_execution_plan"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_readiness_dry_run"]["required_gates"]
            .as_array()
            .expect("activation path readiness gates should be an array")
            .len(),
        8
    );
    assert!(
        review_body["approve_execution_runner_activation_path_readiness_dry_run"]["passed_gates"]
            .as_array()
            .expect("activation path readiness passed gates should be an array")
            .contains(&json!(
                "approve_execution_runner_atomic_activation_required"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_path_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("activation path readiness blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_enablement_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_path_readiness_dry_run"]
            ["inherited_path_blocked_gates"]
            .as_array()
            .expect("activation path readiness inherited path blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_enablement_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_path_readiness_dry_run"]
            ["inherited_activation_enablement_readiness_blocked_gates"]
            .as_array()
            .expect("activation path readiness inherited enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]["status"],
        "approve_execution_runner_activation_execution_plan_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]["plan_name"],
        "approve_execution_runner_guarded_atomic_activation_execution_plan"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]["path_name"],
        "approve_execution_runner_atomic_activation_path"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]
            ["activation_path_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]
            ["atomic_activation_required"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]
            ["write_set_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]
            ["all_or_nothing_guard_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]
            ["rollback_boundary_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]
            ["no_partial_activation_allowed"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]
            ["activation_execution_plan_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]
            ["activation_step_count"],
        15
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]
            ["atomic_write_set"]
            .as_array()
            .expect("activation execution atomic write set should be an array")
            .len(),
        15
    );
    assert!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]
            ["atomic_write_set"]
            .as_array()
            .expect("activation execution atomic write set should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]
            ["would_enable_any_switch"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]
            ["would_persist_activation_switches"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]
            ["required_enablements"]
            .as_array()
            .expect("activation execution required switches should be an array")
            .len(),
        15
    );
    assert!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]
            ["blocked_enablements"]
            .as_array()
            .expect("activation execution blocked switches should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]
            ["inherited_path_blocked_gates"]
            .as_array()
            .expect("activation execution inherited path blockers should be an array")
            .contains(&json!("approve_execution_runner_control_ready"))
    );
    assert!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]
            ["inherited_path_blocked_gates"]
            .as_array()
            .expect("activation execution inherited path blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_enablement_ready"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]["required_gates"]
            .as_array()
            .expect("activation execution gates should be an array")
            .len(),
        5
    );
    assert!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]["passed_gates"]
            .as_array()
            .expect("activation execution passed gates should be an array")
            .contains(&json!("approve_execution_runner_atomic_write_set_ready"))
    );
    assert!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]["passed_gates"]
            .as_array()
            .expect("activation execution passed gates should be an array")
            .contains(&json!(
                "approve_execution_runner_all_or_nothing_guard_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]["passed_gates"]
            .as_array()
            .expect("activation execution passed gates should be an array")
            .contains(&json!("approve_execution_runner_rollback_boundary_ready"))
    );
    assert!(
        review_body["approve_execution_runner_activation_execution_plan_dry_run"]["blocked_gates"]
            .as_array()
            .expect("activation execution blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_path_ready"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["status"],
        "approve_execution_runner_activation_execution_plan_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["gate_name"],
        "approve_execution_runner_activation_execution_plan_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["plan_name"],
        "approve_execution_runner_guarded_atomic_activation_execution_plan"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["path_name"],
        "approve_execution_runner_atomic_activation_path"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["activation_path_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["activation_path_readiness_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["write_set_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["all_or_nothing_guard_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["rollback_boundary_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["no_partial_activation_allowed"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["activation_execution_plan_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["activation_step_count"],
        15
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["would_persist_activation_switches"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["would_unblock_switch_transaction_proof"],
        false
    );
    assert!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("activation execution readiness passed gates should be an array")
            .contains(&json!("approve_execution_runner_atomic_write_set_ready"))
    );
    assert!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("activation execution readiness blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_path_ready"))
    );
    assert!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["inherited_execution_plan_blocked_gates"]
            .as_array()
            .expect("activation execution readiness inherited plan blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_path_ready"))
    );
    assert!(
        review_body["approve_execution_runner_activation_execution_plan_readiness_dry_run"]
            ["inherited_path_readiness_blocked_gates"]
            .as_array()
            .expect("activation execution readiness inherited path blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_enablement_ready"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["status"],
        "approve_execution_runner_activation_switch_transaction_proof_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["transaction_name"],
        "approve_execution_runner_activation_switch_write_transaction"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["plan_name"],
        "approve_execution_runner_guarded_atomic_activation_execution_plan"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["activation_execution_plan_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["write_set_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["all_or_nothing_guard_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["failure_probe_coverage_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["rollback_action_coverage_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["partial_state_proof_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["transaction_failure_proof_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["atomic_write_set_count"],
        15
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["simulated_failure_point_count"],
        15
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["rollback_action_count"],
        15
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["partial_enabled_after_failure_count"],
        0
    );
    assert!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["atomic_write_set"]
            .as_array()
            .expect("activation switch proof write set should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["simulated_failure_points"]
            .as_array()
            .expect("activation switch proof failure points should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["rollback_actions"]
            .as_array()
            .expect("activation switch proof rollback actions should be an array")
            .contains(&json!(
                "disable:approve_execution_runner_activation_enabled"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["would_write_switches"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["would_persist_partial_state"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["inherited_execution_plan_blocked_gates"]
            .as_array()
            .expect("activation switch proof inherited blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_path_ready"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("activation switch proof gates should be an array")
            .len(),
        6
    );
    assert!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("activation switch proof passed gates should be an array")
            .contains(&json!(
                "approve_execution_runner_failure_probe_coverage_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("activation switch proof passed gates should be an array")
            .contains(&json!(
                "approve_execution_runner_rollback_action_coverage_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("activation switch proof passed gates should be an array")
            .contains(&json!("approve_execution_runner_partial_state_proof_ready"))
    );
    assert!(
        review_body["approve_execution_runner_activation_switch_transaction_proof_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("activation switch proof blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_execution_plan_ready"
            ))
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["status"],
        "approve_execution_runner_activation_switch_transaction_proof_readiness_ready_blocked"
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["gate_name"],
        "approve_execution_runner_activation_switch_transaction_proof_ready"
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["transaction_name"],
        "approve_execution_runner_activation_switch_write_transaction"
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["activation_execution_plan_ready"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["activation_execution_plan_readiness_ready"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["write_set_ready"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["all_or_nothing_guard_ready"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["failure_probe_coverage_ready"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["rollback_action_coverage_ready"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["partial_state_proof_ready"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["transaction_failure_proof_ready"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["atomic_write_set_count"],
        15
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["simulated_failure_point_count"],
        15
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["rollback_action_count"],
        15
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["partial_enabled_after_failure_count"],
        0
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["would_write_switches"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["would_persist_partial_state"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["would_unblock_switch_write_transaction_enablement"],
        false
    );
    assert!(review_body
        ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
        ["passed_gates"]
        .as_array()
        .expect("activation switch proof readiness passed gates should be an array")
        .contains(&json!("approve_execution_runner_partial_state_proof_ready")));
    assert!(review_body
        ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
        ["blocked_gates"]
        .as_array()
        .expect("activation switch proof readiness blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_activation_execution_plan_ready"
        )));
    assert!(review_body
        ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
        ["inherited_transaction_proof_blocked_gates"]
        .as_array()
        .expect("activation switch proof readiness inherited proof blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_activation_execution_plan_ready"
        )));
    assert!(
        review_body
            ["approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run"]
            ["inherited_execution_plan_readiness_blocked_gates"]
            .as_array()
            .expect(
                "activation switch proof readiness inherited execution blockers should be an array",
            )
            .contains(&json!("approve_execution_runner_activation_path_ready"))
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["status"],
        "approve_execution_runner_activation_switch_write_transaction_enablement_ready_blocked"
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["switch_name"],
        "approve_execution_runner_activation_switch_write_transaction_enabled"
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["transaction_failure_proof_ready"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["write_set_ready"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["all_or_nothing_guard_ready"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["failure_probe_coverage_ready"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["rollback_action_coverage_ready"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["partial_state_proof_ready"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["partial_enabled_after_failure_count"],
        0
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["transaction_enabled"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["enablement_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["switch_write_transaction_enablement_ready"],
        false
    );
    assert!(review_body
        ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
        ["atomic_write_set"]
        .as_array()
        .expect("activation switch transaction enablement write set should be an array")
        .contains(&json!("approve_execution_runner_activation_enabled")));
    assert!(review_body
        ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
        ["rollback_actions"]
        .as_array()
        .expect("activation switch transaction enablement rollback actions should be an array")
        .contains(&json!(
            "disable:approve_execution_runner_activation_enabled"
        )));
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["would_enable_transaction"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["would_write_switches"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["would_persist_partial_state"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(review_body
        ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
        ["inherited_transaction_proof_blocked_gates"]
        .as_array()
        .expect("activation switch transaction enablement inherited blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_activation_execution_plan_ready"
        )));
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("activation switch transaction enablement gates should be an array")
            .len(),
        7
    );
    assert!(review_body
        ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
        ["passed_gates"]
        .as_array()
        .expect("activation switch transaction enablement passed gates should be an array")
        .contains(&json!("approve_execution_runner_atomic_write_set_ready")));
    assert!(review_body
        ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
        ["blocked_gates"]
        .as_array()
        .expect("activation switch transaction enablement blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_activation_switch_transaction_proof_ready"
        )));
    assert!(review_body
        ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
        ["blocked_gates"]
        .as_array()
        .expect("activation switch transaction enablement blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_activation_switch_write_transaction_enabled"
        )));
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["status"],
        "approve_execution_runner_activation_switch_write_transaction_enablement_readiness_ready_blocked"
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["gate_name"],
        "approve_execution_runner_activation_switch_write_transaction_enablement_ready"
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["switch_name"],
        "approve_execution_runner_activation_switch_write_transaction_enabled"
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["transaction_failure_proof_ready"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["transaction_failure_proof_readiness_ready"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["write_set_ready"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["all_or_nothing_guard_ready"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["failure_probe_coverage_ready"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["rollback_action_coverage_ready"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["partial_state_proof_ready"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["partial_enabled_after_failure_count"],
        0
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["transaction_enabled"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["enablement_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["switch_write_transaction_enablement_ready"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["atomic_write_set"]
            .as_array()
            .expect("activation switch transaction enablement readiness write set should be an array")
            .len(),
        15
    );
    assert!(review_body
        ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
        ["rollback_actions"]
        .as_array()
        .expect("activation switch transaction enablement readiness rollback actions should be an array")
        .contains(&json!(
            "disable:approve_execution_runner_activation_enabled"
        )));
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["would_enable_transaction"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["would_write_switches"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["would_persist_partial_state"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["would_unblock_switch_write_transaction"],
        false
    );
    assert!(review_body
        ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
        ["inherited_enablement_blocked_gates"]
        .as_array()
        .expect("activation switch transaction enablement readiness inherited blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_activation_switch_write_transaction_enabled"
        )));
    assert!(review_body
        ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
        ["inherited_transaction_proof_readiness_blocked_gates"]
        .as_array()
        .expect("activation switch transaction enablement readiness proof blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_activation_execution_plan_ready"
        )));
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("activation switch transaction enablement readiness gates should be an array")
            .len(),
        7
    );
    assert!(review_body
        ["approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run"]
        ["blocked_gates"]
        .as_array()
        .expect("activation switch transaction enablement readiness blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_activation_switch_transaction_proof_ready"
        )));
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["status"],
        "approve_execution_runner_activation_switch_write_transaction_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["transaction_name"],
        "approve_execution_runner_activation_switch_write_transaction"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["transaction_failure_proof_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["write_set_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["all_or_nothing_guard_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["failure_probe_coverage_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["rollback_action_coverage_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["partial_state_proof_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["transaction_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["transaction_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["switch_write_transaction_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["partial_enabled_after_failure_count"],
        0
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["activation_switch_write_order"]
            .as_array()
            .expect("activation switch write order should be an array")
            .len(),
        15
    );
    assert!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["activation_switch_write_order"]
            .as_array()
            .expect("activation switch write order should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["rollback_actions"]
            .as_array()
            .expect("activation switch write rollback actions should be an array")
            .contains(&json!(
                "disable:approve_execution_runner_activation_enabled"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["would_write_switches"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["would_persist_partial_state"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["inherited_transaction_proof_blocked_gates"]
            .as_array()
            .expect("activation switch write inherited blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_execution_plan_ready"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("activation switch write gates should be an array")
            .len(),
        7
    );
    assert!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("activation switch write passed gates should be an array")
            .contains(&json!("approve_execution_runner_atomic_write_set_ready"))
    );
    assert!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("activation switch write blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_switch_transaction_proof_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("activation switch write blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_switch_write_transaction_enabled"
            ))
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["status"],
        "approve_execution_runner_activation_switch_write_transaction_readiness_ready_blocked"
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["gate_name"],
        "approve_execution_runner_activation_switch_write_transaction_ready"
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["transaction_name"],
        "approve_execution_runner_activation_switch_write_transaction"
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["transaction_failure_proof_ready"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["switch_write_transaction_enablement_readiness_ready"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["write_set_ready"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["all_or_nothing_guard_ready"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["failure_probe_coverage_ready"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["rollback_action_coverage_ready"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["partial_state_proof_ready"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["transaction_enabled"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["transaction_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["switch_write_transaction_ready"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["partial_enabled_after_failure_count"],
        0
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["activation_switch_write_order"]
            .as_array()
            .expect("activation switch write readiness order should be an array")
            .len(),
        15
    );
    assert!(review_body
        ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
        ["activation_switch_write_order"]
        .as_array()
        .expect("activation switch write readiness order should be an array")
        .contains(&json!("approve_execution_runner_activation_enabled")));
    assert!(review_body
        ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
        ["rollback_actions"]
        .as_array()
        .expect("activation switch write readiness rollback actions should be an array")
        .contains(&json!(
            "disable:approve_execution_runner_activation_enabled"
        )));
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["would_write_switches"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["would_persist_partial_state"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["would_unblock_activation_transaction_admission"],
        false
    );
    assert!(review_body
        ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
        ["inherited_switch_write_transaction_blocked_gates"]
        .as_array()
        .expect("activation switch write readiness inherited blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_activation_switch_write_transaction_enabled"
        )));
    assert!(review_body
        ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
        ["inherited_enablement_readiness_blocked_gates"]
        .as_array()
        .expect("activation switch write readiness enablement blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_activation_switch_transaction_proof_ready"
        )));
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("activation switch write readiness gates should be an array")
            .len(),
        7
    );
    assert!(review_body
        ["approve_execution_runner_activation_switch_write_transaction_readiness_dry_run"]
        ["blocked_gates"]
        .as_array()
        .expect("activation switch write readiness blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_activation_switch_transaction_proof_ready"
        )));
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["status"],
        "approve_execution_runner_activation_transaction_admission_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["gate_name"],
        "approve_execution_runner_activation_transaction_admission_gate"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["transaction_name"],
        "approve_execution_runner_activation_switch_write_transaction"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["activation_path_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["activation_execution_plan_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["transaction_failure_proof_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["switch_write_transaction_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["switch_write_transaction_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["transaction_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["transaction_shape_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["activation_transaction_admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["partial_enabled_after_failure_count"],
        0
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["activation_switch_write_order"]
            .as_array()
            .expect("activation admission write order should be an array")
            .len(),
        15
    );
    assert!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["activation_switch_write_order"]
            .as_array()
            .expect("activation admission write order should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["rollback_actions"]
            .as_array()
            .expect("activation admission rollback actions should be an array")
            .contains(&json!(
                "disable:approve_execution_runner_activation_enabled"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["would_admit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["would_write_switches"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["would_persist_partial_state"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["inherited_path_blocked_gates"]
            .as_array()
            .expect("activation admission inherited path blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_enablement_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["inherited_execution_plan_blocked_gates"]
            .as_array()
            .expect("activation admission inherited execution blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_path_ready"))
    );
    assert!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["inherited_transaction_proof_blocked_gates"]
            .as_array()
            .expect("activation admission inherited proof blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_execution_plan_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["inherited_transaction_enablement_blocked_gates"]
            .as_array()
            .expect("activation admission inherited enablement blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_switch_write_transaction_enabled"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["inherited_switch_write_transaction_blocked_gates"]
            .as_array()
            .expect("activation admission inherited transaction blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_switch_write_transaction_enabled"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("activation admission gates should be an array")
            .len(),
        5
    );
    assert!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("activation admission blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_path_ready"))
    );
    assert!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("activation admission blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_switch_write_transaction_ready"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
            ["status"],
        "approve_execution_runner_activation_transaction_admission_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
            ["gate_name"],
        "approve_execution_runner_activation_transaction_admission_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
            ["source_gate_name"],
        "approve_execution_runner_activation_transaction_admission_gate"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
            ["activation_path_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
            ["activation_execution_plan_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
            ["transaction_failure_proof_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
            ["switch_write_transaction_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
            ["switch_write_transaction_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
            ["switch_write_transaction_readiness_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
            ["transaction_shape_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
            ["activation_transaction_admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
            ["activation_switch_write_order"]
            .as_array()
            .expect("activation admission readiness write order should be an array")
            .len(),
        15
    );
    assert!(review_body
        ["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
        ["rollback_actions"]
        .as_array()
        .expect("activation admission readiness rollback actions should be an array")
        .contains(&json!(
            "disable:approve_execution_runner_activation_enabled"
        )));
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
            ["would_admit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
            ["would_write_switches"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
            ["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
            ["would_unblock_activation_admission_handoff"],
        false
    );
    assert!(review_body
        ["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
        ["inherited_admission_blocked_gates"]
        .as_array()
        .expect("activation admission readiness inherited blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_activation_switch_write_transaction_ready"
        )));
    assert!(review_body
        ["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
        ["inherited_switch_write_transaction_readiness_blocked_gates"]
        .as_array()
        .expect("activation admission readiness switch blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_activation_switch_write_transaction_enabled"
        )));
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("activation admission readiness gates should be an array")
            .len(),
        5
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]["status"],
        "approve_execution_runner_activation_admission_handoff_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["handoff_name"],
        "approve_execution_runner_activation_admission_handoff"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["source_gate_name"],
        "approve_execution_runner_activation_transaction_admission_gate"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["structural_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["runner_control_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["phase_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["rollback_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["side_effect_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["activation_admission_required"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["activation_transaction_admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["runner_activation_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["activation_handoff_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["runner_activation_handoff_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["activation_switch_write_order"]
            .as_array()
            .expect("activation handoff write order should be an array")
            .len(),
        15
    );
    assert!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["activation_switch_write_order"]
            .as_array()
            .expect("activation handoff write order should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["would_handoff_to_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["inherited_activation_admission_blocked_gates"]
            .as_array()
            .expect("activation handoff inherited admission blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_transaction_admission_ready"
            ))
            == false
    );
    assert!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["inherited_activation_admission_blocked_gates"]
            .as_array()
            .expect("activation handoff inherited admission blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_switch_write_transaction_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["inherited_runner_enablement_blocked_by"]
            .as_array()
            .expect("activation handoff inherited runner blockers should be an array")
            .contains(&json!("approve_execution_runner_control_ready"))
    );
    assert!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["inherited_runner_enablement_blocked_by"]
            .as_array()
            .expect("activation handoff inherited runner blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("activation handoff gates should be an array")
            .len(),
        7
    );
    assert!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("activation handoff passed gates should be an array")
            .contains(&json!("approve_execution_runner_structural_plan_ready"))
    );
    assert!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("activation handoff blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_transaction_admission_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("activation handoff blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["status"],
        "approve_execution_runner_activation_admission_handoff_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["gate_name"],
        "approve_execution_runner_activation_admission_handoff_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["handoff_name"],
        "approve_execution_runner_activation_admission_handoff"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["source_gate_name"],
        "approve_execution_runner_activation_transaction_admission_gate"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["structural_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["runner_control_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["activation_transaction_admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["activation_transaction_admission_readiness_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["runner_activation_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["activation_handoff_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["runner_activation_handoff_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["activation_switch_write_order"]
            .as_array()
            .expect("activation handoff readiness write order should be an array")
            .len(),
        15
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["would_handoff_to_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["would_unblock_handoff_enablement"],
        false
    );
    assert!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["inherited_handoff_blocked_gates"]
            .as_array()
            .expect("activation handoff readiness inherited blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_transaction_admission_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["inherited_admission_readiness_blocked_gates"]
            .as_array()
            .expect("activation handoff readiness admission blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_switch_write_transaction_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["inherited_runner_enablement_blocked_by"]
            .as_array()
            .expect("activation handoff readiness runner blockers should be an array")
            .contains(&json!("approve_execution_runner_control_ready"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_readiness_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("activation handoff readiness gates should be an array")
            .len(),
        7
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]["status"],
        "approve_execution_runner_activation_handoff_enablement_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["switch_name"],
        "approve_execution_runner_activation_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["handoff_name"],
        "approve_execution_runner_activation_admission_handoff"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["structural_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["runner_control_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["phase_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["rollback_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["side_effect_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["activation_admission_required"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["activation_transaction_admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["activation_handoff_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["runner_activation_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["activation_handoff_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["activation_switch_write_order"]
            .as_array()
            .expect("activation handoff enablement write order should be an array")
            .len(),
        15
    );
    assert!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["activation_switch_write_order"]
            .as_array()
            .expect("activation handoff enablement write order should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["would_enable_runner_activation"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["would_handoff_to_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["inherited_handoff_blocked_gates"]
            .as_array()
            .expect("activation handoff enablement inherited blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_transaction_admission_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["inherited_handoff_blocked_gates"]
            .as_array()
            .expect("activation handoff enablement inherited blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("activation handoff enablement gates should be an array")
            .len(),
        7
    );
    assert!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["passed_gates"]
            .as_array()
            .expect("activation handoff enablement passed gates should be an array")
            .contains(&json!("approve_execution_runner_structural_plan_ready"))
    );
    assert!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("activation handoff enablement blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_transaction_admission_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("activation handoff enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["status"],
        "approve_execution_runner_activation_handoff_enablement_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["gate_name"],
        "approve_execution_runner_activation_handoff_enablement_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["switch_name"],
        "approve_execution_runner_activation_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["handoff_name"],
        "approve_execution_runner_activation_admission_handoff"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["structural_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["runner_control_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["phase_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["rollback_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["side_effect_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["activation_admission_required"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["activation_transaction_admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["activation_admission_handoff_readiness_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["activation_handoff_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["runner_activation_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["activation_handoff_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["activation_switch_write_order"]
            .as_array()
            .expect("activation handoff enablement readiness write order should be an array")
            .len(),
        15
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["would_unblock_handoff_attempt"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["would_enable_runner_activation"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["would_handoff_to_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(review_body
        ["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
        ["inherited_enablement_blocked_gates"]
        .as_array()
        .expect("activation handoff enablement readiness blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_activation_transaction_admission_ready"
        )));
    assert!(review_body
        ["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
        ["inherited_handoff_readiness_blocked_gates"]
        .as_array()
        .expect("activation handoff enablement readiness handoff blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_activation_transaction_admission_ready"
        )));
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("activation handoff enablement readiness gates should be an array")
            .len(),
        7
    );
    assert!(review_body
        ["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
        ["passed_gates"]
        .as_array()
        .expect("activation handoff enablement readiness passed gates should be an array")
        .contains(&json!("approve_execution_runner_structural_plan_ready")));
    assert!(review_body
        ["approve_execution_runner_activation_handoff_enablement_readiness_dry_run"]
        ["blocked_gates"]
        .as_array()
        .expect("activation handoff enablement readiness blockers should be an array")
        .contains(&json!("approve_execution_runner_activation_enabled")));
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]["status"],
        "approve_execution_runner_activation_handoff_attempt_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]["attempt_name"],
        "approve_execution_runner_activation_handoff_attempt"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["source_handoff_name"],
        "approve_execution_runner_activation_admission_handoff"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["source_switch_name"],
        "approve_execution_runner_activation_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["structural_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["runner_control_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["phase_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["rollback_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["side_effect_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["activation_admission_required"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["activation_transaction_admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["activation_handoff_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["runner_activation_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["activation_handoff_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["handoff_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["activation_switch_write_order"]
            .as_array()
            .expect("activation handoff attempt write order should be an array")
            .len(),
        15
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["would_start_handoff"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["would_handoff_to_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["inherited_handoff_enablement_blocked_gates"]
            .as_array()
            .expect("activation handoff attempt inherited blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_transaction_admission_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["inherited_handoff_enablement_blocked_gates"]
            .as_array()
            .expect("activation handoff attempt inherited blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("activation handoff attempt gates should be an array")
            .len(),
        1
    );
    assert!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]["blocked_gates"]
            .as_array()
            .expect("activation handoff attempt blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_handoff_enablement_ready"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["status"],
        "approve_execution_runner_activation_handoff_attempt_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["gate_name"],
        "approve_execution_runner_activation_handoff_attempt_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["attempt_name"],
        "approve_execution_runner_activation_handoff_attempt"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["source_handoff_name"],
        "approve_execution_runner_activation_admission_handoff"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["source_switch_name"],
        "approve_execution_runner_activation_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["structural_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["runner_control_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["phase_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["rollback_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["side_effect_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["activation_admission_required"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["activation_transaction_admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["activation_handoff_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["runner_activation_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["activation_handoff_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["activation_handoff_enablement_readiness_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["handoff_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["activation_switch_write_order"]
            .as_array()
            .expect("activation handoff attempt readiness write order should be an array")
            .len(),
        15
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["would_unblock_post_handoff_attempt"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["would_start_handoff"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["would_handoff_to_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["inherited_attempt_blocked_gates"]
            .as_array()
            .expect("activation handoff attempt readiness blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_handoff_enablement_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["inherited_enablement_readiness_blocked_gates"]
            .as_array()
            .expect("activation handoff attempt readiness enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["inherited_handoff_enablement_blocked_gates"]
            .as_array()
            .expect("activation handoff attempt readiness inherited enablement blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_transaction_admission_ready"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("activation handoff attempt readiness gates should be an array")
            .len(),
        1
    );
    assert!(
        review_body["approve_execution_runner_activation_handoff_attempt_readiness_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("activation handoff attempt readiness blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_handoff_enablement_ready"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]["status"],
        "approve_execution_runner_activation_post_handoff_attempt_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["attempt_name"],
        "approve_execution_runner_activation_post_handoff_attempt"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["source_attempt_name"],
        "approve_execution_runner_activation_handoff_attempt"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["source_handoff_name"],
        "approve_execution_runner_activation_admission_handoff"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["source_switch_name"],
        "approve_execution_runner_activation_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["structural_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["runner_control_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["phase_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["rollback_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["side_effect_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["activation_admission_required"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["activation_transaction_admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["activation_handoff_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["runner_activation_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["activation_handoff_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["handoff_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["activation_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["activation_switch_write_order"]
            .as_array()
            .expect("activation post-handoff attempt write order should be an array")
            .len(),
        15
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["would_attempt_activation"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["inherited_handoff_attempt_blocked_gates"]
            .as_array()
            .expect("activation post-handoff attempt inherited attempt blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_handoff_enablement_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["inherited_handoff_enablement_blocked_gates"]
            .as_array()
            .expect(
                "activation post-handoff attempt inherited enablement blockers should be an array",
            )
            .contains(&json!(
                "approve_execution_runner_activation_transaction_admission_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["inherited_handoff_enablement_blocked_gates"]
            .as_array()
            .expect(
                "activation post-handoff attempt inherited enablement blockers should be an array",
            )
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("activation post-handoff attempt gates should be an array")
            .len(),
        1
    );
    assert!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("activation post-handoff attempt blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_handoff_attempt_ready"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["status"],
        "approve_execution_runner_activation_post_handoff_attempt_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["gate_name"],
        "approve_execution_runner_activation_post_handoff_attempt_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["attempt_name"],
        "approve_execution_runner_activation_post_handoff_attempt"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["source_attempt_name"],
        "approve_execution_runner_activation_handoff_attempt"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["source_handoff_name"],
        "approve_execution_runner_activation_admission_handoff"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["source_switch_name"],
        "approve_execution_runner_activation_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["structural_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["runner_control_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["phase_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["rollback_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["side_effect_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["activation_admission_required"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["activation_transaction_admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["activation_handoff_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["runner_activation_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["activation_handoff_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["handoff_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["handoff_attempt_readiness_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["activation_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["activation_post_handoff_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["activation_switch_write_order"]
            .as_array()
            .expect("activation post-handoff readiness write order should be an array")
            .len(),
        15
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["would_unblock_success_admission"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["would_attempt_activation"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(review_body
        ["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
        ["inherited_post_handoff_attempt_blocked_gates"]
        .as_array()
        .expect("activation post-handoff readiness blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_activation_handoff_attempt_ready"
        )));
    assert!(review_body
        ["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
        ["inherited_handoff_attempt_readiness_blocked_gates"]
        .as_array()
        .expect("activation post-handoff readiness handoff readiness blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_activation_handoff_enablement_ready"
        )));
    assert!(review_body
        ["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
        ["inherited_handoff_enablement_blocked_gates"]
        .as_array()
        .expect(
            "activation post-handoff readiness inherited enablement blockers should be an array"
        )
        .contains(&json!("approve_execution_runner_activation_enabled")));
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("activation post-handoff readiness gates should be an array")
            .len(),
        1
    );
    assert!(review_body
        ["approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run"]
        ["blocked_gates"]
        .as_array()
        .expect("activation post-handoff readiness blockers should be an array")
        .contains(&json!(
            "approve_execution_runner_activation_handoff_attempt_ready"
        )));
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]["status"],
        "approve_execution_runner_activation_success_admission_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["admission_name"],
        "approve_execution_runner_activation_success_admission"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["source_attempt_name"],
        "approve_execution_runner_activation_post_handoff_attempt"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["source_handoff_attempt_name"],
        "approve_execution_runner_activation_handoff_attempt"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["source_handoff_name"],
        "approve_execution_runner_activation_admission_handoff"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["source_switch_name"],
        "approve_execution_runner_activation_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["structural_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["runner_control_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["phase_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["rollback_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["side_effect_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["activation_admission_required"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["activation_transaction_admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["activation_handoff_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["runner_activation_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["activation_handoff_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["handoff_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["activation_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["activation_post_handoff_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["activation_success_admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["activation_switch_write_order"]
            .as_array()
            .expect("activation success admission write order should be an array")
            .len(),
        15
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["would_admit_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["inherited_post_handoff_attempt_blocked_gates"]
            .as_array()
            .expect(
                "activation success admission inherited post-handoff blockers should be an array"
            )
            .contains(&json!(
                "approve_execution_runner_activation_handoff_attempt_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["inherited_handoff_attempt_blocked_gates"]
            .as_array()
            .expect("activation success admission inherited handoff attempt blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_handoff_enablement_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["inherited_handoff_enablement_blocked_gates"]
            .as_array()
            .expect("activation success admission inherited enablement blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_transaction_admission_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["inherited_handoff_enablement_blocked_gates"]
            .as_array()
            .expect("activation success admission inherited enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("activation success admission gates should be an array")
            .len(),
        1
    );
    assert!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("activation success admission blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_post_handoff_attempt_ready"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["status"],
        "approve_execution_runner_activation_success_admission_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["gate_name"],
        "approve_execution_runner_activation_success_admission_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["admission_name"],
        "approve_execution_runner_activation_success_admission"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["source_attempt_name"],
        "approve_execution_runner_activation_post_handoff_attempt"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["source_handoff_attempt_name"],
        "approve_execution_runner_activation_handoff_attempt"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["source_handoff_name"],
        "approve_execution_runner_activation_admission_handoff"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["source_switch_name"],
        "approve_execution_runner_activation_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["structural_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["runner_control_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["phase_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["rollback_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["side_effect_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["activation_admission_required"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["activation_transaction_admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["activation_handoff_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["runner_activation_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["activation_handoff_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["handoff_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["activation_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["activation_post_handoff_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["activation_post_handoff_attempt_readiness_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["activation_success_admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["activation_switch_write_order"]
            .as_array()
            .expect("activation success admission readiness write order should be an array")
            .len(),
        15
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["would_unblock_success_return"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["would_admit_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["inherited_success_admission_blocked_gates"]
            .as_array()
            .expect("activation success admission readiness blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_post_handoff_attempt_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["inherited_post_handoff_attempt_readiness_blocked_gates"]
            .as_array()
            .expect("activation success admission readiness post-handoff readiness blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_handoff_attempt_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["inherited_handoff_enablement_blocked_gates"]
            .as_array()
            .expect("activation success admission readiness inherited enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("activation success admission readiness gates should be an array")
            .len(),
        1
    );
    assert!(
        review_body["approve_execution_runner_activation_success_admission_readiness_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("activation success admission readiness blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_post_handoff_attempt_ready"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]["status"],
        "approve_execution_runner_activation_success_return_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]["return_name"],
        "approve_execution_runner_activation_success_return"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["source_admission_name"],
        "approve_execution_runner_activation_success_admission"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["source_attempt_name"],
        "approve_execution_runner_activation_post_handoff_attempt"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["source_handoff_attempt_name"],
        "approve_execution_runner_activation_handoff_attempt"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["source_handoff_name"],
        "approve_execution_runner_activation_admission_handoff"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["source_switch_name"],
        "approve_execution_runner_activation_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["structural_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["runner_control_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["phase_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["rollback_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["side_effect_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["activation_admission_required"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["activation_transaction_admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["activation_handoff_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["runner_activation_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["activation_handoff_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["handoff_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["activation_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["activation_post_handoff_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["activation_success_admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["activation_success_return_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["activation_switch_write_order"]
            .as_array()
            .expect("activation success return write order should be an array")
            .len(),
        15
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["would_admit_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["inherited_success_admission_blocked_gates"]
            .as_array()
            .expect("activation success return inherited admission blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_post_handoff_attempt_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["inherited_post_handoff_attempt_blocked_gates"]
            .as_array()
            .expect("activation success return inherited post-handoff blockers should be an array",)
            .contains(&json!(
                "approve_execution_runner_activation_handoff_attempt_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["inherited_handoff_attempt_blocked_gates"]
            .as_array()
            .expect(
                "activation success return inherited handoff attempt blockers should be an array"
            )
            .contains(&json!(
                "approve_execution_runner_activation_handoff_enablement_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["inherited_handoff_enablement_blocked_gates"]
            .as_array()
            .expect("activation success return inherited enablement blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_transaction_admission_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]
            ["inherited_handoff_enablement_blocked_gates"]
            .as_array()
            .expect("activation success return inherited enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]["required_gates"]
            .as_array()
            .expect("activation success return gates should be an array")
            .len(),
        1
    );
    assert!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]["blocked_gates"]
            .as_array()
            .expect("activation success return blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_success_admission_ready"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["status"],
        "approve_execution_runner_activation_success_return_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["gate_name"],
        "approve_execution_runner_activation_success_return_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["return_name"],
        "approve_execution_runner_activation_success_return"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["source_admission_name"],
        "approve_execution_runner_activation_success_admission"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["source_attempt_name"],
        "approve_execution_runner_activation_post_handoff_attempt"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["source_handoff_attempt_name"],
        "approve_execution_runner_activation_handoff_attempt"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["source_handoff_name"],
        "approve_execution_runner_activation_admission_handoff"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["source_switch_name"],
        "approve_execution_runner_activation_enabled"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["structural_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["runner_control_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["phase_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["rollback_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["side_effect_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["activation_admission_required"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["activation_transaction_admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["activation_handoff_prerequisites_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["runner_activation_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["activation_handoff_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["handoff_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["activation_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["activation_post_handoff_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["activation_success_admission_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["activation_success_admission_readiness_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["activation_success_return_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["activation_switch_write_order"]
            .as_array()
            .expect("activation success return readiness write order should be an array")
            .len(),
        15
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["would_unblock_route_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["would_admit_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["inherited_success_return_blocked_gates"]
            .as_array()
            .expect(
                "activation success return readiness inherited return blockers should be an array"
            )
            .contains(&json!(
                "approve_execution_runner_activation_success_admission_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["inherited_success_admission_readiness_blocked_gates"]
            .as_array()
            .expect("activation success return readiness inherited admission readiness blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_post_handoff_attempt_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["inherited_success_admission_blocked_gates"]
            .as_array()
            .expect("activation success return readiness inherited admission blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_post_handoff_attempt_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["inherited_handoff_enablement_blocked_gates"]
            .as_array()
            .expect("activation success return readiness inherited enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["required_gates"]
            .as_array()
            .expect("activation success return readiness gates should be an array")
            .len(),
        1
    );
    assert!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("activation success return readiness blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_success_admission_ready"
            ))
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]["status"],
        "approve_execution_runner_route_success_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]["gate_name"],
        "approve_execution_runner_route_success_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["route_status_name"],
        "review_approve_executed"
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["source_return_name"],
        "approve_execution_runner_activation_success_return"
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["source_enablement_plan_name"],
        "approve_execution_runner_enablement_plan"
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["structural_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["runner_control_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["phase_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["rollback_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["side_effect_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["runner_activation_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["runner_activation_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["activation_success_return_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["activation_success_return_readiness_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["enablement_plan_success_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["route_success_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["would_set_route_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["would_mark_review_approve_executed"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["inherited_success_return_readiness_blocked_gates"]
            .as_array()
            .expect(
                "route success readiness inherited return readiness blockers should be an array"
            )
            .contains(&json!(
                "approve_execution_runner_activation_success_admission_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["inherited_success_return_blocked_gates"]
            .as_array()
            .expect("route success readiness inherited return blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_success_admission_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["inherited_enablement_plan_blocked_gates"]
            .as_array()
            .expect("route success readiness inherited enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_control_ready"))
    );
    assert!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["inherited_enablement_plan_blocked_enablements"]
            .as_array()
            .expect("route success readiness inherited blocked enablements should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]["required_gates"]
            .as_array()
            .expect("route success readiness gates should be an array")
            .len(),
        2
    );
    assert!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("route success readiness blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_success_return_ready"
            ))
    );
    assert!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("route success readiness blockers should be an array")
            .contains(&json!("approve_runner_success"))
    );
    let route_status_readiness =
        &review_body["approve_execution_runner_route_status_readiness_dry_run"];
    assert_eq!(
        route_status_readiness["status"],
        "approve_execution_runner_route_status_readiness_ready_blocked"
    );
    assert_eq!(
        route_status_readiness["gate_name"],
        "approve_execution_runner_route_status_ready"
    );
    assert_eq!(
        route_status_readiness["current_response_status"],
        "review_decision_execution_blocked"
    );
    assert_eq!(
        route_status_readiness["current_route_status"],
        "review_decision_execution_blocked"
    );
    assert_eq!(
        route_status_readiness["target_response_status"],
        "review_approve_executed"
    );
    assert_eq!(
        route_status_readiness["target_route_status"],
        "review_approve_executed"
    );
    assert_eq!(route_status_readiness["expected_http_status"], 423);
    assert_eq!(
        route_status_readiness["decision_execution_preflight_requested"],
        true
    );
    assert_eq!(route_status_readiness["review_execution_enabled"], false);
    assert_eq!(route_status_readiness["approve_runner_success"], false);
    assert_eq!(route_status_readiness["route_success_ready"], false);
    assert_eq!(route_status_readiness["route_status_ready"], false);
    assert_eq!(route_status_readiness["would_set_response_status"], false);
    assert_eq!(route_status_readiness["would_set_route_status"], false);
    assert_eq!(route_status_readiness["would_return_http_ok"], false);
    assert_eq!(route_status_readiness["would_touch_disk"], false);
    assert!(
        route_status_readiness["inherited_route_success_blocked_gates"]
            .as_array()
            .expect("route status readiness inherited route blockers should be an array")
            .contains(&json!(
                "approve_execution_runner_activation_success_return_ready"
            ))
    );
    assert!(route_status_readiness["inherited_blocked_reasons"]
        .as_array()
        .expect("route status readiness inherited reasons should be an array")
        .contains(&json!("approve_execution_not_enabled")));
    assert_eq!(
        route_status_readiness["required_gates"]
            .as_array()
            .expect("route status readiness gates should be an array")
            .len(),
        1
    );
    assert!(route_status_readiness["blocked_gates"]
        .as_array()
        .expect("route status readiness blockers should be an array")
        .contains(&json!("approve_execution_runner_route_success_ready")));
    let decision_lock_summary = &review_body["approve_execution_decision_lock_summary_dry_run"];
    assert_eq!(
        decision_lock_summary["status"],
        "approve_execution_decision_lock_summary_ready_blocked"
    );
    assert_eq!(decision_lock_summary["action"], "approve");
    assert_eq!(
        decision_lock_summary["response_status"],
        "review_decision_execution_blocked"
    );
    assert_eq!(
        decision_lock_summary["route_status"],
        "review_decision_execution_blocked"
    );
    assert_eq!(
        decision_lock_summary["target_response_status"],
        "review_approve_executed"
    );
    assert_eq!(
        decision_lock_summary["target_route_status"],
        "review_approve_executed"
    );
    assert_eq!(decision_lock_summary["expected_http_status"], 423);
    assert_eq!(
        decision_lock_summary["decision_execution_preflight_requested"],
        true
    );
    assert_eq!(decision_lock_summary["review_execution_enabled"], false);
    assert_eq!(decision_lock_summary["approve_runner_success"], false);
    assert_eq!(decision_lock_summary["route_status_ready"], false);
    assert_eq!(decision_lock_summary["final_execution_locked"], true);
    assert_eq!(
        decision_lock_summary["primary_blocked_reason"],
        "approve_execution_not_enabled"
    );
    assert_eq!(decision_lock_summary["blocked_reason_count"], 3);
    assert_eq!(decision_lock_summary["would_execute_decision"], false);
    assert_eq!(decision_lock_summary["would_mutate_contract"], false);
    assert_eq!(decision_lock_summary["would_return_http_ok"], false);
    assert_eq!(decision_lock_summary["would_touch_disk"], false);
    assert!(
        decision_lock_summary["inherited_route_status_blocked_gates"]
            .as_array()
            .expect("decision lock inherited route status blockers should be an array")
            .contains(&json!("approve_execution_runner_route_success_ready"))
    );
    assert!(decision_lock_summary["inherited_blocked_reasons"]
        .as_array()
        .expect("decision lock inherited blocked reasons should be an array")
        .contains(&json!("contract_mutation_api_disabled")));
    assert_eq!(
        review_body["approve_execution_runner_control_readiness_dry_run"]["status"],
        "approve_execution_runner_control_readiness_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_control_readiness_dry_run"]["gate_name"],
        "approve_execution_runner_control_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_control_readiness_dry_run"]["runner_attempt_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_control_readiness_dry_run"]["runner_execution_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_control_readiness_dry_run"]["dispatch_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_control_readiness_dry_run"]["call_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_control_readiness_dry_run"]["body_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_control_readiness_dry_run"]["phases_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_control_readiness_dry_run"]["runner_control_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_control_readiness_dry_run"]
            ["would_unblock_activation_control"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_control_readiness_dry_run"]["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_control_readiness_dry_run"]["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_control_readiness_dry_run"]["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_control_readiness_dry_run"]
            ["inherited_runner_attempt_blockers"]
            .as_array()
            .expect("runner control attempt blockers should be an array")
            .contains(&json!("approve_execution_runner_attempt_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_control_readiness_dry_run"]
            ["inherited_runner_outcome_blockers"]
            .as_array()
            .expect("runner control outcome blockers should be an array")
            .contains(&json!("approve_execution_runner_attempt_ready"))
    );
    assert!(
        review_body["approve_execution_runner_control_readiness_dry_run"]
            ["inherited_dispatch_blockers"]
            .as_array()
            .expect("runner control dispatch blockers should be an array")
            .contains(&json!("approve_execution_runner_execution_ready"))
    );
    assert!(
        review_body["approve_execution_runner_control_readiness_dry_run"]
            ["inherited_call_blockers"]
            .as_array()
            .expect("runner control call blockers should be an array")
            .contains(&json!("approve_execution_runner_handoff_ready"))
    );
    assert!(
        review_body["approve_execution_runner_control_readiness_dry_run"]
            ["inherited_body_blockers"]
            .as_array()
            .expect("runner control body blockers should be an array")
            .contains(&json!("approve_execution_runner_call_ready"))
    );
    assert!(
        review_body["approve_execution_runner_control_readiness_dry_run"]
            ["inherited_phase_sequence_blockers"]
            .as_array()
            .expect("runner control phase sequence blockers should be an array")
            .contains(&json!("approve_execution_runner_body_ready"))
    );
    assert_eq!(
        review_body["approve_execution_runner_control_readiness_dry_run"]["required_gates"]
            .as_array()
            .expect("runner control required gates should be an array")
            .len(),
        6
    );
    assert!(
        review_body["approve_execution_runner_control_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner control blockers should be an array")
            .contains(&json!("approve_execution_runner_attempt_ready"))
    );
    assert!(
        review_body["approve_execution_runner_control_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner control blockers should be an array")
            .contains(&json!("approve_execution_runner_phases_ready"))
    );
    assert_eq!(
        review_body["approve_execution_runner_enablement_plan_dry_run"]["status"],
        "approve_execution_runner_enablement_plan_ready_blocked"
    );
    assert_eq!(
        review_body["approve_execution_runner_enablement_plan_dry_run"]["structural_plan_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_enablement_plan_dry_run"]["runner_control_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_enablement_plan_dry_run"]["phase_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_enablement_plan_dry_run"]["rollback_chain_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_enablement_plan_dry_run"]
            ["side_effect_enablement_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_enablement_plan_dry_run"]
            ["runner_activation_enabled"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_enablement_plan_dry_run"]["runner_activation_ready"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_enablement_plan_dry_run"]["would_activate_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_enablement_plan_dry_run"]["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_enablement_plan_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_enablement_plan_dry_run"]["required_enablements"]
            .as_array()
            .expect("approve runner required enablements should be an array")
            .len(),
        15
    );
    assert!(
        review_body["approve_execution_runner_enablement_plan_dry_run"]["blocked_enablements"]
            .as_array()
            .expect("approve runner blocked enablements should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_enablement_plan_dry_run"]["blocked_enablements"]
            .as_array()
            .expect("approve runner blocked enablements should be an array")
            .contains(&json!("approve_execution_runner_call_enabled"))
    );
    assert!(
        review_body["approve_execution_runner_enablement_plan_dry_run"]["blocked_enablements"]
            .as_array()
            .expect("approve runner blocked enablements should be an array")
            .contains(&json!(
                "approve_execution_runner_rollback_execution_enabled"
            ))
    );
    assert!(
        review_body["approve_execution_runner_enablement_plan_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_control_ready"))
    );
    assert!(
        review_body["approve_execution_runner_enablement_plan_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve runner enablement blockers should be an array")
            .contains(&json!("approve_execution_runner_activation_enabled"))
    );

    let (detail_status, detail_body) = get_contract_repair_path_with_app(
        app,
        &format!("/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}"),
    )
    .await;

    assert_eq!(detail_status, StatusCode::OK, "{detail_body}");
    assert_eq!(detail_body["record"]["review_state"], "pending");
    assert_eq!(
        detail_body["record"]["transient_review_status"],
        "approve_intent_recorded"
    );
    assert_eq!(detail_body["record"]["transient_review_action"], "approve");
    assert_eq!(detail_body["mutation_enabled"], false);
}

#[tokio::test(flavor = "multi_thread")]
async fn contract_repair_approval_approve_live_route_environment_executes_durable_disk_application()
{
    let (app, dirs) =
        common::test_app_with_dirs("api_v4_productization_contract_repair_approve_live_probe");
    let grants_path = dirs
        .backtest_store_dir
        .parent()
        .expect("test backtest store should have a storage parent")
        .join("contract-repair-reviewer-grants.json");
    std::fs::write(
        &grants_path,
        r#"{"policy_version":"quantpilot/contract-repair-reviewer-role-policy/v1","grants":[{"subject":"user:0","role":"contract_repair_reviewer"}]}"#,
    )
    .expect("reviewer grant file should be writable");
    let contract_repair_store_dir = dirs
        .backtest_store_dir
        .parent()
        .expect("test backtest store should have a storage parent")
        .join("contract-repair-approvals");
    std::fs::create_dir_all(&contract_repair_store_dir)
        .expect("contract repair approval store should be ready for marker guard evidence");
    write_contract_source_fixture(
        &dirs,
        "graph-dual-ma-live",
        "v4-live-test",
        "sha256:test-contract-live",
    );
    let (post_status, post_body) = post_contract_repair_request_with_app(
        app.clone(),
        json!({
            "status": "body_preview_only",
            "payload_kind": "v4_contract_repair_approval_request",
            "request_id": "approval-request:repair-draft:memory_schema/decision.dual_ma/live-route/last_signal_at",
            "target_path": "memory_schema/decision.dual_ma/live-route/last_signal_at",
            "target_kind": "memory_field",
            "changed_fields": ["type_name"],
            "patch_payload": {"type_name": "time?"},
            "contract_source_ref": {
                "source_kind": "v4_machine_graph_contract",
                "source_id": "graph-dual-ma-live",
                "version": "v4-live-test",
                "artifact_digest": "sha256:test-contract-live"
            },
            "mutation_enabled": false,
            "review_required": true
        }),
    )
    .await;
    let approval_id = post_body["approval_record_preview"]["approval_id"]
        .as_str()
        .expect("approval id should be present")
        .to_string();

    assert_eq!(post_status, StatusCode::LOCKED, "{post_body}");

    let (review_status, review_body) =
        post_contract_repair_path_with_app_with_approve_live_route_env(
            app.clone(),
            &format!(
            "/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}/review"
        ),
            json!({
                "action": "approve",
                "reviewer_id": "user:0",
                "reason": "approve with explicit live route evidence",
                "review_enabled": true
            }),
            "verified-live-approve-route",
        )
        .await;

    assert_eq!(review_status, StatusCode::OK, "{review_body}");
    assert_eq!(review_body["status"], "review_approve_executed");
    assert_eq!(review_body["route_status"], "review_approve_executed");
    assert_eq!(review_body["review_enabled"], true);
    assert_eq!(review_body["persistence_enabled"], false);
    assert_eq!(review_body["mutation_enabled"], false);
    assert_eq!(review_body["execution_gate"]["status"], "ready");
    assert!(review_body["execution_gate"]["blocked_gates"]
        .as_array()
        .expect("execution gate blockers should be an array")
        .is_empty());
    assert!(review_body["execution_gate"]["passed_gates"]
        .as_array()
        .expect("execution gate passed gates should be an array")
        .contains(&json!("lifecycle_event_emission_enabled")));
    assert!(review_body["execution_gate"]["passed_gates"]
        .as_array()
        .expect("execution gate passed gates should be an array")
        .contains(&json!("contract_mutation_api_enabled")));
    assert_eq!(
        review_body["execution_plan_preview"]["execution_enabled"],
        true
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["status"],
        "transition_executed"
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["source_write_dry_run"]["status"],
        "contract_source_write_ready_blocked"
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["source_write_dry_run"]["would_write_source"],
        false
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["source_write_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["lifecycle_emission_enablement_gate"]["status"],
        "lifecycle_emission_enablement_ready"
    );
    assert_eq!(
        review_body["lifecycle_emission_enablement_gate"]["lifecycle_effects_ready"],
        true
    );
    assert_eq!(
        review_body["lifecycle_emission_enablement_gate"]["would_touch_lifecycle_log"],
        true
    );
    assert_eq!(
        review_body["approve_execution_lifecycle_effects_readiness_dry_run"]["status"],
        "approve_execution_lifecycle_effects_readiness_ready"
    );
    assert_eq!(
        review_body["contract_mutation_enablement_gate"]["status"],
        "contract_mutation_enablement_ready"
    );
    assert_eq!(
        review_body["contract_mutation_enablement_gate"]["contract_mutation_api_enabled"],
        true
    );
    assert_eq!(
        review_body["contract_mutation_enablement_gate"]["mutation_ready"],
        true
    );
    assert_eq!(
        review_body["contract_mutation_enablement_gate"]["would_mutate_contract"],
        false
    );
    assert_eq!(
        review_body["contract_mutation_enablement_gate"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]["status"],
        "approve_execution_contract_mutation_readiness_ready"
    );
    assert_eq!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]
            ["contract_mutation_ready"],
        true
    );
    assert!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("contract mutation readiness blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]
            ["would_mutate_contract"],
        false
    );
    assert_eq!(
        review_body["approve_execution_contract_mutation_readiness_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_gate"]["status"],
        "approve_execution_ready"
    );
    assert_eq!(
        review_body["approve_execution_gate"]["mutation_api_enabled"],
        true
    );
    assert_eq!(
        review_body["approve_execution_gate"]["approve_execution_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_gate"]["would_execute"],
        false
    );
    assert!(review_body["approve_execution_gate"]["blocked_by"]
        .as_array()
        .expect("approve execution gate blockers should be an array")
        .is_empty());
    assert_eq!(
        review_body["approve_execution_transaction_dry_run"]["status"],
        "approve_execution_transaction_ready"
    );
    assert_eq!(
        review_body["approve_execution_transaction_dry_run"]["execution_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_dry_run"]["mutation_api_enabled"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_dry_run"]["would_execute_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_dry_run"]["would_write_contract_source"],
        false
    );
    assert!(
        review_body["approve_execution_transaction_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve execution transaction blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_admission_gate"]["status"],
        "approve_execution_admission_ready"
    );
    assert_eq!(
        review_body["approve_execution_admission_gate"]["transaction_runner_enabled"],
        true
    );
    assert_eq!(
        review_body["approve_execution_admission_gate"]["admission_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_admission_gate"]["partial_execution_allowed"],
        false
    );
    assert_eq!(
        review_body["approve_execution_admission_gate"]["would_start_transaction"],
        true
    );
    assert_eq!(
        review_body["approve_execution_admission_gate"]["would_persist_any_side_effect"],
        false
    );
    assert!(
        review_body["approve_execution_admission_gate"]["blocked_gates"]
            .as_array()
            .expect("approve execution admission blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]["status"],
        "approve_execution_transaction_runner_enablement_ready"
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]
            ["runner_enablement_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]
            ["would_start_runner"],
        false
    );
    assert!(
        review_body["approve_execution_transaction_runner_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("approve execution runner enablement blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_dry_run"]["status"],
        "approve_execution_transaction_runner_ready"
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_dry_run"]["runner_enabled"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_dry_run"]["admission_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_dry_run"]["commit_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_dry_run"]["would_start_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_dry_run"]["would_write_recovery_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_runner_dry_run"]["would_commit_transaction"],
        false
    );
    assert!(
        review_body["approve_execution_transaction_runner_dry_run"]["blocked_by"]
            .as_array()
            .expect("approve execution runner blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_write_dry_run"]["status"],
        "approve_execution_recovery_marker_write_ready"
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_write_dry_run"]["write_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_write_dry_run"]["runner_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_write_dry_run"]["would_write_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_write_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_idempotency_precheck"]
            ["safe_to_write_marker"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_idempotency_precheck"]["would_write_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_gate"]["status"],
        "approve_execution_recovery_marker_persistence_ready"
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_gate"]["runner_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_gate"]
            ["marker_persistence_enabled"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_gate"]["persistence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_gate"]["would_persist_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_gate"]["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_recovery_marker_persistence_gate"]["blocked_gates"]
            .as_array()
            .expect("recovery marker persistence blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]["status"],
        "approve_execution_recovery_marker_persistence_readiness_ready"
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["marker_persistence_enabled"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["recovery_marker_persistence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["would_persist_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_recovery_marker_persistence_readiness_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("recovery marker persistence readiness blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_gate"]["status"],
        "approve_execution_transaction_commit_ready"
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_gate"]
            ["recovery_marker_persistence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_gate"]["commit_gate_enabled"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_gate"]["commit_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_gate"]["would_commit_transaction"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_gate"]["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_transaction_commit_gate"]["blocked_gates"]
            .as_array()
            .expect("transaction commit blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]["status"],
        "approve_execution_transaction_commit_readiness_ready"
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]
            ["recovery_marker_persistence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]
            ["commit_gate_enabled"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]
            ["transaction_commit_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]
            ["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_transaction_commit_readiness_dry_run"]["blocked_gates"]
            .as_array()
            .expect("transaction commit readiness blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["status"],
        "approve_execution_atomic_side_effects_ready"
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["lifecycle_effects_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["contract_mutation_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]
            ["recovery_marker_persistence_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["transaction_commit_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["atomic_side_effects_enabled"],
        true
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["atomic_side_effects_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["would_emit_lifecycle"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["would_mutate_contract"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["would_persist_recovery_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_gate"]["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_atomic_side_effects_gate"]["blocked_gates"]
            .as_array()
            .expect("atomic side effects blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]["status"],
        "approve_execution_atomic_side_effects_enablement_ready"
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["enablement_prerequisites_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["atomic_side_effects_enablement_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["atomic_side_effects_enabled"],
        true
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["would_emit_lifecycle"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["would_mutate_contract"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["would_persist_recovery_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]
            ["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_enablement_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]["status"],
        "approve_execution_atomic_side_effects_readiness_ready"
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]
            ["atomic_side_effects_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_atomic_side_effects_readiness_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]["status"],
        "approve_execution_runner_attempt_enablement_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]
            ["runner_attempt_enabled"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]
            ["runner_attempt_enablement_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]["would_start_runner"],
        false
    );
    assert!(
        review_body["approve_execution_runner_attempt_enablement_dry_run"]["blocked_gates"]
            .as_array()
            .expect("runner attempt enablement blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt"]["status"],
        "approve_execution_runner_attempt_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt"]["runner_attempt_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_attempt"]["would_start_runner"],
        false
    );
    assert!(
        review_body["approve_execution_runner_attempt"]["blocked_by"]
            .as_array()
            .expect("runner attempt blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]["status"],
        "approve_execution_runner_execution_readiness_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]
            ["runner_execution_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_execution_readiness_dry_run"]["would_start_runner"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_dispatch_readiness_dry_run"]["status"],
        "approve_execution_runner_dispatch_readiness_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_call_readiness_dry_run"]["status"],
        "approve_execution_runner_call_readiness_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_body_readiness_dry_run"]["status"],
        "approve_execution_runner_body_readiness_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_phase_execution_enablement_dry_run"]["status"],
        "approve_execution_runner_phase_execution_enablement_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_body_phase_sequence_dry_run"]["status"],
        "approve_execution_runner_body_phase_sequence_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]["status"],
        "approve_execution_runner_phases_readiness_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_phases_readiness_dry_run"]["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]["status"],
        "approve_execution_runner_lifecycle_phase_enablement_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]
            ["lifecycle_phase_enabled"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_enablement_dry_run"]
            ["lifecycle_phase_enablement_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_dry_run"]["status"],
        "approve_execution_runner_lifecycle_phase_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_dry_run"]["lifecycle_phase_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_dry_run"]["would_emit_lifecycle"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_dry_run"]
            ["would_touch_lifecycle_log"],
        false
    );
    assert!(
        review_body["approve_execution_runner_lifecycle_phase_dry_run"]["blocked_by"]
            .as_array()
            .expect("lifecycle phase blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]["status"],
        "approve_execution_runner_lifecycle_phase_readiness_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["would_unblock_source_mutation_phase"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_lifecycle_phase_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]["status"],
        "approve_execution_runner_source_mutation_phase_enablement_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["source_mutation_phase_enabled"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["source_mutation_phase_enablement_ready"],
        true
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("source mutation phase enablement blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]["status"],
        "approve_execution_runner_source_mutation_phase_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]
            ["source_mutation_phase_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]
            ["would_write_contract_source"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_dry_run"]["blocked_by"]
            .as_array()
            .expect("source mutation phase blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]["status"],
        "approve_execution_runner_source_mutation_phase_readiness_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["source_mutation_phase_ready"],
        true
    );
    assert!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("source mutation phase readiness blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["would_write_contract_source"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_source_mutation_phase_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
            ["status"],
        "approve_execution_runner_recovery_marker_cleanup_phase_enablement_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
            ["cleanup_phase_enabled"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
            ["cleanup_phase_enablement_ready"],
        true
    );
    assert!(review_body
        ["approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run"]
        ["blocked_gates"]
        .as_array()
        .expect("cleanup phase enablement blockers should be an array")
        .is_empty());
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]["status"],
        "approve_execution_runner_recovery_marker_cleanup_phase_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]
            ["source_mutation_phase_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]
            ["cleanup_phase_enabled"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]
            ["cleanup_phase_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]
            ["would_clear_recovery_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_dry_run"]["blocked_by"]
            .as_array()
            .expect("cleanup phase blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["status"],
        "approve_execution_runner_recovery_marker_cleanup_phase_readiness_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["cleanup_phase_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["would_clear_recovery_marker"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(review_body
        ["approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run"]
        ["blocked_gates"]
        .as_array()
        .expect("cleanup phase readiness blockers should be an array")
        .is_empty());
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["status"],
        "approve_execution_runner_transaction_commit_phase_enablement_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["commit_phase_enabled"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["commit_phase_enablement_ready"],
        true
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("commit phase enablement blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]["status"],
        "approve_execution_runner_transaction_commit_phase_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]
            ["cleanup_phase_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]
            ["commit_phase_enabled"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]
            ["commit_phase_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]
            ["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_dry_run"]["blocked_by"]
            .as_array()
            .expect("commit phase blockers should be an array")
            .is_empty()
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["status"]
            == json!("approve_execution_runner_transaction_commit_phase_readiness_ready")
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["commit_phase_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["would_commit_transaction"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert!(
        review_body["approve_execution_runner_transaction_commit_phase_readiness_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("commit phase readiness blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["status"],
        "approve_execution_runner_rollback_execution_phase_enablement_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["commit_phase_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["rollback_execution_enabled"],
        true
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_enablement_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("rollback execution enablement blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]["status"],
        "approve_execution_runner_rollback_execution_phase_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]
            ["commit_phase_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]
            ["rollback_execution_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]
            ["would_restore_contract_source"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["status"],
        "approve_execution_runner_rollback_execution_phase_readiness_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["would_rollback_on_error"],
        false
    );
    assert!(
        review_body["approve_execution_runner_rollback_execution_phase_readiness_dry_run"]
            ["blocked_gates"]
            .as_array()
            .expect("rollback readiness blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_enablement_dry_run"]["status"],
        "approve_execution_runner_activation_enablement_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_path_dry_run"]["status"],
        "approve_execution_runner_activation_path_ready"
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["transaction_enabled"],
        true
    );
    assert_eq!(
        review_body
            ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
            ["status"],
        "approve_execution_runner_activation_switch_write_transaction_enablement_ready"
    );
    assert!(review_body
        ["approve_execution_runner_activation_switch_write_transaction_enablement_dry_run"]
        ["blocked_gates"]
        .as_array()
        .expect("activation switch write transaction blockers should be an array")
        .is_empty());
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["status"],
        "approve_execution_runner_activation_switch_write_transaction_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["would_write_switches"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_switch_write_transaction_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_gate_dry_run"]
            ["status"],
        "approve_execution_runner_activation_transaction_admission_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_transaction_admission_readiness_dry_run"]
            ["status"],
        "approve_execution_runner_activation_transaction_admission_readiness_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_admission_handoff_dry_run"]["status"],
        "approve_execution_runner_activation_admission_handoff_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_enablement_dry_run"]["status"],
        "approve_execution_runner_activation_handoff_enablement_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_handoff_attempt_dry_run"]["status"],
        "approve_execution_runner_activation_handoff_attempt_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_post_handoff_attempt_dry_run"]["status"],
        "approve_execution_runner_activation_post_handoff_attempt_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_admission_dry_run"]["status"],
        "approve_execution_runner_activation_success_admission_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_dry_run"]["status"],
        "approve_execution_runner_activation_success_return_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["status"],
        "approve_execution_runner_activation_success_return_readiness_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["would_return_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_activation_success_return_readiness_dry_run"]
            ["would_touch_disk"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["enablement_plan_success_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]["status"],
        "approve_execution_runner_route_success_readiness_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["route_success_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_route_status_readiness_dry_run"]
            ["route_success_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_route_status_readiness_dry_run"]["status"],
        "approve_execution_runner_route_status_readiness_ready"
    );
    assert_eq!(
        review_body["approve_execution_runner_route_status_readiness_dry_run"]
            ["route_status_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_runner_route_status_readiness_dry_run"]
            ["approve_runner_success"],
        false
    );
    assert_eq!(
        review_body["approve_execution_runner_route_status_readiness_dry_run"]
            ["review_execution_enabled"],
        false
    );
    let formal_review_execution_readiness =
        &review_body["approve_execution_formal_review_execution_readiness_dry_run"];
    assert_eq!(
        formal_review_execution_readiness["status"],
        "approve_execution_formal_review_execution_readiness_ready"
    );
    assert_eq!(
        formal_review_execution_readiness["gate_name"],
        "formal_approve_review_execution_ready"
    );
    assert_eq!(
        formal_review_execution_readiness["review_execution_gate_clear"],
        true
    );
    assert_eq!(
        formal_review_execution_readiness["approve_execution_ready"],
        true
    );
    assert_eq!(
        formal_review_execution_readiness["route_success_ready"],
        true
    );
    assert_eq!(
        formal_review_execution_readiness["formal_approve_review_execution_enabled"],
        true
    );
    assert_eq!(
        formal_review_execution_readiness["review_execution_enabled"],
        false
    );
    assert_eq!(
        formal_review_execution_readiness["formal_review_execution_ready"],
        true
    );
    assert_eq!(
        formal_review_execution_readiness["would_execute_decision"],
        false
    );
    assert_eq!(
        formal_review_execution_readiness["would_persist_approval_record"],
        false
    );
    assert_eq!(
        formal_review_execution_readiness["would_mutate_contract"],
        false
    );
    assert_eq!(
        formal_review_execution_readiness["would_return_http_ok"],
        false
    );
    assert_eq!(formal_review_execution_readiness["would_touch_disk"], false);
    let formal_review_blocked_gates = formal_review_execution_readiness["blocked_gates"]
        .as_array()
        .expect("formal review readiness blockers should be an array");
    assert!(formal_review_blocked_gates.is_empty());
    let formal_review_execution_blockers = formal_review_execution_readiness
        ["inherited_execution_blocked_gates"]
        .as_array()
        .expect("formal review inherited execution blockers should be an array");
    let formal_review_approve_blockers = formal_review_execution_readiness
        ["inherited_approve_execution_blockers"]
        .as_array()
        .expect("formal review inherited approve blockers should be an array");
    let formal_review_route_success_blockers = formal_review_execution_readiness
        ["inherited_route_success_blocked_gates"]
        .as_array()
        .expect("formal review inherited route success blockers should be an array");
    assert!(formal_review_execution_blockers.is_empty());
    assert!(formal_review_approve_blockers.is_empty());
    assert!(formal_review_route_success_blockers.is_empty());
    let final_atomic_readiness = &review_body["approve_execution_final_atomic_readiness_dry_run"];
    assert_eq!(
        final_atomic_readiness["status"],
        "approve_execution_final_atomic_readiness_ready"
    );
    assert_eq!(final_atomic_readiness["review_execution_enabled"], false);
    assert_eq!(
        final_atomic_readiness["formal_review_execution_ready"],
        true
    );
    assert_eq!(final_atomic_readiness["final_atomic_execution_ready"], true);
    assert_eq!(final_atomic_readiness["would_execute_decision"], false);
    assert_eq!(
        final_atomic_readiness["would_persist_approval_record"],
        false
    );
    assert_eq!(final_atomic_readiness["would_mutate_contract"], false);
    assert_eq!(
        final_atomic_readiness["would_persist_recovery_marker"],
        false
    );
    assert_eq!(final_atomic_readiness["would_commit_transaction"], false);
    assert_eq!(final_atomic_readiness["would_return_http_ok"], false);
    assert_eq!(final_atomic_readiness["would_touch_disk"], false);
    let final_atomic_readiness_blockers = final_atomic_readiness["blocked_gates"]
        .as_array()
        .expect("final atomic readiness blockers should be an array");
    assert!(final_atomic_readiness_blockers.is_empty());
    let final_atomic_execution_plan =
        &review_body["approve_execution_final_atomic_execution_plan_dry_run"];
    assert_eq!(
        final_atomic_execution_plan["status"],
        "approve_execution_final_atomic_execution_plan_ready"
    );
    assert_eq!(
        final_atomic_execution_plan["final_atomic_readiness_ready"],
        true
    );
    assert_eq!(
        final_atomic_execution_plan["formal_review_execution_ready"],
        true
    );
    assert_eq!(
        final_atomic_execution_plan["review_execution_enabled"],
        false
    );
    assert_eq!(
        final_atomic_execution_plan["final_atomic_execution_plan_ready"],
        true
    );
    assert_eq!(
        final_atomic_execution_plan["would_start_atomic_execution"],
        false
    );
    assert_eq!(final_atomic_execution_plan["would_touch_disk"], false);
    assert!(final_atomic_execution_plan["blocked_gates"]
        .as_array()
        .expect("final atomic execution plan blockers should be an array")
        .is_empty());
    let final_atomic_admission =
        &review_body["approve_execution_final_atomic_admission_gate_dry_run"];
    assert_eq!(
        final_atomic_admission["status"],
        "approve_execution_final_atomic_admission_gate_ready"
    );
    assert_eq!(final_atomic_admission["final_atomic_readiness_ready"], true);
    assert_eq!(
        final_atomic_admission["final_atomic_execution_plan_ready"],
        true
    );
    assert_eq!(
        final_atomic_admission["formal_review_execution_ready"],
        true
    );
    assert_eq!(final_atomic_admission["review_execution_enabled"], false);
    assert_eq!(final_atomic_admission["admission_ready"], true);
    assert_eq!(final_atomic_admission["would_enter_final_execution"], false);
    assert_eq!(final_atomic_admission["would_touch_disk"], false);
    assert!(final_atomic_admission["blocked_gates"]
        .as_array()
        .expect("final atomic admission blockers should be an array")
        .is_empty());
    let final_execution_entry = &review_body["approve_execution_final_execution_entry_dry_run"];
    assert_eq!(
        final_execution_entry["status"],
        "approve_execution_final_execution_entry_ready"
    );
    assert_eq!(
        final_execution_entry["entry_name"],
        "approve_execution_final_execution_entry"
    );
    assert_eq!(final_execution_entry["admission_ready"], true);
    assert_eq!(final_execution_entry["review_execution_enabled"], false);
    assert_eq!(
        final_execution_entry["approve_final_execution_enabled"],
        true
    );
    assert_eq!(final_execution_entry["route_status_ready"], true);
    assert_eq!(final_execution_entry["rollback_order_ready"], true);
    assert_eq!(final_execution_entry["no_partial_execution_ready"], true);
    assert_eq!(final_execution_entry["final_execution_entry_ready"], true);
    assert_eq!(final_execution_entry["would_enter_final_execution"], true);
    assert_eq!(final_execution_entry["would_execute_decision"], true);
    assert_eq!(
        final_execution_entry["would_persist_approval_record"],
        false
    );
    assert_eq!(final_execution_entry["would_mutate_contract"], false);
    assert_eq!(
        final_execution_entry["would_persist_recovery_marker"],
        false
    );
    assert_eq!(final_execution_entry["would_clear_recovery_marker"], false);
    assert_eq!(final_execution_entry["would_commit_transaction"], false);
    assert_eq!(final_execution_entry["would_return_http_ok"], true);
    assert_eq!(final_execution_entry["would_touch_disk"], true);
    let final_execution_entry_blockers = final_execution_entry["blocked_gates"]
        .as_array()
        .expect("final execution entry blockers should be an array");
    assert!(final_execution_entry_blockers.is_empty());
    assert!(!final_execution_entry_blockers.contains(&json!("review_execution_enabled")));
    assert!(!final_execution_entry_blockers
        .contains(&json!("approve_execution_runner_route_status_ready")));
    assert!(!final_execution_entry_blockers.contains(&json!("rollback_order_ready")));
    assert!(!final_execution_entry_blockers.contains(&json!("no_partial_execution_ready")));
    assert!(final_execution_entry["inherited_admission_blocked_gates"]
        .as_array()
        .expect("final execution entry inherited admission blockers should be an array")
        .is_empty());
    assert!(
        final_execution_entry["inherited_route_status_blocked_gates"]
            .as_array()
            .expect("final execution entry inherited route-status blockers should be an array")
            .is_empty()
    );
    let final_execution_switch =
        &review_body["approve_execution_final_execution_switch_readiness_dry_run"];
    assert_eq!(
        final_execution_switch["status"],
        "approve_execution_final_execution_switch_readiness_ready"
    );
    assert_eq!(
        final_execution_switch["switch_name"],
        "approve_final_execution_enabled"
    );
    assert_eq!(final_execution_switch["final_execution_entry_ready"], true);
    assert_eq!(
        final_execution_switch["approve_final_execution_enabled"],
        true
    );
    assert_eq!(final_execution_switch["record_write_ready"], true);
    assert_eq!(final_execution_switch["contract_mutation_ready"], true);
    assert_eq!(
        final_execution_switch["recovery_marker_persistence_ready"],
        true
    );
    assert_eq!(final_execution_switch["cleanup_phase_ready"], true);
    assert_eq!(
        final_execution_switch["transaction_commit_phase_ready"],
        true
    );
    assert_eq!(final_execution_switch["route_status_ready"], true);
    assert_eq!(final_execution_switch["rollback_order_ready"], true);
    assert_eq!(final_execution_switch["no_partial_execution_ready"], true);
    assert_eq!(final_execution_switch["final_execution_switch_ready"], true);
    assert_eq!(
        final_execution_switch["final_execution_switch_enabled"],
        true
    );
    assert_eq!(final_execution_switch["side_effect_replay_required"], false);
    assert_eq!(final_execution_switch["would_enable_final_execution"], true);
    assert_eq!(final_execution_switch["would_touch_disk"], true);
    assert!(final_execution_switch["blocked_gates"]
        .as_array()
        .expect("final execution switch blockers should be an array")
        .is_empty());
    assert!(
        final_execution_switch["inherited_final_entry_blocked_gates"]
            .as_array()
            .expect("final execution switch inherited entry blockers should be an array")
            .is_empty()
    );
    assert!(final_execution_switch["replay_order"]
        .as_array()
        .expect("final execution switch replay order should be an array")
        .contains(&json!("return_review_approve_executed")));
    let final_execution_rollback =
        &review_body["approve_execution_final_execution_rollback_readiness_dry_run"];
    assert_eq!(
        final_execution_rollback["status"],
        "approve_execution_final_execution_rollback_readiness_ready"
    );
    assert_eq!(
        final_execution_rollback["gate_name"],
        "approve_execution_final_execution_rollback_ready"
    );
    assert_eq!(
        final_execution_rollback["final_execution_switch_ready"],
        true
    );
    assert_eq!(
        final_execution_rollback["approve_final_execution_enabled"],
        true
    );
    assert_eq!(final_execution_rollback["rollback_execution_ready"], true);
    assert_eq!(final_execution_rollback["rollback_plan_ready"], true);
    assert_eq!(final_execution_rollback["rollback_order_ready"], true);
    assert_eq!(final_execution_rollback["failure_window_covered"], true);
    assert_eq!(
        final_execution_rollback["final_execution_rollback_ready"],
        true
    );
    assert_eq!(final_execution_rollback["would_rollback_on_error"], true);
    assert_eq!(
        final_execution_rollback["would_restore_contract_source"],
        true
    );
    assert_eq!(
        final_execution_rollback["would_restore_approval_record"],
        true
    );
    assert_eq!(
        final_execution_rollback["would_mark_recovery_marker_rolled_back"],
        true
    );
    assert_eq!(final_execution_rollback["would_touch_disk"], true);
    assert!(final_execution_rollback["rollback_order"]
        .as_array()
        .expect("final execution rollback order should be an array")
        .contains(&json!("restore_contract_source")));
    assert!(final_execution_rollback["rollback_order"]
        .as_array()
        .expect("final execution rollback order should be an array")
        .contains(&json!("restore_approval_record")));
    assert!(final_execution_rollback["rollback_order"]
        .as_array()
        .expect("final execution rollback order should be an array")
        .contains(&json!("mark_recovery_marker_rolled_back")));
    assert!(final_execution_rollback["blocked_gates"]
        .as_array()
        .expect("final execution rollback blockers should be an array")
        .is_empty());
    assert!(final_execution_rollback["inherited_switch_blocked_gates"]
        .as_array()
        .expect("final execution rollback inherited switch blockers should be an array")
        .is_empty());
    assert!(
        final_execution_rollback["inherited_rollback_execution_blocked_gates"]
            .as_array()
            .expect("final execution rollback inherited phase blockers should be an array")
            .is_empty()
    );
    let final_execution_replay =
        &review_body["approve_execution_final_execution_replay_plan_dry_run"];
    assert_eq!(
        final_execution_replay["status"],
        "approve_execution_final_execution_replay_plan_ready"
    );
    assert_eq!(
        final_execution_replay["plan_name"],
        "approve_execution_final_execution_replay_plan"
    );
    assert_eq!(final_execution_replay["final_execution_switch_ready"], true);
    assert_eq!(
        final_execution_replay["final_execution_rollback_ready"],
        true
    );
    assert_eq!(
        final_execution_replay["approve_final_execution_enabled"],
        true
    );
    assert_eq!(final_execution_replay["replay_order_ready"], true);
    assert_eq!(final_execution_replay["rollback_order_ready"], true);
    assert_eq!(final_execution_replay["replay_plan_ready"], true);
    assert_eq!(final_execution_replay["replay_enabled"], true);
    assert_eq!(final_execution_replay["side_effect_replay_required"], false);
    assert_eq!(final_execution_replay["would_replay_side_effects"], true);
    assert_eq!(final_execution_replay["would_enter_final_execution"], true);
    assert_eq!(final_execution_replay["would_return_http_ok"], true);
    assert_eq!(final_execution_replay["would_touch_disk"], true);
    assert!(final_execution_replay["missing_replay_phases"]
        .as_array()
        .expect("final execution replay missing phases should be an array")
        .is_empty());
    let replay_order = final_execution_replay["replay_order"]
        .as_array()
        .expect("final execution replay order should be an array");
    assert!(replay_order.contains(&json!("write_recovery_marker")));
    assert!(replay_order.contains(&json!("persist_approval_record")));
    assert!(replay_order.contains(&json!("write_contract_source")));
    assert!(replay_order.contains(&json!("return_review_approve_executed")));
    assert!(final_execution_replay["rollback_order"]
        .as_array()
        .expect("final execution replay rollback order should be an array")
        .contains(&json!("restore_contract_source")));
    assert!(final_execution_replay["blocked_gates"]
        .as_array()
        .expect("final execution replay blockers should be an array")
        .is_empty());
    let final_execution_executor =
        &review_body["approve_execution_final_execution_replay_executor_dry_run"];
    assert_eq!(
        final_execution_executor["status"],
        "approve_execution_final_execution_replay_executor_ready"
    );
    assert_eq!(
        final_execution_executor["executor_name"],
        "approve_execution_final_execution_replay_executor"
    );
    assert_eq!(final_execution_executor["replay_plan_ready"], true);
    assert_eq!(
        final_execution_executor["final_execution_rollback_ready"],
        true
    );
    assert_eq!(
        final_execution_executor["approve_final_execution_enabled"],
        true
    );
    assert_eq!(final_execution_executor["replay_executor_ready"], true);
    assert_eq!(final_execution_executor["replay_executor_admitted"], true);
    assert_eq!(
        final_execution_executor["replay_executor_order_ready"],
        true
    );
    assert_eq!(
        final_execution_executor["expected_replay_order"],
        json!([
            "write_recovery_marker",
            "transition_review_state",
            "persist_approval_record",
            "emit_lifecycle_event",
            "append_lifecycle_entry",
            "write_contract_source",
            "clear_recovery_marker",
            "commit_transaction",
            "dispatch_route_success",
            "return_review_approve_executed"
        ])
    );
    assert!(final_execution_executor["missing_executor_replay_phases"]
        .as_array()
        .expect("final execution executor missing phases should be an array")
        .is_empty());
    assert!(
        final_execution_executor["unexpected_executor_replay_phases"]
            .as_array()
            .expect("final execution executor unexpected phases should be an array")
            .is_empty()
    );
    assert_eq!(
        final_execution_executor["would_start_replay_executor"],
        true
    );
    assert_eq!(
        final_execution_executor["would_write_recovery_marker"],
        true
    );
    assert_eq!(
        final_execution_executor["would_persist_approval_record"],
        true
    );
    assert_eq!(
        final_execution_executor["would_write_contract_source"],
        true
    );
    assert_eq!(
        final_execution_executor["would_dispatch_route_success"],
        true
    );
    assert_eq!(final_execution_executor["would_return_http_ok"], true);
    assert_eq!(final_execution_executor["would_touch_disk"], true);
    assert!(final_execution_executor["blocked_gates"]
        .as_array()
        .expect("final execution executor blockers should be an array")
        .is_empty());
    assert!(!final_execution_executor["blocked_gates"]
        .as_array()
        .expect("final execution executor blockers should be an array")
        .contains(&json!("replay_executor_order_ready")));
    assert!(
        final_execution_executor["inherited_replay_plan_blocked_gates"]
            .as_array()
            .expect("final execution executor inherited replay blockers should be an array")
            .is_empty()
    );
    assert!(final_execution_executor["replay_order"]
        .as_array()
        .expect("final execution executor replay order should be an array")
        .contains(&json!("write_recovery_marker")));
    let final_execution_routing =
        &review_body["approve_execution_final_execution_replay_executor_routing_dry_run"];
    assert_eq!(
        final_execution_routing["status"],
        "approve_execution_final_execution_replay_executor_routing_ready"
    );
    assert_eq!(
        final_execution_routing["routing_name"],
        "approve_execution_final_execution_replay_executor_routing"
    );
    assert_eq!(final_execution_routing["replay_executor_ready"], true);
    assert_eq!(final_execution_routing["replay_executor_order_ready"], true);
    assert_eq!(
        final_execution_routing["approve_final_execution_enabled"],
        true
    );
    assert_eq!(final_execution_routing["handler_routing_ready"], true);
    assert_eq!(final_execution_routing["replay_executor_admitted"], true);
    assert_eq!(final_execution_routing["routing_admitted"], true);
    assert_eq!(final_execution_routing["executor_routing_required"], false);
    assert_eq!(
        final_execution_routing["would_route_through_executor"],
        true
    );
    assert_eq!(final_execution_routing["would_start_replay_executor"], true);
    assert_eq!(final_execution_routing["would_touch_disk"], true);
    assert_eq!(
        final_execution_routing["handler_phases"],
        final_execution_routing["expected_replay_order"]
    );
    assert!(final_execution_routing["handler_routes"]
        .as_array()
        .expect("final execution routing handler routes should be an array")
        .contains(&json!(
            "write_recovery_marker:approve_execution_recovery_marker_writer"
        )));
    assert!(final_execution_routing["handler_routes"]
        .as_array()
        .expect("final execution routing handler routes should be an array")
        .contains(&json!(
            "write_contract_source:contract_repair_approval_contract_source_writer"
        )));
    assert!(final_execution_routing["missing_handler_phases"]
        .as_array()
        .expect("final execution routing missing phases should be an array")
        .is_empty());
    assert!(final_execution_routing["unexpected_handler_phases"]
        .as_array()
        .expect("final execution routing unexpected phases should be an array")
        .is_empty());
    assert!(final_execution_routing["inherited_executor_blocked_gates"]
        .as_array()
        .expect("final execution routing inherited executor blockers should be an array")
        .is_empty());
    assert!(final_execution_routing["blocked_gates"]
        .as_array()
        .expect("final execution routing blockers should be an array")
        .is_empty());
    assert!(!final_execution_routing["blocked_gates"]
        .as_array()
        .expect("final execution routing blockers should be an array")
        .contains(&json!("handler_routing_ready")));
    let final_execution_handoff =
        &review_body["approve_execution_final_execution_routed_write_handoff_dry_run"];
    assert_eq!(
        final_execution_handoff["status"],
        "approve_execution_final_execution_routed_write_handoff_ready"
    );
    assert_eq!(
        final_execution_handoff["handoff_name"],
        "approve_execution_final_execution_routed_write_handoff"
    );
    assert_eq!(
        final_execution_handoff["approve_final_execution_enabled"],
        true
    );
    assert_eq!(
        final_execution_handoff["routed_write_handoff_enabled"],
        true
    );
    assert_eq!(
        final_execution_handoff["legacy_inline_final_writes_enabled"],
        false
    );
    assert_eq!(final_execution_handoff["replay_executor_ready"], true);
    assert_eq!(final_execution_handoff["replay_executor_order_ready"], true);
    assert_eq!(final_execution_handoff["handler_routing_ready"], true);
    assert_eq!(final_execution_handoff["routing_ready"], true);
    assert_eq!(
        final_execution_handoff["legacy_inline_final_writes_blocked"],
        true
    );
    assert_eq!(final_execution_handoff["routed_write_handoff_ready"], true);
    assert_eq!(
        final_execution_handoff["routed_write_handoff_admitted"],
        true
    );
    assert_eq!(
        final_execution_handoff["would_route_writes_through_executor"],
        true
    );
    assert_eq!(final_execution_handoff["would_write_recovery_marker"], true);
    assert_eq!(
        final_execution_handoff["would_persist_approval_record"],
        true
    );
    assert_eq!(final_execution_handoff["would_write_contract_source"], true);
    assert_eq!(
        final_execution_handoff["would_dispatch_route_success"],
        true
    );
    assert_eq!(final_execution_handoff["would_return_http_ok"], true);
    assert_eq!(final_execution_handoff["would_touch_disk"], true);
    assert_eq!(
        final_execution_handoff["handoff_phases"],
        final_execution_routing["expected_replay_order"]
    );
    assert!(final_execution_handoff["handler_routes"]
        .as_array()
        .expect("final execution handoff handler routes should be an array")
        .contains(&json!(
            "commit_transaction:approve_execution_transaction_commit_phase"
        )));
    assert!(final_execution_handoff["inline_final_write_gates"]
        .as_array()
        .expect("final execution handoff inline gates should be an array")
        .contains(&json!("contract_source_write_execution_enabled")));
    assert!(final_execution_handoff["inline_final_write_gates"]
        .as_array()
        .expect("final execution handoff inline gates should be an array")
        .contains(&json!("route_success_execution_enabled")));
    assert!(final_execution_handoff["missing_handoff_phases"]
        .as_array()
        .expect("final execution handoff missing phases should be an array")
        .is_empty());
    assert!(final_execution_handoff["unexpected_handoff_phases"]
        .as_array()
        .expect("final execution handoff unexpected phases should be an array")
        .is_empty());
    assert!(final_execution_handoff["inherited_routing_blocked_gates"]
        .as_array()
        .expect("final execution handoff inherited routing blockers should be an array")
        .is_empty());
    assert!(final_execution_handoff["blocked_gates"]
        .as_array()
        .expect("final execution handoff blockers should be an array")
        .is_empty());
    assert!(!final_execution_handoff["blocked_gates"]
        .as_array()
        .expect("final execution handoff blockers should be an array")
        .contains(&json!("routed_write_handoff_enabled")));
    let final_execution_handler_plan =
        &review_body["approve_execution_final_execution_routed_handler_plan_dry_run"];
    assert_eq!(
        final_execution_handler_plan["status"],
        "approve_execution_final_execution_routed_handler_plan_ready"
    );
    assert_eq!(
        final_execution_handler_plan["plan_name"],
        "approve_execution_final_execution_routed_handler_plan"
    );
    assert_eq!(
        final_execution_handler_plan["routed_write_handoff_ready"],
        true
    );
    assert_eq!(
        final_execution_handler_plan["routed_write_handoff_admitted"],
        true
    );
    assert_eq!(final_execution_handler_plan["routing_ready"], true);
    assert_eq!(
        final_execution_handler_plan["handler_execution_plan_ready"],
        true
    );
    assert_eq!(final_execution_handler_plan["handler_count"], 10);
    assert_eq!(final_execution_handler_plan["ready_handler_count"], 10);
    assert_eq!(final_execution_handler_plan["blocked_handler_count"], 0);
    assert_eq!(final_execution_handler_plan["would_execute_handlers"], true);
    assert_eq!(
        final_execution_handler_plan["would_write_recovery_marker"],
        true
    );
    assert_eq!(
        final_execution_handler_plan["would_transition_review"],
        true
    );
    assert_eq!(
        final_execution_handler_plan["would_write_contract_source"],
        true
    );
    assert_eq!(final_execution_handler_plan["would_return_http_ok"], true);
    assert_eq!(final_execution_handler_plan["would_touch_disk"], true);
    assert_eq!(
        final_execution_handler_plan["handler_phases"],
        final_execution_handoff["handoff_phases"]
    );
    assert!(final_execution_handler_plan["handler_readiness"]
        .as_array()
        .expect("final execution handler readiness should be an array")
        .contains(&json!("write_contract_source:ready")));
    assert!(final_execution_handler_plan["handler_readiness"]
        .as_array()
        .expect("final execution handler readiness should be an array")
        .contains(&json!("return_review_approve_executed:ready")));
    assert!(final_execution_handler_plan["ready_handlers"]
        .as_array()
        .expect("final execution ready handlers should be an array")
        .contains(&json!("clear_recovery_marker")));
    assert!(final_execution_handler_plan["blocked_handlers"]
        .as_array()
        .expect("final execution blocked handlers should be an array")
        .is_empty());
    assert!(
        final_execution_handler_plan["inherited_handoff_blocked_gates"]
            .as_array()
            .expect("final execution handler inherited handoff blockers should be an array")
            .is_empty()
    );
    assert!(final_execution_handler_plan["blocked_gates"]
        .as_array()
        .expect("final execution handler plan blockers should be an array")
        .is_empty());
    assert!(!final_execution_handler_plan["blocked_gates"]
        .as_array()
        .expect("final execution handler plan blockers should be an array")
        .contains(&json!("all_handlers_ready")));
    let final_execution_attempt =
        &review_body["approve_execution_final_execution_routed_execution_attempt_dry_run"];
    assert_eq!(
        final_execution_attempt["status"],
        "approve_execution_final_execution_routed_execution_attempt_ready"
    );
    assert_eq!(
        final_execution_attempt["attempt_name"],
        "approve_execution_final_execution_routed_execution_attempt"
    );
    assert_eq!(
        final_execution_attempt["handler_execution_plan_ready"],
        true
    );
    assert_eq!(
        final_execution_attempt["routed_write_handoff_admitted"],
        true
    );
    assert_eq!(final_execution_attempt["execution_attempt_ready"], true);
    assert_eq!(final_execution_attempt["execution_attempt_admitted"], true);
    assert_eq!(
        final_execution_attempt["execution_attempt_blocked_reason"],
        "none"
    );
    assert_eq!(final_execution_attempt["handler_count"], 10);
    assert_eq!(final_execution_attempt["ready_handler_count"], 10);
    assert_eq!(final_execution_attempt["blocked_handler_count"], 0);
    assert_eq!(final_execution_attempt["would_execute_handlers"], true);
    assert_eq!(final_execution_attempt["would_write_recovery_marker"], true);
    assert_eq!(final_execution_attempt["would_transition_review"], true);
    assert_eq!(
        final_execution_attempt["would_persist_approval_record"],
        true
    );
    assert_eq!(final_execution_attempt["would_write_contract_source"], true);
    assert_eq!(
        final_execution_attempt["would_dispatch_route_success"],
        true
    );
    assert_eq!(final_execution_attempt["would_return_http_ok"], true);
    assert_eq!(final_execution_attempt["would_touch_disk"], true);
    assert_eq!(
        final_execution_attempt["execution_order"],
        final_execution_handler_plan["handler_phases"]
    );
    assert!(final_execution_attempt["handler_readiness"]
        .as_array()
        .expect("final execution attempt handler readiness should be an array")
        .contains(&json!("commit_transaction:ready")));
    assert!(
        final_execution_attempt["inherited_handler_plan_blocked_gates"]
            .as_array()
            .expect("final execution attempt inherited handler blockers should be an array")
            .is_empty()
    );
    assert!(final_execution_attempt["blocked_gates"]
        .as_array()
        .expect("final execution attempt blockers should be an array")
        .is_empty());
    assert!(!final_execution_attempt["blocked_gates"]
        .as_array()
        .expect("final execution attempt blockers should be an array")
        .contains(&json!("handler_execution_plan_ready")));
    let ordered_handler_confirmation = &review_body
        ["approve_execution_final_execution_ordered_handler_execution_confirmation_dry_run"];
    assert_eq!(
        ordered_handler_confirmation["status"],
        "approve_execution_final_execution_ordered_handler_execution_confirmation_ready"
    );
    assert_eq!(
        ordered_handler_confirmation["confirmation_name"],
        "approve_execution_final_execution_ordered_handler_execution_confirmation"
    );
    assert_eq!(
        ordered_handler_confirmation["execution_attempt_admitted"],
        true
    );
    assert_eq!(ordered_handler_confirmation["execution_order_ready"], true);
    assert_eq!(
        ordered_handler_confirmation["handler_readiness_ready"],
        true
    );
    assert_eq!(ordered_handler_confirmation["handler_count"], 10);
    assert_eq!(ordered_handler_confirmation["ready_handler_count"], 10);
    assert_eq!(ordered_handler_confirmation["blocked_handler_count"], 0);
    assert_eq!(
        ordered_handler_confirmation["rollback_confirmation_ready"],
        true
    );
    assert_eq!(
        ordered_handler_confirmation["no_partial_write_guard_ready"],
        true
    );
    assert_eq!(
        ordered_handler_confirmation["ordered_handler_execution_ready"],
        true
    );
    assert_eq!(
        ordered_handler_confirmation["ordered_handler_execution_connection_preflight_ready"],
        true
    );
    assert_eq!(
        ordered_handler_confirmation["ordered_handler_execution_connection_enabled"],
        true
    );
    assert_eq!(
        ordered_handler_confirmation["ordered_handler_execution_connected"],
        true
    );
    assert_eq!(
        ordered_handler_confirmation["ordered_handler_execution_confirmed"],
        true
    );
    assert_eq!(
        ordered_handler_confirmation["ordered_handler_execution_dry_run_ready"],
        true
    );
    assert_eq!(
        ordered_handler_confirmation["ordered_handler_execution_dry_run_complete"],
        true
    );
    assert_eq!(
        ordered_handler_confirmation["dry_run_handler_execution_count"],
        10
    );
    assert_eq!(
        ordered_handler_confirmation["dry_run_handler_execution_effects_blocked"],
        false
    );
    assert_eq!(
        ordered_handler_confirmation["confirmation_blocked_reason"],
        "none"
    );
    assert_eq!(ordered_handler_confirmation["would_execute_handlers"], true);
    assert_eq!(
        ordered_handler_confirmation["would_write_recovery_marker"],
        true
    );
    assert_eq!(
        ordered_handler_confirmation["would_dispatch_route_success"],
        true
    );
    assert_eq!(ordered_handler_confirmation["would_return_http_ok"], true);
    assert_eq!(ordered_handler_confirmation["would_touch_disk"], true);
    assert!(ordered_handler_confirmation["execution_order"]
        .as_array()
        .expect("ordered handler confirmation execution order should be an array")
        .contains(&json!("return_review_approve_executed")));
    assert!(ordered_handler_confirmation["handler_readiness"]
        .as_array()
        .expect("ordered handler confirmation readiness should be an array")
        .contains(&json!("dispatch_route_success:ready")));
    assert!(
        ordered_handler_confirmation["dry_run_handler_execution_order"]
            .as_array()
            .expect("ordered handler dry-run execution order should be an array")
            .contains(&json!("return_review_approve_executed"))
    );
    assert!(
        ordered_handler_confirmation["dry_run_handler_execution_receipts"]
            .as_array()
            .expect("ordered handler dry-run execution receipts should be an array")
            .contains(&json!("return_review_approve_executed:dry_run_executed"))
    );
    assert!(ordered_handler_confirmation["unconfirmed_handlers"]
        .as_array()
        .expect("ordered handler confirmation unconfirmed handlers should be an array")
        .is_empty());
    assert!(
        ordered_handler_confirmation["inherited_attempt_blocked_gates"]
            .as_array()
            .expect("ordered handler confirmation inherited attempt blockers should be an array")
            .is_empty()
    );
    assert!(ordered_handler_confirmation["blocked_gates"]
        .as_array()
        .expect("ordered handler confirmation blockers should be an array")
        .is_empty());
    assert!(!ordered_handler_confirmation["blocked_gates"]
        .as_array()
        .expect("ordered handler confirmation blockers should be an array")
        .contains(&json!("execution_attempt_admitted")));
    let route_success_release =
        &review_body["approve_execution_final_execution_routed_route_success_release_dry_run"];
    assert_eq!(
        route_success_release["status"],
        "approve_execution_final_execution_routed_route_success_release_ready"
    );
    assert_eq!(
        route_success_release["release_name"],
        "approve_execution_final_execution_routed_route_success_release"
    );
    assert_eq!(route_success_release["execution_attempt_admitted"], true);
    assert_eq!(route_success_release["route_success_phase_ready"], true);
    assert_eq!(
        route_success_release["legacy_inline_final_writes_enabled"],
        false
    );
    assert_eq!(
        route_success_release["legacy_inline_final_writes_blocked"],
        true
    );
    assert_eq!(
        route_success_release["response_status_transition_ready"],
        true
    );
    assert_eq!(route_success_release["route_status_transition_ready"], true);
    assert_eq!(
        route_success_release["ordered_handler_execution_required"],
        true
    );
    assert_eq!(
        route_success_release["ordered_handler_execution_confirmed"],
        true
    );
    assert_eq!(
        route_success_release["response_status_connection_ready"],
        true
    );
    assert_eq!(
        route_success_release["response_status_connection_blocked_reason"],
        "none"
    );
    assert_eq!(
        route_success_release["routed_route_success_release_ready"],
        true
    );
    assert_eq!(
        route_success_release["routed_route_success_release_connected"],
        true
    );
    assert_eq!(
        route_success_release["routed_route_success_release_admitted"],
        true
    );
    assert_eq!(
        route_success_release["routed_route_success_release_application_ready"],
        true
    );
    assert_eq!(
        route_success_release["routed_route_success_release_application_enabled"],
        true
    );
    assert_eq!(
        route_success_release["routed_route_success_release_applied"],
        true
    );
    assert_eq!(
        route_success_release["response_status_application_blocked_reason"],
        "none"
    );
    assert_eq!(
        route_success_release["current_response_status"],
        "review_decision_execution_blocked"
    );
    assert_eq!(
        route_success_release["current_route_status"],
        "review_decision_execution_blocked"
    );
    assert_eq!(
        route_success_release["target_response_status"],
        "review_approve_executed"
    );
    assert_eq!(
        route_success_release["target_route_status"],
        "review_approve_executed"
    );
    assert_eq!(route_success_release["would_release_route_success"], true);
    assert_eq!(route_success_release["would_set_response_status"], true);
    assert_eq!(route_success_release["would_set_route_status"], true);
    assert_eq!(route_success_release["would_return_http_ok"], true);
    assert_eq!(route_success_release["would_touch_disk"], false);
    assert!(route_success_release["execution_order"]
        .as_array()
        .expect("route success release execution order should be an array")
        .contains(&json!("dispatch_route_success")));
    assert!(route_success_release["handler_readiness"]
        .as_array()
        .expect("route success release handler readiness should be an array")
        .contains(&json!("return_review_approve_executed:ready")));
    assert!(route_success_release["inherited_attempt_blocked_gates"]
        .as_array()
        .expect("route success release inherited attempt blockers should be an array")
        .is_empty());
    assert!(
        route_success_release["inherited_ordered_handler_execution_blocked_gates"]
            .as_array()
            .expect("route success release inherited ordered-handler blockers should be an array")
            .is_empty()
    );
    assert!(route_success_release["blocked_gates"]
        .as_array()
        .expect("route success release blockers should be an array")
        .is_empty());
    assert!(!route_success_release["blocked_gates"]
        .as_array()
        .expect("route success release blockers should be an array")
        .contains(&json!("ordered_handler_execution_confirmed")));
    assert!(!route_success_release["blocked_gates"]
        .as_array()
        .expect("route success release blockers should be an array")
        .contains(&json!("execution_attempt_admitted")));
    let durable_writeback_bundle =
        &review_body["approve_execution_final_execution_durable_writeback_bundle_dry_run"];
    assert_eq!(
        durable_writeback_bundle["status"],
        "approve_execution_final_execution_durable_writeback_bundle_ready"
    );
    assert_eq!(
        durable_writeback_bundle["bundle_name"],
        "approve_execution_final_execution_durable_writeback_bundle"
    );
    assert_eq!(
        durable_writeback_bundle["response_application_success"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["approval_record_write_ready"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["approval_record_persistence_enabled"],
        false
    );
    assert_eq!(
        durable_writeback_bundle["contract_source_write_ready"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["contract_source_write_enabled"],
        false
    );
    assert_eq!(
        durable_writeback_bundle["recovery_marker_persistence_ready"],
        true
    );
    assert_eq!(durable_writeback_bundle["transaction_commit_ready"], true);
    assert_eq!(
        durable_writeback_bundle["recovery_marker_cleanup_ready"],
        true
    );
    assert_eq!(durable_writeback_bundle["rollback_ready"], true);
    assert_eq!(
        durable_writeback_bundle["no_partial_write_guard_ready"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["durable_writeback_bundle_ready"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["durable_writeback_bundle_enabled"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["durable_writeback_bundle_admitted"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["durable_writeback_bundle_execution_enabled"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["durable_writeback_bundle_execution_admitted"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["durable_writeback_bundle_disk_application_enabled"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["durable_writeback_bundle_disk_application_admitted"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["durable_writeback_bundle_execution_preflight_ready"],
        true
    );
    assert_eq!(durable_writeback_bundle["dry_run_execution_ready"], true);
    assert_eq!(durable_writeback_bundle["dry_run_execution_complete"], true);
    assert_eq!(durable_writeback_bundle["dry_run_execution_count"], 8);
    assert_eq!(
        durable_writeback_bundle["dry_run_execution_effects_blocked"],
        false
    );
    assert_eq!(durable_writeback_bundle["rollback_dry_run_ready"], true);
    assert_eq!(durable_writeback_bundle["rollback_dry_run_complete"], true);
    assert_eq!(durable_writeback_bundle["rollback_dry_run_count"], 3);
    assert_eq!(
        durable_writeback_bundle["rollback_dry_run_effects_blocked"],
        false
    );
    assert_eq!(durable_writeback_bundle["rollback_coverage_ready"], true);
    assert_eq!(
        durable_writeback_bundle["durable_execution_rollback_barrier_ready"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_plan_ready"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_plan_complete"],
        true
    );
    assert_eq!(durable_writeback_bundle["disk_application_effect_count"], 5);
    assert_eq!(
        durable_writeback_bundle["disk_application_effects_blocked"],
        false
    );
    assert_eq!(
        durable_writeback_bundle["approval_record_disk_write_planned"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["contract_source_disk_write_planned"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["recovery_marker_disk_write_planned"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["recovery_marker_cleanup_planned"],
        true
    );
    assert_eq!(durable_writeback_bundle["transaction_commit_planned"], true);
    let approval_record_file_name = durable_writeback_bundle["approval_record_file_name"]
        .as_str()
        .expect("approval record file name should be present");
    assert!(approval_record_file_name.ends_with(".json"));
    let planned_recovery_marker_file_name = durable_writeback_bundle["recovery_marker_file_name"]
        .as_str()
        .expect("planned recovery marker file name should be present");
    assert!(planned_recovery_marker_file_name.ends_with(".json"));
    let contract_source_digest_before = durable_writeback_bundle["contract_source_digest_before"]
        .as_str()
        .expect("contract source digest before should be present");
    let contract_source_digest_after = durable_writeback_bundle["contract_source_digest_after"]
        .as_str()
        .expect("contract source digest after should be present");
    assert!(contract_source_digest_before.starts_with("sha256:"));
    assert!(contract_source_digest_after.starts_with("sha256:"));
    assert_ne!(contract_source_digest_before, contract_source_digest_after);
    assert_eq!(
        durable_writeback_bundle["disk_application_transaction_proof_ready"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_atomic_write_set_ready"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_all_or_nothing_guard_ready"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_failure_probe_coverage_ready"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_rollback_action_coverage_ready"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_cleanup_recovery_action_ready"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_commit_terminal_verification_ready"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_terminal_recovery_proof_ready"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_partial_state_proof_ready"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_transaction_failure_proof_ready"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_executor_name"],
        "approve_execution_durable_disk_application_executor"
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_executor_ready"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_executor_admitted"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_executor_effects_blocked"],
        false
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_executor_blocked_reason"],
        "none"
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_endpoint_helper_name"],
        "contract_repair_approval_execute_durable_disk_application_handlers"
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_endpoint_helper_wired"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_endpoint_helper_admission_ready"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_endpoint_helper_admitted"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_endpoint_helper_execution_connected"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_endpoint_helper_execution_blocked"],
        false
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_endpoint_helper_would_execute"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_endpoint_helper_blocked_reason"],
        "none"
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_endpoint_helper_execution_blocked_reason"],
        "none"
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_executor_handler_count"],
        5
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_executor_ready_handler_count"],
        5
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_executor_blocked_handler_count"],
        0
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_atomic_write_set_count"],
        5
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_simulated_failure_point_count"],
        5
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_rollback_action_count"],
        5
    );
    assert_eq!(
        durable_writeback_bundle["disk_application_partial_enabled_after_failure_count"],
        0
    );
    assert_eq!(durable_writeback_bundle["bundle_blocked_reason"], "none");
    assert_eq!(
        durable_writeback_bundle["would_persist_approval_record"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["would_write_contract_source"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["would_persist_recovery_marker"],
        true
    );
    assert_eq!(
        durable_writeback_bundle["would_clear_recovery_marker"],
        true
    );
    assert_eq!(durable_writeback_bundle["would_commit_transaction"], true);
    assert_eq!(durable_writeback_bundle["would_touch_disk"], true);
    assert!(durable_writeback_bundle["execution_order"]
        .as_array()
        .expect("durable writeback bundle execution order should be an array")
        .contains(&json!("return_review_approve_executed")));
    assert!(durable_writeback_bundle["rollback_order"]
        .as_array()
        .expect("durable writeback bundle rollback order should be an array")
        .contains(&json!("restore_contract_source")));
    assert!(durable_writeback_bundle["dry_run_durable_execution_order"]
        .as_array()
        .expect("durable writeback bundle dry-run execution order should be an array")
        .contains(&json!("write_contract_source")));
    assert!(!durable_writeback_bundle["dry_run_durable_execution_order"]
        .as_array()
        .expect("durable writeback bundle dry-run execution order should be an array")
        .contains(&json!("return_review_approve_executed")));
    assert!(
        durable_writeback_bundle["dry_run_durable_execution_receipts"]
            .as_array()
            .expect("durable writeback bundle dry-run receipts should be an array")
            .contains(&json!("commit_transaction:dry_run_executed"))
    );
    assert!(durable_writeback_bundle["dry_run_rollback_order"]
        .as_array()
        .expect("durable writeback bundle dry-run rollback order should be an array")
        .contains(&json!("restore_approval_record")));
    assert!(durable_writeback_bundle["dry_run_rollback_receipts"]
        .as_array()
        .expect("durable writeback bundle rollback receipts should be an array")
        .contains(&json!(
            "mark_recovery_marker_rolled_back:dry_run_rollback_ready"
        )));
    assert!(durable_writeback_bundle["rollback_coverage_pairs"]
        .as_array()
        .expect("durable writeback bundle rollback coverage pairs should be an array")
        .contains(&json!("write_contract_source->restore_contract_source")));
    assert!(durable_writeback_bundle["uncovered_durable_phases"]
        .as_array()
        .expect("durable writeback bundle uncovered phases should be an array")
        .is_empty());
    assert!(durable_writeback_bundle["disk_application_plan_receipts"]
        .as_array()
        .expect("durable writeback bundle disk application plan receipts should be an array")
        .contains(&json!("write_contract_source:planned")));
    assert!(durable_writeback_bundle["disk_application_plan_receipts"]
        .as_array()
        .expect("durable writeback bundle disk application plan receipts should be an array")
        .contains(&json!("commit_transaction:planned")));
    assert!(
        durable_writeback_bundle["disk_application_atomic_write_set"]
            .as_array()
            .expect("durable writeback bundle disk application atomic write set should be an array")
            .contains(&json!("clear_recovery_marker"))
    );
    assert!(
        durable_writeback_bundle["disk_application_atomic_write_set"]
            .as_array()
            .expect("durable writeback bundle disk application atomic write set should be an array")
            .contains(&json!("commit_transaction"))
    );
    assert!(durable_writeback_bundle["disk_application_simulated_failure_points"]
        .as_array()
        .expect("durable writeback bundle disk application simulated failure points should be an array")
        .contains(&json!("after:commit_transaction")));
    assert!(
        durable_writeback_bundle["disk_application_rollback_actions"]
            .as_array()
            .expect("durable writeback bundle disk application rollback actions should be an array")
            .contains(&json!("restore_contract_source"))
    );
    assert!(
        durable_writeback_bundle["disk_application_rollback_actions"]
            .as_array()
            .expect("durable writeback bundle disk application rollback actions should be an array")
            .contains(&json!("restore_recovery_marker"))
    );
    assert!(
        durable_writeback_bundle["disk_application_rollback_actions"]
            .as_array()
            .expect("durable writeback bundle disk application rollback actions should be an array")
            .contains(&json!("verify_committed_transaction"))
    );
    assert!(durable_writeback_bundle["disk_application_executor_handler_routes"]
        .as_array()
        .expect("durable writeback bundle disk application executor handler routes should be an array")
        .contains(&json!(
            "write_contract_source->contract_repair_approval_contract_source_write_with_gate"
        )));
    assert!(durable_writeback_bundle["disk_application_executor_handler_routes"]
        .as_array()
        .expect("durable writeback bundle disk application executor handler routes should be an array")
        .contains(&json!(
            "commit_transaction->contract_repair_approval_approve_execution_runner_transaction_commit_phase_with_gate"
        )));
    assert!(durable_writeback_bundle["disk_application_executor_ready_handlers"]
        .as_array()
        .expect("durable writeback bundle disk application executor ready handlers should be an array")
        .contains(&json!("commit_transaction")));
    assert!(durable_writeback_bundle["disk_application_executor_blocked_handlers"]
        .as_array()
        .expect("durable writeback bundle disk application executor blocked handlers should be an array")
        .is_empty());
    assert!(
        durable_writeback_bundle["disk_application_endpoint_helper_required_inputs"]
            .as_array()
            .expect("durable writeback bundle endpoint helper required inputs should be an array")
            .contains(&json!("contract_patch_apply"))
    );
    assert!(
        durable_writeback_bundle["disk_application_endpoint_helper_ready_inputs"]
            .as_array()
            .expect("durable writeback bundle endpoint helper ready inputs should be an array")
            .contains(&json!("transaction_commit"))
    );
    assert!(
        durable_writeback_bundle["disk_application_endpoint_helper_blocked_inputs"]
            .as_array()
            .expect("durable writeback bundle endpoint helper blocked inputs should be an array")
            .is_empty()
    );
    assert!(!durable_writeback_bundle["blocked_gates"]
        .as_array()
        .expect("durable writeback bundle blockers should be an array")
        .contains(&json!("disk_application_transaction_failure_proof_ready")));
    assert!(!durable_writeback_bundle["blocked_gates"]
        .as_array()
        .expect("durable writeback bundle blockers should be an array")
        .contains(&json!("disk_application_executor_ready")));
    assert!(!durable_writeback_bundle["blocked_gates"]
        .as_array()
        .expect("durable writeback bundle blockers should be an array")
        .contains(&json!("disk_application_endpoint_helper_admission_ready")));
    assert!(!durable_writeback_bundle["blocked_gates"]
        .as_array()
        .expect("durable writeback bundle blockers should be an array")
        .contains(&json!(
            "disk_application_endpoint_helper_execution_connected"
        )));
    assert!(!durable_writeback_bundle["blocked_gates"]
        .as_array()
        .expect("durable writeback bundle blockers should be an array")
        .contains(&json!(
            "approve_final_execution_durable_writeback_bundle_disk_application_enabled"
        )));
    let durable_disk_application_execution =
        &review_body["approve_execution_durable_disk_application_execution"];
    assert_eq!(
        durable_disk_application_execution["status"],
        "approve_execution_durable_disk_application_executor_committed"
    );
    assert_eq!(
        durable_disk_application_execution["executor_name"],
        "approve_execution_durable_disk_application_executor"
    );
    assert_eq!(
        durable_disk_application_execution["endpoint_helper_execution_connected"],
        true
    );
    assert_eq!(
        durable_disk_application_execution["endpoint_helper_would_execute"],
        true
    );
    assert_eq!(
        durable_disk_application_execution["executor_admitted"],
        true
    );
    assert_eq!(
        durable_disk_application_execution["recovery_marker_written"],
        true
    );
    assert_eq!(
        durable_disk_application_execution["approval_record_persisted"],
        true
    );
    assert_eq!(
        durable_disk_application_execution["contract_source_written"],
        true
    );
    assert_eq!(
        durable_disk_application_execution["recovery_marker_cleared"],
        true
    );
    assert_eq!(
        durable_disk_application_execution["transaction_committed"],
        true
    );
    assert_eq!(durable_disk_application_execution["would_touch_disk"], true);
    assert!(durable_disk_application_execution["blocked_by"]
        .as_array()
        .expect("durable disk application execution blockers should be an array")
        .is_empty());
    assert!(durable_disk_application_execution["execution_receipts"]
        .as_array()
        .expect("durable disk application execution receipts should be an array")
        .contains(&json!("commit_transaction:executed")));
    assert!(!durable_writeback_bundle["blocked_gates"]
        .as_array()
        .expect("durable writeback bundle blockers should be an array")
        .contains(&json!(
            "approve_final_execution_durable_writeback_bundle_execution_enabled"
        )));
    assert!(!durable_writeback_bundle["blocked_gates"]
        .as_array()
        .expect("durable writeback bundle blockers should be an array")
        .contains(&json!(
            "approve_final_execution_durable_writeback_bundle_enabled"
        )));
    assert!(!durable_writeback_bundle["blocked_gates"]
        .as_array()
        .expect("durable writeback bundle blockers should be an array")
        .contains(&json!("response_application_success")));
    assert!(!durable_writeback_bundle["blocked_gates"]
        .as_array()
        .expect("durable writeback bundle blockers should be an array")
        .contains(&json!("durable_execution_rollback_barrier_ready")));
    assert!(!durable_writeback_bundle["blocked_gates"]
        .as_array()
        .expect("durable writeback bundle blockers should be an array")
        .contains(&json!("disk_application_plan_ready")));
    let recovery_marker_file_name = review_body["approve_execution_recovery_marker_write_dry_run"]
        ["file_name"]
        .as_str()
        .expect("recovery marker file name should be present");
    assert!(
        !contract_repair_store_dir
            .join(recovery_marker_file_name)
            .exists(),
        "durable disk application should clear the recovery marker after commit"
    );
    let approval_record_path = contract_repair_store_dir.join(approval_record_file_name);
    assert!(
        approval_record_path.exists(),
        "durable disk application should persist the approval record"
    );
    assert_eq!(
        review_body["approve_execution_decision_lock_summary_dry_run"]["expected_http_status"],
        200
    );
    assert_eq!(
        review_body["approve_execution_decision_lock_summary_dry_run"]
            ["final_execution_entry_ready"],
        true
    );
    assert_eq!(
        review_body["approve_execution_decision_lock_summary_dry_run"]
            ["approve_final_execution_enabled"],
        true
    );
    assert_eq!(
        review_body["approve_execution_decision_lock_summary_dry_run"]
            ["final_execution_switch_enabled"],
        true
    );
    assert_eq!(
        review_body["approve_execution_decision_lock_summary_dry_run"]["final_execution_locked"],
        false
    );
    assert_eq!(
        review_body["approve_execution_decision_lock_summary_dry_run"]["primary_blocked_reason"],
        "none"
    );
    assert_eq!(
        review_body["approve_execution_decision_lock_summary_dry_run"]
            ["routed_route_success_release_applied"],
        true
    );
    assert_eq!(
        review_body["approve_execution_decision_lock_summary_dry_run"]
            ["final_response_application_success"],
        true
    );
    assert_eq!(
        review_body["approve_execution_decision_lock_summary_dry_run"]["would_return_http_ok"],
        true
    );
    assert_eq!(
        review_body["approve_execution_decision_lock_summary_dry_run"]["would_touch_disk"],
        true
    );
    assert!(
        review_body["approve_execution_decision_lock_summary_dry_run"]
            ["inherited_final_execution_entry_blocked_gates"]
            .as_array()
            .expect("decision lock inherited final-entry blockers should be an array")
            .is_empty()
    );
    assert_eq!(
        review_body["approve_execution_runner_route_success_readiness_dry_run"]
            ["would_return_success"],
        false
    );

    let source_after =
        std::fs::read_to_string(dirs.graph_store_dir.join("graph-dual-ma-live.json"))
            .expect("contract source fixture should remain readable");
    let source_after_json: Value =
        serde_json::from_str(&source_after).expect("contract source should remain valid json");
    assert_eq!(
        source_after_json["machines"][0]["memory"],
        json!([{"name": "last_signal_at", "type_name": "time?"}]),
        "durable disk application should apply the approved memory field patch"
    );
}

#[cfg(windows)]
#[tokio::test(flavor = "multi_thread")]
async fn contract_repair_approval_approve_live_route_rolls_back_when_source_write_is_file_locked() {
    use std::os::windows::fs::OpenOptionsExt;

    const FILE_SHARE_READ: u32 = 0x00000001;

    let (app, dirs) =
        common::test_app_with_dirs("api_v4_productization_contract_repair_approve_live_lock");
    let grants_path = dirs
        .backtest_store_dir
        .parent()
        .expect("test backtest store should have a storage parent")
        .join("contract-repair-reviewer-grants.json");
    std::fs::write(
        &grants_path,
        r#"{"policy_version":"quantpilot/contract-repair-reviewer-role-policy/v1","grants":[{"subject":"user:0","role":"contract_repair_reviewer"}]}"#,
    )
    .expect("reviewer grant file should be writable");
    let contract_repair_store_dir = dirs
        .backtest_store_dir
        .parent()
        .expect("test backtest store should have a storage parent")
        .join("contract-repair-approvals");
    std::fs::create_dir_all(&contract_repair_store_dir)
        .expect("contract repair approval store should be ready");
    write_contract_source_fixture(
        &dirs,
        "graph-dual-ma-live-locked",
        "v4-live-test",
        "sha256:test-contract-live-locked",
    );
    let source_path = dirs.graph_store_dir.join("graph-dual-ma-live-locked.json");
    let source_lock = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(&source_path)
        .expect("source fixture should be lockable for read-only sharing");

    let (post_status, post_body) = post_contract_repair_request_with_app(
        app.clone(),
        json!({
            "status": "body_preview_only",
            "payload_kind": "v4_contract_repair_approval_request",
            "request_id": "approval-request:repair-draft:memory_schema/decision.dual_ma/live-route-locked/last_signal_at",
            "target_path": "memory_schema/decision.dual_ma/live-route-locked/last_signal_at",
            "target_kind": "memory_field",
            "changed_fields": ["type_name"],
            "patch_payload": {"type_name": "time?"},
            "contract_source_ref": {
                "source_kind": "v4_machine_graph_contract",
                "source_id": "graph-dual-ma-live-locked",
                "version": "v4-live-test",
                "artifact_digest": "sha256:test-contract-live-locked"
            },
            "mutation_enabled": false,
            "review_required": true
        }),
    )
    .await;
    let approval_id = post_body["approval_record_preview"]["approval_id"]
        .as_str()
        .expect("approval id should be present")
        .to_string();

    assert_eq!(post_status, StatusCode::LOCKED, "{post_body}");

    let (review_status, review_body) =
        post_contract_repair_path_with_app_with_approve_live_route_env(
            app.clone(),
            &format!(
            "/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}/review"
        ),
            json!({
                "action": "approve",
                "reviewer_id": "user:0",
                "reason": "approve with locked source fixture",
                "review_enabled": true
            }),
            "verified-live-approve-route",
        )
        .await;
    drop(source_lock);

    assert_eq!(review_status, StatusCode::LOCKED, "{review_body}");
    assert_eq!(
        review_body["status"],
        "review_approve_durable_disk_application_blocked"
    );
    assert_eq!(
        review_body["route_status"],
        "review_approve_durable_disk_application_blocked"
    );
    assert_eq!(review_body["review_enabled"], false);
    assert_eq!(
        review_body["approval_record_preview"]["review_state"],
        "pending"
    );
    assert_eq!(
        review_body["approve_execution_decision_lock_summary_dry_run"]["final_execution_locked"],
        true
    );
    assert!(
        review_body["approve_execution_decision_lock_summary_dry_run"]["inherited_blocked_reasons"]
            .as_array()
            .expect("decision lock inherited blockers should be an array")
            .contains(&json!("durable_disk_application_committed"))
    );

    let durable_disk_application_execution =
        &review_body["approve_execution_durable_disk_application_execution"];
    assert_eq!(
        durable_disk_application_execution["status"],
        "approve_execution_durable_disk_application_executor_write_contract_source_blocked"
    );
    assert_eq!(
        durable_disk_application_execution["endpoint_helper_would_execute"],
        true
    );
    assert_eq!(
        durable_disk_application_execution["recovery_marker_written"],
        true
    );
    assert_eq!(
        durable_disk_application_execution["approval_record_persisted"],
        true
    );
    assert_eq!(
        durable_disk_application_execution["contract_source_written"],
        false
    );
    assert_eq!(
        durable_disk_application_execution["transaction_committed"],
        false
    );
    assert_eq!(
        durable_disk_application_execution["rollback_executed"],
        true
    );
    assert!(durable_disk_application_execution["blocked_by"]
        .as_array()
        .expect("durable disk application blockers should be an array")
        .contains(&json!("contract_source_atomic_write")));
    assert!(durable_disk_application_execution["rollback_receipts"]
        .as_array()
        .expect("durable disk application rollback receipts should be an array")
        .contains(&json!("restore_approval_record:executed")));
    assert!(durable_disk_application_execution["rollback_receipts"]
        .as_array()
        .expect("durable disk application rollback receipts should be an array")
        .contains(&json!("mark_recovery_marker_rolled_back:executed")));

    let recovery_marker_file_name = review_body["approve_execution_recovery_marker_write_dry_run"]
        ["file_name"]
        .as_str()
        .expect("recovery marker file name should be present");
    assert!(
        !contract_repair_store_dir
            .join(recovery_marker_file_name)
            .exists(),
        "failed durable disk application should remove the recovery marker"
    );

    let detail_status = get_contract_repair_path_with_app(
        app,
        &format!("/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}"),
    )
    .await
    .1;
    assert_eq!(detail_status["record"]["review_state"], "pending");

    let source_after = std::fs::read_to_string(&source_path)
        .expect("locked source fixture should remain readable after rollback");
    let source_after_json: Value =
        serde_json::from_str(&source_after).expect("source should remain valid json");
    assert_eq!(
        source_after_json["machines"][0]["memory"],
        json!([]),
        "failed durable disk application should leave source unchanged"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn contract_repair_approval_authorized_reject_executes_without_contract_mutation() {
    let (app, dirs) =
        common::test_app_with_dirs("api_v4_productization_contract_repair_reject_execute");
    let grants_path = dirs
        .backtest_store_dir
        .parent()
        .expect("test backtest store should have a storage parent")
        .join("contract-repair-reviewer-grants.json");
    std::fs::write(
        &grants_path,
        r#"{"policy_version":"quantpilot/contract-repair-reviewer-role-policy/v1","grants":[{"subject":"user:0","role":"contract_repair_reviewer"}]}"#,
    )
    .expect("reviewer grant file should be writable");
    let (post_status, post_body) = post_contract_repair_request_with_app(
        app.clone(),
        json!({
            "status": "body_preview_only",
            "payload_kind": "v4_contract_repair_approval_request",
            "request_id": "approval-request:repair-draft:memory_schema/decision.dual_ma/reject-execute",
            "target_path": "memory_schema/decision.dual_ma/reject-execute",
            "target_kind": "memory_field",
            "changed_fields": ["type_name"],
            "mutation_enabled": false,
            "review_required": true
        }),
    )
    .await;
    let approval_id = post_body["approval_record_preview"]["approval_id"]
        .as_str()
        .expect("approval id should be present")
        .to_string();

    assert_eq!(post_status, StatusCode::LOCKED, "{post_body}");

    let (review_status, review_body) = post_contract_repair_path_with_app(
        app.clone(),
        &format!(
            "/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}/review"
        ),
        json!({
            "action": "reject",
            "reviewer_id": "user:0",
            "reason": "reject with explicit reviewer grant",
            "review_enabled": true
        }),
    )
    .await;

    assert_eq!(review_status, StatusCode::OK, "{review_body}");
    assert_eq!(review_body["status"], "review_reject_executed");
    assert_eq!(review_body["route_status"], "review_reject_executed");
    assert_eq!(review_body["review_enabled"], true);
    assert_eq!(review_body["persistence_enabled"], true);
    assert_eq!(review_body["mutation_enabled"], false);
    assert_eq!(review_body["execution_gate"]["status"], "ready");
    assert!(review_body["execution_gate"]["blocked_gates"]
        .as_array()
        .expect("blocked gates should be an array")
        .is_empty());
    assert_eq!(
        review_body["execution_plan_preview"]["execution_enabled"],
        true
    );
    assert_eq!(
        review_body["execution_plan_preview"]["target_review_state"],
        "rejected"
    );
    assert_eq!(
        review_body["execution_plan_preview"]["would_mutate_contract"],
        false
    );
    assert_eq!(
        review_body["execution_plan_preview"]["would_emit_lifecycle_event"],
        true
    );
    assert_eq!(
        review_body["approval_record_preview"]["review_state"],
        "rejected"
    );
    assert_eq!(
        review_body["approval_record_preview"]["transient_review_status"],
        "reject_executed"
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["status"],
        "transition_executed"
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["target_review_state"],
        "rejected"
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["would_transition"],
        true
    );
    assert_eq!(
        review_body["record_write_dry_run"]["status"],
        "record_write_executed"
    );
    assert_eq!(review_body["record_write_dry_run"]["would_write"], true);
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["status"],
        "lifecycle_emitted"
    );
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["event_id"],
        format!("contract-repair-review-reject:{approval_id}")
    );
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["event_payload_ready"],
        true
    );
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["emission_ready"],
        true
    );
    assert_eq!(review_body["lifecycle_event_dry_run"]["would_emit"], true);
    assert!(review_body["lifecycle_event_dry_run"]["blocked_by"]
        .as_array()
        .expect("lifecycle blockers should be an array")
        .is_empty());
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["status"],
        "lifecycle_entry_appended"
    );
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["event_id"],
        format!("contract-repair-review-reject:{approval_id}")
    );
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["review_state"],
        "rejected"
    );
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["sequence_no"],
        1
    );
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["would_append"],
        true
    );
    assert!(review_body["lifecycle_entry_append_dry_run"]["blocked_by"]
        .as_array()
        .expect("lifecycle append blockers should be an array")
        .is_empty());
    assert_eq!(
        review_body["contract_writeback_dry_run"]["eligible_after_approval"],
        false
    );
    assert_eq!(
        review_body["contract_writeback_dry_run"]["would_mutate_contract"],
        false
    );

    let (detail_status, detail_body) = get_contract_repair_path_with_app(
        app,
        &format!("/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}"),
    )
    .await;

    assert_eq!(detail_status, StatusCode::OK, "{detail_body}");
    assert_eq!(detail_body["record"]["review_state"], "rejected");
    assert_eq!(
        detail_body["record"]["transient_review_status"],
        "reject_executed"
    );
    assert_eq!(detail_body["record"]["transient_review_action"], "reject");
    assert_eq!(
        detail_body["record"]["lifecycle"][0]["event_id"],
        format!("contract-repair-review-reject:{approval_id}")
    );
    assert_eq!(
        detail_body["record"]["lifecycle"][0]["review_state"],
        "rejected"
    );
    assert_eq!(detail_body["record"]["lifecycle"][0]["sequence_no"], 1);
    assert_eq!(detail_body["mutation_enabled"], false);

    let restarted_app = common::test_app_from_dirs(dirs);
    let (restarted_detail_status, restarted_detail_body) = get_contract_repair_path_with_app(
        restarted_app,
        &format!("/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}"),
    )
    .await;

    assert_eq!(
        restarted_detail_status,
        StatusCode::OK,
        "{restarted_detail_body}"
    );
    assert_eq!(restarted_detail_body["record"]["review_state"], "rejected");
    assert_eq!(
        restarted_detail_body["record"]["transient_review_status"],
        "reject_executed"
    );
    assert_eq!(
        restarted_detail_body["record"]["lifecycle"][0]["event_id"],
        format!("contract-repair-review-reject:{approval_id}")
    );
    assert_eq!(
        restarted_detail_body["record"]["lifecycle"][0]["review_state"],
        "rejected"
    );
    assert_eq!(
        restarted_detail_body["record"]["lifecycle"][0]["sequence_no"],
        1
    );
    assert_eq!(restarted_detail_body["mutation_enabled"], false);
}

#[tokio::test(flavor = "multi_thread")]
async fn contract_repair_approval_authorized_claim_executes_without_contract_mutation() {
    let (app, dirs) =
        common::test_app_with_dirs("api_v4_productization_contract_repair_claim_execute");
    let grants_path = dirs
        .backtest_store_dir
        .parent()
        .expect("test backtest store should have a storage parent")
        .join("contract-repair-reviewer-grants.json");
    std::fs::write(
        &grants_path,
        r#"{"policy_version":"quantpilot/contract-repair-reviewer-role-policy/v1","grants":[{"subject":"user:0","role":"contract_repair_reviewer"}]}"#,
    )
    .expect("reviewer grant file should be writable");
    let (post_status, post_body) = post_contract_repair_request_with_app(
        app.clone(),
        json!({
            "status": "body_preview_only",
            "payload_kind": "v4_contract_repair_approval_request",
            "request_id": "approval-request:repair-draft:memory_schema/decision.dual_ma/claim-execute",
            "target_path": "memory_schema/decision.dual_ma/claim-execute",
            "target_kind": "memory_field",
            "changed_fields": ["type_name"],
            "mutation_enabled": false,
            "review_required": true
        }),
    )
    .await;
    let approval_id = post_body["approval_record_preview"]["approval_id"]
        .as_str()
        .expect("approval id should be present")
        .to_string();

    assert_eq!(post_status, StatusCode::LOCKED, "{post_body}");

    let (review_status, review_body) = post_contract_repair_path_with_app(
        app.clone(),
        &format!(
            "/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}/review"
        ),
        json!({
            "action": "claim",
            "reviewer_id": "user:0",
            "reason": "claim with explicit reviewer grant",
            "review_enabled": true
        }),
    )
    .await;

    assert_eq!(review_status, StatusCode::OK, "{review_body}");
    assert_eq!(review_body["status"], "review_claim_executed");
    assert_eq!(review_body["route_status"], "review_claim_executed");
    assert_eq!(review_body["review_enabled"], true);
    assert_eq!(review_body["persistence_enabled"], true);
    assert_eq!(review_body["mutation_enabled"], false);
    assert_eq!(review_body["execution_gate"]["status"], "ready");
    assert!(review_body["execution_gate"]["blocked_gates"]
        .as_array()
        .expect("blocked gates should be an array")
        .is_empty());
    assert_eq!(
        review_body["execution_plan_preview"]["status"],
        "execution_plan_ready"
    );
    assert_eq!(
        review_body["execution_plan_preview"]["execution_enabled"],
        true
    );
    assert_eq!(
        review_body["execution_plan_preview"]["would_persist_approval_record"],
        true
    );
    assert_eq!(
        review_body["execution_plan_preview"]["would_mutate_contract"],
        false
    );
    assert_eq!(
        review_body["execution_plan_preview"]["would_emit_lifecycle_event"],
        true
    );
    assert_eq!(
        review_body["approval_record_preview"]["review_state"],
        "under_review"
    );
    assert_eq!(
        review_body["approval_record_preview"]["transient_review_status"],
        "claim_executed"
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["status"],
        "transition_executed"
    );
    assert_eq!(
        review_body["review_transition_dry_run"]["would_transition"],
        true
    );
    assert_eq!(
        review_body["record_write_dry_run"]["status"],
        "record_write_executed"
    );
    assert_eq!(review_body["record_write_dry_run"]["write_ready"], true);
    assert_eq!(review_body["record_write_dry_run"]["would_write"], true);
    assert!(review_body["record_write_dry_run"]["blocked_by"]
        .as_array()
        .expect("record write blockers should be an array")
        .is_empty());
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["status"],
        "lifecycle_emitted"
    );
    assert_eq!(
        review_body["lifecycle_event_dry_run"]["event_id"],
        format!("contract-repair-review-claim:{approval_id}")
    );
    assert_eq!(review_body["lifecycle_event_dry_run"]["would_emit"], true);
    assert!(review_body["lifecycle_event_dry_run"]["blocked_by"]
        .as_array()
        .expect("lifecycle blockers should be an array")
        .is_empty());
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["status"],
        "lifecycle_entry_appended"
    );
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["event_id"],
        format!("contract-repair-review-claim:{approval_id}")
    );
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["review_state"],
        "under_review"
    );
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["sequence_no"],
        1
    );
    assert_eq!(
        review_body["lifecycle_entry_append_dry_run"]["would_append"],
        true
    );
    assert!(review_body["lifecycle_entry_append_dry_run"]["blocked_by"]
        .as_array()
        .expect("lifecycle append blockers should be an array")
        .is_empty());
    assert_eq!(
        review_body["contract_writeback_dry_run"]["would_mutate_contract"],
        false
    );

    let (detail_status, detail_body) = get_contract_repair_path_with_app(
        app,
        &format!("/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}"),
    )
    .await;

    assert_eq!(detail_status, StatusCode::OK, "{detail_body}");
    assert_eq!(detail_body["record"]["review_state"], "under_review");
    assert_eq!(
        detail_body["record"]["transient_review_status"],
        "claim_executed"
    );
    assert_eq!(
        detail_body["record"]["lifecycle"][0]["event_id"],
        format!("contract-repair-review-claim:{approval_id}")
    );
    assert_eq!(
        detail_body["record"]["lifecycle"][0]["review_state"],
        "under_review"
    );
    assert_eq!(detail_body["record"]["lifecycle"][0]["sequence_no"], 1);
    assert_eq!(detail_body["mutation_enabled"], false);

    let restarted_app = common::test_app_from_dirs(dirs);
    let (restarted_detail_status, restarted_detail_body) = get_contract_repair_path_with_app(
        restarted_app,
        &format!("/api/runtime/v4/productization/contract-repairs/approval-requests/{approval_id}"),
    )
    .await;

    assert_eq!(
        restarted_detail_status,
        StatusCode::OK,
        "{restarted_detail_body}"
    );
    assert_eq!(
        restarted_detail_body["record"]["review_state"],
        "under_review"
    );
    assert_eq!(
        restarted_detail_body["record"]["transient_review_status"],
        "claim_executed"
    );
    assert_eq!(
        restarted_detail_body["record"]["lifecycle"][0]["event_id"],
        format!("contract-repair-review-claim:{approval_id}")
    );
    assert_eq!(
        restarted_detail_body["record"]["lifecycle"][0]["review_state"],
        "under_review"
    );
    assert_eq!(
        restarted_detail_body["record"]["lifecycle"][0]["sequence_no"],
        1
    );
    assert_eq!(restarted_detail_body["mutation_enabled"], false);
}

#[tokio::test(flavor = "multi_thread")]
async fn contract_repair_approval_review_intent_requires_existing_preview() {
    let app = common::test_app("api_v4_productization_contract_repair_review_missing");
    let (status, body) = post_contract_repair_path_with_app(
        app,
        "/api/runtime/v4/productization/contract-repairs/approval-requests/contract-repair-apr-missing/review",
        json!({
            "action": "claim",
            "reviewer_id": "reviewer-a",
            "reason": "inspect transient preview",
            "review_enabled": false
        }),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"], "contract_repair_approval_preview_not_found");
}
