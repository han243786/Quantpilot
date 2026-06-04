use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::Symbol;

mod allocation;
mod data_request;
mod order_contract;

pub use allocation::*;
pub use data_request::*;
pub use order_contract::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FeedbackKind {
    OrderSubmitted,
    OrderRejected,
    OrderPartiallyFilled,
    OrderFilled,
    OrderCancelled,
    OrderExpired,
    VenueError,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionFeedback {
    pub feedback_id: String,
    pub order_id: String,
    pub kind: FeedbackKind,
    pub fill_qty: Option<f64>,
    pub fill_price: Option<f64>,
    pub remaining_qty: Option<f64>,
    pub reject_reason: Option<String>,
    pub venue_message: Option<String>,
    pub occurred_at_ms: u64,
    pub ingested_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HandoffSnapshot {
    pub snapshot_id: String,
    pub source_strategy_id: String,
    pub positions: BTreeMap<Symbol, f64>,
    pub open_orders: Vec<Order>,
    pub cash_balance: f64,
    pub available_cash_balance: f64,
    pub frozen_cash_balance: f64,
    pub current_cycle: u64,
    pub snapshot_at_ms: u64,
}

impl HandoffSnapshot {
    pub fn validate_completeness(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.snapshot_id.is_empty() {
            errors.push("snapshot_id 涓嶈兘涓虹┖".to_string());
        }
        if self.source_strategy_id.is_empty() {
            errors.push("source_strategy_id 涓嶈兘涓虹┖".to_string());
        }
        for order in &self.open_orders {
            if order.order_id.is_empty() {
                errors.push(format!(
                    "鏈粨璁㈠崟缂哄皯 order_id (symbol={:?})",
                    order.instrument
                ));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
