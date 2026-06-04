use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{Exchange, OrderSide, OrderType, PortfolioTarget, Symbol, TimeInForce};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MarketScope {
    Spot,
    Margin,
    Perpetual,
    Futures,
    Options,
    Index,
    Composite,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PrimaryDataType {
    FactPrice,
    KlineRange,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SourceType {
    SpotTrade,
    SpotTicker,
    PerpetualTrade,
    PerpetualMark,
    PerpetualIndex,
    FuturesTrade,
    FuturesMark,
    FuturesIndex,
    IndexPrice,
    Aggregated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Timeframe {
    Tick,
    Ms100,
    Sec1,
    Sec5,
    Min1,
    Min3,
    Min5,
    Min15,
    Min30,
    Hour1,
    Hour4,
    Day1,
    Week1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeRange {
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoundingMode {
    Floor,
    Ceil,
    Round,
    Truncate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrecisionPolicy {
    pub price_scale: u8,
    pub quantity_scale: u8,
    pub rounding_mode: RoundingMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UsageTag {
    LiveExecution,
    IntentComputation,
    FactSimulation,
    HistoricalBacktest,
    Diagnostics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataRequest {
    pub request_id: String,
    pub instrument: Symbol,
    pub market_scope: MarketScope,
    pub primary_data_type: PrimaryDataType,
    pub source_type: SourceType,
    pub timeframe: Option<Timeframe>,
    pub lookback_count: Option<u32>,
    pub time_range: Option<TimeRange>,
    pub precision_policy: PrecisionPolicy,
    pub usage_tag: UsageTag,
    pub priority: u8,
    pub is_realtime: bool,
    pub requested_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AllocationMethod {
    EqualWeight,
    FixedWeight,
    RankWeight,
    ScoreWeight,
    RiskParity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Allocation {
    pub allocation_id: String,
    pub method: AllocationMethod,
    pub weights: BTreeMap<Symbol, f64>,
    pub total_budget: f64,
    pub min_weight: Option<f64>,
    pub max_weight: Option<f64>,
    pub constraint_source: Option<String>,
    pub created_at_ms: u64,
}

impl Allocation {
    pub fn apply_to_targets(&self, targets: &[PortfolioTarget]) -> BTreeMap<Symbol, f64> {
        let mut allocated: BTreeMap<Symbol, f64> = BTreeMap::new();
        for target in targets {
            for tw in &target.target_weights {
                let weight = self
                    .weights
                    .get(&tw.symbol)
                    .copied()
                    .unwrap_or(tw.target_weight);
                let amount = self.total_budget * weight;
                let clamped = match (self.min_weight, self.max_weight) {
                    (Some(min), Some(max)) => amount
                        .max(min * self.total_budget)
                        .min(max * self.total_budget),
                    (Some(min), None) => amount.max(min * self.total_budget),
                    (None, Some(max)) => amount.min(max * self.total_budget),
                    (None, None) => amount,
                };
                allocated.insert(tw.symbol.clone(), clamped);
            }
        }
        allocated
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderStatus {
    Created,
    Submitted,
    Accepted,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
}

impl OrderStatus {
    pub fn can_transition_to(&self, next: &OrderStatus) -> bool {
        if matches!(
            self,
            Self::Filled | Self::Cancelled | Self::Rejected | Self::Expired
        ) {
            return false;
        }
        matches!(
            (self, next),
            (Self::Created, Self::Submitted)
                | (Self::Created, Self::Cancelled)
                | (Self::Created, Self::Rejected)
                | (Self::Submitted, Self::Accepted)
                | (Self::Submitted, Self::Rejected)
                | (Self::Accepted, Self::PartiallyFilled)
                | (Self::Accepted, Self::Cancelled)
                | (Self::Accepted, Self::Rejected)
                | (Self::Accepted, Self::Expired)
                | (Self::PartiallyFilled, Self::PartiallyFilled)
                | (Self::PartiallyFilled, Self::Filled)
                | (Self::PartiallyFilled, Self::Cancelled)
                | (Self::PartiallyFilled, Self::Rejected)
                | (Self::PartiallyFilled, Self::Expired)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Order {
    pub order_id: String,
    pub client_order_id: Option<String>,
    pub exchange: Exchange,
    pub instrument: Symbol,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Option<f64>,
    pub quantity: f64,
    pub executed_qty: f64,
    pub time_in_force: TimeInForce,
    pub status: OrderStatus,
    pub source_intent_id: Option<String>,
    pub source_agent_id: Option<String>,
    pub venue_order_id: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

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
