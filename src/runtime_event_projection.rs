use super::*;

#[derive(Clone, Copy)]
struct BacktestEventProjectionContext<'a> {
    session_index: usize,
    cycle_name: &'a str,
    session_started_at_ms: u64,
}

fn build_frontend_events(
    events: &[RuntimeEvent],
    runtime_targets: &CompileRuntimeTargets,
    projection_context: Option<BacktestEventProjectionContext<'_>>,
) -> Vec<FrontendRuntimeEvent> {
    events
        .iter()
        .map(|event| FrontendRuntimeEvent {
            event_id: event.event_id.clone(),
            event_type: runtime_event_name(&event.event_type).to_string(),
            source_id: event.source_id.clone(),
            node_id: resolve_event_node_id(
                event,
                &runtime_targets.source_to_node,
                runtime_targets.runtime_node_id.as_deref().unwrap_or_default(),
                runtime_targets.execution_node_id.as_deref().unwrap_or_default(),
            ),
            event_time_ms: event.ts_ms,
            severity: severity_for_event(&event.event_type).to_string(),
            summary: summarize_event(event),
            payload: annotate_frontend_event_payload(
                &event.payload,
                &event.trace_id,
                projection_context,
            ),
        })
        .collect()
}

fn annotate_frontend_event_payload(
    payload: &Value,
    trace_id: &str,
    projection_context: Option<BacktestEventProjectionContext<'_>>,
) -> Value {
    let Some(context) = projection_context else {
        return payload.clone();
    };

    let Value::Object(mut object) = payload.clone() else {
        return payload.clone();
    };

    object.insert("trace_id".to_string(), Value::String(trace_id.to_string()));
    object.insert(
        "artifact_projection".to_string(),
        json!({
            "session_index": context.session_index,
            "cycle_name": context.cycle_name,
            "session_started_at_ms": context.session_started_at_ms,
        }),
    );

    Value::Object(object)
}

pub(super) fn collect_frontend_events(
    session: &SessionOutput,
    runtime_targets: &CompileRuntimeTargets,
) -> Vec<FrontendRuntimeEvent> {
    let mut events =
        build_frontend_events(&session.slow_cycle.runtime_events, runtime_targets, None);
    events.extend(build_frontend_events(
        &session.fast_cycle.runtime_events,
        runtime_targets,
        None,
    ));
    events.sort_by_key(|item| item.event_time_ms);
    events
}

pub(super) fn collect_frontend_events_for_backtest(
    backtest: &BacktestOutput,
    runtime_targets: &CompileRuntimeTargets,
) -> Vec<FrontendRuntimeEvent> {
    let mut events = backtest
        .sessions
        .iter()
        .enumerate()
        .flat_map(|(session_index, session)| {
            let session_started_at_ms = backtest
                .equity_curve
                .get(session_index)
                .map(|point| point.ts_ms)
                .unwrap_or(session.final_portfolio.updated_at_ms);
            let slow_context = BacktestEventProjectionContext {
                session_index,
                cycle_name: "slow",
                session_started_at_ms,
            };
            let fast_context = BacktestEventProjectionContext {
                session_index,
                cycle_name: "fast",
                session_started_at_ms,
            };
            let mut session_events = build_frontend_events(
                &session.slow_cycle.runtime_events,
                runtime_targets,
                Some(slow_context),
            );
            session_events.extend(build_frontend_events(
                &session.fast_cycle.runtime_events,
                runtime_targets,
                Some(fast_context),
            ));
            session_events
        })
        .collect::<Vec<_>>();
    events.sort_by_key(|item| item.event_time_ms);
    events
}

fn resolve_event_node_id(
    event: &RuntimeEvent,
    source_to_node: &BTreeMap<String, String>,
    runtime_node_id: &str,
    execution_node_id: &str,
) -> String {
    if let Some(node_id) = source_to_node.get(&event.source_id) {
        return node_id.clone();
    }

    match event.event_type {
        RuntimeEventType::ExecutionPlanned | RuntimeEventType::ExecutionFilled => {
            execution_node_id.to_string()
        }
        _ => runtime_node_id.to_string(),
    }
}

