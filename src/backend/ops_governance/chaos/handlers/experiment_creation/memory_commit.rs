use crate::*;

pub(super) async fn commit_experiment_to_memory(
    experiments: &tokio::sync::RwLock<std::collections::BTreeMap<String, ChaosExperimentReport>>,
    user_id: &auth::UserId,
    experiment_id: &str,
    report: &ChaosExperimentReport,
) {
    experiments
        .write()
        .await
        .insert(auth::scoped_key(user_id, experiment_id), report.clone());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn commit_experiment_to_memory_inserts_scoped_report_clone() {
        let experiments = tokio::sync::RwLock::new(std::collections::BTreeMap::new());
        let report = report("chaos-commit");
        let user_id = auth::UserId(42);

        commit_experiment_to_memory(&experiments, &user_id, "chaos-commit", &report).await;

        let experiments = experiments.read().await;
        let stored = experiments
            .get(&auth::scoped_key(&user_id, "chaos-commit"))
            .expect("stored report");
        assert_eq!(stored.experiment_id, report.experiment_id);
        assert_eq!(stored.notes.as_deref(), Some("memory-commit"));
    }

    fn report(experiment_id: &str) -> ChaosExperimentReport {
        ChaosExperimentReport {
            experiment_id: experiment_id.to_string(),
            experiment_type: ChaosExperimentType::EventLossInjection,
            executed_at: "1970-01-01T00:00:01.000Z".to_string(),
            injection: ChaosInjectionSpec {
                target: "data_module".to_string(),
                parameter: "test".to_string(),
                value: 0.0,
                duration_ms: 100,
            },
            steady_state_metrics_before: metrics(),
            steady_state_metrics_during: metrics(),
            steady_state_metrics_after: metrics(),
            alerts_triggered: vec!["event_orphan_detected".to_string()],
            degradation_actions: vec!["run_marked_untrusted".to_string()],
            recovery_duration_ms: 35000,
            passed: true,
            notes: Some("memory-commit".to_string()),
        }
    }

    fn metrics() -> ChaosSteadyStateMetrics {
        ChaosSteadyStateMetrics {
            data_freshness_p95_ms: 120.0,
            execution_planned_rate_per_min: 4.0,
        }
    }
}
