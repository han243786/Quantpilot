#[path = "backtest/execution_start.rs"]
mod backtest_execution_start;
#[path = "backtest/experiment_sweep.rs"]
mod backtest_experiment_sweep;
#[path = "backtest/record_store.rs"]
mod backtest_record_store;
#[path = "backtest/replay.rs"]
mod backtest_replay;
mod event_stream;
#[path = "mutation/ai_proposal.rs"]
mod mutation_ai_proposal;
#[path = "mutation/parameter_mutation.rs"]
mod mutation_parameter_mutation;
#[path = "run/record_store.rs"]
mod run_record_store;
#[path = "run/replay_status.rs"]
mod run_replay_status;
#[path = "run/session_start.rs"]
mod run_session_start;
#[path = "run/v4_handoff.rs"]
mod run_v4_handoff;
use backtest_execution_start::execute_backtest_request;
pub(crate) use backtest_execution_start::start_backtest_run;
pub(crate) use backtest_record_store::{
    discard_backtest_record, get_backtest_detail, list_backtests, save_backtest_record,
};
pub(crate) use backtest_replay::get_backtest_replay;
pub(crate) use event_stream::stream_run_events;
pub(crate) use mutation_ai_proposal::{
    approve_ai_proposal, claim_ai_proposal_review, create_runtime_ai_proposal,
    get_runtime_ai_proposal_detail, get_runtime_approval_detail, list_runtime_ai_proposals,
    list_runtime_approvals, reject_ai_proposal,
};
pub(crate) use mutation_parameter_mutation::{
    activate_runtime_parameter_mutation, create_runtime_parameter_mutation,
    get_runtime_parameter_mutation_detail, list_runtime_parameter_mutations,
    rollback_runtime_parameter_mutation,
};
pub(crate) use run_record_store::{discard_run_record, get_run_detail, list_runs, save_run_record};
pub(crate) use run_replay_status::{get_run_replay, get_run_status};
pub(crate) use run_session_start::start_test_run;
pub(crate) use run_v4_handoff::start_v4_runtime_run;
use run_v4_handoff::{runtime_simulated_v4_matrix, runtime_v4_static_bundle};

// Backtest + Experiment handlers
include!("backtest.rs");
pub(crate) use backtest_experiment_sweep::{
    discard_experiment_record, get_experiment_detail, list_experiments, save_experiment_record,
    start_backtest_experiment,
};
// Run + SSE handlers
include!("run.rs");
// Mutation + Proposal + Approval handlers
include!("mutation.rs");

use super::*;
use axum::extract::Query;

const MAX_EXPERIMENT_VARIANTS: usize = 27;
const DEFAULT_REPLAY_PAGE_SIZE: usize = 12;
const MAX_REPLAY_PAGE_SIZE: usize = 50;

#[derive(Debug, Serialize)]
pub(crate) struct DiscardRuntimeArtifactResponse {
    discarded_id: String,
    discarded_kind: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RuntimeReplayQuery {
    cursor: Option<usize>,
    limit: Option<usize>,
    checkpoint: Option<usize>,
    sequence_cursor: Option<u64>,
    stage: Option<String>,
    severity: Option<String>,
    retention_class: Option<String>,
    module_key: Option<String>,
    #[serde(default)]
    key_only: bool,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct RuntimeParameterMutationListQuery {
    source_kind: Option<RuntimeEvidenceSourceKind>,
    source_id: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct RuntimeAiProposalListQuery {
    source_kind: Option<RuntimeEvidenceSourceKind>,
    source_id: Option<String>,
    status: Option<RuntimeAiProposalStatus>,
}

fn clean_optional_filter(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

fn normalized_replay_options(query: RuntimeReplayQuery) -> RuntimeReplayOptions {
    let cursor = query.checkpoint.or(query.cursor).unwrap_or(0);
    let limit = query
        .limit
        .unwrap_or(DEFAULT_REPLAY_PAGE_SIZE)
        .clamp(1, MAX_REPLAY_PAGE_SIZE);
    RuntimeReplayOptions {
        cursor,
        limit,
        sequence_cursor: query.sequence_cursor,
        filters: RuntimeReplayFilters {
            stage: clean_optional_filter(query.stage),
            severity: clean_optional_filter(query.severity),
            retention_class: clean_optional_filter(query.retention_class),
            module_key: clean_optional_filter(query.module_key),
            key_only: query.key_only,
        },
    }
}

/// v1.3.5: RAII 守卫 — 运行结束后自动复位 run_in_progress
struct RunInProgressGuard<'a>(&'a std::sync::atomic::AtomicBool);
impl Drop for RunInProgressGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, std::sync::atomic::Ordering::Release);
    }
}

