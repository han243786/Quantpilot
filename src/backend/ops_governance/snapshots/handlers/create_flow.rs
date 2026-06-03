use super::{build_signature_input, persist_snapshot};
use crate::*;

// ── 快照生成 ──

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CreateSnapshotRequest {
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

pub(super) async fn create_snapshot(
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

#[cfg(test)]
mod tests {
    use super::*;

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
