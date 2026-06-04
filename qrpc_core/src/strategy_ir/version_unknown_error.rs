use serde::{Deserialize, Serialize};
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
