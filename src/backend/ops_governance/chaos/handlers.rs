use crate::*;
mod experiment_creation;
mod read_routes;
mod report_persistence;
mod route_facade;

pub(super) fn register_chaos_routes(router: Router<AppState>) -> Router<AppState> {
    route_facade::register_chaos_routes(router)
}

async fn create_experiment(
    user_id: auth::UserId,
    state: State<AppState>,
    request: Json<CreateChaosExperimentRequest>,
) -> Result<Json<ChaosExperimentReport>, (StatusCode, String)> {
    experiment_creation::create_experiment(user_id, state, request).await
}

async fn list_experiments(
    user_id: auth::UserId,
    state: State<AppState>,
) -> Result<Json<Vec<ChaosExperimentReport>>, (StatusCode, String)> {
    read_routes::list_experiments(user_id, state).await
}

async fn get_experiment(
    user_id: auth::UserId,
    state: State<AppState>,
    experiment_id: Path<String>,
) -> Result<Json<ChaosExperimentReport>, (StatusCode, String)> {
    read_routes::get_experiment(user_id, state, experiment_id).await
}

async fn persist_chaos_report(
    store_dir: &FsPath,
    report: &ChaosExperimentReport,
) -> std::io::Result<()> {
    report_persistence::persist_chaos_report(store_dir, report).await
}

async fn load_chaos_report_from_disk(
    store_dir: &FsPath,
    experiment_id: &str,
) -> Result<ChaosExperimentReport, (StatusCode, String)> {
    report_persistence::load_chaos_report_from_disk(store_dir, experiment_id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_four_experiment_types_are_distinct() {
        let types = [
            ChaosExperimentType::DataLatencyInjection,
            ChaosExperimentType::EventLossInjection,
            ChaosExperimentType::DiskPressureInjection,
            ChaosExperimentType::ClockSkewInjection,
        ];
        for i in 0..types.len() {
            for j in (i + 1)..types.len() {
                assert_ne!(types[i], types[j]);
            }
        }
    }

    #[test]
    fn injection_spec_holds_duration() {
        let spec = ChaosInjectionSpec {
            target: "data_module".to_string(),
            parameter: "artificial_latency_ms".to_string(),
            value: 1500.0,
            duration_ms: 120_000,
        };
        assert_eq!(spec.duration_ms, 120_000);
        assert_eq!(spec.value, 1500.0);
    }
}
