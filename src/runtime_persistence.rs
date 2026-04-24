use super::*;

pub(super) async fn persist_run_record(
    run_store_dir: &PathBuf,
    record: &RunRecord,
) -> std::io::Result<()> {
    let path = run_store_dir.join(format!("{}.json", record.run_id));
    let body = serde_json::to_string_pretty(record)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error.to_string()))?;
    fs::write(path, body).await
}

pub(super) async fn persist_backtest_record(
    backtest_store_dir: &PathBuf,
    record: &BacktestRecord,
) -> std::io::Result<BacktestArtifactViews> {
    persist_backtest_artifacts(backtest_store_dir, record).await
}

pub(super) async fn persist_experiment_record(
    experiment_store_dir: &PathBuf,
    record: &ExperimentRecord,
) -> std::io::Result<()> {
    fs::create_dir_all(experiment_store_dir).await?;
    let path = experiment_store_dir.join(format!("{}.json", record.experiment_id));
    let body = serde_json::to_string_pretty(record)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error.to_string()))?;
    fs::write(path, body).await
}

pub(super) async fn load_run_record_from_state(
    state: &AppState,
    run_id: &str,
) -> Result<RunRecord, (StatusCode, String)> {
    if let Some(record) = state.runs.read().await.get(run_id).cloned() {
        return Ok(normalize_run_record(record));
    }

    let path = state.run_store_dir.join(format!("{}.json", run_id));
    let content = fs::read_to_string(&path)
        .await
        .map_err(not_found_io_error)?;
    let record = serde_json::from_str(&content).map_err(|error| internal_error(error.into()))?;
    Ok(normalize_run_record(record))
}

pub(super) async fn load_backtest_record_from_state(
    state: &AppState,
    backtest_id: &str,
) -> Result<BacktestRecord, (StatusCode, String)> {
    if let Some(record) = state.backtests.read().await.get(backtest_id).cloned() {
        return Ok(normalize_backtest_record(record));
    }

    let dir = state.backtest_store_dir.join(backtest_id);
    if fs::try_exists(&dir).await.map_err(io_error)? {
        let record = load_backtest_record_from_directory(&dir)
            .await
            .map_err(io_error)?;
        return Ok(normalize_backtest_record(record));
    }

    Err((
        StatusCode::NOT_FOUND,
        format!("backtest `{}` not found", backtest_id),
    ))
}

pub(super) async fn load_experiment_record_from_state(
    state: &AppState,
    experiment_id: &str,
) -> Result<ExperimentRecord, (StatusCode, String)> {
    if let Some(record) = state.experiments.read().await.get(experiment_id).cloned() {
        return Ok(record);
    }

    let path = state.experiment_store_dir.join(format!("{}.json", experiment_id));
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
            records.push(normalize_run_record(record));
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
        if let Ok(record) = load_backtest_record_from_directory(&path).await {
            records.push(normalize_backtest_record(record));
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
