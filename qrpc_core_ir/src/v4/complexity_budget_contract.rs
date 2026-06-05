use serde::{Deserialize, Serialize};

use super::{
    collect_machine_family, machine_nested_depth, MachineTemplateKind, V4MachineGraphContract,
    V4_COMPLEXITY_BUDGET_CONTRACT_VERSION,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComplexityBudgetContract {
    #[serde(default = "default_complexity_budget_contract_version")]
    pub schema_version: String,
    pub max_state_count: u32,
    pub max_transition_count: u32,
    pub max_memory_field_count: u32,
    #[serde(default = "default_max_nested_machine_depth")]
    pub max_nested_machine_depth: u32,
    #[serde(default = "default_max_event_processing_path_count")]
    pub max_event_processing_path_count: u32,
    pub max_plugin_call_count: u32,
    pub max_mode_count: u32,
    pub max_stale_dependency_count: u32,
    pub max_estimated_order_paths: u32,
    pub max_event_rate_estimate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComplexityMetrics {
    pub state_count: u32,
    pub transition_count: u32,
    pub memory_field_count: u32,
    #[serde(default)]
    pub nested_machine_depth: u32,
    #[serde(default)]
    pub event_processing_path_count: u32,
    pub plugin_call_count: u32,
    pub mode_count: u32,
    pub stale_dependency_count: u32,
    pub estimated_order_paths: u32,
    pub event_rate_estimate: u32,
}

impl Default for ComplexityBudgetContract {
    fn default() -> Self {
        Self {
            schema_version: V4_COMPLEXITY_BUDGET_CONTRACT_VERSION.to_string(),
            max_state_count: 1_024,
            max_transition_count: 2_048,
            max_memory_field_count: 1_024,
            max_nested_machine_depth: 2,
            max_event_processing_path_count: 4_096,
            max_plugin_call_count: 256,
            max_mode_count: 4,
            max_stale_dependency_count: 128,
            max_estimated_order_paths: 512,
            max_event_rate_estimate: 100_000,
        }
    }
}

impl ComplexityBudgetContract {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_COMPLEXITY_BUDGET_CONTRACT_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_COMPLEXITY_BUDGET_CONTRACT_VERSION
            ));
        }
        for (name, value) in [
            ("max_state_count", self.max_state_count),
            ("max_transition_count", self.max_transition_count),
            ("max_memory_field_count", self.max_memory_field_count),
            ("max_nested_machine_depth", self.max_nested_machine_depth),
            (
                "max_event_processing_path_count",
                self.max_event_processing_path_count,
            ),
            ("max_plugin_call_count", self.max_plugin_call_count),
            ("max_mode_count", self.max_mode_count),
            (
                "max_stale_dependency_count",
                self.max_stale_dependency_count,
            ),
            ("max_estimated_order_paths", self.max_estimated_order_paths),
            ("max_event_rate_estimate", self.max_event_rate_estimate),
        ] {
            if value == 0 {
                errors.push(format!("{name} must be greater than 0"));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn validate_metrics(&self, metrics: &ComplexityMetrics) -> Result<(), Vec<String>> {
        let mut errors = self.validate_static_contract().err().unwrap_or_default();

        for (name, value, limit) in [
            ("state_count", metrics.state_count, self.max_state_count),
            (
                "transition_count",
                metrics.transition_count,
                self.max_transition_count,
            ),
            (
                "memory_field_count",
                metrics.memory_field_count,
                self.max_memory_field_count,
            ),
            (
                "nested_machine_depth",
                metrics.nested_machine_depth,
                self.max_nested_machine_depth,
            ),
            (
                "event_processing_path_count",
                metrics.event_processing_path_count,
                self.max_event_processing_path_count,
            ),
            (
                "plugin_call_count",
                metrics.plugin_call_count,
                self.max_plugin_call_count,
            ),
            ("mode_count", metrics.mode_count, self.max_mode_count),
            (
                "stale_dependency_count",
                metrics.stale_dependency_count,
                self.max_stale_dependency_count,
            ),
            (
                "estimated_order_paths",
                metrics.estimated_order_paths,
                self.max_estimated_order_paths,
            ),
            (
                "event_rate_estimate",
                metrics.event_rate_estimate,
                self.max_event_rate_estimate,
            ),
        ] {
            if value > limit {
                errors.push(format!("{name} {value} exceeds budget {limit}"));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl ComplexityMetrics {
    pub fn from_machine_graph(
        graph: &V4MachineGraphContract,
        mode_count: u32,
        plugin_call_count: u32,
    ) -> Self {
        let mut all_machines = Vec::new();
        for machine in &graph.machines {
            collect_machine_family(machine, &mut all_machines);
        }
        let state_count: u32 = all_machines
            .iter()
            .map(|machine| machine.states.len() as u32)
            .sum();
        let transition_count: u32 = all_machines
            .iter()
            .map(|machine| machine.transitions.len() as u32)
            .sum();
        let memory_field_count: u32 = all_machines
            .iter()
            .map(|machine| machine.memory.len() as u32)
            .sum();
        let nested_machine_depth = graph
            .machines
            .iter()
            .map(machine_nested_depth)
            .max()
            .unwrap_or(0);
        let event_rate_estimate = graph
            .event_catalog
            .as_ref()
            .map(|catalog| catalog.events.len() as u32)
            .unwrap_or_default()
            .saturating_mul(1_000);
        let estimated_order_paths = all_machines
            .iter()
            .filter(|machine| matches!(machine.template, MachineTemplateKind::Execution))
            .count() as u32;
        let event_processing_path_count =
            transition_count.saturating_mul(nested_machine_depth.max(1));

        Self {
            state_count,
            transition_count,
            memory_field_count,
            nested_machine_depth,
            event_processing_path_count,
            plugin_call_count,
            mode_count,
            stale_dependency_count: 0,
            estimated_order_paths,
            event_rate_estimate,
        }
    }
}

fn default_complexity_budget_contract_version() -> String {
    V4_COMPLEXITY_BUDGET_CONTRACT_VERSION.to_string()
}

fn default_max_nested_machine_depth() -> u32 {
    2
}

fn default_max_event_processing_path_count() -> u32 {
    4_096
}
