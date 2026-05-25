use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const STRATEGY_IR_V0_VERSION: &str = "strategy_ir/v0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum KnownOrUnknown<T> {
    Known(T),
    Unknown(String),
}

impl<T> KnownOrUnknown<T> {
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StrategyIr {
    pub ir_version: String,
    pub metadata: StrategyMetadata,
    pub signals: Vec<SignalDefinition>,
    pub logic: StrategyLogic,
    pub risk_rules: StrategyRiskRules,
    #[serde(default)]
    pub risk_profile: Option<StrategyRiskProfileRef>,
    pub data_requirements: Vec<DataRequirement>,
    pub execution: StrategyExecution,
    #[serde(default)]
    pub execution_profile: Option<StrategyExecutionProfileRef>,
    #[serde(default)]
    pub gap_annotations: Vec<GapAnnotation>,
    #[serde(default)]
    pub unknowns: Vec<StrategyUnknown>,
}

impl StrategyIr {
    pub fn validation_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.ir_version != STRATEGY_IR_V0_VERSION {
            errors.push(format!(
                "ir_version 必须是 {STRATEGY_IR_V0_VERSION}，但实际为 {}",
                self.ir_version
            ));
        }

        if self.metadata.strategy_id.trim().is_empty() {
            errors.push("metadata.strategy_id 是必需的".to_string());
        }
        if self.metadata.name.trim().is_empty() {
            errors.push("metadata.name 是必需的".to_string());
        }
        if self.metadata.summary.trim().is_empty() {
            errors.push("metadata.summary 是必需的".to_string());
        }

        if self.signals.is_empty() {
            errors.push("signals 必须包含至少一个信号".to_string());
        }
        if self.data_requirements.is_empty() {
            errors.push("data_requirements 必须包含至少一个数据要求".to_string());
        }
        if self.logic.entry_rules.is_empty() {
            errors.push("logic.entry_rules 必须包含至少一条规则".to_string());
        }

        validate_unique_ids(
            self.signals.iter().map(|item| item.signal_id.as_str()),
            "signals",
            &mut errors,
        );
        validate_unique_ids(
            self.data_requirements
                .iter()
                .map(|item| item.data_id.as_str()),
            "data_requirements",
            &mut errors,
        );
        validate_unique_ids(
            self.logic
                .entry_rules
                .iter()
                .chain(self.logic.exit_rules.iter())
                .map(|item| item.rule_id.as_str()),
            "logic rules",
            &mut errors,
        );

        for (index, signal) in self.signals.iter().enumerate() {
            if signal.signal_id.trim().is_empty() {
                errors.push(format!("signals[{index}].signal_id 是必需的"));
            }
            if signal.name.trim().is_empty() {
                errors.push(format!("signals[{index}].name 是必需的"));
            }
            if signal.indicator.inputs.is_empty() {
                errors.push(format!(
                    "signals[{index}].indicator.inputs 必须包含至少一个输入"
                ));
            }
            if matches!(signal.indicator.kind, IndicatorKind::Spread)
                && signal.indicator.inputs.len() < 2
            {
                errors.push(format!(
                    "signals[{index}].indicator.inputs 对于 spread 必须包含至少两个输入"
                ));
            }
            if !indicator_kind_supported(&signal.indicator.kind) {
                errors.push(format!(
                    "signals[{index}].indicator.kind {:?} 不被当前运行时支持",
                    signal.indicator.kind
                ));
            }
        }

        for (index, rule) in self.logic.entry_rules.iter().enumerate() {
            validate_logic_rule(rule, &format!("logic.entry_rules[{index}]"), &mut errors);
        }
        for (index, rule) in self.logic.exit_rules.iter().enumerate() {
            validate_logic_rule(rule, &format!("logic.exit_rules[{index}]"), &mut errors);
        }

        validate_unknownable(
            &self.logic.position_sizing.value,
            "logic.position_sizing.value",
            &mut errors,
        );
        if let Some(rule) = &self.logic.rebalance_rule {
            validate_unknownable(
                &rule.frequency,
                "logic.rebalance_rule.frequency",
                &mut errors,
            );
        }

