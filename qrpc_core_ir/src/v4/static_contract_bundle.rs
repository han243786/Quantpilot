use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use super::{
    ComplexityBudgetContract, ComplexityMetrics, DeveloperLearningPipelineContract,
    PluginGovernanceContract, PluginManifestSpec, QsStateMachineProfile, QsTypeSystemContract,
    ReproducibilityContract, RuntimeModeContract, V4MachineGraphContract, V4VersionManifest,
    VenueCapabilityMatrix, V4_STATIC_CONTRACT_BUNDLE_VERSION,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4StaticContractBundle {
    #[serde(default = "default_static_contract_bundle_version")]
    pub schema_version: String,
    #[serde(default)]
    pub version_manifest: V4VersionManifest,
    #[serde(default)]
    pub qs_profile: QsStateMachineProfile,
    #[serde(default)]
    pub type_system: QsTypeSystemContract,
    #[serde(default)]
    pub runtime_modes: RuntimeModeContract,
    #[serde(default)]
    pub plugin_governance: PluginGovernanceContract,
    #[serde(default)]
    pub reproducibility: ReproducibilityContract,
    #[serde(default)]
    pub complexity_budget: ComplexityBudgetContract,
    #[serde(default)]
    pub learning_pipeline: DeveloperLearningPipelineContract,
    #[serde(default)]
    pub machine_graphs: Vec<V4MachineGraphContract>,
    #[serde(default)]
    pub venue_matrices: Vec<VenueCapabilityMatrix>,
    #[serde(default)]
    pub plugin_manifests: Vec<PluginManifestSpec>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

impl Default for V4StaticContractBundle {
    fn default() -> Self {
        Self {
            schema_version: V4_STATIC_CONTRACT_BUNDLE_VERSION.to_string(),
            version_manifest: V4VersionManifest::default(),
            qs_profile: QsStateMachineProfile::default(),
            type_system: QsTypeSystemContract::default(),
            runtime_modes: RuntimeModeContract::default(),
            plugin_governance: PluginGovernanceContract::default(),
            reproducibility: ReproducibilityContract::default(),
            complexity_budget: ComplexityBudgetContract::default(),
            learning_pipeline: DeveloperLearningPipelineContract::default(),
            machine_graphs: Vec::new(),
            venue_matrices: Vec::new(),
            plugin_manifests: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }
}

impl V4StaticContractBundle {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_STATIC_CONTRACT_BUNDLE_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_STATIC_CONTRACT_BUNDLE_VERSION
            ));
        }
        errors.extend(
            self.version_manifest
                .validate_static_contract()
                .err()
                .unwrap_or_default(),
        );
        errors.extend(
            self.qs_profile
                .validate_static_contract()
                .err()
                .unwrap_or_default(),
        );
        errors.extend(
            self.type_system
                .validate_static_contract()
                .err()
                .unwrap_or_default(),
        );
        errors.extend(
            self.runtime_modes
                .validate_static_contract()
                .err()
                .unwrap_or_default(),
        );
        errors.extend(
            self.plugin_governance
                .validate_static_contract()
                .err()
                .unwrap_or_default(),
        );
        errors.extend(
            self.reproducibility
                .validate_static_contract()
                .err()
                .unwrap_or_default(),
        );
        errors.extend(
            self.complexity_budget
                .validate_static_contract()
                .err()
                .unwrap_or_default(),
        );
        errors.extend(
            self.learning_pipeline
                .validate_static_contract()
                .err()
                .unwrap_or_default(),
        );

        if self.machine_graphs.is_empty() {
            errors
                .push("static contract bundle must include at least one machine graph".to_string());
        }
        if self.venue_matrices.is_empty() {
            errors
                .push("static contract bundle must include at least one venue matrix".to_string());
        }

        for graph in &self.machine_graphs {
            errors.extend(graph.validate_static_contract().err().unwrap_or_default());
            let metrics = ComplexityMetrics::from_machine_graph(
                graph,
                self.runtime_modes.modes.len() as u32,
                self.plugin_manifests.len() as u32,
            );
            errors.extend(
                self.complexity_budget
                    .validate_metrics(&metrics)
                    .err()
                    .unwrap_or_default(),
            );
        }

        for matrix in &self.venue_matrices {
            errors.extend(
                matrix
                    .validate_v4_first_wave_contract()
                    .err()
                    .unwrap_or_default(),
            );
        }

        for manifest in &self.plugin_manifests {
            errors.extend(
                self.plugin_governance
                    .validate_plugin_manifest(manifest, &self.type_system, &self.runtime_modes)
                    .err()
                    .unwrap_or_default(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn default_static_contract_bundle_version() -> String {
    V4_STATIC_CONTRACT_BUNDLE_VERSION.to_string()
}
