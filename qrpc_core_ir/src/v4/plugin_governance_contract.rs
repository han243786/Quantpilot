use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::{
    default_true, required_runtime_trading_modes, v4_first_wave_execution_capabilities, QsTypeRef,
    QsTypeSystemContract, RuntimeModeContract, VenueCapabilityMatrix, V4_PLUGIN_GOVERNANCE_VERSION,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginGovernanceContract {
    #[serde(default = "default_plugin_governance_version")]
    pub schema_version: String,
    #[serde(default = "default_plugin_kinds")]
    pub allowed_kinds: Vec<PluginKind>,
    #[serde(default = "default_plugin_required_fields")]
    pub required_fields: Vec<PluginManifestField>,
    #[serde(default = "default_true")]
    pub qs_declares_capabilities_only: bool,
    #[serde(default = "default_true")]
    pub real_order_requires_venue_plugin_and_risk_plane: bool,
    #[serde(default = "default_true")]
    pub pure_plugins_must_be_deterministic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginManifestSpec {
    pub plugin_id: String,
    pub name: String,
    pub version: String,
    pub kind: PluginKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<QsTypeRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<QsTypeRef>,
    #[serde(default)]
    pub deterministic: bool,
    pub side_effect: PluginSideEffect,
    pub runtime_permission: PluginRuntimePermission,
    pub network_permission: PluginNetworkPermission,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_matrix: Option<VenueCapabilityMatrix>,
    pub test_fixture_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PluginKind {
    Pure,
    Runtime,
    Venue,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PluginManifestField {
    Name,
    Version,
    InputSchema,
    OutputSchema,
    Deterministic,
    SideEffect,
    RuntimePermission,
    NetworkPermission,
    CapabilityMatrix,
    TestFixture,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginSideEffect {
    None,
    LocalRuntimeState,
    ProviderNetwork,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginRuntimePermission {
    None,
    LocalSimulation,
    RuntimeState,
    VenueAdapter,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginNetworkPermission {
    None,
    ProviderOnly,
}

impl Default for PluginGovernanceContract {
    fn default() -> Self {
        Self {
            schema_version: V4_PLUGIN_GOVERNANCE_VERSION.to_string(),
            allowed_kinds: default_plugin_kinds(),
            required_fields: default_plugin_required_fields(),
            qs_declares_capabilities_only: true,
            real_order_requires_venue_plugin_and_risk_plane: true,
            pure_plugins_must_be_deterministic: true,
        }
    }
}

impl PluginGovernanceContract {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_PLUGIN_GOVERNANCE_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_PLUGIN_GOVERNANCE_VERSION
            ));
        }

        let allowed = self.allowed_kinds.iter().copied().collect::<BTreeSet<_>>();
        for kind in [PluginKind::Pure, PluginKind::Runtime, PluginKind::Venue] {
            if !allowed.contains(&kind) {
                errors.push(format!("plugin governance must allow `{:?}` plugins", kind));
            }
        }

        let fields = self
            .required_fields
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        for field in default_plugin_required_fields() {
            if !fields.contains(&field) {
                errors.push(format!(
                    "plugin governance must require manifest field `{:?}`",
                    field
                ));
            }
        }

        if !self.qs_declares_capabilities_only {
            errors.push("QS must declare capabilities only; plugins implement them".to_string());
        }
        if !self.real_order_requires_venue_plugin_and_risk_plane {
            errors.push(
                "real order paths must require a venue plugin and runtime risk plane".to_string(),
            );
        }
        if !self.pure_plugins_must_be_deterministic {
            errors.push("pure plugins must be deterministic".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn validate_plugin_manifest(
        &self,
        manifest: &PluginManifestSpec,
        type_system: &QsTypeSystemContract,
        runtime_modes: &RuntimeModeContract,
    ) -> Result<(), Vec<String>> {
        let mut errors = self.validate_static_contract().err().unwrap_or_default();

        if manifest.plugin_id.trim().is_empty() {
            errors.push("plugin_id is required".to_string());
        }
        if manifest.name.trim().is_empty() {
            errors.push("plugin name is required".to_string());
        }
        if manifest.version.trim().is_empty() {
            errors.push("plugin version is required".to_string());
        }
        if manifest.test_fixture_id.trim().is_empty() {
            errors.push("plugin test_fixture_id is required".to_string());
        }
        if !self.allowed_kinds.contains(&manifest.kind) {
            errors.push(format!("plugin kind `{:?}` is not allowed", manifest.kind));
        }
        match &manifest.input_schema {
            Some(type_ref) => {
                errors.extend(
                    type_system
                        .validate_type_ref(type_ref)
                        .err()
                        .unwrap_or_default(),
                );
            }
            None => errors.push("plugin input_schema is required".to_string()),
        }
        match &manifest.output_schema {
            Some(type_ref) => {
                errors.extend(
                    type_system
                        .validate_type_ref(type_ref)
                        .err()
                        .unwrap_or_default(),
                );
            }
            None => errors.push("plugin output_schema is required".to_string()),
        }

        match manifest.kind {
            PluginKind::Pure => {
                if self.pure_plugins_must_be_deterministic && !manifest.deterministic {
                    errors.push("pure plugins must be deterministic".to_string());
                }
                if !matches!(manifest.side_effect, PluginSideEffect::None) {
                    errors.push("pure plugins must not declare side effects".to_string());
                }
                if !matches!(manifest.runtime_permission, PluginRuntimePermission::None) {
                    errors.push("pure plugins must not require runtime permission".to_string());
                }
                if !matches!(manifest.network_permission, PluginNetworkPermission::None) {
                    errors.push("pure plugins must not require network permission".to_string());
                }
                if manifest.capability_matrix.is_some() {
                    errors.push(
                        "pure plugins must not declare a venue capability matrix".to_string(),
                    );
                }
            }
            PluginKind::Runtime => {
                if matches!(
                    manifest.network_permission,
                    PluginNetworkPermission::ProviderOnly
                ) {
                    errors.push("runtime plugins must not access provider network".to_string());
                }
                if !matches!(
                    manifest.runtime_permission,
                    PluginRuntimePermission::LocalSimulation
                        | PluginRuntimePermission::RuntimeState
                ) {
                    errors.push(
                        "runtime plugins must declare local simulation or runtime state permission"
                            .to_string(),
                    );
                }
            }
            PluginKind::Venue => {
                if !matches!(manifest.side_effect, PluginSideEffect::ProviderNetwork) {
                    errors.push(
                        "venue plugins must declare provider network side effects".to_string(),
                    );
                }
                if !matches!(
                    manifest.runtime_permission,
                    PluginRuntimePermission::VenueAdapter
                ) {
                    errors.push(
                        "venue plugins must require venue_adapter runtime permission".to_string(),
                    );
                }
                if !matches!(
                    manifest.network_permission,
                    PluginNetworkPermission::ProviderOnly
                ) {
                    errors.push(
                        "venue plugins must use provider_only network permission".to_string(),
                    );
                }
                match &manifest.capability_matrix {
                    Some(matrix) => {
                        errors.extend(
                            matrix
                                .validate_v4_first_wave_contract()
                                .err()
                                .unwrap_or_default(),
                        );
                        for capability in v4_first_wave_execution_capabilities() {
                            for mode in required_runtime_trading_modes() {
                                let _ = matrix.require_supported_for_mode(
                                    capability,
                                    mode,
                                    runtime_modes,
                                );
                            }
                        }
                    }
                    None => errors.push("venue plugins must declare capability_matrix".to_string()),
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn default_plugin_governance_version() -> String {
    V4_PLUGIN_GOVERNANCE_VERSION.to_string()
}

fn default_plugin_kinds() -> Vec<PluginKind> {
    vec![PluginKind::Pure, PluginKind::Runtime, PluginKind::Venue]
}

fn default_plugin_required_fields() -> Vec<PluginManifestField> {
    vec![
        PluginManifestField::Name,
        PluginManifestField::Version,
        PluginManifestField::InputSchema,
        PluginManifestField::OutputSchema,
        PluginManifestField::Deterministic,
        PluginManifestField::SideEffect,
        PluginManifestField::RuntimePermission,
        PluginManifestField::NetworkPermission,
        PluginManifestField::CapabilityMatrix,
        PluginManifestField::TestFixture,
    ]
}
