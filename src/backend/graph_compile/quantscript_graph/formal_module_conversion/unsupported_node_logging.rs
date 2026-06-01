use serde_json::Value;

pub(super) fn log_if_unsupported_node(node: &Value) {
    let node_type = node.get("type").and_then(Value::as_str).unwrap_or("");
    if let Some(message) = unsupported_node_message(node_type) {
        safe_eprintln!("{}", message);
    }
}

fn unsupported_node_message(node_type: &str) -> Option<String> {
    match node_type {
        "data" | "intent" | "agent" | "runtime" | "runtime_control" => None,
        _ => Some(format!(
            "[graph->QS] 未知节点类型 '{}', 跳过 QS 生成",
            node_type
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::unsupported_node_message;

    #[test]
    fn unsupported_node_logging_keeps_known_node_types_silent() {
        for node_type in ["data", "intent", "agent", "runtime", "runtime_control"] {
            assert_eq!(unsupported_node_message(node_type), None);
        }
    }

    #[test]
    fn unsupported_node_logging_formats_unknown_node_message() {
        assert_eq!(
            unsupported_node_message("custom"),
            Some("[graph->QS] 未知节点类型 'custom', 跳过 QS 生成".to_string())
        );
    }
}
