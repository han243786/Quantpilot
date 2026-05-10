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
