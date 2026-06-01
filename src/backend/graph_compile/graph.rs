use crate::*;
use axum::extract::Query;
use std::sync::OnceLock;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/graphs/save", post(save_graph))
        .route("/api/graphs", get(list_graphs))
        .route("/api/graphs/latest", get(load_latest_graph))
        .route("/api/graphs/:graph_id/audit", get(list_graph_audit_history))
        .route("/api/graphs/:graph_id/versions", get(list_graph_versions))
        .route(
            "/api/graphs/:graph_id/versions/compare/:left_version_id/:right_version_id",
            get(compare_graph_versions),
        )
        .route(
            "/api/graphs/:graph_id/versions/:version_id",
            get(load_graph_version),
        )
        .route(
            "/api/graphs/:graph_id/versions/:version_id/restore",
            post(restore_graph_version),
        )
        .route("/api/graphs/:graph_id/reveal", post(reveal_graph_file))
        .route(
            "/api/graphs/:graph_id",
            get(load_graph).delete(delete_graph),
        )
}

fn get_save_lock() -> &'static Mutex<()> {
    static SAVE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    SAVE_LOCK.get_or_init(|| Mutex::new(()))
}

pub(super) async fn save_graph(
    State(state): State<AppState>,
    Json(request): Json<SaveGraphRequest>,
) -> Result<Json<SaveGraphResponse>, (StatusCode, String)> {
    let graph_id = request
        .graph
        .get("metadata")
        .and_then(|item| item.get("graph_id"))
        .and_then(Value::as_str)
        .filter(|item| !item.trim().is_empty())
        .ok_or((
            StatusCode::BAD_REQUEST,
            "graph.metadata.graph_id 是必需的".to_string(),
        ))?;
    validate_graph_id(graph_id).map_err(internal_error)?;
    let graph_id = graph_id.to_string();
    let actor = normalize_actor_identity(request.actor);
    let existing_graph = read_optional_graph_json(&state.graph_store_dir, &graph_id).await?;
    let collaboration = collaboration_with_saved_actor(existing_graph.as_ref(), &actor)?;

    let _guard = get_save_lock().lock().await;
    let response = persist_graph_version(
        &state.graph_store_dir,
        &graph_id,
        &request.graph,
        request.version_label.as_deref(),
        request.save_note.as_deref(),
        collaboration.clone(),
    )
    .await?;
    persist_graph_audit_entry(
        &state.audit_store_dir,
        &build_graph_audit_entry(
            &graph_id,
            &actor,
            GraphAuditAction::GraphSaved,
            Some(response.version_id.clone()),
            format!("Saved graph version {}", response.version_id),
        ),
    )
    .await
    .map_err(io_error)?;
    Ok(Json(response))
}

pub(super) async fn load_latest_graph(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let latest_path = state.graph_store_dir.join("latest.json");
    read_graph_json(&latest_path).await.map(Json)
}

