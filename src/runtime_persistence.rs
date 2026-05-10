use super::*;

macro_rules! check_storage_quota {
    ($dir:literal, $lifecycle:ident) => {
        crate::storage_lifecycle::ensure_storage_quota(
            std::path::Path::new("storage"),
            $dir,
            crate::storage_lifecycle::StorageLifecycle::$lifecycle,
        )?;
    };
}

async fn atomic_write_json(path: &FsPath, value: &impl serde::Serialize) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    let json = serde_json::to_string_pretty(value)
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    fs::write(&tmp, json).await?;
    fs::rename(&tmp, path).await
}

pub(super) async fn persist_run_record(
    run_store_dir: &FsPath,
    record: &RunRecord,
) -> std::io::Result<()> {
    check_storage_quota!("runs", Temporary);
    fs::create_dir_all(run_store_dir).await?;
    let path = run_store_dir.join(format!("{}.json", record.run_id));
    atomic_write_json(&path, record).await
}

pub(super) async fn persist_backtest_record(
    backtest_store_dir: &FsPath,
    record: &BacktestRecord,
) -> std::io::Result<BacktestArtifactViews> {
    check_storage_quota!("backtests", Temporary);
    persist_backtest_artifacts(backtest_store_dir, record).await
}

pub(super) async fn persist_experiment_record(
    experiment_store_dir: &FsPath,
    record: &ExperimentRecord,
) -> std::io::Result<()> {
    check_storage_quota!("experiments", Temporary);
    fs::create_dir_all(experiment_store_dir).await?;
    let path = experiment_store_dir.join(format!("{}.json", record.experiment_id));
    atomic_write_json(&path, record).await
}

