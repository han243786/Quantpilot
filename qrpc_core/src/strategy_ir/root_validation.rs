use serde::{Deserialize, Serialize};

use super::{
    supported_indicator_kinds, DataRequirement, GapAnnotation, IndicatorKind, KnownOrUnknown,
    LogicRule, SignalDefinition, StrategyExecution, StrategyExecutionProfileRef,
    StrategyIrValidationError, StrategyLogic, StrategyMetadata, StrategyRiskProfileRef,
    StrategyRiskRules, StrategyUnknown, STRATEGY_IR_V0_VERSION,
};

mod data_execution_validation;
mod identity_required_validation;
mod risk_validation;
mod signal_logic_validation;
mod unknown_marker_validation;

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

        data_execution_validation::validate_data_and_execution(self, &mut errors);

        unknown_marker_validation::validate_unknowns(self, &mut errors);

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
    unknown_marker_validation::validate_unknownable(value, path, errors);
}

fn validate_unknownable_opt<T>(
    value: Option<&KnownOrUnknown<T>>,
    path: &str,
    errors: &mut Vec<String>,
) {
    unknown_marker_validation::validate_unknownable_opt(value, path, errors);
}

#[cfg(test)]
mod tests;
