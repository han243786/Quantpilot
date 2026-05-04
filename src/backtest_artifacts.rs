use super::{
    AccountSummary, ActorIdentity, BacktestRecord, FrontendRuntimeEvent, RuntimeGovernanceSnapshot,
};
use anyhow::{anyhow, Context};
use qrpc_core::{
    canonical_json_sha256_digest, ArtifactDigest, BacktestEquityPoint, BacktestOutput,
    BacktestSpec, BacktestSummary, CompileArtifactBundle, ExecutionAssumptionSourceSummary,
    ExecutionAssumptionSpec, ExecutionAssumptionValueSource, ExecutionStatus, OrderSide,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::fs;

pub const EVENT_LOG_ARTIFACT_V1_VERSION: &str = "quantpilot/event-log-artifact/v1";
pub const TRADE_LEDGER_ARTIFACT_V1_VERSION: &str = "quantpilot/trade-ledger-artifact/v1";
pub const EQUITY_CURVE_ARTIFACT_V1_VERSION: &str = "quantpilot/equity-curve-artifact/v1";
pub const METRICS_ARTIFACT_V1_VERSION: &str = "quantpilot/metrics-artifact/v1";
pub const REPRODUCIBILITY_MANIFEST_V1_VERSION: &str = "quantpilot/reproducibility-manifest/v1";

const MANIFEST_FILE: &str = "manifest.json";
const EVENT_LOG_FILE: &str = "event_log.json";
const TRADE_LEDGER_FILE: &str = "trade_ledger.json";
const EQUITY_CURVE_FILE: &str = "equity_curve.json";
const METRICS_FILE: &str = "metrics.json";
const BACKTEST_OUTPUT_FILE: &str = "backtest_output.json";
const TRANSIENT_METADATA_FILE: &str = "transient_metadata.json";
const SAVING_DIR_PREFIX: &str = ".saving-";
const REPLACING_DIR_PREFIX: &str = ".replacing-";
const TRANSIENT_BACKTEST_DIR_PREFIX: &str = "transient-backtest-";
const TRANSIENT_SAVING_DIR_PREFIX: &str = ".saving-transient-backtest-";
const PROMOTION_WORK_DIR_TTL_MS: u64 = 24 * 60 * 60 * 1000;
const PROMOTION_WORK_DIR_MAX_COUNT: usize = 32;
const PROMOTION_WORK_DIR_MAX_BYTES: u64 = 512 * 1024 * 1024;
const PROMOTION_WORK_DIR_MAX_SINGLE_BYTES: u64 = 256 * 1024 * 1024;
pub const DEFAULT_TRANSIENT_BACKTEST_SPILL_THRESHOLD_BYTES: u64 = 4 * 1024 * 1024;
const TRANSIENT_BACKTEST_TTL_MS: u64 = 24 * 60 * 60 * 1000;
const TRANSIENT_BACKTEST_MAX_COUNT: usize = 32;
const TRANSIENT_BACKTEST_MAX_BYTES: u64 = 512 * 1024 * 1024;
const TRANSIENT_BACKTEST_MAX_SINGLE_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogArtifact {
    pub schema_version: String,
    pub artifact_id: String,
    pub backtest_id: String,
    pub event_count: usize,
    pub digest: ArtifactDigest,
    pub events: Vec<FrontendRuntimeEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeLedgerEntry {
    pub fill_id: String,
    pub plan_id: String,
    pub exchange: String,
    pub symbol: String,
    pub side: OrderSide,
    pub filled_qty: f64,
    pub filled_price: f64,
    pub fee_paid: f64,
    pub filled_at_ms: u64,
    pub status: ExecutionStatus,
    pub trace_id: String,
    pub session_index: usize,
    pub cycle_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeLedgerArtifact {
    pub schema_version: String,
    pub artifact_id: String,
    pub backtest_id: String,
    pub trade_count: usize,
    pub digest: ArtifactDigest,
    #[serde(default)]
    pub summary: Option<TradeLedgerSummary>,
    pub trades: Vec<TradeLedgerEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradeLedgerSummary {
    pub trade_count: usize,
    pub buy_fill_count: usize,
    pub sell_fill_count: usize,
    pub total_fees_paid: f64,
    pub buy_fees_paid: f64,
    pub sell_fees_paid: f64,
    pub total_filled_notional: f64,
    pub buy_filled_notional: f64,
    pub sell_filled_notional: f64,
    pub average_fill_price: f64,
    pub average_buy_fill_price: Option<f64>,
    pub average_sell_fill_price: Option<f64>,
    pub average_fee_per_fill: f64,
    pub average_buy_fee: Option<f64>,
    pub average_sell_fee: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityCurveArtifact {
    pub schema_version: String,
    pub artifact_id: String,
    pub backtest_id: String,
    pub point_count: usize,
    pub digest: ArtifactDigest,
    pub points: Vec<BacktestEquityPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsArtifact {
    pub schema_version: String,
    pub artifact_id: String,
    pub backtest_id: String,
    pub digest: ArtifactDigest,
    pub summary: BacktestSummary,
    pub event_count: usize,
    pub session_count: usize,
    pub started_at_ms: u64,
    pub ended_at_ms: u64,
    pub final_account: AccountSummary,
    #[serde(default)]
    pub execution_assumptions: Option<ExecutionAssumptionsModule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionAssumptionsSummary {
    pub fee_bps: f64,
    pub slippage_bps: f64,
    pub latency_ms: u64,
    #[serde(default)]
    pub sources: Option<ExecutionAssumptionSourceSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionAssumptionsTag {
    pub label: String,
    pub sources_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionAssumptionsModule {
    pub summary: ExecutionAssumptionsSummary,
    pub list_tag: ExecutionAssumptionsTag,
}

impl From<&ExecutionAssumptionSpec> for ExecutionAssumptionsSummary {
    fn from(value: &ExecutionAssumptionSpec) -> Self {
        Self {
            fee_bps: value.taker_fee_bps,
            slippage_bps: value.default_slippage_bps,
            latency_ms: value.latency_assumption_ms.unwrap_or(0),
            sources: None,
        }
    }
}

impl ExecutionAssumptionsModule {
    pub fn from_summary(summary: ExecutionAssumptionsSummary) -> Self {
        let list_tag = ExecutionAssumptionsTag::from(&summary);
        Self { summary, list_tag }
    }
}

impl From<&ExecutionAssumptionsSummary> for ExecutionAssumptionsTag {
    fn from(value: &ExecutionAssumptionsSummary) -> Self {
        let sources_label = value
            .sources
            .as_ref()
            .map(|sources| {
                format!(
                    "fee:{} slip:{} lat:{}",
                    execution_assumption_source_compact_label(&sources.fee_bps),
                    execution_assumption_source_compact_label(&sources.slippage_bps),
                    execution_assumption_source_compact_label(&sources.latency_ms)
                )
            })
            .unwrap_or_else(|| "fee:na slip:na lat:na".to_string());
        Self {
            label: format!(
                "fee={} slip={} lat={}",
                value.fee_bps, value.slippage_bps, value.latency_ms
            ),
            sources_label,
        }
    }
}

fn execution_assumption_source_compact_label(
    source: &ExecutionAssumptionValueSource,
) -> &'static str {
    match source {
        ExecutionAssumptionValueSource::RequestOverride => "req",
        ExecutionAssumptionValueSource::ProfileDefault => "profile",
        ExecutionAssumptionValueSource::BackendFallback => "backend",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactFileRef {
    pub kind: String,
    pub artifact_id: String,
    pub digest: ArtifactDigest,
    pub file_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReproducibilityManifest {
    pub schema_version: String,
    pub manifest_id: String,
    pub backtest_id: String,
    pub graph_id: String,
    pub compile_id: String,
    pub created_at_ms: u64,
    pub protocol_name: String,
    pub config_hash: String,
    pub account: AccountSummary,
    pub summary: BacktestSummary,
    #[serde(default)]
    pub backtest_spec: Option<BacktestSpec>,
    #[serde(default)]
    pub compile_artifacts: Option<CompileArtifactBundle>,
    #[serde(default)]
    pub governance: RuntimeGovernanceSnapshot,
    pub output_artifacts: Vec<ArtifactFileRef>,
    pub backtest_output_digest: ArtifactDigest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestArtifactViews {
    pub event_log: EventLogArtifact,
    pub trade_ledger: TradeLedgerArtifact,
    pub equity_curve: EquityCurveArtifact,
    pub metrics: MetricsArtifact,
    pub manifest: ReproducibilityManifest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TransientBacktestMetadata {
    #[serde(default)]
    actor: Option<ActorIdentity>,
}

pub fn build_backtest_artifact_views(
    record: &BacktestRecord,
) -> anyhow::Result<BacktestArtifactViews> {
    let event_log = build_event_log_artifact(&record.backtest_id, &record.events)?;
    let trade_ledger = build_trade_ledger_artifact(&record.backtest_id, &event_log)?;
    let projected_portfolios = project_session_portfolios(&event_log)?;
    let equity_curve = build_equity_curve_artifact(&record.backtest_id, &projected_portfolios)?;
    let metrics = build_metrics_artifact(
        &record.backtest_id,
        &event_log,
        &trade_ledger,
        &equity_curve,
        &projected_portfolios,
        record
            .backtest_spec
            .as_ref()
            .map(|spec| &spec.run_spec.execution_assumptions),
        record
            .backtest_spec
            .as_ref()
            .and_then(|spec| spec.run_spec.execution_assumption_sources.as_ref()),
    )?;
    let backtest_output_digest =
        canonical_json_sha256_digest(&record.backtest).context("failed to hash backtest output")?;
    let manifest = build_reproducibility_manifest(
        record,
        &event_log,
        &trade_ledger,
        &equity_curve,
        &metrics,
        backtest_output_digest,
    );

    Ok(BacktestArtifactViews {
        event_log,
        trade_ledger,
        equity_curve,
        metrics,
        manifest,
    })
}

pub async fn persist_backtest_artifacts(
    backtest_store_dir: &Path,
    record: &BacktestRecord,
) -> std::io::Result<BacktestArtifactViews> {
    let views = build_backtest_artifact_views(record).map_err(to_io_error)?;
    let dir = backtest_store_dir.join(&record.backtest_id);
    fs::create_dir_all(backtest_store_dir).await?;
    cleanup_backtest_promotion_work_dirs(backtest_store_dir).await?;
    enforce_backtest_promotion_work_quota(backtest_store_dir).await?;

    let temp_dir =
        unique_backtest_promotion_dir(backtest_store_dir, SAVING_DIR_PREFIX, &record.backtest_id);
    fs::create_dir_all(&temp_dir).await?;

    let result = async {
        write_backtest_artifact_bundle(&temp_dir, record, &views).await?;
        enforce_single_promotion_work_dir_quota(&temp_dir).await?;
        validate_backtest_artifact_bundle(&temp_dir, record).await?;
        promote_backtest_artifact_dir(backtest_store_dir, &temp_dir, &dir, &record.backtest_id)
            .await
    }
    .await;

    if let Err(error) = result {
        let _ = fs::remove_dir_all(&temp_dir).await;
        return Err(error);
    }

    Ok(views)
}

pub async fn cleanup_backtest_promotion_work_dirs(
    backtest_store_dir: &Path,
) -> std::io::Result<()> {
    cleanup_expired_backtest_promotion_work_dirs(backtest_store_dir, current_epoch_ms()).await
}

pub async fn maybe_spill_transient_backtest_record(
    transient_store_dir: &Path,
    record: &BacktestRecord,
    threshold_bytes: u64,
) -> std::io::Result<bool> {
    if !should_spill_transient_backtest_record(record, threshold_bytes)? {
        return Ok(false);
    }

    persist_transient_backtest_record(transient_store_dir, record).await?;
    Ok(true)
}

pub fn should_spill_transient_backtest_record(
    record: &BacktestRecord,
    threshold_bytes: u64,
) -> std::io::Result<bool> {
    let bytes = serde_json::to_vec(record).map_err(to_io_error)?;
    Ok(bytes.len() as u64 > threshold_bytes)
}

pub async fn persist_transient_backtest_record(
    transient_store_dir: &Path,
    record: &BacktestRecord,
) -> std::io::Result<()> {
    let views = record
        .backtest_artifacts
        .clone()
        .map(Ok)
        .unwrap_or_else(|| build_backtest_artifact_views(record).map_err(to_io_error))?;

    fs::create_dir_all(transient_store_dir).await?;
    cleanup_transient_backtest_records(transient_store_dir).await?;
    ensure_has_room_for_transient_backtest(transient_store_dir).await?;

    let final_dir = transient_backtest_record_dir(transient_store_dir, &record.backtest_id);
    let temp_dir = unique_transient_backtest_work_dir(transient_store_dir, &record.backtest_id);
    fs::create_dir_all(&temp_dir).await?;

    let result = async {
        write_backtest_artifact_bundle(&temp_dir, record, &views).await?;
        write_json(
            temp_dir.join(TRANSIENT_METADATA_FILE),
            &TransientBacktestMetadata {
                actor: record.actor.clone(),
            },
        )
        .await?;
        enforce_single_transient_backtest_quota(&temp_dir).await?;
        validate_backtest_artifact_bundle(&temp_dir, record).await?;
        if fs::try_exists(&final_dir).await? {
            fs::remove_dir_all(&final_dir).await?;
        }
        fs::rename(&temp_dir, &final_dir).await?;
        enforce_transient_backtest_quota(transient_store_dir).await
    }
    .await;

    if let Err(error) = result {
        let _ = fs::remove_dir_all(&temp_dir).await;
        let _ = fs::remove_dir_all(&final_dir).await;
        return Err(error);
    }

    Ok(())
}

pub async fn load_transient_backtest_record(
    transient_store_dir: &Path,
    backtest_id: &str,
) -> std::io::Result<Option<BacktestRecord>> {
    let dir = transient_backtest_record_dir(transient_store_dir, backtest_id);
    if !fs::try_exists(&dir).await? {
        return Ok(None);
    }

    let mut record = load_backtest_record_from_directory(&dir).await?;
    if let Ok(metadata) =
        read_json::<TransientBacktestMetadata>(dir.join(TRANSIENT_METADATA_FILE)).await
    {
        record.actor = metadata.actor;
    }
    Ok(Some(record))
}

pub async fn delete_transient_backtest_record(
    transient_store_dir: &Path,
    backtest_id: &str,
) -> std::io::Result<bool> {
    let dir = transient_backtest_record_dir(transient_store_dir, backtest_id);
    if !fs::try_exists(&dir).await? {
        return Ok(false);
    }
    fs::remove_dir_all(dir).await?;
    Ok(true)
}

pub async fn cleanup_transient_backtest_records(transient_store_dir: &Path) -> std::io::Result<()> {
    cleanup_expired_transient_backtest_records(transient_store_dir, current_epoch_ms()).await
}

async fn cleanup_expired_transient_backtest_records(
    transient_store_dir: &Path,
    now_ms: u64,
) -> std::io::Result<()> {
    if !fs::try_exists(transient_store_dir).await? {
        return Ok(());
    }

    let mut entries = fs::read_dir(transient_store_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if !path.is_dir() || !is_transient_backtest_dir(&path) {
            continue;
        }
        if is_transient_backtest_expired(&path, now_ms).await? {
            fs::remove_dir_all(&path).await?;
        }
    }
    Ok(())
}

fn transient_backtest_record_dir(transient_store_dir: &Path, backtest_id: &str) -> PathBuf {
    transient_store_dir.join(format!(
        "{TRANSIENT_BACKTEST_DIR_PREFIX}{}",
        sanitize_path_segment(backtest_id)
    ))
}

fn unique_transient_backtest_work_dir(transient_store_dir: &Path, backtest_id: &str) -> PathBuf {
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    transient_store_dir.join(format!(
        "{TRANSIENT_SAVING_DIR_PREFIX}{}-{}-{}",
        sanitize_path_segment(backtest_id),
        std::process::id(),
        now_nanos
    ))
}

fn is_transient_backtest_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.starts_with(TRANSIENT_BACKTEST_DIR_PREFIX)
                || name.starts_with(TRANSIENT_SAVING_DIR_PREFIX)
        })
}

async fn is_transient_backtest_expired(path: &Path, now_ms: u64) -> std::io::Result<bool> {
    let modified_ms = path_modified_epoch_ms(path).await?;
    Ok(now_ms.saturating_sub(modified_ms) > TRANSIENT_BACKTEST_TTL_MS)
}

async fn ensure_has_room_for_transient_backtest(transient_store_dir: &Path) -> std::io::Result<()> {
    let (count, total_bytes) = transient_backtest_quota_state(transient_store_dir).await?;
    if count >= TRANSIENT_BACKTEST_MAX_COUNT || total_bytes > TRANSIENT_BACKTEST_MAX_BYTES {
        return Err(std::io::Error::other(format!(
            "transient backtest quota exceeded before spill: count {count}/{TRANSIENT_BACKTEST_MAX_COUNT}, bytes {total_bytes}/{TRANSIENT_BACKTEST_MAX_BYTES}"
        )));
    }
    Ok(())
}

async fn enforce_transient_backtest_quota(transient_store_dir: &Path) -> std::io::Result<()> {
    let (count, total_bytes) = transient_backtest_quota_state(transient_store_dir).await?;
    if count > TRANSIENT_BACKTEST_MAX_COUNT || total_bytes > TRANSIENT_BACKTEST_MAX_BYTES {
        return Err(std::io::Error::other(format!(
            "transient backtest quota exceeded after spill: count {count}/{TRANSIENT_BACKTEST_MAX_COUNT}, bytes {total_bytes}/{TRANSIENT_BACKTEST_MAX_BYTES}"
        )));
    }
    Ok(())
}

async fn transient_backtest_quota_state(
    transient_store_dir: &Path,
) -> std::io::Result<(usize, u64)> {
    let mut count = 0usize;
    let mut total_bytes = 0u64;

    if !fs::try_exists(transient_store_dir).await? {
        return Ok((count, total_bytes));
    }

    let mut entries = fs::read_dir(transient_store_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if !path.is_dir() || !is_transient_backtest_dir(&path) {
            continue;
        }
        count += 1;
        total_bytes = total_bytes.saturating_add(directory_size_bytes(&path).await?);
    }

    Ok((count, total_bytes))
}

async fn enforce_single_transient_backtest_quota(path: &Path) -> std::io::Result<()> {
    let bytes = directory_size_bytes(path).await?;
    if bytes > TRANSIENT_BACKTEST_MAX_SINGLE_BYTES {
        return Err(std::io::Error::other(format!(
            "transient backtest exceeds single-artifact quota: bytes {bytes}/{TRANSIENT_BACKTEST_MAX_SINGLE_BYTES}"
        )));
    }
    Ok(())
}

async fn cleanup_expired_backtest_promotion_work_dirs(
    backtest_store_dir: &Path,
    now_ms: u64,
) -> std::io::Result<()> {
    if !fs::try_exists(backtest_store_dir).await? {
        return Ok(());
    }

    let mut entries = fs::read_dir(backtest_store_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if !path.is_dir() || !is_backtest_promotion_work_dir(&path) {
            continue;
        }
        if is_promotion_work_dir_expired(&path, now_ms).await? {
            fs::remove_dir_all(&path).await?;
        }
    }
    Ok(())
}

pub fn is_backtest_promotion_work_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.starts_with(SAVING_DIR_PREFIX) || name.starts_with(REPLACING_DIR_PREFIX)
        })
}

pub async fn load_backtest_record_from_directory(dir: &Path) -> std::io::Result<BacktestRecord> {
    let manifest: ReproducibilityManifest = read_json(dir.join(MANIFEST_FILE)).await?;
    let event_log: EventLogArtifact = read_json(dir.join(EVENT_LOG_FILE)).await?;
    let backtest: BacktestOutput = read_json(dir.join(BACKTEST_OUTPUT_FILE)).await?;
    let trade_ledger: TradeLedgerArtifact = read_json(dir.join(TRADE_LEDGER_FILE)).await?;
    let equity_curve: EquityCurveArtifact = read_json(dir.join(EQUITY_CURVE_FILE)).await?;
    let metrics: MetricsArtifact = read_json(dir.join(METRICS_FILE)).await?;
    let governance = manifest.governance.clone();

    Ok(BacktestRecord {
        backtest_id: manifest.backtest_id.clone(),
        graph_id: manifest.graph_id.clone(),
        compile_id: manifest.compile_id.clone(),
        created_at_ms: manifest.created_at_ms,
        protocol_name: manifest.protocol_name.clone(),
        config_hash: manifest.config_hash.clone(),
        account: manifest.account.clone(),
        events: event_log.events.clone(),
        backtest,
        backtest_spec: manifest.backtest_spec.clone(),
        artifacts: manifest.compile_artifacts.clone(),
        backtest_artifacts: Some(BacktestArtifactViews {
            event_log,
            trade_ledger,
            equity_curve,
            metrics,
            manifest,
        }),
        governance,
        actor: None,
    })
}

async fn write_backtest_artifact_bundle(
    dir: &Path,
    record: &BacktestRecord,
    views: &BacktestArtifactViews,
) -> std::io::Result<()> {
    write_json(dir.join(EVENT_LOG_FILE), &views.event_log).await?;
    write_json(dir.join(TRADE_LEDGER_FILE), &views.trade_ledger).await?;
    write_json(dir.join(EQUITY_CURVE_FILE), &views.equity_curve).await?;
    write_json(dir.join(METRICS_FILE), &views.metrics).await?;
    write_json(dir.join(BACKTEST_OUTPUT_FILE), &record.backtest).await?;
    write_json(dir.join(MANIFEST_FILE), &views.manifest).await
}

async fn validate_backtest_artifact_bundle(
    dir: &Path,
    record: &BacktestRecord,
) -> std::io::Result<()> {
    let loaded = load_backtest_record_from_directory(dir).await?;
    if loaded.backtest_id != record.backtest_id {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "backtest artifact id mismatch: expected `{}`, got `{}`",
                record.backtest_id, loaded.backtest_id
            ),
        ));
    }
    Ok(())
}

async fn promote_backtest_artifact_dir(
    backtest_store_dir: &Path,
    temp_dir: &Path,
    final_dir: &Path,
    backtest_id: &str,
) -> std::io::Result<()> {
    if fs::try_exists(final_dir).await? {
        let backup_dir =
            unique_backtest_promotion_dir(backtest_store_dir, REPLACING_DIR_PREFIX, backtest_id);
        fs::rename(final_dir, &backup_dir).await?;
        match fs::rename(temp_dir, final_dir).await {
            Ok(()) => {
                let _ = fs::remove_dir_all(&backup_dir).await;
                Ok(())
            }
            Err(error) => {
                let _ = fs::rename(&backup_dir, final_dir).await;
                Err(error)
            }
        }
    } else {
        fs::rename(temp_dir, final_dir).await
    }
}

fn unique_backtest_promotion_dir(store_dir: &Path, prefix: &str, backtest_id: &str) -> PathBuf {
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    store_dir.join(format!(
        "{}{}-{}-{}",
        prefix,
        sanitize_path_segment(backtest_id),
        std::process::id(),
        now_nanos
    ))
}

fn sanitize_path_segment(value: &str) -> String {
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

async fn is_promotion_work_dir_expired(path: &Path, now_ms: u64) -> std::io::Result<bool> {
    let modified_ms = path_modified_epoch_ms(path).await?;
    Ok(now_ms.saturating_sub(modified_ms) > PROMOTION_WORK_DIR_TTL_MS)
}

async fn path_modified_epoch_ms(path: &Path) -> std::io::Result<u64> {
    let modified = fs::metadata(path).await?.modified().unwrap_or(UNIX_EPOCH);
    Ok(modified
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default())
}

fn current_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

async fn enforce_backtest_promotion_work_quota(store_dir: &Path) -> std::io::Result<()> {
    let mut count = 0usize;
    let mut total_bytes = 0u64;

    if !fs::try_exists(store_dir).await? {
        return Ok(());
    }

    let mut entries = fs::read_dir(store_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if !path.is_dir() || !is_backtest_promotion_work_dir(&path) {
            continue;
        }
        count += 1;
        total_bytes = total_bytes.saturating_add(directory_size_bytes(&path).await?);
    }

    if count > PROMOTION_WORK_DIR_MAX_COUNT || total_bytes > PROMOTION_WORK_DIR_MAX_BYTES {
        return Err(std::io::Error::other(format!(
            "backtest promotion temp quota exceeded: count {count}/{PROMOTION_WORK_DIR_MAX_COUNT}, bytes {total_bytes}/{PROMOTION_WORK_DIR_MAX_BYTES}"
        )));
    }
    Ok(())
}

async fn enforce_single_promotion_work_dir_quota(path: &Path) -> std::io::Result<()> {
    let bytes = directory_size_bytes(path).await?;
    if bytes > PROMOTION_WORK_DIR_MAX_SINGLE_BYTES {
        return Err(std::io::Error::other(format!(
            "backtest promotion temp artifact exceeds single-artifact quota: bytes {bytes}/{PROMOTION_WORK_DIR_MAX_SINGLE_BYTES}"
        )));
    }
    Ok(())
}

async fn directory_size_bytes(path: &Path) -> std::io::Result<u64> {
    let mut total = 0u64;
    let mut stack = vec![path.to_path_buf()];

    while let Some(current) = stack.pop() {
        let metadata = fs::metadata(&current).await?;
        if metadata.is_file() {
            total = total.saturating_add(metadata.len());
            continue;
        }
        if metadata.is_dir() {
            let mut entries = fs::read_dir(&current).await?;
            while let Some(entry) = entries.next_entry().await? {
                stack.push(entry.path());
            }
        }
    }

    Ok(total)
}

fn build_event_log_artifact(
    backtest_id: &str,
    events: &[FrontendRuntimeEvent],
) -> anyhow::Result<EventLogArtifact> {
    let digest =
        canonical_json_sha256_digest(events).context("failed to hash event log artifact")?;
    Ok(EventLogArtifact {
        schema_version: EVENT_LOG_ARTIFACT_V1_VERSION.to_string(),
        artifact_id: artifact_id("event_log_artifact", &digest),
        backtest_id: backtest_id.to_string(),
        event_count: events.len(),
        digest,
        events: events.to_vec(),
    })
}

fn build_trade_ledger_artifact(
    backtest_id: &str,
    event_log: &EventLogArtifact,
) -> anyhow::Result<TradeLedgerArtifact> {
    let trades = event_log
        .events
        .iter()
        .filter(|event| event.event_type == "ExecutionFilled")
        .map(trade_entry_from_event)
        .collect::<anyhow::Result<Vec<_>>>()?;
    let summary = summarize_trade_ledger_entries(&trades);
    let payload = serde_json::json!({
        "trade_count": trades.len(),
        "summary": summary,
        "trades": trades,
    });
    let digest =
        canonical_json_sha256_digest(&payload).context("failed to hash trade ledger artifact")?;
    Ok(TradeLedgerArtifact {
        schema_version: TRADE_LEDGER_ARTIFACT_V1_VERSION.to_string(),
        artifact_id: artifact_id("trade_ledger_artifact", &digest),
        backtest_id: backtest_id.to_string(),
        trade_count: trades.len(),
        digest,
        summary: Some(summary),
        trades,
    })
}

fn summarize_trade_ledger_entries(trades: &[TradeLedgerEntry]) -> TradeLedgerSummary {
    let buy_fills = trades
        .iter()
        .filter(|trade| trade.side == OrderSide::Buy)
        .collect::<Vec<_>>();
    let sell_fills = trades
        .iter()
        .filter(|trade| trade.side == OrderSide::Sell)
        .collect::<Vec<_>>();
    let total_fees_paid = trades.iter().map(|trade| trade.fee_paid).sum::<f64>();
    let buy_fees_paid = buy_fills.iter().map(|trade| trade.fee_paid).sum::<f64>();
    let sell_fees_paid = sell_fills.iter().map(|trade| trade.fee_paid).sum::<f64>();
    let total_filled_notional = trades
        .iter()
        .map(|trade| trade.filled_qty * trade.filled_price)
        .sum::<f64>();
    let buy_notional = buy_fills
        .iter()
        .map(|trade| trade.filled_qty * trade.filled_price)
        .sum::<f64>();
    let sell_notional = sell_fills
        .iter()
        .map(|trade| trade.filled_qty * trade.filled_price)
        .sum::<f64>();
    let total_qty = trades.iter().map(|trade| trade.filled_qty).sum::<f64>();
    let buy_qty = buy_fills.iter().map(|trade| trade.filled_qty).sum::<f64>();
    let sell_qty = sell_fills.iter().map(|trade| trade.filled_qty).sum::<f64>();

    TradeLedgerSummary {
        trade_count: trades.len(),
        buy_fill_count: buy_fills.len(),
        sell_fill_count: sell_fills.len(),
        total_fees_paid,
        buy_fees_paid,
        sell_fees_paid,
        total_filled_notional,
        buy_filled_notional: buy_notional,
        sell_filled_notional: sell_notional,
        average_fill_price: average_or_zero(total_filled_notional, total_qty),
        average_buy_fill_price: average_or_option(buy_notional, buy_qty),
        average_sell_fill_price: average_or_option(sell_notional, sell_qty),
        average_fee_per_fill: average_or_zero(total_fees_paid, trades.len() as f64),
        average_buy_fee: average_or_option(buy_fees_paid, buy_fills.len() as f64),
        average_sell_fee: average_or_option(sell_fees_paid, sell_fills.len() as f64),
    }
}

fn average_or_zero(total: f64, qty: f64) -> f64 {
    average_or_option(total, qty).unwrap_or(0.0)
}

fn average_or_option(total: f64, qty: f64) -> Option<f64> {
    if qty.abs() > f64::EPSILON {
        Some(total / qty)
    } else {
        None
    }
}

fn trade_entry_from_event(event: &FrontendRuntimeEvent) -> anyhow::Result<TradeLedgerEntry> {
    let projection = projection_context(event)?;
    Ok(TradeLedgerEntry {
        fill_id: payload_string(&event.payload, "fill_id")?,
        plan_id: payload_string(&event.payload, "plan_id")?,
        exchange: payload_string(&event.payload, "exchange")?,
        symbol: payload_string(&event.payload, "symbol")?,
        side: payload_enum(&event.payload, "side")?,
        filled_qty: payload_number_with_fallback(&event.payload, &["filled_qty", "qty"])?,
        filled_price: payload_number_with_fallback(&event.payload, &["filled_price", "price"])?,
        fee_paid: payload_number(&event.payload, "fee_paid")?,
        filled_at_ms: payload_u64_with_fallback(&event.payload, &["filled_at_ms"])
            .unwrap_or(event.event_time_ms),
        status: payload_enum_with_fallback(&event.payload, &["status", "exec_status"])?,
        trace_id: payload_string(&event.payload, "trace_id")?,
        session_index: projection.session_index,
        cycle_name: projection.cycle_name,
    })
}

fn build_equity_curve_artifact(
    backtest_id: &str,
    projected_portfolios: &[ProjectedPortfolioSnapshot],
) -> anyhow::Result<EquityCurveArtifact> {
    let points = projected_portfolios
        .iter()
        .map(|snapshot| BacktestEquityPoint {
            ts_ms: snapshot.session_started_at_ms,
            equity: snapshot.equity_estimate,
            cash_balance: snapshot.cash_balance,
            net_notional: snapshot.total_net_notional,
        })
        .collect::<Vec<_>>();
    let digest =
        canonical_json_sha256_digest(&points).context("failed to hash equity curve artifact")?;
    Ok(EquityCurveArtifact {
        schema_version: EQUITY_CURVE_ARTIFACT_V1_VERSION.to_string(),
        artifact_id: artifact_id("equity_curve_artifact", &digest),
        backtest_id: backtest_id.to_string(),
        point_count: points.len(),
        digest,
        points,
    })
}

fn build_metrics_artifact(
    backtest_id: &str,
    event_log: &EventLogArtifact,
    trade_ledger: &TradeLedgerArtifact,
    equity_curve: &EquityCurveArtifact,
    projected_portfolios: &[ProjectedPortfolioSnapshot],
    execution_assumptions: Option<&ExecutionAssumptionSpec>,
    execution_assumption_sources: Option<&ExecutionAssumptionSourceSummary>,
) -> anyhow::Result<MetricsArtifact> {
    let trade_ledger_summary = trade_ledger
        .summary
        .clone()
        .unwrap_or_else(|| summarize_trade_ledger_entries(&trade_ledger.trades));
    let summary = summarize_equity_curve(&equity_curve.points, &trade_ledger_summary);
    let final_account = projected_portfolios
        .last()
        .map(ProjectedPortfolioSnapshot::account_summary)
        .unwrap_or_else(empty_account_summary);
    let execution_assumptions = execution_assumptions.map(|value| {
        let mut summary = ExecutionAssumptionsSummary::from(value);
        summary.sources = execution_assumption_sources.cloned();
        ExecutionAssumptionsModule::from_summary(summary)
    });
    let started_at_ms = equity_curve
        .points
        .first()
        .map(|point| point.ts_ms)
        .unwrap_or_default();
    let ended_at_ms = equity_curve
        .points
        .last()
        .map(|point| point.ts_ms)
        .unwrap_or(started_at_ms);
    let payload = serde_json::json!({
        "summary": summary,
        "event_count": event_log.event_count,
        "session_count": equity_curve.point_count,
        "started_at_ms": started_at_ms,
        "ended_at_ms": ended_at_ms,
        "final_account": final_account,
        "execution_assumptions": execution_assumptions,
    });
    let digest =
        canonical_json_sha256_digest(&payload).context("failed to hash metrics artifact")?;
    Ok(MetricsArtifact {
        schema_version: METRICS_ARTIFACT_V1_VERSION.to_string(),
        artifact_id: artifact_id("metrics_artifact", &digest),
        backtest_id: backtest_id.to_string(),
        digest,
        summary,
        event_count: event_log.event_count,
        session_count: equity_curve.point_count,
        started_at_ms,
        ended_at_ms,
        final_account,
        execution_assumptions,
    })
}

fn build_reproducibility_manifest(
    record: &BacktestRecord,
    event_log: &EventLogArtifact,
    trade_ledger: &TradeLedgerArtifact,
    equity_curve: &EquityCurveArtifact,
    metrics: &MetricsArtifact,
    backtest_output_digest: ArtifactDigest,
) -> ReproducibilityManifest {
    ReproducibilityManifest {
        schema_version: REPRODUCIBILITY_MANIFEST_V1_VERSION.to_string(),
        manifest_id: format!("manifest_{}", record.backtest_id),
        backtest_id: record.backtest_id.clone(),
        graph_id: record.graph_id.clone(),
        compile_id: record.compile_id.clone(),
        created_at_ms: record.created_at_ms,
        protocol_name: record.protocol_name.clone(),
        config_hash: record.config_hash.clone(),
        account: metrics.final_account.clone(),
        summary: metrics.summary.clone(),
        backtest_spec: record.backtest_spec.clone(),
        compile_artifacts: record.artifacts.clone(),
        governance: record.governance.clone(),
        output_artifacts: vec![
            artifact_ref(
                "event_log",
                event_log.artifact_id.clone(),
                event_log.digest.clone(),
                EVENT_LOG_FILE,
            ),
            artifact_ref(
                "trade_ledger",
                trade_ledger.artifact_id.clone(),
                trade_ledger.digest.clone(),
                TRADE_LEDGER_FILE,
            ),
            artifact_ref(
                "equity_curve",
                equity_curve.artifact_id.clone(),
                equity_curve.digest.clone(),
                EQUITY_CURVE_FILE,
            ),
            artifact_ref(
                "metrics",
                metrics.artifact_id.clone(),
                metrics.digest.clone(),
                METRICS_FILE,
            ),
        ],
        backtest_output_digest,
    }
}

#[derive(Clone)]
struct EventProjectionContext {
    session_index: usize,
    cycle_name: String,
    session_started_at_ms: u64,
}

#[derive(Clone)]
struct ProjectedPortfolioSnapshot {
    session_index: usize,
    session_started_at_ms: u64,
    equity_estimate: f64,
    cash_balance: f64,
    available_cash_balance: f64,
    frozen_cash_balance: f64,
    total_gross_notional: f64,
    total_net_notional: f64,
    total_leverage: f64,
    positions: usize,
    open_orders: Vec<super::OpenOrderSummary>,
}

impl ProjectedPortfolioSnapshot {
    fn account_summary(&self) -> AccountSummary {
        AccountSummary {
            equity_estimate: self.equity_estimate,
            cash_balance: self.cash_balance,
            available_cash_balance: self.available_cash_balance,
            frozen_cash_balance: self.frozen_cash_balance,
            total_leverage: self.total_leverage,
            total_gross_notional: self.total_gross_notional,
            total_net_notional: self.total_net_notional,
            positions: self.positions,
            open_order_count: self.open_orders.len(),
            open_orders: self.open_orders.clone(),
        }
    }
}

fn project_session_portfolios(
    event_log: &EventLogArtifact,
) -> anyhow::Result<Vec<ProjectedPortfolioSnapshot>> {
    let mut snapshots = BTreeMap::new();
    for event in event_log
        .events
        .iter()
        .filter(|event| event.event_type == "PortfolioUpdated")
    {
        let snapshot = portfolio_snapshot_from_event(event)?;
        snapshots.insert(snapshot.session_index, snapshot);
    }
    Ok(snapshots.into_values().collect())
}

fn portfolio_snapshot_from_event(
    event: &FrontendRuntimeEvent,
) -> anyhow::Result<ProjectedPortfolioSnapshot> {
    let projection = projection_context(event)?;
    let open_orders = payload_array(&event.payload, "open_orders")
        .map(|orders| {
            orders
                .iter()
                .map(open_order_summary_from_value)
                .collect::<anyhow::Result<Vec<_>>>()
        })
        .transpose()?
        .unwrap_or_default();

    Ok(ProjectedPortfolioSnapshot {
        session_index: projection.session_index,
        session_started_at_ms: projection.session_started_at_ms,
        equity_estimate: payload_number(&event.payload, "equity_estimate")?,
        cash_balance: payload_number(&event.payload, "cash_balance")?,
        available_cash_balance: payload_number(&event.payload, "available_cash_balance")?,
        frozen_cash_balance: payload_number(&event.payload, "frozen_cash_balance")?,
        total_gross_notional: payload_number(&event.payload, "total_gross_notional")?,
        total_net_notional: payload_number(&event.payload, "total_net_notional")?,
        total_leverage: payload_number(&event.payload, "total_leverage")?,
        positions: payload_usize(&event.payload, "positions")?,
        open_orders,
    })
}

fn summarize_equity_curve(
    points: &[BacktestEquityPoint],
    trade_ledger: &TradeLedgerSummary,
) -> BacktestSummary {
    let step_count = points.len();
    let initial_equity = points.first().map(|point| point.equity).unwrap_or_default();
    let final_equity = points
        .last()
        .map(|point| point.equity)
        .unwrap_or(initial_equity);
    let net_profit = final_equity - initial_equity;
    let total_return_ratio = if initial_equity.abs() > f64::EPSILON {
        net_profit / initial_equity
    } else {
        0.0
    };
    let mut peak_equity = initial_equity;
    let mut max_drawdown_ratio = 0.0_f64;
    for point in points {
        peak_equity = peak_equity.max(point.equity);
        if peak_equity.abs() > f64::EPSILON {
            max_drawdown_ratio = max_drawdown_ratio.max((peak_equity - point.equity) / peak_equity);
        }
    }

    let turnover_ratio = if initial_equity.abs() > f64::EPSILON {
        trade_ledger.total_filled_notional / initial_equity.abs()
    } else {
        0.0
    };
    let average_trade_notional = if trade_ledger.trade_count > 0 {
        trade_ledger.total_filled_notional / trade_ledger.trade_count as f64
    } else {
        0.0
    };
    let fee_drag_ratio = if initial_equity.abs() > f64::EPSILON {
        trade_ledger.total_fees_paid / initial_equity.abs()
    } else {
        0.0
    };

    BacktestSummary {
        step_count,
        trade_count: trade_ledger.trade_count,
        total_return_ratio,
        max_drawdown_ratio,
        final_equity,
        net_profit,
        turnover_ratio,
        average_trade_notional,
        fee_drag_ratio,
    }
}

fn projection_context(event: &FrontendRuntimeEvent) -> anyhow::Result<EventProjectionContext> {
    let projection = event
        .payload
        .get("artifact_projection")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow!("missing artifact projection context on {}", event.event_id))?;
    Ok(EventProjectionContext {
        session_index: projection
            .get("session_index")
            .and_then(Value::as_u64)
            .ok_or_else(|| anyhow!("missing session_index on {}", event.event_id))?
            as usize,
        cycle_name: projection
            .get("cycle_name")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("missing cycle_name on {}", event.event_id))?
            .to_string(),
        session_started_at_ms: projection
            .get("session_started_at_ms")
            .and_then(Value::as_u64)
            .ok_or_else(|| anyhow!("missing session_started_at_ms on {}", event.event_id))?,
    })
}

fn open_order_summary_from_value(value: &Value) -> anyhow::Result<super::OpenOrderSummary> {
    Ok(super::OpenOrderSummary {
        order_id: value
            .get("order_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("missing open order id"))?
            .to_string(),
        side: value
            .get("side")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("missing open order side"))?
            .to_string(),
        remaining_qty: value
            .get("remaining_qty")
            .and_then(Value::as_f64)
            .ok_or_else(|| anyhow!("missing open order remaining_qty"))?,
        limit_price: value.get("limit_price").and_then(Value::as_f64),
        reserved_cash: value
            .get("reserved_cash")
            .and_then(Value::as_f64)
            .ok_or_else(|| anyhow!("missing open order reserved_cash"))?,
        reserved_qty: value
            .get("reserved_qty")
            .and_then(Value::as_f64)
            .ok_or_else(|| anyhow!("missing open order reserved_qty"))?,
    })
}

fn payload_string(payload: &Value, key: &str) -> anyhow::Result<String> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| anyhow!("missing string field {key}"))
}

fn payload_number(payload: &Value, key: &str) -> anyhow::Result<f64> {
    payload
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| anyhow!("missing numeric field {key}"))
}

