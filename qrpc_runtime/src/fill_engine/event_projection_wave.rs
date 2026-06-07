use super::*;

pub(super) fn open_event(open_order: &OpenOrder, now_ms: u64, trace_id: &str) -> RuntimeEvent {
    RuntimeEvent {
        event_id: format!("evt-order-open-{}-{now_ms}", open_order.order_id),
        event_type: RuntimeEventType::ExecutionPlanned,
        trace_id: trace_id.to_string(),
        source_id: open_order.plan_id.clone(),
        ts_ms: now_ms,
        payload: json!({
            "status": "Open",
            "lifecycle_stage": "open",
            "order_id": open_order.order_id,
            "remaining_qty": open_order.remaining_qty,
            "limit_price": open_order.limit_price,
            "reserved_cash": open_order.reserved_cash,
            "reserved_qty": open_order.reserved_qty,
            "reason_text": "resting order remains open until the market reaches the limit price",
            "explanation_summary": "Resting order is open and waiting for the market to cross the limit price.",
        }),
    }
}

pub(super) fn partial_event(
    plan_id: &str,
    order_id: &str,
    remaining_qty: f64,
    now_ms: u64,
    trace_id: &str,
) -> RuntimeEvent {
    RuntimeEvent {
        event_id: format!("evt-order-partial-{}-{now_ms}", order_id),
        event_type: RuntimeEventType::ExecutionFilled,
        trace_id: trace_id.to_string(),
        source_id: plan_id.to_string(),
        ts_ms: now_ms,
        payload: json!({
            "status": "PartiallyFilled",
            "lifecycle_stage": "partial_fill",
            "order_id": order_id,
            "remaining_qty": remaining_qty,
            "reason_text": "available liquidity filled only part of the order",
            "explanation_summary": format!(
                "Order partially filled and still has {:.4} remaining.",
                remaining_qty
            ),
        }),
    }
}

pub(super) fn cancel_event(
    plan_id: &str,
    order_id: &str,
    reason: &str,
    now_ms: u64,
    trace_id: &str,
) -> RuntimeEvent {
    RuntimeEvent {
        event_id: format!("evt-order-cancel-{}-{now_ms}", order_id),
        event_type: RuntimeEventType::ExecutionPlanned,
        trace_id: trace_id.to_string(),
        source_id: plan_id.to_string(),
        ts_ms: now_ms,
        payload: json!({
            "status": "Cancelled",
            "lifecycle_stage": "cancelled",
            "order_id": order_id,
            "reason": reason,
            "reason_text": reason,
            "explanation_summary": format!("Order cancelled: {reason}."),
        }),
    }
}

pub(super) fn reject_event(
    plan_id: &str,
    order_id: &str,
    reason: &str,
    now_ms: u64,
    trace_id: &str,
) -> RuntimeEvent {
    let explanation_summary = if reason.ends_with('?') {
        format!("Order rejected: {reason}")
    } else {
        format!("Order rejected: {reason}.")
    };
    RuntimeEvent {
        event_id: format!("evt-order-reject-{}-{now_ms}", order_id),
        event_type: RuntimeEventType::ExecutionPlanned,
        trace_id: trace_id.to_string(),
        source_id: plan_id.to_string(),
        ts_ms: now_ms,
        payload: json!({
            "status": "Rejected",
            "lifecycle_stage": "rejected",
            "order_id": order_id,
            "reason": reason,
            "reason_text": reason,
            "explanation_summary": explanation_summary,
        }),
    }
}

pub(super) fn fill_event(
    fill: &FillReport,
    order_id: &str,
    now_ms: u64,
    trace_id: &str,
) -> RuntimeEvent {
    RuntimeEvent {
        event_id: format!("evt-fill-{}-{now_ms}", fill.fill_id),
        event_type: RuntimeEventType::ExecutionFilled,
        trace_id: trace_id.to_string(),
        source_id: fill.plan_id.clone(),
        ts_ms: now_ms,
        payload: json!({
            "fill_id": fill.fill_id,
            "plan_id": fill.plan_id,
            "exchange": format!("{:?}", fill.exchange),
            "symbol": format!("{:?}", fill.symbol),
            "side": format!("{:?}", fill.side),
            "qty": fill.filled_qty,
            "price": fill.filled_price,
            "fee_paid": fill.fee_paid,
            "exec_status": format!("{:?}", fill.status),
            "filled_at_ms": fill.filled_at_ms,
            "order_id": order_id,
            "lifecycle_stage": if matches!(fill.status, ExecutionStatus::Filled) {
                "completed"
            } else {
                "partial_fill"
            },
            "explanation_summary": format!(
                "Filled {:.4} at {:.2} after execution reached the market.",
                fill.filled_qty,
                fill.filled_price
            ),
        }),
    }
}
