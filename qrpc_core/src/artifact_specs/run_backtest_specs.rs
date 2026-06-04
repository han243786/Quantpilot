use serde::{Deserialize, Serialize};

use super::ArtifactDigest;
use crate::{
    DataKind, DataSourceConfig, Exchange, MarketType, RuntimeProtocolCoreConfig, Symbol,
    TimeInForce, BACKTEST_SPEC_V1_VERSION, RUN_SPEC_V1_VERSION,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunModeSpec {
    Paper,
    Backtest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BacktestReplaySource {
    HistoricalReplay,
    DeterministicMock,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatasetSpec {
    pub dataset_id: String,
    pub data_id: String,
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub market_type: MarketType,
    pub kind: DataKind,
    pub interval: Option<String>,
    pub lookback_days: Option<u32>,
    pub enabled: bool,
}

impl From<&DataSourceConfig> for DatasetSpec {
    fn from(value: &DataSourceConfig) -> Self {
        Self {
            dataset_id: value.data_id.clone(),
            data_id: value.data_id.clone(),
            exchange: value.exchange.clone(),
            symbol: value.symbol.clone(),
            market_type: value.market_type.clone(),
            kind: value.kind.clone(),
            interval: value.interval.clone(),
            lookback_days: value.days,
            enabled: value.enabled,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionAssumptionSpec {
    pub initial_cash_balance: f64,
    pub taker_fee_bps: f64,
    pub default_slippage_bps: f64,
    pub total_cost_buffer_bps: f64,
    pub time_in_force: TimeInForce,
    pub allow_partial_fills: bool,
    pub latency_assumption_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionAssumptionValueSource {
    BackendFallback,
    ProfileDefault,
    RequestOverride,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionAssumptionSourceSummary {
    pub fee_bps: ExecutionAssumptionValueSource,
    pub slippage_bps: ExecutionAssumptionValueSource,
    pub latency_ms: ExecutionAssumptionValueSource,
}

impl From<&RuntimeProtocolCoreConfig> for ExecutionAssumptionSpec {
    fn from(value: &RuntimeProtocolCoreConfig) -> Self {
        Self {
            initial_cash_balance: value.initial_cash_balance,
            taker_fee_bps: value.taker_fee_bps,
            default_slippage_bps: value.default_slippage_bps,
            total_cost_buffer_bps: value.total_cost_buffer_bps,
            time_in_force: TimeInForce::Gtc,
            allow_partial_fills: true,
            latency_assumption_ms: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarketDataSnapshotSpec {
    pub snapshot_id: String,
    pub replay_source: BacktestReplaySource,
    pub captured_at_ms: u64,
    #[serde(default)]
    pub datasets: Vec<DatasetSpec>,
}

impl MarketDataSnapshotSpec {
    pub fn from_runtime_protocol(
        snapshot_id: impl Into<String>,
        replay_source: BacktestReplaySource,
        captured_at_ms: u64,
        config: &RuntimeProtocolCoreConfig,
    ) -> Self {
        Self {
            snapshot_id: snapshot_id.into(),
            replay_source,
            captured_at_ms,
            datasets: config.data_sources.iter().map(DatasetSpec::from).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunSpec {
    pub schema_version: String,
    pub run_mode: RunModeSpec,
    pub graph_id: String,
    pub compile_id: String,
    pub runtime_mode: String,
    pub protocol_name: String,
    pub config_hash: String,
    #[serde(default)]
    pub datasets: Vec<DatasetSpec>,
    pub execution_assumptions: ExecutionAssumptionSpec,
    #[serde(default)]
    pub execution_assumption_sources: Option<ExecutionAssumptionSourceSummary>,
    pub core_ir_digest: ArtifactDigest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunSpecRuntimeProtocolInput {
    pub graph_id: String,
    pub compile_id: String,
    pub run_mode: RunModeSpec,
    pub runtime_mode: String,
    pub protocol_name: String,
    pub config_hash: String,
    pub core_ir_digest: ArtifactDigest,
}

impl RunSpec {
    pub fn from_runtime_protocol(
        input: RunSpecRuntimeProtocolInput,
        config: &RuntimeProtocolCoreConfig,
    ) -> Self {
        Self {
            schema_version: RUN_SPEC_V1_VERSION.to_string(),
            run_mode: input.run_mode,
            graph_id: input.graph_id,
            compile_id: input.compile_id,
            runtime_mode: input.runtime_mode,
            protocol_name: input.protocol_name,
            config_hash: input.config_hash,
            datasets: config.data_sources.iter().map(DatasetSpec::from).collect(),
            execution_assumptions: ExecutionAssumptionSpec::from(config),
            execution_assumption_sources: None,
            core_ir_digest: input.core_ir_digest,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BacktestSpec {
    pub schema_version: String,
    pub backtest_id: String,
    pub replay_source: BacktestReplaySource,
    pub requested_at_ms: u64,
    pub run_spec: RunSpec,
    pub market_data_snapshot: MarketDataSnapshotSpec,
}

impl BacktestSpec {
    pub fn new(
        backtest_id: impl Into<String>,
        replay_source: BacktestReplaySource,
        requested_at_ms: u64,
        run_spec: RunSpec,
        market_data_snapshot: MarketDataSnapshotSpec,
    ) -> Self {
        Self {
            schema_version: BACKTEST_SPEC_V1_VERSION.to_string(),
            backtest_id: backtest_id.into(),
            replay_source,
            requested_at_ms,
            run_spec,
            market_data_snapshot,
        }
    }
}
