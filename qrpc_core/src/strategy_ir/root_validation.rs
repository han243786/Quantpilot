use serde::{Deserialize, Serialize};

use super::{
    supported_indicator_kinds, DataRequirement, GapAnnotation, IndicatorKind, KnownOrUnknown,
    LogicRule, SignalDefinition, StrategyExecution, StrategyExecutionProfileRef,
    StrategyIrValidationError, StrategyLogic, StrategyMetadata, StrategyRiskProfileRef,
    StrategyRiskRules, StrategyUnknown, STRATEGY_IR_V0_VERSION,
};

mod identity_required_validation;
mod risk_validation;
mod signal_logic_validation;

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

        identity_required_validation::validate_identity_and_required_fields(self, &mut errors);
        signal_logic_validation::validate_signal_and_logic(self, &mut errors);

        risk_validation::validate_risk(self, &mut errors);

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
