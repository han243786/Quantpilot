use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{
    DecisionStatus, Exchange, IntentKind, OrderSide, RiskDecisionMode, RiskReasonCode, SignalSide,
    Symbol,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentSignal {
    pub signal_id: String,
    pub intent_id: String,
    pub kind: IntentKind,
    pub exchange_scope: Vec<Exchange>,
    pub symbol_scope: Vec<Symbol>,
    pub side: SignalSide,
    pub strength: f64,
    pub confidence: f64,
    pub reference_price: Option<f64>,
    pub derived_metrics: BTreeMap<String, f64>,
    pub reason: String,
    pub triggered_at_ms: u64,
    pub ttl_ms: u64,
    pub trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedAction {
    /// Fraction of current portfolio equity to allocate as trade notional.
    pub exchange: Exchange,
    pub side: OrderSide,
    pub quantity_ratio: f64,
    pub reference_price: f64,
    pub strategy_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TargetWeight {
    pub exchange: Exchange,
    pub symbol: Symbol,
    /// Target portfolio weight for this basket member, expressed as a 0..1 ratio of equity.
    pub target_weight: f64,
    /// Observed current weight at decision time, expressed as a 0..1 ratio of equity.
    pub current_weight: f64,
    pub reference_price: f64,
    #[serde(default)]
    pub signal_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortfolioTarget {
    pub allocation_kind: String,
    #[serde(default)]
    pub target_weights: Vec<TargetWeight>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortfolioTargetDecision {
    pub target_id: String,
    pub target: PortfolioTarget,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDecision {
    pub decision_id: String,
    pub agent_id: String,
    pub symbol: Symbol,
    pub exchange_targets: Vec<Exchange>,
    pub net_side: SignalSide,
    pub net_strength: f64,
    #[serde(default)]
    pub portfolio_target_decision: Option<PortfolioTargetDecision>,
    pub proposed_actions: Vec<ProposedAction>,
    pub reason: String,
    pub produced_at_ms: u64,
    pub trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDecision {
    pub risk_decision_id: String,
    pub risk_id: String,
    pub agent_decision_id: String,
    pub symbol: Symbol,
    pub status: DecisionStatus,
    #[serde(default)]
    pub mode: RiskDecisionMode,
    #[serde(default)]
    pub adjusted_portfolio_target_decision: Option<PortfolioTargetDecision>,
    pub adjusted_actions: Vec<ProposedAction>,
    pub reason_codes: Vec<RiskReasonCode>,
    pub reason_text: String,
    pub produced_at_ms: u64,
    pub trace_id: String,
}
