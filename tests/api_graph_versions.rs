mod common;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tokio::time::{sleep, Duration};
use tower::ServiceExt;

fn sample_graph(graph_id: &str, name: &str, window_size: u64, remove_edge: bool) -> Value {
    let mut edges = vec![
        json!({
            "id": "edge_data_data_1_intent_intent_1_market_data_out_data_input",
            "source_node_id": "data_data_1",
            "source_port": "market_data_out",
            "target_node_id": "intent_intent_1",
            "target_port": "data_input",
            "edge_type": "builtin.data.kline_to_builtin.intent.double_ma"
        }),
        json!({
            "id": "edge_intent_intent_1_agent_agent_1_intent_out_intent_input",
            "source_node_id": "intent_intent_1",
            "source_port": "intent_out",
            "target_node_id": "agent_agent_1",
            "target_port": "intent_input",
            "edge_type": "builtin.intent.double_ma_to_builtin.agent.weighted"
        }),
        json!({
            "id": "edge_agent_agent_1_risk_risk_1_agent_out_agent_input",
            "source_node_id": "agent_agent_1",
            "source_port": "agent_out",
            "target_node_id": "risk_risk_1",
            "target_port": "agent_input",
            "edge_type": "builtin.agent.weighted_to_builtin.risk.global"
        }),
        json!({
            "id": "edge_risk_risk_1_execution_execution_1_risk_out_risk_input",
            "source_node_id": "risk_risk_1",
            "source_port": "risk_out",
            "target_node_id": "execution_execution_1",
            "target_port": "risk_input",
            "edge_type": "builtin.risk.global_to_builtin.execution.paper"
        }),
    ];

    if remove_edge {
        edges.pop();
    }

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
                    "window_size": window_size
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
                "name": if remove_edge { "Global Risk Updated" } else { "Global Risk" },
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
        "edges": edges,
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
async fn graph_version_endpoints_expose_labels_notes_and_compare_diff() {
    let app = common::test_app("api_graph_versions_contract");

    let first_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/graphs/save")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph": sample_graph("alpha_strategy", "Alpha baseline", 20, false),
                        "version_label": "baseline",
                        "save_note": "Initial persisted strategy snapshot."
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(first_response.status(), StatusCode::OK);
    let first_body = to_bytes(first_response.into_body(), usize::MAX).await.unwrap();
    let first_saved: Value = serde_json::from_slice(&first_body).unwrap();
    let first_version_id = first_saved["version_id"].as_str().unwrap().to_string();
    assert_eq!(first_saved["version_label"], "baseline");
    assert_eq!(first_saved["save_note"], "Initial persisted strategy snapshot.");

    sleep(Duration::from_millis(2)).await;

    let second_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/graphs/save")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph": sample_graph("alpha_strategy", "Alpha tuned", 55, true),
                        "version_label": "tuned",
                        "save_note": "Raised window size and removed one execution edge."
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(second_response.status(), StatusCode::OK);
    let second_body = to_bytes(second_response.into_body(), usize::MAX).await.unwrap();
    let second_saved: Value = serde_json::from_slice(&second_body).unwrap();
    let second_version_id = second_saved["version_id"].as_str().unwrap().to_string();
    assert_eq!(second_saved["version_label"], "tuned");
    assert_eq!(
        second_saved["save_note"],
        "Raised window size and removed one execution edge."
    );

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/graphs/alpha_strategy/versions")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body = to_bytes(list_response.into_body(), usize::MAX).await.unwrap();
    let versions: Value = serde_json::from_slice(&list_body).unwrap();
    let versions = versions.as_array().unwrap();
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0]["version_id"], second_version_id);
    assert_eq!(versions[0]["version_label"], "tuned");
    assert_eq!(versions[0]["node_count"], 6);
    assert_eq!(versions[0]["edge_count"], 3);
    assert_eq!(versions[1]["version_id"], first_version_id);
    assert_eq!(versions[1]["version_label"], "baseline");
    assert_eq!(versions[1]["node_count"], 6);
    assert_eq!(versions[1]["edge_count"], 4);

    let compare_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/graphs/alpha_strategy/versions/compare/{first_version_id}/{second_version_id}"
                ))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(compare_response.status(), StatusCode::OK);
    let compare_body = to_bytes(compare_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let compare: Value = serde_json::from_slice(&compare_body).unwrap();

    assert_eq!(compare["graph_id"], "alpha_strategy");
    assert_eq!(compare["left"]["version_label"], "baseline");
    assert_eq!(compare["right"]["version_label"], "tuned");
    assert_eq!(compare["has_changes"], Value::Bool(true));
    assert!(compare["metadata_rows"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["key"] == "version_label" && row["left_value"] == "baseline" && row["right_value"] == "tuned"));
    assert!(compare["node_diff"]["changed_ids"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "risk_risk_1"));
    assert!(compare["edge_diff"]["removed_ids"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "edge_risk_risk_1_execution_execution_1_risk_out_risk_input"));
    assert!(compare["config_diffs"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| {
            row["node_id"] == "data_data_1"
                && row["field_path"] == "window_size"
                && row["left_value"] == "20"
                && row["right_value"] == "55"
        }));
}
