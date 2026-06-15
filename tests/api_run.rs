mod common;
include!("common/re_exports.rs");

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    Router,
};
use qrpc_core_ir::v4::{
    CapabilitySupportSource, ExecutionCapabilityKind, RuntimeTradingMode, V4MachineGraphContract,
    V4StaticContractBundle,
};
use serde_json::{json, Value};
use tower::ServiceExt;

const VALID_V4_RUNTIME_QS: &str = r#"
v4_strategy strategy.v4.api_run {
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

fn v4_runtime_static_bundle() -> V4StaticContractBundle {
    let mut matrix = qrpc_core_ir::v4::unsupported_v4_first_wave_matrix("paper-local");
    let market = matrix
        .capabilities
        .iter_mut()
        .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
        .expect("first-wave matrix should include market capability");
    market.source = CapabilitySupportSource::RuntimeSimulated;
    market.supported_modes = vec![RuntimeTradingMode::PaperSimulated];
    V4StaticContractBundle {
        venue_matrices: vec![matrix],
        ..V4StaticContractBundle::default()
    }
}

fn parsed_v4_runtime_graph() -> V4MachineGraphContract {
    let report =
        quantscript::audit_v4_quant_script_static(VALID_V4_RUNTIME_QS, &v4_runtime_static_bundle());
    assert_eq!(
        report.verdict,
        quantscript::V4QsStaticAuditVerdict::Accepted,
        "{:?}",
        report.diagnostics
    );
    report
        .parsed_graph
        .expect("accepted v4 QS should produce a machine graph")
}

async fn post_v4_runtime_run(app: Router, payload: Value) -> (StatusCode, Value) {
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/runtime/v4/run")
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
async fn start_v4_runtime_run_accepts_paper_simulated_qs_source() {
    let app = common::test_app("api_v4_run_source");

    let (status, body) = post_v4_runtime_run(app, json!({ "source": VALID_V4_RUNTIME_QS })).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["graph_id"], "strategy.v4.api_run");
    assert!(body["run_id"].as_str().unwrap().starts_with("v4_run_"));
    assert!(body["event_count"].as_u64().unwrap() > 0);
    assert_eq!(body["output"]["runtime_mode"], "paper_simulated");
    assert_eq!(
        body["output"]["provider_order_submission_attached"],
        Value::Bool(false)
    );
    assert_eq!(body["handoff"]["accepted_for_runtime_handoff"], true);
    assert_eq!(body["handoff"]["paper_simulated_start_allowed"], true);
    assert_eq!(body["handoff"]["provider_order_submission_attached"], false);
    assert!(body["diagnostics"].as_array().unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn start_v4_runtime_run_uses_default_catalog_event_for_source() {
    let app = common::test_app("api_v4_run_default_event");

    let (status, body) = post_v4_runtime_run(app, json!({ "source": VALID_V4_RUNTIME_QS })).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    let first_event = &body["output"]["events"][0];
    assert_eq!(first_event["event_type"], "bar_closed");
    assert_eq!(first_event["source"], "runtime");
    assert_eq!(
        body["output"]["memory_snapshot"]["graph_id"],
        "strategy.v4.api_run"
    );
    assert_eq!(
        body["output"]["memory_snapshot"]["provider_order_submission_attached"],
        false
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn start_v4_runtime_run_accepts_preparsed_graph_without_handoff() {
    let app = common::test_app("api_v4_run_graph");
    let graph = parsed_v4_runtime_graph();

    let (status, body) = post_v4_runtime_run(app, json!({ "graph": graph })).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["graph_id"], "strategy.v4.api_run");
    assert!(body.get("handoff").is_none());
    assert!(body["diagnostics"].as_array().unwrap().is_empty());
    assert_eq!(body["output"]["runtime_mode"], "paper_simulated");
}

#[tokio::test(flavor = "multi_thread")]
async fn start_v4_runtime_run_accepts_initial_event_override() {
    let app = common::test_app("api_v4_run_initial_event");
    let graph = parsed_v4_runtime_graph();

    let (status, body) = post_v4_runtime_run(
        app,
        json!({
            "graph": graph,
            "initial_event": {
                "event_type": "market.tick",
                "source": "manual.smoke",
                "payload": {},
                "ts_ms": 1730000000000_u64
            }
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    let first_event = &body["output"]["events"][0];
    assert_eq!(first_event["event_type"], "market.tick");
    assert_eq!(first_event["source"], "manual.smoke");
    assert_eq!(first_event["ts_ms"], Value::from(1730000000000_u64));
}

#[tokio::test(flavor = "multi_thread")]
async fn start_v4_runtime_run_rejects_missing_source_and_graph() {
    let app = common::test_app("api_v4_run_missing_source");

    let (status, body) = post_v4_runtime_run(app, json!({})).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "v4_source_missing");
    assert_eq!(body["error_code"], "QSC4001");
}

#[tokio::test(flavor = "multi_thread")]
async fn start_v4_runtime_run_rejects_non_paper_simulated_handoff() {
    let app = common::test_app("api_v4_run_live_actual_rejected");
    let live_actual_source =
        VALID_V4_RUNTIME_QS.replace("mode paper_simulated", "mode live_actual");

    let (status, body) = post_v4_runtime_run(app, json!({ "source": live_actual_source })).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "v4_runtime_handoff_rejected");
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("PaperSimulated start"));
}

#[tokio::test(flavor = "multi_thread")]
async fn start_v4_runtime_run_rejects_graph_without_runtime_event_catalog() {
    let app = common::test_app("api_v4_run_empty_catalog");
    let mut graph = parsed_v4_runtime_graph();
    graph.machines.truncate(1);
    graph.machines[0].transitions.clear();
    graph.edges.clear();
    graph.risk_plane = None;
    graph.event_catalog = None;

    let (status, body) = post_v4_runtime_run(app, json!({ "graph": graph })).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "v4_event_catalog_missing");
}
