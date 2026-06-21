use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

mod static_validation;

use super::{
    ComplexityBudgetContract, DeveloperLearningPipelineContract,
    MachineGraphGuardDescriptorProjection, PluginGovernanceContract, PluginManifestSpec,
    QsStateMachineProfile, QsTypeSystemContract, ReproducibilityContract, RuntimeModeContract,
    V4MachineGraphContract, V4VersionManifest, VenueCapabilityMatrix,
    V4_STATIC_CONTRACT_BUNDLE_VERSION,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StaticContractBundleGuardDescriptorProjection {
    pub graph_id: String,
    pub guard: MachineGraphGuardDescriptorProjection,
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
    pub fn guard_descriptor_projections(
        &self,
    ) -> Vec<StaticContractBundleGuardDescriptorProjection> {
        self.machine_graphs
            .iter()
            .flat_map(|graph| {
                graph
                    .guard_descriptor_projections()
                    .into_iter()
                    .map(move |guard| StaticContractBundleGuardDescriptorProjection {
                        graph_id: graph.graph_id.clone(),
                        guard,
                    })
            })
            .collect()
    }
}

fn default_static_contract_bundle_version() -> String {
    V4_STATIC_CONTRACT_BUNDLE_VERSION.to_string()
}
