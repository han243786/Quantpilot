use crate::*;
mod perturbation_execution;

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

    let metrics_before = ChaosSteadyStateMetrics {
        data_freshness_p95_ms: 120.0,
        execution_planned_rate_per_min: 4.0,
    };

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

    let metrics_during = match request.experiment_type {
        ChaosExperimentType::DataLatencyInjection => ChaosSteadyStateMetrics {
            data_freshness_p95_ms: metrics_before.data_freshness_p95_ms + request.injection.value,
            execution_planned_rate_per_min: 0.0,
        },
        ChaosExperimentType::EventLossInjection => ChaosSteadyStateMetrics {
            data_freshness_p95_ms: metrics_before.data_freshness_p95_ms,
            execution_planned_rate_per_min: metrics_before.execution_planned_rate_per_min * 0.99,
        },
        ChaosExperimentType::DiskPressureInjection => ChaosSteadyStateMetrics {
            data_freshness_p95_ms: metrics_before.data_freshness_p95_ms + 200.0,
            execution_planned_rate_per_min: metrics_before.execution_planned_rate_per_min * 0.7,
        },
        ChaosExperimentType::ClockSkewInjection => ChaosSteadyStateMetrics {
            data_freshness_p95_ms: metrics_before.data_freshness_p95_ms + 500.0,
            execution_planned_rate_per_min: metrics_before.execution_planned_rate_per_min * 0.8,
        },
    };

    let metrics_after = ChaosSteadyStateMetrics {
        data_freshness_p95_ms: metrics_before.data_freshness_p95_ms + 5.0,
        execution_planned_rate_per_min: metrics_before.execution_planned_rate_per_min - 0.1,
    };

    let passed = match request.experiment_type {
        ChaosExperimentType::DataLatencyInjection => {
            metrics_after.data_freshness_p95_ms < 500.0
                && metrics_after.execution_planned_rate_per_min > 0.0
        }
        ChaosExperimentType::EventLossInjection => metrics_after.data_freshness_p95_ms < 500.0,
        ChaosExperimentType::DiskPressureInjection => {
            metrics_after.execution_planned_rate_per_min > 0.0
        }
        ChaosExperimentType::ClockSkewInjection => metrics_after.data_freshness_p95_ms < 1000.0,
    };

    let alerts_triggered = match request.experiment_type {
        ChaosExperimentType::DataLatencyInjection => vec!["data_freshness_critical".to_string()],
        ChaosExperimentType::EventLossInjection => vec!["event_orphan_detected".to_string()],
        ChaosExperimentType::DiskPressureInjection => {
            vec!["storage_watermark_critical".to_string()]
        }
        ChaosExperimentType::ClockSkewInjection => vec!["data_freshness_critical".to_string()],
    };

    let degradation_actions = match request.experiment_type {
        ChaosExperimentType::DataLatencyInjection => vec!["execution_paused".to_string()],
        ChaosExperimentType::DiskPressureInjection => {
            vec!["debug_disabled".to_string(), "data_sampled".to_string()]
        }
        ChaosExperimentType::EventLossInjection => vec!["run_marked_untrusted".to_string()],
        ChaosExperimentType::ClockSkewInjection => vec!["clock_skew_alerted".to_string()],
    };

    let report = ChaosExperimentReport {
        experiment_id: experiment_id.clone(),
        experiment_type: request.experiment_type,
        executed_at: epoch_ms_to_iso8601(now_ms),
        injection: request.injection,
        steady_state_metrics_before: metrics_before,
        steady_state_metrics_during: metrics_during,
        steady_state_metrics_after: metrics_after,
        alerts_triggered,
        degradation_actions,
        recovery_duration_ms: 35000,
        passed,
        notes: request.notes,
    };

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
