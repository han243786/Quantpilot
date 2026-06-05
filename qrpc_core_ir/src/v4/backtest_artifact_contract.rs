use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    default_v4_backtest_artifact_version, CapabilitySupportSource, ExecutionCapabilityKind,
    MachineTemplateKind, RuntimeTradingMode,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4BacktestArtifact {
    #[serde(default = "default_v4_backtest_artifact_version")]
    pub schema_version: String,
    pub graph_id: String,
    pub started_at_ms: u64,
    pub ended_at_ms: u64,
    pub replay_mode: String,
    pub input_bar_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tick_count: Option<usize>,
    #[serde(default)]
    pub symbols: Vec<String>,
    #[serde(default)]
    pub machine_trajectory: Vec<V4BacktestMachineTrajectoryPoint>,
    #[serde(default)]
    pub risk_plane_decisions: Vec<V4BacktestRiskPlaneDecisionRecord>,
    #[serde(default)]
    pub execution_capability_sources: Vec<V4BacktestExecutionCapabilitySourceRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub microstructure_metrics: Option<V4BacktestMicrostructureMetrics>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub final_snapshot: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct V4BacktestMicrostructureMetrics {
    pub submitted_order_count: u64,
    pub filled_order_count: u64,
    pub fill_rate: f64,
    pub average_slippage_bps: f64,
    pub queue_position_estimate: f64,
    pub vwap_deviation_bps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4BacktestMachineTrajectoryPoint {
    pub ts_ms: u64,
    pub event_sequence: u64,
    pub machine_id: String,
    pub template: MachineTemplateKind,
    pub state_id: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4BacktestRiskPlaneDecisionRecord {
    pub decision_id: String,
    pub target_machine_id: String,
    pub source_machine_id: String,
    pub event_type: String,
    pub approved: bool,
    pub reason: String,
    pub ts_ms: u64,
    pub sequence: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4BacktestExecutionCapabilitySourceRecord {
    pub decision_id: String,
    pub target_machine_id: String,
    pub venue_id: String,
    pub runtime_mode: RuntimeTradingMode,
    pub accepted: bool,
    pub reason: String,
    pub capability: ExecutionCapabilityKind,
    pub source: CapabilitySupportSource,
    pub status: String,
    pub ts_ms: u64,
    pub sequence: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}