pub(super) async fn list_graphs(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<GraphListEntry>>, (StatusCode, String)> {
    let mut items: Vec<GraphListEntry> = read_graph_index(&state.graph_store_dir).await?;
    items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(Json(paginate(items, pagination)))
}

pub(super) async fn list_graph_versions(
    State(state): State<AppState>,
    Path(graph_id): Path<String>,
) -> Result<Json<Vec<GraphVersionEntry>>, (StatusCode, String)> {
    validate_graph_id(&graph_id).map_err(internal_error)?;
    read_graph_versions(&state.graph_store_dir, &graph_id)
        .await
        .map(Json)
}

pub(super) async fn load_graph_version(
    State(state): State<AppState>,
    Path((graph_id, version_id)): Path<(String, String)>,
) -> Result<Json<Value>, (StatusCode, String)> {
    validate_graph_id(&graph_id).map_err(internal_error)?;
    validate_graph_version_id(&version_id).map_err(internal_error)?;
    let version_path = graph_version_json_path(&state.graph_store_dir, &graph_id, &version_id);
    read_graph_json(&version_path).await.map(Json)
}

pub(super) async fn compare_graph_versions(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path((graph_id, left_version_id, right_version_id)): Path<(String, String, String)>,
    axum::extract::Query(query): axum::extract::Query<GraphVersionCompareQuery>,
) -> Result<Json<GraphVersionCompareResponse>, (StatusCode, String)> {
    validate_graph_id(&graph_id).map_err(internal_error)?;
    validate_graph_version_id(&left_version_id).map_err(internal_error)?;
    validate_graph_version_id(&right_version_id).map_err(internal_error)?;

    let left_path = graph_version_json_path(&state.graph_store_dir, &graph_id, &left_version_id);
    let right_path = graph_version_json_path(&state.graph_store_dir, &graph_id, &right_version_id);
    let left_graph = read_graph_json(&left_path).await?;
    let right_graph = read_graph_json(&right_path).await?;
    let left_qs_source = read_optional_graph_version_quantscript(
        &state.graph_store_dir,
        &graph_id,
        &left_version_id,
    )
    .await?;
    let right_qs_source = read_optional_graph_version_quantscript(
        &state.graph_store_dir,
        &graph_id,
        &right_version_id,
    )
    .await?;
    let versions = read_graph_versions(&state.graph_store_dir, &graph_id).await?;
    let left = versions
        .iter()
        .find(|entry| entry.version_id == left_version_id)
        .cloned()
        .ok_or_else(|| {
            not_found_io_error(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("未找到图版本 `{left_version_id}`"),
            ))
        })?;
    let right = versions
        .iter()
        .find(|entry| entry.version_id == right_version_id)
        .cloned()
        .ok_or_else(|| {
            not_found_io_error(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("未找到图版本 `{right_version_id}`"),
            ))
        })?;

    let mut response = build_graph_version_compare_response(
        &graph_id,
        left.clone(),
        &left_graph,
        right.clone(),
        &right_graph,
    );
    let strategy_config_diff = build_strategy_config_version_diff(
        &graph_id,
        &left,
        &left_graph,
        left_qs_source,
        &right,
        &right_graph,
        right_qs_source,
    )?;
    response.has_changes = response.has_changes || strategy_config_diff.changed;
    response.strategy_config_diff = Some(strategy_config_diff);
    let strategy_config_evidence_diff = build_strategy_config_evidence_diff_for_backtests(
        &state,
        &user_id,
        &graph_id,
        query.left_backtest_id.as_deref(),
        query.right_backtest_id.as_deref(),
    )
    .await;
    response.has_changes = response.has_changes || strategy_config_evidence_diff.changed;
    response.strategy_config_evidence_diff = Some(strategy_config_evidence_diff);

    Ok(Json(response))
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct GraphVersionCompareQuery {
    #[serde(default)]
    left_backtest_id: Option<String>,
    #[serde(default)]
    right_backtest_id: Option<String>,
}

pub(super) async fn restore_graph_version(
    State(state): State<AppState>,
    Path((graph_id, version_id)): Path<(String, String)>,
    request: Option<Json<GraphMutationActorRequest>>,
) -> Result<Json<SaveGraphResponse>, (StatusCode, String)> {
    validate_graph_id(&graph_id).map_err(internal_error)?;
    validate_graph_version_id(&version_id).map_err(internal_error)?;
    let actor = normalize_actor_identity(request.and_then(|Json(body)| body.actor));
    let version_path = graph_version_json_path(&state.graph_store_dir, &graph_id, &version_id);
    let graph = read_graph_json(&version_path).await?;
    let collaboration = collaboration_with_saved_actor(Some(&graph), &actor)?;
    let _save_guard = get_save_lock().lock().await;
    let response = persist_graph_version(
        &state.graph_store_dir,
        &graph_id,
        &graph,
        None,
        None,
        collaboration,
    )
    .await?;
    persist_graph_audit_entry(
        &state.audit_store_dir,
        &build_graph_audit_entry(
            &graph_id,
            &actor,
            GraphAuditAction::GraphRestored,
            Some(response.version_id.clone()),
            format!("Restored graph from persisted version {version_id}"),
        ),
    )
    .await
    .map_err(io_error)?;
    Ok(Json(response))
}

pub(super) async fn list_graph_audit_history(
    State(state): State<AppState>,
    Path(graph_id): Path<String>,
) -> Result<Json<Vec<GraphAuditEntry>>, (StatusCode, String)> {
    validate_graph_id(&graph_id).map_err(internal_error)?;
    load_graph_audit_entries(&state.audit_store_dir, &graph_id)
        .await
        .map(Json)
        .map_err(io_error)
}