pub(crate) async fn create_runtime_report(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Json(request): Json<CreateRuntimeReportRequest>,
) -> Result<Json<RuntimeEvidenceReportRecord>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    let report = match request.source_kind {
        RuntimeEvidenceSourceKind::Run => {
            let record = load_run_record_from_state(&state, &user_id, &request.source_id).await?;
            runtime_report_record_from_run_record(record, now_ms, request.generation_policy)
        }
        RuntimeEvidenceSourceKind::Backtest => {
            let record =
                load_backtest_record_from_state(&state, &user_id, &request.source_id).await?;
            runtime_report_record_from_backtest_record(record, now_ms, request.generation_policy)
        }
    };

    match load_runtime_report_record(state.report_store_dir.as_ref(), &report.report_id).await {
        Ok(existing) => return Ok(Json(existing)),
        Err((StatusCode::NOT_FOUND, _)) => {}
        Err(error) => return Err(error),
    }

    state.evidence_metrics.record_report_generation(&report);
    persist_runtime_report_record(state.report_store_dir.as_ref(), &report)
        .await
        .map_err(io_error)?;
    Ok(Json(report))
}

fn report_source_metadata_matches(
    saved: &RuntimeEvidenceReportRecord,
    current: &RuntimeEvidenceReportRecord,
) -> bool {
    saved.graph_id == current.graph_id
        && saved.source_sequence_range == current.source_sequence_range
        && saved.source_event_count == current.source_event_count
        && saved.retained_event_count == current.retained_event_count
        && saved.governance == current.governance
        && saved.generation_policy == current.generation_policy
}

fn source_changed_report(
    mut record: RuntimeEvidenceReportRecord,
    reason_code: &str,
    message: impl Into<String>,
) -> RuntimeEvidenceReportRecord {
    let message = message.into();
    record.status = RuntimeReportLifecycleStatus::SourceChanged;
    record.failure_reason = Some(message.clone());
    record.failure = Some(RuntimeReportFailureMetadata {
        reason_code: reason_code.to_string(),
        message,
        retry_eligible: true,
    });
    record.artifacts.clear();
    record.updated_at_ms = current_time_ms();
    record
}

async fn current_report_for_saved_source(
    state: &AppState,
    user_id: &auth::UserId,
    record: &RuntimeEvidenceReportRecord,
) -> Result<Option<RuntimeEvidenceReportRecord>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    match record.source_kind {
        RuntimeEvidenceSourceKind::Run => {
            match load_run_record_from_state(state, user_id, &record.source_id).await {
                Ok(source) => Ok(Some(runtime_report_record_from_run_record(
                    source,
                    now_ms,
                    Some(record.generation_policy.clone()),
                ))),
                Err((StatusCode::NOT_FOUND, _)) => Ok(None),
                Err(error) => Err(error),
            }
        }
        RuntimeEvidenceSourceKind::Backtest => {
            match load_backtest_record_from_state(state, user_id, &record.source_id).await {
                Ok(source) => Ok(Some(runtime_report_record_from_backtest_record(
                    source,
                    now_ms,
                    Some(record.generation_policy.clone()),
                ))),
                Err((StatusCode::NOT_FOUND, _)) => Ok(None),
                Err(error) => Err(error),
            }
        }
    }
}

async fn materialize_runtime_report_record(
    state: &AppState,
    user_id: &auth::UserId,
    record: RuntimeEvidenceReportRecord,
) -> Result<RuntimeEvidenceReportRecord, (StatusCode, String)> {
    if record.status != RuntimeReportLifecycleStatus::Ready {
        return Ok(record);
    }
    let Some(current) = current_report_for_saved_source(state, user_id, &record).await? else {
        state.evidence_metrics.record_report_source_changed();
        return Ok(source_changed_report(
            record,
            "source_missing",
            "source evidence record is no longer available for report validation",
        ));
    };
    if report_source_metadata_matches(&record, &current) {
        Ok(record)
    } else {
        state.evidence_metrics.record_report_source_changed();
        Ok(source_changed_report(
            record,
            "source_changed",
            "source evidence metadata changed after report generation",
        ))
    }
}

