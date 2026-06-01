use crate::{
    current_time_ms, internal_error, json_bad_request, not_found_io_error, validate_graph_id,
    AppState, CompileRuntimeTargets, ParseGraphQuantScriptRequest,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::Value;
use tokio::fs;

mod artifact_target_projection;
mod formal_module_conversion;
mod graph_to_qs_generation;
mod strategy_graph_parser;
pub(crate) use formal_module_conversion::convert_graph_json_to_script_module;
pub(crate) use graph_to_qs_generation::generate_quantscript_from_graph_value;

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/graphs/:graph_id/quantscript",
            get(load_graph_quantscript),
        )
        .route(
            "/api/quantscript/graph/parse",
            post(parse_graph_quantscript),
        )
}

async fn load_graph_quantscript(
    State(state): State<AppState>,
    Path(graph_id): Path<String>,
) -> Result<String, (StatusCode, String)> {
    validate_graph_id(&graph_id).map_err(internal_error)?;
    let source_path = state.graph_store_dir.join(format!("{}.qs", graph_id));
    fs::read_to_string(&source_path)
        .await
        .map_err(not_found_io_error)
}

async fn parse_graph_quantscript(
    Json(request): Json<ParseGraphQuantScriptRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    parse_graph_quantscript_source(&request.source)
        .map(Json)
        .map_err(|error| {
            json_bad_request(
                "bad_request",
                format!("strategy_graph QuantScript 解析失败: {error:#}"),
            )
        })
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
