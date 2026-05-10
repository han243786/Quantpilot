mod common;
include!("common/re_exports.rs");

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use qrpc_compiler::compile_runtime_protocol_config;
use quantscript::{analyze_formal_quant_script, parse_formal_quant_script_config};
use serde_json::Value;
use std::{fs, path::Path};
use tower::ServiceExt;

struct StrategySample<'a> {
    name: &'a str,
    path: &'a str,
    expected_effective_kinds: &'a [&'a str],
    expected_mixed: bool,
}

const STRATEGY_SAMPLES: &[StrategySample<'_>] = &[
    StrategySample {
        name: "ma_trend_profiles",
        path: "quantscript/authoring_samples/ma_trend_profiles.qs",
        expected_effective_kinds: &["risk", "execution", "data", "intent"],
        expected_mixed: false,
    },
    StrategySample {
        name: "rsi_reversion_profiles",
        path: "quantscript/authoring_samples/rsi_reversion_profiles.qs",
        expected_effective_kinds: &["risk", "execution", "data", "intent"],
        expected_mixed: false,
    },
    StrategySample {
        name: "spread_bps_observer_profiles",
        path: "quantscript/authoring_samples/spread_bps_observer_profiles.qs",
        expected_effective_kinds: &["risk", "execution", "data", "intent"],
        expected_mixed: false,
    },
    StrategySample {
        name: "equal_weight_rotation_profiles",
        path: "quantscript/authoring_samples/equal_weight_rotation_profiles.qs",
        expected_effective_kinds: &["risk", "execution", "agent"],
        expected_mixed: true,
    },
];

async fn compile_formal_sample_via_api(
    source: &str,
    compile_id: &str,
    universe_snapshot: Option<Value>,
) -> (StatusCode, Value) {
    let app = common::test_app(compile_id);
    let mut payload = serde_json::json!({
        "graph_id": format!("graph_{compile_id}"),
        "compile_id": compile_id,
        "runtime_template": common::sample_runtime_request()["runtime_config"].clone(),
        "source": source,
    });
    if let Some(snapshot) = universe_snapshot {
        payload["universe_snapshot"] = snapshot;
    }

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: Value = serde_json::from_slice(&body).unwrap();
    (status, value)
}

fn read_sample(path: &str) -> String {
    fs::read_to_string(Path::new(path))
        .unwrap_or_else(|err| panic!("failed to read sample {path}: {err}"))
}

fn trial_universe_snapshot() -> Value {
    serde_json::json!({
        "snapshot_id": "authoring_trial_universe_snapshot",
        "as_of_ms": 1_710_000_000_000u64,
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
                "enabled": true
            },
            {
                "symbol": "SOLUSDT",
                "exchange": "Binance",
                "market_type": "Spot",
                "quote": "USDT",
                "market_cap": 120_000_000_000.0,
                "volume_24h": 4_000_000_000.0,
                "listed_at_ms": 1_520_000_000_000u64,
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

#[tokio::test]
async fn retained_v1_authoring_samples_analyze_compile_and_emit_authoring_view() {
    for sample in STRATEGY_SAMPLES {
        let source = read_sample(sample.path);

        let analysis = analyze_formal_quant_script(&source)
            .unwrap_or_else(|err| panic!("analysis failed for {}: {err}", sample.name));
        assert!(
            !analysis.has_errors(),
            "analysis diagnostics for {} should not contain errors: {:#?}",
            sample.name,
            analysis.diagnostics
        );

        let config = parse_formal_quant_script_config(&source)
            .unwrap_or_else(|err| panic!("config lowering failed for {}: {err}", sample.name));
        compile_runtime_protocol_config(&config)
            .unwrap_or_else(|err| panic!("runtime compilation failed for {}: {err}", sample.name));

        let (status, value) =
            compile_formal_sample_via_api(&source, &format!("real_strategy_{}", sample.name), None)
                .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "formal compile endpoint should succeed for {}: {value:#}",
            sample.name
        );

        let authoring_view =
            &value["artifacts"]["strategy"]["metadata"]["quantscript_authoring_view"];
        assert_eq!(
            authoring_view["kind"], "quantscript_authoring_view",
            "authoring view kind should be present for {}",
            sample.name
        );
        assert!(
            authoring_view["sections"]
                .as_array()
                .map(|sections| !sections.is_empty())
                .unwrap_or(false),
            "authoring view should emit non-empty sections for {}",
            sample.name
        );

        let effective_kinds: Vec<&str> = authoring_view["sections"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|section| section["effective_kind"].as_str())
            .collect();
        for expected_kind in sample.expected_effective_kinds {
            assert!(
                effective_kinds.iter().any(|kind| kind == expected_kind),
                "authoring view for {} should include effective kind `{}` but got {:?}",
                sample.name,
                expected_kind,
                effective_kinds
            );
        }
        if sample.expected_mixed {
            assert!(
                effective_kinds.iter().any(|kind| kind == &"mixed"),
                "authoring view for {} should expose current mixed section boundary but got {:?}",
                sample.name,
                effective_kinds
            );
        }

        assert!(
            value["runtime_config"]["intent_generators"]
                .as_array()
                .map(|intents| !intents.is_empty())
                .unwrap_or(false),
            "runtime config should contain at least one intent generator for {}",
            sample.name
        );
    }
}

#[tokio::test]
async fn cross_sectional_metadata_rotation_compiles_with_universe_snapshot() {
    let source =
        read_sample("quantscript/authoring_samples/universe_metadata_rotation_rank_weight.qs");
    let (status, value) = compile_formal_sample_via_api(
        &source,
        "cross_sectional_metadata_rotation",
        Some(trial_universe_snapshot()),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "cross-sectional metadata rotation should compile successfully: {value:#}"
    );
    assert_eq!(value["compilable"], true);
    assert_eq!(
        value["artifacts"]["core_ir"]["core_ir"]["agent_policies"][0]["kind"],
        "portfolio_rebalance"
    );
    assert_eq!(
        value["artifacts"]["strategy"]["metadata"]["quantscript_authoring_view"]["kind"],
        "quantscript_authoring_view"
    );
}

#[tokio::test]
async fn cross_sectional_metadata_signal_gated_rotation_compiles_with_universe_snapshot() {
    let source =
        read_sample("quantscript/authoring_samples/universe_metadata_signal_gated_rotation.qs");
    let (status, value) = compile_formal_sample_via_api(
        &source,
        "cross_sectional_metadata_signal_gated_rotation",
        Some(trial_universe_snapshot()),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "cross-sectional metadata signal-gated rotation should compile successfully: {value:#}"
    );
    assert_eq!(value["compilable"], true);
    assert_eq!(
        value["artifacts"]["core_ir"]["core_ir"]["agent_policies"][0]["kind"],
        "portfolio_rebalance"
    );
    assert_eq!(
        value["artifacts"]["strategy"]["metadata"]["quantscript_authoring_view"]["kind"],
        "quantscript_authoring_view"
    );
}

#[tokio::test]
async fn cross_sectional_factor_ranking_hits_first_retained_sort_key_boundary() {
    let source = read_sample("quantscript/boundary_samples/factor_rank_rotation_boundary.qs");
    let (status, value) = compile_formal_sample_via_api(
        &source,
        "cross_sectional_factor_rank_boundary",
        Some(trial_universe_snapshot()),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(value["error"], "quantscript_lowering_failed");
    let details = value["details"]
        .as_array()
        .expect("details should be an array");
    assert!(
        details.iter().any(|detail| detail["code"] == "QPQSLOW011"),
        "expected QPQSLOW011 unsupported universe sort key diagnostic, got {value:#}"
    );
}
