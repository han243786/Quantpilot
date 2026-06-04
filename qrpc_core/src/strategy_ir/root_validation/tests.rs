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