fn payload_number_with_fallback(payload: &Value, keys: &[&str]) -> anyhow::Result<f64> {
    keys.iter()
        .find_map(|key| payload.get(*key).and_then(Value::as_f64))
        .ok_or_else(|| anyhow!("missing numeric fields {}", keys.join(", ")))
}

fn payload_u64_with_fallback(payload: &Value, keys: &[&str]) -> Option<u64> {
    keys.iter()
        .find_map(|key| payload.get(*key).and_then(Value::as_u64))
}

fn payload_usize(payload: &Value, key: &str) -> anyhow::Result<usize> {
    payload
        .get(key)
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        .ok_or_else(|| anyhow!("missing usize field {key}"))
}

fn payload_array<'a>(payload: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    payload.get(key).and_then(Value::as_array)
}

fn payload_enum<T>(payload: &Value, key: &str) -> anyhow::Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let value = payload
        .get(key)
        .cloned()
        .ok_or_else(|| anyhow!("missing enum field {key}"))?;
    serde_json::from_value(value).map_err(|error| anyhow!("failed to parse {key}: {error}"))
}

fn payload_enum_with_fallback<T>(payload: &Value, keys: &[&str]) -> anyhow::Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    for key in keys {
        if let Some(value) = payload.get(*key).cloned() {
            return serde_json::from_value(value)
                .map_err(|error| anyhow!("failed to parse {key}: {error}"));
        }
    }
    Err(anyhow!("missing enum fields {}", keys.join(", ")))
}