pub(super) async fn delete_graph(
    State(state): State<AppState>,
    Path(graph_id): Path<String>,
) -> Result<Json<DeleteGraphResponse>, (StatusCode, String)> {
    let _save_guard = get_save_lock().lock().await;
    validate_graph_id(&graph_id).map_err(internal_error)?;
    let graph_path = state.graph_store_dir.join(format!("{}.json", graph_id));
    if !fs::try_exists(&graph_path).await.map_err(io_error)? {
        // v1.3.7: DELETE幂等 — 已不存在的资源返回200
        return Ok(Json(DeleteGraphResponse {
            graph_id,
            deleted: false,
        }));
    }

    // v1.1.14: 修复审计写入目录 — graph_store_dir→audit_store_dir
    let _ = persist_graph_audit_entry(
        &state.audit_store_dir,
        &GraphAuditEntry {
            audit_id: format!("audit_{}_{}", graph_id, current_time_ms()),
            graph_id: graph_id.clone(),
            action: GraphAuditAction::GraphDeleted,
            created_at_ms: current_time_ms(),
            actor: ActorIdentity {
                actor_id: "system".into(),
                display_name: "系统".into(),
            },
            target_id: None,
            summary: format!("graph_id={} 被删除", graph_id),
        },
    )
    .await;
    remove_file_if_exists(&graph_path).await?;
    remove_file_if_exists(&state.graph_store_dir.join(format!("{}.qs", graph_id))).await?;
    remove_dir_if_exists(&graph_version_dir(&state.graph_store_dir, &graph_id)).await?;
    refresh_latest_graph_after_delete(&state.graph_store_dir, &graph_id).await?;

    Ok(Json(DeleteGraphResponse {
        graph_id,
        deleted: true,
    }))
}

pub(super) async fn reveal_graph_file(
    State(state): State<AppState>,
    Path(graph_id): Path<String>,
) -> Result<Json<RevealGraphResponse>, (StatusCode, String)> {
    validate_graph_id(&graph_id).map_err(internal_error)?;
    let graph_path = state.graph_store_dir.join(format!("{}.json", graph_id));
    if !fs::try_exists(&graph_path).await.map_err(io_error)? {
        return Err(not_found_io_error(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("未找到图 `{}`", graph_id),
        )));
    }

    let reveal_path = resolve_graph_reveal_path(&graph_path)
        .await
        .map_err(internal_error)?;
    reveal_path_in_file_manager(&reveal_path).map_err(internal_error)?;

    Ok(Json(RevealGraphResponse {
        graph_id,
        path: reveal_path.to_string_lossy().to_string(),
    }))
}

pub(super) async fn load_graph(
    State(state): State<AppState>,
    Path(graph_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    validate_graph_id(&graph_id).map_err(internal_error)?;
    let graph_path = state.graph_store_dir.join(format!("{}.json", graph_id));
    read_graph_json(&graph_path).await.map(Json)
}

async fn read_graph_json(path: &FsPath) -> Result<Value, (StatusCode, String)> {
    let content = crate::runtime_persistence::read_to_string_bounded(
        path,
        crate::runtime_persistence::MAX_BOUNDED_JSON_READ_BYTES,
    )
    .await
    .map_err(crate::runtime_persistence::bounded_read_not_found_api_error)?;
    serde_json::from_str(&content).map_err(|error| internal_error(error.into()))
}

#[allow(dead_code)]
async fn atomic_write(path: &FsPath, content: &str) -> Result<(), (StatusCode, String)> {
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, content).await.map_err(io_error)?;
    // fsync tmp 文件确保数据落盘后再 rename
    if let Ok(f) = tokio::fs::File::open(&tmp_path).await {
        let _ = f.sync_all().await;
    }
    fs::rename(&tmp_path, path).await.map_err(io_error)?;
    // fsync 父目录确保 rename 落盘
    if let Some(parent) = path.parent() {
        if let Ok(f) = tokio::fs::File::open(parent).await {
            let _ = f.sync_all().await;
        }
    }
    Ok(())
}

