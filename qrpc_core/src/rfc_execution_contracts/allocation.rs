use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{PortfolioTarget, Symbol};

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
