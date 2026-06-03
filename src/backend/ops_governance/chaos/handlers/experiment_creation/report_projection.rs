use crate::*;

pub(super) fn baseline_metrics() -> ChaosSteadyStateMetrics {
    ChaosSteadyStateMetrics {
        data_freshness_p95_ms: 120.0,
        execution_planned_rate_per_min: 4.0,
    }
}

pub(super) fn build_experiment_report(
    experiment_id: String,
    now_ms: u64,
    request: CreateChaosExperimentRequest,
    metrics_before: ChaosSteadyStateMetrics,
) -> ChaosExperimentReport {
    let metrics_during =
        project_metrics_during(request.experiment_type, &request.injection, &metrics_before);
    let metrics_after = project_metrics_after(&metrics_before);
    let passed = passed(request.experiment_type, &metrics_after);
    let alerts_triggered = alerts_triggered(request.experiment_type);
    let degradation_actions = degradation_actions(request.experiment_type);

    ChaosExperimentReport {
        experiment_id,
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
    }
}

fn project_metrics_during(
    experiment_type: ChaosExperimentType,
    injection: &ChaosInjectionSpec,
    metrics_before: &ChaosSteadyStateMetrics,
) -> ChaosSteadyStateMetrics {
    match experiment_type {
        ChaosExperimentType::DataLatencyInjection => ChaosSteadyStateMetrics {
            data_freshness_p95_ms: metrics_before.data_freshness_p95_ms + injection.value,
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
    }
}

fn project_metrics_after(metrics_before: &ChaosSteadyStateMetrics) -> ChaosSteadyStateMetrics {
    ChaosSteadyStateMetrics {
        data_freshness_p95_ms: metrics_before.data_freshness_p95_ms + 5.0,
        execution_planned_rate_per_min: metrics_before.execution_planned_rate_per_min - 0.1,
    }
}

fn passed(experiment_type: ChaosExperimentType, metrics_after: &ChaosSteadyStateMetrics) -> bool {
    match experiment_type {
        ChaosExperimentType::DataLatencyInjection => {
            metrics_after.data_freshness_p95_ms < 500.0
                && metrics_after.execution_planned_rate_per_min > 0.0
        }
        ChaosExperimentType::EventLossInjection => metrics_after.data_freshness_p95_ms < 500.0,
        ChaosExperimentType::DiskPressureInjection => {
            metrics_after.execution_planned_rate_per_min > 0.0
        }
        ChaosExperimentType::ClockSkewInjection => metrics_after.data_freshness_p95_ms < 1000.0,
    }
}

fn alerts_triggered(experiment_type: ChaosExperimentType) -> Vec<String> {
    match experiment_type {
        ChaosExperimentType::DataLatencyInjection => vec!["data_freshness_critical".to_string()],
        ChaosExperimentType::EventLossInjection => vec!["event_orphan_detected".to_string()],
        ChaosExperimentType::DiskPressureInjection => {
            vec!["storage_watermark_critical".to_string()]
        }
        ChaosExperimentType::ClockSkewInjection => vec!["data_freshness_critical".to_string()],
    }
}

fn degradation_actions(experiment_type: ChaosExperimentType) -> Vec<String> {
    match experiment_type {
        ChaosExperimentType::DataLatencyInjection => vec!["execution_paused".to_string()],
        ChaosExperimentType::DiskPressureInjection => {
            vec!["debug_disabled".to_string(), "data_sampled".to_string()]
        }
        ChaosExperimentType::EventLossInjection => vec!["run_marked_untrusted".to_string()],
        ChaosExperimentType::ClockSkewInjection => vec!["clock_skew_alerted".to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_report_preserves_latency_projection() {
        let report = build_experiment_report(
            "chaos-1".to_string(),
            1_000,
            request(ChaosExperimentType::DataLatencyInjection, 25.0),
            baseline_metrics(),
        );

        assert_eq!(report.experiment_id, "chaos-1");
        assert_eq!(
            report.steady_state_metrics_before.data_freshness_p95_ms,
            120.0
        );
        assert_eq!(
            report.steady_state_metrics_during.data_freshness_p95_ms,
            145.0
        );
        assert_eq!(
            report
                .steady_state_metrics_during
                .execution_planned_rate_per_min,
            0.0
        );
        assert_eq!(
            report.steady_state_metrics_after.data_freshness_p95_ms,
            125.0
        );
        assert_eq!(
            report
                .steady_state_metrics_after
                .execution_planned_rate_per_min,
            3.9
        );
        assert_eq!(report.alerts_triggered, vec!["data_freshness_critical"]);
        assert_eq!(report.degradation_actions, vec!["execution_paused"]);
        assert!(report.passed);
    }

    #[test]
    fn build_report_preserves_disk_projection_actions_and_notes() {
        let report = build_experiment_report(
            "chaos-2".to_string(),
            1_000,
            request(ChaosExperimentType::DiskPressureInjection, 0.0),
            baseline_metrics(),
        );

        assert_eq!(
            report.steady_state_metrics_during.data_freshness_p95_ms,
            320.0
        );
        assert_eq!(
            report
                .steady_state_metrics_during
                .execution_planned_rate_per_min,
            2.8
        );
        assert_eq!(report.alerts_triggered, vec!["storage_watermark_critical"]);
        assert_eq!(
            report.degradation_actions,
            vec!["debug_disabled", "data_sampled"]
        );
        assert_eq!(report.notes.as_deref(), Some("projection-test"));
        assert_eq!(report.recovery_duration_ms, 35000);
        assert!(report.passed);
    }

    fn request(experiment_type: ChaosExperimentType, value: f64) -> CreateChaosExperimentRequest {
        CreateChaosExperimentRequest {
            experiment_type,
            injection: ChaosInjectionSpec {
                target: "data_module".to_string(),
                parameter: "test".to_string(),
                value,
                duration_ms: 100,
            },
            notes: Some("projection-test".to_string()),
        }
    }
}
