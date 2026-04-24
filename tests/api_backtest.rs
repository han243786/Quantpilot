mod common;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

const BTC_DUAL_MA_STABILITY: &str = include_str!("../quantscript/btc_dual_ma_stability.qs");

fn compare_status_for_values(left: &Value, right: &Value) -> &'static str {
    if left.is_null() || right.is_null() {
        "missing"
    } else if left == right {
        "same"
    } else {
        "different"
    }
}

fn scalar_value_string(value: &Value) -> String {
    if let Some(integer) = value.as_u64() {
        integer.to_string()
    } else if let Some(integer) = value.as_i64() {
        integer.to_string()
    } else if let Some(float) = value.as_f64() {
        float.to_string()
    } else if let Some(text) = value.as_str() {
        text.to_string()
    } else {
        value.to_string()
    }
}

fn expected_metrics_drilldown(left: &Value, right: &Value) -> Value {
    fn field(key: &str, left_summary: &Value, right_summary: &Value) -> Value {
        let left_value = left_summary[key].clone();
        let right_value = right_summary[key].clone();
        json!({
            "key": key,
            "status": compare_status_for_values(&left_value, &right_value),
            "left_value": scalar_value_string(&left_value),
            "right_value": scalar_value_string(&right_value)
        })
    }

    let performance = vec![
        field("total_return_ratio", left, right),
        field("net_profit", left, right),
        field("final_equity", left, right),
        field("max_drawdown_ratio", left, right),
    ];
    let activity = vec![
        field("step_count", left, right),
        field("trade_count", left, right),
        field("turnover_ratio", left, right),
        field("average_trade_notional", left, right),
    ];
    let costs = vec![field("fee_drag_ratio", left, right)];

    let group_status = |fields: &[Value]| {
        if fields.iter().all(|field| field["status"] == json!("same")) {
            "same"
        } else if fields
            .iter()
            .any(|field| field["status"] == json!("missing"))
        {
            "missing"
        } else {
            "different"
        }
    };

    json!({
        "performance": {
            "status": group_status(&performance),
            "fields": performance,
        },
        "activity": {
            "status": group_status(&activity),
            "fields": activity,
        },
        "costs": {
            "status": group_status(&costs),
            "fields": costs,
        }
    })
}

fn expected_trade_ledger_fields(left: &Value, right: &Value) -> Value {
    let field = |key: &str| {
        json!({
            "status": compare_status_for_values(&left[key], &right[key])
        })
    };
    json!({
        "trade_count": field("trade_count"),
        "buy_fill_count": field("buy_fill_count"),
        "sell_fill_count": field("sell_fill_count"),
        "total_fees_paid": field("total_fees_paid"),
        "buy_fees_paid": field("buy_fees_paid"),
        "sell_fees_paid": field("sell_fees_paid"),
        "total_filled_notional": field("total_filled_notional"),
        "buy_filled_notional": field("buy_filled_notional"),
        "sell_filled_notional": field("sell_filled_notional"),
        "average_fill_price": field("average_fill_price"),
        "average_buy_fill_price": field("average_buy_fill_price"),
        "average_sell_fill_price": field("average_sell_fill_price"),
        "average_fee_per_fill": field("average_fee_per_fill"),
        "average_buy_fee": field("average_buy_fee"),
        "average_sell_fee": field("average_sell_fee")
    })
}

fn format_metrics_summary_value(summary: &Value) -> String {
    format!(
        "steps={}, trades={}, return={}, drawdown={}, final_equity={}, net_profit={}, turnover_ratio={}, average_trade_notional={}, fee_drag_ratio={}",
        summary["step_count"],
        summary["trade_count"],
        scalar_value_string(&summary["total_return_ratio"]),
        scalar_value_string(&summary["max_drawdown_ratio"]),
        scalar_value_string(&summary["final_equity"]),
        scalar_value_string(&summary["net_profit"]),
        scalar_value_string(&summary["turnover_ratio"]),
        scalar_value_string(&summary["average_trade_notional"]),
        scalar_value_string(&summary["fee_drag_ratio"]),
    )
}

fn format_trade_ledger_summary_value(summary: &Value) -> String {
    format!(
        "trade_count={}, buy_fill_count={}, sell_fill_count={}, total_fees_paid={}, buy_fees_paid={}, sell_fees_paid={}, total_filled_notional={}, buy_filled_notional={}, sell_filled_notional={}, average_fill_price={}, average_buy_fill_price={}, average_sell_fill_price={}, average_fee_per_fill={}, average_buy_fee={}, average_sell_fee={}",
        summary["trade_count"],
        summary["buy_fill_count"],
        summary["sell_fill_count"],
        scalar_value_string(&summary["total_fees_paid"]),
        scalar_value_string(&summary["buy_fees_paid"]),
        scalar_value_string(&summary["sell_fees_paid"]),
        scalar_value_string(&summary["total_filled_notional"]),
        scalar_value_string(&summary["buy_filled_notional"]),
        scalar_value_string(&summary["sell_filled_notional"]),
        scalar_value_string(&summary["average_fill_price"]),
        if summary["average_buy_fill_price"].is_null() {
            "na".to_string()
        } else {
            scalar_value_string(&summary["average_buy_fill_price"])
        },
        if summary["average_sell_fill_price"].is_null() {
            "na".to_string()
        } else {
            scalar_value_string(&summary["average_sell_fill_price"])
        },
        scalar_value_string(&summary["average_fee_per_fill"]),
        if summary["average_buy_fee"].is_null() {
            "na".to_string()
        } else {
            scalar_value_string(&summary["average_buy_fee"])
        },
        if summary["average_sell_fee"].is_null() {
            "na".to_string()
        } else {
            scalar_value_string(&summary["average_sell_fee"])
        },
    )
}

