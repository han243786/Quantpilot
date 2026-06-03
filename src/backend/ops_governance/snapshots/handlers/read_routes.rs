use super::load_snapshot_from_disk;
use crate::*;
use axum::extract::Query;

// ── 快照查询 ──

pub(super) async fn list_snapshots(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<DeploymentSignatureSnapshot>>, (StatusCode, String)> {
    let mut snapshots: Vec<DeploymentSignatureSnapshot> =
        state.snapshots.read().await.values().cloned().collect();
    snapshots.sort_by(|a, b| b.created_at_ms.cmp(&a.created_at_ms));
    Ok(Json(paginate(snapshots, pagination)))
}

pub(super) async fn get_snapshot(
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