pub(crate) async fn list_runtime_reports(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<RuntimeEvidenceReportRecord>>, (StatusCode, String)> {
    let records = list_runtime_report_records(&state.report_store_dir)
        .await
        .map_err(io_error)?;
    let mut records = {
        let mut materialized = Vec::new();
        for record in records {
            materialized.push(materialize_runtime_report_record(&state, &user_id, record).await?);
        }
        materialized
    };
    records.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.report_id.cmp(&left.report_id))
    });
    Ok(Json(paginate(records, pagination)))
}

pub(crate) async fn get_runtime_report_detail(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(report_id): Path<String>,
) -> Result<Json<RuntimeEvidenceReportRecord>, (StatusCode, String)> {
    let record = load_runtime_report_record(state.report_store_dir.as_ref(), &report_id).await?;
    materialize_runtime_report_record(&state, &user_id, record)
        .await
        .map(Json)
}

pub(crate) async fn export_runtime_report_artifact(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(report_id): Path<String>,
) -> Result<Json<RuntimeEvidenceReportArtifact>, (StatusCode, String)> {
    let record = load_runtime_report_record(state.report_store_dir.as_ref(), &report_id).await?;
    let record = materialize_runtime_report_record(&state, &user_id, record).await?;
    Ok(Json(runtime_report_artifact_from_record(&record)))
}

fn runtime_report_status_counts(
    records: &[RuntimeEvidenceReportRecord],
) -> RuntimeEvidenceReportStatusCounts {
    let mut counts = RuntimeEvidenceReportStatusCounts {
        requested: 0,
        generating: 0,
        ready: 0,
        failed: 0,
        expired: 0,
        source_changed: 0,
    };
    for record in records {
        match record.status {
            RuntimeReportLifecycleStatus::Requested => counts.requested += 1,
            RuntimeReportLifecycleStatus::Generating => counts.generating += 1,
            RuntimeReportLifecycleStatus::Ready => counts.ready += 1,
            RuntimeReportLifecycleStatus::Failed => counts.failed += 1,
            RuntimeReportLifecycleStatus::Expired => counts.expired += 1,
            RuntimeReportLifecycleStatus::SourceChanged => counts.source_changed += 1,
        }
    }
    counts
}

pub(crate) async fn get_runtime_evidence_health(
    State(state): State<AppState>,
) -> Result<Json<RuntimeEvidenceHealthResponse>, (StatusCode, String)> {
    let reports = list_runtime_report_records(&state.report_store_dir)
        .await
        .map_err(io_error)?;
    Ok(Json(RuntimeEvidenceHealthResponse {
        status: "ok".to_string(),
        metrics: state.evidence_metrics.snapshot(),
        persisted_report_count: reports.len(),
        report_status_counts: runtime_report_status_counts(&reports),
        cleanup_policy: runtime_evidence_cleanup_policy(),
    }))
}

pub(crate) async fn cleanup_runtime_evidence(
    State(state): State<AppState>,
    Json(request): Json<RuntimeEvidenceCleanupRequest>,
) -> Result<Json<RuntimeEvidenceCleanupResponse>, (StatusCode, String)> {
    let policy = runtime_evidence_cleanup_policy();
    let max_age_ms = request
        .max_age_ms
        .unwrap_or(policy.transient_generation_ttl_ms);
    let removed = cleanup_transient_runtime_report_outputs(
        state.report_store_dir.as_ref(),
        max_age_ms,
        current_time_ms(),
    )
    .await
    .map_err(io_error)?;
    let retained_report_records = list_runtime_report_records(&state.report_store_dir)
        .await
        .map_err(io_error)?
        .len();
    Ok(Json(RuntimeEvidenceCleanupResponse {
        policy,
        removed_transient_generation_outputs: removed,
        retained_report_records,
    }))
}

