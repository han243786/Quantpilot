use crate::*;

const DEFAULT_CHAOS_MAX_DURATION_MS: u64 = 10_000;

pub(super) async fn execute_perturbation(
    store_dir: &FsPath,
    experiment_type: ChaosExperimentType,
    duration_ms: u64,
) {
    let clamped_duration_ms = duration_ms.min(max_duration_ms());

    match experiment_type {
        ChaosExperimentType::DiskPressureInjection => {
            let temp_dir = store_dir.join("temp_pressure");
            let _ = tokio::fs::create_dir_all(&temp_dir).await;
            for i in 0..10 {
                let data = vec![0u8; 1024 * 1024];
                let _ = tokio::fs::write(temp_dir.join(format!("pressure_{}.bin", i)), &data).await;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(clamped_duration_ms)).await;
            let _ = tokio::fs::remove_dir_all(&temp_dir).await;
        }
        ChaosExperimentType::DataLatencyInjection
        | ChaosExperimentType::EventLossInjection
        | ChaosExperimentType::ClockSkewInjection => {
            tokio::time::sleep(tokio::time::Duration::from_millis(clamped_duration_ms)).await;
        }
    }
}

fn max_duration_ms() -> u64 {
    resolve_max_duration_ms(
        std::env::var("QUANTPILOT_CHAOS_MAX_DURATION_MS")
            .ok()
            .as_deref(),
    )
}

fn resolve_max_duration_ms(value: Option<&str>) -> u64 {
    value
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_CHAOS_MAX_DURATION_MS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_max_duration_ms_accepts_parseable_override() {
        assert_eq!(resolve_max_duration_ms(Some("2500")), 2500);
    }

    #[test]
    fn resolve_max_duration_ms_falls_back_for_missing_or_invalid_override() {
        assert_eq!(resolve_max_duration_ms(None), DEFAULT_CHAOS_MAX_DURATION_MS);
        assert_eq!(
            resolve_max_duration_ms(Some("not-a-number")),
            DEFAULT_CHAOS_MAX_DURATION_MS
        );
    }
}
