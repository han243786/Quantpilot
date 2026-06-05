use serde::{Deserialize, Serialize};

use super::{
    default_machine_contract_version, default_qs_type_system_version, default_true,
    default_venue_capability_matrix_version, V4_MACHINE_CONTRACT_VERSION,
    V4_QS_TYPE_SYSTEM_VERSION, V4_VENUE_CAPABILITY_MATRIX_VERSION, V4_VERSION_MANIFEST_VERSION,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4VersionManifest {
    #[serde(default = "default_version_manifest_version")]
    pub schema_version: String,
    #[serde(default = "default_qs_language_version")]
    pub qs_language_version: String,
    #[serde(default = "default_qs_type_system_version")]
    pub type_schema_version: String,
    #[serde(default = "default_machine_contract_version")]
    pub machine_template_version: String,
    #[serde(default = "default_venue_capability_matrix_version")]
    pub capability_matrix_version: String,
    #[serde(default = "default_true")]
    pub additive_types_are_compatible: bool,
    #[serde(default = "default_true")]
    pub additive_defaulted_fields_are_compatible: bool,
    #[serde(default = "default_true")]
    pub type_tightening_requires_migration: bool,
    #[serde(default = "default_true")]
    pub type_deletion_requires_deprecation_first: bool,
    #[serde(default = "default_true")]
    pub semantic_change_requires_schema_bump: bool,
}

impl Default for V4VersionManifest {
    fn default() -> Self {
        Self {
            schema_version: V4_VERSION_MANIFEST_VERSION.to_string(),
            qs_language_version: default_qs_language_version(),
            type_schema_version: V4_QS_TYPE_SYSTEM_VERSION.to_string(),
            machine_template_version: V4_MACHINE_CONTRACT_VERSION.to_string(),
            capability_matrix_version: V4_VENUE_CAPABILITY_MATRIX_VERSION.to_string(),
            additive_types_are_compatible: true,
            additive_defaulted_fields_are_compatible: true,
            type_tightening_requires_migration: true,
            type_deletion_requires_deprecation_first: true,
            semantic_change_requires_schema_bump: true,
        }
    }
}

impl V4VersionManifest {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_VERSION_MANIFEST_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_VERSION_MANIFEST_VERSION
            ));
        }
        if self.qs_language_version.trim().is_empty() {
            errors.push("qs_language_version is required".to_string());
        }
        if self.type_schema_version != V4_QS_TYPE_SYSTEM_VERSION {
            errors.push(format!(
                "type_schema_version must be `{}`",
                V4_QS_TYPE_SYSTEM_VERSION
            ));
        }
        if self.machine_template_version != V4_MACHINE_CONTRACT_VERSION {
            errors.push(format!(
                "machine_template_version must be `{}`",
                V4_MACHINE_CONTRACT_VERSION
            ));
        }
        if self.capability_matrix_version != V4_VENUE_CAPABILITY_MATRIX_VERSION {
            errors.push(format!(
                "capability_matrix_version must be `{}`",
                V4_VENUE_CAPABILITY_MATRIX_VERSION
            ));
        }
        if !self.additive_types_are_compatible {
            errors.push("additive types must stay compatible".to_string());
        }
        if !self.additive_defaulted_fields_are_compatible {
            errors.push("additive defaulted fields must stay compatible".to_string());
        }
        if !self.type_tightening_requires_migration {
            errors.push("type tightening must require migration".to_string());
        }
        if !self.type_deletion_requires_deprecation_first {
            errors.push("type deletion must require deprecation first".to_string());
        }
        if !self.semantic_change_requires_schema_bump {
            errors.push("semantic changes must require a schema bump".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn default_version_manifest_version() -> String {
    V4_VERSION_MANIFEST_VERSION.to_string()
}

fn default_qs_language_version() -> String {
    "quantpilot/qs-language/v4".to_string()
}
