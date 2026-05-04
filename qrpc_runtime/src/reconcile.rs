use qrpc_core::{ExecutionStatus, OpenOrder, OrderSide, RuntimeEvent, RuntimeEventType};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconcileStrategy {
    Drain,
    CancelOpen,
    ReduceOnly,
}

impl Default for ReconcileStrategy {
    fn default() -> Self {
        Self::CancelOpen
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationResult {
    pub strategy: ReconcileStrategy,
    pub orders_before: usize,
    pub orders_after: usize,
    pub cancelled: Vec<String>,
    pub retained: Vec<String>,
    pub drained: Vec<String>,
    pub discrepancies: Vec<ReconciliationDiscrepancy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationDiscrepancy {
    pub order_id: String,
    pub local_status: String,
    pub external_status: Option<String>,
    pub resolved_action: String,
    pub resolved_by: String,
}

#[derive(Debug, Clone, Default)]
pub struct OrderReconciler;

impl OrderReconciler {
    pub fn reconcile(
        open_orders: &[OpenOrder],
        external_statuses: &[(String, ExecutionStatus)],
        strategy: ReconcileStrategy,
    ) -> ReconciliationResult {
        let orders_before = open_orders.len();
        let external_map: std::collections::BTreeMap<_, _> =
            external_statuses.iter().cloned().collect();

        let mut cancelled = Vec::new();
        let mut retained = Vec::new();
        let mut drained = Vec::new();
        let mut discrepancies = Vec::new();

        // First pass: detect discrepancies between local and external state
        for order in open_orders {
            if let Some(external_status) = external_map.get(&order.order_id) {
                let local_terminal = matches!(
                    order_remaining_status(order),
                    "filled" | "cancelled" | "expired" | "rejected"
                );
                let external_terminal = matches!(
                    external_status,
                    ExecutionStatus::Filled
                        | ExecutionStatus::Cancelled
                        | ExecutionStatus::Expired
                        | ExecutionStatus::Rejected
                        | ExecutionStatus::Failed
                );

                if !local_terminal && external_terminal {
                    discrepancies.push(ReconciliationDiscrepancy {
                        order_id: order.order_id.clone(),
                        local_status: order_remaining_status(order).to_string(),
                        external_status: Some(format!("{:?}", external_status)),
                        resolved_action: "accept_external".to_string(),
                        resolved_by: "exchange".to_string(),
                    });
                }
            }
        }

        // Second pass: apply the reconciliation strategy
        for order in open_orders {
            let local_terminal = matches!(
                order_remaining_status(order),
                "filled" | "cancelled" | "expired" | "rejected"
            );

            let externally_resolved = external_map.get(&order.order_id).map_or(false, |ext| {
                matches!(
                    ext,
                    ExecutionStatus::Filled
                        | ExecutionStatus::Cancelled
                        | ExecutionStatus::Expired
                        | ExecutionStatus::Rejected
                        | ExecutionStatus::Failed
                )
            });

            if local_terminal || externally_resolved {
                drained.push(order.order_id.clone());
                continue;
            }

            match strategy {
                ReconcileStrategy::Drain => {
                    retained.push(order.order_id.clone());
                }
                ReconcileStrategy::CancelOpen => {
                    cancelled.push(order.order_id.clone());
                }
                ReconcileStrategy::ReduceOnly => {
                    if matches!(order.side, OrderSide::Sell)
                        && order.remaining_qty > 0.0
                    {
                        retained.push(order.order_id.clone());
                    } else {
                        cancelled.push(order.order_id.clone());
                    }
                }
            }
        }

        ReconciliationResult {
            strategy,
            orders_before,
            orders_after: retained.len(),
            cancelled,
            retained,
            drained,
            discrepancies,
        }
    }

    pub fn build_reconciliation_events(
        result: &ReconciliationResult,
        now_ms: u64,
        trace_id: &str,
    ) -> Vec<RuntimeEvent> {
        let mut events = Vec::new();

        for discrepancy in &result.discrepancies {
            events.push(RuntimeEvent {
                event_id: format!("evt-reconcile-disc-{}-{now_ms}", discrepancy.order_id),
                event_type: RuntimeEventType::RuntimeWarning,
                trace_id: trace_id.to_string(),
                source_id: "order_reconciler".to_string(),
                ts_ms: now_ms,
                payload: json!({
                    "event": "reconciliation_discrepancy",
                    "order_id": discrepancy.order_id,
                    "local_status": discrepancy.local_status,
                    "external_status": discrepancy.external_status,
                    "resolved_action": discrepancy.resolved_action,
                    "resolved_by": discrepancy.resolved_by,
                }),
            });
        }

        if !result.cancelled.is_empty() || !result.drained.is_empty() {
            events.push(RuntimeEvent {
                event_id: format!("evt-reconcile-summary-{now_ms}"),
                event_type: RuntimeEventType::RuntimeWarning,
                trace_id: trace_id.to_string(),
                source_id: "order_reconciler".to_string(),
                ts_ms: now_ms,
                payload: json!({
                    "event": "reconciliation_summary",
                    "strategy": format!("{:?}", result.strategy),
                    "orders_before": result.orders_before,
                    "orders_after": result.orders_after,
                    "cancelled_count": result.cancelled.len(),
                    "retained_count": result.retained.len(),
                    "drained_count": result.drained.len(),
                    "discrepancy_count": result.discrepancies.len(),
                }),
            });
        }

        events
    }
}

fn order_remaining_status(order: &OpenOrder) -> &'static str {
    if order.remaining_qty <= 0.0 {
        "filled"
    } else {
        "open"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_core::{Exchange, OrderType, Symbol, TimeInForce};

    fn sample_order(id: &str, side: OrderSide, remaining_qty: f64) -> OpenOrder {
        let is_buy = matches!(side, OrderSide::Buy);
        let is_sell = matches!(side, OrderSide::Sell);
        OpenOrder {
            order_id: id.to_string(),
            plan_id: format!("plan-{id}"),
            exchange: Exchange::Binance,
            symbol: Symbol::BtcUsdt,
            side,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::Gtc,
            remaining_qty,
            reserved_cash: if is_buy { remaining_qty * 50_000.0 } else { 0.0 },
            reserved_qty: if is_sell { remaining_qty } else { 0.0 },
            limit_price: Some(50_000.0),
            reference_price: 50_000.0,
            created_at_ms: 0,
            updated_at_ms: 0,
            trace_id: "trace".to_string(),
        }
    }

    #[test]
    fn drain_keeps_all_open_orders() {
        let orders = vec![
            sample_order("o1", OrderSide::Buy, 0.5),
            sample_order("o2", OrderSide::Sell, 0.3),
        ];
        let result = OrderReconciler::reconcile(&orders, &[], ReconcileStrategy::Drain);
        assert_eq!(result.orders_before, 2);
        assert_eq!(result.orders_after, 2);
        assert!(result.cancelled.is_empty());
        assert_eq!(result.retained.len(), 2);
    }

    #[test]
    fn cancel_open_cancels_all() {
        let orders = vec![
            sample_order("o1", OrderSide::Buy, 0.5),
            sample_order("o2", OrderSide::Sell, 0.3),
        ];
        let result = OrderReconciler::reconcile(&orders, &[], ReconcileStrategy::CancelOpen);
        assert_eq!(result.orders_before, 2);
        assert_eq!(result.orders_after, 0);
        assert_eq!(result.cancelled.len(), 2);
    }

    #[test]
    fn reduce_only_retains_only_sells() {
        let orders = vec![
            sample_order("buy1", OrderSide::Buy, 0.5),
            sample_order("sell1", OrderSide::Sell, 0.3),
            sample_order("buy2", OrderSide::Buy, 0.2),
            sample_order("sell2", OrderSide::Sell, 0.1),
        ];
        let result = OrderReconciler::reconcile(&orders, &[], ReconcileStrategy::ReduceOnly);
        assert_eq!(result.orders_before, 4);
        assert_eq!(result.orders_after, 2);
        assert_eq!(result.cancelled.len(), 2);
        assert!(result.retained.iter().all(|id| id.starts_with("sell")));
    }

    #[test]
    fn detects_external_discrepancies() {
        let orders = vec![sample_order("o1", OrderSide::Buy, 0.5)];
        let externals = vec![("o1".to_string(), ExecutionStatus::Filled)];
        let result =
            OrderReconciler::reconcile(&orders, &externals, ReconcileStrategy::CancelOpen);
        assert_eq!(result.discrepancies.len(), 1);
        assert_eq!(result.discrepancies[0].order_id, "o1");
        assert_eq!(result.discrepancies[0].resolved_action, "accept_external");
        assert_eq!(result.drained.len(), 1);
    }

    #[test]
    fn empty_orders_yields_empty_result() {
        let result = OrderReconciler::reconcile(&[], &[], ReconcileStrategy::Drain);
        assert_eq!(result.orders_before, 0);
        assert_eq!(result.orders_after, 0);
    }

    #[test]
    fn reconciliation_events_are_produced() {
        let orders = vec![
            sample_order("buy1", OrderSide::Buy, 0.5),
            sample_order("sell1", OrderSide::Sell, 0.3),
        ];
        let result = OrderReconciler::reconcile(&orders, &[], ReconcileStrategy::CancelOpen);
        let events = OrderReconciler::build_reconciliation_events(&result, 1_000, "trace");
        assert!(!events.is_empty());
        // Should have at least the summary event
        assert!(events.iter().any(|e| e
            .payload["event"]
            .as_str()
            .map_or(false, |s| s == "reconciliation_summary")));
    }
}
