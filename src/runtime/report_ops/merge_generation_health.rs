use crate::runtime::{MergeRecordEntry, MergeRecordsResponse};
use crate::{auth, AppState};
use axum::{extract::State, http::StatusCode, Json};

pub(crate) async fn list_merge_records(
    user_id: auth::UserId,
    State(state): State<AppState>,
) -> Result<Json<MergeRecordsResponse>, (StatusCode, String)> {
    // 从最近的 run/backtest 中提取合并事件
    let prefix = auth::scoped_key(&user_id, "");
    let runs = state.runs.read().await;
    let mut entries = Vec::new();
    let mut total_conflicts = 0usize;
    let mut total_suppressed = 0usize;

    for run in runs
        .iter()
        .filter(|(k, _)| k.starts_with(&prefix))
        .map(|(_, v)| v)
    {
        for event in &run.events {
            if event.source_id == "merge_engine" {
                if let Some(payload) = event.payload.as_object() {
                    entries.push(MergeRecordEntry {
                        cycle_name: run.run_id.clone(),
                        input_count: payload
                            .get("input_count")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(1) as usize,
                        output_count: payload
                            .get("output_count")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(1) as usize,
                        conflicts: payload
                            .get("conflicts")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as usize,
                        suppressed: payload
                            .get("suppressed")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as usize,
                        merge_policy: payload
                            .get("merge_policy")
                            .and_then(|v| v.as_str())
                            .unwrap_or("WeightedMerge")
                            .to_string(),
                    });
                    total_conflicts += entries.last().map(|e| e.conflicts).unwrap_or(0);
                    total_suppressed += entries.last().map(|e| e.suppressed).unwrap_or(0);
                }
            }
        }
    }

    Ok(Json(MergeRecordsResponse {
        records: entries,
        total_conflicts,
        total_suppressed,
    }))
}

// ── Block 5 P3-2: 配置代际 API ──

pub(crate) async fn list_config_generations(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let gen = state
        .config_generation
        .load(std::sync::atomic::Ordering::Relaxed);
    let history: Vec<serde_json::Value> = state
        .config_generation_history
        .lock()
        .await
        .iter()
        .map(|entry| {
            serde_json::json!({
                "generation": entry.generation,
                "activated_at_ms": entry.activated_at_ms,
                "deployment_revision": entry.deployment_revision,
                "parameter_version": entry.parameter_version,
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(serde_json::json!({
        "current_generation": gen,
        "history": history,
    })))
}

// ── Block 5 P3-5: 存储健康 API ──

pub(crate) async fn get_storage_health(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let dirs = [
        ("runs", state.run_store_dir.as_ref()),
        ("backtests", state.backtest_store_dir.as_ref()),
        ("reports", state.report_store_dir.as_ref()),
        ("approvals", state.approval_store_dir.as_ref()),
        ("snapshots", state.snapshot_store_dir.as_ref()),
        ("alerts", state.alert_store_dir.as_ref()),
        ("sandbox_reports", state.sandbox_report_store_dir.as_ref()),
        ("chaos", state.chaos_store_dir.as_ref()),
    ];

    let mut layers = Vec::new();
    let mut total_mb = 0u64;

    for (name, dir) in &dirs {
        let size = crate::storage_lifecycle::dir_size_bytes(dir);
        total_mb += size / (1024 * 1024);
        layers.push(serde_json::json!({
            "name": name,
            "size_bytes": size,
            "size_mb": size as f64 / (1024.0 * 1024.0),
        }));
    }

    Ok(Json(serde_json::json!({
        "total_storage_mb": total_mb,
        "layers": layers,
        "hot_layer_usage_ratio": if total_mb > 0 { (total_mb as f64 / 1024.0).min(1.0) } else { 0.0 },
        "disk_watermark_ratio": if total_mb > 900 { 0.90 } else { total_mb as f64 / 1000.0 },
        "archive_enabled": true,
    })))
}
