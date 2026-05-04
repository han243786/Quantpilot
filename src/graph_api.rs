use super::*;

pub(super) fn register_graph_routes(router: Router<AppState>) -> Router<AppState> {
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
            "graph.metadata.graph_id is required".to_string(),
        ))?;
    validate_graph_id(graph_id).map_err(internal_error)?;
    let graph_id = graph_id.to_string();
    let actor = normalize_actor_identity(request.actor);
    let existing_graph = read_optional_graph_json(&state.graph_store_dir, &graph_id).await?;
    let collaboration = collaboration_with_saved_actor(existing_graph.as_ref(), &actor)?;

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
) -> Result<Json<Vec<GraphListEntry>>, (StatusCode, String)> {
    read_graph_index(&state.graph_store_dir).await.map(Json)
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
    State(state): State<AppState>,
    Path((graph_id, left_version_id, right_version_id)): Path<(String, String, String)>,
) -> Result<Json<GraphVersionCompareResponse>, (StatusCode, String)> {
    validate_graph_id(&graph_id).map_err(internal_error)?;
    validate_graph_version_id(&left_version_id).map_err(internal_error)?;
    validate_graph_version_id(&right_version_id).map_err(internal_error)?;

    let left_path = graph_version_json_path(&state.graph_store_dir, &graph_id, &left_version_id);
    let right_path = graph_version_json_path(&state.graph_store_dir, &graph_id, &right_version_id);
    let left_graph = read_graph_json(&left_path).await?;
    let right_graph = read_graph_json(&right_path).await?;
    let versions = read_graph_versions(&state.graph_store_dir, &graph_id).await?;
    let left = versions
        .iter()
        .find(|entry| entry.version_id == left_version_id)
        .cloned()
        .ok_or_else(|| {
            not_found_io_error(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("graph version `{left_version_id}` not found"),
            ))
        })?;
    let right = versions
        .iter()
        .find(|entry| entry.version_id == right_version_id)
        .cloned()
        .ok_or_else(|| {
            not_found_io_error(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("graph version `{right_version_id}` not found"),
            ))
        })?;

    Ok(Json(build_graph_version_compare_response(
        &graph_id,
        left,
        &left_graph,
        right,
        &right_graph,
    )))
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
    validate_graph_id(&graph_id).map_err(internal_error)?;
    let graph_path = state.graph_store_dir.join(format!("{}.json", graph_id));
    if !fs::try_exists(&graph_path).await.map_err(io_error)? {
        return Err(not_found_io_error(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("graph `{}` not found", graph_id),
        )));
    }

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
            format!("graph `{}` not found", graph_id),
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
    let content = fs::read_to_string(path).await.map_err(not_found_io_error)?;
    serde_json::from_str(&content).map_err(|error| internal_error(error.into()))
}

async fn persist_graph_version(
    graph_store_dir: &FsPath,
    graph_id: &str,
    input_graph: &Value,
    version_label: Option<&str>,
    save_note: Option<&str>,
    collaboration: GraphCollaborationMetadata,
) -> Result<SaveGraphResponse, (StatusCode, String)> {
    let saved_at = current_time_ms();
    let version_id = saved_at.to_string();
    let graph_path = graph_store_dir.join(format!("{}.json", graph_id));
    let latest_path = graph_store_dir.join("latest.json");
    let quantscript_path = graph_store_dir.join(format!("{}.qs", graph_id));
    let version_dir = graph_version_dir(graph_store_dir, graph_id);
    let version_graph_path = version_dir.join(format!("{}.json", version_id));
    let version_quantscript_path = version_dir.join(format!("{}.qs", version_id));
    let quantscript = generate_quantscript_from_graph_value(input_graph).map_err(internal_error)?;
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
    fs::write(&graph_path, &body).await.map_err(io_error)?;
    fs::write(&latest_path, &body).await.map_err(io_error)?;
    fs::write(&quantscript_path, &quantscript)
        .await
        .map_err(io_error)?;
    fs::write(&version_graph_path, &body)
        .await
        .map_err(io_error)?;
    fs::write(&version_quantscript_path, &quantscript)
        .await
        .map_err(io_error)?;

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

        let content = fs::read_to_string(&path).await.map_err(io_error)?;
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
    let latest_body = fs::read_to_string(&latest_path).await.ok();
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

        let content = fs::read_to_string(&path).await.map_err(io_error)?;
        let value: Value =
            serde_json::from_str(&content).map_err(|error| internal_error(error.into()))?;
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
            .ok_or_else(|| internal_error(anyhow::anyhow!("invalid graph version filename")))?;
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
    graph_store_dir.join("versions").join(graph_id)
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
        let content = fs::read_to_string(&next_latest_path)
            .await
            .map_err(not_found_io_error)?;
        fs::write(&latest_path, content).await.map_err(io_error)?;
    } else {
        remove_file_if_exists(&latest_path).await?;
    }

    Ok(())
}

async fn remove_file_if_exists(path: &FsPath) -> Result<(), (StatusCode, String)> {
    match fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error(error)),
    }
}

async fn remove_dir_if_exists(path: &FsPath) -> Result<(), (StatusCode, String)> {
    match fs::remove_dir_all(path).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error(error)),
    }
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
        bail!("graph version id must be a non-empty file-safe token");
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
    let content = fs::read_to_string(graph_json_path).await?;
    let value: Value = serde_json::from_str(&content)?;
    resolve_graph_reveal_path_from_value(&value, graph_json_path).await
}

pub(super) async fn resolve_graph_reveal_path_from_value(
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
        let candidates = if path.is_absolute() {
            vec![path]
        } else {
            let mut items = vec![path.clone()];
            if let Some(parent) = fallback_path.parent() {
                items.push(parent.join(&path));
            }
            items
        };

        for candidate in candidates {
            if fs::try_exists(&candidate).await.unwrap_or(false) {
                return canonical_existing_path(&candidate).await;
            }
        }
    }

    canonical_existing_path(fallback_path).await
}

async fn canonical_existing_path(path: &FsPath) -> anyhow::Result<PathBuf> {
    fs::canonicalize(path)
        .await
        .with_context(|| format!("failed to resolve graph reveal path `{}`", path.display()))
}

fn reveal_path_in_file_manager(path: &FsPath) -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn()
            .context("failed to reveal graph file in Explorer")?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()
            .context("failed to reveal graph file in Finder")?;
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let parent = path.parent().unwrap_or(path);
        Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .context("failed to open graph directory")?;
    }

    Ok(())
}
