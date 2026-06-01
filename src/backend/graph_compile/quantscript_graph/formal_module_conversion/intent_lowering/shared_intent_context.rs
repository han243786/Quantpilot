use serde_json::Value;

pub(super) struct IntentLoweringContext<'a> {
    pub(super) module_key: &'a str,
    pub(super) cfg: &'a Value,
    pub(super) instrument: &'a str,
    pub(super) node_id: &'a str,
    pub(super) source_var: String,
}

pub(super) fn resolve_intent_lowering_context<'a>(
    node: &'a Value,
    edges: &'a [Value],
) -> IntentLoweringContext<'a> {
    let module_key = node.get("module_key").and_then(Value::as_str).unwrap_or("");
    let cfg = node.get("config").unwrap_or(&Value::Null);
    let instrument = cfg
        .get("instrument")
        .and_then(Value::as_str)
        .unwrap_or("BTCUSDT");
    let node_id = node.get("id").and_then(Value::as_str).unwrap_or("");
    let upstream_edge = edges
        .iter()
        .find(|e| e.get("target_node_id").and_then(Value::as_str) == Some(node_id));
    let source_id = upstream_edge
        .and_then(|e| e.get("source_node_id").and_then(Value::as_str))
        .unwrap_or("data");
    let source_var = source_id.replace(['-', '.'], "_");

    IntentLoweringContext {
        module_key,
        cfg,
        instrument,
        node_id,
        source_var,
    }
}
