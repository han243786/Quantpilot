use crate::{
    auth, backtest_replay_response_from_record, json_bad_request, load_backtest_record_from_state,
    runtime::{normalized_replay_options, RuntimeReplayQuery},
    AppState, RuntimeReplayResponse,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::time::Instant;

pub(crate) async fn get_backtest_replay(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(backtest_id): Path<String>,
    Query(query): Query<RuntimeReplayQuery>,
) -> Result<Json<RuntimeReplayResponse>, (StatusCode, String)> {
    let started = Instant::now();
    let options = normalized_replay_options(query);
    let record = load_backtest_record_from_state(&state, &user_id, &backtest_id).await?;
    let response = backtest_replay_response_from_record(record, options)
        .map_err(|message| json_bad_request("bad_replay_cursor", message))?;
    state
        .evidence_metrics
        .record_replay_page(started.elapsed().as_millis() as u64);
    Ok(Json(response))
}
