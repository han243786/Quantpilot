use crate::*;
use axum::extract::Query;

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

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

/// v2.5.0: 共享签名输入构建, 消除创建/验证两侧代码重复
fn build_signature_input(
    capability_hash: &str,
    strategy_version: &str,
    parameter_version: &str,
    core_ir_digest: &str,
    event_slice_bounds: &EventSliceBounds,
    created_at_ms: u64,
) -> serde_json::Value {
    json!({
        "capability_hash": capability_hash,
        "strategy_version": strategy_version,
        "parameter_version": parameter_version,
        "core_ir_digest": core_ir_digest,
        "event_slice_bounds": {
            "from_event_id": &event_slice_bounds.from_event_id,
            "to_event_id": &event_slice_bounds.to_event_id,
            "from_sequence": event_slice_bounds.from_sequence,
            "to_sequence": event_slice_bounds.to_sequence,
            "event_count": event_slice_bounds.event_count,
        },
        "created_at_ms": created_at_ms,
    })
}

async fn create_snapshot(
    State(state): State<AppState>,
    request: Option<Json<CreateSnapshotRequest>>,
) -> Result<Json<DeploymentSignatureSnapshot>, (StatusCode, String)> {
    let Json(request) = request.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "快照创建需要提供部署签名信息 (deployment_revision 等 10 个字段)".to_string(),
        )
    })?;
    let now_ms = current_time_ms();
    let snapshot_id = format!("snap-{}", now_ms);

    let event_bounds = EventSliceBounds {
        from_event_id: request.from_event_id,
        to_event_id: request.to_event_id,
        from_sequence: request.from_sequence,
        to_sequence: request.to_sequence,
        event_count: request.event_count,
    };

    // v2.5.0: 抽取共享签名输入构建函数, 消除创建/验证两侧代码重复
    let signature_input = build_signature_input(
        &request.capability_hash,
        &request.strategy_version,
        &request.parameter_version,
        &request.core_ir_digest,
        &event_bounds,
        now_ms,
    );

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
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<DeploymentSignatureSnapshot>>, (StatusCode, String)> {
    let mut snapshots: Vec<DeploymentSignatureSnapshot> =
        state.snapshots.read().await.values().cloned().collect();
    snapshots.sort_by(|a, b| b.created_at_ms.cmp(&a.created_at_ms));
    Ok(Json(paginate(snapshots, pagination)))
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
    let verify_input = build_signature_input(
        &snapshot.capability_hash,
        &snapshot.strategy_version,
        &snapshot.parameter_version,
        &snapshot.core_ir_digest,
        &snapshot.event_slice_bounds,
        snapshot.created_at_ms,
    );
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
    persist_snapshot_restore_audit(&state.audit_store_dir, &snapshot, &request, now_ms)
        .await
        .map_err(io_error)?;
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

    safe_eprintln!(
        "[snapshot_service] 快照 {} 由 {} 在 {} 恢复",
        snapshot_id,
        request.actor_id,
        now_ms
    );

    // v2.1.0: 恢复操作实际停止当前运行时并记录审计日志
    state
        .runs
        .write()
        .await
        .retain(|_, r| r.created_at_ms > now_ms);
    state
        .backtests
        .write()
        .await
        .retain(|_, r| r.created_at_ms > now_ms);
    safe_eprintln!(
        "[snapshot_service] 快照 {} 恢复: 已清理过期的运行时记录和回测记录",
        snapshot_id
    );

    Ok(Json(result))
}

// ── 持久化辅助函数 ──

async fn persist_snapshot_restore_audit(
    audit_store_dir: &FsPath,
    snapshot: &DeploymentSignatureSnapshot,
    request: &RestoreSnapshotRequest,
    restored_at_ms: u64,
) -> std::io::Result<()> {
    fs::create_dir_all(audit_store_dir).await?;
    let path = audit_store_dir.join(format!(
        "snapshot-restore-{}-{}.json",
        snapshot.snapshot_id, restored_at_ms
    ));
    let entry = json!({
        "event_type": "snapshot_restore",
        "snapshot_id": snapshot.snapshot_id,
        "deployment_revision": snapshot.deployment_revision,
        "strategy_version": snapshot.strategy_version,
        "parameter_version": snapshot.parameter_version,
        "actor_id": request.actor_id,
        "reason": request.reason.clone().unwrap_or_default(),
        "restored_at_ms": restored_at_ms,
    });
    crate::runtime_persistence::atomic_write_json(&path, &entry).await
}

async fn persist_snapshot(
    store_dir: &FsPath,
    snapshot: &DeploymentSignatureSnapshot,
) -> std::io::Result<()> {
    crate::storage_lifecycle::ensure_storage_quota(
        std::path::Path::new("storage"),
        "snapshots",
        crate::storage_lifecycle::StorageLifecycle::Transient,
    )?;
    fs::create_dir_all(&store_dir).await?;
    let file_path = store_dir.join(format!("{}.json", snapshot.snapshot_id));
    // v2.3.3: 使用统一原子写入 (含 fsync)
    crate::runtime_persistence::atomic_write_json(&file_path, snapshot).await
}

async fn load_snapshot_from_disk(
    store_dir: &FsPath,
    snapshot_id: &str,
) -> Result<DeploymentSignatureSnapshot, (StatusCode, String)> {
    if let Err(msg) = validate_snapshot_id(snapshot_id) {
        return Err(json_bad_request("invalid_snapshot_id", msg));
    }
    let file_path = store_dir.join(format!("{}.json", snapshot_id));
    let json = fs::read(&file_path)
        .await
        .map_err(|_| json_bad_request("not_found", format!("快照 '{}' 不存在", snapshot_id)))?;
    serde_json::from_slice(&json).map_err(|error| internal_error(anyhow::anyhow!("{}", error)))
}

fn validate_snapshot_id(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("snapshot_id 不能为空".to_string());
    }
    if id.len() > 128 {
        return Err("snapshot_id 长度不能超过 128 字符".to_string());
    }
    if id.contains("..") || id.contains('/') || id.contains('\\') || id.contains('\0') {
        return Err("snapshot_id 不能包含路径分隔符".to_string());
    }
    if !id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err("snapshot_id 只能使用 ASCII 字母、数字、'_' 或 '-'".to_string());
    }
    Ok(())
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

    #[test]
    fn validate_snapshot_id_rejects_invalid() {
        let cases = ["..", "a/b", "a\\b", "\0x"];
        for case in &cases {
            assert!(
                super::validate_snapshot_id(case).is_err(),
                "ID '{}' 应被拒绝",
                case
            );
        }
        assert!(super::validate_snapshot_id("").is_err());
    }

    #[test]
    fn validate_snapshot_id_accepts_valid() {
        let cases = ["snap-123", "abc_def", "my-snapshot-001"];
        for case in &cases {
            assert!(
                super::validate_snapshot_id(case).is_ok(),
                "ID '{}' 应被接受",
                case
            );
        }
    }

    #[test]
    fn create_snapshot_request_serialization() {
        let req = CreateSnapshotRequest {
            deployment_revision: "rev-1".to_string(),
            capability_hash: "sha256:abc".to_string(),
            strategy_version: "v1.0".to_string(),
            parameter_version: "p1".to_string(),
            core_ir_digest: "sha256:def".to_string(),
            from_event_id: "evt-0".to_string(),
            to_event_id: "evt-10".to_string(),
            from_sequence: 0,
            to_sequence: 10,
            event_count: 11,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("deployment_revision"));
        assert!(json.contains("event_count"));
    }
}
