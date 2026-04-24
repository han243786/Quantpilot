mod common;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

fn collaboration_graph(graph_id: &str, name: &str) -> Value {
    json!({
        "metadata": {
            "graph_id": graph_id,
            "name": name,
            "version": "1.0.0",
            "created_at": 1700000000000u64,
            "updated_at": 1700000000000u64,
            "runtime_binding": {
                "current_run_id": null,
                "last_compile_id": null
            },
            "editor": {
                "viewport": { "x": 0, "y": 0, "zoom": 0.8 },
                "recent_node_ids": []
            },
            "source_mode": "graph",
            "artifacts": {}
        },
        "nodes": [
            {
                "id": "runtime_runtime_1",
                "module_key": "builtin.runtime.control",
                "name": "Runtime Control",
                "type": "builtin.runtime.control",
                "position": { "x": 0, "y": 0 },
                "input_ports": [],
                "output_ports": [],
                "config": { "mode": "paper" },
                "ui_state": { "collapsed": false }
            },
            {
                "id": "data_data_1",
                "module_key": "builtin.data.kline",
                "name": "Primary Kline",
                "type": "builtin.data.kline",
                "position": { "x": 10, "y": 10 },
                "input_ports": [],
                "output_ports": ["market_data_out"],
                "config": {
                    "exchange": "okx",
                    "instrument": "BTCUSDT",
                    "timeframe": "1d",
                    "window_size": 120
                },
                "ui_state": { "collapsed": false }
            },
            {
                "id": "intent_intent_1",
                "module_key": "builtin.intent.double_ma",
                "name": "Double MA",
                "type": "builtin.intent.double_ma",
                "position": { "x": 30, "y": 10 },
                "input_ports": ["data_input"],
                "output_ports": ["intent_out"],
                "config": {
                    "fast_period": 20,
                    "slow_period": 50,
                    "entry_ratio": 0.2
                },
                "ui_state": { "collapsed": false }
            },
            {
                "id": "agent_agent_1",
                "module_key": "builtin.agent.weighted",
                "name": "Weighted Agent",
                "type": "builtin.agent.weighted",
                "position": { "x": 50, "y": 10 },
                "input_ports": ["intent_input"],
                "output_ports": ["agent_out"],
                "config": {
                    "decision_threshold": 0.05,
                    "max_quantity_ratio": 0.2
                },
                "ui_state": { "collapsed": false }
            },
            {
                "id": "risk_risk_1",
                "module_key": "builtin.risk.global",
                "name": "Global Risk",
                "type": "builtin.risk.global",
                "position": { "x": 70, "y": 10 },
                "input_ports": ["agent_input"],
                "output_ports": ["risk_out"],
                "config": {
                    "max_position": 0.2,
                    "max_total_leverage": 3,
                    "max_exchange_leverage": 3
                },
                "ui_state": { "collapsed": false }
            },
            {
                "id": "execution_execution_1",
                "module_key": "builtin.execution.paper",
                "name": "Paper Execution",
                "type": "builtin.execution.paper",
                "position": { "x": 90, "y": 10 },
                "input_ports": ["risk_input"],
                "output_ports": [],
                "config": {
                    "mode": "paper",
                    "slippage_bps": 5
                },
                "ui_state": { "collapsed": false }
            }
        ],
        "edges": [
            {
                "id": "edge_data_data_1_intent_intent_1_market_data_out_data_input",
                "source_node_id": "data_data_1",
                "source_port": "market_data_out",
                "target_node_id": "intent_intent_1",
                "target_port": "data_input",
                "edge_type": "builtin.data.kline_to_builtin.intent.double_ma"
            },
            {
                "id": "edge_intent_intent_1_agent_agent_1_intent_out_intent_input",
                "source_node_id": "intent_intent_1",
                "source_port": "intent_out",
                "target_node_id": "agent_agent_1",
                "target_port": "intent_input",
                "edge_type": "builtin.intent.double_ma_to_builtin.agent.weighted"
            },
            {
                "id": "edge_agent_agent_1_risk_risk_1_agent_out_agent_input",
                "source_node_id": "agent_agent_1",
                "source_port": "agent_out",
                "target_node_id": "risk_risk_1",
                "target_port": "agent_input",
                "edge_type": "builtin.agent.weighted_to_builtin.risk.global"
            },
            {
                "id": "edge_risk_risk_1_execution_execution_1_risk_out_risk_input",
                "source_node_id": "risk_risk_1",
                "source_port": "risk_out",
                "target_node_id": "execution_execution_1",
                "target_port": "risk_input",
                "edge_type": "builtin.risk.global_to_builtin.execution.paper"
            }
        ],
        "validation_state": {
            "is_valid": true,
            "is_runnable": true,
            "node_issues": {},
            "edge_issues": {},
            "graph_issues": [],
            "issue_counts": { "error": 0, "warning": 0, "info": 0 },
            "last_validated_at": null
        },
        "compile_summary": {
            "compilable": true,
            "last_compile_id": null,
            "last_compile_at": null,
            "topology_order": [],
            "outputs": {
                "data_sources": 1,
                "intent_generators": 1,
                "agents": 1,
                "risk_controls": 1,
                "executions": 1
            },
            "warnings": [],
            "errors": []
        }
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn graph_save_sets_owner_and_exposes_audit_history() {
    let app = common::test_app("api_graph_collaboration");

    let save_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/graphs/save")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph": collaboration_graph("graph_test", "Graph Test"),
                        "actor": {
                            "actor_id": "owner_alice",
                            "display_name": "Alice"
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(save_response.status(), StatusCode::OK);
    let save_body = to_bytes(save_response.into_body(), usize::MAX).await.unwrap();
    let save_payload: Value = serde_json::from_slice(&save_body).unwrap();
    assert_eq!(save_payload["collaboration"]["owner"]["actor_id"], "owner_alice");
    assert_eq!(
        save_payload["collaboration"]["last_saved_by"]["display_name"],
        "Alice"
    );

    let audit_response = app
        .oneshot(
            Request::builder()
                .uri("/api/graphs/graph_test/audit")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(audit_response.status(), StatusCode::OK);
    let audit_body = to_bytes(audit_response.into_body(), usize::MAX).await.unwrap();
    let audit_entries: Value = serde_json::from_slice(&audit_body).unwrap();
    let entries = audit_entries.as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["action"], "graph_saved");
    assert_eq!(entries[0]["actor"]["actor_id"], "owner_alice");
  }

#[tokio::test(flavor = "multi_thread")]
async fn runtime_creation_rejects_unauthorized_actor_for_owned_graph() {
    let app = common::test_app("api_graph_permissions");

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/graphs/save")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph": collaboration_graph("graph_test", "Graph Test"),
                        "actor": {
                            "actor_id": "owner_alice",
                            "display_name": "Alice"
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let mut runtime_payload = common::sample_runtime_request();
    runtime_payload["actor"] = json!({
        "actor_id": "viewer_bob",
        "display_name": "Bob"
    });

    let run_response = app
        .oneshot(
            Request::builder()
                .uri("/api/runtime/test-run")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(runtime_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(run_response.status(), StatusCode::FORBIDDEN);
}