fn empty_account_summary() -> AccountSummary {
    AccountSummary {
        equity_estimate: 0.0,
        cash_balance: 0.0,
        available_cash_balance: 0.0,
        frozen_cash_balance: 0.0,
        total_leverage: 0.0,
        total_gross_notional: 0.0,
        total_net_notional: 0.0,
        positions: 0,
        open_order_count: 0,
        open_orders: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::items_after_test_module)]

    use super::*;
    use qrpc_core::{
        ArtifactDigest, ArtifactDigestAlgorithm, BacktestReplaySource,
        ExecutionAssumptionSourceSummary, ExecutionAssumptionSpec, ExecutionAssumptionValueSource,
        PortfolioState, RunModeSpec, RunSpec, Symbol, TimeInForce,
    };
    use serde_json::json;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_PROMOTION_TEST_ID: AtomicU64 = AtomicU64::new(0);

    fn promotion_test_dir(label: &str) -> PathBuf {
        let sequence = NEXT_PROMOTION_TEST_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "quantpilot-promotion-{}-{}-{}",
            label,
            std::process::id(),
            sequence
        ))
    }

    #[tokio::test]
    async fn promotion_work_cleanup_removes_expired_work_dirs_only() {
        let dir = promotion_test_dir("ttl");
        let saved_dir = dir.join("backtest_saved");
        let stale_work_dir = dir.join(format!("{SAVING_DIR_PREFIX}stale"));
        fs::create_dir_all(&saved_dir).await.unwrap();
        fs::create_dir_all(&stale_work_dir).await.unwrap();

        cleanup_expired_backtest_promotion_work_dirs(&dir, u64::MAX)
            .await
            .unwrap();

        assert!(saved_dir.exists());
        assert!(!stale_work_dir.exists());
        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn promotion_work_quota_rejects_excess_work_directories() {
        let dir = promotion_test_dir("quota");
        fs::create_dir_all(&dir).await.unwrap();

        for index in 0..=PROMOTION_WORK_DIR_MAX_COUNT {
            fs::create_dir_all(dir.join(format!("{SAVING_DIR_PREFIX}quota-{index}")))
                .await
                .unwrap();
        }

        let result = enforce_backtest_promotion_work_quota(&dir).await;

        assert!(result.is_err());
        let _ = fs::remove_dir_all(&dir).await;
    }

    #[test]
    fn artifact_views_project_from_event_log_instead_of_backtest_output() {
        let record = BacktestRecord {
            backtest_id: "backtest_projection_test".to_string(),
            graph_id: "graph_projection_test".to_string(),
            compile_id: "compile_projection_test".to_string(),
            created_at_ms: 1_700_000_000_000,
            protocol_name: "quantpilot/runtime-config/v1".to_string(),
            config_hash: "projection_config_hash".to_string(),
            account: AccountSummary {
                cash_balance: 1.0,
                equity_estimate: 1.0,
                ..empty_account_summary()
            },
            events: vec![
                fill_event(
                    "fill-1",
                    "plan-1",
                    0,
                    "slow",
                    1_700_000_000_000,
                    0.5,
                    20_000.0,
                    2.0,
                ),
                portfolio_event(
                    "portfolio-fast-0",
                    0,
                    "fast",
                    1_700_000_000_000,
                    10_000.0,
                    200.0,
                ),
                fill_event(
                    "fill-2",
                    "plan-2",
                    1,
                    "fast",
                    1_700_000_060_000,
                    0.25,
                    20_200.0,
                    1.0,
                ),
                portfolio_event(
                    "portfolio-fast-1",
                    1,
                    "fast",
                    1_700_000_060_000,
                    12_050.0,
                    0.0,
                ),
            ],
            backtest: BacktestOutput {
                mode: "historical_replay".to_string(),
                started_at_ms: 99,
                ended_at_ms: 100,
                sessions: Vec::new(),
                equity_curve: vec![BacktestEquityPoint {
                    ts_ms: 99,
                    equity: 1.0,
                    cash_balance: 1.0,
                    net_notional: 0.0,
                }],
                summary: BacktestSummary {
                    step_count: 999,
                    trade_count: 999,
                    total_return_ratio: -0.99,
                    max_drawdown_ratio: 0.99,
                    final_equity: 1.0,
                    net_profit: -99.0,
                    turnover_ratio: 9.99,
                    average_trade_notional: 1.0,
                    fee_drag_ratio: 0.5,
                },
                final_portfolio: PortfolioState::new(1.0, 100),
            },
            backtest_spec: Some(BacktestSpec::new(
                "backtest_projection_test",
                BacktestReplaySource::HistoricalReplay,
                1_700_000_000_000,
                RunSpec {
                    schema_version: "quantpilot/run-spec/v1".to_string(),
                    run_mode: RunModeSpec::Backtest,
                    graph_id: "graph_projection_test".to_string(),
                    compile_id: "compile_projection_test".to_string(),
                    runtime_mode: "backtest".to_string(),
                    protocol_name: "quantpilot/runtime-config/v1".to_string(),
                    config_hash: "projection_config_hash".to_string(),
                    datasets: Vec::new(),
                    execution_assumptions: ExecutionAssumptionSpec {
                        initial_cash_balance: 10_000.0,
                        taker_fee_bps: 3.5,
                        default_slippage_bps: 1.25,
                        total_cost_buffer_bps: 0.0,
                        time_in_force: TimeInForce::Gtc,
                        allow_partial_fills: true,
                        latency_assumption_ms: Some(250),
                    },
                    execution_assumption_sources: Some(ExecutionAssumptionSourceSummary {
                        fee_bps: ExecutionAssumptionValueSource::RequestOverride,
                        slippage_bps: ExecutionAssumptionValueSource::ProfileDefault,
                        latency_ms: ExecutionAssumptionValueSource::BackendFallback,
                    }),
                    core_ir_digest: ArtifactDigest {
                        algorithm: ArtifactDigestAlgorithm::Sha256CanonicalJson,
                        value: "projection_core_ir_digest".to_string(),
                    },
                },
                qrpc_core::MarketDataSnapshotSpec {
                    snapshot_id: "snapshot_projection_test".to_string(),
                    replay_source: BacktestReplaySource::HistoricalReplay,
                    captured_at_ms: 1_700_000_000_000,
                    datasets: Vec::new(),
                },
            )),
            artifacts: None,
            backtest_artifacts: None,
            governance: RuntimeGovernanceSnapshot::default(),
            actor: None,
        };

        let views = build_backtest_artifact_views(&record).expect("artifact views should build");
        let manifest_spec = views
            .manifest
            .backtest_spec
            .as_ref()
            .expect("manifest should carry backtest spec");

        assert_eq!(views.trade_ledger.trade_count, 2);
        assert_eq!(views.trade_ledger.trades[0].fill_id, "fill-1");
        assert_eq!(views.trade_ledger.trades[1].session_index, 1);
        assert_eq!(views.trade_ledger.trades[1].cycle_name, "fast");

        assert_eq!(views.equity_curve.point_count, 2);
        assert_eq!(views.equity_curve.points[0].ts_ms, 1_700_000_000_000);
        assert_eq!(views.equity_curve.points[0].equity, 10_200.0);
        assert_eq!(views.equity_curve.points[1].ts_ms, 1_700_000_060_000);
        assert_eq!(views.equity_curve.points[1].equity, 12_050.0);

        assert_eq!(views.metrics.summary.step_count, 2);
        assert_eq!(views.metrics.summary.trade_count, 2);
        assert_eq!(views.metrics.summary.final_equity, 12_050.0);
        assert_eq!(views.metrics.summary.net_profit, 1_850.0);
        assert!(views.metrics.summary.total_return_ratio > 0.18);
        assert!(views.metrics.summary.turnover_ratio > 1.0);
        assert!(views.metrics.summary.average_trade_notional > 5_000.0);
        assert!(views.metrics.summary.fee_drag_ratio > 0.0);
        assert_eq!(views.metrics.started_at_ms, 1_700_000_000_000);
        assert_eq!(views.metrics.ended_at_ms, 1_700_000_060_000);
        assert_eq!(views.metrics.final_account.cash_balance, 12_050.0);
        assert_eq!(views.metrics.final_account.positions, 0);
        assert_eq!(
            serde_json::to_value(&views.metrics.execution_assumptions).unwrap(),
            json!({
                "summary": {
                    "fee_bps": 3.5,
                    "slippage_bps": 1.25,
                    "latency_ms": 250,
                    "sources": {
                        "fee_bps": "request_override",
                        "slippage_bps": "profile_default",
                        "latency_ms": "backend_fallback",
                    }
                },
                "list_tag": {
                    "label": "fee=3.5 slip=1.25 lat=250",
                    "sources_label": "fee:req slip:profile lat:backend"
                }
            })
        );

        assert_eq!(views.manifest.summary.final_equity, 12_050.0);
        assert_eq!(views.manifest.account.cash_balance, 12_050.0);
        assert_eq!(
            json!({
                "taker_fee_bps": manifest_spec.run_spec.execution_assumptions.taker_fee_bps,
                "default_slippage_bps": manifest_spec.run_spec.execution_assumptions.default_slippage_bps,
                "latency_assumption_ms": manifest_spec.run_spec.execution_assumptions.latency_assumption_ms,
            }),
            json!({
                "taker_fee_bps": 3.5,
                "default_slippage_bps": 1.25,
                "latency_assumption_ms": 250
            })
        );
        assert_eq!(
            views.metrics.execution_assumptions,
            Some(ExecutionAssumptionsModule {
                summary: ExecutionAssumptionsSummary {
                    fee_bps: manifest_spec.run_spec.execution_assumptions.taker_fee_bps,
                    slippage_bps: manifest_spec
                        .run_spec
                        .execution_assumptions
                        .default_slippage_bps,
                    latency_ms: manifest_spec
                        .run_spec
                        .execution_assumptions
                        .latency_assumption_ms
                        .unwrap(),
                    sources: manifest_spec.run_spec.execution_assumption_sources.clone(),
                },
                list_tag: ExecutionAssumptionsTag {
                    label: "fee=3.5 slip=1.25 lat=250".to_string(),
                    sources_label: "fee:req slip:profile lat:backend".to_string(),
                },
            })
        );
        assert_ne!(
            views.metrics.summary.final_equity,
            record.backtest.summary.final_equity
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn fill_event(
        fill_id: &str,
        plan_id: &str,
        session_index: usize,
        cycle_name: &str,
        session_started_at_ms: u64,
        qty: f64,
        price: f64,
        fee_paid: f64,
    ) -> FrontendRuntimeEvent {
        FrontendRuntimeEvent {
            event_id: format!("evt-{fill_id}"),
            event_type: "ExecutionFilled".to_string(),
            source_id: plan_id.to_string(),
            node_id: "execution".to_string(),
            event_time_ms: session_started_at_ms,
            severity: "Info".to_string(),
            summary: "fill".to_string(),
            payload: json!({
                "fill_id": fill_id,
                "plan_id": plan_id,
                "exchange": "Binance",
                "symbol": format!("{:?}", Symbol::BtcUsdt),
                "side": "Buy",
                "qty": qty,
                "price": price,
                "fee_paid": fee_paid,
                "exec_status": "Filled",
                "filled_at_ms": session_started_at_ms,
                "trace_id": format!("trace-{cycle_name}-{session_started_at_ms}"),
                "artifact_projection": {
                    "session_index": session_index,
                    "cycle_name": cycle_name,
                    "session_started_at_ms": session_started_at_ms,
                },
            }),
            envelope: Default::default(),
        }
    }

    fn portfolio_event(
        event_id: &str,
        session_index: usize,
        cycle_name: &str,
        session_started_at_ms: u64,
        cash_balance: f64,
        total_net_notional: f64,
    ) -> FrontendRuntimeEvent {
        FrontendRuntimeEvent {
            event_id: event_id.to_string(),
            event_type: "PortfolioUpdated".to_string(),
            source_id: "portfolio".to_string(),
            node_id: "runtime".to_string(),
            event_time_ms: session_started_at_ms,
            severity: "Info".to_string(),
            summary: "portfolio".to_string(),
            payload: json!({
                "cash_balance": cash_balance,
                "available_cash_balance": cash_balance,
                "frozen_cash_balance": 0.0,
                "total_gross_notional": total_net_notional.abs(),
                "total_net_notional": total_net_notional,
                "total_leverage": 0.0,
                "equity_estimate": cash_balance + total_net_notional,
                "positions": usize::from(total_net_notional.abs() > f64::EPSILON),
                "open_order_count": 0,
                "open_orders": [],
                "trace_id": format!("trace-{cycle_name}-{session_started_at_ms}"),
                "artifact_projection": {
                    "session_index": session_index,
                    "cycle_name": cycle_name,
                    "session_started_at_ms": session_started_at_ms,
                },
            }),
            envelope: Default::default(),
        }
    }
}

fn artifact_ref(
    kind: &str,
    artifact_id: String,
    digest: ArtifactDigest,
    file_name: &str,
) -> ArtifactFileRef {
    ArtifactFileRef {
        kind: kind.to_string(),
        artifact_id,
        digest,
        file_name: file_name.to_string(),
    }
}

fn artifact_id(prefix: &str, digest: &ArtifactDigest) -> String {
    let short = &digest.value[..digest.value.len().min(12)];
    format!("{prefix}_{short}")
}

async fn write_json<T: Serialize>(path: PathBuf, value: &T) -> std::io::Result<()> {
    let body = serde_json::to_string_pretty(value).map_err(to_io_error)?;
    fs::write(path, body).await
}

async fn read_json<T: for<'de> Deserialize<'de>>(path: PathBuf) -> std::io::Result<T> {
    let content = fs::read_to_string(path).await?;
    serde_json::from_str(&content).map_err(to_io_error)
}

fn to_io_error(error: impl std::fmt::Display) -> std::io::Error {
    std::io::Error::other(error.to_string())
}