async fn persist_graph_version(
    graph_store_dir: &FsPath,
    graph_id: &str,
    input_graph: &Value,
    version_label: Option<&str>,
    save_note: Option<&str>,
    collaboration: GraphCollaborationMetadata,
) -> Result<SaveGraphResponse, (StatusCode, String)> {
    if let Err(e) = crate::storage_lifecycle::ensure_storage_quota(
        std::path::Path::new("storage"),
        "graphs",
        crate::storage_lifecycle::StorageLifecycle::Permanent,
    ) {
        return Err((StatusCode::INSUFFICIENT_STORAGE, e.to_string()));
    }
    let saved_at = current_time_ms();
    let version_id = saved_at.to_string();
    let graph_path = graph_store_dir.join(format!("{}.json", graph_id));
    let latest_path = graph_store_dir.join("latest.json");
    let quantscript_path = graph_store_dir.join(format!("{}.qs", graph_id));
    let version_dir = graph_version_dir(graph_store_dir, graph_id);
    let version_graph_path = version_dir.join(format!("{}.json", version_id));
    let version_quantscript_path = version_dir.join(format!("{}.qs", version_id));
    let quantscript = if input_graph
        .get("metadata")
        .and_then(|m| m.get("source_mode"))
        .and_then(Value::as_str)
        == Some("quantscript")
    {
        let qs_path = graph_store_dir.join(format!("{}.qs", graph_id));
        if qs_path.exists() {
            crate::runtime_persistence::read_to_string_bounded(
                &qs_path,
                crate::runtime_persistence::MAX_BOUNDED_JSON_READ_BYTES,
            )
            .await
            .unwrap_or_default()
        } else {
            generate_quantscript_from_graph_value(input_graph).map_err(internal_error)?
        }
    } else {
        generate_quantscript_from_graph_value(input_graph).map_err(internal_error)?
    };
    let mut graph = input_graph.clone();
    if let Some(root) = graph.as_object_mut() {
        let metadata = root
            .entry("metadata")
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
        if let Some(metadata) = metadata.as_object_mut() {
            metadata.insert(
                "updated_at".to_string(),
                Value::Number(serde_json::Number::from(saved_at)),
            );
            metadata
                .entry("created_at".to_string())
                .or_insert_with(|| Value::Number(serde_json::Number::from(saved_at)));
            apply_optional_metadata_text(metadata, "version_label", version_label);
            apply_optional_metadata_text(metadata, "save_note", save_note);
        }
    }
    write_graph_collaboration_metadata(&mut graph, &collaboration);
    let persisted_version_label = graph
        .get("metadata")
        .and_then(|item| item.get("version_label"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let persisted_save_note = graph
        .get("metadata")
        .and_then(|item| item.get("save_note"))
        .and_then(Value::as_str)
        .map(str::to_string);
    attach_quantscript_artifacts(&mut graph, &quantscript, saved_at, &quantscript_path);
    let body =
        serde_json::to_string_pretty(&graph).map_err(|error| internal_error(error.into()))?;

    fs::create_dir_all(&version_dir).await.map_err(io_error)?;

    // v2.3.3: 使用临时目录批量准备所有文件，减少不一致窗口
    let staging = graph_store_dir.join(format!(".staging-{}-{}", graph_id, std::process::id()));
    if fs::try_exists(&staging).await.unwrap_or(false) {
        let _ = fs::remove_dir_all(&staging).await;
    }
    fs::create_dir_all(&staging).await.map_err(io_error)?;

    // 在 staging 目录中准备所有文件
    let staging_graph = staging.join(format!("{}.json", graph_id));
    let staging_latest = staging.join("latest.json");
    let staging_qs = staging.join(format!("{}.qs", graph_id));
    let staging_version_dir = staging.join("version");
    let staging_version_graph = staging_version_dir.join(format!("{}.json", version_id));
    let staging_version_qs = staging_version_dir.join(format!("{}.qs", version_id));
    fs::create_dir_all(&staging_version_dir)
        .await
        .map_err(io_error)?;

    let stage_result = async {
        write_synced_staging_file(&staging_graph, body.as_bytes()).await?;
        write_synced_staging_file(&staging_latest, body.as_bytes()).await?;
        write_synced_staging_file(&staging_qs, quantscript.as_bytes()).await?;
        write_synced_staging_file(&staging_version_graph, body.as_bytes()).await?;
        write_synced_staging_file(&staging_version_qs, quantscript.as_bytes()).await?;
        sync_directory_for_graph_commit(&staging)?;
        sync_directory_for_graph_commit(&staging_version_dir)?;
        Ok::<(), (StatusCode, String)>(())
    }
    .await;
    if let Err(error) = stage_result {
        let _ = fs::remove_dir_all(&staging).await;
        return Err(error);
    }

    let commit_items = vec![
        GraphArtifactCommitItem::new(staging_graph, graph_path.clone()),
        GraphArtifactCommitItem::new(staging_latest, latest_path.clone()),
        GraphArtifactCommitItem::new(staging_qs, quantscript_path.clone()),
        GraphArtifactCommitItem::new(staging_version_graph, version_graph_path),
        GraphArtifactCommitItem::new(staging_version_qs, version_quantscript_path),
    ];
    let replacements = match commit_graph_artifact_bundle(
        &commit_items,
        &version_id,
        &[graph_store_dir, &version_dir],
    )
    .await
    {
        Ok(replacements) => replacements,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging).await;
            return Err(error);
        }
    };
    if replacements.is_empty() {
        let _ = fs::remove_dir_all(&staging).await;
    }

    for replacement in &replacements {
        if let Some(backup) = &replacement.backup_path {
            let _ = fs::remove_file(backup).await;
        }
    }
    sync_directory_for_graph_commit(graph_store_dir)?;
    sync_directory_for_graph_commit(&version_dir)?;
    let _ = fs::remove_dir_all(&staging).await;

    Ok(SaveGraphResponse {
        graph_id: graph_id.to_string(),
        version_id,
        saved_at,
        version_label: persisted_version_label,
        save_note: persisted_save_note,
        path: graph_path.to_string_lossy().to_string(),
        quantscript_path: quantscript_path.to_string_lossy().to_string(),
        collaboration,
    })
}

