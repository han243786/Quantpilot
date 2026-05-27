use super::*;

pub(super) fn build_graph_version_compare_response(
    graph_id: &str,
    left: GraphVersionEntry,
    left_graph: &Value,
    right: GraphVersionEntry,
    right_graph: &Value,
) -> GraphVersionCompareResponse {
    let metadata_rows = build_metadata_diff_rows(left_graph, right_graph);
    let node_diff = build_node_diff(left_graph, right_graph);
    let edge_diff = build_edge_diff(left_graph, right_graph);
    let config_diffs = build_config_diffs(left_graph, right_graph);
    let has_changes = metadata_rows
        .iter()
        .any(|row| row.status != GraphVersionDiffStatus::Same)
        || !node_diff.added_ids.is_empty()
        || !node_diff.removed_ids.is_empty()
        || !node_diff.changed_ids.is_empty()
        || !edge_diff.added_ids.is_empty()
        || !edge_diff.removed_ids.is_empty()
        || !edge_diff.changed_ids.is_empty()
        || !config_diffs.is_empty();

    GraphVersionCompareResponse {
        graph_id: graph_id.to_string(),
        left,
        right,
        metadata_rows,
        node_diff,
        edge_diff,
        config_diffs,
        strategy_config_diff: None,
        strategy_config_evidence_diff: None,
        has_changes,
    }
}

fn build_metadata_diff_rows(left_graph: &Value, right_graph: &Value) -> Vec<GraphVersionDiffRow> {
    let fields = [
        ("name", "Graph name"),
        ("version", "Graph version"),
        ("source_mode", "Source mode"),
        ("version_label", "Version label"),
        ("save_note", "Save note"),
        ("created_at", "Created at"),
        ("updated_at", "Updated at"),
    ];

    fields
        .into_iter()
        .map(|(key, label)| build_metadata_diff_row(left_graph, right_graph, key, label))
        .collect()
}

fn build_metadata_diff_row(
    left_graph: &Value,
    right_graph: &Value,
    key: &str,
    label: &str,
) -> GraphVersionDiffRow {
    let left_value = graph_metadata_value(left_graph, key);
    let right_value = graph_metadata_value(right_graph, key);
    let status = diff_status(left_value.as_ref(), right_value.as_ref());

    GraphVersionDiffRow {
        key: key.to_string(),
        label: label.to_string(),
        status,
        left_value,
        right_value,
    }
}

fn graph_metadata_value(graph: &Value, key: &str) -> Option<String> {
    graph
        .get("metadata")
        .and_then(|item| item.get(key))
        .map(stringify_json_value)
}

fn build_node_diff(left_graph: &Value, right_graph: &Value) -> GraphVersionCollectionDiff {
    let left_nodes = graph_nodes_by_id(left_graph);
    let right_nodes = graph_nodes_by_id(right_graph);
    let left_ids: BTreeSet<String> = left_nodes.keys().cloned().collect();
    let right_ids: BTreeSet<String> = right_nodes.keys().cloned().collect();

    let added_ids = right_ids.difference(&left_ids).cloned().collect::<Vec<_>>();
    let removed_ids = left_ids.difference(&right_ids).cloned().collect::<Vec<_>>();
    let changed_ids = left_ids
        .intersection(&right_ids)
        .filter_map(|node_id| {
            let left_node = left_nodes.get(node_id)?;
            let right_node = right_nodes.get(node_id)?;
            (node_signature(left_node) != node_signature(right_node)).then(|| node_id.clone())
        })
        .collect::<Vec<_>>();

    GraphVersionCollectionDiff {
        left_count: left_ids.len(),
        right_count: right_ids.len(),
        added_ids,
        removed_ids,
        changed_ids,
    }
}

fn build_edge_diff(left_graph: &Value, right_graph: &Value) -> GraphVersionCollectionDiff {
    let left_edges = graph_edges_by_id(left_graph);
    let right_edges = graph_edges_by_id(right_graph);
    let left_ids: BTreeSet<String> = left_edges.keys().cloned().collect();
    let right_ids: BTreeSet<String> = right_edges.keys().cloned().collect();

    let added_ids = right_ids.difference(&left_ids).cloned().collect::<Vec<_>>();
    let removed_ids = left_ids.difference(&right_ids).cloned().collect::<Vec<_>>();
    let changed_ids = left_ids
        .intersection(&right_ids)
        .filter_map(|edge_id| {
            let left_edge = left_edges.get(edge_id)?;
            let right_edge = right_edges.get(edge_id)?;
            (stringify_json_value(left_edge) != stringify_json_value(right_edge))
                .then(|| edge_id.clone())
        })
        .collect::<Vec<_>>();

    GraphVersionCollectionDiff {
        left_count: left_ids.len(),
        right_count: right_ids.len(),
        added_ids,
        removed_ids,
        changed_ids,
    }
}

