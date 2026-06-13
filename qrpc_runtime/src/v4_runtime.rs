mod event_replay_orchestration;
mod graph_symbol_expansion;
mod machine_transition_engine;
mod risk_execution_gate;
mod runtime_constructor_mode_gate;
mod simulated_execution_engine;
mod type_surface;

use self::graph_symbol_expansion::symbol_for_machine_id;
pub use self::graph_symbol_expansion::{
    expand_v4_graph_for_symbols, normalize_v4_backtest_symbols,
};
pub use self::type_surface::*;

include!("v4_runtime_types.rs");

impl V4PaperSimulatedRuntime {
    pub fn pull_machine(
        &mut self,
        machine_id: &str,
        now_ms: u64,
    ) -> Result<Vec<V4RuntimeEventEnvelope>> {
        let start_index = self.event_log.len();
        let cache_policy = self
            .machine_spec(machine_id)
            .ok_or_else(|| anyhow!("未知 machine `{machine_id}`"))?
            .cache_policy
            .clone();
        let mut cached_to_return = None;
        let mut recovery_started = false;
        {
            let state = self
                .machines
                .get_mut(machine_id)
                .ok_or_else(|| anyhow!("未知 machine `{machine_id}`"))?;
            state.last_pulled_at_ms = Some(now_ms);
            if state.status == V4MachineRuntimeStatus::SoftSilent {
                if matches!(cache_policy, MachineCachePolicy::ReturnLastThenRecover) {
                    cached_to_return = state.cached_output.clone();
                }
                state.status = V4MachineRuntimeStatus::Recovering;
                recovery_started = true;
            }
        }

        self.record_control_event(
            EVENT_DOWNSTREAM_PULL,
            "runtime",
            json!({ "machine_id": machine_id }),
            now_ms,
        );

        if let Some(cached) = cached_to_return {
            self.record_control_event(
                EVENT_CACHE_RETURNED,
                machine_id,
                json!({ "machine_id": machine_id, "cached_output": cached }),
                now_ms,
            );
        }
        if recovery_started {
            self.record_control_event(
                EVENT_RECOVERY_STARTED,
                "runtime",
                json!({ "machine_id": machine_id }),
                now_ms,
            );
        }

        Ok(self.events_since_and_trim(start_index))
    }

    pub fn complete_recovery(
        &mut self,
        machine_id: &str,
        now_ms: u64,
    ) -> Result<Vec<V4RuntimeEventEnvelope>> {
        let start_index = self.event_log.len();
        let should_complete = {
            let state = self
                .machines
                .get_mut(machine_id)
                .ok_or_else(|| anyhow!("未知 machine `{machine_id}`"))?;
            let should_complete = state.status == V4MachineRuntimeStatus::Recovering;
            if should_complete {
                state.status = V4MachineRuntimeStatus::Active;
                state.last_event_at_ms = Some(now_ms);
            }
            should_complete
        };

        if should_complete {
            self.record_control_event(
                EVENT_RECOVERY_COMPLETED,
                "runtime",
                json!({ "machine_id": machine_id }),
                now_ms,
            );
            self.record_control_event(
                EVENT_SILENCE_EXITED,
                "runtime",
                json!({ "machine_id": machine_id }),
                now_ms,
            );
        }

        Ok(self.events_since_and_trim(start_index))
    }

    pub fn memory_snapshot(&self, now_ms: u64) -> V4RuntimeMemorySnapshot {
        V4RuntimeMemorySnapshot {
            graph_id: self.graph.graph_id.clone(),
            runtime_mode: self.runtime_mode,
            ts_ms: now_ms,
            machines: self
                .graph
                .machines
                .iter()
                .filter_map(|machine| self.machine_snapshot(machine))
                .collect(),
            risk_plane: self.risk_plane_snapshot(),
            execution: self.execution_snapshot(),
            simulated_execution: self.simulated_execution_snapshot(),
            venue_adapter_boundary: self.venue_adapter_boundary(),
            complexity_metrics: Some(ComplexityMetrics::from_machine_graph(
                &self.graph,
                default_v4_runtime_mode_contract().modes.len() as u32,
                0,
            )),
            event_sequence: self.sequence,
            provider_order_submission_attached: self.provider_order_submission_attached,
        }
    }

    fn machine_snapshot(&self, machine: &V4MachineContract) -> Option<V4MachineRuntimeSnapshot> {
        let state = self.machines.get(&machine.machine_id)?;
        let children = machine
            .states
            .iter()
            .find(|candidate| candidate.state_id == state.state_id)
            .and_then(|active_state| active_state.child_machine.as_deref())
            .and_then(|child_machine| self.machine_snapshot(child_machine))
            .into_iter()
            .collect();
        Some(V4MachineRuntimeSnapshot {
            machine_id: machine.machine_id.clone(),
            template: machine.template.clone(),
            state_id: state.state_id.clone(),
            status: state.status,
            memory: state.memory.clone(),
            cached_output: state.cached_output.clone(),
            last_pulled_at_ms: state.last_pulled_at_ms,
            last_event_at_ms: state.last_event_at_ms,
            children,
        })
    }

