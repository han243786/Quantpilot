mod event_catalog;
mod static_validation;
mod traversal_helpers;

pub use event_catalog::*;
pub(super) use traversal_helpers::{collect_machine_family, machine_nested_depth};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use super::{
    default_machine_graph_contract_version, default_machine_graph_edge_activation,
    default_risk_plane_min_priority, default_true, V4MachineContract,
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
