use serde_json::Value;
use std::collections::BTreeMap;

use crate::v4::{
    MachineEventCatalog, MachineEventPayloadField, MachineEventScope, MachineEventSourceKind,
    MachineEventTypeSpec, V4_COMPAT_CORE_IR_LOADED_EVENT, V4_COMPAT_DECISION_MACHINE_ID,
    V4_COMPAT_EXECUTION_MACHINE_ID, V4_COMPAT_OBSERVATION_MACHINE_ID,
    V4_COMPAT_OBSERVATION_READY_EVENT, V4_COMPAT_RISK_APPROVED_EVENT,
    V4_CORE_IR_COMPAT_BRIDGE_VERSION, V4_MACHINE_EVENT_CATALOG_VERSION,
};
pub(super) fn build_core_ir_compat_event_catalog(
    core_ir: &crate::CoreStrategyIr,
) -> MachineEventCatalog {
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "compat_bridge_version".to_string(),
        Value::String(V4_CORE_IR_COMPAT_BRIDGE_VERSION.to_string()),
    );
    metadata.insert(
        "core_strategy_id".to_string(),
        Value::String(core_ir.metadata.strategy_id.clone()),
    );

    MachineEventCatalog {
        schema_version: V4_MACHINE_EVENT_CATALOG_VERSION.to_string(),
        events: vec![
            MachineEventTypeSpec {
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                source_kind: MachineEventSourceKind::Runtime,
                scope: MachineEventScope::Runtime,
                payload_fields: vec![MachineEventPayloadField {
                    name: "strategy_id".to_string(),
                    type_name: "string".to_string(),
                    required: true,
                    nullable: false,
                }],
                allowed_emitters: Vec::new(),
                allowed_consumers: vec![V4_COMPAT_OBSERVATION_MACHINE_ID.to_string()],
                replayable: true,
            },
            MachineEventTypeSpec {
                event_type: V4_COMPAT_OBSERVATION_READY_EVENT.to_string(),
                source_kind: MachineEventSourceKind::Machine,
                scope: MachineEventScope::Graph,
                payload_fields: vec![MachineEventPayloadField {
                    name: "data_binding_count".to_string(),
                    type_name: "u64".to_string(),
                    required: true,
                    nullable: false,
                }],
                allowed_emitters: vec![V4_COMPAT_OBSERVATION_MACHINE_ID.to_string()],
                allowed_consumers: vec![V4_COMPAT_DECISION_MACHINE_ID.to_string()],
                replayable: true,
            },
            MachineEventTypeSpec {
                event_type: V4_COMPAT_RISK_APPROVED_EVENT.to_string(),
                source_kind: MachineEventSourceKind::RiskPlane,
                scope: MachineEventScope::Graph,
                payload_fields: vec![MachineEventPayloadField {
                    name: "execution_id".to_string(),
                    type_name: "string".to_string(),
                    required: true,
                    nullable: false,
                }],
                allowed_emitters: vec![V4_COMPAT_DECISION_MACHINE_ID.to_string()],
                allowed_consumers: vec![V4_COMPAT_EXECUTION_MACHINE_ID.to_string()],
                replayable: true,
            },
        ],
        metadata,
    }
}
