use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

use super::{default_true, V4_RUNTIME_MODE_CONTRACT_VERSION};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeTradingMode {
    PaperActual,
    PaperSimulated,
    LiveActual,
    LiveSimulated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimeModeContract {
    #[serde(default = "default_runtime_mode_contract_version")]
    pub schema_version: String,
    #[serde(default = "default_runtime_mode_specs")]
    pub modes: Vec<RuntimeTradingModeSpec>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeTradingModeSpec {
    pub mode: RuntimeTradingMode,
    pub account_domain: RuntimeAccountDomain,
    pub settlement_authority: RuntimeSettlementAuthority,
    pub execution_event_source: RuntimeExecutionEventSource,
    #[serde(default)]
    pub provider_order_submission_allowed: bool,
    #[serde(default)]
    pub provider_fill_required: bool,
    #[serde(default)]
    pub local_fill_engine_required: bool,
    #[serde(default)]
    pub local_ledger_required: bool,
    #[serde(default)]
    pub provider_account_context_required: bool,
    #[serde(default = "default_true")]
    pub risk_plane_required: bool,
    #[serde(default = "default_runtime_execution_events")]
    pub required_events: Vec<RuntimeExecutionEventKind>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeAccountDomain {
    Paper,
    Live,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeSettlementAuthority {
    ProviderActual,
    LocalSimulated,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeExecutionEventSource {
    ProviderActual,
    LocalSimulated,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeExecutionEventKind {
    OrderAcknowledged,
    OrderRejected,
    OrderPartiallyFilled,
    OrderFilled,
    FeeCharged,
    PortfolioChanged,
}

pub const V4_RUNTIME_EXECUTION_EVENTS: [RuntimeExecutionEventKind; 6] = [
    RuntimeExecutionEventKind::OrderAcknowledged,
    RuntimeExecutionEventKind::OrderRejected,
    RuntimeExecutionEventKind::OrderPartiallyFilled,
    RuntimeExecutionEventKind::OrderFilled,
    RuntimeExecutionEventKind::FeeCharged,
    RuntimeExecutionEventKind::PortfolioChanged,
];

pub fn v4_runtime_execution_events() -> &'static [RuntimeExecutionEventKind] {
    &V4_RUNTIME_EXECUTION_EVENTS
}

pub fn default_v4_runtime_mode_contract() -> RuntimeModeContract {
    RuntimeModeContract {
        schema_version: V4_RUNTIME_MODE_CONTRACT_VERSION.to_string(),
        modes: default_runtime_mode_specs(),
        metadata: BTreeMap::new(),
    }
}

impl Default for RuntimeModeContract {
    fn default() -> Self {
        default_v4_runtime_mode_contract()
    }
}

impl RuntimeModeContract {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_RUNTIME_MODE_CONTRACT_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_RUNTIME_MODE_CONTRACT_VERSION
            ));
        }

        let mut seen_modes = BTreeSet::new();
        for spec in &self.modes {
            if !seen_modes.insert(spec.mode) {
                errors.push(format!("duplicate runtime trading mode `{:?}`", spec.mode));
            }
            errors.extend(validate_runtime_mode_spec(spec));
        }

        for mode in required_runtime_trading_modes() {
            if !seen_modes.contains(&mode) {
                errors.push(format!("runtime mode contract must declare `{:?}`", mode));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn mode_spec(&self, mode: RuntimeTradingMode) -> Option<&RuntimeTradingModeSpec> {
        self.modes.iter().find(|spec| spec.mode == mode)
    }

    pub fn settlement_authority_for(
        &self,
        mode: RuntimeTradingMode,
    ) -> Option<RuntimeSettlementAuthority> {
        self.mode_spec(mode).map(|spec| spec.settlement_authority)
    }
}

fn validate_runtime_mode_spec(spec: &RuntimeTradingModeSpec) -> Vec<String> {
    let mut errors = Vec::new();
    let expected = expected_runtime_mode_spec(spec.mode);

    if spec.account_domain != expected.account_domain {
        errors.push(format!(
            "`{:?}` account_domain must be `{:?}`",
            spec.mode, expected.account_domain
        ));
    }
    if spec.settlement_authority != expected.settlement_authority {
        errors.push(format!(
            "`{:?}` settlement_authority must be `{:?}`",
            spec.mode, expected.settlement_authority
        ));
    }
    if spec.execution_event_source != expected.execution_event_source {
        errors.push(format!(
            "`{:?}` execution_event_source must be `{:?}`",
            spec.mode, expected.execution_event_source
        ));
    }
    if spec.provider_order_submission_allowed != expected.provider_order_submission_allowed {
        errors.push(format!(
            "`{:?}` provider_order_submission_allowed must be {}",
            spec.mode, expected.provider_order_submission_allowed
        ));
    }
    if spec.provider_fill_required != expected.provider_fill_required {
        errors.push(format!(
            "`{:?}` provider_fill_required must be {}",
            spec.mode, expected.provider_fill_required
        ));
    }
    if spec.local_fill_engine_required != expected.local_fill_engine_required {
        errors.push(format!(
            "`{:?}` local_fill_engine_required must be {}",
            spec.mode, expected.local_fill_engine_required
        ));
    }
    if spec.local_ledger_required != expected.local_ledger_required {
        errors.push(format!(
            "`{:?}` local_ledger_required must be {}",
            spec.mode, expected.local_ledger_required
        ));
    }
    if spec.provider_account_context_required != expected.provider_account_context_required {
        errors.push(format!(
            "`{:?}` provider_account_context_required must be {}",
            spec.mode, expected.provider_account_context_required
        ));
    }
    if !spec.risk_plane_required {
        errors.push(format!("`{:?}` must require runtime risk plane", spec.mode));
    }

    let declared_events = spec
        .required_events
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    for event in v4_runtime_execution_events() {
        if !declared_events.contains(event) {
            errors.push(format!(
                "`{:?}` must declare runtime execution event `{:?}`",
                spec.mode, event
            ));
        }
    }

    errors
}

pub(in crate::v4) fn required_runtime_trading_modes() -> [RuntimeTradingMode; 4] {
    [
        RuntimeTradingMode::PaperActual,
        RuntimeTradingMode::PaperSimulated,
        RuntimeTradingMode::LiveActual,
        RuntimeTradingMode::LiveSimulated,
    ]
}

fn default_runtime_mode_specs() -> Vec<RuntimeTradingModeSpec> {
    required_runtime_trading_modes()
        .into_iter()
        .map(expected_runtime_mode_spec)
        .collect()
}

fn expected_runtime_mode_spec(mode: RuntimeTradingMode) -> RuntimeTradingModeSpec {
    match mode {
        RuntimeTradingMode::PaperActual => RuntimeTradingModeSpec {
            mode,
            account_domain: RuntimeAccountDomain::Paper,
            settlement_authority: RuntimeSettlementAuthority::ProviderActual,
            execution_event_source: RuntimeExecutionEventSource::ProviderActual,
            provider_order_submission_allowed: true,
            provider_fill_required: true,
            local_fill_engine_required: false,
            local_ledger_required: false,
            provider_account_context_required: true,
            risk_plane_required: true,
            required_events: default_runtime_execution_events(),
        },
        RuntimeTradingMode::PaperSimulated => RuntimeTradingModeSpec {
            mode,
            account_domain: RuntimeAccountDomain::Paper,
            settlement_authority: RuntimeSettlementAuthority::LocalSimulated,
            execution_event_source: RuntimeExecutionEventSource::LocalSimulated,
            provider_order_submission_allowed: false,
            provider_fill_required: false,
            local_fill_engine_required: true,
            local_ledger_required: true,
            provider_account_context_required: false,
            risk_plane_required: true,
            required_events: default_runtime_execution_events(),
        },
        RuntimeTradingMode::LiveActual => RuntimeTradingModeSpec {
            mode,
            account_domain: RuntimeAccountDomain::Live,
            settlement_authority: RuntimeSettlementAuthority::ProviderActual,
            execution_event_source: RuntimeExecutionEventSource::ProviderActual,
            provider_order_submission_allowed: true,
            provider_fill_required: true,
            local_fill_engine_required: false,
            local_ledger_required: false,
            provider_account_context_required: true,
            risk_plane_required: true,
            required_events: default_runtime_execution_events(),
        },
        RuntimeTradingMode::LiveSimulated => RuntimeTradingModeSpec {
            mode,
            account_domain: RuntimeAccountDomain::Live,
            settlement_authority: RuntimeSettlementAuthority::LocalSimulated,
            execution_event_source: RuntimeExecutionEventSource::LocalSimulated,
            provider_order_submission_allowed: false,
            provider_fill_required: false,
            local_fill_engine_required: true,
            local_ledger_required: true,
            provider_account_context_required: true,
            risk_plane_required: true,
            required_events: default_runtime_execution_events(),
        },
    }
}

fn default_runtime_mode_contract_version() -> String {
    V4_RUNTIME_MODE_CONTRACT_VERSION.to_string()
}

fn default_runtime_execution_events() -> Vec<RuntimeExecutionEventKind> {
    v4_runtime_execution_events().to_vec()
}