fn sanitize_storage_path_segment(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

const RUNTIME_REPORT_TRANSIENT_OUTPUT_TTL_MS: u64 = 24 * 60 * 60 * 1000;
const RUNTIME_REPORT_TRANSIENT_OUTPUT_PREFIXES: [&str; 2] =
    ["report-generation-tmp-", "report-generation-partial-"];

pub(super) fn runtime_evidence_cleanup_policy() -> RuntimeEvidenceCleanupPolicy {
    RuntimeEvidenceCleanupPolicy {
        policy_version: "quantpilot/evidence-cleanup/v1".to_string(),
        transient_generation_ttl_ms: RUNTIME_REPORT_TRANSIENT_OUTPUT_TTL_MS,
        protects_persisted_report_records: true,
        transient_output_prefixes: RUNTIME_REPORT_TRANSIENT_OUTPUT_PREFIXES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
    }
}

fn is_runtime_report_transient_output(name: &str) -> bool {
    RUNTIME_REPORT_TRANSIENT_OUTPUT_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

fn modified_at_ms(metadata: &std::fs::Metadata) -> u64 {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

pub(super) async fn cleanup_transient_runtime_report_outputs(
    report_store_dir: &FsPath,
    max_age_ms: u64,
    now_ms: u64,
) -> std::io::Result<usize> {
    if !fs::try_exists(report_store_dir).await? {
        return Ok(0);
    }

    let cutoff_ms = now_ms.saturating_sub(max_age_ms);
    let mut removed = 0;
    let mut entries = fs::read_dir(report_store_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if !is_runtime_report_transient_output(name) {
            continue;
        }
        let metadata = entry.metadata().await?;
        if modified_at_ms(&metadata) > cutoff_ms {
            continue;
        }
        if metadata.is_dir() {
            fs::remove_dir_all(path).await?;
        } else {
            fs::remove_file(path).await?;
        }
        removed += 1;
    }
    Ok(removed)
}

pub(super) async fn persist_runtime_report_record(
    report_store_dir: &FsPath,
    record: &RuntimeEvidenceReportRecord,
) -> std::io::Result<()> {
    check_storage_quota!("reports", Temporary);
    fs::create_dir_all(report_store_dir).await?;
    let path = report_store_dir.join(format!(
        "{}.json",
        sanitize_storage_path_segment(&record.report_id)
    ));
    atomic_write_json(&path, record).await
}

pub(super) async fn load_runtime_report_record(
    report_store_dir: &FsPath,
    report_id: &str,
) -> Result<RuntimeEvidenceReportRecord, (StatusCode, String)> {
    let path = report_store_dir.join(format!("{}.json", sanitize_storage_path_segment(report_id)));
    let content = fs::read_to_string(&path)
        .await
        .map_err(not_found_io_error)?;
    let record = serde_json::from_str(&content).map_err(|error| internal_error(error.into()))?;
    Ok(record)
}

pub(super) async fn list_runtime_report_records(
    report_store_dir: &PathBuf,
) -> std::io::Result<Vec<RuntimeEvidenceReportRecord>> {
    let mut records = Vec::new();
    if !fs::try_exists(report_store_dir).await? {
        return Ok(records);
    }

    let mut entries = fs::read_dir(report_store_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let content = fs::read_to_string(entry.path()).await?;
        if let Ok(record) = serde_json::from_str::<RuntimeEvidenceReportRecord>(&content) {
            records.push(record);
        }
    }

    Ok(records)
}

pub(super) async fn persist_runtime_parameter_mutation_record(
    mutation_store_dir: &FsPath,
    record: &RuntimeParameterMutationRecord,
) -> std::io::Result<()> {
    check_storage_quota!("mutations", Temporary);
    fs::create_dir_all(mutation_store_dir).await?;
    let path = mutation_store_dir.join(format!(
        "{}.json",
        sanitize_storage_path_segment(&record.proposal_id)
    ));
    atomic_write_json(&path, record).await
}

pub(super) async fn load_runtime_parameter_mutation_record(
    mutation_store_dir: &FsPath,
    proposal_id: &str,
) -> Result<RuntimeParameterMutationRecord, (StatusCode, String)> {
    let path = mutation_store_dir.join(format!(
        "{}.json",
        sanitize_storage_path_segment(proposal_id)
    ));
    let content = fs::read_to_string(&path)
        .await
        .map_err(not_found_io_error)?;
    let record = serde_json::from_str(&content).map_err(|error| internal_error(error.into()))?;
    Ok(record)
}

pub(super) async fn list_runtime_parameter_mutation_records(
    mutation_store_dir: &PathBuf,
) -> std::io::Result<Vec<RuntimeParameterMutationRecord>> {
    let mut records = Vec::new();
    if !fs::try_exists(mutation_store_dir).await? {
        return Ok(records);
    }

    let mut entries = fs::read_dir(mutation_store_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let content = fs::read_to_string(entry.path()).await?;
        if let Ok(record) = serde_json::from_str::<RuntimeParameterMutationRecord>(&content) {
            records.push(record);
        }
    }

    Ok(records)
}

pub(super) async fn persist_runtime_ai_proposal_record(
    ai_proposal_store_dir: &FsPath,
    record: &RuntimeAiProposalRecord,
) -> std::io::Result<()> {
    check_storage_quota!("ai-proposals", Transient);
    fs::create_dir_all(ai_proposal_store_dir).await?;
    let path = ai_proposal_store_dir.join(format!(
        "{}.json",
        sanitize_storage_path_segment(&record.ai_proposal_id)
    ));
    atomic_write_json(&path, record).await
}

pub(super) async fn load_runtime_ai_proposal_record(
    ai_proposal_store_dir: &FsPath,
    ai_proposal_id: &str,
) -> Result<RuntimeAiProposalRecord, (StatusCode, String)> {
    let path = ai_proposal_store_dir.join(format!(
        "{}.json",
        sanitize_storage_path_segment(ai_proposal_id)
    ));
    let content = fs::read_to_string(&path)
        .await
        .map_err(not_found_io_error)?;
    let record = serde_json::from_str(&content).map_err(|error| internal_error(error.into()))?;
    Ok(record)
}

pub(super) async fn list_runtime_ai_proposal_records(
    ai_proposal_store_dir: &PathBuf,
) -> std::io::Result<Vec<RuntimeAiProposalRecord>> {
    let mut records = Vec::new();
    if !fs::try_exists(ai_proposal_store_dir).await? {
        return Ok(records);
    }

    let mut entries = fs::read_dir(ai_proposal_store_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let content = fs::read_to_string(entry.path()).await?;
        if let Ok(record) = serde_json::from_str::<RuntimeAiProposalRecord>(&content) {
            records.push(record);
        }
    }

    Ok(records)
}

pub(super) async fn load_run_record_from_state(
    state: &AppState,
    run_id: &str,
) -> Result<RunRecord, (StatusCode, String)> {
    if let Some(record) = state.runs.read().await.get(run_id).cloned() {
        return Ok(normalize_run_record(
            record,
            RuntimeGovernanceMaterialization::CurrentRuntime,
        ));
    }

    let path = state.run_store_dir.join(format!("{}.json", run_id));
    let content = fs::read_to_string(&path)
        .await
        .map_err(not_found_io_error)?;
    let record = serde_json::from_str(&content).map_err(|error| internal_error(error.into()))?;
    Ok(normalize_run_record(
        record,
        RuntimeGovernanceMaterialization::LoadedManifest,
    ))
}

pub(super) async fn load_backtest_record_from_state(
    state: &AppState,
    backtest_id: &str,
) -> Result<BacktestRecord, (StatusCode, String)> {
    if let Some(record) = state.backtests.read().await.get(backtest_id).cloned() {
        return Ok(normalize_backtest_record(
            record,
            RuntimeGovernanceMaterialization::CurrentRuntime,
        ));
    }

    let dir = state.backtest_store_dir.join(backtest_id);
    if fs::try_exists(&dir).await.map_err(io_error)? {
        let record = load_backtest_record_from_directory(&dir)
            .await
            .map_err(io_error)?;
        return Ok(normalize_backtest_record(
            record,
            RuntimeGovernanceMaterialization::LoadedManifest,
        ));
    }

    if let Some(record) =
        load_transient_backtest_record(state.transient_backtest_store_dir.as_ref(), backtest_id)
            .await
            .map_err(io_error)?
    {
        return Ok(normalize_backtest_record(
            record,
            RuntimeGovernanceMaterialization::LoadedManifest,
        ));
    }

    Err((
        StatusCode::NOT_FOUND,
        format!("回测 `{}` 不存在", backtest_id),
    ))
}

pub(super) async fn load_experiment_record_from_state(
    state: &AppState,
    experiment_id: &str,
) -> Result<ExperimentRecord, (StatusCode, String)> {
    if let Some(record) = state.experiments.read().await.get(experiment_id).cloned() {
        return Ok(record);
    }

    let path = state
        .experiment_store_dir
        .join(format!("{}.json", experiment_id));
    let content = fs::read_to_string(&path)
        .await
        .map_err(not_found_io_error)?;
    let record = serde_json::from_str(&content).map_err(|error| internal_error(error.into()))?;
    Ok(record)
}

pub(super) async fn list_run_records(run_store_dir: &PathBuf) -> std::io::Result<Vec<RunRecord>> {
    let mut entries = fs::read_dir(run_store_dir).await?;
    let mut records = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let content = fs::read_to_string(entry.path()).await?;
        if let Ok(record) = serde_json::from_str::<RunRecord>(&content) {
            records.push(normalize_run_record(
                record,
                RuntimeGovernanceMaterialization::LoadedManifest,
            ));
        }
    }

    Ok(records)
}

pub(super) async fn list_backtest_records(
    backtest_store_dir: &PathBuf,
) -> std::io::Result<Vec<BacktestRecord>> {
    let mut entries = fs::read_dir(backtest_store_dir).await?;
    let mut records = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if is_backtest_promotion_work_dir(&path) {
            continue;
        }
        if let Ok(record) = load_backtest_record_from_directory(&path).await {
            records.push(normalize_backtest_record(
                record,
                RuntimeGovernanceMaterialization::LoadedManifest,
            ));
        }
    }

    Ok(records)
}

pub(super) async fn list_experiment_records(
    experiment_store_dir: &PathBuf,
) -> std::io::Result<Vec<ExperimentRecord>> {
    let mut records = Vec::new();
    if !fs::try_exists(experiment_store_dir).await? {
        return Ok(records);
    }

    let mut entries = fs::read_dir(experiment_store_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let content = fs::read_to_string(entry.path()).await?;
        if let Ok(record) = serde_json::from_str::<ExperimentRecord>(&content) {
            records.push(record);
        }
    }

    Ok(records)
}

// ── Block 5: 通用 JSON 持久化辅助函数 ──

pub(super) async fn persist_json<T: serde::Serialize>(
    store_dir: &FsPath,
    id: &str,
    data: &T,
) -> std::io::Result<()> {
    fs::create_dir_all(store_dir).await?;
    let file_path = store_dir.join(format!("{}.json", id));
    atomic_write_json(&file_path, data).await
}