fn summarize_equity_curve_value(points: &Value) -> Value {
    let points = points
        .as_array()
        .expect("equity curve points should be an array");
    let first = points
        .first()
        .expect("equity curve should have a first point");
    let last = points
        .last()
        .expect("equity curve should have a last point");
    let min_equity = points
        .iter()
        .filter_map(|point| point["equity"].as_f64())
        .fold(f64::INFINITY, f64::min);
    let max_equity = points
        .iter()
        .filter_map(|point| point["equity"].as_f64())
        .fold(f64::NEG_INFINITY, f64::max);
    json!({
        "point_count": points.len(),
        "started_at_ms": first["ts_ms"].clone(),
        "ended_at_ms": last["ts_ms"].clone(),
        "first_equity": first["equity"].clone(),
        "final_equity": last["equity"].clone(),
        "min_equity": min_equity,
        "max_equity": max_equity,
    })
}

fn expected_equity_curve_fields(left_summary: &Value, right_summary: &Value) -> Value {
    let field = |key: &str| {
        json!({
            "status": compare_status_for_values(&left_summary[key], &right_summary[key])
        })
    };
    json!({
        "point_count": field("point_count"),
        "started_at_ms": field("started_at_ms"),
        "ended_at_ms": field("ended_at_ms"),
        "first_equity": field("first_equity"),
        "final_equity": field("final_equity"),
        "min_equity": field("min_equity"),
        "max_equity": field("max_equity"),
    })
}

fn expected_equity_curve_drilldown(left_points: &Value, right_points: &Value) -> Value {
    let left_points = left_points
        .as_array()
        .expect("left equity curve points should be an array");
    let right_points = right_points
        .as_array()
        .expect("right equity curve points should be an array");
    let samples = vec![
        ("start", 0usize, 0usize),
        ("middle", left_points.len() / 2, right_points.len() / 2),
        ("end", left_points.len() - 1, right_points.len() - 1),
    ]
    .into_iter()
    .map(|(key, left_index, right_index)| {
        let left_sample = left_points[left_index].clone();
        let right_sample = right_points[right_index].clone();
        json!({
            "key": key,
            "status": compare_status_for_values(&left_sample, &right_sample),
            "left": left_sample,
            "right": right_sample
        })
    })
    .collect::<Vec<_>>();
    json!({ "samples": samples })
}

fn format_equity_curve_summary_value(summary: &Value) -> String {
    format!(
        "point_count={}, started_at_ms={}, ended_at_ms={}, first_equity={}, final_equity={}, min_equity={}, max_equity={}",
        summary["point_count"],
        summary["started_at_ms"],
        summary["ended_at_ms"],
        scalar_value_string(&summary["first_equity"]),
        scalar_value_string(&summary["final_equity"]),
        scalar_value_string(&summary["min_equity"]),
        scalar_value_string(&summary["max_equity"]),
    )
}

fn differing_equity_curve_fields(left_summary: &Value, right_summary: &Value) -> Vec<&'static str> {
    let pairs = [
        ("point_count", "point_count"),
        ("started_at_ms", "started_at_ms"),
        ("ended_at_ms", "ended_at_ms"),
        ("first_equity", "first_equity"),
        ("final_equity", "final_equity"),
        ("min_equity", "min_equity"),
        ("max_equity", "max_equity"),
    ];
    pairs
        .into_iter()
        .filter_map(|(key, label)| {
            if compare_status_for_values(&left_summary[key], &right_summary[key]) == "different" {
                Some(label)
            } else {
                None
            }
        })
        .collect()
}

