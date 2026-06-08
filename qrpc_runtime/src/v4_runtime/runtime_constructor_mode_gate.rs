use super::{
    initialize_machine_family_state, V4ExecutionCapabilityRuntimePolicy, V4ExecutionRuntimeState,
    V4PaperSimulatedRuntime, V4RiskPlaneRuntimeState, V4SimulatedExecutionConfig,
    V4SimulatedExecutionRuntimeState,
};
use anyhow::{anyhow, Result};
use qrpc_core_ir::v4::{
    ExecutionCapabilityKind, RuntimeTradingMode, V4MachineGraphContract, VenueCapabilityMatrix,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

impl V4PaperSimulatedRuntime {
    pub fn new(graph: V4MachineGraphContract) -> Result<Self> {
        Self::new_for_mode(graph, RuntimeTradingMode::PaperSimulated)
    }

    pub fn new_for_mode(
        graph: V4MachineGraphContract,
        runtime_mode: RuntimeTradingMode,
    ) -> Result<Self> {
        Self::new_for_mode_inner(graph, runtime_mode, false)
    }

    fn new_for_mode_inner(
        graph: V4MachineGraphContract,
        runtime_mode: RuntimeTradingMode,
        live_actual_capability_policy_will_be_attached: bool,
    ) -> Result<Self> {
        if !matches!(
            runtime_mode,
            RuntimeTradingMode::PaperSimulated | RuntimeTradingMode::LiveActual
        ) {
            return Err(anyhow!(
                "v4 Phase 5 runtime 只允许 PaperSimulated 模式，实际收到 {:?}",
                runtime_mode
            ));
        }
        if runtime_mode == RuntimeTradingMode::LiveActual
            && !graph
                .risk_plane
                .as_ref()
                .map(|plane| plane.required)
                .unwrap_or(false)
        {
            return Err(anyhow!(
                "v4 LiveActual runtime requires an explicit required Risk Plane"
            ));
        }
        if runtime_mode == RuntimeTradingMode::LiveActual
            && !live_actual_capability_policy_will_be_attached
        {
            return Err(anyhow!(
                "v4 LiveActual runtime requires an explicit execution capability policy"
            ));
        }
        graph.validate_static_contract().map_err(|errors| {
            anyhow!(
                "v4 machine graph 在进入 PaperSimulated runtime 前未通过静态契约: {:?}",
                errors
            )
        })?;

        let mut machines = BTreeMap::new();
        for machine in &graph.machines {
            initialize_machine_family_state(machine, &mut machines)?;
        }
        let risk_plane = graph
            .risk_plane
            .as_ref()
            .map(|risk_plane| V4RiskPlaneRuntimeState {
                required: risk_plane.required,
                machine_ids: risk_plane.machine_ids.iter().cloned().collect(),
                min_priority: risk_plane.min_priority,
                approved_event_count: 0,
                rejected_event_count: 0,
                last_decision: None,
            })
            .unwrap_or_else(|| V4RiskPlaneRuntimeState {
                required: false,
                machine_ids: BTreeSet::new(),
                min_priority: 0,
                approved_event_count: 0,
                rejected_event_count: 0,
                last_decision: None,
            });

        Ok(Self {
            graph,
            runtime_mode,
            machines,
            risk_plane,
            execution: V4ExecutionRuntimeState {
                capability_policy: None,
                accepted_count: 0,
                rejected_count: 0,
                last_decision: None,
            },
            simulated_execution: V4SimulatedExecutionRuntimeState::new(
                V4SimulatedExecutionConfig::default(),
                0,
            ),
            event_queue: VecDeque::new(),
            event_log: Vec::new(),
            sequence: 0,
            provider_order_submission_attached: false,
        })
    }

    pub fn new_with_execution_capabilities(
        graph: V4MachineGraphContract,
        venue_matrix: VenueCapabilityMatrix,
        required_capabilities: Vec<ExecutionCapabilityKind>,
    ) -> Result<Self> {
        Self::new(graph)?.with_execution_capabilities(venue_matrix, required_capabilities)
    }

    pub fn new_for_mode_with_execution_capabilities(
        graph: V4MachineGraphContract,
        runtime_mode: RuntimeTradingMode,
        venue_matrix: VenueCapabilityMatrix,
        required_capabilities: Vec<ExecutionCapabilityKind>,
    ) -> Result<Self> {
        Self::new_for_mode_inner(graph, runtime_mode, true)?
            .with_execution_capabilities(venue_matrix, required_capabilities)
    }

    pub fn new_for_backtest(
        graph: V4MachineGraphContract,
        venue_matrix: VenueCapabilityMatrix,
        required_capabilities: Vec<ExecutionCapabilityKind>,
    ) -> Result<Self> {
        Self::new_with_execution_capabilities(graph, venue_matrix, required_capabilities)
    }

    pub fn with_execution_capabilities(
        mut self,
        venue_matrix: VenueCapabilityMatrix,
        required_capabilities: Vec<ExecutionCapabilityKind>,
    ) -> Result<Self> {
        venue_matrix
            .validate_required_capability_sources(&required_capabilities)
            .map_err(|errors| {
                anyhow!(
                    "v4 execution capability policy 未通过静态契约: {:?}",
                    errors
                )
            })?;
        self.execution.capability_policy = Some(V4ExecutionCapabilityRuntimePolicy {
            venue_matrix,
            required_capabilities,
        });
        Ok(self)
    }
}
