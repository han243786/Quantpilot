use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

use super::{
    RuntimeModeContract, RuntimeSettlementAuthority, RuntimeTradingMode,
    V4_VENUE_CAPABILITY_MATRIX_VERSION,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VenueCapabilityMatrix {
    #[serde(default = "default_venue_capability_matrix_version")]
    pub schema_version: String,
    pub venue_id: String,
    #[serde(default)]
    pub capabilities: Vec<VenueCapability>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionCapabilityKind {
    Market,
    Limit,
    PostOnly,
    StopMarket,
    StopLimit,
    TakeProfitMarket,
    TakeProfitLimit,
    Ioc,
    Fok,
    OcoBracket,
    TrailingStop,
    ReduceOnly,
    CloseOnly,
    OpenLong,
    CloseLong,
    OpenShort,
    CloseShort,
    OneWayPositionMode,
    HedgePositionMode,
    Gtc,
    Day,
    Gtd,
    ClientOrderId,
    CancelReplaceAmend,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilitySupportSource {
    ProviderNative,
    RuntimeSimulated,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VenueCapability {
    pub capability: ExecutionCapabilityKind,
    pub source: CapabilitySupportSource,
    #[serde(default)]
    pub supported_modes: Vec<RuntimeTradingMode>,
    #[serde(default)]
    pub constraints: BTreeMap<String, Value>,
}

pub const V4_FIRST_WAVE_EXECUTION_CAPABILITIES: [ExecutionCapabilityKind; 24] = [
    ExecutionCapabilityKind::Market,
    ExecutionCapabilityKind::Limit,
    ExecutionCapabilityKind::PostOnly,
    ExecutionCapabilityKind::StopMarket,
    ExecutionCapabilityKind::StopLimit,
    ExecutionCapabilityKind::TakeProfitMarket,
    ExecutionCapabilityKind::TakeProfitLimit,
    ExecutionCapabilityKind::Ioc,
    ExecutionCapabilityKind::Fok,
    ExecutionCapabilityKind::OcoBracket,
    ExecutionCapabilityKind::TrailingStop,
    ExecutionCapabilityKind::ReduceOnly,
    ExecutionCapabilityKind::CloseOnly,
    ExecutionCapabilityKind::OpenLong,
    ExecutionCapabilityKind::CloseLong,
    ExecutionCapabilityKind::OpenShort,
    ExecutionCapabilityKind::CloseShort,
    ExecutionCapabilityKind::OneWayPositionMode,
    ExecutionCapabilityKind::HedgePositionMode,
    ExecutionCapabilityKind::Gtc,
    ExecutionCapabilityKind::Day,
    ExecutionCapabilityKind::Gtd,
    ExecutionCapabilityKind::ClientOrderId,
    ExecutionCapabilityKind::CancelReplaceAmend,
];

pub fn v4_first_wave_execution_capabilities() -> &'static [ExecutionCapabilityKind] {
    &V4_FIRST_WAVE_EXECUTION_CAPABILITIES
}

pub fn unsupported_v4_first_wave_matrix(venue_id: impl Into<String>) -> VenueCapabilityMatrix {
    VenueCapabilityMatrix {
        schema_version: V4_VENUE_CAPABILITY_MATRIX_VERSION.to_string(),
        venue_id: venue_id.into(),
        capabilities: v4_first_wave_execution_capabilities()
            .iter()
            .copied()
            .map(|capability| VenueCapability {
                capability,
                source: CapabilitySupportSource::Unsupported,
                supported_modes: Vec::new(),
                constraints: BTreeMap::new(),
            })
            .collect(),
        metadata: BTreeMap::new(),
    }
}

impl VenueCapabilityMatrix {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_VENUE_CAPABILITY_MATRIX_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_VENUE_CAPABILITY_MATRIX_VERSION
            ));
        }
        if self.venue_id.trim().is_empty() {
            errors.push("venue_id is required".to_string());
        }

        let mut seen = BTreeSet::new();
        for capability in &self.capabilities {
            if !seen.insert(&capability.capability) {
                errors.push(format!(
                    "duplicate execution capability `{:?}`",
                    capability.capability
                ));
            }
            if !matches!(capability.source, CapabilitySupportSource::Unsupported)
                && capability.supported_modes.is_empty()
            {
                errors.push(format!(
                    "capability `{:?}` needs at least one supported mode",
                    capability.capability
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn capability_entry(
        &self,
        capability: &ExecutionCapabilityKind,
    ) -> Option<&VenueCapability> {
        self.capabilities
            .iter()
            .find(|entry| &entry.capability == capability)
    }

    pub fn support_source(&self, capability: &ExecutionCapabilityKind) -> CapabilitySupportSource {
        self.capability_entry(capability)
            .map(|entry| entry.source)
            .unwrap_or(CapabilitySupportSource::Unsupported)
    }

    pub fn require_supported(
        &self,
        capability: &ExecutionCapabilityKind,
    ) -> Result<CapabilitySupportSource, String> {
        let source = self.support_source(capability);
        if matches!(source, CapabilitySupportSource::Unsupported) {
            Err(format!(
                "execution capability `{:?}` is unsupported for venue `{}`",
                capability, self.venue_id
            ))
        } else {
            Ok(source)
        }
    }

    pub fn require_supported_for_mode(
        &self,
        capability: &ExecutionCapabilityKind,
        mode: RuntimeTradingMode,
        runtime_mode_contract: &RuntimeModeContract,
    ) -> Result<CapabilitySupportSource, String> {
        let mode_spec = runtime_mode_contract
            .mode_spec(mode)
            .ok_or_else(|| format!("runtime mode `{:?}` is not declared", mode))?;
        let entry = self.capability_entry(capability).ok_or_else(|| {
            format!(
                "execution capability `{:?}` is unsupported for venue `{}`",
                capability, self.venue_id
            )
        })?;

        if matches!(entry.source, CapabilitySupportSource::Unsupported) {
            return Err(format!(
                "execution capability `{:?}` is unsupported for venue `{}`",
                capability, self.venue_id
            ));
        }
        if !entry.supported_modes.contains(&mode) {
            return Err(format!(
                "execution capability `{:?}` is not supported in runtime mode `{:?}` for venue `{}`",
                capability, mode, self.venue_id
            ));
        }

        match mode_spec.settlement_authority {
            RuntimeSettlementAuthority::ProviderActual => {
                if !matches!(entry.source, CapabilitySupportSource::ProviderNative) {
                    return Err(format!(
                        "provider_actual mode `{:?}` requires provider_native support for `{:?}`",
                        mode, capability
                    ));
                }
            }
            RuntimeSettlementAuthority::LocalSimulated => {
                if !matches!(entry.source, CapabilitySupportSource::RuntimeSimulated) {
                    return Err(format!(
                        "local_simulated mode `{:?}` requires runtime_simulated support for `{:?}`",
                        mode, capability
                    ));
                }
            }
        }

        Ok(entry.source)
    }

    pub fn validate_required_capability_sources(
        &self,
        required: &[ExecutionCapabilityKind],
    ) -> Result<(), Vec<String>> {
        let mut errors = self.validate_static_contract().err().unwrap_or_default();
        let declared = self
            .capabilities
            .iter()
            .map(|entry| entry.capability)
            .collect::<BTreeSet<_>>();

        for capability in required {
            if !declared.contains(capability) {
                errors.push(format!(
                    "required execution capability `{:?}` must be explicitly marked as provider_native, runtime_simulated, or unsupported",
                    capability
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn validate_v4_first_wave_contract(&self) -> Result<(), Vec<String>> {
        self.validate_required_capability_sources(v4_first_wave_execution_capabilities())
    }
}

pub(in crate::v4) fn default_venue_capability_matrix_version() -> String {
    V4_VENUE_CAPABILITY_MATRIX_VERSION.to_string()
}