fn summarize_equity_curve_samples_label(drilldown: &Value) -> String {
    let samples = drilldown["samples"]
        .as_array()
        .expect("equity curve drilldown should include samples");
    if samples
        .iter()
        .all(|sample| sample["status"] == json!("same"))
    {
        "same".to_string()
    } else if samples
        .iter()
        .any(|sample| sample["status"] == json!("missing"))
    {
        "missing".to_string()
    } else {
        format!(
            "different on {}",
            samples
                .iter()
                .filter(|sample| sample["status"] == json!("different"))
                .map(|sample| sample["key"].as_str().unwrap())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn compare_equity_curve_status(
    left_summary: &Value,
    right_summary: &Value,
    drilldown: &Value,
) -> &'static str {
    let fields = expected_equity_curve_fields(left_summary, right_summary);
    if fields
        .as_object()
        .expect("equity curve fields should be an object")
        .values()
        .all(|field| field["status"] == json!("same"))
        && drilldown["samples"]
            .as_array()
            .expect("equity curve drilldown should include samples")
            .iter()
            .all(|sample| sample["status"] == json!("same"))
    {
        "same"
    } else if fields
        .as_object()
        .expect("equity curve fields should be an object")
        .values()
        .any(|field| field["status"] == json!("missing"))
        || drilldown["samples"]
            .as_array()
            .expect("equity curve drilldown should include samples")
            .iter()
            .any(|sample| sample["status"] == json!("missing"))
    {
        "missing"
    } else {
        "different"
    }
}

fn summarize_equity_curve_highlight_value(left_summary: &Value, right_summary: &Value) -> String {
    let diff_fields = differing_equity_curve_fields(left_summary, right_summary);
    if diff_fields.is_empty() {
        "Equity curve matches on all tracked fields.".to_string()
    } else {
        format!("Equity curve differs on: {}.", diff_fields.join(", "))
    }
}

fn expected_execution_assumptions_report_module(
    status: &str,
    summary: &str,
    left_values_line: &str,
    right_values_line: &str,
    source_explanations: Vec<String>,
) -> Value {
    json!({
        "status": status,
        "summary": summary,
        "lines": [
            format!("Status: {}.", status),
            left_values_line,
            right_values_line
        ],
        "source_explanations": source_explanations
    })
}

fn expected_metrics_report_module(
    status: &str,
    summary: &str,
    left_summary: &Value,
    right_summary: &Value,
    drilldown: Value,
) -> Value {
    let performance_status = drilldown["performance"]["status"].as_str().unwrap();
    let activity_status = drilldown["activity"]["status"].as_str().unwrap();
    let costs_status = drilldown["costs"]["status"].as_str().unwrap();
    json!({
        "status": status,
        "summary": summary,
        "lines": [
            format!("Status: {}.", status),
            format!("Performance drilldown: {}.", performance_status),
            format!("Activity drilldown: {}.", activity_status),
            format!("Cost drilldown: {}.", costs_status),
            format!("Left summary: {}.", format_metrics_summary_value(left_summary)),
            format!("Right summary: {}.", format_metrics_summary_value(right_summary))
        ],
        "drilldown": drilldown
    })
}

fn expected_trade_ledger_report_module(
    status: &str,
    summary: &str,
    left_summary: &Value,
    right_summary: &Value,
) -> Value {
    json!({
        "status": status,
        "summary": summary,
        "lines": [
            format!("Status: {}.", status),
            format!("Left summary: {}.", format_trade_ledger_summary_value(left_summary)),
            format!("Right summary: {}.", format_trade_ledger_summary_value(right_summary))
        ]
    })
}

fn expected_equity_curve_report_module(
    status: &str,
    summary: &str,
    left_summary: &Value,
    right_summary: &Value,
    drilldown: Value,
) -> Value {
    json!({
        "status": status,
        "summary": summary,
        "lines": [
            format!("Status: {}.", status),
            format!("Sample drilldown: {}.", summarize_equity_curve_samples_label(&drilldown)),
            format!("Left summary: {}.", format_equity_curve_summary_value(left_summary)),
            format!("Right summary: {}.", format_equity_curve_summary_value(right_summary))
        ],
        "drilldown": drilldown
    })
}

#[allow(clippy::too_many_arguments)]
fn expected_report_narrative_from_modules(
    status: &str,
    headline: &str,
    bullets: Vec<String>,
    highlights: Vec<String>,
    assumptions_module: &Value,
    metrics_module: &Value,
    trade_ledger_module: &Value,
    equity_curve_module: &Value,
) -> Value {
    json!({
        "status": status,
        "headline": headline,
        "bullets": bullets,
        "highlights": highlights,
        "source_explanations": assumptions_module["source_explanations"].clone(),
        "sections": [
            {
                "title": "Execution assumptions",
                "status": assumptions_module["status"].clone(),
                "summary": assumptions_module["summary"].clone(),
                "lines": assumptions_module["lines"].clone()
            },
            {
                "title": "Metrics summary",
                "status": metrics_module["status"].clone(),
                "summary": metrics_module["summary"].clone(),
                "lines": metrics_module["lines"].clone()
            },
            {
                "title": "Trade ledger summary",
                "status": trade_ledger_module["status"].clone(),
                "summary": trade_ledger_module["summary"].clone(),
                "lines": trade_ledger_module["lines"].clone()
            },
            {
                "title": "Equity curve",
                "status": equity_curve_module["status"].clone(),
                "summary": equity_curve_module["summary"].clone(),
                "lines": equity_curve_module["lines"].clone()
            }
        ]
    })
}

#[allow(clippy::too_many_arguments)]
fn expected_compare_report_from_modules(
    status: &str,
    headline: &str,
    bullets: Vec<String>,
    highlights: Vec<String>,
    assumptions_module: Value,
    metrics_module: Value,
    trade_ledger_module: Value,
    equity_curve_module: Value,
) -> Value {
    json!({
        "status": status,
        "headline": headline,
        "overview": {
            "bullets": bullets,
            "highlights": highlights
        },
        "modules": {
            "execution_assumptions": assumptions_module,
            "metrics": metrics_module,
            "trade_ledger": trade_ledger_module,
            "equity_curve": equity_curve_module
        }
    })
}

#[allow(clippy::too_many_arguments)]
fn expected_compare_outputs_from_modules(
    status: &str,
    headline: &str,
    bullets: Vec<String>,
    highlights: Vec<String>,
    assumptions_module: Value,
    metrics_module: Value,
    trade_ledger_module: Value,
    equity_curve_module: Value,
) -> (Value, Value) {
    let report_narrative = expected_report_narrative_from_modules(
        status,
        headline,
        bullets.clone(),
        highlights.clone(),
        &assumptions_module,
        &metrics_module,
        &trade_ledger_module,
        &equity_curve_module,
    );
    let compare_report = expected_compare_report_from_modules(
        status,
        headline,
        bullets,
        highlights,
        assumptions_module,
        metrics_module,
        trade_ledger_module,
        equity_curve_module,
    );
    (report_narrative, compare_report)
}

#[tokio::test(flavor = "multi_thread")]
async fn backtest_start_endpoint_supports_deterministic_mock_happy_path() {
    let app = common::test_app("api_backtest_happy_path");
    let mut payload = common::sample_runtime_request();
    payload["backtest_options"] = json!({
        "replay_source": "deterministic_mock"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtest")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: Value = serde_json::from_slice(&body).unwrap();
    let backtest_id = created["backtest_id"].as_str().unwrap().to_string();

    assert_eq!(created["graph_id"], "graph_test");
    assert_eq!(created["compile_id"], "compile_test");
    assert!(!created["protocol_name"].as_str().unwrap().is_empty());
    assert!(!created["config_hash"].as_str().unwrap().is_empty());
    assert_eq!(
        created["event_count"].as_u64().unwrap(),
        created["backtest_artifacts"]["event_log"]["events"]
            .as_array()
            .unwrap()
            .len() as u64
    );
    assert!(created["event_count"].as_u64().unwrap() > 0);
    assert_eq!(
        created["backtest_artifacts"]["manifest"]["backtest_spec"]["replay_source"],
        "deterministic_mock"
    );
    assert_eq!(
        created["execution_assumptions"],
        created["backtest_artifacts"]["metrics"]["execution_assumptions"]
    );
    assert_eq!(
        created["backtest_artifacts"]["manifest"]["output_artifacts"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert!(created["backtest_artifacts"]["metrics"]["summary"].is_object());

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtests")
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
    let backtests: Value = serde_json::from_slice(&list_body).unwrap();
    let items = backtests.as_array().unwrap();
    let listed = items
        .iter()
        .find(|item| item["backtest_id"] == backtest_id)
        .expect("created backtest should be listed");

    assert_eq!(listed["graph_id"], "graph_test");
    assert_eq!(listed["compile_id"], "compile_test");
    assert_eq!(listed["protocol_name"], created["protocol_name"]);
    assert_eq!(listed["config_hash"], created["config_hash"]);
    assert_eq!(listed["event_count"], created["event_count"]);
    assert_eq!(listed["filters"]["replay_source"], "deterministic_mock");
    assert!(!listed["filters"]["dataset_labels"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(
        listed["filters"]["execution_assumptions_tag"]["label"],
        created["execution_assumptions"]["list_tag"]["label"]
    );
    assert_eq!(
        listed["filters"]["execution_assumptions_tag"]["sources_label"],
        created["execution_assumptions"]["list_tag"]["sources_label"]
    );
    assert!(listed["filters"]["started_at_ms"].as_u64().unwrap() > 0);
    assert!(listed["filters"]["ended_at_ms"].as_u64().unwrap() > 0);

    let detail_response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/backtests/{backtest_id}"))
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

    assert_eq!(detail["backtest_id"], backtest_id);
    assert_eq!(detail["graph_id"], "graph_test");
    assert_eq!(detail["compile_id"], "compile_test");
    assert_eq!(detail["protocol_name"], created["protocol_name"]);
    assert_eq!(detail["config_hash"], created["config_hash"]);
    assert_eq!(detail["event_count"], created["event_count"]);
    assert_eq!(
        detail["backtest_artifacts"]["event_log"]["events"]
            .as_array()
            .unwrap()
            .len() as u64,
        detail["event_count"].as_u64().unwrap()
    );
    assert_eq!(
        detail["backtest_artifacts"]["metrics"]["summary"],
        created["backtest_artifacts"]["metrics"]["summary"]
    );
    assert_eq!(
        detail["execution_assumptions"],
        created["execution_assumptions"]
    );
    assert_eq!(
        detail["runtime_diagnostics"]["source"],
        Value::String("backtest_event_log".to_string())
    );
    assert!(!detail["runtime_diagnostics"]["active_nodes"]
        .as_array()
        .unwrap()
        .is_empty());
    let selected_node_id = detail["runtime_diagnostics"]["default_selected_node_id"]
        .as_str()
        .unwrap();
    assert!(
        detail["runtime_diagnostics"]["node_details"][selected_node_id]["latest_event"].is_object()
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
}

#[tokio::test(flavor = "multi_thread")]
async fn backtest_start_endpoint_keeps_historical_replay_failure_contract() {
    let app = common::test_app("api_backtest_historical_failure");
    let mut payload = common::sample_runtime_request();
    payload["backtest_options"] = json!({
        "replay_source": "historical_replay"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtest")
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

    assert_eq!(error["error"], "bad_request");
    assert!(error["message"]
        .as_str()
        .unwrap()
        .contains("failed to load historical replay bars"));
    assert!(error["details"].as_array().unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn backtest_endpoint_honors_runtime_targets_for_event_node_mapping() {
    let app = common::test_app("api_backtest_runtime_targets");
    let mut payload = common::sample_runtime_request();
    payload["runtime_targets"] = json!({
        "source_to_node": {
            "data_data_1": "custom_data",
            "intent_intent_1": "custom_intent",
            "agent_agent_1": "custom_agent",
            "risk_risk_1": "custom_risk"
        },
        "runtime_node_id": "custom_runtime",
        "execution_node_id": "custom_execution"
    });
    payload["backtest_options"] = json!({
        "replay_source": "deterministic_mock"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtest")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: Value = serde_json::from_slice(&body).unwrap();
    let node_ids = created["backtest_artifacts"]["event_log"]["events"]
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
async fn backtest_start_endpoint_applies_execution_assumption_overrides_to_manifest() {
    let app = common::test_app("api_backtest_execution_assumption_overrides");
    let mut payload = common::sample_runtime_request();
    payload["runtime_config"]["executions"][0]["config"] = json!({
        "profile_id": "paper",
        "mode": "paper",
        "fee_bps": 12.5,
        "slippage_bps": 7.5
    });
    payload["backtest_options"] = json!({
        "replay_source": "deterministic_mock",
        "execution_assumptions": {
            "fee_bps": 3.5,
            "slippage_bps": 1.25,
            "latency_ms": 250
        }
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtest")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: Value = serde_json::from_slice(&body).unwrap();
    let assumptions = &created["backtest_artifacts"]["manifest"]["backtest_spec"]["run_spec"]
        ["execution_assumptions"];
    let assumption_sources = &created["backtest_artifacts"]["manifest"]["backtest_spec"]
        ["run_spec"]["execution_assumption_sources"];
    let metrics_assumptions = &created["backtest_artifacts"]["metrics"]["execution_assumptions"];
    let golden_like_shape = json!({
        "manifest": {
            "taker_fee_bps": assumptions["taker_fee_bps"].clone(),
            "default_slippage_bps": assumptions["default_slippage_bps"].clone(),
            "latency_assumption_ms": assumptions["latency_assumption_ms"].clone(),
            "sources": assumption_sources.clone(),
        },
        "metrics": {
            "summary": metrics_assumptions["summary"].clone(),
            "list_tag": metrics_assumptions["list_tag"].clone(),
        }
    });

    assert_eq!(assumptions["taker_fee_bps"], json!(3.5));
    assert_eq!(assumptions["default_slippage_bps"], json!(1.25));
    assert_eq!(assumptions["latency_assumption_ms"], json!(250));
    assert_eq!(metrics_assumptions["summary"]["fee_bps"], json!(3.5));
    assert_eq!(metrics_assumptions["summary"]["slippage_bps"], json!(1.25));
    assert_eq!(metrics_assumptions["summary"]["latency_ms"], json!(250));
    assert_eq!(
        created["execution_assumptions"],
        metrics_assumptions.clone()
    );
    assert_eq!(
        assumption_sources,
        &json!({
            "fee_bps": "request_override",
            "slippage_bps": "request_override",
            "latency_ms": "request_override",
        })
    );
    assert_eq!(
        golden_like_shape,
        json!({
            "manifest": {
                "taker_fee_bps": 3.5,
                "default_slippage_bps": 1.25,
                "latency_assumption_ms": 250,
                "sources": {
                    "fee_bps": "request_override",
                    "slippage_bps": "request_override",
                    "latency_ms": "request_override",
                }
            },
            "metrics": {
                "summary": {
                    "fee_bps": 3.5,
                    "slippage_bps": 1.25,
                    "latency_ms": 250,
                    "sources": {
                        "fee_bps": "request_override",
                        "slippage_bps": "request_override",
                        "latency_ms": "request_override",
                    }
                },
                "list_tag": {
                    "label": "fee=3.5 slip=1.25 lat=250",
                    "sources_label": "fee:req slip:req lat:req"
                }
            }
        })
    );

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtests")
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
    let backtests: Value = serde_json::from_slice(&list_body).unwrap();
    let listed = backtests
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["backtest_id"] == created["backtest_id"])
        .expect("created backtest should be listed");

    assert_eq!(
        listed["filters"]["execution_assumptions_tag"],
        json!({
            "label": "fee=3.5 slip=1.25 lat=250",
            "sources_label": "fee:req slip:req lat:req"
        })
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn backtest_replay_endpoint_exposes_paginated_ordered_timeline() {
    let app = common::test_app("api_backtest_replay");
    let mut payload = common::sample_runtime_request();
    payload["backtest_options"] = json!({
        "replay_source": "deterministic_mock"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtest")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: Value = serde_json::from_slice(&body).unwrap();
    let backtest_id = created["backtest_id"].as_str().unwrap();

    let replay_response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/runtime/backtests/{backtest_id}/replay?limit=3"
                ))
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

    assert_eq!(replay["kind"], "backtest");
    assert_eq!(replay["record_id"], backtest_id);
    assert_eq!(replay["graph_id"], "graph_test");
    assert_eq!(replay["cursor"], 0);
    assert_eq!(replay["limit"], 3);
    assert!(!replay["checkpoints"].as_array().unwrap().is_empty());
    assert!(replay["fill_event_count"].as_u64().is_some());
    assert!(replay["events"].as_array().unwrap().len() <= 3);
    let events = replay["events"].as_array().unwrap();
    assert!(!events.is_empty());
    assert_eq!(events[0]["sequence_no"], 1);
    for window in events.windows(2) {
        let left = window[0]["sequence_no"].as_u64().unwrap();
        let right = window[1]["sequence_no"].as_u64().unwrap();
        assert_eq!(right, left + 1);
    }
    if replay["total_events"].as_u64().unwrap() > 3 {
        assert_eq!(replay["next_cursor"], 3);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn backtest_start_endpoint_applies_latency_override_to_execution_timestamps() {
    let app = common::test_app("api_backtest_execution_assumption_latency_contract");
    let mut payload = common::sample_runtime_request();
    payload["backtest_options"] = json!({
        "replay_source": "deterministic_mock",
        "execution_assumptions": {
            "latency_ms": 250
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtest")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: Value = serde_json::from_slice(&body).unwrap();
    let events = created["backtest_artifacts"]["event_log"]["events"]
        .as_array()
        .unwrap();
    let first_execution = events
        .iter()
        .find(|event| event["event_type"] == "ExecutionPlanned")
        .expect("backtest should produce at least one execution plan");
    let session_index = first_execution["payload"]["artifact_projection"]["session_index"]
        .as_u64()
        .expect("execution plan should include artifact projection session index");
    let first_data_update = events
        .iter()
        .find(|event| {
            event["event_type"] == "DataUpdated"
                && event["payload"]["artifact_projection"]["session_index"] == session_index
        })
        .expect("execution plan session should include a projected data update");

    let data_update_time_ms = first_data_update["event_time_ms"].as_u64().unwrap();
    let execution_time_ms = first_execution["event_time_ms"].as_u64().unwrap();
    assert!(
        execution_time_ms >= data_update_time_ms + 250,
        "expected execution timestamp {execution_time_ms} to reflect latency after data update {data_update_time_ms}"
    );

    let equity_points = created["backtest_artifacts"]["equity_curve"]["points"]
        .as_array()
        .expect("equity curve points should be projected");
    let projected_session_started_at_ms = first_execution["payload"]["artifact_projection"]
        ["session_started_at_ms"]
        .as_u64()
        .unwrap();
    assert_eq!(
        equity_points[session_index as usize]["ts_ms"],
        json!(projected_session_started_at_ms),
        "equity curve should project the delayed execution clock for the same session"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn backtest_start_endpoint_rejects_negative_execution_assumption_override() {
    let app = common::test_app("api_backtest_execution_assumption_negative_fee");
    let mut payload = common::sample_runtime_request();
    payload["backtest_options"] = json!({
        "replay_source": "deterministic_mock",
        "execution_assumptions": {
            "fee_bps": -1.0
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtest")
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

    assert_eq!(error["error"], "bad_request");
    assert!(error["message"]
        .as_str()
        .unwrap()
        .contains("execution_assumptions.fee_bps"));
}

#[tokio::test(flavor = "multi_thread")]
async fn backtest_compare_endpoint_reports_same_execution_assumptions() {
    let app = common::test_app("api_backtest_compare_same_assumptions");
    let mut payload = common::sample_runtime_request();
    payload["backtest_options"] = json!({
        "replay_source": "deterministic_mock"
    });

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtest")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    let first_body = to_bytes(first.into_body(), usize::MAX).await.unwrap();
    let first_created: Value = serde_json::from_slice(&first_body).unwrap();

    let second = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtest")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::OK);
    let second_body = to_bytes(second.into_body(), usize::MAX).await.unwrap();
    let second_created: Value = serde_json::from_slice(&second_body).unwrap();

    let compare = app
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtests/compare")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "backtest_ids": [
                            first_created["backtest_id"].clone(),
                            second_created["backtest_id"].clone()
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(compare.status(), StatusCode::OK);
    let compare_body = to_bytes(compare.into_body(), usize::MAX).await.unwrap();
    let compared: Value = serde_json::from_slice(&compare_body).unwrap();
    let first_equity_summary = summarize_equity_curve_value(
        &first_created["backtest_artifacts"]["equity_curve"]["points"],
    );
    let second_equity_summary = summarize_equity_curve_value(
        &second_created["backtest_artifacts"]["equity_curve"]["points"],
    );
    let equity_drilldown = expected_equity_curve_drilldown(
        &first_created["backtest_artifacts"]["equity_curve"]["points"],
        &second_created["backtest_artifacts"]["equity_curve"]["points"],
    );
    let equity_status = compare_equity_curve_status(
        &first_equity_summary,
        &second_equity_summary,
        &equity_drilldown,
    );
    let equity_highlight =
        summarize_equity_curve_highlight_value(&first_equity_summary, &second_equity_summary);
    let report_status = if equity_status == "same" {
        "same"
    } else {
        "different"
    };
    let report_headline = if report_status == "same" {
        "Compared runs share the same execution assumptions, metrics summary, trade ledger summary, and equity curve."
    } else {
        "Compared runs differ across one or more execution/report dimensions."
    };
    let report_bullets = vec![
        "Execution assumptions: same.".to_string(),
        "Metrics summary: same.".to_string(),
        "Trade ledger summary: same.".to_string(),
        format!("Equity curve: {}.", equity_status),
    ];
    let report_highlights = vec![
        "Execution assumptions match on all tracked fields.".to_string(),
        "Metrics summary matches on all tracked fields.".to_string(),
        "Trade ledger summary matches on all tracked fields.".to_string(),
        equity_highlight.clone(),
    ];
    let assumptions_module = expected_execution_assumptions_report_module(
        "same",
        "Execution assumptions match on all tracked fields.",
        "Left resolved values: fee_bps=10, slippage_bps=5, latency_ms=0.",
        "Right resolved values: fee_bps=10, slippage_bps=5, latency_ms=0.",
        vec![
            "Fee source matches on both runs: Backend fallback.".to_string(),
            "Slippage source matches on both runs: Execution profile default.".to_string(),
            "Latency source matches on both runs: Backend fallback.".to_string(),
        ],
    );
    let metrics_drilldown = expected_metrics_drilldown(
        &first_created["backtest_artifacts"]["metrics"]["summary"],
        &second_created["backtest_artifacts"]["metrics"]["summary"],
    );
    let metrics_module = expected_metrics_report_module(
        "same",
        "Metrics summary matches on all tracked fields.",
        &first_created["backtest_artifacts"]["metrics"]["summary"],
        &second_created["backtest_artifacts"]["metrics"]["summary"],
        metrics_drilldown.clone(),
    );
    let trade_ledger_module = expected_trade_ledger_report_module(
        "same",
        "Trade ledger summary matches on all tracked fields.",
        &first_created["backtest_artifacts"]["trade_ledger"]["summary"],
        &second_created["backtest_artifacts"]["trade_ledger"]["summary"],
    );
    let equity_module = expected_equity_curve_report_module(
        equity_status,
        &equity_highlight,
        &first_equity_summary,
        &second_equity_summary,
        equity_drilldown.clone(),
    );
    let (expected_report_narrative, expected_compare_report) =
        expected_compare_outputs_from_modules(
            report_status,
            report_headline,
            report_bullets,
            report_highlights,
            assumptions_module,
            metrics_module,
            trade_ledger_module,
            equity_module,
        );

    assert_eq!(compared["execution_assumptions"]["status"], "same");
    assert_eq!(
        compared["execution_assumptions"]["fields"],
        json!({
            "fee_bps": { "status": "same" },
            "slippage_bps": { "status": "same" },
            "latency_ms": { "status": "same" },
            "sources": { "status": "same" }
        })
    );
    assert_eq!(
        compared["execution_assumptions"]["left"],
        first_created["execution_assumptions"]
    );
    assert_eq!(
        compared["execution_assumptions"]["right"],
        second_created["execution_assumptions"]
    );
    assert_eq!(compared["metrics"]["status"], "same");
    assert_eq!(
        compared["metrics"]["fields"],
        json!({
            "step_count": { "status": "same" },
            "trade_count": { "status": "same" },
            "total_return_ratio": { "status": "same" },
            "max_drawdown_ratio": { "status": "same" },
            "final_equity": { "status": "same" },
            "net_profit": { "status": "same" },
            "turnover_ratio": { "status": "same" },
            "average_trade_notional": { "status": "same" },
            "fee_drag_ratio": { "status": "same" }
        })
    );
    assert_eq!(compared["metrics"]["drilldown"], metrics_drilldown);
    assert_eq!(
        compared["metrics"]["left"],
        first_created["backtest_artifacts"]["metrics"]["summary"]
    );
    assert_eq!(
        compared["metrics"]["right"],
        second_created["backtest_artifacts"]["metrics"]["summary"]
    );
    assert_eq!(compared["trade_ledger"]["status"], "same");
    assert_eq!(
        compared["trade_ledger"]["fields"],
        expected_trade_ledger_fields(
            &first_created["backtest_artifacts"]["trade_ledger"]["summary"],
            &second_created["backtest_artifacts"]["trade_ledger"]["summary"]
        )
    );
    assert_eq!(compared["equity_curve"]["status"], equity_status);
    assert_eq!(
        compared["equity_curve"]["left"],
        first_equity_summary.clone()
    );
    assert_eq!(
        compared["equity_curve"]["right"],
        second_equity_summary.clone()
    );
    assert_eq!(
        compared["equity_curve"]["fields"],
        expected_equity_curve_fields(&first_equity_summary, &second_equity_summary)
    );
    assert_eq!(
        compared["equity_curve"]["drilldown"],
        equity_drilldown.clone()
    );
    assert_eq!(compared["report_narrative"], expected_report_narrative);
    assert_eq!(compared["compare_report"], expected_compare_report);
}

#[tokio::test(flavor = "multi_thread")]
async fn backtest_compare_endpoint_reports_different_execution_assumptions() {
    let app = common::test_app("api_backtest_compare_different_assumptions");
    let mut baseline_payload = common::sample_runtime_request();
    baseline_payload["backtest_options"] = json!({
        "replay_source": "deterministic_mock"
    });

    let baseline = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtest")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(baseline_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(baseline.status(), StatusCode::OK);
    let baseline_body = to_bytes(baseline.into_body(), usize::MAX).await.unwrap();
    let baseline_created: Value = serde_json::from_slice(&baseline_body).unwrap();

    let mut override_payload = common::sample_runtime_request();
    override_payload["backtest_options"] = json!({
        "replay_source": "deterministic_mock",
        "execution_assumptions": {
            "fee_bps": 3.5,
            "slippage_bps": 1.25,
            "latency_ms": 250
        }
    });

    let overridden = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtest")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(override_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(overridden.status(), StatusCode::OK);
    let overridden_body = to_bytes(overridden.into_body(), usize::MAX).await.unwrap();
    let overridden_created: Value = serde_json::from_slice(&overridden_body).unwrap();

    let compare = app
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtests/compare")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "backtest_ids": [
                            baseline_created["backtest_id"].clone(),
                            overridden_created["backtest_id"].clone()
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(compare.status(), StatusCode::OK);
    let compare_body = to_bytes(compare.into_body(), usize::MAX).await.unwrap();
    let compared: Value = serde_json::from_slice(&compare_body).unwrap();
    let baseline_equity_summary = summarize_equity_curve_value(
        &baseline_created["backtest_artifacts"]["equity_curve"]["points"],
    );
    let overridden_equity_summary = summarize_equity_curve_value(
        &overridden_created["backtest_artifacts"]["equity_curve"]["points"],
    );
    let equity_drilldown = expected_equity_curve_drilldown(
        &baseline_created["backtest_artifacts"]["equity_curve"]["points"],
        &overridden_created["backtest_artifacts"]["equity_curve"]["points"],
    );
    let equity_diff_fields =
        differing_equity_curve_fields(&baseline_equity_summary, &overridden_equity_summary);
    let report_bullets = vec![
        "Execution assumptions: different.".to_string(),
        "Metrics summary: same.".to_string(),
        "Trade ledger summary: same.".to_string(),
        "Equity curve: different.".to_string(),
    ];
    let report_highlights = vec![
        "Execution assumptions differ on: fee_bps, slippage_bps, latency_ms, sources.".to_string(),
        "Metrics summary matches on all tracked fields.".to_string(),
        "Trade ledger summary matches on all tracked fields.".to_string(),
        format!(
            "Equity curve differs on: {}.",
            equity_diff_fields.join(", ")
        ),
    ];
    let assumptions_module = expected_execution_assumptions_report_module(
        "different",
        "Execution assumptions differ on: fee_bps, slippage_bps, latency_ms, sources.",
        "Left resolved values: fee_bps=10, slippage_bps=5, latency_ms=0.",
        "Right resolved values: fee_bps=3.5, slippage_bps=1.25, latency_ms=250.",
        vec![
            "Fee source differs: left=Backend fallback, right=Request override.".to_string(),
            "Slippage source differs: left=Execution profile default, right=Request override."
                .to_string(),
            "Latency source differs: left=Backend fallback, right=Request override.".to_string(),
        ],
    );
    let metrics_drilldown = expected_metrics_drilldown(
        &baseline_created["backtest_artifacts"]["metrics"]["summary"],
        &overridden_created["backtest_artifacts"]["metrics"]["summary"],
    );
    let metrics_module = expected_metrics_report_module(
        "same",
        "Metrics summary matches on all tracked fields.",
        &baseline_created["backtest_artifacts"]["metrics"]["summary"],
        &overridden_created["backtest_artifacts"]["metrics"]["summary"],
        metrics_drilldown.clone(),
    );
    let trade_ledger_module = expected_trade_ledger_report_module(
        "same",
        "Trade ledger summary matches on all tracked fields.",
        &baseline_created["backtest_artifacts"]["trade_ledger"]["summary"],
        &overridden_created["backtest_artifacts"]["trade_ledger"]["summary"],
    );
    let equity_module = expected_equity_curve_report_module(
        "different",
        &format!(
            "Equity curve differs on: {}.",
            equity_diff_fields.join(", ")
        ),
        &baseline_equity_summary,
        &overridden_equity_summary,
        equity_drilldown.clone(),
    );
    let (expected_report_narrative, expected_compare_report) =
        expected_compare_outputs_from_modules(
            "different",
            "Compared runs differ across one or more execution/report dimensions.",
            report_bullets,
            report_highlights,
            assumptions_module,
            metrics_module,
            trade_ledger_module,
            equity_module,
        );
    assert_eq!(compared["execution_assumptions"]["status"], "different");
    assert_eq!(
        compared["execution_assumptions"]["fields"],
        json!({
            "fee_bps": { "status": "different" },
            "slippage_bps": { "status": "different" },
            "latency_ms": { "status": "different" },
            "sources": { "status": "different" }
        })
    );
    assert_eq!(
        compared["execution_assumptions"]["left"],
        baseline_created["execution_assumptions"]
    );
    assert_eq!(
        compared["execution_assumptions"]["right"],
        overridden_created["execution_assumptions"]
    );
    assert_eq!(compared["metrics"]["status"], "same");
    assert_eq!(
        compared["metrics"]["fields"],
        json!({
            "step_count": { "status": "same" },
            "trade_count": { "status": "same" },
            "total_return_ratio": { "status": "same" },
            "max_drawdown_ratio": { "status": "same" },
            "final_equity": { "status": "same" },
            "net_profit": { "status": "same" },
            "turnover_ratio": { "status": "same" },
            "average_trade_notional": { "status": "same" },
            "fee_drag_ratio": { "status": "same" }
        })
    );
    assert_eq!(compared["metrics"]["drilldown"], metrics_drilldown);
    assert_eq!(
        compared["metrics"]["left"],
        baseline_created["backtest_artifacts"]["metrics"]["summary"]
    );
    assert_eq!(
        compared["metrics"]["right"],
        overridden_created["backtest_artifacts"]["metrics"]["summary"]
    );
    assert_eq!(compared["trade_ledger"]["status"], "same");
    assert_eq!(
        compared["trade_ledger"]["fields"],
        expected_trade_ledger_fields(
            &baseline_created["backtest_artifacts"]["trade_ledger"]["summary"],
            &overridden_created["backtest_artifacts"]["trade_ledger"]["summary"]
        )
    );
    assert_eq!(compared["equity_curve"]["status"], "different");
    assert_eq!(compared["equity_curve"]["left"], baseline_equity_summary);
    assert_eq!(compared["equity_curve"]["right"], overridden_equity_summary);
    assert_eq!(
        compared["equity_curve"]["fields"],
        expected_equity_curve_fields(&baseline_equity_summary, &overridden_equity_summary)
    );
    assert_eq!(compared["equity_curve"]["drilldown"], equity_drilldown);
    assert_eq!(compared["report_narrative"], expected_report_narrative);
    assert_eq!(compared["compare_report"], expected_compare_report);
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_btc_strategy_compiles_and_backtests_stably() {
    let app = common::test_app("api_backtest_formal_quantscript_btc_dual_ma");
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let compile_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "btc_dual_ma_stability_graph",
                        "compile_id": "btc_dual_ma_stability_compile",
                        "runtime_template": runtime_template,
                        "source": BTC_DUAL_MA_STABILITY,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let compile_status = compile_response.status();
    let compile_body = to_bytes(compile_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(
        compile_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&compile_body)
    );

    let compiled: Value = serde_json::from_slice(&compile_body).unwrap();
    assert_eq!(compiled["graph_id"], "btc_dual_ma_stability_graph");
    assert_eq!(compiled["compile_id"], "btc_dual_ma_stability_compile");
    assert_eq!(compiled["counts"]["data_sources"], 1);
    assert_eq!(compiled["counts"]["intent_generators"], 2);
    assert_eq!(
        compiled["runtime_config"]["data_sources"][0]["config"]["instrument"],
        "BTCUSDT"
    );

    let backtest_response = app
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtest")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "runtime_config": compiled["runtime_config"].clone(),
                        "runtime_targets": compiled["runtime_targets"].clone(),
                        "backtest_options": {
                            "replay_source": "deterministic_mock"
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let backtest_status = backtest_response.status();
    let backtest_body = to_bytes(backtest_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(
        backtest_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&backtest_body)
    );

    let created: Value = serde_json::from_slice(&backtest_body).unwrap();
    assert_eq!(created["graph_id"], "btc_dual_ma_stability_graph");
    assert_eq!(created["compile_id"], "btc_dual_ma_stability_compile");
    assert_eq!(
        created["backtest_artifacts"]["manifest"]["backtest_spec"]["replay_source"],
        "deterministic_mock"
    );
    assert!(created["event_count"].as_u64().unwrap() > 0);
    assert!(created["backtest_artifacts"]["trade_ledger"]["trade_count"].is_number());
    assert!(
        created["backtest_artifacts"]["metrics"]["summary"]["step_count"]
            .as_u64()
            .unwrap()
            > 0
    );
}
