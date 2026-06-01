mod data_source_lowering;
mod input_shape_validation;
mod intent_lowering;
mod profile_lowering;
mod terminal_parse;
mod unsupported_node_logging;

use quantscript::ScriptModule;
use serde_json::Value;

pub(crate) fn convert_graph_json_to_script_module(
    graph_value: &Value,
) -> anyhow::Result<ScriptModule> {
    let (nodes, edges) = input_shape_validation::require_graph_nodes_and_edges(graph_value)?;

    // Build a minimal QS source from the graph
    let mut qs_lines: Vec<String> = vec!["fn strategy() {".to_string()];

    data_source_lowering::append_data_source_lowering_lines(nodes, &mut qs_lines);

    // Agent/runtime nodes are represented by the inferred runtime graph and metadata, not calls.
    for node in nodes {
        if profile_lowering::append_profile_lowering_line(node, &mut qs_lines) {
            continue;
        }

        unsupported_node_logging::log_if_unsupported_node(node);
    }

    intent_lowering::append_intent_lowering_lines(nodes, edges, &mut qs_lines)?;

    terminal_parse::parse_generated_qs_lines(qs_lines)
}
