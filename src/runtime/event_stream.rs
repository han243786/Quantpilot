use crate::{
    auth, json_sse_event, load_run_record_from_state, sleep, stream, AppState, Duration, Event,
    Infallible, KeepAlive, Path, Sse, State, StatusCode, SSE_EVENT_DELAY_MS,
};

pub(crate) async fn stream_run_events(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)>
{
    let record = load_run_record_from_state(&state, &user_id, &run_id).await?;
    let event_count = record.events.len();

    let stream = stream! {
        yield Ok(json_sse_event("run_started", serde_json::json!({
            "run_id": record.run_id,
            "graph_id": record.graph_id,
            "compile_id": record.compile_id,
            "status": "started"
        })));

        for event in record.events {
            yield Ok(json_sse_event("runtime_event", &event));
            sleep(Duration::from_millis(SSE_EVENT_DELAY_MS)).await;
        }

        yield Ok(json_sse_event("account", &record.account));

        yield Ok(json_sse_event("run_completed", serde_json::json!({
            "run_id": record.run_id,
            "status": "completed",
            "event_count": event_count,
        })));
    };

    // v2.4.0 NOTE: SSE 超时保护需要 tokio-stream 依赖或 stream 级别 timeout wrapper,
    // Axum 0.7 的 Sse 类型不提供 max_age。当前由 TCP keepalive + 浏览器端超时处理。
    // 计划 v2.5.0 添加 tokio-stream 依赖后实现。
    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(5))
            .text("keepalive"),
    ))
}
