mod market_data_io;

pub use market_data_io::*;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::{
    DecisionStatus, Exchange, ExecutionStatus, IntentKind, OrderSide, OrderType, RiskDecisionMode,
    RiskReasonCode, RuntimeEventType, SignalSide, Symbol, TimeInForce,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub net_qty: f64,
    pub frozen_qty: f64,
    pub avg_entry_price: f64,
    pub mark_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeExposure {
    pub exchange: Exchange,
    pub gross_notional: f64,
    pub net_notional: f64,
    pub leverage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortfolioState {
    pub cash_balance: f64,
    pub available_cash_balance: f64,
    pub frozen_cash_balance: f64,
    pub open_orders: Vec<OpenOrder>,
    pub positions: Vec<Position>,
    pub exchange_exposures: Vec<ExchangeExposure>,
    pub total_gross_notional: f64,
    pub total_net_notional: f64,
    pub total_leverage: f64,
    pub updated_at_ms: u64,
}

impl PortfolioState {
    pub fn new(initial_cash_balance: f64, ts_ms: u64) -> Self {
        Self {
            cash_balance: initial_cash_balance,
            available_cash_balance: initial_cash_balance,
            frozen_cash_balance: 0.0,
            open_orders: Vec::new(),
            positions: Vec::new(),
            exchange_exposures: Vec::new(),
            total_gross_notional: 0.0,
            total_net_notional: 0.0,
            total_leverage: 0.0,
            updated_at_ms: ts_ms,
        }
    }

    /// v2.5.0: debug 模式下验证跨字段一致性
    pub fn debug_assert_invariants(&self) {
        if cfg!(debug_assertions) {
            // 可用现金 + 冻结现金 = 总现金
            debug_assert!(
                (self.available_cash_balance + self.frozen_cash_balance - self.cash_balance).abs()
                    < 0.01,
                "PortfolioState: available({}) + frozen({}) != cash({})",
                self.available_cash_balance,
                self.frozen_cash_balance,
                self.cash_balance
            );
            // 杠杆为非负
            debug_assert!(self.total_leverage >= 0.0, "total_leverage 不能为负");
            // 余额非负
            debug_assert!(self.cash_balance >= 0.0, "cash_balance 不能为负");
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEvent {
    pub event_id: String,
    pub event_type: RuntimeEventType,
    pub trace_id: String,
    pub source_id: String,
    pub ts_ms: u64,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeCycleOutput {
    pub cycle_name: String,
    pub trace_id: String,
    pub normalized_data: Vec<NormalizedMarketData>,
    pub intent_signals: Vec<IntentSignal>,
    pub agent_decisions: Vec<AgentDecision>,
    pub risk_decisions: Vec<RiskDecision>,
    pub execution_plans: Vec<ExecutionPlan>,
    pub fill_reports: Vec<FillReport>,
    pub portfolio_state: PortfolioState,
    pub runtime_events: Vec<RuntimeEvent>,
    pub data_fetch_counts: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionOutput {
    pub slow_cycle: RuntimeCycleOutput,
    pub fast_cycle: RuntimeCycleOutput,
    pub final_portfolio: PortfolioState,
    pub data_fetch_counts: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestEquityPoint {
    pub ts_ms: u64,
    pub equity: f64,
    pub cash_balance: f64,
    pub net_notional: f64,
}

/// v1.1.0: 风险调整收益指标组
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BacktestRiskAdjusted {
    #[serde(default)]
    pub sharpe_ratio: f64,
    #[serde(default)]
    pub sortino_ratio: f64,
    #[serde(default)]
    pub calmar_ratio: f64,
    /// v1.1.0 P2: 95% VaR (日收益率, 历史模拟法)
    #[serde(default)]
    pub var_95: f64,
    /// v1.1.0 P2: 95% CVaR / Expected Shortfall
    #[serde(default)]
    pub cvar_95: f64,
}

/// v1.1.0: 交易分析指标组
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BacktestTradeAnalysis {
    #[serde(default)]
    pub profit_factor: f64,
    #[serde(default)]
    pub avg_win: f64,
    #[serde(default)]
    pub avg_loss: f64,
    #[serde(default)]
    pub max_consecutive_wins: u32,
    #[serde(default)]
    pub max_consecutive_losses: u32,
}

/// v1.1.0: 回撤分析指标组
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BacktestDrawdownAnalysis {
    #[serde(default)]
    pub max_drawdown_ratio: f64,
    #[serde(default)]
    pub max_drawdown_duration_days: f64,
    #[serde(default)]
    pub avg_drawdown_duration_days: f64,
}

/// v1.1.0: 基准比较指标组 — None 表示基准未启用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestBenchmarkComparison {
    #[serde(default)]
    pub benchmark_total_return: f64,
    #[serde(default)]
    pub alpha: f64,
    #[serde(default)]
    pub beta: f64,
    #[serde(default)]
    pub information_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestSummary {
    pub step_count: usize,
    pub trade_count: usize,
    pub total_return_ratio: f64,
    pub final_equity: f64,
    #[serde(default)]
    pub net_profit: f64,
    #[serde(default)]
    pub win_rate: f64,
    #[serde(default)]
    pub annualized_return: f64,
    #[serde(default)]
    pub annualized_volatility: f64,
    #[serde(default)]
    pub risk_adjusted: BacktestRiskAdjusted,
    #[serde(default)]
    pub trade_analysis: BacktestTradeAnalysis,
    #[serde(default)]
    pub drawdown_analysis: BacktestDrawdownAnalysis,
    #[serde(default)]
    pub benchmark_comparison: Option<BacktestBenchmarkComparison>,
    /// v1.1.0 P2: 日收益率偏度 (正偏 = 多小亏少大赚)
    #[serde(default)]
    pub skewness: f64,
    /// v1.1.0 P2: 日收益率峰度 (高值 = 肥尾风险)
    #[serde(default)]
    pub kurtosis: f64,
}

/// v1.1.0 P2: 月度/季度收益率分解
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodReturn {
    pub period: String, // e.g. "2025-01", "2025-Q1", "2025"
    pub return_ratio: f64,
    pub trade_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestOutput {
    pub mode: String,
    pub started_at_ms: u64,
    pub ended_at_ms: u64,
    /// v1.3.4: 回测实际墙钟耗时（毫秒）
    #[serde(default)]
    pub elapsed_ms: Option<u64>,
    pub sessions: Vec<SessionOutput>,
    pub equity_curve: Vec<BacktestEquityPoint>,
    /// v1.1.0: 买入持有基准权益曲线（等权重持有全部交易标的）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub benchmark_equity_curve: Vec<BacktestEquityPoint>,
    /// v1.1.0 P2: 月度/季度/年度收益率分解
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub period_returns: Vec<PeriodReturn>,
    pub summary: BacktestSummary,
    pub final_portfolio: PortfolioState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v4_artifact: Option<qrpc_core_ir::v4::V4BacktestArtifact>,
    #[serde(default)]
    pub debug_values: Option<Vec<std::collections::BTreeMap<String, f64>>>,
}