        validate_unknownable(
            &self.risk_rules.max_position_ratio,
            "risk_rules.max_position_ratio",
            &mut errors,
        );
        validate_unknownable(
            &self.risk_rules.stop_loss_ratio,
            "risk_rules.stop_loss_ratio",
            &mut errors,
        );
        validate_unknownable_opt(
            self.risk_rules.take_profit_ratio.as_ref(),
            "risk_rules.take_profit_ratio",
            &mut errors,
        );
        validate_unknownable_opt(
            self.risk_rules.max_drawdown_ratio.as_ref(),
            "risk_rules.max_drawdown_ratio",
            &mut errors,
        );
        validate_unknownable_opt(
            self.risk_rules.max_trades_per_day.as_ref(),
            "risk_rules.max_trades_per_day",
            &mut errors,
        );
        if let Some(profile) = &self.risk_profile {
            if profile.profile_id.trim() != "global" {
                errors.push("risk_profile.profile_id 在当前运行时中必须为 \"global\"".to_string());
            }
            if let Some(value) = profile.max_position {
                if !value.is_finite() || value <= 0.0 {
                    errors.push("risk_profile.max_position 必须大于 0".to_string());
                }
            }
            if let Some(value) = profile.max_total_leverage {
                if value < 1.0 {
                    errors.push("risk_profile.max_total_leverage 必须大于等于 1".to_string());
                }
            }
            if let Some(value) = profile.max_exchange_leverage {
                if value < 1.0 {
                    errors.push("risk_profile.max_exchange_leverage 必须大于等于 1".to_string());
                }
            }
        }

        for (index, requirement) in self.data_requirements.iter().enumerate() {
            if requirement.data_id.trim().is_empty() {
                errors.push(format!("data_requirements[{index}].data_id 是必需的"));
            }
            if requirement.fields.is_empty() {
                errors.push(format!(
                    "data_requirements[{index}].fields 必须包含至少一个字段"
                ));
            }
            validate_unknownable(
                &requirement.venue,
                &format!("data_requirements[{index}].venue"),
                &mut errors,
            );
            validate_unknownable(
                &requirement.symbol,
                &format!("data_requirements[{index}].symbol"),
                &mut errors,
            );
            validate_unknownable(
                &requirement.granularity,
                &format!("data_requirements[{index}].granularity"),
                &mut errors,
            );
            validate_unknownable(
                &requirement.lookback,
                &format!("data_requirements[{index}].lookback"),
                &mut errors,
            );
        }

        validate_unknownable(
            &self.execution.venue_type,
            "execution.venue_type",
            &mut errors,
        );
        validate_unknownable(
            &self.execution.order_type,
            "execution.order_type",
            &mut errors,
        );
        validate_unknownable(
            &self.execution.slippage_model,
            "execution.slippage_model",
            &mut errors,
        );
        validate_unknownable_opt(
            self.execution.time_in_force.as_ref(),
            "execution.time_in_force",
            &mut errors,
        );
        validate_unknownable_opt(
            self.execution.latency_assumption_ms.as_ref(),
            "execution.latency_assumption_ms",
            &mut errors,
        );
        validate_unknownable_opt(
            self.execution.capital_base.as_ref(),
            "execution.capital_base",
            &mut errors,
        );
        if let Some(profile) = &self.execution_profile {
            if profile.profile_id.trim() != "paper" {
                errors.push(
                    "execution_profile.profile_id 在当前运行时中必须为 \"paper\"".to_string(),
                );
            }
            if let Some(value) = profile.fee_bps {
                if !value.is_finite() || value < 0.0 {
                    errors.push("execution_profile.fee_bps 必须是有限数且大于等于 0".to_string());
                }
            }
            if let Some(value) = profile.slippage_bps {
                if !value.is_finite() || value < 0.0 {
                    errors.push(
                        "execution_profile.slippage_bps 必须是有限数且大于等于 0".to_string(),
                    );
                }
            }
        }

        for (index, item) in self.unknowns.iter().enumerate() {
            if item.path.trim().is_empty() {
                errors.push(format!("unknowns[{index}].path 是必需的"));
            }
            if item.reason.trim().is_empty() {
                errors.push(format!("unknowns[{index}].reason 是必需的"));
            }
        }

