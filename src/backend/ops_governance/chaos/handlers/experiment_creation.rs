use crate::*;
mod perturbation_execution;
mod report_projection;

pub(super) async fn create_experiment(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Json(request): Json<CreateChaosExperimentRequest>,
) -> Result<Json<ChaosExperimentReport>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    let experiment_id = format!("chaos-{}", now_ms);

    state
        .chaos_mode
        .store(true, std::sync::atomic::Ordering::SeqCst);

    let _events_before = state
        .evidence_metrics
        .compact_projection_source_event_count_total
        .load(Ordering::Relaxed);
    let _retained_before = state
        .evidence_metrics
        .compact_projection_retained_event_count_total
        .load(Ordering::Relaxed);
    let _failures_before = state
        .evidence_metrics
        .report_generation_failure_count
        .load(Ordering::Relaxed);

    let metrics_before = baseline_metrics();

    execute_perturbation(
        state.chaos_store_dir.as_ref(),
        request.experiment_type,
        request.injection.duration_ms,
    )
    .await;

    state
        .chaos_mode
        .store(false, std::sync::atomic::Ordering::SeqCst);

    let _events_after = state
        .evidence_metrics
        .compact_projection_source_event_count_total
        .load(Ordering::Relaxed);
    let _failures_after = state
        .evidence_metrics
        .report_generation_failure_count
        .load(Ordering::Relaxed);

    let report = build_experiment_report(experiment_id.clone(), now_ms, request, metrics_before);

    super::persist_chaos_report(&state.chaos_store_dir, &report)
        .await
        .map_err(io_error)?;
    state
        .chaos_experiments
        .write()
        .await
        .insert(auth::scoped_key(&user_id, &experiment_id), report.clone());

    Ok(Json(report))
}

async fn execute_perturbation(
    store_dir: &FsPath,
    experiment_type: ChaosExperimentType,
    duration_ms: u64,
) {
    perturbation_execution::execute_perturbation(store_dir, experiment_type, duration_ms).await
}

fn baseline_metrics() -> ChaosSteadyStateMetrics {
    report_projection::baseline_metrics()
}

fn build_experiment_report(
    experiment_id: String,
    now_ms: u64,
    request: CreateChaosExperimentRequest,
    metrics_before: ChaosSteadyStateMetrics,
) -> ChaosExperimentReport {
    report_projection::build_experiment_report(experiment_id, now_ms, request, metrics_before)
}