#[derive(Debug)]
struct GraphArtifactReplacement {
    target_path: PathBuf,
    backup_path: Option<PathBuf>,
}

#[derive(Debug)]
struct GraphArtifactCommitItem {
    staged_path: PathBuf,
    target_path: PathBuf,
}

impl GraphArtifactCommitItem {
    fn new(staged_path: PathBuf, target_path: PathBuf) -> Self {
        Self {
            staged_path,
            target_path,
        }
    }
}

async fn commit_graph_artifact_bundle(
    items: &[GraphArtifactCommitItem],
    version_id: &str,
    sync_dirs: &[&FsPath],
) -> Result<Vec<GraphArtifactReplacement>, (StatusCode, String)> {
    for item in items {
        if !fs::try_exists(&item.staged_path).await.map_err(io_error)? {
            return Err(internal_error(anyhow::anyhow!(
                "图工件提交缺少 staging 文件: {}",
                item.staged_path.display()
            )));
        }
    }

    let mut replacements = Vec::new();
    for item in items {
        if let Err(error) = replace_graph_artifact(
            &item.staged_path,
            &item.target_path,
            &mut replacements,
            version_id,
        )
        .await
        {
            rollback_graph_replacements(&replacements).await;
            return Err(error);
        }
    }

    for dir in sync_dirs {
        if let Err(error) = sync_directory_for_graph_commit(dir) {
            rollback_graph_replacements(&replacements).await;
            return Err(error);
        }
    }

    Ok(replacements)
}

async fn write_synced_staging_file(
    path: &FsPath,
    content: &[u8],
) -> Result<(), (StatusCode, String)> {
    let mut file = tokio::fs::File::create(path).await.map_err(io_error)?;
    file.write_all(content).await.map_err(io_error)?;
    file.sync_all().await.map_err(io_error)?;
    Ok(())
}

fn graph_backup_path(
    target_path: &FsPath,
    version_id: &str,
) -> Result<PathBuf, (StatusCode, String)> {
    let parent = target_path.parent().ok_or_else(|| {
        internal_error(anyhow::anyhow!(
            "图工件路径缺少父目录: {}",
            target_path.display()
        ))
    })?;
    let file_name = target_path
        .file_name()
        .and_then(|item| item.to_str())
        .ok_or_else(|| {
            internal_error(anyhow::anyhow!(
                "图工件路径缺少文件名: {}",
                target_path.display()
            ))
        })?;
    Ok(parent.join(format!(".{}.{}.bak", file_name, version_id)))
}

async fn replace_graph_artifact(
    staged_path: &FsPath,
    target_path: &FsPath,
    replacements: &mut Vec<GraphArtifactReplacement>,
    version_id: &str,
) -> Result<(), (StatusCode, String)> {
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).await.map_err(io_error)?;
    }

    let backup_path = if fs::try_exists(target_path).await.map_err(io_error)? {
        let backup = graph_backup_path(target_path, version_id)?;
        remove_file_if_exists(&backup).await?;
        fs::rename(target_path, &backup).await.map_err(io_error)?;
        Some(backup)
    } else {
        None
    };

    if let Err(error) = fs::rename(staged_path, target_path).await {
        if let Some(backup) = &backup_path {
            let _ = fs::rename(backup, target_path).await;
        }
        let _ = fs::remove_file(staged_path).await;
        return Err(io_error(error));
    }

    replacements.push(GraphArtifactReplacement {
        target_path: target_path.to_path_buf(),
        backup_path,
    });
    Ok(())
}

async fn rollback_graph_replacements(replacements: &[GraphArtifactReplacement]) {
    for replacement in replacements.iter().rev() {
        let _ = fs::remove_file(&replacement.target_path).await;
        if let Some(backup) = &replacement.backup_path {
            if fs::try_exists(backup).await.unwrap_or(false) {
                if let Some(parent) = replacement.target_path.parent() {
                    let _ = fs::create_dir_all(parent).await;
                }
                let _ = fs::rename(backup, &replacement.target_path).await;
            }
        }
    }
}

fn sync_directory_for_graph_commit(path: &FsPath) -> Result<(), (StatusCode, String)> {
    crate::storage_lifecycle::sync_directory(path).map_err(io_error)
}