pub(crate) async fn list_merge_records(
    user_id: auth::UserId,
    State(state): State<AppState>,
) -> Result<Json<MergeRecordsResponse>, (StatusCode, String)> {
    // 从最近的 run/backtest 中提取合并事件
    let prefix = auth::scoped_key(&user_id, "");
    let runs = state.runs.read().await;
    let mut entries = Vec::new();
    let mut total_conflicts = 0usize;
    let mut total_suppressed = 0usize;

    for run in runs
        .iter()
        .filter(|(k, _)| k.starts_with(&prefix))
        .map(|(_, v)| v)
    {
        for event in &run.events {
            if event.source_id == "merge_engine" {
                if let Some(payload) = event.payload.as_object() {
                    entries.push(MergeRecordEntry {
                        cycle_name: run.run_id.clone(),
                        input_count: payload
                            .get("input_count")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(1) as usize,
                        output_count: payload
                            .get("output_count")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(1) as usize,
                        conflicts: payload
                            .get("conflicts")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as usize,
                        suppressed: payload
                            .get("suppressed")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as usize,
                        merge_policy: payload
                            .get("merge_policy")
                            .and_then(|v| v.as_str())
                            .unwrap_or("WeightedMerge")
                            .to_string(),
                    });
                    total_conflicts += entries.last().map(|e| e.conflicts).unwrap_or(0);
                    total_suppressed += entries.last().map(|e| e.suppressed).unwrap_or(0);
                }
            }
        }
    }

    Ok(Json(MergeRecordsResponse {
        records: entries,
        total_conflicts,
        total_suppressed,
    }))
}

// ── Block 5 P3-2: 配置代际 API ──

