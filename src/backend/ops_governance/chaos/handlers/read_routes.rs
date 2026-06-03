use crate::*;

pub(super) async fn list_experiments(
    user_id: auth::UserId,
    State(state): State<AppState>,
) -> Result<Json<Vec<ChaosExperimentReport>>, (StatusCode, String)> {
    let experiments = state.chaos_experiments.read().await;
    Ok(Json(sorted_user_reports(&experiments, &user_id)))
}

pub(super) async fn get_experiment(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<ChaosExperimentReport>, (StatusCode, String)> {
    if let Some(report) = {
        let experiments = state.chaos_experiments.read().await;
        find_scoped_report(&experiments, &user_id, &experiment_id)
    } {
        return Ok(Json(report));
    }

    super::load_chaos_report_from_disk(&state.chaos_store_dir, &experiment_id)
        .await
        .map(Json)
}

fn sorted_user_reports(
    experiments: &std::collections::BTreeMap<String, ChaosExperimentReport>,
    user_id: &auth::UserId,
) -> Vec<ChaosExperimentReport> {
    let prefix = auth::scoped_key(user_id, "");
    let mut reports: Vec<ChaosExperimentReport> = experiments
        .iter()
        .filter(|(key, _)| key.starts_with(&prefix))
        .map(|(_, value)| value.clone())
        .collect();
    reports.sort_by(|a, b| b.executed_at.cmp(&a.executed_at));
    reports
}

fn find_scoped_report(
    experiments: &std::collections::BTreeMap<String, ChaosExperimentReport>,
    user_id: &auth::UserId,
    experiment_id: &str,
) -> Option<ChaosExperimentReport> {
    experiments
        .get(&auth::scoped_key(user_id, experiment_id))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted_user_reports_filters_by_scope_and_sorts_newest_first() {
        let user_id = auth::UserId(7);
        let other_user_id = auth::UserId(8);
        let mut experiments = std::collections::BTreeMap::new();
        experiments.insert(
            auth::scoped_key(&user_id, "old"),
            report("old", "2026-01-01T00:00:00.000Z"),
        );
        experiments.insert(
            auth::scoped_key(&user_id, "new"),
            report("new", "2026-01-02T00:00:00.000Z"),
        );
        experiments.insert(
            auth::scoped_key(&other_user_id, "other"),
            report("other", "2026-01-03T00:00:00.000Z"),
        );

        let reports = sorted_user_reports(&experiments, &user_id);

        assert_eq!(reports.len(), 2);
        assert_eq!(reports[0].experiment_id, "new");
        assert_eq!(reports[1].experiment_id, "old");
    }

    #[test]
    fn find_scoped_report_returns_only_matching_user_report() {
        let user_id = auth::UserId(7);
        let other_user_id = auth::UserId(8);
        let mut experiments = std::collections::BTreeMap::new();
        experiments.insert(
            auth::scoped_key(&other_user_id, "same-id"),
            report("same-id", "2026-01-03T00:00:00.000Z"),
        );
        experiments.insert(
            auth::scoped_key(&user_id, "same-id"),
            report("same-id", "2026-01-02T00:00:00.000Z"),
        );

        let report = find_scoped_report(&experiments, &user_id, "same-id").expect("scoped report");

        assert_eq!(report.executed_at, "2026-01-02T00:00:00.000Z");
    }

    fn report(experiment_id: &str, executed_at: &str) -> ChaosExperimentReport {
        ChaosExperimentReport {
            experiment_id: experiment_id.to_string(),
            experiment_type: ChaosExperimentType::EventLossInjection,
            executed_at: executed_at.to_string(),
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
            notes: None,
        }
    }

    fn metrics() -> ChaosSteadyStateMetrics {
        ChaosSteadyStateMetrics {
            data_freshness_p95_ms: 120.0,
            execution_planned_rate_per_min: 4.0,
        }
    }
}
