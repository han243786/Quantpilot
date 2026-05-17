use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const CORE_IR_V1_VERSION: &str = "quantpilot/core-ir/v1";
pub const CUSTOM_EXPR_V1_VERSION: &str = "quantpilot/custom-expr/v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoreStrategyIr {
    pub ir_version: String,
    pub metadata: CoreMetadata,
    #[serde(default)]
    pub data_bindings: Vec<DataBinding>,
    #[serde(default)]
    pub indicators: Vec<IndicatorNode>,
    #[serde(default)]
    pub signal_rules: Vec<SignalRule>,
    #[serde(default)]
    pub agent_policies: Vec<AgentPolicy>,
    #[serde(default)]
    pub risk_policies: Vec<RiskPolicy>,
    pub execution: ExecutionRule,
    /// v1.0.0 DAG 边: 显式声明 data→indicator→signal→agent→risk→exec 的连接
    /// 为空时退化为线性 pipeline (向后兼容)
    #[serde(default)]
    pub edges: Vec<CoreIREdge>,
}

/// v1.0.0 DAG 边 — 连接两个节点的有向边
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CoreIREdge {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub port: Option<String>,
}

impl CoreStrategyIr {
    pub fn new(metadata: CoreMetadata, execution: ExecutionRule) -> Self {
        Self {
            ir_version: CORE_IR_V1_VERSION.to_string(),
            metadata,
            data_bindings: Vec::new(),
            indicators: Vec::new(),
            signal_rules: Vec::new(),
            agent_policies: Vec::new(),
            risk_policies: Vec::new(),
            execution,
            edges: Vec::new(),
        }
    }

    /// v1.0.0 DAG 环检测: DFS 拓扑验证，有环返回环路径
    pub fn validate_dag(&self) -> Result<(), Vec<String>> {
        if self.edges.is_empty() {
            return Ok(()); // 无显式边 = 线性 pipeline, 无 DAG 约束
        }

        use std::collections::{BTreeMap, BTreeSet};

        let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for edge in &self.edges {
            adjacency
                .entry(&edge.source)
                .or_default()
                .push(&edge.target);
        }

        // DFS 环检测
        let mut visited = BTreeSet::new();
        let mut in_stack = BTreeSet::new();
        let mut cycle_path: Vec<String> = Vec::new();

        fn dfs<'a>(
            node: &'a str,
            adjacency: &BTreeMap<&'a str, Vec<&'a str>>,
            visited: &mut BTreeSet<&'a str>,
            in_stack: &mut BTreeSet<&'a str>,
            cycle_path: &mut Vec<String>,
        ) -> bool {
            visited.insert(node);
            in_stack.insert(node);
            if let Some(neighbors) = adjacency.get(node) {
                for &next in neighbors {
                    if !visited.contains(next) {
                        if dfs(next, adjacency, visited, in_stack, cycle_path) {
                            cycle_path.push(node.to_string());
                            return true;
                        }
                    } else if in_stack.contains(next) {
                        cycle_path.push(next.to_string());
                        cycle_path.push(node.to_string());
                        return true;
                    }
                }
            }
            in_stack.remove(node);
            false
        }

