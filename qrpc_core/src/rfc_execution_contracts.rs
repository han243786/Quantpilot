use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::Symbol;

mod allocation;
mod data_request;
mod execution_feedback;
mod order_contract;

pub use allocation::*;
pub use data_request::*;
pub use execution_feedback::*;
pub use order_contract::*;

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