fn summarize_event(event: &RuntimeEvent) -> String {
    if let Some(summary) = event
        .payload
        .get("explanation_summary")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
    {
        return summary.to_string();
    }

    match event.event_type {
        RuntimeEventType::DataUpdated => format!("{} data updated", event.source_id),
        RuntimeEventType::IntentTriggered => {
            let side = event
                .payload
                .get("side")
                .and_then(Value::as_str)
                .unwrap_or("UNKNOWN");
            format!("{} triggered side={}", event.source_id, side)
        }
        RuntimeEventType::AgentDecisionProduced => {
            let side = event
                .payload
                .get("net_side")
                .and_then(Value::as_str)
                .unwrap_or("UNKNOWN");
            let strength = event
                .payload
                .get("net_strength")
                .and_then(Value::as_f64)
                .unwrap_or_default();
            format!("{} produced {} {:.4}", event.source_id, side, strength)
        }
        RuntimeEventType::RiskDecisionProduced => {
            let status = event
                .payload
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("UNKNOWN");
            format!("{} risk {}", event.source_id, status)
        }
        RuntimeEventType::ExecutionPlanned => {
            let orders = event
                .payload
                .get("orders")
                .and_then(Value::as_u64)
                .unwrap_or_default();
            format!("execution planned with {} orders", orders)
        }
        RuntimeEventType::ExecutionFilled => {
            let qty = event
                .payload
                .get("qty")
                .and_then(Value::as_f64)
                .unwrap_or_default();
            let price = event
                .payload
                .get("price")
                .and_then(Value::as_f64)
                .unwrap_or_default();
            format!("filled qty {:.6} @ {:.2}", qty, price)
        }
        RuntimeEventType::PortfolioUpdated => {
            let equity = event
                .payload
                .get("equity_estimate")
                .and_then(Value::as_f64)
                .or_else(|| {
                    let cash = event.payload.get("cash_balance").and_then(Value::as_f64)?;
                    let exposure = event
                        .payload
                        .get("total_net_notional")
                        .and_then(Value::as_f64)
                        .unwrap_or_default();
                    Some(cash + exposure)
                })
                .unwrap_or_default();
            format!("portfolio updated equity {:.2}", equity)
        }
        RuntimeEventType::IntentEvaluated => format!("{} evaluated", event.source_id),
        RuntimeEventType::RuntimeWarning => format!("warning from {}", event.source_id),
        RuntimeEventType::RuntimeError => format!("error from {}", event.source_id),
    }
}

fn runtime_event_name(event_type: &RuntimeEventType) -> &'static str {
    match event_type {
        RuntimeEventType::DataUpdated => "DataUpdated",
        RuntimeEventType::IntentEvaluated => "IntentEvaluated",
        RuntimeEventType::IntentTriggered => "IntentTriggered",
        RuntimeEventType::AgentDecisionProduced => "AgentDecisionProduced",
        RuntimeEventType::RiskDecisionProduced => "RiskDecisionProduced",
        RuntimeEventType::ExecutionPlanned => "ExecutionPlanned",
        RuntimeEventType::ExecutionFilled => "ExecutionFilled",
        RuntimeEventType::PortfolioUpdated => "PortfolioUpdated",
        RuntimeEventType::RuntimeWarning => "RuntimeWarning",
        RuntimeEventType::RuntimeError => "RuntimeError",
    }
}

fn severity_for_event(event_type: &RuntimeEventType) -> &'static str {
    match event_type {
        RuntimeEventType::RuntimeError => "Error",
        RuntimeEventType::RuntimeWarning => "Warn",
        _ => "Info",
    }
}

pub(super) fn json_sse_event(name: &str, payload: impl Serialize) -> Event {
    let data = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
    Event::default().event(name).data(data)
}
