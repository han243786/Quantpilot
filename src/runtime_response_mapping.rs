use std::collections::BTreeMap;

use super::*;

fn map_open_order_summary(order: &OpenOrder) -> OpenOrderSummary {
    OpenOrderSummary {
        order_id: order.order_id.clone(),
        side: format!("{}", order.side),
        remaining_qty: order.remaining_qty,
        limit_price: order.limit_price,
        reserved_cash: order.reserved_cash,
        reserved_qty: order.reserved_qty,
    }
}

pub(super) fn account_summary(session: &SessionOutput) -> AccountSummary {
    account_summary_from_portfolio(&session.final_portfolio)
}

pub(super) fn account_summary_from_portfolio(portfolio: &PortfolioState) -> AccountSummary {
    AccountSummary {
        equity_estimate: portfolio.cash_balance + portfolio.total_net_notional,
        cash_balance: portfolio.cash_balance,
        available_cash_balance: portfolio.available_cash_balance,
        frozen_cash_balance: portfolio.frozen_cash_balance,
        total_leverage: portfolio.total_leverage,
        total_gross_notional: portfolio.total_gross_notional,
        total_net_notional: portfolio.total_net_notional,
        positions: portfolio.positions.len(),
        open_order_count: portfolio.open_orders.len(),
        open_orders: portfolio
            .open_orders
            .iter()
            .map(map_open_order_summary)
            .collect(),
    }
}

fn normalize_account_summary(mut account: AccountSummary) -> AccountSummary {
    if account.equity_estimate.abs() <= f64::EPSILON
        && (account.cash_balance.abs() > f64::EPSILON
            || account.total_net_notional.abs() > f64::EPSILON)
    {
        account.equity_estimate = account.cash_balance + account.total_net_notional;
    }
    account
}

#[derive(Clone, Copy)]
pub(super) enum RuntimeGovernanceMaterialization {
    CurrentRuntime,
    LoadedManifest,
}

fn is_legacy_default_governance(governance: &RuntimeGovernanceSnapshot) -> bool {
    let capability_hash = governance.capability_hash.trim();
    let deployment_revision = governance.deployment_revision.trim();
    capability_hash.is_empty()
        || capability_hash == "unknown"
        || deployment_revision.is_empty()
        || deployment_revision == "unknown"
}

fn normalize_governance_source(
    mut governance: RuntimeGovernanceSnapshot,
    materialization: RuntimeGovernanceMaterialization,
) -> RuntimeGovernanceSnapshot {
    governance.governance_source = if is_legacy_default_governance(&governance) {
        "legacy_default".to_string()
    } else {
        match materialization {
            RuntimeGovernanceMaterialization::CurrentRuntime => "current_runtime".to_string(),
            RuntimeGovernanceMaterialization::LoadedManifest => "loaded_manifest".to_string(),
        }
    };
    governance
}

pub(super) fn normalize_run_record(
    mut record: RunRecord,
    materialization: RuntimeGovernanceMaterialization,
) -> RunRecord {
    record.account = normalize_account_summary(record.account);
    record.governance = normalize_governance_source(record.governance, materialization);
    let mode = infer_runtime_event_mode(&record.events);
    if validate_runtime_event_envelopes(&record.events, &record.run_id, &record.governance).is_err()
    {
        repair_runtime_event_envelopes(
            &mut record.events,
            &record.run_id,
            mode.as_str(),
            &record.governance,
        );
    }
    record
}

pub(super) fn normalize_backtest_record(
    mut record: BacktestRecord,
    materialization: RuntimeGovernanceMaterialization,
) -> BacktestRecord {
    record.account = normalize_account_summary(record.account);
    record.governance = normalize_governance_source(record.governance, materialization);
    let mode = infer_runtime_event_mode(&record.events);
    let should_rebuild_artifacts = record
        .backtest_artifacts
        .as_ref()
        .map(|artifacts| {
            // 如果已有有效数据（step_count > 0），跳过重建
            if artifacts.metrics.summary.step_count > 0 {
                return false;
            }
            backtest_artifacts_need_governance_rebuild(artifacts, &record.governance)
        })
        .unwrap_or(false);
    if validate_runtime_event_envelopes(&record.events, &record.backtest_id, &record.governance)
        .is_err()
    {
        repair_runtime_event_envelopes(
            &mut record.events,
            &record.backtest_id,
            mode.as_str(),
            &record.governance,
        );
    }
    if should_rebuild_artifacts {
        if let Ok(artifacts) = build_backtest_artifact_views(&record) {
            record.backtest_artifacts = Some(artifacts);
        }
    }
    if let Some(artifacts) = &mut record.backtest_artifacts {
        if validate_runtime_event_envelopes(
            &artifacts.event_log.events,
            &record.backtest_id,
            &record.governance,
        )
        .is_err()
        {
            repair_runtime_event_envelopes(
                &mut artifacts.event_log.events,
                &record.backtest_id,
                mode.as_str(),
                &record.governance,
            );
        }
        artifacts.manifest.governance = record.governance.clone();
        artifacts.metrics.final_account =
            normalize_account_summary(artifacts.metrics.final_account.clone());
        artifacts.manifest.account = normalize_account_summary(artifacts.manifest.account.clone());
    }
    record
}

fn backtest_artifacts_need_governance_rebuild(
    artifacts: &BacktestArtifactViews,
    governance: &RuntimeGovernanceSnapshot,
) -> bool {
    artifacts
        .event_log
        .events
        .iter()
        .any(|event| event.envelope.event_id.trim().is_empty())
        || serde_json::to_value(&artifacts.manifest.governance).ok()
            != serde_json::to_value(governance).ok()
}