fn build_config_diffs(left_graph: &Value, right_graph: &Value) -> Vec<GraphVersionConfigDiffEntry> {
    let left_nodes = graph_nodes_by_id(left_graph);
    let right_nodes = graph_nodes_by_id(right_graph);
    let mut diffs = Vec::new();

    for node_id in left_nodes
        .keys()
        .filter(|node_id| right_nodes.contains_key(*node_id))
    {
        let left_node = match left_nodes.get(node_id) {
            Some(value) => value,
            None => continue,
        };
        let right_node = match right_nodes.get(node_id) {
            Some(value) => value,
            None => continue,
        };

        let mut left_fields = BTreeMap::new();
        let mut right_fields = BTreeMap::new();
        flatten_config_fields(
            left_node.get("config").unwrap_or(&Value::Null),
            "",
            &mut left_fields,
        );
        flatten_config_fields(
            right_node.get("config").unwrap_or(&Value::Null),
            "",
            &mut right_fields,
        );

        let field_keys = left_fields
            .keys()
            .chain(right_fields.keys())
            .cloned()
            .collect::<BTreeSet<_>>();
        let node_name = right_node
            .get("name")
            .and_then(Value::as_str)
            .or_else(|| left_node.get("name").and_then(Value::as_str))
            .unwrap_or(node_id)
            .to_string();

        for field_path in field_keys {
            let left_value = left_fields.get(&field_path).cloned();
            let right_value = right_fields.get(&field_path).cloned();
            let status = diff_status(left_value.as_ref(), right_value.as_ref());
            if status == GraphVersionDiffStatus::Same {
                continue;
            }

            diffs.push(GraphVersionConfigDiffEntry {
                node_id: node_id.clone(),
                node_name: node_name.clone(),
                field_path,
                status,
                left_value,
                right_value,
            });
        }
    }

    diffs.sort_by(|left, right| {
        left.node_id
            .cmp(&right.node_id)
            .then_with(|| left.field_path.cmp(&right.field_path))
    });
    diffs
}

fn graph_nodes_by_id(graph: &Value) -> BTreeMap<String, &Value> {
    graph
        .get("nodes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|node| {
            node.get("id")
                .and_then(Value::as_str)
                .map(|node_id| (node_id.to_string(), node))
        })
        .collect()
}

fn graph_edges_by_id(graph: &Value) -> BTreeMap<String, &Value> {
    graph
        .get("edges")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|edge| {
            edge.get("id")
                .and_then(Value::as_str)
                .map(|edge_id| (edge_id.to_string(), edge))
        })
        .collect()
}

fn node_signature(node: &Value) -> String {
    json!({
        "name": node.get("name"),
        "module_key": node.get("module_key"),
        "input_ports": node.get("input_ports"),
        "output_ports": node.get("output_ports"),
    })
    .to_string()
}

fn flatten_config_fields(value: &Value, prefix: &str, target: &mut BTreeMap<String, String>) {
    match value {
        Value::Object(map) => {
            for (key, nested) in map {
                let next_prefix = if prefix.is_empty() {
                    key.to_string()
                } else {
                    format!("{prefix}.{key}")
                };
                flatten_config_fields(nested, &next_prefix, target);
            }
        }
        Value::Array(_) => {
            let key = if prefix.is_empty() {
                "config".to_string()
            } else {
                prefix.to_string()
            };
            target.insert(key, stringify_json_value(value));
        }
        _ => {
            let key = if prefix.is_empty() {
                "config".to_string()
            } else {
                prefix.to_string()
            };
            target.insert(key, stringify_json_value(value));
        }
    }
}

fn diff_status(left: Option<&String>, right: Option<&String>) -> GraphVersionDiffStatus {
    match (left, right) {
        (Some(left), Some(right)) if left == right => GraphVersionDiffStatus::Same,
        (Some(_), Some(_)) => GraphVersionDiffStatus::Different,
        (None, Some(_)) => GraphVersionDiffStatus::Added,
        (Some(_), None) => GraphVersionDiffStatus::Removed,
        (None, None) => GraphVersionDiffStatus::Same,
    }
}

fn stringify_json_value(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(string) => string.clone(),
        _ => value.to_string(),
    }
}
