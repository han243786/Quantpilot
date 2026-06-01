use serde_json::Value;

pub(super) fn require_graph_nodes_and_edges(
    graph_value: &Value,
) -> anyhow::Result<(&[Value], &[Value])> {
    let nodes = graph_value
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("graph.nodes 必须是数组"))?;
    let edges = graph_value
        .get("edges")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("graph.edges 必须是数组"))?;

    Ok((nodes, edges))
}

#[cfg(test)]
mod tests {
    use super::require_graph_nodes_and_edges;
    use serde_json::json;

    #[test]
    fn input_shape_validation_returns_nodes_and_edges_arrays() {
        let graph = json!({
            "nodes": [{"id": "data_1"}],
            "edges": [{"source_node_id": "data_1", "target_node_id": "intent_1"}]
        });

        let (nodes, edges) = require_graph_nodes_and_edges(&graph).unwrap();

        assert_eq!(nodes.len(), 1);
        assert_eq!(edges.len(), 1);
    }

    #[test]
    fn input_shape_validation_rejects_missing_nodes_array() {
        let graph = json!({ "edges": [] });

        let error = require_graph_nodes_and_edges(&graph).unwrap_err();

        assert_eq!(error.to_string(), "graph.nodes 必须是数组");
    }

    #[test]
    fn input_shape_validation_rejects_missing_edges_array() {
        let graph = json!({ "nodes": [] });

        let error = require_graph_nodes_and_edges(&graph).unwrap_err();

        assert_eq!(error.to_string(), "graph.edges 必须是数组");
    }
}
