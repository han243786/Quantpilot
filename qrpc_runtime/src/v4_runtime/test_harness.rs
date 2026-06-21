use super::*;
use qrpc_core_ir::v4::{
    bridge_core_ir_to_v4_machine_graph, unsupported_v4_first_wave_matrix, CapabilitySupportSource,
    ExecutionCapabilityKind, MachineActionSpec, MachineEventSelector, MachineGuardDescriptor,
    MachineGuardReadRef, MachineGuardReadSource, MachineMemoryField, MachineState,
    MachineTransition, RuntimeTradingMode, StateGroup, V4MachineContract, V4MachineGraphContract,
    VenueCapabilityMatrix, V4_COMPAT_CORE_IR_LOADED_EVENT, V4_COMPAT_DECISION_MACHINE_ID,
    V4_COMPAT_EXECUTION_MACHINE_ID, V4_COMPAT_OBSERVATION_MACHINE_ID,
    V4_COMPAT_OBSERVATION_READY_EVENT, V4_COMPAT_RISK_APPROVED_EVENT,
};
use qrpc_core_ir::{
    moving_average_compare_expr, AgentPolicy, AgentPolicyKind, ComparisonOp, CoreIndicatorKind,
    CoreMetadata, CoreSourceKind, CoreStrategyIr, CoreTimeInForce, DataBinding, DataBindingKind,
    ExecutionRule, ExecutionSizingKind, IndicatorNode, RiskPolicy, SeriesExpr, SignalKind,
    SignalRule,
};

mod fixture_builders;
mod graph_replay_scenarios;
mod live_capability_guard_tests;
mod payload_validation_tests;
mod runtime_recovery_snapshot_tests;
mod simulated_execution_scenarios;

use fixture_builders::*;
