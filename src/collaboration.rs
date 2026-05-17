use super::*;

const DEFAULT_LOCAL_ACTOR_ID: &str = "local_operator";
const DEFAULT_LOCAL_ACTOR_NAME: &str = "Local operator";

pub(super) fn default_actor_identity() -> ActorIdentity {
    ActorIdentity {
        actor_id: DEFAULT_LOCAL_ACTOR_ID.to_string(),
        display_name: DEFAULT_LOCAL_ACTOR_NAME.to_string(),
    }
}

pub(super) fn normalize_actor_identity(actor: Option<ActorIdentity>) -> ActorIdentity {
    let actor = actor.unwrap_or_else(default_actor_identity);
    let actor_id = actor.actor_id.trim();
    let display_name = actor.display_name.trim();
    ActorIdentity {
        actor_id: if actor_id.is_empty() {
            DEFAULT_LOCAL_ACTOR_ID.to_string()
        } else {
            actor_id.to_string()
        },
        display_name: if display_name.is_empty() {
            DEFAULT_LOCAL_ACTOR_NAME.to_string()
        } else {
            display_name.to_string()
        },
    }
}

pub(super) fn collaboration_from_graph(graph: &Value) -> GraphCollaborationMetadata {
    let metadata = graph
        .get("metadata")
        .and_then(Value::as_object)
        .and_then(|metadata| metadata.get("collaboration"))
        .and_then(Value::as_object);

    let owner = metadata
        .and_then(|item| item.get("owner"))
        .and_then(parse_actor_identity);
    let editors = metadata
        .and_then(|item| item.get("editors"))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(parse_actor_identity)
                .fold(Vec::new(), |mut acc, actor| {
                    if !acc
                        .iter()
                        .any(|item: &ActorIdentity| item.actor_id == actor.actor_id)
                    {
                        acc.push(actor);
                    }
                    acc
                })
        })
        .unwrap_or_default();
    let last_saved_by = metadata
        .and_then(|item| item.get("last_saved_by"))
        .and_then(parse_actor_identity);
    let last_run_actor = metadata
        .and_then(|item| item.get("last_run_actor"))
        .and_then(parse_actor_identity);

    GraphCollaborationMetadata {
        owner,
        editors,
        last_saved_by,
        last_run_actor,
    }
}

pub(super) fn collaboration_with_saved_actor(
    existing: Option<&Value>,
    actor: &ActorIdentity,
) -> Result<GraphCollaborationMetadata, (StatusCode, String)> {
    let mut collaboration = existing.map(collaboration_from_graph).unwrap_or_default();

    if collaboration.owner.is_none() {
        collaboration.owner = Some(actor.clone());
    }

    ensure_actor_has_access(&collaboration, actor)?;
    collaboration.last_saved_by = Some(actor.clone());
    Ok(collaboration)
}

pub(super) async fn authorize_graph_actor(
    graph_store_dir: &FsPath,
    graph_id: &str,
    actor: &ActorIdentity,
) -> Result<GraphCollaborationMetadata, (StatusCode, String)> {
    let graph_path = graph_store_dir.join(format!("{}.json", graph_id));
    if !graph_path.exists() {
        return Ok(GraphCollaborationMetadata::default());
    }

    let body = tokio::fs::read_to_string(&graph_path).await.map_err(io_error)?;
    let graph: Value = serde_json::from_str(&body).map_err(|error| internal_error(error.into()))?;
    let collaboration = collaboration_from_graph(&graph);
    if collaboration.owner.is_some() {
        ensure_actor_has_access(&collaboration, actor)?;
    }
    Ok(collaboration)
}

pub(super) async fn collaboration_with_run_actor(
    graph_store_dir: &FsPath,
    graph_id: &str,
    actor: &ActorIdentity,
) -> Result<GraphCollaborationMetadata, (StatusCode, String)> {
    let mut collaboration = authorize_graph_actor(graph_store_dir, graph_id, actor).await?;
    if collaboration.owner.is_none() {
        collaboration.owner = Some(actor.clone());
    }
    collaboration.last_run_actor = Some(actor.clone());
    Ok(collaboration)
}

pub(super) fn write_graph_collaboration_metadata(
    graph: &mut Value,
    collaboration: &GraphCollaborationMetadata,
) {
    let Some(root) = graph.as_object_mut() else {
        return;
    };
    let metadata = root
        .entry("metadata")
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    let Some(metadata) = metadata.as_object_mut() else {
        return;
    };
    let collaboration_value =
        serde_json::to_value(collaboration).expect("graph collaboration metadata should serialize");
    metadata.insert("collaboration".to_string(), collaboration_value);
}

pub(super) async fn persist_graph_audit_entry(
    audit_store_dir: &FsPath,
    entry: &GraphAuditEntry,
) -> std::io::Result<()> {
    fs::create_dir_all(audit_store_dir).await?;
    let path = audit_store_dir.join(format!("{}.json", entry.graph_id));
    let mut entries = load_graph_audit_entries(audit_store_dir, &entry.graph_id).await?;
    entries.push(entry.clone());
    let body = serde_json::to_string_pretty(&entries)
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, body).await?;
    fs::rename(&tmp, &path).await
}

pub(super) async fn load_graph_audit_entries(
    audit_store_dir: &FsPath,
    graph_id: &str,
) -> std::io::Result<Vec<GraphAuditEntry>> {
    let path = audit_store_dir.join(format!("{}.json", graph_id));
    if !fs::try_exists(&path).await? {
        return Ok(Vec::new());
    }
    let body = fs::read_to_string(path).await?;
    let mut entries = serde_json::from_str::<Vec<GraphAuditEntry>>(&body).unwrap_or_default();
    entries.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.audit_id.cmp(&left.audit_id))
    });
    Ok(entries)
}

pub(super) fn build_graph_audit_entry(
    graph_id: &str,
    actor: &ActorIdentity,
    action: GraphAuditAction,
    target_id: Option<String>,
    summary: impl Into<String>,
) -> GraphAuditEntry {
    let created_at_ms = current_time_ms();
    GraphAuditEntry {
        audit_id: format!("audit_{}_{}", graph_id, created_at_ms),
        graph_id: graph_id.to_string(),
        action,
        created_at_ms,
        actor: actor.clone(),
        target_id,
        summary: summary.into(),
    }
}

fn ensure_actor_has_access(
    collaboration: &GraphCollaborationMetadata,
    actor: &ActorIdentity,
) -> Result<(), (StatusCode, String)> {
    if collaboration
        .owner
        .as_ref()
        .is_some_and(|owner| owner.actor_id == actor.actor_id)
        || collaboration
            .editors
            .iter()
            .any(|editor| editor.actor_id == actor.actor_id)
    {
        return Ok(());
    }

    Err((
        StatusCode::FORBIDDEN,
        format!(
            "操作者 `{}` 无权修改图 `{}`",
            actor.actor_id,
            collaboration
                .owner
                .as_ref()
                .map(|owner| owner.actor_id.as_str())
                .unwrap_or("unowned_graph")
        ),
    ))
}

fn parse_actor_identity(value: &Value) -> Option<ActorIdentity> {
    let object = value.as_object()?;
    let actor_id = object
        .get("actor_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())?;
    let display_name = object
        .get("display_name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(actor_id);
    Some(ActorIdentity {
        actor_id: actor_id.to_string(),
        display_name: display_name.to_string(),
    })
}