        errors
    }

    pub fn validate(&self) -> Result<(), StrategyIrValidationError> {
        let errors = self.validation_errors();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(StrategyIrValidationError { errors })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyIrValidationError {
    pub errors: Vec<String>,
}

impl fmt::Display for StrategyIrValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Strategy IR validation failed: {}",
            self.errors.join("; ")
        )
    }
}

impl std::error::Error for StrategyIrValidationError {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StrategyMetadata {
    pub strategy_id: String,
    pub name: String,
    pub summary: String,
    pub source: StrategySource,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StrategySource {
    pub source_type: StrategySourceType,
    pub paper_title: String,
    pub paper_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StrategySourceType {
    ManualPaperAnalysis,
    LlmPaperAnalysis,
    HumanAuthored,
    Imported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SignalDefinition {
    pub signal_id: String,
    pub name: String,
    pub indicator: IndicatorDefinition,
    #[serde(default)]
    pub transforms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct IndicatorDefinition {
    pub kind: IndicatorKind,
    pub inputs: Vec<String>,
    #[serde(default)]
    pub params: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IndicatorKind {
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

const DECLARED_INDICATOR_KINDS: [IndicatorKind; 18] = [
    IndicatorKind::MaCross,
    IndicatorKind::Rsi,
    IndicatorKind::Macd,
    IndicatorKind::Momentum,
    IndicatorKind::Spread,
    IndicatorKind::ZScore,
    IndicatorKind::Custom,
    IndicatorKind::QuoteObserve,
    IndicatorKind::Atr,
    IndicatorKind::BollingerBands,
    IndicatorKind::Obv,
    IndicatorKind::Cmf,
    IndicatorKind::Adx,
    IndicatorKind::Stochastic,
    IndicatorKind::Cci,
    IndicatorKind::ParabolicSar,
    IndicatorKind::KeltnerChannel,
    IndicatorKind::DonchianChannel,
];

const SUPPORTED_INDICATOR_KINDS: [IndicatorKind; 18] = [
    IndicatorKind::MaCross,
    IndicatorKind::Rsi,
    IndicatorKind::Macd,
    IndicatorKind::Momentum,
    IndicatorKind::Spread,
    IndicatorKind::ZScore,
    IndicatorKind::Custom,
    IndicatorKind::QuoteObserve,
    IndicatorKind::Atr,
    IndicatorKind::BollingerBands,
    IndicatorKind::Obv,
    IndicatorKind::Cmf,
    IndicatorKind::Adx,
    IndicatorKind::Stochastic,
    IndicatorKind::Cci,
    IndicatorKind::ParabolicSar,
    IndicatorKind::KeltnerChannel,
    IndicatorKind::DonchianChannel,
];

pub fn declared_indicator_kinds() -> &'static [IndicatorKind] {
    &DECLARED_INDICATOR_KINDS
}

pub fn supported_indicator_kinds() -> &'static [IndicatorKind] {
    &SUPPORTED_INDICATOR_KINDS
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StrategyLogic {
    pub entry_rules: Vec<LogicRule>,
    #[serde(default)]
    pub exit_rules: Vec<LogicRule>,
    pub position_sizing: PositionSizing,
    pub rebalance_rule: Option<RebalanceRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogicRule {
    pub rule_id: String,
    pub condition: String,
    pub action: LogicAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LogicAction {
    OpenLong,
    CloseLong,
    OpenShort,
    CloseShort,
    Rebalance,
    Hold,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PositionSizing {
    pub method: PositionSizingMethod,
    pub value: KnownOrUnknown<f64>,
    pub unit: PositionSizingUnit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PositionSizingMethod {
    FixedRatio,
    VolatilityTarget,
    EqualWeight,
    RiskParity,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PositionSizingUnit {
    PortfolioRatio,
    Leverage,
    Quantity,
    Notional,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RebalanceRule {
    pub frequency: KnownOrUnknown<String>,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StrategyRiskRules {
    pub max_position_ratio: KnownOrUnknown<f64>,
    pub stop_loss_ratio: KnownOrUnknown<f64>,
    pub take_profit_ratio: Option<KnownOrUnknown<f64>>,
    pub max_drawdown_ratio: Option<KnownOrUnknown<f64>>,
    pub max_trades_per_day: Option<KnownOrUnknown<u32>>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategyRiskProfileRef {
    pub profile_id: String,
    #[serde(default)]
    pub max_position: Option<f64>,
    #[serde(default)]
    pub max_total_leverage: Option<f64>,
    #[serde(default)]
    pub max_exchange_leverage: Option<f64>,
    #[serde(default)]
    pub min_action_interval_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DataRequirement {
    pub data_id: String,
    pub venue: KnownOrUnknown<String>,
    pub symbol: KnownOrUnknown<String>,
    pub data_type: DataRequirementType,
    pub granularity: KnownOrUnknown<String>,
    pub lookback: KnownOrUnknown<u32>,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataRequirementType {
    Kline,
    Quote,
    Tick,
    OrderBook,
    Fundamental,
    Event,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StrategyExecution {
    pub venue_type: KnownOrUnknown<String>,
    pub order_type: KnownOrUnknown<String>,
    pub time_in_force: Option<KnownOrUnknown<String>>,
    pub slippage_model: KnownOrUnknown<String>,
    pub latency_assumption_ms: Option<KnownOrUnknown<u32>>,
    pub capital_base: Option<KnownOrUnknown<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategyExecutionProfileRef {
    pub profile_id: String,
    #[serde(default)]
    pub fee_bps: Option<f64>,
    #[serde(default)]
    pub slippage_bps: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GapAnnotation {
    pub gap_type: GapType,
    pub summary: String,
    pub severity: GapSeverity,
    pub blocking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GapType {
    Expression,
    Data,
    Execution,
    Risk,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GapSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StrategyUnknown {
    pub path: String,
    pub reason: String,
}

fn validate_unique_ids<'a>(
    values: impl Iterator<Item = &'a str>,
    label: &str,
    errors: &mut Vec<String>,
) {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value.to_string()) {
            errors.push(format!("{label} 包含重复的 id: {value}"));
        }
    }
}

fn validate_logic_rule(rule: &LogicRule, path: &str, errors: &mut Vec<String>) {
    if rule.rule_id.trim().is_empty() {
        errors.push(format!("{path}.rule_id 是必需的"));
    }
    if rule.condition.trim().is_empty() {
        errors.push(format!("{path}.condition 是必需的"));
    }
}

fn indicator_kind_supported(kind: &IndicatorKind) -> bool {
    supported_indicator_kinds().contains(kind)
}

fn validate_unknownable<T>(value: &KnownOrUnknown<T>, path: &str, errors: &mut Vec<String>) {
    if let KnownOrUnknown::Unknown(marker) = value {
        if marker != "unknown" {
            errors.push(format!("{path} 未知标记必须为 \"unknown\""));
        }
    }
}

fn validate_unknownable_opt<T>(
    value: Option<&KnownOrUnknown<T>>,
    path: &str,
    errors: &mut Vec<String>,
) {
    if let Some(value) = value {
        validate_unknownable(value, path, errors);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_JSON: &str = r#"
{
  "ir_version": "strategy_ir/v0",
  "metadata": {
    "strategy_id": "paper_dual_ma_v1",
    "name": "Dual Moving Average Trend Strategy",
    "summary": "Go long when the fast moving average crosses above the slow moving average.",
    "source": {
      "source_type": "manual_paper_analysis",
      "paper_title": "A Dual Moving Average Strategy",
      "paper_reference": "doi:10.0000/example"
    },
    "authors": ["QuantPilot"],
    "tags": ["trend", "moving_average"]
  },
  "signals": [
    {
      "signal_id": "ma_cross",
      "name": "Fast and slow MA crossover",
      "indicator": {
        "kind": "ma_cross",
        "inputs": ["close"],
        "params": {
          "fast": 20,
          "slow": 50
        }
      },
      "transforms": ["normalize"]
    }
  ],
  "logic": {
    "entry_rules": [
      {
        "rule_id": "entry_long",
        "condition": "ma_cross == bullish",
        "action": "open_long"
      }
    ],
    "exit_rules": [
      {
        "rule_id": "exit_long",
        "condition": "ma_cross == bearish",
        "action": "close_long"
      }
    ],
    "position_sizing": {
      "method": "fixed_ratio",
      "value": 0.2,
      "unit": "portfolio_ratio"
    },
    "rebalance_rule": {
      "frequency": "1d",
      "condition": "on_bar_close"
    }
  },
  "risk_rules": {
    "max_position_ratio": 0.2,
    "stop_loss_ratio": 0.02,
    "take_profit_ratio": "unknown",
    "max_drawdown_ratio": 0.1,
    "max_trades_per_day": 5,
    "notes": ["Take-profit is not defined in the paper."]
  },
  "data_requirements": [
    {
      "data_id": "btc_daily_kline",
      "venue": "binance",
      "symbol": "BTCUSDT",
      "data_type": "kline",
      "granularity": "1d",
      "lookback": 200,
      "fields": ["open", "high", "low", "close", "volume"]
    }
  ],
  "execution": {
    "venue_type": "paper",
    "order_type": "market",
    "time_in_force": "gtc",
    "slippage_model": "fixed_bps",
    "latency_assumption_ms": 250,
    "capital_base": 100000.0
  },
  "gap_annotations": [
    {
      "gap_type": "execution",
      "summary": "The paper assumes zero slippage.",
      "severity": "high",
      "blocking": false
    }
  ],
  "unknowns": [
    {
      "path": "risk_rules.take_profit_ratio",
      "reason": "The paper does not describe a take-profit rule."
    }
  ]
}
"#;

    #[test]
    fn parses_and_validates_strategy_ir_v0() {
        let ir: StrategyIr = serde_json::from_str(SAMPLE_JSON).unwrap();
        ir.validate().unwrap();
        assert_eq!(ir.ir_version, STRATEGY_IR_V0_VERSION);
        assert_eq!(ir.signals.len(), 1);
        assert!(matches!(
            ir.risk_rules.take_profit_ratio,
            Some(KnownOrUnknown::Unknown(_))
        ));
    }

    #[test]
    fn rejects_invalid_unknown_marker() {
        let mut ir: StrategyIr = serde_json::from_str(SAMPLE_JSON).unwrap();
        ir.execution.order_type = KnownOrUnknown::Unknown("tbd".to_string());
        let err = ir.validate().unwrap_err();
        assert!(err
            .errors
            .iter()
            .any(|item| item.contains("execution.order_type")));
    }

    #[test]
    fn rejects_duplicate_signal_ids() {
        let mut ir: StrategyIr = serde_json::from_str(SAMPLE_JSON).unwrap();
        ir.signals.push(ir.signals[0].clone());
        let err = ir.validate().unwrap_err();
        assert!(err.errors.iter().any(|item| item.contains("重复的 id")));
    }

    #[test]
    fn rejects_non_finite_execution_profile_costs() {
        let mut ir: StrategyIr = serde_json::from_str(SAMPLE_JSON).unwrap();
        ir.execution_profile = Some(StrategyExecutionProfileRef {
            profile_id: "paper".to_string(),
            fee_bps: Some(f64::INFINITY),
            slippage_bps: None,
        });
        let err = ir.validate().unwrap_err();
        assert!(err
            .errors
            .iter()
            .any(|item| item.contains("execution_profile.fee_bps")));

        ir.execution_profile = Some(StrategyExecutionProfileRef {
            profile_id: "paper".to_string(),
            fee_bps: None,
            slippage_bps: Some(f64::NAN),
        });
        let err = ir.validate().unwrap_err();
        assert!(err
            .errors
            .iter()
            .any(|item| item.contains("execution_profile.slippage_bps")));
    }

    #[test]
    fn accepts_custom_indicator_when_runtime_support_is_declared() {
        let mut ir: StrategyIr = serde_json::from_str(SAMPLE_JSON).unwrap();
        ir.signals[0].indicator.kind = IndicatorKind::Custom;
        ir.validate().unwrap();
    }
}
