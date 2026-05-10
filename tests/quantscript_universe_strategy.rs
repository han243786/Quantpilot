mod common;
include!("common/re_exports.rs");

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

const UNIVERSE_TOP2_SOURCE: &str = r#"
fn strategy() {
    let base = universe(exchange="binance", market="spot", quote="USDT")
    let selected = top(sort_by(base, key="market_cap", order="desc"), 2)

    for s in selected {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_POINT_IN_TIME_FILTER_SOURCE: &str = r#"
fn strategy() {
    let base = universe(exchange="binance", market="spot", quote="USDT")
    let liquid = filter(base, min_volume_24h=1000000000, min_listing_age_days=100)
    let selected = top(sort_by(liquid, key="market_cap", order="desc"), 2)

    for s in selected {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_SORT_ORDER_SOURCE: &str = r#"
fn strategy() {
    let base = universe(exchange="binance", market="spot", quote="USDT")
    let selected = top(sort_by(base, key="market_cap", order="sideways"), 2)

    for s in selected {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_REBALANCE_FREQUENCY_SOURCE: &str = r#"
fn strategy() {
    let base = universe(exchange="binance", market="spot", quote="USDT")
    rebalance(equal_weight(base), every="monthly")

    for s in base {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_REBALANCE_ALLOCATION_FORM_SOURCE: &str = r#"
fn strategy() {
    let base = top(sort_by(universe(exchange="binance", market="spot", quote="USDT"), key="market_cap", order="desc"), 2)
    rebalance(base, every="1d")

    for s in base {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_REBALANCE_MISSING_ALLOCATION_SOURCE: &str = r#"
fn strategy() {
    let base = top(sort_by(universe(exchange="binance", market="spot", quote="USDT"), key="market_cap", order="desc"), 2)
    rebalance(every="1d")

    for s in base {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_REBALANCE_MISSING_EQUAL_WEIGHT_SELECTION_SOURCE: &str = r#"
fn strategy() {
    let base = top(sort_by(universe(exchange="binance", market="spot", quote="USDT"), key="market_cap", order="desc"), 2)
    rebalance(equal_weight(), every="1d")

    for s in base {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_REBALANCE_MISSING_FIXED_WEIGHT_SELECTION_SOURCE: &str = r#"
fn strategy() {
    let base = top(sort_by(universe(exchange="binance", market="spot", quote="USDT"), key="market_cap", order="desc"), 2)
    rebalance(fixed_weights(weights=[1.0, 1.0]), every="1d")

    for s in base {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_REBALANCE_MISSING_RANK_WEIGHT_SELECTION_SOURCE: &str = r#"
fn strategy() {
    let base = top(sort_by(universe(exchange="binance", market="spot", quote="USDT"), key="market_cap", order="desc"), 2)
    rebalance(rank_weight(method="linear"), every="1d")

    for s in base {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_REBALANCE_MISSING_SCORE_WEIGHT_SELECTION_SOURCE: &str = r#"
fn strategy() {
    let base = top(sort_by(universe(exchange="binance", market="spot", quote="USDT"), key="market_cap", order="desc"), 2)
    rebalance(score_weight(normalize="sum"), every="1d")

    for s in base {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_REBALANCE_ALLOCATION_UNIVERSE_SOURCE: &str = r#"
fn strategy() {
    rebalance(equal_weight(1), every="1d")

    let base = top(sort_by(universe(exchange="binance", market="spot", quote="USDT"), key="market_cap", order="desc"), 2)
    for s in base {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_REBALANCE_EMPTY_SELECTION_SOURCE: &str = r#"
fn strategy() {
    let base = symbols([])
    rebalance(equal_weight(base), every="1d")

    for s in base {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_TOP_INPUT_SOURCE: &str = r#"
fn strategy() {
    let selected = top(1, 2)

    for s in selected {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_TOP_COUNT_SOURCE: &str = r#"
fn strategy() {
    let base = universe(exchange="binance", market="spot", quote="USDT")
    let selected = top(base)

    for s in selected {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_TOP_MISSING_INPUT_SOURCE: &str = r#"
fn strategy() {
    let selected = top()

    for s in selected {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_SYMBOLS_LIST_LITERAL_SOURCE: &str = r#"
fn strategy() {
    let selected = symbols("BTCUSDT")

    for s in selected {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_SYMBOLS_ITEM_LITERAL_SOURCE: &str = r#"
fn strategy() {
    let selected = symbols(["BTCUSDT", 1])

    for s in selected {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_SYMBOLS_MISSING_LIST_SOURCE: &str = r#"
fn strategy() {
    let selected = symbols()

    for s in selected {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_FIXED_WEIGHTS_COUNT_SOURCE: &str = r#"
fn strategy() {
    let base = top(sort_by(universe(exchange="binance", market="spot", quote="USDT"), key="market_cap", order="desc"), 2)
    rebalance(fixed_weights(base, weights=[1.0]), every="1d")

    for s in base {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_FIXED_WEIGHTS_MISSING_LITERAL_SOURCE: &str = r#"
fn strategy() {
    let base = top(sort_by(universe(exchange="binance", market="spot", quote="USDT"), key="market_cap", order="desc"), 2)
    rebalance(fixed_weights(base), every="1d")

    for s in base {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_SCORE_NORMALIZE_SOURCE: &str = r#"
fn strategy() {
    let base = top(sort_by(universe(exchange="binance", market="spot", quote="USDT"), key="market_cap", order="desc"), 2)
    rebalance(score_weight(base, normalize="max"), every="1d")

    for s in base {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_FIXED_WEIGHTS_NEGATIVE_SOURCE: &str = r#"
fn strategy() {
    let base = top(sort_by(universe(exchange="binance", market="spot", quote="USDT"), key="market_cap", order="desc"), 2)
    rebalance(fixed_weights(base, weights=[1.0, -1.0]), every="1d")

    for s in base {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_FIXED_WEIGHTS_TOTAL_SOURCE: &str = r#"
fn strategy() {
    let base = top(sort_by(universe(exchange="binance", market="spot", quote="USDT"), key="market_cap", order="desc"), 2)
    rebalance(fixed_weights(base, weights=[0.0, 0.0]), every="1d")

    for s in base {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_RANK_METHOD_SOURCE: &str = r#"
fn strategy() {
    let base = top(sort_by(universe(exchange="binance", market="spot", quote="USDT"), key="market_cap", order="desc"), 2)
    rebalance(rank_weight(base, method="quadratic"), every="1d")

    for s in base {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

const UNIVERSE_BAD_FIXED_WEIGHTS_LITERAL_SOURCE: &str = r#"
fn strategy() {
    let base = top(sort_by(universe(exchange="binance", market="spot", quote="USDT"), key="market_cap", order="desc"), 2)
    rebalance(fixed_weights(base, weights=["heavy", 1.0]), every="1d")

    for s in base {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
"#;

fn sample_universe_snapshot() -> Value {
    json!({
        "snapshot_id": "snapshot_top2_market_cap",
        "as_of_ms": 1_700_000_000_000u64,
        "assets": [
            {
                "symbol": "BTCUSDT",
                "exchange": "Binance",
                "market_type": "Spot",
                "quote": "USDT",
                "market_cap": 1_500_000_000_000.0,
                "enabled": true
            },
            {
                "symbol": "ETHUSDT",
                "exchange": "Binance",
                "market_type": "Spot",
                "quote": "USDT",
                "market_cap": 500_000_000_000.0,
                "enabled": true
            },
            {
                "symbol": "SOLUSDT",
                "exchange": "Binance",
                "market_type": "Spot",
                "quote": "USDT",
                "market_cap": 80_000_000_000.0,
                "enabled": true
            }
        ]
    })
}

fn point_in_time_universe_snapshot(as_of_ms: u64) -> Value {
    json!({
        "snapshot_id": format!("snapshot_point_in_time_{as_of_ms}"),
        "as_of_ms": as_of_ms,
        "assets": [
            {
                "symbol": "BTCUSDT",
                "exchange": "Binance",
                "market_type": "Spot",
                "quote": "USDT",
                "market_cap": 1_500_000_000_000.0,
                "volume_24h": 40_000_000_000.0,
                "listed_at_ms": 1_500_000_000_000u64,
                "enabled": true
            },
            {
                "symbol": "ETHUSDT",
                "exchange": "Binance",
                "market_type": "Spot",
                "quote": "USDT",
                "market_cap": 500_000_000_000.0,
                "volume_24h": 18_000_000_000.0,
                "listed_at_ms": 1_510_000_000_000u64,
                "metadata_history": [
                    {
                        "as_of_ms": 1_700_000_000_000u64,
                        "market_cap": 700_000_000_000.0,
                        "volume_24h": 12_000_000_000.0
                    },
                    {
                        "as_of_ms": 1_710_000_000_000u64,
                        "market_cap": 450_000_000_000.0,
                        "volume_24h": 9_000_000_000.0
                    }
                ],
                "enabled": true
            },
            {
                "symbol": "SOLUSDT",
                "exchange": "Binance",
                "market_type": "Spot",
                "quote": "USDT",
                "listed_at_ms": 1_705_000_000_000u64,
                "metadata_history": [
                    {
                        "as_of_ms": 1_710_000_000_000u64,
                        "market_cap": 600_000_000_000.0,
                        "volume_24h": 4_000_000_000.0
                    }
                ],
                "enabled": true
            },
            {
                "symbol": "DOGEUSDT",
                "exchange": "Binance",
                "market_type": "Spot",
                "quote": "USDT",
                "market_cap": 70_000_000_000.0,
                "volume_24h": 500_000_000.0,
                "listed_at_ms": 1_520_000_000_000u64,
                "enabled": true
            }
        ]
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_expands_top2_market_cap_universe() {
    let app = common::test_app("formal_quantscript_compile_expands_top2_market_cap_universe");
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_top2",
                        "compile_id": "compile_universe_top2",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_TOP2_SOURCE,
                        "universe_snapshot": sample_universe_snapshot(),
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
    assert_eq!(compiled["counts"]["data_sources"], 2);
    assert_eq!(compiled["counts"]["intent_generators"], 4);

    let instruments = compiled["runtime_config"]["data_sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["config"]["instrument"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        instruments,
        vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()]
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_snapshot_dependent_universe_without_snapshot() {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_snapshot_dependent_universe_without_snapshot",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_missing_snapshot",
                        "compile_id": "compile_universe_missing_snapshot",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_TOP2_SOURCE,
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW010");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_unsupported_universe_sort_order_with_structured_diagnostic(
) {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_unsupported_universe_sort_order_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_sort_order",
                        "compile_id": "compile_universe_bad_sort_order",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_SORT_ORDER_SOURCE,
                        "universe_snapshot": sample_universe_snapshot(),
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW012");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_unsupported_rebalance_frequency_with_structured_diagnostic(
) {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_unsupported_rebalance_frequency_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_rebalance_frequency",
                        "compile_id": "compile_universe_bad_rebalance_frequency",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_REBALANCE_FREQUENCY_SOURCE,
                        "universe_snapshot": sample_universe_snapshot(),
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW009");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_non_helper_rebalance_allocation_with_structured_diagnostic(
) {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_non_helper_rebalance_allocation_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_rebalance_allocation_form",
                        "compile_id": "compile_universe_bad_rebalance_allocation_form",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_REBALANCE_ALLOCATION_FORM_SOURCE,
                        "universe_snapshot": sample_universe_snapshot(),
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW013");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_missing_rebalance_allocation_with_structured_diagnostic(
) {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_missing_rebalance_allocation_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_rebalance_missing_allocation",
                        "compile_id": "compile_universe_bad_rebalance_missing_allocation",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_REBALANCE_MISSING_ALLOCATION_SOURCE,
                        "universe_snapshot": sample_universe_snapshot(),
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW013");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_non_universe_rebalance_selection_with_structured_diagnostic(
) {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_non_universe_rebalance_selection_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_rebalance_allocation_universe",
                        "compile_id": "compile_universe_bad_rebalance_allocation_universe",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_REBALANCE_ALLOCATION_UNIVERSE_SOURCE,
                        "universe_snapshot": sample_universe_snapshot(),
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW014");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_missing_equal_weight_selection_with_structured_diagnostic(
) {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_missing_equal_weight_selection_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_missing_equal_weight_selection",
                        "compile_id": "compile_universe_bad_missing_equal_weight_selection",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_REBALANCE_MISSING_EQUAL_WEIGHT_SELECTION_SOURCE,
                        "universe_snapshot": sample_universe_snapshot(),
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW014");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_missing_fixed_weight_selection_with_structured_diagnostic(
) {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_missing_fixed_weight_selection_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_missing_fixed_weight_selection",
                        "compile_id": "compile_universe_bad_missing_fixed_weight_selection",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_REBALANCE_MISSING_FIXED_WEIGHT_SELECTION_SOURCE,
                        "universe_snapshot": sample_universe_snapshot(),
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW014");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_missing_rank_weight_selection_with_structured_diagnostic(
) {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_missing_rank_weight_selection_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_missing_rank_weight_selection",
                        "compile_id": "compile_universe_bad_missing_rank_weight_selection",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_REBALANCE_MISSING_RANK_WEIGHT_SELECTION_SOURCE,
                        "universe_snapshot": sample_universe_snapshot(),
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW014");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_missing_score_weight_selection_with_structured_diagnostic(
) {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_missing_score_weight_selection_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_missing_score_weight_selection",
                        "compile_id": "compile_universe_bad_missing_score_weight_selection",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_REBALANCE_MISSING_SCORE_WEIGHT_SELECTION_SOURCE,
                        "universe_snapshot": sample_universe_snapshot(),
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW014");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_empty_rebalance_selection_with_structured_diagnostic() {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_empty_rebalance_selection_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_rebalance_empty_selection",
                        "compile_id": "compile_universe_bad_rebalance_empty_selection",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_REBALANCE_EMPTY_SELECTION_SOURCE,
                        "universe_snapshot": sample_universe_snapshot(),
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW015");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_non_universe_top_input_with_structured_diagnostic() {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_non_universe_top_input_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_top_input",
                        "compile_id": "compile_universe_bad_top_input",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_TOP_INPUT_SOURCE,
                        "universe_snapshot": sample_universe_snapshot(),
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW025");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_missing_top_count_with_structured_diagnostic() {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_missing_top_count_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_top_count",
                        "compile_id": "compile_universe_bad_top_count",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_TOP_COUNT_SOURCE,
                        "universe_snapshot": sample_universe_snapshot(),
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW028");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_missing_universe_top_input_with_structured_diagnostic()
{
    let app = common::test_app(
        "formal_quantscript_compile_rejects_missing_universe_top_input_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_top_missing_input",
                        "compile_id": "compile_universe_bad_top_missing_input",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_TOP_MISSING_INPUT_SOURCE,
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW025");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_non_list_symbols_literal_with_structured_diagnostic() {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_non_list_symbols_literal_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_symbols_list_literal",
                        "compile_id": "compile_universe_bad_symbols_list_literal",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_SYMBOLS_LIST_LITERAL_SOURCE,
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW026");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_missing_symbols_list_with_structured_diagnostic() {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_missing_symbols_list_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_symbols_missing_list",
                        "compile_id": "compile_universe_bad_symbols_missing_list",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_SYMBOLS_MISSING_LIST_SOURCE,
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW026");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_non_string_symbols_items_with_structured_diagnostic() {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_non_string_symbols_items_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_symbols_item_literal",
                        "compile_id": "compile_universe_bad_symbols_item_literal",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_SYMBOLS_ITEM_LITERAL_SOURCE,
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW027");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_fixed_weights_count_mismatch_with_structured_diagnostic(
) {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_fixed_weights_count_mismatch_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_fixed_weights_count",
                        "compile_id": "compile_universe_bad_fixed_weights_count",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_FIXED_WEIGHTS_COUNT_SOURCE,
                        "universe_snapshot": sample_universe_snapshot(),
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW016");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_missing_fixed_weights_literal_with_structured_diagnostic(
) {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_missing_fixed_weights_literal_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_fixed_weights_missing_literal",
                        "compile_id": "compile_universe_bad_fixed_weights_missing_literal",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_FIXED_WEIGHTS_MISSING_LITERAL_SOURCE,
                        "universe_snapshot": sample_universe_snapshot(),
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW021");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_unsupported_score_weight_normalize_with_structured_diagnostic(
) {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_unsupported_score_weight_normalize_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_score_normalize",
                        "compile_id": "compile_universe_bad_score_normalize",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_SCORE_NORMALIZE_SOURCE,
                        "universe_snapshot": sample_universe_snapshot(),
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW020");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_negative_fixed_weights_with_structured_diagnostic() {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_negative_fixed_weights_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_fixed_weights_negative",
                        "compile_id": "compile_universe_bad_fixed_weights_negative",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_FIXED_WEIGHTS_NEGATIVE_SOURCE,
                        "universe_snapshot": sample_universe_snapshot(),
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW017");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_zero_total_fixed_weights_with_structured_diagnostic() {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_zero_total_fixed_weights_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_fixed_weights_total",
                        "compile_id": "compile_universe_bad_fixed_weights_total",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_FIXED_WEIGHTS_TOTAL_SOURCE,
                        "universe_snapshot": sample_universe_snapshot(),
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW018");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_unsupported_rank_weight_method_with_structured_diagnostic(
) {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_unsupported_rank_weight_method_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_rank_method",
                        "compile_id": "compile_universe_bad_rank_method",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_RANK_METHOD_SOURCE,
                        "universe_snapshot": sample_universe_snapshot(),
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW019");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_rejects_non_numeric_fixed_weights_literal_with_structured_diagnostic(
) {
    let app = common::test_app(
        "formal_quantscript_compile_rejects_non_numeric_fixed_weights_literal_with_structured_diagnostic",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_bad_fixed_weights_literal",
                        "compile_id": "compile_universe_bad_fixed_weights_literal",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_BAD_FIXED_WEIGHTS_LITERAL_SOURCE,
                        "universe_snapshot": sample_universe_snapshot(),
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
    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW021");
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_uses_point_in_time_market_cap_history() {
    let app = common::test_app("formal_quantscript_compile_uses_point_in_time_market_cap_history");
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let early_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_point_in_time_early",
                        "compile_id": "compile_universe_point_in_time_early",
                        "runtime_template": runtime_template.clone(),
                        "source": UNIVERSE_TOP2_SOURCE,
                        "universe_snapshot": point_in_time_universe_snapshot(1_700_000_000_000u64),
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let early_status = early_response.status();
    let early_body = to_bytes(early_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(
        early_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&early_body)
    );
    let early_compiled: Value = serde_json::from_slice(&early_body).unwrap();
    let early_instruments = early_compiled["runtime_config"]["data_sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["config"]["instrument"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        early_instruments,
        vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()]
    );

    let late_response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_point_in_time_late",
                        "compile_id": "compile_universe_point_in_time_late",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_TOP2_SOURCE,
                        "universe_snapshot": point_in_time_universe_snapshot(1_710_000_000_000u64),
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let late_status = late_response.status();
    let late_body = to_bytes(late_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(
        late_status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&late_body)
    );
    let late_compiled: Value = serde_json::from_slice(&late_body).unwrap();
    let late_instruments = late_compiled["runtime_config"]["data_sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["config"]["instrument"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        late_instruments,
        vec!["BTCUSDT".to_string(), "SOLUSDT".to_string()]
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn formal_quantscript_compile_filters_by_point_in_time_volume_and_listing_age() {
    let app = common::test_app(
        "formal_quantscript_compile_filters_by_point_in_time_volume_and_listing_age",
    );
    let runtime_template = common::sample_runtime_request()["runtime_config"].clone();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "graph_id": "graph_universe_point_in_time_filters",
                        "compile_id": "compile_universe_point_in_time_filters",
                        "runtime_template": runtime_template,
                        "source": UNIVERSE_POINT_IN_TIME_FILTER_SOURCE,
                        "universe_snapshot": point_in_time_universe_snapshot(1_710_000_000_000u64),
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
    let instruments = compiled["runtime_config"]["data_sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["config"]["instrument"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        instruments,
        vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()]
    );
}
