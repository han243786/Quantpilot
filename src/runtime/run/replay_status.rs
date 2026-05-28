use super::*;

pub(crate) async fn get_run_replay(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Query(query): Query<RuntimeReplayQuery>,
) -> Result<Json<RuntimeReplayResponse>, (StatusCode, String)> {
    let started = Instant::now();
    let options = normalized_replay_options(query);
    let record = load_run_record_from_state(&state, &user_id, &run_id).await?;
    let response = run_replay_response_from_record(record, options)
        .map_err(|message| json_bad_request("bad_replay_cursor", message))?;
    state
        .evidence_metrics
        .record_replay_page(started.elapsed().as_millis() as u64);
    Ok(Json(response))
}

pub(crate) async fn get_run_status(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<RunStatusResponse>, (StatusCode, String)> {
    let record = load_run_record_from_state(&state, &user_id, &run_id).await?;
    Ok(Json(run_status_response_from_record(record)))
}