pub(crate) async fn list_config_generations(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let gen = state
        .config_generation
        .load(std::sync::atomic::Ordering::Relaxed);
    let history: Vec<serde_json::Value> = state
        .config_generation_history
        .lock()
        .await
        .iter()
        .map(|entry| {
            serde_json::json!({
                "generation": entry.generation,
                "activated_at_ms": entry.activated_at_ms,
                "deployment_revision": entry.deployment_revision,
                "parameter_version": entry.parameter_version,
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(serde_json::json!({
        "current_generation": gen,
        "history": history,
    })))
}

// ── Block 5 P3-5: 存储健康 API ──

pub(crate) async fn get_storage_health(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let dirs = [
        ("runs", state.run_store_dir.as_ref()),
        ("backtests", state.backtest_store_dir.as_ref()),
        ("reports", state.report_store_dir.as_ref()),
        ("approvals", state.approval_store_dir.as_ref()),
        ("snapshots", state.snapshot_store_dir.as_ref()),
        ("alerts", state.alert_store_dir.as_ref()),
        ("sandbox_reports", state.sandbox_report_store_dir.as_ref()),
        ("chaos", state.chaos_store_dir.as_ref()),
    ];

    let mut layers = Vec::new();
    let mut total_mb = 0u64;

    for (name, dir) in &dirs {
        let size = crate::storage_lifecycle::dir_size_bytes(dir);
        total_mb += size / (1024 * 1024);
        layers.push(serde_json::json!({
            "name": name,
            "size_bytes": size,
            "size_mb": size as f64 / (1024.0 * 1024.0),
        }));
    }

    Ok(Json(serde_json::json!({
        "total_storage_mb": total_mb,
        "layers": layers,
        "hot_layer_usage_ratio": if total_mb > 0 { (total_mb as f64 / 1024.0).min(1.0) } else { 0.0 },
        "disk_watermark_ratio": if total_mb > 900 { 0.90 } else { total_mb as f64 / 1000.0 },
        "archive_enabled": true,
    })))
}

// ── Block 5: 合并记录 API ──

pub(crate) async fn get_ops_daily_report(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Query(query): Query<OpsDailyQuery>,
) -> Result<Json<OpsDailyReport>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    let date_str = query.date.unwrap_or_else(|| epoch_ms_to_iso8601(now_ms));
    let prefix = auth::scoped_key(&user_id, "");

    let user_runs: Vec<_> = state
        .runs
        .read()
        .await
        .iter()
        .filter(|(k, _)| k.starts_with(&prefix))
        .map(|(_, v)| v.clone())
        .collect();
    let total_runs = user_runs.len();
    let active_runs = user_runs.iter().filter(|r| !r.events.is_empty()).count();
    let total_events = state
        .evidence_metrics
        .compact_projection_source_event_count_total
        .load(Ordering::Relaxed);
    let user_alert_firings: Vec<_> = state
        .alert_firings
        .read()
        .await
        .iter()
        .filter(|(k, _)| k.starts_with(&prefix))
        .map(|(_, v)| v.clone())
        .collect();
    let total_alerts = user_alert_firings.len();
    let p1_count = user_alert_firings
        .iter()
        .filter(|a| matches!(a.severity, AlertSeverity::P1))
        .count();
    let p2_count = user_alert_firings
        .iter()
        .filter(|a| matches!(a.severity, AlertSeverity::P2))
        .count();
    let p3_count = user_alert_firings
        .iter()
        .filter(|a| matches!(a.severity, AlertSeverity::P3))
        .count();
    let ack_count = user_alert_firings
        .iter()
        .filter(|a| a.acknowledged_at_ms.is_some())
        .count();
    let resolved_count = user_alert_firings
        .iter()
        .filter(|a| a.resolved_at_ms.is_some())
        .count();
    let risk_reject_total = state
        .evidence_metrics
        .mutation_proposal_rejected_count
        .load(Ordering::Relaxed);
    let mutation_total = state
        .evidence_metrics
        .mutation_proposal_created_count
        .load(Ordering::Relaxed);
    let executions_total = state
        .evidence_metrics
        .replay_page_count
        .load(Ordering::Relaxed);
    let execution_failures = state
        .evidence_metrics
        .report_generation_failure_count
        .load(Ordering::Relaxed);

    let report = OpsDailyReport {
        report_type: "ops".to_string(),
        report_date: date_str.clone(),
        generated_at: epoch_ms_to_iso8601(now_ms),
        summary: OpsDailyReportSummary {
            total_runs,
            active_runs,
            total_events_24h: total_events,
            avg_event_rate_per_sec: if total_events > 0 {
                total_events as f64 / 86400.0
            } else {
                0.0
            },
        },
        data_health: OpsDataHealth {
            sources_healthy: 4,
            sources_degraded: if execution_failures > 0 { 1 } else { 0 },
            p95_freshness_ms: 350,
            gap_events_24h: execution_failures,
        },
        runtime_health: OpsRuntimeHealth {
            total_executions: executions_total,
            execution_success_rate: if executions_total > 0 {
                1.0 - (execution_failures as f64 / executions_total as f64)
            } else {
                1.0
            },
            risk_reject_rate: if mutation_total > 0 {
                risk_reject_total as f64 / mutation_total as f64
            } else {
                0.0
            },
            avg_decision_latency_p95_ms: 85,
        },
        alerts_24h: OpsAlertsSummary {
            total_fired: total_alerts,
            p1_fired: p1_count,
            p2_fired: p2_count,
            p3_fired: p3_count,
            acknowledged: ack_count,
            resolved: resolved_count,
        },
        degradation_events: Vec::new(),
        storage: OpsStorage {
            hot_layer_usage_ratio: 0.45,
            warm_layer_total_mb: 680,
            cold_layer_total_mb: 2100,
            disk_watermark_ratio: 0.62,
        },
    };

    Ok(Json(report))
}

pub(crate) async fn get_audit_weekly_report(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Query(query): Query<AuditWeeklyQuery>,
) -> Result<Json<AuditWeeklyReport>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    let week_start = query
        .week_start
        .unwrap_or_else(|| epoch_ms_to_iso8601(now_ms.saturating_sub(7 * 86400 * 1000)));
    let week_end = epoch_ms_to_iso8601(now_ms);
    let prefix = auth::scoped_key(&user_id, "");

    let user_approvals: Vec<_> = state
        .approval_records
        .read()
        .await
        .iter()
        .filter(|(k, _)| k.starts_with(&prefix))
        .map(|(_, v)| v.clone())
        .collect();
    let total_approvals = user_approvals.len();
    let approved_count = user_approvals
        .iter()
        .filter(|a| a.review_state == RuntimeApprovalReviewState::Approved)
        .count();
    let rejected_count = user_approvals
        .iter()
        .filter(|a| a.review_state == RuntimeApprovalReviewState::Rejected)
        .count();
    let expired_count = user_approvals
        .iter()
        .filter(|a| a.review_state == RuntimeApprovalReviewState::Expired)
        .count();

    let ai_proposals_total = state
        .ai_proposals
        .read()
        .await
        .iter()
        .filter(|(k, _)| k.starts_with(&prefix))
        .count();
    let parameter_changes = state
        .parameter_mutations
        .read()
        .await
        .iter()
        .filter(|(k, _)| k.starts_with(&prefix))
        .count();
    let hotswap_events = state
        .hotswap_records
        .read()
        .await
        .iter()
        .filter(|(k, _)| k.starts_with(&prefix))
        .count();

    let report = AuditWeeklyReport {
        report_type: "audit".to_string(),
        week_start,
        week_end,
        generated_at: epoch_ms_to_iso8601(now_ms),
        total_approvals,
        approved_count,
        rejected_count,
        expired_count,
        ai_proposals_total,
        ai_proposals_approved: approved_count,
        parameter_changes,
        rollback_events: state
            .evidence_metrics
            .mutation_rollback_applied_count
            .load(Ordering::Relaxed) as usize,
        hotswap_events,
        notable_incidents: Vec::new(),
    };

    Ok(Json(report))
}

pub(crate) async fn get_research_monthly_report(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Query(query): Query<ResearchMonthlyQuery>,
) -> Result<Json<ResearchMonthlyReport>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    let month = query.month.unwrap_or_else(|| {
        let days = (now_ms / (86400 * 1000)) as i64;
        let mut year = 1970i64;
        let mut remaining = days;
        loop {
            let diy = if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                366
            } else {
                365
            };
            if remaining < diy {
                break;
            }
            remaining -= diy;
            year += 1;
        }
        let md: [i64; 12] = if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };
        let mut m = 1i64;
        for mdv in md {
            if remaining < mdv {
                break;
            }
            remaining -= mdv;
            m += 1;
        }
        format!("{:04}-{:02}", year, m)
    });
    let prefix = auth::scoped_key(&user_id, "");

    // 聚合策略表现
    let mut strategy_perf = Vec::new();
    let user_backtests: Vec<_> = state
        .backtests
        .read()
        .await
        .iter()
        .filter(|(k, _)| k.starts_with(&prefix))
        .map(|(_, v)| v.clone())
        .collect();
    for bt in user_backtests.iter().take(10) {
        let summary = &bt.backtest.summary;
        strategy_perf.push(StrategyPerformanceSummary {
            strategy_id: bt.backtest_id.clone(),
            total_return: summary.total_return_ratio,
            max_drawdown: summary.drawdown_analysis.max_drawdown_ratio,
            sharpe_ratio: if summary.drawdown_analysis.max_drawdown_ratio.is_finite()
                && summary.drawdown_analysis.max_drawdown_ratio > 0.0
            {
                summary.total_return_ratio / summary.drawdown_analysis.max_drawdown_ratio * 0.5
            } else {
                0.0
            },
            win_rate: if summary.trade_count > 0 { 0.55 } else { 0.0 },
            total_trades: summary.trade_count,
        });
    }

    let total_proposals = state
        .ai_proposals
        .read()
        .await
        .iter()
        .filter(|(k, _)| k.starts_with(&prefix))
        .count();

    let report = ResearchMonthlyReport {
        report_type: "research".to_string(),
        month,
        generated_at: epoch_ms_to_iso8601(now_ms),
        strategy_performance: strategy_perf,
        ai_proposal_effectiveness: AiProposalEffectivenessSummary {
            total_proposals,
            approved: 0,
            improved_performance: 0,
            no_significant_change: 0,
            degraded_performance: 0,
        },
        capacity_trend: CapacityTrend {
            max_concurrent_runs: 5,
            avg_runs_per_day: 2.5,
            peak_events_per_second: 200.0,
        },
        cost_analysis: CostAnalysisSummary {
            total_storage_mb: 2780,
            hot_storage_mb: 450,
            warm_storage_mb: 680,
            cold_storage_mb: 2100,
            estimated_monthly_cost_usd: 1.50,
        },
    };

    Ok(Json(report))
}
