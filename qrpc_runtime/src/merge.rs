use anyhow::Result;
use qrpc_core::{AgentDecision, ProposedAction, SignalSide, Symbol};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergePolicy {
    WeightedMerge,
    ConfidenceFirst,
    ConflictSuppression,
}

impl Default for MergePolicy {
    fn default() -> Self {
        Self::WeightedMerge
    }
}

#[derive(Debug, Clone)]
pub struct StrategyInput {
    pub strategy_id: String,
    pub weight: f64,
    pub agent_decisions: Vec<AgentDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeDecisionRecord {
    pub source_strategy_id: String,
    pub source_weight: f64,
    pub contribution: f64,
    pub suppressed: bool,
    pub suppression_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MergedOutput {
    pub decisions: Vec<AgentDecision>,
    pub merge_records: Vec<MergeDecisionRecord>,
    pub conflict_count: usize,
    pub suppressed_count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct StrategyMergeEngine {
    pub policy: MergePolicy,
    pub max_total_exposure_ratio: Option<f64>,
    pub symbol_concentration_limit: Option<f64>,
}

impl StrategyMergeEngine {
    pub fn new(policy: MergePolicy) -> Self {
        Self {
            policy,
            ..Default::default()
        }
    }

    pub fn with_limits(
        policy: MergePolicy,
        max_total_exposure_ratio: Option<f64>,
        symbol_concentration_limit: Option<f64>,
    ) -> Self {
        Self {
            policy,
            max_total_exposure_ratio,
            symbol_concentration_limit,
        }
    }

    pub fn merge(&self, strategy_inputs: &[StrategyInput]) -> Result<MergedOutput> {
        match self.policy {
            MergePolicy::WeightedMerge => self.merge_weighted(strategy_inputs),
            MergePolicy::ConfidenceFirst => self.merge_confidence_first(strategy_inputs),
            MergePolicy::ConflictSuppression => self.merge_conflict_suppression(strategy_inputs),
        }
    }

    fn merge_weighted(&self, strategy_inputs: &[StrategyInput]) -> Result<MergedOutput> {
        let total_weight: f64 = strategy_inputs.iter().map(|s| s.weight).sum();
        if !total_weight.is_finite() || total_weight <= 0.0 {
            return Ok(MergedOutput {
                decisions: Vec::new(),
                merge_records: Vec::new(),
                conflict_count: 0,
                suppressed_count: 0,
            });
        }

        let mut records = Vec::new();
        let conflict_count = 0;
        let suppressed_count = 0;

        let mut merged_by_symbol: BTreeMap<Symbol, (f64, f64, Vec<ProposedAction>, Vec<String>)> =
            BTreeMap::new();

        for input in strategy_inputs {
            let norm_weight = input.weight / total_weight;
            for decision in &input.agent_decisions {
                let entry = merged_by_symbol
                    .entry(decision.symbol.clone())
                    .or_insert_with(|| (0.0, 0.0, Vec::new(), Vec::new()));

                entry.0 += decision.net_strength * norm_weight;
                entry.1 += norm_weight;
                entry.3.push(input.strategy_id.clone());

                for action in &decision.proposed_actions {
                    let mut scaled_action = action.clone();
                    scaled_action.quantity_ratio *= norm_weight;
                    entry.2.push(scaled_action);
                }
            }
            records.push(MergeDecisionRecord {
                source_strategy_id: input.strategy_id.clone(),
                source_weight: input.weight,
                contribution: norm_weight,
                suppressed: false,
                suppression_reason: None,
            });
        }

        let decisions: Vec<AgentDecision> = merged_by_symbol
            .into_iter()
            .map(
                |(symbol, (total_strength, weight_sum, actions, source_strategy_ids))| {
                    let net_strength = if weight_sum.is_finite() && weight_sum > 0.0 {
                        total_strength / weight_sum
                    } else {
                        0.0
                    };
                    let net_side = if net_strength > 0.01 {
                        SignalSide::Long
                    } else if net_strength < -0.01 {
                        SignalSide::Short
                    } else {
                        SignalSide::Neutral
                    };

                    let reason = format!(
                        "weighted merge from strategies: {}",
                        source_strategy_ids.join(", ")
                    );

                    AgentDecision {
                        decision_id: format!("merged-{symbol:?}-{}", source_strategy_ids.len()),
                        agent_id: "merge_engine".to_string(),
                        symbol,
                        exchange_targets: Vec::new(),
                        net_side,
                        net_strength,
                        portfolio_target_decision: None,
                        proposed_actions: actions,
                        reason,
                        produced_at_ms: 0,
                        trace_id: "merge".to_string(),
                    }
                },
            )
            .collect();

        Ok(MergedOutput {
            decisions,
            merge_records: records,
            conflict_count,
            suppressed_count,
        })
    }

    fn merge_confidence_first(&self, strategy_inputs: &[StrategyInput]) -> Result<MergedOutput> {
        let mut records = Vec::new();
        let conflict_count = 0;
        let mut suppressed_count = 0;

        let mut best_by_symbol: BTreeMap<Symbol, (f64, AgentDecision, String)> = BTreeMap::new();

        for input in strategy_inputs {
            for decision in &input.agent_decisions {
                let confidence = decision.net_strength.abs();
                let current_best = best_by_symbol
                    .entry(decision.symbol.clone())
                    .or_insert_with(|| (0.0, decision.clone(), input.strategy_id.clone()));

                if confidence > current_best.0 {
                    suppressed_count += 1;
                    *current_best = (confidence, decision.clone(), input.strategy_id.clone());
                    records.push(MergeDecisionRecord {
                        source_strategy_id: input.strategy_id.clone(),
                        source_weight: input.weight,
                        contribution: 1.0,
                        suppressed: false,
                        suppression_reason: None,
                    });
                } else {
                    records.push(MergeDecisionRecord {
                        source_strategy_id: input.strategy_id.clone(),
                        source_weight: input.weight,
                        contribution: 0.0,
                        suppressed: true,
                        suppression_reason: Some(format!(
                            "confidence {:.4} below best {:.4}",
                            confidence, current_best.0
                        )),
                    });
                    suppressed_count += 1;
                }
            }
        }

        let decisions: Vec<AgentDecision> = best_by_symbol
            .into_values()
            .map(|(_, mut decision, source_id)| {
                decision.decision_id = format!("cf-{}", decision.decision_id);
                decision.reason = format!(
                    "confidence-first merge, selected from strategy: {}",
                    source_id
                );
                decision
            })
            .collect();

        Ok(MergedOutput {
            decisions,
            merge_records: records,
            conflict_count,
            suppressed_count,
        })
    }

    fn merge_conflict_suppression(
        &self,
        strategy_inputs: &[StrategyInput],
    ) -> Result<MergedOutput> {
        let mut records = Vec::new();
        let mut conflict_count = 0;
        let mut suppressed_count = 0;

        let mut signals_by_symbol: BTreeMap<Symbol, Vec<(SignalSide, f64, &AgentDecision, &str)>> =
            BTreeMap::new();

        for input in strategy_inputs {
            for decision in &input.agent_decisions {
                signals_by_symbol
                    .entry(decision.symbol.clone())
                    .or_default()
                    .push((
                        decision.net_side.clone(),
                        decision.net_strength.abs(),
                        decision,
                        input.strategy_id.as_str(),
                    ));
            }
        }

        let mut decisions = Vec::new();

        for (_symbol, entries) in signals_by_symbol {
            let has_long = entries
                .iter()
                .any(|(side, _, _, _)| matches!(side, SignalSide::Long));
            let has_short = entries
                .iter()
                .any(|(side, _, _, _)| matches!(side, SignalSide::Short));

            if has_long && has_short {
                conflict_count += 1;
                let long_strength: f64 = entries
                    .iter()
                    .filter(|(s, _, _, _)| matches!(s, SignalSide::Long))
                    .map(|(_, strength, _, _)| strength)
                    .sum();
                let short_strength: f64 = entries
                    .iter()
                    .filter(|(s, _, _, _)| matches!(s, SignalSide::Short))
                    .map(|(_, strength, _, _)| strength)
                    .sum();

                if (long_strength - short_strength).abs() < 0.05 {
                    for (_side, _strength, _decision, strategy_id) in &entries {
                        records.push(MergeDecisionRecord {
                            source_strategy_id: strategy_id.to_string(),
                            source_weight: 1.0,
                            contribution: 0.0,
                            suppressed: true,
                            suppression_reason: Some(format!(
                                "conflict detected: {:?} vs {:?}, strengths {:.4} vs {:.4}",
                                SignalSide::Long,
                                SignalSide::Short,
                                long_strength,
                                short_strength
                            )),
                        });
                        suppressed_count += 1;
                    }
                    continue;
                }

                let winning_side = if long_strength > short_strength {
                    SignalSide::Long
                } else {
                    SignalSide::Short
                };

                for (side, _strength, decision, strategy_id) in &entries {
                    let is_winner = matches!(
                        (side, &winning_side),
                        (SignalSide::Long, SignalSide::Long)
                            | (SignalSide::Short, SignalSide::Short)
                            | (SignalSide::Neutral, SignalSide::Neutral)
                    );
                    if is_winner {
                        decisions.push((*decision).clone());
                        records.push(MergeDecisionRecord {
                            source_strategy_id: strategy_id.to_string(),
                            source_weight: 1.0,
                            contribution: 1.0,
                            suppressed: false,
                            suppression_reason: None,
                        });
                    } else {
                        records.push(MergeDecisionRecord {
                            source_strategy_id: strategy_id.to_string(),
                            source_weight: 1.0,
                            contribution: 0.0,
                            suppressed: true,
                            suppression_reason: Some(format!(
                                "conflict suppressed: {:?} losing to {:?}",
                                side, winning_side
                            )),
                        });
                        suppressed_count += 1;
                    }
                }
            } else {
                for (_side, _strength, decision, strategy_id) in &entries {
                    decisions.push((*decision).clone());
                    records.push(MergeDecisionRecord {
                        source_strategy_id: strategy_id.to_string(),
                        source_weight: 1.0,
                        contribution: 1.0,
                        suppressed: false,
                        suppression_reason: None,
                    });
                }
            }
        }

        if let Some(limit) = self.symbol_concentration_limit {
            // v2.1.1: limit 为比率 (0.0-1.0), 计算实际允许的最大决策数
            let mut symbol_counts: BTreeMap<Symbol, usize> = BTreeMap::new();
            for decision in &decisions {
                *symbol_counts.entry(decision.symbol.clone()).or_default() += 1;
            }
            if symbol_counts.values().any(|count| *count as f64 > limit) {
                let max_allowed = (limit * decisions.len() as f64).ceil() as usize;
                // v2.5.0: 按 net_strength 降序排序后截断, 保留最强信号
                decisions.sort_by(|a, b| {
                    b.net_strength
                        .partial_cmp(&a.net_strength)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                decisions.truncate(max_allowed.max(1));
            }
        }

        Ok(MergedOutput {
            decisions,
            merge_records: records,
            conflict_count,
            suppressed_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_core::{Exchange, OrderSide};

    fn sample_decision(id: &str, symbol: Symbol, side: SignalSide, strength: f64) -> AgentDecision {
        let order_side = if matches!(side, SignalSide::Long) {
            OrderSide::Buy
        } else {
            OrderSide::Sell
        };
        AgentDecision {
            decision_id: id.to_string(),
            agent_id: "test_agent".to_string(),
            symbol,
            exchange_targets: vec![Exchange::Binance],
            net_side: side,
            net_strength: strength,
            portfolio_target_decision: None,
            proposed_actions: vec![ProposedAction {
                exchange: Exchange::Binance,
                side: order_side,
                quantity_ratio: strength.abs().min(1.0),
                reference_price: 50_000.0,
                strategy_tag: id.to_string(),
            }],
            reason: "test".to_string(),
            produced_at_ms: 0,
            trace_id: "test_trace".to_string(),
        }
    }

    #[test]
    fn weighted_merge_averages_signals() {
        let engine = StrategyMergeEngine::new(MergePolicy::WeightedMerge);
        let inputs = vec![
            StrategyInput {
                strategy_id: "strategy_a".to_string(),
                weight: 1.0,
                agent_decisions: vec![sample_decision("a", Symbol::BtcUsdt, SignalSide::Long, 0.6)],
            },
            StrategyInput {
                strategy_id: "strategy_b".to_string(),
                weight: 1.0,
                agent_decisions: vec![sample_decision("b", Symbol::BtcUsdt, SignalSide::Long, 0.2)],
            },
        ];

        let result = engine.merge(&inputs).unwrap();
        assert_eq!(result.decisions.len(), 1);
        assert!((result.decisions[0].net_strength - 0.4).abs() < 1e-9);
        assert_eq!(result.decisions[0].agent_id, "merge_engine");
    }

    #[test]
    fn confidence_first_picks_strongest() {
        let engine = StrategyMergeEngine::new(MergePolicy::ConfidenceFirst);
        let inputs = vec![
            StrategyInput {
                strategy_id: "weak".to_string(),
                weight: 1.0,
                agent_decisions: vec![sample_decision("w", Symbol::BtcUsdt, SignalSide::Long, 0.3)],
            },
            StrategyInput {
                strategy_id: "strong".to_string(),
                weight: 1.0,
                agent_decisions: vec![sample_decision("s", Symbol::BtcUsdt, SignalSide::Long, 0.9)],
            },
        ];

        let result = engine.merge(&inputs).unwrap();
        assert_eq!(result.decisions.len(), 1);
        assert!((result.decisions[0].net_strength - 0.9).abs() < 1e-9);
        assert!(result.suppressed_count >= 1);
    }

    #[test]
    fn conflict_suppression_resolves_opposing_signals() {
        let engine = StrategyMergeEngine::new(MergePolicy::ConflictSuppression);
        let inputs = vec![
            StrategyInput {
                strategy_id: "long_strat".to_string(),
                weight: 1.0,
                agent_decisions: vec![sample_decision(
                    "long",
                    Symbol::BtcUsdt,
                    SignalSide::Long,
                    0.8,
                )],
            },
            StrategyInput {
                strategy_id: "short_strat".to_string(),
                weight: 1.0,
                agent_decisions: vec![sample_decision(
                    "short",
                    Symbol::BtcUsdt,
                    SignalSide::Short,
                    0.3,
                )],
            },
        ];

        let result = engine.merge(&inputs).unwrap();
        assert!(result.conflict_count >= 1);
        assert_eq!(result.decisions.len(), 1);
        assert!(matches!(result.decisions[0].net_side, SignalSide::Long));
    }

    #[test]
    fn conflict_suppression_cancels_equal_strength() {
        let engine = StrategyMergeEngine::new(MergePolicy::ConflictSuppression);
        let inputs = vec![
            StrategyInput {
                strategy_id: "long_strat".to_string(),
                weight: 1.0,
                agent_decisions: vec![sample_decision(
                    "long",
                    Symbol::BtcUsdt,
                    SignalSide::Long,
                    0.5,
                )],
            },
            StrategyInput {
                strategy_id: "short_strat".to_string(),
                weight: 1.0,
                agent_decisions: vec![sample_decision(
                    "short",
                    Symbol::BtcUsdt,
                    SignalSide::Short,
                    0.5,
                )],
            },
        ];

        let result = engine.merge(&inputs).unwrap();
        assert!(result.conflict_count >= 1);
        assert!(result.decisions.is_empty());
        assert!(result.suppressed_count >= 2);
    }

    #[test]
    fn weighted_merge_handles_different_symbols() {
        let engine = StrategyMergeEngine::new(MergePolicy::WeightedMerge);
        let btc = Symbol::BtcUsdt;
        let eth = Symbol::parse("ETHUSDT");
        let inputs = vec![StrategyInput {
            strategy_id: "multi".to_string(),
            weight: 1.0,
            agent_decisions: vec![
                sample_decision("btc", btc.clone(), SignalSide::Long, 0.7),
                sample_decision("eth", eth.clone(), SignalSide::Short, -0.4),
            ],
        }];

        let result = engine.merge(&inputs).unwrap();
        assert_eq!(result.decisions.len(), 2);
        let btc_decision = result.decisions.iter().find(|d| d.symbol == btc).unwrap();
        let eth_decision = result.decisions.iter().find(|d| d.symbol == eth).unwrap();
        assert!(matches!(btc_decision.net_side, SignalSide::Long));
        assert!(matches!(eth_decision.net_side, SignalSide::Short));
    }

    #[test]
    fn merge_records_are_traceable() {
        let engine = StrategyMergeEngine::new(MergePolicy::WeightedMerge);
        let inputs = vec![
            StrategyInput {
                strategy_id: "strat_x".to_string(),
                weight: 2.0,
                agent_decisions: vec![sample_decision("x", Symbol::BtcUsdt, SignalSide::Long, 0.5)],
            },
            StrategyInput {
                strategy_id: "strat_y".to_string(),
                weight: 1.0,
                agent_decisions: vec![sample_decision("y", Symbol::BtcUsdt, SignalSide::Long, 0.3)],
            },
        ];

        let result = engine.merge(&inputs).unwrap();
        assert_eq!(result.merge_records.len(), 2);
        assert!((result.merge_records[0].contribution - 2.0 / 3.0).abs() < 1e-9);
        assert!((result.merge_records[1].contribution - 1.0 / 3.0).abs() < 1e-9);
    }
}
