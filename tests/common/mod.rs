use axum::Router;
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) mod backend {
    include!("../../src/main.rs");

    pub fn test_app_router(
        graph_store_dir: std::path::PathBuf,
        run_store_dir: std::path::PathBuf,
        backtest_store_dir: std::path::PathBuf,
    ) -> axum::Router {
        build_app_router(new_app_state(
            graph_store_dir,
            run_store_dir,
            backtest_store_dir,
        ))
    }
}

static NEXT_TEST_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
pub struct TestAppDirs {
    pub graph_store_dir: PathBuf,
    pub run_store_dir: PathBuf,
    pub backtest_store_dir: PathBuf,
}

#[allow(dead_code)]
pub fn sample_runtime_request() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/runtime/minimal_runtime_request.json"
    ))
    .expect("runtime request fixture should be valid json")
}

pub fn test_app(test_name: &str) -> Router {
    let (router, _) = test_app_with_dirs(test_name);
    router
}

pub fn test_app_with_dirs(test_name: &str) -> (Router, TestAppDirs) {
    let base_dir = unique_test_dir(test_name);
    let dirs = TestAppDirs {
        graph_store_dir: base_dir.join("graphs"),
        run_store_dir: base_dir.join("runs"),
        backtest_store_dir: base_dir.join("backtests"),
    };

    create_test_app_dirs(&dirs);

    (test_app_from_dirs(dirs.clone()), dirs)
}

pub fn test_app_from_dirs(dirs: TestAppDirs) -> Router {
    create_test_app_dirs(&dirs);
    backend::test_app_router(
        dirs.graph_store_dir,
        dirs.run_store_dir,
        dirs.backtest_store_dir,
    )
}

fn create_test_app_dirs(dirs: &TestAppDirs) {
    fs::create_dir_all(&dirs.graph_store_dir).expect("graph store dir should be created");
    fs::create_dir_all(&dirs.run_store_dir).expect("run store dir should be created");
    fs::create_dir_all(&dirs.backtest_store_dir).expect("backtest store dir should be created");
}

fn unique_test_dir(test_name: &str) -> PathBuf {
    let sequence = NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("test-artifacts")
        .join(format!("{test_name}_{timestamp}_{sequence}"))
}

// v1.0.1: 共享测试辅助 — 校验事件信封完整性
#[allow(dead_code)]
pub(crate) fn assert_complete_event_envelopes(
    events: &[serde_json::Value],
    record_id: &str,
    governance: &serde_json::Value,
) {
    for (index, event) in events.iter().enumerate() {
        let envelope = &event["envelope"];
        assert_eq!(envelope["event_id"], event["event_id"]);
        assert_eq!(envelope["event_type"], event["event_type"]);
        assert_eq!(envelope["run_id"], record_id);
        assert_eq!(envelope["sequence_no"], serde_json::Value::from(index as u64 + 1));
        assert_eq!(envelope["occurred_at_ms"], event["event_time_ms"]);
        assert_eq!(envelope["capability_hash"], governance["capability_hash"]);
        assert_eq!(envelope["deployment_revision"], governance["deployment_revision"]);
        for key in [
            "event_id", "event_type", "run_id", "stage", "strategy_version",
            "parameter_version", "deployment_revision", "capability_hash",
            "mode", "severity", "retention_class",
        ] {
            assert!(
                envelope[key].as_str().map(|v| !v.trim().is_empty()).unwrap_or(false),
                "event {} has empty envelope field {key}", event["event_id"]
            );
        }
    }
}
