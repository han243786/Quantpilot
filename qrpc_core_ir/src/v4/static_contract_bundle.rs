use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

mod static_validation;

use super::{
    ComplexityBudgetContract, DeveloperLearningPipelineContract,
    MachineGraphGuardDescriptorProjection, MachineGuardExecutionReadinessState,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StaticContractBundleGuardDescriptorProjection {
    pub graph_id: String,
    pub guard: MachineGraphGuardDescriptorProjection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct StaticContractBundleGuardDescriptorSummary {
    pub guard_descriptor_count: usize,
    pub read_guard_descriptor_count: usize,
    pub read_count: usize,
    pub event_payload_read_count: usize,
    pub machine_memory_read_count: usize,
    pub readonly_runtime_fact_read_count: usize,
    pub parameterized_guard_descriptor_count: usize,
    pub parameter_path_count: usize,
    pub guard_parameter_path_count: usize,
    pub timeout_parameter_path_count: usize,
    pub cooldown_parameter_path_count: usize,
    pub threshold_parameter_path_count: usize,
    pub risk_limit_parameter_path_count: usize,
    pub parameter_path_proposal_only_count: usize,
    pub proposal_only_guard_descriptor_count: usize,
    pub parameter_path_active_strategy_write_enabled_count: usize,
    pub parameter_path_active_strategy_write_disabled_count: usize,
    pub conditional_guard_descriptor_count: usize,
    pub condition_count: usize,
    pub equal_condition_count: usize,
    pub not_equal_condition_count: usize,
    pub greater_than_condition_count: usize,
    pub greater_than_or_equal_condition_count: usize,
    pub less_than_condition_count: usize,
    pub less_than_or_equal_condition_count: usize,
    pub condition_event_payload_read_count: usize,
    pub condition_machine_memory_read_count: usize,
    pub condition_readonly_runtime_fact_read_count: usize,
    pub condition_guard_parameter_path_count: usize,
    pub condition_timeout_parameter_path_count: usize,
    pub condition_cooldown_parameter_path_count: usize,
    pub condition_threshold_parameter_path_count: usize,
    pub condition_risk_limit_parameter_path_count: usize,
    pub condition_evaluation_enabled_count: usize,
    pub condition_evaluation_disabled_fail_closed_guard_descriptor_count: usize,
    pub condition_evaluation_disabled_fail_closed_count: usize,
    pub policy_declared_count: usize,
    pub timing_policy_declared_count: usize,
    pub timeout_declared_count: usize,
    pub cooldown_declared_count: usize,
    pub fallback_declared_count: usize,
    pub fallback_fail_closed_declared_count: usize,
    pub policy_timing_execution_enabled_count: usize,
    pub policy_execution_disabled_fail_closed_guard_descriptor_count: usize,
    pub policy_timing_execution_disabled_fail_closed_count: usize,
    pub policy_fallback_execution_enabled_count: usize,
    pub policy_fallback_execution_disabled_fail_closed_count: usize,
    pub policy_active_strategy_write_enabled_count: usize,
    pub policy_active_strategy_write_disabled_count: usize,
    pub active_strategy_write_disabled_guard_descriptor_count: usize,
    pub execution_enabled_count: usize,
    pub execution_disabled_fail_closed_count: usize,
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

    pub fn guard_descriptor_summary(&self) -> StaticContractBundleGuardDescriptorSummary {
        let mut summary = StaticContractBundleGuardDescriptorSummary::default();
        for projection in self.guard_descriptor_projections() {
            let readiness = &projection.guard.guard.readiness;
            summary.guard_descriptor_count += 1;
            summary.read_guard_descriptor_count += usize::from(readiness.read_count > 0);
            summary.read_count += readiness.read_count;
            summary.event_payload_read_count += readiness.event_payload_read_count;
            summary.machine_memory_read_count += readiness.machine_memory_read_count;
            summary.readonly_runtime_fact_read_count += readiness.readonly_runtime_fact_read_count;
            summary.parameterized_guard_descriptor_count +=
                usize::from(readiness.parameter_path_count > 0);
            summary.parameter_path_count += readiness.parameter_path_count;
            summary.guard_parameter_path_count += readiness.guard_parameter_path_count;
            summary.timeout_parameter_path_count += readiness.timeout_parameter_path_count;
            summary.cooldown_parameter_path_count += readiness.cooldown_parameter_path_count;
            summary.threshold_parameter_path_count += readiness.threshold_parameter_path_count;
            summary.risk_limit_parameter_path_count += readiness.risk_limit_parameter_path_count;
            let mut has_proposal_only_parameter_surface = false;
            let mut has_active_strategy_write_disabled_surface = false;
            for parameter_path in &projection.guard.guard.parameter_path_projections {
                summary.parameter_path_proposal_only_count +=
                    usize::from(parameter_path.proposal_only);
                has_proposal_only_parameter_surface |= parameter_path.proposal_only;
                summary.parameter_path_active_strategy_write_enabled_count +=
                    usize::from(parameter_path.active_strategy_write_enabled);
                summary.parameter_path_active_strategy_write_disabled_count +=
                    usize::from(!parameter_path.active_strategy_write_enabled);
                has_active_strategy_write_disabled_surface |=
                    !parameter_path.active_strategy_write_enabled;
            }
            summary.proposal_only_guard_descriptor_count +=
                usize::from(has_proposal_only_parameter_surface);
            summary.conditional_guard_descriptor_count +=
                usize::from(readiness.condition_count > 0);
            summary.condition_count += readiness.condition_count;
            summary.equal_condition_count += readiness.equal_condition_count;
            summary.not_equal_condition_count += readiness.not_equal_condition_count;
            summary.greater_than_condition_count += readiness.greater_than_condition_count;
            summary.greater_than_or_equal_condition_count +=
                readiness.greater_than_or_equal_condition_count;
            summary.less_than_condition_count += readiness.less_than_condition_count;
            summary.less_than_or_equal_condition_count +=
                readiness.less_than_or_equal_condition_count;
            summary.condition_event_payload_read_count +=
                readiness.condition_event_payload_read_count;
            summary.condition_machine_memory_read_count +=
                readiness.condition_machine_memory_read_count;
            summary.condition_readonly_runtime_fact_read_count +=
                readiness.condition_readonly_runtime_fact_read_count;
            summary.condition_guard_parameter_path_count +=
                readiness.condition_guard_parameter_path_count;
            summary.condition_timeout_parameter_path_count +=
                readiness.condition_timeout_parameter_path_count;
            summary.condition_cooldown_parameter_path_count +=
                readiness.condition_cooldown_parameter_path_count;
            summary.condition_threshold_parameter_path_count +=
                readiness.condition_threshold_parameter_path_count;
            summary.condition_risk_limit_parameter_path_count +=
                readiness.condition_risk_limit_parameter_path_count;
            let mut has_disabled_fail_closed_condition = false;
            for condition in &projection.guard.guard.condition_projections {
                let is_disabled_fail_closed_condition = !condition.evaluation_enabled
                    && condition.evaluation_blocker_code
                        == MachineGuardExecutionReadinessState::DisabledFailClosed.blocker_code();
                summary.condition_evaluation_enabled_count +=
                    usize::from(condition.evaluation_enabled);
                summary.condition_evaluation_disabled_fail_closed_count +=
                    usize::from(is_disabled_fail_closed_condition);
                has_disabled_fail_closed_condition |= is_disabled_fail_closed_condition;
            }
            summary.condition_evaluation_disabled_fail_closed_guard_descriptor_count +=
                usize::from(has_disabled_fail_closed_condition);
            summary.policy_declared_count += usize::from(readiness.policy_declared);
            summary.timing_policy_declared_count += usize::from(readiness.timing_policy_declared);
            summary.timeout_declared_count += usize::from(readiness.timeout_declared);
            summary.cooldown_declared_count += usize::from(readiness.cooldown_declared);
            summary.fallback_declared_count += usize::from(readiness.fallback_declared);
            summary.fallback_fail_closed_declared_count +=
                usize::from(readiness.fallback_fail_closed_declared);
            if let Some(policy) = &projection.guard.guard.policy_projection {
                summary.policy_timing_execution_enabled_count +=
                    usize::from(policy.timing_execution_enabled);
                let timing_execution_disabled_fail_closed = !policy.timing_execution_enabled
                    && policy.timing_policy_declared
                    && policy.execution_blocker_code
                        == MachineGuardExecutionReadinessState::DisabledFailClosed.blocker_code();
                summary.policy_timing_execution_disabled_fail_closed_count +=
                    usize::from(timing_execution_disabled_fail_closed);
                summary.policy_fallback_execution_enabled_count +=
                    usize::from(policy.fallback_execution_enabled);
                let fallback_execution_disabled_fail_closed = !policy.fallback_execution_enabled
                    && policy.fallback_declared
                    && policy.execution_blocker_code
                        == MachineGuardExecutionReadinessState::DisabledFailClosed.blocker_code();
                summary.policy_fallback_execution_disabled_fail_closed_count +=
                    usize::from(fallback_execution_disabled_fail_closed);
                summary.policy_execution_disabled_fail_closed_guard_descriptor_count += usize::from(
                    timing_execution_disabled_fail_closed
                        || fallback_execution_disabled_fail_closed,
                );
                summary.policy_active_strategy_write_enabled_count +=
                    usize::from(policy.active_strategy_write_enabled);
                summary.policy_active_strategy_write_disabled_count +=
                    usize::from(!policy.active_strategy_write_enabled);
                has_active_strategy_write_disabled_surface |= !policy.active_strategy_write_enabled;
            }
            summary.active_strategy_write_disabled_guard_descriptor_count +=
                usize::from(has_active_strategy_write_disabled_surface);
            summary.execution_enabled_count += usize::from(readiness.execution_enabled);
            summary.execution_disabled_fail_closed_count += usize::from(
                readiness.execution_state
                    == MachineGuardExecutionReadinessState::DisabledFailClosed,
            );
        }
        summary
    }
}

fn default_static_contract_bundle_version() -> String {
    V4_STATIC_CONTRACT_BUNDLE_VERSION.to_string()
}
