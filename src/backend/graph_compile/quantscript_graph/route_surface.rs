use crate::{
    internal_error, json_bad_request, not_found_io_error, validate_graph_id, AppState,
    ParseGraphQuantScriptRequest,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::Value;
use tokio::fs;

pub(super) fn register_routes(router: Router<AppState>) -> Router<AppState> {
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
    super::parse_graph_quantscript_source(&request.source)
        .map(Json)
        .map_err(|error| {
            json_bad_request(
                "bad_request",
                format!("strategy_graph QuantScript 解析失败: {error:#}"),
            )
        })
}
