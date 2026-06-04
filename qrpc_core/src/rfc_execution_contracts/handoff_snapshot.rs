use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::Order;
use crate::Symbol;

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
            errors.push("snapshot_id 娑撳秷鍏樻稉铏光敄".to_string());
        }
        if self.source_strategy_id.is_empty() {
            errors.push("source_strategy_id 娑撳秷鍏樻稉铏光敄".to_string());
        }
        for order in &self.open_orders {
            if order.order_id.is_empty() {
                errors.push(format!(
                    "閺堫亞绮ㄧ拋銏犲礋缂傚搫鐨?order_id (symbol={:?})",
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
