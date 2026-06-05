use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

use crate::v4::{
    default_machine_event_catalog_version, default_true, V4_MACHINE_EVENT_CATALOG_VERSION,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MachineEventCatalog {
    #[serde(default = "default_machine_event_catalog_version")]
    pub schema_version: String,
    #[serde(default)]
    pub events: Vec<MachineEventTypeSpec>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MachineEventTypeSpec {
    pub event_type: String,
    pub source_kind: MachineEventSourceKind,
    pub scope: MachineEventScope,
    #[serde(default)]
    pub payload_fields: Vec<MachineEventPayloadField>,
    #[serde(default)]
    pub allowed_emitters: Vec<String>,
    #[serde(default)]
    pub allowed_consumers: Vec<String>,
    #[serde(default = "default_true")]
    pub replayable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MachineEventPayloadField {
    pub name: String,
    pub type_name: String,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub nullable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MachineEventSourceKind {
    MarketData,
    Machine,
    RiskPlane,
    VenueProvider,
    Runtime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MachineEventScope {
    MachineInternal,
    Graph,
    Runtime,
    Venue,
}

impl MachineEventCatalog {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_MACHINE_EVENT_CATALOG_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_MACHINE_EVENT_CATALOG_VERSION
            ));
        }
        if self.events.is_empty() {
            errors.push("event catalog must declare at least one event".to_string());
        }

        let mut event_types = BTreeSet::new();
        for event in &self.events {
            if event.event_type.trim().is_empty() {
                errors.push("event_type is required".to_string());
            } else if !event_types.insert(event.event_type.as_str()) {
                errors.push(format!("duplicate event_type `{}`", event.event_type));
            }

            let mut payload_names = BTreeSet::new();
            for field in &event.payload_fields {
                if field.name.trim().is_empty() {
                    errors.push(format!(
                        "event `{}` payload field name is required",
                        event.event_type
                    ));
                } else if !payload_names.insert(field.name.as_str()) {
                    errors.push(format!(
                        "event `{}` has duplicate payload field `{}`",
                        event.event_type, field.name
                    ));
                }
                if field.type_name.trim().is_empty() {
                    errors.push(format!(
                        "event `{}` payload field `{}` must declare a type_name",
                        event.event_type, field.name
                    ));
                }
                if field.required && field.nullable {
                    errors.push(format!(
                        "event `{}` payload field `{}` cannot be required and nullable",
                        event.event_type, field.name
                    ));
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
