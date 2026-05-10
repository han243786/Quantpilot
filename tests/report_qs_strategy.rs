mod common;
include!("common/re_exports.rs");

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

const REPORT_SMA20_100_DAILY_BTC: &str = include_str!("fixtures/report_sma20_100_daily_btc.qs");

const DONCHIAN_STYLE_SOURCE: &str = r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let upper = closes[55..].max()
    let lower = closes[20..].min()

    if closes.last() > upper {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else if closes.last() < lower {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#;

#[tokio::test(flavor = "multi_thread")]
async fn report_sma20_100_strategy_compiles_via_formal_quantscript_endpoint() {
    let app = common::test_app("report_sma20_100_strategy_compiles");
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "report_sma20_100_daily_btc",
                        "compile_id": "compile_report_sma20_100_daily_btc",
                        "runtime_template": runtime_template,
                        "source": REPORT_SMA20_100_DAILY_BTC,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));

    let compiled: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(compiled["compilable"], true);
    assert_eq!(compiled["counts"]["data_sources"], 1);
    assert_eq!(compiled["counts"]["intent_generators"], 2);
    assert_eq!(
        compiled["runtime_config"]["intent_generators"][0]["module_key"],
        "builtin.intent.double_ma"
    );
    assert_eq!(
        compiled["runtime_config"]["intent_generators"][1]["module_key"],
        "builtin.intent.ma_deviation"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_source_cannot_be_parsed_as_strategy_graph_source() {
    let app = common::test_app("report_sma20_100_graph_parse_reject");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/graph/parse")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "source": REPORT_SMA20_100_DAILY_BTC,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let value: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(value["error"], "bad_request");
    assert!(value["message"]
        .as_str()
        .unwrap_or_default()
        .contains("strategy_graph"));
}

#[tokio::test(flavor = "multi_thread")]
async fn donchian_style_report_strategy_is_rejected_by_current_formal_lowering() {
    let app = common::test_app("report_donchian_strategy_reject");
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "report_donchian55_20_daily_btc",
                        "compile_id": "compile_report_donchian55_20_daily_btc",
                        "runtime_template": runtime_template,
                        "source": DONCHIAN_STYLE_SOURCE,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "{}",
        String::from_utf8_lossy(&body)
    );

    let value: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert!(value["details"][0]["code"] == "QPQSLOW001");
    assert!(value["details"][0]["message"]
        .as_str()
        .unwrap_or_default()
        .contains("不支持的条件下发 Intent 下层转换"));
    assert!(value["details"][0]["reason"]
        .as_str()
        .unwrap_or_default()
        .contains("将条件下发重写为支持的指标或价差意图"));
}
