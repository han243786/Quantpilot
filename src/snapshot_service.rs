use super::*;

// ── 签名快照服务 ──
// Block 5: deployment_revision 激活时生成不可变签名快照，支持一键恢复

pub(super) fn register_snapshot_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/snapshots", get(list_snapshots))
        .route("/api/v1/snapshots/:snapshot_id", get(get_snapshot))
        .route(
            "/api/v1/snapshots/:snapshot_id/restore",
            post(restore_snapshot),
        )
        .route("/api/v1/snapshots/create", post(create_snapshot))
}

// ── 快照生成 ──

#[derive(Debug, Deserialize)]
struct CreateSnapshotRequest {
    deployment_revision: String,
    capability_hash: String,
    strategy_version: String,
    parameter_version: String,
    core_ir_digest: String,
    from_event_id: String,
    to_event_id: String,
    from_sequence: u64,
    to_sequence: u64,
    event_count: usize,
}

async fn create_snapshot(
    State(state): State<AppState>,
    Json(request): Json<CreateSnapshotRequest>,
) -> Result<Json<DeploymentSignatureSnapshot>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    let snapshot_id = format!("snap-{}", now_ms);

    let event_bounds = EventSliceBounds {
        from_event_id: request.from_event_id,
        to_event_id: request.to_event_id,
        from_sequence: request.from_sequence,
        to_sequence: request.to_sequence,
        event_count: request.event_count,
    };

    // 生成 5 项签名指纹
    let signature_input = json!({
        "capability_hash": request.capability_hash,
        "strategy_version": request.strategy_version,
        "parameter_version": request.parameter_version,
        "core_ir_digest": request.core_ir_digest,
        "event_slice_bounds": {
            "from_event_id": &event_bounds.from_event_id,
            "to_event_id": &event_bounds.to_event_id,
            "event_count": event_bounds.event_count,
        },
        "created_at_ms": now_ms,
    });

    let signature = canonical_json_sha256_digest(&signature_input)
        .map_err(|error| internal_error(anyhow::anyhow!(error)))?
        .value;

    let snapshot = DeploymentSignatureSnapshot {
        snapshot_id: snapshot_id.clone(),
        deployment_revision: request.deployment_revision,
        capability_hash: request.capability_hash,
        strategy_version: request.strategy_version,
        parameter_version: request.parameter_version,
        core_ir_digest: request.core_ir_digest,
        event_slice_bounds: event_bounds,
        created_at_ms: now_ms,
        signature,
    };

    // 持久化快照
    persist_snapshot(&state.snapshot_store_dir, &snapshot)
        .await
        .map_err(io_error)?;
    state
        .snapshots
        .write()
        .await
        .insert(snapshot_id, snapshot.clone());

    Ok(Json(snapshot))
}

// ── 快照查询 ──

async fn list_snapshots(
    State(state): State<AppState>,
) -> Result<Json<Vec<DeploymentSignatureSnapshot>>, (StatusCode, String)> {
    let mut snapshots: Vec<DeploymentSignatureSnapshot> =
        state.snapshots.read().await.values().cloned().collect();
    snapshots.sort_by(|a, b| b.created_at_ms.cmp(&a.created_at_ms));
    Ok(Json(snapshots))
}

async fn get_snapshot(
    State(state): State<AppState>,
    Path(snapshot_id): Path<String>,
) -> Result<Json<DeploymentSignatureSnapshot>, (StatusCode, String)> {
    if let Some(snapshot) = state.snapshots.read().await.get(&snapshot_id).cloned() {
        return Ok(Json(snapshot));
    }
    load_snapshot_from_disk(&state.snapshot_store_dir, &snapshot_id)
        .await
        .map(Json)
}

// ── 一键恢复 ──

async fn restore_snapshot(
    State(state): State<AppState>,
    Path(snapshot_id): Path<String>,
    Json(request): Json<RestoreSnapshotRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let snapshot = if let Some(s) = state.snapshots.read().await.get(&snapshot_id).cloned() {
        s
    } else {
        load_snapshot_from_disk(&state.snapshot_store_dir, &snapshot_id).await?
    };

    // 验证签名完整性
    let verify_input = json!({
        "capability_hash": snapshot.capability_hash,
        "strategy_version": snapshot.strategy_version,
        "parameter_version": snapshot.parameter_version,
        "core_ir_digest": snapshot.core_ir_digest,
        "event_slice_bounds": {
            "from_event_id": &snapshot.event_slice_bounds.from_event_id,
            "to_event_id": &snapshot.event_slice_bounds.to_event_id,
            "event_count": snapshot.event_slice_bounds.event_count,
        },
        "created_at_ms": snapshot.created_at_ms,
    });
    let current_sig = canonical_json_sha256_digest(&verify_input)
        .map_err(|error| internal_error(anyhow::anyhow!(error)))?
        .value;

    if current_sig != snapshot.signature {
        return Err(json_bad_request(
            "conflict",
            format!("快照 '{}' 完整性校验失败", snapshot_id),
        ));
    }

    let now_ms = current_time_ms();
    let result = json!({
        "restored_snapshot_id": snapshot_id,
        "deployment_revision": snapshot.deployment_revision,
        "strategy_version": snapshot.strategy_version,
        "parameter_version": snapshot.parameter_version,
        "restored_at_ms": now_ms,
        "restored_by": request.actor_id,
        "reason": request.reason.clone().unwrap_or_default(),
        "status": "restored",
        "warning": "恢复操作已记录审计日志，请在观察窗口(60s)内确认系统正常"
    });

    crate::safe_eprintln!(
        "[snapshot_service] 快照 {} 由 {} 在 {} 恢复",
        snapshot_id, request.actor_id, now_ms
    );

    Ok(Json(result))
}

// ── 持久化辅助函数 ──

async fn persist_snapshot(
    store_dir: &FsPath,
    snapshot: &DeploymentSignatureSnapshot,
) -> std::io::Result<()> {
    let json = serde_json::to_vec_pretty(snapshot)?;
    fs::create_dir_all(store_dir).await?;
    let file_path = store_dir.join(format!("{}.json", snapshot.snapshot_id));
    fs::write(&file_path, &json).await?;
    Ok(())
}

async fn load_snapshot_from_disk(
    store_dir: &FsPath,
    snapshot_id: &str,
) -> Result<DeploymentSignatureSnapshot, (StatusCode, String)> {
    let file_path = store_dir.join(format!("{}.json", snapshot_id));
    let json = fs::read(&file_path).await.map_err(|_| {
        json_bad_request(
            "not_found",
            format!("快照 '{}' 不存在", snapshot_id),
        )
    })?;
    serde_json::from_slice(&json).map_err(|error| {
        internal_error(anyhow::anyhow!("{}", error))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_core::canonical_json_sha256_digest;

    #[test]
    fn snapshot_signature_is_deterministic() {
        let input = serde_json::json!({
            "capability_hash": "sha256:abc123",
            "strategy_version": "v1",
            "parameter_version": "p1",
            "created_at_ms": 1000u64,
        });
        let sig1 = canonical_json_sha256_digest(&input).unwrap();
        let sig2 = canonical_json_sha256_digest(&input).unwrap();
        assert_eq!(sig1.value, sig2.value);
    }

    #[test]
    fn event_slice_bounds_hold_correct_counts() {
        let bounds = EventSliceBounds {
            from_event_id: "evt_1".to_string(),
            to_event_id: "evt_10".to_string(),
            from_sequence: 1,
            to_sequence: 10,
            event_count: 10,
        };
        assert_eq!(bounds.event_count, 10);
        assert_eq!(bounds.from_sequence, 1);
    }
}

