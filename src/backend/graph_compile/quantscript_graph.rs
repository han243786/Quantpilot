use crate::{current_time_ms, AppState, CompileRuntimeTargets};
use axum::Router;
use serde_json::Value;

mod artifact_target_projection;
mod formal_module_conversion;
mod graph_to_qs_generation;
mod route_surface;
mod strategy_graph_parser;
pub(crate) use formal_module_conversion::convert_graph_json_to_script_module;
pub(crate) use graph_to_qs_generation::generate_quantscript_from_graph_value;

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    route_surface::register_routes(router)
}

pub(crate) fn attach_quantscript_artifacts(
    graph: &mut Value,
    quantscript: &str,
    generated_at: u64,
    quantscript_path: &std::path::Path,
) {
    artifact_target_projection::attach_quantscript_artifacts(
        graph,
        quantscript,
        generated_at,
        quantscript_path,
        graph_to_qs_generation::generate_node_quantscript,
    )
}

pub(crate) fn build_compile_runtime_targets_from_graph(graph: &Value) -> CompileRuntimeTargets {
    artifact_target_projection::build_compile_runtime_targets_from_graph(graph)
}

pub(crate) fn parse_graph_quantscript_source(source: &str) -> anyhow::Result<Value> {
    let now = current_time_ms();
    let mut graph = strategy_graph_parser::parse_strategy_graph_source(source, now)?;
    attach_quantscript_artifacts(&mut graph, source, now, std::path::Path::new(""));
    Ok(graph)
}