        for edge in &self.edges {
            if !visited.contains(edge.source.as_str())
                && dfs(
                    edge.source.as_str(),
                    &adjacency,
                    &mut visited,
                    &mut in_stack,
                    &mut cycle_path,
                )
            {
                cycle_path.reverse();
                return Err(vec![format!("DAG 环检测失败: {}", cycle_path.join(" → "))]);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CoreMetadata {
    pub strategy_id: String,
    pub name: String,
    pub source_kind: CoreSourceKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoreSourceKind {
    StrategyIr,
    FormalQuantScript,
    RuntimeProtocol,
    FrontendGraph,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataBinding {
    pub data_id: String,
    pub kind: DataBindingKind,
    #[serde(default)]
    pub source_hints: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataBindingKind {
    KlineSeries,
    Quote,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IndicatorNode {
    pub indicator_id: String,
    pub kind: CoreIndicatorKind,
    #[serde(default)]
    pub inputs: Vec<SeriesExpr>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spread_spec: Option<SpreadSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_expr: Option<CustomExprSpec>,
    #[serde(default)]
    pub params: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CoreIndicatorKind {
    MaCross,
    Rsi,
    Macd,
    Momentum,
    Spread,
    ZScore,
    Custom,
    QuoteObserve,
    Atr,
    BollingerBands,
    Obv,
    Cmf,
    Adx,
    Stochastic,
    Cci,
    ParabolicSar,
    KeltnerChannel,
    DonchianChannel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomExprSpec {
    pub schema_version: String,
    pub signal_kind: SignalKind,
    pub predicate: CustomPredicateExpr,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strength: Option<CustomValueExpr>,
    #[serde(default = "default_custom_confidence")]
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomPredicateExpr {
    pub left: CustomValueExpr,
    pub op: ComparisonOp,
    pub right: CustomValueExpr,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CustomValueExpr {
    Number {
        value: f64,
    },
    Input {
        data_id: String,
        field: SeriesField,
    },
    WindowAgg {
        data_id: String,
        field: SeriesField,
        window_size: usize,
        agg: SeriesAggregation,
    },
    Binary {
        left: Box<CustomValueExpr>,
        op: ArithmeticOp,
        right: Box<CustomValueExpr>,
    },
    Unary {
        op: ArithmeticUnaryOp,
        value: Box<CustomValueExpr>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArithmeticOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArithmeticUnaryOp {
    Abs,
    Negate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SeriesExpr {
    DataRef {
        data_id: String,
    },
    DataField {
        data_id: String,
        field: SeriesField,
    },
    Resample {
        input: Box<SeriesExpr>,
        period_ms: u64,
        agg: SeriesAggregation,
    },
    WindowAgg {
        input: Box<SeriesExpr>,
        window_size: usize,
        agg: SeriesAggregation,
    },
    IndicatorRef {
        indicator_id: String,
    },
    RawText {
        source: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SeriesField {
    MidOrClose,
    BidOrClose,
    AskOrClose,
    Close,
    Open,
    High,
    Low,
    Volume,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SeriesAggregation {
    Last,
    Mean,
    Sum,
    Min,
    Max,
    StdDev,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlignDirection {
    Backward,
    Forward,
    Nearest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlignAsofSpec {
    pub direction: AlignDirection,
    pub tolerance_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpreadValueKind {
    Ratio,
    Bps,
    Absolute,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpreadSpec {
    pub left: SeriesExpr,
    pub right: SeriesExpr,
    pub align: AlignAsofSpec,
    pub output: SpreadValueKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignalRule {
    pub signal_id: String,
    pub indicator_id: String,
    pub signal_kind: SignalKind,
    pub condition: ScalarExpr,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SignalKind {
    Long,
    Short,
    Observe,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ScalarExpr {
    Number {
        value: f64,
    },
    Bool {
        value: bool,
    },
    Series {
        expr: SeriesExpr,
    },
    Ref {
        name: String,
    },
    Compare {
        left: Box<ScalarExpr>,
        op: ComparisonOp,
        right: Box<ScalarExpr>,
    },
    RawText {
        source: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonOp {
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
}

pub fn close_series_expr(data_id: impl Into<String>) -> SeriesExpr {
    SeriesExpr::DataField {
        data_id: data_id.into(),
        field: SeriesField::Close,
    }
}

pub fn moving_average_series_expr(data_id: impl Into<String>, period: usize) -> Option<SeriesExpr> {
    if period == 0 {
        return None;
    }

    Some(SeriesExpr::WindowAgg {
        input: Box::new(close_series_expr(data_id)),
        window_size: period,
        agg: SeriesAggregation::Mean,
    })
}

pub fn moving_average_compare_expr(
    data_id: impl Into<String>,
    left_period: usize,
    op: ComparisonOp,
    right_period: usize,
) -> Option<ScalarExpr> {
    let data_id = data_id.into();
    let left = moving_average_series_expr(data_id.clone(), left_period)?;
    let right = moving_average_series_expr(data_id, right_period)?;
    Some(ScalarExpr::Compare {
        left: Box::new(ScalarExpr::Series { expr: left }),
        op,
        right: Box::new(ScalarExpr::Series { expr: right }),
    })
}

pub fn indicator_threshold_compare_expr(
    indicator_id: impl Into<String>,
    op: ComparisonOp,
    threshold: f64,
) -> Option<ScalarExpr> {
    if !threshold.is_finite() {
        return None;
    }

    Some(ScalarExpr::Compare {
        left: Box::new(ScalarExpr::Ref {
            name: indicator_id.into(),
        }),
        op,
        right: Box::new(ScalarExpr::Number { value: threshold }),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentPolicy {
    pub agent_id: String,
    pub name: String,
    pub kind: AgentPolicyKind,
    #[serde(default)]
    pub input_signal_ids: Vec<String>,
    #[serde(default)]
    pub rebalance_symbols: Vec<String>,
    #[serde(default)]
    pub rebalance_schedule: Option<RebalanceSchedule>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rebalance_allocation_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rebalance_rank_method: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rebalance_score_normalize: Option<String>,
    #[serde(default)]
    pub rebalance_target_weights: Vec<f64>,
    #[serde(default)]
    pub decision_threshold: Option<f64>,
    pub max_quantity_ratio: f64,
    #[serde(default)]
    pub spread_trigger_bps: Option<f64>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentPolicyKind {
    WeightedSignals,
    CrossVenueArbitrage,
    PortfolioRebalance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RebalanceSchedule {
    EverySlow,
    Every1d,
    Weekly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiskPolicy {
    pub policy_id: String,
    pub name: String,
    #[serde(default)]
    pub observed_agent_ids: Vec<String>,
    pub max_position_ratio: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_single_weight: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_concentration_ratio: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_symbol_net_exposure_ratio: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_portfolio_net_exposure_ratio: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_turnover: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_trade_weight: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_new_positions_per_rebalance: Option<u32>,
    pub max_total_leverage: f64,
    pub max_exchange_leverage: f64,
    pub min_action_interval_ms: u64,
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// v1.1.0: 所有标的合计杠杆上限（跨标的联合约束 Phase 2）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_cross_symbol_leverage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionSizingKind {
    EquityNotionalRatio,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoreTimeInForce {
    Gtc,
    Ioc,
    Fok,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionRule {
    pub execution_id: String,
    pub venue_kind: String,
    #[serde(default = "default_execution_sizing_kind")]
    pub sizing_kind: ExecutionSizingKind,
    #[serde(default)]
    pub slippage_bps: f64,
    #[serde(default)]
    pub taker_fee_bps: f64,
    #[serde(default)]
    pub total_cost_buffer_bps: f64,
    #[serde(default = "default_time_in_force")]
    pub time_in_force: CoreTimeInForce,
    #[serde(default)]
    pub params: BTreeMap<String, Value>,
}

fn default_true() -> bool {
    true
}

fn default_execution_sizing_kind() -> ExecutionSizingKind {
    ExecutionSizingKind::EquityNotionalRatio
}

fn default_time_in_force() -> CoreTimeInForce {
    CoreTimeInForce::Gtc
}

fn default_custom_confidence() -> f64 {
    0.8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_ir_round_trips() {
        let mut core_ir = CoreStrategyIr::new(
            CoreMetadata {
                strategy_id: "sample".into(),
                name: "Sample".into(),
                source_kind: CoreSourceKind::StrategyIr,
            },
            ExecutionRule {
                execution_id: "exec".into(),
                venue_kind: "paper".into(),
                sizing_kind: ExecutionSizingKind::EquityNotionalRatio,
                slippage_bps: 5.0,
                taker_fee_bps: 10.0,
                total_cost_buffer_bps: 20.0,
                time_in_force: CoreTimeInForce::Gtc,
                params: BTreeMap::new(),
            },
        );
        core_ir.data_bindings.push(DataBinding {
            data_id: "btc_1d".into(),
            kind: DataBindingKind::KlineSeries,
            source_hints: BTreeMap::new(),
        });
        core_ir.indicators.push(IndicatorNode {
            indicator_id: "rsi_1".into(),
            kind: CoreIndicatorKind::Rsi,
            inputs: vec![SeriesExpr::DataRef {
                data_id: "btc_1d".into(),
            }],
            spread_spec: None,
            custom_expr: None,
            params: BTreeMap::new(),
        });
        core_ir.signal_rules.push(SignalRule {
            signal_id: "rule_1".into(),
            indicator_id: "rsi_1".into(),
            signal_kind: SignalKind::Long,
            condition: moving_average_compare_expr("btc_1d", 20, ComparisonOp::Gt, 100).unwrap(),
        });
        core_ir.agent_policies.push(AgentPolicy {
            agent_id: "agent_1".into(),
            name: "Weighted Agent".into(),
            kind: AgentPolicyKind::WeightedSignals,
            input_signal_ids: vec!["rsi_1".into()],
            rebalance_symbols: vec![],
            rebalance_schedule: None,
            rebalance_allocation_kind: None,
            rebalance_rank_method: None,
            rebalance_score_normalize: None,
            rebalance_target_weights: vec![],
            decision_threshold: Some(0.05),
            max_quantity_ratio: 0.2,
            spread_trigger_bps: None,
            enabled: true,
        });
        core_ir.risk_policies.push(RiskPolicy {
            policy_id: "risk_1".into(),
            name: "Global Risk".into(),
            observed_agent_ids: vec!["agent_1".into()],
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
            min_action_interval_ms: 1000,
            enabled: true,
                        max_cross_symbol_leverage: None,
        });

        let encoded = serde_json::to_string(&core_ir).unwrap();
        let decoded: CoreStrategyIr = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded.ir_version, CORE_IR_V1_VERSION);
        assert_eq!(decoded, core_ir);
    }
}