    pub fn machine_status(&self, machine_id: &str) -> Option<V4MachineRuntimeStatus> {
        self.machines.get(machine_id).map(|state| state.status)
    }

    pub fn machine_state_id(&self, machine_id: &str) -> Option<&str> {
        self.machines
            .get(machine_id)
            .map(|state| state.state_id.as_str())
    }

    pub fn event_log(&self) -> &[V4RuntimeEventEnvelope] {
        &self.event_log
    }

    pub fn risk_plane_snapshot(&self) -> V4RiskPlaneRuntimeSnapshot {
        V4RiskPlaneRuntimeSnapshot {
            required: self.risk_plane.required,
            machine_ids: self.risk_plane.machine_ids.iter().cloned().collect(),
            min_priority: self.risk_plane.min_priority,
            approved_event_count: self.risk_plane.approved_event_count,
            rejected_event_count: self.risk_plane.rejected_event_count,
            real_order_path_unlocked: self.risk_plane.approved_event_count > 0
                && self.risk_plane.rejected_event_count == 0,
            last_decision: self.risk_plane.last_decision.clone(),
        }
    }

    pub fn execution_snapshot(&self) -> V4ExecutionRuntimeSnapshot {
        V4ExecutionRuntimeSnapshot {
            venue_id: self
                .execution
                .capability_policy
                .as_ref()
                .map(|policy| policy.venue_matrix.venue_id.clone()),
            required_capabilities: self
                .execution
                .capability_policy
                .as_ref()
                .map(|policy| policy.required_capabilities.clone())
                .unwrap_or_default(),
            accepted_count: self.execution.accepted_count,
            rejected_count: self.execution.rejected_count,
            last_decision: self.execution.last_decision.clone(),
        }
    }

    pub fn venue_adapter_boundary(&self) -> V4VenueAdapterRuntimeBoundary {
        let runtime_mode_contract = default_v4_runtime_mode_contract();
        let mode_spec = runtime_mode_contract
            .mode_spec(self.runtime_mode)
            .expect("default v4 runtime mode contract declares all runtime modes");
        V4VenueAdapterRuntimeBoundary {
            provider_order_submission_attached: self.provider_order_submission_attached,
            provider_order_submission_allowed: mode_spec.provider_order_submission_allowed,
            settlement_authority: mode_spec.settlement_authority,
            live_actual_submission_allowed: false,
            rejection_before_provider_submit: !self.provider_order_submission_attached,
            reason: if self.provider_order_submission_attached {
                "运行时配置已接入 provider 下单提交".to_string()
            } else {
                "v4 PaperSimulated runtime 保持 VenueAdapter 提交断开；provider_native 下单必须在提交 provider 前拒绝".to_string()
            },
        }
    }

    fn validate_event_payload(&self, event: &V4RuntimeEventEnvelope) -> Result<(), String> {
        let Some(catalog) = &self.graph.event_catalog else {
            return Ok(());
        };
        let Some(spec) = catalog
            .events
            .iter()
            .find(|candidate| candidate.event_type == event.event_type)
        else {
            return Err(format!(
                "event `{}` is not declared in MachineEventCatalog",
                event.event_type
            ));
        };
        let Some(payload) = event.payload.as_object() else {
            return Err(format!(
                "event `{}` payload must be a JSON object",
                event.event_type
            ));
        };

        for field in &spec.payload_fields {
            let value = payload.get(field.name.as_str());
            match value {
                None if field.required => {
                    return Err(format!(
                        "event `{}` payload missing required field `{}`",
                        event.event_type, field.name
                    ));
                }
                None => continue,
                Some(Value::Null) if field.nullable => continue,
                Some(Value::Null) => {
                    return Err(format!(
                        "event `{}` payload field `{}` is null but not nullable",
                        event.event_type, field.name
                    ));
                }
                Some(value) => validate_payload_field_type(field, value).map_err(|reason| {
                    format!(
                        "event `{}` payload field `{}` type mismatch: {}",
                        event.event_type, field.name, reason
                    )
                })?,
            }
        }

        if event.origin == V4RuntimeEventOrigin::ExternalInput
            && event.source != V4_DEFAULT_MARKET_DATA_SOURCE
            && matches!(
                spec.source_kind,
                MachineEventSourceKind::Runtime | MachineEventSourceKind::MarketData
            )
        {
            let declared_fields = spec
                .payload_fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<BTreeSet<_>>();
            if let Some(unknown_field) = payload
                .keys()
                .find(|key| !declared_fields.contains(key.as_str()))
            {
                return Err(format!(
                    "event `{}` payload contains unknown field `{}`",
                    event.event_type, unknown_field
                ));
            }
        }

        Ok(())
    }

