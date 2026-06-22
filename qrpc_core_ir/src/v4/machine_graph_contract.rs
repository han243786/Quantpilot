mod event_catalog;
mod static_validation;
mod traversal_helpers;

pub use event_catalog::*;
pub(super) use traversal_helpers::{collect_machine_family, machine_nested_depth};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

use super::{
    default_machine_graph_contract_version, default_machine_graph_edge_activation,
    default_risk_plane_min_priority, default_true, MachineGuardDescriptorProjection,
    MachineGuardExecutionReadinessState, MachineTemplateKind, V4MachineContract,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4MachineGraphContract {
    #[serde(default = "default_machine_graph_contract_version")]
    pub schema_version: String,
    pub graph_id: String,
    #[serde(default)]
    pub machines: Vec<V4MachineContract>,
    #[serde(default)]
    pub edges: Vec<MachineGraphEdge>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_catalog: Option<MachineEventCatalog>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_plane: Option<MachineGraphRiskPlane>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MachineGraphEdge {
    pub edge_id: String,
    pub source_machine_id: String,
    pub target_machine_id: String,
    pub event_type: String,
    #[serde(default = "default_machine_graph_edge_activation")]
    pub activation: MachineGraphEdgeActivation,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MachineGraphEdgeActivation {
    Always,
    RuntimeGated,
    MutedWhenUnpulled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MachineGraphRiskPlane {
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub machine_ids: Vec<String>,
    #[serde(default = "default_risk_plane_min_priority")]
    pub min_priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MachineGraphGuardDescriptorProjection {
    pub machine_id: String,
    pub machine_template: MachineTemplateKind,
    pub guard: MachineGuardDescriptorProjection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct MachineGraphGuardDescriptorSummary {
    pub guard_descriptor_count: usize,
    pub guarded_machine_count: usize,
    pub observation_guard_descriptor_count: usize,
    pub decision_guard_descriptor_count: usize,
    pub execution_guard_descriptor_count: usize,
    pub event_source_declared_count: usize,
    pub event_source_missing_count: usize,
    pub read_count: usize,
    pub event_payload_read_count: usize,
    pub machine_memory_read_count: usize,
    pub readonly_runtime_fact_read_count: usize,
    pub parameter_path_count: usize,
    pub guard_parameter_path_count: usize,
    pub timeout_parameter_path_count: usize,
    pub cooldown_parameter_path_count: usize,
    pub threshold_parameter_path_count: usize,
    pub risk_limit_parameter_path_count: usize,
    pub parameter_path_proposal_only_count: usize,
    pub parameter_path_active_strategy_write_enabled_count: usize,
    pub parameter_path_active_strategy_write_disabled_count: usize,
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
    pub condition_evaluation_disabled_fail_closed_count: usize,
    pub policy_declared_count: usize,
    pub timing_policy_declared_count: usize,
    pub timeout_declared_count: usize,
    pub cooldown_declared_count: usize,
    pub fallback_declared_count: usize,
    pub fallback_fail_closed_declared_count: usize,
    pub policy_timing_execution_enabled_count: usize,
    pub policy_timing_execution_disabled_fail_closed_count: usize,
    pub policy_fallback_execution_enabled_count: usize,
    pub policy_fallback_execution_disabled_fail_closed_count: usize,
    pub policy_active_strategy_write_enabled_count: usize,
    pub policy_active_strategy_write_disabled_count: usize,
    pub execution_enabled_count: usize,
    pub execution_disabled_fail_closed_count: usize,
}

impl V4MachineGraphContract {
    pub fn guard_descriptor_projections(&self) -> Vec<MachineGraphGuardDescriptorProjection> {
        let mut all_machines = Vec::new();
        for machine in &self.machines {
            collect_machine_family(machine, &mut all_machines);
        }

        all_machines
            .into_iter()
            .flat_map(|machine| {
                machine
                    .guard_descriptor_projections()
                    .into_iter()
                    .map(move |guard| MachineGraphGuardDescriptorProjection {
                        machine_id: machine.machine_id.clone(),
                        machine_template: machine.template.clone(),
                        guard,
                    })
            })
            .collect()
    }

    pub fn guard_descriptor_summary(&self) -> MachineGraphGuardDescriptorSummary {
        let mut summary = MachineGraphGuardDescriptorSummary::default();
        let mut guarded_machine_ids = BTreeSet::new();
        for projection in self.guard_descriptor_projections() {
            let readiness = &projection.guard.readiness;
            summary.guard_descriptor_count += 1;
            guarded_machine_ids.insert(projection.machine_id.clone());
            match &projection.machine_template {
                MachineTemplateKind::Observation => summary.observation_guard_descriptor_count += 1,
                MachineTemplateKind::Decision => summary.decision_guard_descriptor_count += 1,
                MachineTemplateKind::Execution => summary.execution_guard_descriptor_count += 1,
            }
            if projection.guard.event_source.is_some() {
                summary.event_source_declared_count += 1;
            } else {
                summary.event_source_missing_count += 1;
            }
            summary.read_count += readiness.read_count;
            summary.event_payload_read_count += readiness.event_payload_read_count;
            summary.machine_memory_read_count += readiness.machine_memory_read_count;
            summary.readonly_runtime_fact_read_count += readiness.readonly_runtime_fact_read_count;
            summary.parameter_path_count += readiness.parameter_path_count;
            summary.guard_parameter_path_count += readiness.guard_parameter_path_count;
            summary.timeout_parameter_path_count += readiness.timeout_parameter_path_count;
            summary.cooldown_parameter_path_count += readiness.cooldown_parameter_path_count;
            summary.threshold_parameter_path_count += readiness.threshold_parameter_path_count;
            summary.risk_limit_parameter_path_count += readiness.risk_limit_parameter_path_count;
            for parameter_path in &projection.guard.parameter_path_projections {
                summary.parameter_path_proposal_only_count +=
                    usize::from(parameter_path.proposal_only);
                summary.parameter_path_active_strategy_write_enabled_count +=
                    usize::from(parameter_path.active_strategy_write_enabled);
                summary.parameter_path_active_strategy_write_disabled_count +=
                    usize::from(!parameter_path.active_strategy_write_enabled);
            }
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
            for condition in &projection.guard.condition_projections {
                summary.condition_evaluation_enabled_count +=
                    usize::from(condition.evaluation_enabled);
                summary.condition_evaluation_disabled_fail_closed_count += usize::from(
                    !condition.evaluation_enabled
                        && condition.evaluation_blocker_code
                            == MachineGuardExecutionReadinessState::DisabledFailClosed
                                .blocker_code(),
                );
            }
            summary.policy_declared_count += usize::from(readiness.policy_declared);
            summary.timing_policy_declared_count += usize::from(readiness.timing_policy_declared);
            summary.timeout_declared_count += usize::from(readiness.timeout_declared);
            summary.cooldown_declared_count += usize::from(readiness.cooldown_declared);
            summary.fallback_declared_count += usize::from(readiness.fallback_declared);
            summary.fallback_fail_closed_declared_count +=
                usize::from(readiness.fallback_fail_closed_declared);
            if let Some(policy) = &projection.guard.policy_projection {
                summary.policy_timing_execution_enabled_count +=
                    usize::from(policy.timing_execution_enabled);
                summary.policy_timing_execution_disabled_fail_closed_count += usize::from(
                    !policy.timing_execution_enabled
                        && policy.timing_policy_declared
                        && policy.execution_blocker_code
                            == MachineGuardExecutionReadinessState::DisabledFailClosed
                                .blocker_code(),
                );
                summary.policy_fallback_execution_enabled_count +=
                    usize::from(policy.fallback_execution_enabled);
                summary.policy_fallback_execution_disabled_fail_closed_count += usize::from(
                    !policy.fallback_execution_enabled
                        && policy.fallback_declared
                        && policy.execution_blocker_code
                            == MachineGuardExecutionReadinessState::DisabledFailClosed
                                .blocker_code(),
                );
                summary.policy_active_strategy_write_enabled_count +=
                    usize::from(policy.active_strategy_write_enabled);
                summary.policy_active_strategy_write_disabled_count +=
                    usize::from(!policy.active_strategy_write_enabled);
            }
            summary.execution_enabled_count += usize::from(readiness.execution_enabled);
            summary.execution_disabled_fail_closed_count += usize::from(
                readiness.execution_state
                    == MachineGuardExecutionReadinessState::DisabledFailClosed,
            );
        }
        summary.guarded_machine_count = guarded_machine_ids.len();
        summary
    }
}
