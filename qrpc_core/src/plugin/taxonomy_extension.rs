use serde::{Deserialize, Serialize};

use super::PluginCapabilityContract;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginKind {
    Data,
    Intent,
    Agent,
    Risk,
    Execution,
}

impl PluginKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Data => "data",
            Self::Intent => "intent",
            Self::Agent => "agent",
            Self::Risk => "risk",
            Self::Execution => "execution",
        }
    }

    pub fn supported_extension_points(&self) -> &'static [ExtensionPoint] {
        match self {
            Self::Data => &[ExtensionPoint::DataModuleProvider],
            Self::Intent => &[ExtensionPoint::IntentModuleProvider],
            Self::Agent => &[ExtensionPoint::AgentModuleProvider],
            Self::Risk => &[ExtensionPoint::RiskCheckerProvider],
            Self::Execution => &[ExtensionPoint::ExecutionModuleProvider],
        }
    }

    pub fn supported_capability_contracts(&self) -> &'static [PluginCapabilityContract] {
        match self {
            Self::Data => &[PluginCapabilityContract::DataModuleProvider],
            Self::Intent => &[PluginCapabilityContract::IntentModuleProvider],
            Self::Agent => &[PluginCapabilityContract::AgentModuleProvider],
            Self::Risk => &[PluginCapabilityContract::RiskCheckerProvider],
            Self::Execution => &[PluginCapabilityContract::ExecutionModuleProvider],
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionPoint {
    DataModuleProvider,
    IntentModuleProvider,
    AgentModuleProvider,
    RiskCheckerProvider,
    ExecutionModuleProvider,
}

impl ExtensionPoint {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DataModuleProvider => "data_module_provider",
            Self::IntentModuleProvider => "intent_module_provider",
            Self::AgentModuleProvider => "agent_module_provider",
            Self::RiskCheckerProvider => "risk_checker_provider",
            Self::ExecutionModuleProvider => "execution_module_provider",
        }
    }
}