fn infer_runtime_event_mode(events: &[FrontendRuntimeEvent]) -> String {
    events
        .iter()
        .find_map(|event| {
            let envelope_mode = event.envelope.mode.trim();
            if !envelope_mode.is_empty() && envelope_mode != "unknown" {
                return Some(envelope_mode.to_string());
            }
            event
                .payload
                .get("runtime_mode")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn dataset_filter_labels(datasets: &[DatasetSpec]) -> Vec<String> {
    datasets
        .iter()
        .map(|dataset| {
            let interval = dataset.interval.as_deref().unwrap_or("na");
            format!("{:?}:{:?}:{interval}", dataset.exchange, dataset.symbol)
        })
        .collect()
}

pub(super) fn backtest_filter_metadata(record: &BacktestRecord) -> BacktestFilterMetadata {
    let artifacts = record
        .backtest_artifacts
        .as_ref()
        .expect("backtest artifact views should exist for backtest filters");
    let spec = artifacts
        .manifest
        .backtest_spec
        .as_ref()
        .expect("backtest manifest should embed backtest spec");
    let metrics = &artifacts.metrics;

    BacktestFilterMetadata {
        replay_source: Some(spec.replay_source.clone()),
        dataset_labels: dataset_filter_labels(&spec.run_spec.datasets),
        execution_assumptions_tag: metrics
            .execution_assumptions
            .as_ref()
            .map(|module| module.list_tag.clone()),
        started_at_ms: Some(metrics.started_at_ms),
        ended_at_ms: Some(metrics.ended_at_ms),
    }
}

pub(super) fn run_start_response(
    run_id: String,
    graph_id: String,
    compile_id: String,
    event_count: usize,
) -> RunStartResponse {
    RunStartResponse {
        run_id,
        graph_id,
        compile_id,
        event_count,
        status: "queued",
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn backtest_run_response(
    backtest_id: String,
    graph_id: String,
    compile_id: String,
    protocol_name: String,
    config_hash: String,
    event_count: usize,
    account: AccountSummary,
    backtest_artifacts: BacktestArtifactViews,
) -> BacktestRunResponse {
    BacktestRunResponse {
        backtest_id,
        graph_id,
        compile_id,
        protocol_name,
        config_hash,
        event_count,
        account,
        execution_assumptions: backtest_artifacts.metrics.execution_assumptions.clone(),
        backtest_artifacts,
    }
}

pub(super) fn run_list_item_from_record(record: RunRecord) -> RunListItem {
    RunListItem {
        run_id: record.run_id,
        graph_id: record.graph_id,
        compile_id: record.compile_id,
        created_at_ms: record.created_at_ms,
        event_count: record.events.len(),
        account: record.account,
        actor: record.actor,
    }
}

pub(super) fn run_detail_response_from_record(record: RunRecord) -> RunDetailResponse {
    let runtime_diagnostics =
        build_runtime_diagnostics_from_events(&record.events, "runtime_events");
    let timeline = timeline_items_from_events(&record.events);
    let retained_key_event_index =
        retained_key_event_index_from_timeline(&timeline, record.events.len());
    let compact_evidence =
        compact_evidence_projection_from_timeline(&timeline, &retained_key_event_index);
    RunDetailResponse {
        run_id: record.run_id,
        graph_id: record.graph_id,
        compile_id: record.compile_id,
        created_at_ms: record.created_at_ms,
        event_count: record.events.len(),
        account: record.account,
        events: record.events,
        timeline,
        retained_key_event_index,
        compact_evidence,
        runtime_diagnostics,
        session: record.session,
        governance: record.governance,
        actor: record.actor,
    }
}

pub(super) fn run_status_response_from_record(record: RunRecord) -> RunStatusResponse {
    RunStatusResponse {
        run_id: record.run_id,
        graph_id: record.graph_id,
        compile_id: record.compile_id,
        event_count: record.events.len(),
        account: record.account,
    }
}

pub(super) fn backtest_list_item_from_record(record: BacktestRecord) -> BacktestListItem {
    let filters = backtest_filter_metadata(&record);
    BacktestListItem {
        backtest_id: record.backtest_id,
        graph_id: record.graph_id,
        compile_id: record.compile_id,
        created_at_ms: record.created_at_ms,
        protocol_name: record.protocol_name,
        config_hash: record.config_hash,
        event_count: record.events.len(),
        account: record.account,
        summary: record
            .backtest_artifacts
            .as_ref()
            .expect("backtest artifact views should exist for list responses")
            .metrics
            .summary
            .clone(),
        filters,
        actor: record.actor,
    }
}

pub(super) fn backtest_detail_response_from_record(
    record: BacktestRecord,
) -> BacktestDetailResponse {
    let diagnostics_source = if record.backtest_artifacts.is_some() {
        "backtest_event_log"
    } else {
        "runtime_events"
    };
    let diagnostic_events = record
        .backtest_artifacts
        .as_ref()
        .map(|artifacts| artifacts.event_log.events.as_slice())
        .unwrap_or(record.events.as_slice());
    let timeline = timeline_items_from_events(diagnostic_events);
    let retained_key_event_index =
        retained_key_event_index_from_timeline(&timeline, diagnostic_events.len());
    let compact_evidence =
        compact_evidence_projection_from_timeline(&timeline, &retained_key_event_index);
    BacktestDetailResponse {
        backtest_id: record.backtest_id,
        graph_id: record.graph_id,
        compile_id: record.compile_id,
        created_at_ms: record.created_at_ms,
        protocol_name: record.protocol_name,
        config_hash: record.config_hash,
        event_count: record.events.len(),
        account: record.account,
        execution_assumptions: record
            .backtest_artifacts
            .as_ref()
            .and_then(|artifacts| artifacts.metrics.execution_assumptions.clone()),
        runtime_diagnostics: build_runtime_diagnostics_from_events(
            diagnostic_events,
            diagnostics_source,
        ),
        timeline,
        retained_key_event_index,
        compact_evidence,
        backtest_artifacts: record
            .backtest_artifacts
            .expect("backtest artifact views should exist for detail responses"),
        governance: record.governance,
        actor: record.actor,
    }
}

fn timeline_compactability_for_retention(
    retention_class: RuntimeEventRetentionClass,
) -> RuntimeTimelineCompactability {
    match retention_class {
        RuntimeEventRetentionClass::Key => RuntimeTimelineCompactability::Retain,
        RuntimeEventRetentionClass::Summary => RuntimeTimelineCompactability::Summarize,
        RuntimeEventRetentionClass::Debug => RuntimeTimelineCompactability::DropCandidate,
    }
}

pub(super) fn timeline_item_from_event(event: &FrontendRuntimeEvent) -> RuntimeTimelineItem {
    let envelope = &event.envelope;
    let occurred_at_ms = if envelope.occurred_at_ms > 0 {
        envelope.occurred_at_ms
    } else {
        event.event_time_ms
    };
    let ingested_at_ms = if envelope.ingested_at_ms > 0 {
        envelope.ingested_at_ms
    } else {
        occurred_at_ms
    };
    let module_key = if envelope.module_key.trim().is_empty() {
        event.source_id.clone()
    } else {
        envelope.module_key.clone()
    };
    let severity = if envelope.severity.trim().is_empty() {
        event.severity.clone()
    } else {
        envelope.severity.clone()
    };

    RuntimeTimelineItem {
        timeline_item_version: 1,
        event_id: event.event_id.clone(),
        event_type: event.event_type.clone(),
        sequence_no: envelope.sequence_no,
        occurred_at_ms,
        ingested_at_ms,
        stage: envelope.stage,
        retention_class: envelope.retention_class,
        severity,
        module_key,
        node_id: event.node_id.clone(),
        summary: event.summary.clone(),
        reason_code: envelope.reason_code.clone(),
        governance: RuntimeTimelineGovernanceIdentity {
            capability_hash: envelope.capability_hash.clone(),
            deployment_revision: envelope.deployment_revision.clone(),
            strategy_version: envelope.strategy_version.clone(),
            parameter_version: envelope.parameter_version.clone(),
        },
        payload_version: envelope.payload_version,
        compactability: timeline_compactability_for_retention(envelope.retention_class),
    }
}

pub(super) fn timeline_items_from_events(
    events: &[FrontendRuntimeEvent],
) -> Vec<RuntimeTimelineItem> {
    events.iter().map(timeline_item_from_event).collect()
}

fn is_system_governance_event_type(event_type: &str) -> bool {
    matches!(
        event_type,
        "CapabilitySnapshotTaken"
            | "SecurityViolationDetected"
            | "AIProposalCreated"
            | "AIProposalDenied"
            | "AIProposalStaticCheckPassed"
            | "AIProposalStaticCheckFailed"
            | "ParameterMutationProposed"
            | "ParameterMutationRejected"
            | "ParameterMutationActivationScheduled"
            | "ParameterMutationActivated"
            | "ParameterMutationActivationFailed"
            | "ParameterMutationSafeWindowDenied"
            | "ParameterMutationRollbackScheduled"
            | "ParameterMutationRolledBack"
            | "ParameterMutationRollbackFailed"
    )
}

fn stage_contract_value(stage: RuntimeEventStage) -> &'static str {
    match stage {
        RuntimeEventStage::Data => "data",
        RuntimeEventStage::Intent => "intent",
        RuntimeEventStage::Agent => "agent",
        RuntimeEventStage::Risk => "risk",
        RuntimeEventStage::Execution => "execution",
        RuntimeEventStage::Fill => "fill",
        RuntimeEventStage::System => "system",
    }
}

fn retention_contract_value(retention_class: RuntimeEventRetentionClass) -> &'static str {
    match retention_class {
        RuntimeEventRetentionClass::Key => "key",
        RuntimeEventRetentionClass::Summary => "summary",
        RuntimeEventRetentionClass::Debug => "debug",
    }
}

pub(super) fn is_retained_key_timeline_item(item: &RuntimeTimelineItem) -> bool {
    item.retention_class == RuntimeEventRetentionClass::Key
        || is_system_governance_event_type(&item.event_type)
}

pub(super) fn retained_key_event_index_from_timeline(
    timeline: &[RuntimeTimelineItem],
    source_event_count: usize,
) -> RuntimeRetainedKeyEventIndex {
    let entries = timeline
        .iter()
        .filter(|item| is_retained_key_timeline_item(item))
        .cloned()
        .collect::<Vec<_>>();
    let key_event_count = entries
        .iter()
        .filter(|item| item.retention_class == RuntimeEventRetentionClass::Key)
        .count();
    let system_event_count = entries
        .iter()
        .filter(|item| is_system_governance_event_type(&item.event_type))
        .count();

    RuntimeRetainedKeyEventIndex {
        index_version: 1,
        policy_version: "quantpilot/key-event-index/v1".to_string(),
        source_event_count,
        retained_event_count: entries.len(),
        key_event_count,
        system_event_count,
        entries,
    }
}

fn unknown_timeline_governance_identity() -> RuntimeTimelineGovernanceIdentity {
    RuntimeTimelineGovernanceIdentity {
        capability_hash: "unknown".to_string(),
        deployment_revision: "unknown".to_string(),
        strategy_version: "unknown".to_string(),
        parameter_version: "unknown".to_string(),
    }
}

pub(super) fn compact_evidence_projection_from_timeline(
    timeline: &[RuntimeTimelineItem],
    retained_index: &RuntimeRetainedKeyEventIndex,
) -> RuntimeCompactEvidenceProjection {
    let retained_ids = retained_index
        .entries
        .iter()
        .map(|item| item.event_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let mut dropped_by_retention = BTreeMap::new();
    let mut dropped_by_stage = BTreeMap::new();

    for item in timeline
        .iter()
        .filter(|item| !retained_ids.contains(item.event_id.as_str()))
    {
        *dropped_by_retention
            .entry(retention_contract_value(item.retention_class).to_string())
            .or_insert(0) += 1;
        *dropped_by_stage
            .entry(stage_contract_value(item.stage).to_string())
            .or_insert(0) += 1;
    }

    let governance = retained_index
        .entries
        .last()
        .or_else(|| timeline.last())
        .map(|item| item.governance.clone())
        .unwrap_or_else(unknown_timeline_governance_identity);

    RuntimeCompactEvidenceProjection {
        projection_version: 1,
        policy_version: "quantpilot/evidence-compaction/v1".to_string(),
        source_event_count: timeline.len(),
        retained_event_count: retained_index.entries.len(),
        dropped_event_count: timeline.len().saturating_sub(retained_index.entries.len()),
        dropped_by_retention,
        dropped_by_stage,
        key_event_count: retained_index.key_event_count,
        system_event_count: retained_index.system_event_count,
        governance,
        entries: retained_index.entries.clone(),
    }
}

const RUNTIME_REPORT_GENERATION_POLICY_VERSION: &str = "quantpilot/report-policy/v1";

fn report_source_sequence_range(
    compact: &RuntimeCompactEvidenceProjection,
) -> Option<RuntimeReportSourceSequenceRange> {
    let from = compact.entries.iter().map(|item| item.sequence_no).min()?;
    let to = compact.entries.iter().map(|item| item.sequence_no).max()?;
    Some(RuntimeReportSourceSequenceRange { from, to })
}

fn has_report_governance_identity(governance: &RuntimeTimelineGovernanceIdentity) -> bool {
    for value in [
        governance.capability_hash.as_str(),
        governance.deployment_revision.as_str(),
        governance.strategy_version.as_str(),
        governance.parameter_version.as_str(),
    ] {
        let value = value.trim();
        if value.is_empty() || value == "unknown" {
            return false;
        }
    }
    true
}

fn sanitize_report_id_segment(value: &str) -> String {
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

fn report_source_kind_value(source_kind: RuntimeEvidenceSourceKind) -> &'static str {
    match source_kind {
        RuntimeEvidenceSourceKind::Run => "run",
        RuntimeEvidenceSourceKind::Backtest => "backtest",
    }
}

fn report_id_for_source(
    source_kind: RuntimeEvidenceSourceKind,
    source_id: &str,
    graph_id: &str,
    range: &Option<RuntimeReportSourceSequenceRange>,
    generation_policy: &str,
    governance: &RuntimeTimelineGovernanceIdentity,
) -> String {
    let digest = canonical_json_sha256_digest(&json!({
        "source_kind": report_source_kind_value(source_kind),
        "source_id": source_id,
        "graph_id": graph_id,
        "range": range,
        "generation_policy": generation_policy,
        "governance": governance,
    }))
    .expect("report metadata payload should serialize for canonical hashing");
    let digest_short = &digest.value[..digest.value.len().min(12)];
    format!(
        "report_{}_{}_{}",
        report_source_kind_value(source_kind),
        sanitize_report_id_segment(source_id),
        digest_short
    )
}

fn report_record_from_compact_evidence(
    source_kind: RuntimeEvidenceSourceKind,
    source_id: String,
    graph_id: String,
    compact: RuntimeCompactEvidenceProjection,
    now_ms: u64,
    generation_policy: Option<String>,
) -> RuntimeEvidenceReportRecord {
    let generation_policy = generation_policy
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| RUNTIME_REPORT_GENERATION_POLICY_VERSION.to_string());
    let source_sequence_range = report_source_sequence_range(&compact);
    let mutation_lifecycle_event_count = compact
        .entries
        .iter()
        .filter(|entry| entry.event_type.starts_with("ParameterMutation"))
        .count();
    let ready = source_sequence_range.is_some()
        && compact.source_event_count > 0
        && compact.retained_event_count > 0
        && has_report_governance_identity(&compact.governance);
    let status = if ready {
        RuntimeReportLifecycleStatus::Ready
    } else {
        RuntimeReportLifecycleStatus::Failed
    };
    let failure = if ready {
        None
    } else {
        Some(RuntimeReportFailureMetadata {
            reason_code: "missing_source_evidence_or_governance".to_string(),
            message:
                "source evidence metadata, retained evidence, and governance identity are required"
                    .to_string(),
            retry_eligible: true,
        })
    };
    let failure_reason = failure.as_ref().map(|failure| failure.message.clone());
    let report_id = report_id_for_source(
        source_kind,
        &source_id,
        &graph_id,
        &source_sequence_range,
        &generation_policy,
        &compact.governance,
    );
    let artifacts = if ready {
        vec![
            RuntimeReportArtifactMetadata {
                kind: "metadata".to_string(),
                artifact_id: format!("{}_metadata", report_id),
                file_name: format!("{}.json", report_id),
                content_type: "application/json".to_string(),
            },
            RuntimeReportArtifactMetadata {
                kind: "evidence_report".to_string(),
                artifact_id: format!("{}_evidence_report", report_id),
                file_name: format!("{}_evidence_report.json", report_id),
                content_type: "application/json".to_string(),
            },
        ]
    } else {
        Vec::new()
    };

    RuntimeEvidenceReportRecord {
        report_id,
        source_kind,
        source_id,
        graph_id,
        status,
        source_sequence_range,
        source_event_count: compact.source_event_count,
        retained_event_count: compact.retained_event_count,
        mutation_lifecycle_event_count,
        governance: compact.governance,
        generation_policy,
        artifacts,
        failure_reason,
        failure,
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    }
}

pub(super) fn runtime_report_record_from_run_record(
    record: RunRecord,
    now_ms: u64,
    generation_policy: Option<String>,
) -> RuntimeEvidenceReportRecord {
    let detail = run_detail_response_from_record(record);
    report_record_from_compact_evidence(
        RuntimeEvidenceSourceKind::Run,
        detail.run_id,
        detail.graph_id,
        detail.compact_evidence,
        now_ms,
        generation_policy,
    )
}

pub(super) fn runtime_report_record_from_backtest_record(
    record: BacktestRecord,
    now_ms: u64,
    generation_policy: Option<String>,
) -> RuntimeEvidenceReportRecord {
    let detail = backtest_detail_response_from_record(record);
    report_record_from_compact_evidence(
        RuntimeEvidenceSourceKind::Backtest,
        detail.backtest_id,
        detail.graph_id,
        detail.compact_evidence,
        now_ms,
        generation_policy,
    )
}

fn runtime_report_evidence_digest(record: &RuntimeEvidenceReportRecord) -> String {
    let digest = canonical_json_sha256_digest(&json!({
        "report_id": record.report_id,
        "source_kind": report_source_kind_value(record.source_kind),
        "source_id": record.source_id,
        "graph_id": record.graph_id,
        "source_sequence_range": record.source_sequence_range,
        "source_event_count": record.source_event_count,
        "retained_event_count": record.retained_event_count,
        "governance": record.governance,
        "generation_policy": record.generation_policy,
    }))
    .expect("report evidence payload should serialize for canonical hashing");
    format!("sha256:{}", digest.value)
}

pub(super) fn runtime_report_artifact_from_record(
    record: &RuntimeEvidenceReportRecord,
) -> RuntimeEvidenceReportArtifact {
    let range_summary = record
        .source_sequence_range
        .as_ref()
        .map(|range| format!("sequence {}-{}", range.from, range.to))
        .unwrap_or_else(|| "no retained source sequence range".to_string());
    let lifecycle_summary = match record.status {
        RuntimeReportLifecycleStatus::Ready => {
            format!(
                "Ready report for {} {} with {} retained evidence items from {} source events.",
                report_source_kind_value(record.source_kind),
                record.source_id,
                record.retained_event_count,
                record.source_event_count
            )
        }
        RuntimeReportLifecycleStatus::Failed => record
            .failure_reason
            .clone()
            .unwrap_or_else(|| "Report generation failed.".to_string()),
        RuntimeReportLifecycleStatus::Requested => "Report generation was requested.".to_string(),
        RuntimeReportLifecycleStatus::Generating => "Report generation is running.".to_string(),
        RuntimeReportLifecycleStatus::Expired => "Report artifact expired.".to_string(),
        RuntimeReportLifecycleStatus::SourceChanged => {
            "Report source evidence changed after generation.".to_string()
        }
    };

    RuntimeEvidenceReportArtifact {
        schema_version: "quantpilot/evidence-report-artifact/v1".to_string(),
        report_id: record.report_id.clone(),
        source_kind: record.source_kind,
        source_id: record.source_id.clone(),
        graph_id: record.graph_id.clone(),
        status: record.status,
        source_sequence_range: record.source_sequence_range.clone(),
        source_event_count: record.source_event_count,
        retained_event_count: record.retained_event_count,
        mutation_lifecycle_event_count: record.mutation_lifecycle_event_count,
        governance: record.governance.clone(),
        generation_policy: record.generation_policy.clone(),
        evidence_digest: runtime_report_evidence_digest(record),
        loading_strategy: RuntimeReportLoadingStrategy {
            primary_source: "compact_evidence".to_string(),
            source_event_count: record.source_event_count,
            retained_event_count: record.retained_event_count,
            requires_detail_window: record.retained_event_count == 0
                || record.retained_event_count == record.source_event_count,
        },
        sections: {
            let mut sections = vec![
                RuntimeEvidenceReportSection {
                    section_id: "lifecycle".to_string(),
                    title: "Lifecycle".to_string(),
                    summary: lifecycle_summary,
                    evidence_count: record.retained_event_count,
                },
                RuntimeEvidenceReportSection {
                    section_id: "source_range".to_string(),
                    title: "Source sequence range".to_string(),
                    summary: range_summary,
                    evidence_count: record.source_event_count,
                },
                RuntimeEvidenceReportSection {
                    section_id: "governance".to_string(),
                    title: "Governance identity".to_string(),
                    summary: format!(
                        "{} / {} / {} / {}",
                        record.governance.capability_hash,
                        record.governance.deployment_revision,
                        record.governance.strategy_version,
                        record.governance.parameter_version
                    ),
                    evidence_count: 1,
                },
            ];
            if record.mutation_lifecycle_event_count > 0 {
                sections.push(RuntimeEvidenceReportSection {
                    section_id: "mutation_lifecycle".to_string(),
                    title: "Mutation lifecycle".to_string(),
                    summary: format!(
                        "{} retained parameter mutation lifecycle events are linked to this report.",
                        record.mutation_lifecycle_event_count
                    ),
                    evidence_count: record.mutation_lifecycle_event_count,
                });
            }
            sections
        },
    }
}

fn experiment_sweep_axes(definition: &ExperimentDefinitionSummary) -> Vec<String> {
    let mut axes = Vec::new();
    if definition.parameter_grid.fee_bps.len() > 1 {
        axes.push("fee_bps".to_string());
    }
    if definition.parameter_grid.slippage_bps.len() > 1 {
        axes.push("slippage_bps".to_string());
    }
    if definition.parameter_grid.latency_ms.len() > 1 {
        axes.push("latency_ms".to_string());
    }
    axes
}

pub(super) fn experiment_list_item_from_record(record: ExperimentRecord) -> ExperimentListItem {
    let best_variant = record.variants.iter().max_by(|left, right| {
        left.summary
            .total_return_ratio
            .partial_cmp(&right.summary.total_return_ratio)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    ExperimentListItem {
        experiment_id: record.experiment_id,
        graph_id: record.graph_id,
        compile_id: record.compile_id,
        created_at_ms: record.created_at_ms,
        experiment_name: record.definition.experiment_name.clone(),
        replay_source: record.definition.replay_source,
        variant_count: record.variants.len(),
        sweep_axes: experiment_sweep_axes(&record.definition),
        best_backtest_id: best_variant.map(|variant| variant.backtest_id.clone()),
        best_total_return_ratio: best_variant.map(|variant| variant.summary.total_return_ratio),
    }
}

pub(super) fn experiment_detail_response_from_record(
    record: ExperimentRecord,
) -> ExperimentDetailResponse {
    ExperimentDetailResponse {
        experiment_id: record.experiment_id,
        graph_id: record.graph_id,
        compile_id: record.compile_id,
        created_at_ms: record.created_at_ms,
        definition: record.definition,
        variants: record.variants,
    }
}

fn replay_checkpoint_interval(total_events: usize, limit: usize) -> usize {
    total_events.clamp(limit.max(1), 50)
}

fn event_sequence_no(event: &FrontendRuntimeEvent, fallback: usize) -> u64 {
    if event.envelope.sequence_no > 0 {
        event.envelope.sequence_no
    } else {
        fallback as u64
    }
}

fn replay_checkpoints(
    events: &[FrontendRuntimeEvent],
    limit: usize,
) -> Vec<RuntimeReplayCheckpoint> {
    if events.is_empty() {
        return Vec::new();
    }

    let interval = replay_checkpoint_interval(events.len(), limit);
    events
        .iter()
        .enumerate()
        .step_by(interval)
        .map(|(index, event)| RuntimeReplayCheckpoint {
            cursor: index,
            sequence_cursor: event_sequence_no(event, index + 1),
            label: format!(
                "{}-{}",
                event_sequence_no(event, index + 1),
                events
                    .get((index + limit).min(events.len()).saturating_sub(1))
                    .map(|item| event_sequence_no(item, index + limit))
                    .unwrap_or_else(|| event_sequence_no(event, index + 1))
            ),
            event_id: Some(event.event_id.clone()),
            event_time_ms: Some(event.event_time_ms),
        })
        .collect()
}

fn stage_filter_value(stage: RuntimeEventStage) -> &'static str {
    match stage {
        RuntimeEventStage::Data => "data",
        RuntimeEventStage::Intent => "intent",
        RuntimeEventStage::Agent => "agent",
        RuntimeEventStage::Risk => "risk",
        RuntimeEventStage::Execution => "execution",
        RuntimeEventStage::Fill => "fill",
        RuntimeEventStage::System => "system",
    }
}

fn retention_filter_value(retention_class: RuntimeEventRetentionClass) -> &'static str {
    match retention_class {
        RuntimeEventRetentionClass::Key => "key",
        RuntimeEventRetentionClass::Summary => "summary",
        RuntimeEventRetentionClass::Debug => "debug",
    }
}

fn normalized_filter_text(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn event_matches_replay_filters(
    event: &FrontendRuntimeEvent,
    filters: &RuntimeReplayFilters,
) -> bool {
    let envelope = &event.envelope;
    if filters.key_only && envelope.retention_class != RuntimeEventRetentionClass::Key {
        return false;
    }
    if let Some(stage) = filters.stage.as_deref() {
        if normalized_filter_text(stage) != stage_filter_value(envelope.stage) {
            return false;
        }
    }
    if let Some(retention_class) = filters.retention_class.as_deref() {
        if normalized_filter_text(retention_class)
            != retention_filter_value(envelope.retention_class)
        {
            return false;
        }
    }
    if let Some(severity) = filters.severity.as_deref() {
        if !event.severity.eq_ignore_ascii_case(severity)
            && !envelope.severity.eq_ignore_ascii_case(severity)
        {
            return false;
        }
    }
    if let Some(module_key) = filters.module_key.as_deref() {
        if envelope.module_key != module_key && event.source_id != module_key {
            return false;
        }
    }
    true
}

fn filtered_replay_events(
    events: &[FrontendRuntimeEvent],
    filters: &RuntimeReplayFilters,
) -> Vec<FrontendRuntimeEvent> {
    let mut filtered = events
        .iter()
        .filter(|event| event_matches_replay_filters(event, filters))
        .cloned()
        .collect::<Vec<_>>();
    filtered.sort_by_key(|event| event_sequence_no(event, 0));
    filtered
}

fn cursor_from_replay_options(
    events: &[FrontendRuntimeEvent],
    options: &RuntimeReplayOptions,
) -> Result<usize, String> {
    if events.is_empty() {
        if options.cursor == 0 && options.sequence_cursor.is_none() {
            return Ok(0);
        }
        return Err("重放游标超出空过滤重放窗口范围".to_string());
    }
    if let Some(sequence_cursor) = options.sequence_cursor {
        return events
            .iter()
            .position(|event| event_sequence_no(event, 0) >= sequence_cursor)
            .ok_or_else(|| format!("sequence_cursor `{sequence_cursor}` 超出过滤重放窗口范围"));
    }
    if options.cursor >= events.len() {
        return Err(format!(
            "游标 `{}` 超出 {} 个事件的过滤重放窗口范围",
            options.cursor,
            events.len()
        ));
    }
    Ok(options.cursor)
}

fn replay_event_items(
    events: &[FrontendRuntimeEvent],
    cursor: usize,
    limit: usize,
) -> Vec<RuntimeReplayEventItem> {
    let start = cursor.min(events.len());
    let end = (start + limit).min(events.len());
    events[start..end]
        .iter()
        .enumerate()
        .map(|(offset, event)| RuntimeReplayEventItem {
            sequence_no: event_sequence_no(event, start + offset + 1) as usize,
            event: event.clone(),
        })
        .collect()
}

fn runtime_replay_response(
    kind: RuntimeReplayRecordKind,
    record_id: String,
    graph_id: String,
    account: AccountSummary,
    events: &[FrontendRuntimeEvent],
    options: RuntimeReplayOptions,
) -> Result<RuntimeReplayResponse, String> {
    let filtered_events = filtered_replay_events(events, &options.filters);
    let bounded_limit = options.limit.max(1);
    let bounded_cursor = cursor_from_replay_options(&filtered_events, &options)?;
    let items = replay_event_items(&filtered_events, bounded_cursor, bounded_limit);
    let timeline = timeline_items_from_events(
        &filtered_events[bounded_cursor.min(filtered_events.len())
            ..(bounded_cursor + bounded_limit).min(filtered_events.len())],
    );
    let window_end = items
        .last()
        .map(|item| item.sequence_no)
        .unwrap_or(bounded_cursor);
    let fill_event_count = items
        .iter()
        .filter(|item| item.event.event_type == "ExecutionFilled")
        .count();
    let previous_cursor = if bounded_cursor == 0 {
        None
    } else {
        Some(bounded_cursor.saturating_sub(bounded_limit))
    };
    let next_cursor = if bounded_cursor + bounded_limit < filtered_events.len() {
        Some(bounded_cursor + bounded_limit)
    } else {
        None
    };
    let sequence_cursor = filtered_events
        .get(bounded_cursor)
        .map(|event| event_sequence_no(event, bounded_cursor + 1));
    let previous_sequence_cursor = previous_cursor.and_then(|cursor| {
        filtered_events
            .get(cursor)
            .map(|event| event_sequence_no(event, cursor + 1))
    });
    let next_sequence_cursor = next_cursor.and_then(|cursor| {
        filtered_events
            .get(cursor)
            .map(|event| event_sequence_no(event, cursor + 1))
    });

    Ok(RuntimeReplayResponse {
        kind,
        record_id,
        graph_id,
        source_event_count: events.len(),
        total_events: filtered_events.len(),
        cursor: bounded_cursor,
        sequence_cursor,
        limit: bounded_limit,
        window_end,
        fill_event_count,
        account,
        filters: options.filters,
        checkpoints: replay_checkpoints(&filtered_events, bounded_limit),
        events: items,
        timeline,
        previous_cursor,
        next_cursor,
        previous_sequence_cursor,
        next_sequence_cursor,
    })
}

pub(super) fn run_replay_response_from_record(
    record: RunRecord,
    options: RuntimeReplayOptions,
) -> Result<RuntimeReplayResponse, String> {
    runtime_replay_response(
        RuntimeReplayRecordKind::Run,
        record.run_id,
        record.graph_id,
        record.account,
        &record.events,
        options,
    )
}

pub(super) fn backtest_replay_response_from_record(
    record: BacktestRecord,
    options: RuntimeReplayOptions,
) -> Result<RuntimeReplayResponse, String> {
    let replay_events = record
        .backtest_artifacts
        .as_ref()
        .map(|artifacts| artifacts.event_log.events.as_slice())
        .unwrap_or(record.events.as_slice());
    runtime_replay_response(
        RuntimeReplayRecordKind::Backtest,
        record.backtest_id,
        record.graph_id,
        record.account,
        replay_events,
        options,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_timeline_event() -> FrontendRuntimeEvent {
        FrontendRuntimeEvent {
            event_id: "evt_timeline_001".to_string(),
            event_type: "RiskDecisionProduced".to_string(),
            source_id: "builtin.risk.global".to_string(),
            node_id: "risk_node".to_string(),
            event_time_ms: 1_710_000_000_000,
            severity: "Warn".to_string(),
            summary: "Risk limit clamped the target.".to_string(),
            payload: json!({ "reason_code": "MAX_WEIGHT_CLAMPED" }),
            envelope: RuntimeEventEnvelope {
                event_id: "evt_timeline_001".to_string(),
                event_type: "RiskDecisionProduced".to_string(),
                stage: RuntimeEventStage::Risk,
                run_id: "run_timeline_001".to_string(),
                sequence_no: 42,
                occurred_at_ms: 1_710_000_000_010,
                ingested_at_ms: 1_710_000_000_020,
                trace_id: Some("trace_timeline_001".to_string()),
                module_key: "builtin.risk.global".to_string(),
                strategy_version: "strategy:v1".to_string(),
                parameter_version: "config:abc".to_string(),
                deployment_revision: "sha256:deployment".to_string(),
                capability_hash: "sha256:capability".to_string(),
                mode: "paper".to_string(),
                severity: "Warn".to_string(),
                retention_class: RuntimeEventRetentionClass::Key,
                reason_code: Some("MAX_WEIGHT_CLAMPED".to_string()),
                payload_version: 1,
            },
        }
    }

    #[test]
    fn timeline_item_contract_derives_required_governed_fields_from_event_envelope() {
        let item = timeline_item_from_event(&sample_timeline_event());

        assert_eq!(item.timeline_item_version, 1);
        assert_eq!(item.event_id, "evt_timeline_001");
        assert_eq!(item.event_type, "RiskDecisionProduced");
        assert_eq!(item.sequence_no, 42);
        assert_eq!(item.stage, RuntimeEventStage::Risk);
        assert_eq!(item.retention_class, RuntimeEventRetentionClass::Key);
        assert_eq!(item.severity, "Warn");
        assert_eq!(item.module_key, "builtin.risk.global");
        assert_eq!(item.node_id, "risk_node");
        assert_eq!(item.summary, "Risk limit clamped the target.");
        assert_eq!(item.reason_code.as_deref(), Some("MAX_WEIGHT_CLAMPED"));
        assert_eq!(item.governance.capability_hash, "sha256:capability");
        assert_eq!(item.governance.deployment_revision, "sha256:deployment");
        assert_eq!(item.governance.strategy_version, "strategy:v1");
        assert_eq!(item.governance.parameter_version, "config:abc");
        assert_eq!(item.payload_version, 1);
        assert_eq!(item.compactability, RuntimeTimelineCompactability::Retain);
    }

    #[test]
    fn timeline_item_contract_serializes_to_stable_frontend_strings() {
        let value = serde_json::to_value(timeline_item_from_event(&sample_timeline_event()))
            .expect("timeline item should serialize");

        assert_eq!(value["timeline_item_version"], 1);
        assert_eq!(value["stage"], "risk");
        assert_eq!(value["retention_class"], "key");
        assert_eq!(value["compactability"], "retain");
        assert_eq!(value["governance"]["capability_hash"], "sha256:capability");
        assert_eq!(
            value["governance"]["deployment_revision"],
            "sha256:deployment"
        );
    }

    #[test]
    fn retained_key_event_index_keeps_key_and_system_governance_events() {
        let key_event = sample_timeline_event();
        let mut debug_event = sample_timeline_event();
        debug_event.event_id = "evt_debug".to_string();
        debug_event.event_type = "AgentDecisionProduced".to_string();
        debug_event.envelope.event_id = debug_event.event_id.clone();
        debug_event.envelope.event_type = debug_event.event_type.clone();
        debug_event.envelope.sequence_no = 43;
        debug_event.envelope.stage = RuntimeEventStage::Agent;
        debug_event.envelope.retention_class = RuntimeEventRetentionClass::Debug;

        let mut system_event = sample_timeline_event();
        system_event.event_id = "evt_security".to_string();
        system_event.event_type = "SecurityViolationDetected".to_string();
        system_event.envelope.event_id = system_event.event_id.clone();
        system_event.envelope.event_type = system_event.event_type.clone();
        system_event.envelope.sequence_no = 44;
        system_event.envelope.stage = RuntimeEventStage::System;
        system_event.envelope.retention_class = RuntimeEventRetentionClass::Summary;

        let timeline = timeline_items_from_events(&[key_event, debug_event, system_event]);
        let index = retained_key_event_index_from_timeline(&timeline, 3);

        assert_eq!(index.index_version, 1);
        assert_eq!(index.policy_version, "quantpilot/key-event-index/v1");
        assert_eq!(index.source_event_count, 3);
        assert_eq!(index.retained_event_count, 2);
        assert_eq!(index.key_event_count, 1);
        assert_eq!(index.system_event_count, 1);
        assert_eq!(
            index
                .entries
                .iter()
                .map(|item| item.event_type.as_str())
                .collect::<Vec<_>>(),
            vec!["RiskDecisionProduced", "SecurityViolationDetected"]
        );
    }

    #[test]
    fn compact_evidence_projection_records_drop_counts_and_keeps_key_entries() {
        let key_event = sample_timeline_event();
        let mut summary_event = sample_timeline_event();
        summary_event.event_id = "evt_summary".to_string();
        summary_event.event_type = "RuntimeWarning".to_string();
        summary_event.envelope.event_id = summary_event.event_id.clone();
        summary_event.envelope.event_type = summary_event.event_type.clone();
        summary_event.envelope.sequence_no = 43;
        summary_event.envelope.stage = RuntimeEventStage::System;
        summary_event.envelope.retention_class = RuntimeEventRetentionClass::Summary;

        let mut debug_event = sample_timeline_event();
        debug_event.event_id = "evt_debug".to_string();
        debug_event.event_type = "AgentDecisionProduced".to_string();
        debug_event.envelope.event_id = debug_event.event_id.clone();
        debug_event.envelope.event_type = debug_event.event_type.clone();
        debug_event.envelope.sequence_no = 44;
        debug_event.envelope.stage = RuntimeEventStage::Agent;
        debug_event.envelope.retention_class = RuntimeEventRetentionClass::Debug;

        let timeline = timeline_items_from_events(&[key_event, summary_event, debug_event]);
        let index = retained_key_event_index_from_timeline(&timeline, 3);
        let compact = compact_evidence_projection_from_timeline(&timeline, &index);

        assert_eq!(compact.projection_version, 1);
        assert_eq!(compact.policy_version, "quantpilot/evidence-compaction/v1");
        assert_eq!(compact.source_event_count, 3);
        assert_eq!(compact.retained_event_count, 1);
        assert_eq!(compact.dropped_event_count, 2);
        assert_eq!(compact.dropped_by_retention.get("summary"), Some(&1));
        assert_eq!(compact.dropped_by_retention.get("debug"), Some(&1));
        assert_eq!(compact.dropped_by_stage.get("system"), Some(&1));
        assert_eq!(compact.dropped_by_stage.get("agent"), Some(&1));
        assert_eq!(compact.entries[0].event_type, "RiskDecisionProduced");
        assert_eq!(compact.governance.capability_hash, "sha256:capability");
    }

    #[test]
    fn report_lifecycle_refuses_ready_without_source_evidence_and_governance_identity() {
        let compact = RuntimeCompactEvidenceProjection {
            projection_version: 1,
            policy_version: "quantpilot/evidence-compaction/v1".to_string(),
            source_event_count: 1,
            retained_event_count: 1,
            dropped_event_count: 0,
            dropped_by_retention: BTreeMap::new(),
            dropped_by_stage: BTreeMap::new(),
            key_event_count: 1,
            system_event_count: 0,
            governance: unknown_timeline_governance_identity(),
            entries: vec![timeline_item_from_event(&sample_timeline_event())],
        };

        let report = report_record_from_compact_evidence(
            RuntimeEvidenceSourceKind::Run,
            "run_legacy".to_string(),
            "graph_legacy".to_string(),
            compact,
            1_710_000_000_000,
            None,
        );

        assert_eq!(report.status, RuntimeReportLifecycleStatus::Failed);
        assert!(report.failure_reason.is_some());
        assert!(report.artifacts.is_empty());
    }
}