    fn record_event_rejected(&mut self, event: &V4RuntimeEventEnvelope, reason: String) {
        self.record_control_event(
            V4_RUNTIME_EVENT_REJECTED_EVENT,
            "runtime.validation",
            json!({
                "rejected_event_sequence": event.sequence,
                "rejected_event_type": event.event_type,
                "rejected_event_source": event.source,
                "reason": reason,
                "payload": event.payload,
            }),
            event.ts_ms,
        );
    }

    fn event_source_kind(&self, event_type: &str) -> Option<MachineEventSourceKind> {
        self.graph
            .event_catalog
            .as_ref()?
            .events
            .iter()
            .find(|candidate| candidate.event_type == event_type)
            .map(|event| event.source_kind.clone())
    }

    fn enqueue_graph_event(
        &mut self,
        event_type: impl Into<String>,
        source: impl Into<String>,
        payload: Value,
        ts_ms: u64,
        replayable: bool,
        origin: V4RuntimeEventOrigin,
    ) {
        self.sequence += 1;
        let event = V4RuntimeEventEnvelope {
            sequence: self.sequence,
            event_type: event_type.into(),
            source: source.into(),
            origin,
            ts_ms,
            payload,
            replayable,
        };
        self.event_log.push(event.clone());
        self.event_queue.push_back(event);
    }

    fn record_control_event(
        &mut self,
        event_type: impl Into<String>,
        source: impl Into<String>,
        payload: Value,
        ts_ms: u64,
    ) {
        self.sequence += 1;
        self.event_log.push(V4RuntimeEventEnvelope {
            sequence: self.sequence,
            event_type: event_type.into(),
            source: source.into(),
            origin: V4RuntimeEventOrigin::RuntimeControl,
            ts_ms,
            payload,
            replayable: true,
        });
    }

    fn machine_spec(&self, machine_id: &str) -> Option<&V4MachineContract> {
        self.graph
            .machines
            .iter()
            .find_map(|machine| find_machine_spec(machine, machine_id))
    }
}

fn find_machine_spec<'a>(
    machine: &'a V4MachineContract,
    machine_id: &str,
) -> Option<&'a V4MachineContract> {
    if machine.machine_id == machine_id {
        return Some(machine);
    }
    machine
        .states
        .iter()
        .filter_map(|state| state.child_machine.as_deref())
        .find_map(|child_machine| find_machine_spec(child_machine, machine_id))
}

fn flatten_machine_snapshots<'a>(
    machines: &'a [V4MachineRuntimeSnapshot],
) -> Vec<&'a V4MachineRuntimeSnapshot> {
    let mut flattened = Vec::new();
    for machine in machines {
        flattened.push(machine);
        flattened.extend(flatten_machine_snapshots(&machine.children));
    }
    flattened
}

fn validate_payload_field_type(
    field: &MachineEventPayloadField,
    value: &Value,
) -> Result<(), String> {
    let type_name = field.type_name.trim().to_ascii_lowercase();
    let ok = match type_name.as_str() {
        "string" | "symbol" | "venue" | "account" | "side" | "position_side" | "order_type"
        | "time_in_force" | "freshness" | "runtime_mode" | "order_permission" => value.is_string(),
        "bool" | "boolean" => value.is_boolean(),
        "u64" | "uint" => value.as_u64().is_some(),
        "i64" | "int" | "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "f64" | "decimal" | "number" | "price" | "quantity" | "notional" | "percent" | "ratio"
        | "fee" | "slippage" | "leverage" => value.as_f64().is_some_and(f64::is_finite),
        "object" | "map" => value.is_object(),
        "array" | "list" => value.is_array(),
        other => return Err(format!("unsupported catalog type `{}`", other)),
    };

    if ok {
        Ok(())
    } else {
        Err(format!(
            "expected `{}`, got {}",
            field.type_name,
            payload_type_label(value)
        ))
    }
}

fn payload_type_label(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(number) if number.is_i64() => "i64",
        Value::Number(number) if number.is_u64() => "u64",
        Value::Number(_) => "f64",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[allow(dead_code)]
fn recovery_policy_allows_async(policy: &MachineRecoveryPolicy) -> bool {
    matches!(policy, MachineRecoveryPolicy::AsyncRecover)
}

#[cfg(test)]
mod test_harness;
