use super::*;

fn map_open_order_summary(order: &OpenOrder) -> OpenOrderSummary {
    OpenOrderSummary {
        order_id: order.order_id.clone(),
        side: format!("{:?}", order.side),
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

pub(super) fn normalize_run_record(mut record: RunRecord) -> RunRecord {
    record.account = normalize_account_summary(record.account);
    record
}

pub(super) fn normalize_backtest_record(mut record: BacktestRecord) -> BacktestRecord {
    record.account = normalize_account_summary(record.account);
    if let Some(artifacts) = &mut record.backtest_artifacts {
        artifacts.metrics.final_account =
            normalize_account_summary(artifacts.metrics.final_account.clone());
        artifacts.manifest.account = normalize_account_summary(artifacts.manifest.account.clone());
    }
    record
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
    let runtime_diagnostics = build_runtime_diagnostics_from_events(&record.events, "runtime_events");
    RunDetailResponse {
        run_id: record.run_id,
        graph_id: record.graph_id,
        compile_id: record.compile_id,
        created_at_ms: record.created_at_ms,
        event_count: record.events.len(),
        account: record.account,
        events: record.events,
        runtime_diagnostics,
        session: record.session,
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
        backtest_artifacts: record
            .backtest_artifacts
            .expect("backtest artifact views should exist for detail responses"),
        actor: record.actor,
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
    let best_variant = record
        .variants
        .iter()
        .max_by(|left, right| {
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
            label: format!("{}-{}", index + 1, (index + limit).min(events.len())),
            event_id: Some(event.event_id.clone()),
            event_time_ms: Some(event.event_time_ms),
        })
        .collect()
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
            sequence_no: start + offset + 1,
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
    cursor: usize,
    limit: usize,
) -> RuntimeReplayResponse {
    let bounded_limit = limit.max(1);
    let bounded_cursor = cursor.min(events.len().saturating_sub(1));
    let items = replay_event_items(events, bounded_cursor, bounded_limit);
    let window_end = items
        .last()
        .map(|item| item.sequence_no)
        .unwrap_or(bounded_cursor);
    let fill_event_count = items
        .iter()
        .filter(|item| item.event.event_type == "ExecutionFilled")
        .count();
    let previous_cursor = bounded_cursor.checked_sub(bounded_limit);
    let next_cursor = if bounded_cursor + bounded_limit < events.len() {
        Some(bounded_cursor + bounded_limit)
    } else {
        None
    };

    RuntimeReplayResponse {
        kind,
        record_id,
        graph_id,
        total_events: events.len(),
        cursor: bounded_cursor,
        limit: bounded_limit,
        window_end,
        fill_event_count,
        account,
        checkpoints: replay_checkpoints(events, bounded_limit),
        events: items,
        previous_cursor,
        next_cursor,
    }
}

pub(super) fn run_replay_response_from_record(
    record: RunRecord,
    cursor: usize,
    limit: usize,
) -> RuntimeReplayResponse {
    runtime_replay_response(
        RuntimeReplayRecordKind::Run,
        record.run_id,
        record.graph_id,
        record.account,
        &record.events,
        cursor,
        limit,
    )
}

pub(super) fn backtest_replay_response_from_record(
    record: BacktestRecord,
    cursor: usize,
    limit: usize,
) -> RuntimeReplayResponse {
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
        cursor,
        limit,
    )
}