async fn read_optional_graph_json(
    graph_store_dir: &FsPath,
    graph_id: &str,
) -> Result<Option<Value>, (StatusCode, String)> {
    let graph_path = graph_store_dir.join(format!("{}.json", graph_id));
    if !fs::try_exists(&graph_path).await.map_err(io_error)? {
        return Ok(None);
    }
    read_graph_json(&graph_path).await.map(Some)
}

async fn read_graph_index(
    graph_store_dir: &FsPath,
) -> Result<Vec<GraphListEntry>, (StatusCode, String)> {
    let mut entries = fs::read_dir(graph_store_dir).await.map_err(io_error)?;
    let mut graphs = Vec::new();

    while let Some(entry) = entries.next_entry().await.map_err(io_error)? {
        let path = entry.path();
        if path.extension().and_then(|item| item.to_str()) != Some("json") {
            continue;
        }
        if path.file_name().and_then(|item| item.to_str()) == Some("latest.json") {
            continue;
        }

        let content = crate::runtime_persistence::read_to_string_bounded(
            &path,
            crate::runtime_persistence::MAX_BOUNDED_JSON_READ_BYTES,
        )
        .await
        .map_err(crate::runtime_persistence::bounded_read_api_error)?;
        let value: Value =
            serde_json::from_str(&content).map_err(|error| internal_error(error.into()))?;
        let reveal_path = resolve_graph_reveal_path_from_value(&value, &path)
            .await
            .map_err(internal_error)?;
        let graph_id = value
            .get("metadata")
            .and_then(|item| item.get("graph_id"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .ok_or_else(|| {
                internal_error(anyhow::anyhow!(
                    "graph file `{}` missing metadata.graph_id",
                    path.to_string_lossy()
                ))
            })?
            .to_string();
        let name = value
            .get("metadata")
            .and_then(|item| item.get("name"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .unwrap_or(&graph_id)
            .to_string();
        let updated_at = value
            .get("metadata")
            .and_then(|item| item.get("updated_at"))
            .and_then(Value::as_u64)
            .unwrap_or(0);

        graphs.push(GraphListEntry {
            graph_id,
            name,
            updated_at,
            path: reveal_path.to_string_lossy().to_string(),
        });
    }

    graphs.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| left.graph_id.cmp(&right.graph_id))
    });

    Ok(graphs)
}

async fn read_graph_versions(
    graph_store_dir: &FsPath,
    graph_id: &str,
) -> Result<Vec<GraphVersionEntry>, (StatusCode, String)> {
    let version_dir = graph_version_dir(graph_store_dir, graph_id);
    if !fs::try_exists(&version_dir).await.map_err(io_error)? {
        return Ok(Vec::new());
    }

    let latest_path = graph_store_dir.join(format!("{}.json", graph_id));
    let latest_body = crate::runtime_persistence::read_to_string_bounded(
        &latest_path,
        crate::runtime_persistence::MAX_BOUNDED_JSON_READ_BYTES,
    )
    .await
    .ok();
    let latest_value = latest_body
        .as_deref()
        .and_then(|body| serde_json::from_str::<Value>(body).ok());
    let latest_updated_at = latest_value
        .as_ref()
        .and_then(|value| value.get("metadata"))
        .and_then(|item| item.get("updated_at"))
        .and_then(Value::as_u64);

    let mut entries = fs::read_dir(&version_dir).await.map_err(io_error)?;
    let mut versions = Vec::new();
    while let Some(entry) = entries.next_entry().await.map_err(io_error)? {
        let path = entry.path();
        if path.extension().and_then(|item| item.to_str()) != Some("json") {
            continue;
        }

        let content = crate::runtime_persistence::read_to_string_bounded(
            &path,
            crate::runtime_persistence::MAX_BOUNDED_JSON_READ_BYTES,
        )
        .await
        .map_err(crate::runtime_persistence::bounded_read_api_error)?;
        let value: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                safe_eprintln!("[graph] 跳过损坏的版本文件 {}: {}", path.display(), e);
                continue;
            }
        };
        let updated_at = value
            .get("metadata")
            .and_then(|item| item.get("updated_at"))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let name = value
            .get("metadata")
            .and_then(|item| item.get("name"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .unwrap_or(graph_id)
            .to_string();
        let version_id = path
            .file_stem()
            .and_then(|item| item.to_str())
            .ok_or_else(|| internal_error(anyhow::anyhow!("无效的图版本文件名")))?;
        let quantscript_path = version_dir.join(format!("{}.qs", version_id));
        let version_label = value
            .get("metadata")
            .and_then(|item| item.get("version_label"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_string);
        let save_note = value
            .get("metadata")
            .and_then(|item| item.get("save_note"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_string);
        let node_count = value
            .get("nodes")
            .and_then(Value::as_array)
            .map(|items| items.len())
            .unwrap_or(0);
        let edge_count = value
            .get("edges")
            .and_then(Value::as_array)
            .map(|items| items.len())
            .unwrap_or(0);

        versions.push(GraphVersionEntry {
            graph_id: graph_id.to_string(),
            version_id: version_id.to_string(),
            name,
            updated_at,
            version_label,
            save_note,
            node_count,
            edge_count,
            path: path.to_string_lossy().to_string(),
            quantscript_path: quantscript_path.to_string_lossy().to_string(),
            is_latest: Some(updated_at) == latest_updated_at,
        });
    }

    versions.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| right.version_id.cmp(&left.version_id))
    });

    Ok(versions)
}

