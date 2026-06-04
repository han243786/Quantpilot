use serde::{Deserialize, Serialize};

use super::RuntimeEvent;
use crate::{Exchange, ExecutionStatus, OrderSide, OrderType, Symbol, TimeInForce};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimOrder {
    pub order_id: String,
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub limit_price: Option<f64>,
    pub time_in_force: TimeInForce,
    pub allow_partial: bool,
    pub reference_price: f64,
    pub slippage_bps: f64,
    pub fee_bps: f64,
    pub strategy_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub plan_id: String,
    pub source_risk_decision_id: String,
    pub orders: Vec<SimOrder>,
    pub created_at_ms: u64,
    pub trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillReport {
    pub fill_id: String,
    pub plan_id: String,
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub side: OrderSide,
    pub filled_qty: f64,
    pub filled_price: f64,
    pub fee_paid: f64,
    pub filled_at_ms: u64,
    pub status: ExecutionStatus,
    pub trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenOrder {
    pub order_id: String,
    pub plan_id: String,
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub time_in_force: TimeInForce,
    pub remaining_qty: f64,
    pub reserved_cash: f64,
    pub reserved_qty: f64,
    pub limit_price: Option<f64>,
    pub reference_price: f64,
    #[serde(default)]
    pub slippage_bps: f64,
    #[serde(default)]
    pub fee_bps: f64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillResult {
    pub plan_id: String,
    pub status: ExecutionStatus,
    pub fills: Vec<FillReport>,
    pub open_orders: Vec<OpenOrder>,
    pub events: Vec<RuntimeEvent>,
}
