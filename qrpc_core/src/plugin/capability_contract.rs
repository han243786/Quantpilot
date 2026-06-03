use serde::{Deserialize, Serialize};

pub const PLUGIN_CAPABILITY_CONTRACT_V1_VERSION: &str = "v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginCapabilityDeclaration {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum PluginCapabilityContract {
    DataModuleProvider,
    IntentModuleProvider,
    AgentModuleProvider,
    RiskCheckerProvider,
    ExecutionModuleProvider,
}

impl PluginCapabilityContract {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DataModuleProvider => "quantpilot.capability.data_module_provider",
            Self::IntentModuleProvider => "quantpilot.capability.intent_module_provider",
            Self::AgentModuleProvider => "quantpilot.capability.agent_module_provider",
            Self::RiskCheckerProvider => "quantpilot.capability.risk_checker_provider",
            Self::ExecutionModuleProvider => "quantpilot.capability.execution_module_provider",
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        match input {
            "quantpilot.capability.data_module_provider" => Some(Self::DataModuleProvider),
            "quantpilot.capability.intent_module_provider" => Some(Self::IntentModuleProvider),
            "quantpilot.capability.agent_module_provider" => Some(Self::AgentModuleProvider),
            "quantpilot.capability.risk_checker_provider" => Some(Self::RiskCheckerProvider),
            "quantpilot.capability.execution_module_provider" => {
                Some(Self::ExecutionModuleProvider)
            }
            _ => None,
        }
    }
}
