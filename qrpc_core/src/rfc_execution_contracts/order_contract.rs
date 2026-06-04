use serde::{Deserialize, Serialize};

use crate::{Exchange, OrderSide, OrderType, Symbol, TimeInForce};

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
