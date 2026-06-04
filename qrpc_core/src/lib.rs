mod artifact_specs;
pub mod error;
mod plugin;
mod protocol_primitives;
mod runtime_io_contract;
mod runtime_protocol_config;
mod strategy_ir;

pub use artifact_specs::*;
pub use plugin::*;
pub use protocol_primitives::*;
pub use qrpc_core_ir::{CoreStrategyIr, CORE_IR_V1_VERSION};
pub use runtime_io_contract::*;
pub use runtime_protocol_config::*;
pub use strategy_ir::*;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ── v1.0.0 RFC-001 数据请求协议 ──────────────────────

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

// ── v1.0.0 RFC-010 分配协议 ──────────────────────────

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
    /// v1.0.0: 对一组目标仓位按权重分配资金
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

// ── v1.0.0 RFC-012 订单协议 ──────────────────────────

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
    /// v1.0.1: 校验状态转移合法性。终态不可再转移。
    pub fn can_transition_to(&self, next: &OrderStatus) -> bool {
        // 终态不可转移
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

// ── v1.0.0 RFC-013 执行反馈协议 ──────────────────────

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

// ── v1.0.0 热接管协议 (RFC 暂未独立编号, Phase 3) ──

/// 策略 A → Sandbox → 策略 B 的状态快照
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
    /// 校验快照完整性 — 未结订单必须携带 order_id
    pub fn validate_completeness(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.snapshot_id.is_empty() {
            errors.push("snapshot_id 不能为空".to_string());
        }
        if self.source_strategy_id.is_empty() {
            errors.push("source_strategy_id 不能为空".to_string());
        }
        for order in &self.open_orders {
            if order.order_id.is_empty() {
                errors.push(format!(
                    "未结订单缺少 order_id (symbol={:?})",
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_runtime_protocol() -> RuntimeProtocolCoreConfig {
        RuntimeProtocolCoreConfig {
            data_sources: vec![DataSourceConfig {
                data_id: "binance_btc_1d".into(),
                exchange: Exchange::Binance,
                symbol: Symbol::BtcUsdt,
                market_type: MarketType::Spot,
                kind: DataKind::KlineSeries,
                days: Some(200),
                interval: Some("1d".into()),
                ping_enabled: false,
                request_interval_ms: None,
                enabled: true,
            }],
            intents: vec![IntentConfig {
                intent_id: "intent_rsi".into(),
                name: "RSI".into(),
                kind: IntentKind::Rsi,
                input_data_ids: vec!["binance_btc_1d".into()],
                params: BTreeMap::new(),
                enabled: true,
            }],
            agents: vec![AgentConfig {
                agent_id: "agent_main".into(),
                name: "Main Agent".into(),
                input_intent_ids: vec!["intent_rsi".into()],
                rebalance_symbols: vec![],
                rebalance_schedule: None,
                rebalance_allocation_kind: None,
                rebalance_rank_method: None,
                rebalance_score_normalize: None,
                rebalance_target_weights: vec![],
                params: BTreeMap::new(),
                enabled: true,
            }],
            risks: vec![RiskConfig {
                risk_id: "risk_main".into(),
                name: "Main Risk".into(),
                observed_agent_ids: vec!["agent_main".into()],
                max_position_ratio: 0.2,
                max_single_weight: None,
                max_concentration_ratio: None,
                max_symbol_net_exposure_ratio: None,
                max_portfolio_net_exposure_ratio: None,
                max_turnover: None,
                min_trade_weight: None,
                max_new_positions_per_rebalance: None,
                max_total_leverage: 3.0,
                max_exchange_leverage: 3.0,
                min_action_interval_ms: 100,
                enabled: true,
            }],
            initial_cash_balance: 100_000.0,
            taker_fee_bps: 10.0,
            default_slippage_bps: 5.0,
            total_cost_buffer_bps: 20.0,
        }
    }

    #[test]
    fn canonical_digest_is_stable_for_equivalent_payloads() {
        let left = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_test",
            "mode": "paper"
        });
        let right = serde_json::json!({
            "compile_id": "compile_test",
            "mode": "paper",
            "graph_id": "graph_test"
        });

        let left_digest = canonical_json_sha256_digest(&left).unwrap();
        let right_digest = canonical_json_sha256_digest(&right).unwrap();

        assert_eq!(left_digest, right_digest);
        assert_eq!(
            left_digest.algorithm,
            ArtifactDigestAlgorithm::Sha256CanonicalJson
        );
    }

    #[test]
    fn run_and_backtest_specs_capture_protocol_boundary() {
        let config = sample_runtime_protocol();
        let core_ir_digest = canonical_json_sha256_digest(&serde_json::json!({
            "ir_version": "quantpilot/core-ir/v1"
        }))
        .unwrap();

        let run_spec = RunSpec::from_runtime_protocol(
            RunSpecRuntimeProtocolInput {
                graph_id: "graph_test".to_string(),
                compile_id: "compile_test".to_string(),
                run_mode: RunModeSpec::Backtest,
                runtime_mode: "paper".to_string(),
                protocol_name: "quantpilot/minimal-sim/v1".to_string(),
                config_hash: "runtime-spec-hash".to_string(),
                core_ir_digest: core_ir_digest.clone(),
            },
            &config,
        );
        let snapshot = MarketDataSnapshotSpec::from_runtime_protocol(
            "snapshot_test",
            BacktestReplaySource::DeterministicMock,
            1_700_000_000_000,
            &config,
        );
        let backtest_spec = BacktestSpec::new(
            "backtest_test",
            BacktestReplaySource::DeterministicMock,
            1_700_000_000_000,
            run_spec.clone(),
            snapshot.clone(),
        );

        assert_eq!(run_spec.schema_version, RUN_SPEC_V1_VERSION);
        assert_eq!(run_spec.datasets.len(), 1);
        assert_eq!(
            run_spec.execution_assumptions.time_in_force,
            TimeInForce::Gtc
        );
        assert_eq!(snapshot.datasets[0].data_id, "binance_btc_1d");
        assert_eq!(backtest_spec.schema_version, BACKTEST_SPEC_V1_VERSION);
        assert_eq!(backtest_spec.run_spec.core_ir_digest, core_ir_digest);
        assert_eq!(backtest_spec.market_data_snapshot, snapshot);
    }
}