fn graph_version_dir(graph_store_dir: &FsPath, graph_id: &str) -> PathBuf {
    // v2.3.3 修复 S0-4: 纵深防御 — 即使调用方已验证 graph_id, 此处也做 sanitize
    let safe_segment = crate::runtime_persistence::sanitize_storage_path_segment(graph_id);
    graph_store_dir.join("versions").join(&safe_segment)
}

async fn read_optional_graph_version_quantscript(
    graph_store_dir: &FsPath,
    graph_id: &str,
    version_id: &str,
) -> Result<Option<String>, (StatusCode, String)> {
    let path = graph_version_dir(graph_store_dir, graph_id).join(format!("{version_id}.qs"));
    if !fs::try_exists(&path).await.map_err(io_error)? {
        return Ok(None);
    }
    crate::runtime_persistence::read_to_string_bounded(
        &path,
        crate::runtime_persistence::MAX_BOUNDED_JSON_READ_BYTES,
    )
    .await
    .map(Some)
    .map_err(crate::runtime_persistence::bounded_read_api_error)
}

async fn refresh_latest_graph_after_delete(
    graph_store_dir: &FsPath,
    deleted_graph_id: &str,
) -> Result<(), (StatusCode, String)> {
    let latest_path = graph_store_dir.join("latest.json");
    if !fs::try_exists(&latest_path).await.map_err(io_error)? {
        return Ok(());
    }

    let latest = read_graph_json(&latest_path).await?;
    let latest_graph_id = latest
        .get("metadata")
        .and_then(|metadata| metadata.get("graph_id"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    if latest_graph_id != deleted_graph_id {
        return Ok(());
    }

    let remaining = read_graph_index(graph_store_dir).await?;
    if let Some(next_latest) = remaining.first() {
        let next_latest_path = graph_store_dir.join(format!("{}.json", next_latest.graph_id));
        let content = crate::runtime_persistence::read_to_string_bounded(
            &next_latest_path,
            crate::runtime_persistence::MAX_BOUNDED_JSON_READ_BYTES,
        )
        .await
        .map_err(crate::runtime_persistence::bounded_read_not_found_api_error)?;
        // v4.7.1: 原子写入包含 tmp fsync + rename + 父目录 fsync。
        atomic_write(&latest_path, &content).await?;
    } else {
        remove_file_if_exists(&latest_path).await?;
    }

    Ok(())
}

async fn remove_file_if_exists(path: &FsPath) -> Result<(), (StatusCode, String)> {
    for attempt in 0..3 {
        match fs::remove_file(path).await {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied && attempt < 2 => {
                sleep(Duration::from_millis(100)).await;
                continue;
            }
            Err(error) => return Err(io_error(error)),
        }
    }
    Ok(())
}

async fn remove_dir_if_exists(path: &FsPath) -> Result<(), (StatusCode, String)> {
    for attempt in 0..3 {
        match fs::remove_dir_all(path).await {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied && attempt < 2 => {
                sleep(Duration::from_millis(100)).await;
                continue;
            }
            Err(error) => return Err(io_error(error)),
        }
    }
    Ok(())
}

fn graph_version_json_path(graph_store_dir: &FsPath, graph_id: &str, version_id: &str) -> PathBuf {
    graph_version_dir(graph_store_dir, graph_id).join(format!("{}.json", version_id))
}

fn validate_graph_version_id(version_id: &str) -> anyhow::Result<()> {
    if version_id.trim().is_empty()
        || version_id.contains('/')
        || version_id.contains('\\')
        || version_id.contains("..")
    {
        bail!("图版本 ID 必须是非空且文件安全的令牌");
    }

    Ok(())
}

fn apply_optional_metadata_text(
    metadata: &mut serde_json::Map<String, Value>,
    key: &str,
    value: Option<&str>,
) {
    let Some(value) = value else {
        return;
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        metadata.remove(key);
        return;
    }
    metadata.insert(key.to_string(), Value::String(trimmed.to_string()));
}

async fn resolve_graph_reveal_path(graph_json_path: &FsPath) -> anyhow::Result<PathBuf> {
    let content = crate::runtime_persistence::read_to_string_bounded(
        graph_json_path,
        crate::runtime_persistence::MAX_BOUNDED_JSON_READ_BYTES,
    )
    .await?;
    let value: Value = serde_json::from_str(&content)?;
    resolve_graph_reveal_path_from_value(&value, graph_json_path).await
}

pub(crate) async fn resolve_graph_reveal_path_from_value(
    value: &Value,
    fallback_path: &FsPath,
) -> anyhow::Result<PathBuf> {
    let saved_path = value
        .get("metadata")
        .and_then(|item| item.get("artifacts"))
        .and_then(|item| item.get("quantscript"))
        .and_then(|item| item.get("saved_path"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(PathBuf::from);

    if let Some(path) = saved_path {
        let allowed_root = fallback_path
            .parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
            .canonicalize()
            .with_context(|| {
                format!(
                    "无法解析图存储目录: {}",
                    fallback_path
                        .parent()
                        .unwrap_or_else(|| std::path::Path::new("."))
                        .display()
                )
            })?;
        let candidates = if path.is_absolute() {
            // v1.1.2/v3.7.1: 绝对路径需规范化后确认在当前图存储目录内
            let resolved = path
                .canonicalize()
                .with_context(|| format!("saved_path 指向不存在的路径: {}", path.display()))?;
            if !resolved.starts_with(&allowed_root) {
                anyhow::bail!("saved_path 必须在图存储目录内: {}", path.display());
            }
            vec![resolved]
        } else {
            let mut items = vec![path.clone()];
            if let Some(parent) = fallback_path.parent() {
                items.push(parent.join(&path));
            }
            items
        };

        for candidate in candidates {
            if fs::try_exists(&candidate).await.unwrap_or(false) {
                let resolved = canonical_existing_path(&candidate).await?;
                // 验证相对路径解析后仍在当前图存储目录内
                if !resolved.starts_with(&allowed_root) {
                    anyhow::bail!("saved_path 必须在图存储目录内: {}", path.display());
                }
                return Ok(resolved);
            }
        }
    }

    canonical_existing_path(fallback_path).await
}

async fn canonical_existing_path(path: &FsPath) -> anyhow::Result<PathBuf> {
    fs::canonicalize(path)
        .await
        .with_context(|| format!("未能解析图显示路径 `{}`", path.display()))
}

fn reveal_path_in_file_manager(path: &FsPath) -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(format!("/select,\"{}\"", path.display()))
            .spawn()
            .context("未能显示图文件在资源管理器中")?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()
            .context("未能显示图文件在 Finder 中")?;
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let parent = path.parent().unwrap_or(path);
        Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .context("未能打开图目录")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_graph_test_dir() -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "quantpilot_graph_txn_test_{}_{}",
            std::process::id(),
            unique
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[tokio::test]
    async fn rollback_restores_replaced_graph_artifact() {
        let dir = temp_graph_test_dir();
        let target = dir.join("graph.json");
        let staged = dir.join("staged.json");
        std::fs::write(&target, "old").unwrap();
        std::fs::write(&staged, "new").unwrap();

        let mut replacements = Vec::new();
        replace_graph_artifact(&staged, &target, &mut replacements, "test-version")
            .await
            .unwrap();
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "new");

        rollback_graph_replacements(&replacements).await;
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "old");
        assert!(!staged.exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn rollback_restores_multiple_replaced_graph_artifacts() {
        let dir = temp_graph_test_dir();
        let target_a = dir.join("a.json");
        let target_b = dir.join("b.json");
        let staged_a = dir.join("staged-a.json");
        let staged_b = dir.join("staged-b.json");
        std::fs::write(&target_a, "old-a").unwrap();
        std::fs::write(&target_b, "old-b").unwrap();
        std::fs::write(&staged_a, "new-a").unwrap();
        std::fs::write(&staged_b, "new-b").unwrap();

        let replacements = commit_graph_artifact_bundle(
            &[
                GraphArtifactCommitItem::new(staged_a, target_a.clone()),
                GraphArtifactCommitItem::new(staged_b, target_b.clone()),
            ],
            "test-version",
            &[],
        )
        .await
        .unwrap();

        assert_eq!(std::fs::read_to_string(&target_a).unwrap(), "new-a");
        assert_eq!(std::fs::read_to_string(&target_b).unwrap(), "new-b");

        rollback_graph_replacements(&replacements).await;
        assert_eq!(std::fs::read_to_string(&target_a).unwrap(), "old-a");
        assert_eq!(std::fs::read_to_string(&target_b).unwrap(), "old-b");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
