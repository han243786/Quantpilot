use super::*;

impl V4PaperSimulatedRuntime {
    pub(super) fn process_event(&mut self, event: V4RuntimeEventEnvelope) -> Result<()> {
        if let Err(reason) = self.validate_event_payload(&event) {
            self.record_event_rejected(&event, reason);
            return Ok(());
        }

        if self.machines.contains_key(event.source.as_str()) {
            if let Some(source_state) = self.machines.get_mut(event.source.as_str()) {
                source_state.last_pulled_at_ms = Some(event.ts_ms);
            }
        }

        let mut candidates = self
            .graph
            .machines
            .iter()
            .filter_map(|machine| self.transition_candidate_for_machine(machine, &event))
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.sort_id.cmp(&right.sort_id))
        });

        for candidate in candidates {
            let machine_id = candidate.machine_id;
            let transition = candidate.transition;
            let Some(preflight) = self.machine_transition_preflight(&machine_id, &transition)
            else {
                continue;
            };
            if let Some(guard) = transition
                .guard
                .as_ref()
                .filter(|guard| !guard.trim().is_empty())
            {
                self.record_event_rejected(
                    &event,
                    format!(
                        "transition `{}` declares unsupported guard `{}`; v4 runtime fails closed",
                        transition.transition_id, guard
                    ),
                );
                continue;
            }
            if let Some(guard_descriptor) = &transition.guard_descriptor {
                let readiness = guard_descriptor.readiness();
                self.record_event_rejected(
                    &event,
                    format!(
                        "transition `{}` declares structured guard `{}` with {} reads; {}",
                        transition.transition_id,
                        readiness.guard_id,
                        readiness.read_count,
                        readiness.execution_blocker_reason
                    ),
                );
                continue;
            }
            if let Some(memory_name) = preflight.undeclared_memory_write {
                self.record_event_rejected(
                    &event,
                    format!(
                        "transition `{}` writes undeclared memory field `{}`",
                        transition.transition_id, memory_name
                    ),
                );
                continue;
            }
            if preflight.is_execution_machine {
                let decision = self.evaluate_risk_plane_for_execution(&machine_id, &event);
                let approved = decision.approved;
                self.record_risk_plane_decision(decision, event.ts_ms);
                if !approved {
                    continue;
                }

                let execution_decision =
                    self.evaluate_execution_capabilities_for_execution(&machine_id, event.ts_ms);
                let execution_accepted = execution_decision.accepted;
                self.record_execution_decision(execution_decision, event.ts_ms);
                if !execution_accepted {
                    continue;
                }
            }
            let emitted_events = transition
                .action
                .as_ref()
                .map(|action| action.emits.clone())
                .unwrap_or_default();
            let mut silence_exited = false;
            let mut recovery_completed = false;

            {
                let Some(runtime_state) = self.machines.get_mut(machine_id.as_str()) else {
                    continue;
                };
                if runtime_state.state_id != transition.from_state {
                    continue;
                }
                if runtime_state.status == V4MachineRuntimeStatus::SoftSilent {
                    runtime_state.status = V4MachineRuntimeStatus::Active;
                    silence_exited = true;
                }
                if runtime_state.status == V4MachineRuntimeStatus::Recovering {
                    runtime_state.status = V4MachineRuntimeStatus::Active;
                    recovery_completed = true;
                }

                runtime_state.state_id = transition.to_state.clone();
                runtime_state.last_event_at_ms = Some(event.ts_ms);

                if let Some(action) = &transition.action {
                    for memory_name in &action.memory_writes {
                        if let Some(value) = event.payload.get(memory_name).cloned() {
                            runtime_state.memory.insert(memory_name.clone(), value);
                        }
                    }
                }

                if preflight.cache_return_last_then_recover {
                    runtime_state.cached_output = Some(V4CachedMachineOutput {
                        machine_id: machine_id.clone(),
                        state_id: runtime_state.state_id.clone(),
                        event_type: event.event_type.clone(),
                        emitted_events: emitted_events.clone(),
                        payload: event.payload.clone(),
                        updated_at_ms: event.ts_ms,
                        sequence: self.sequence,
                    });
                }
            }

            if silence_exited {
                self.record_control_event(
                    EVENT_SILENCE_EXITED,
                    "runtime",
                    json!({ "machine_id": machine_id, "reason": "event_arrived" }),
                    event.ts_ms,
                );
            }
            if recovery_completed {
                self.record_control_event(
                    EVENT_RECOVERY_COMPLETED,
                    "runtime",
                    json!({ "machine_id": machine_id, "reason": "event_arrived" }),
                    event.ts_ms,
                );
            }

            self.record_control_event(
                EVENT_TRANSITION_APPLIED,
                machine_id.as_str(),
                json!({
                    "machine_id": machine_id,
                    "transition_id": transition.transition_id,
                    "from_state": transition.from_state,
                    "to_state": transition.to_state,
                    "input_event_type": event.event_type,
                }),
                event.ts_ms,
            );

            if preflight.is_execution_machine {
                let outcome = self.apply_runtime_simulated_execution_for_transition(
                    machine_id.as_str(),
                    &event,
                    event.ts_ms,
                )?;
                self.record_simulated_execution_events(outcome, event.ts_ms);
            }

            for emitted_event in emitted_events {
                let payload = self.payload_for_emitted_event(
                    emitted_event.as_str(),
                    machine_id.as_str(),
                    &event,
                );
                self.enqueue_graph_event(
                    emitted_event,
                    machine_id.clone(),
                    payload,
                    event.ts_ms,
                    true,
                    V4RuntimeEventOrigin::MachineEmit,
                );
            }
        }

        Ok(())
    }

    fn payload_for_emitted_event(
        &self,
        event_type: &str,
        machine_id: &str,
        input_event: &V4RuntimeEventEnvelope,
    ) -> Value {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "emitted_by".to_string(),
            Value::String(machine_id.to_string()),
        );
        payload.insert(
            "input_event_type".to_string(),
            Value::String(input_event.event_type.clone()),
        );

        if let Some(spec) = self.graph.event_catalog.as_ref().and_then(|catalog| {
            catalog
                .events
                .iter()
                .find(|candidate| candidate.event_type == event_type)
        }) {
            if let Some(state) = self.machines.get(machine_id) {
                for field in &spec.payload_fields {
                    if let Some(value) = state.memory.get(field.name.as_str()) {
                        payload.insert(field.name.clone(), value.clone());
                    }
                }
            }
            for field in &spec.payload_fields {
                if payload.contains_key(field.name.as_str()) {
                    continue;
                }
                if let Some(value) = self.graph.metadata.get(field.name.as_str()) {
                    payload.insert(field.name.clone(), value.clone());
                } else if field.name == "execution_id" {
                    payload.insert(
                        field.name.clone(),
                        self.graph
                            .machines
                            .iter()
                            .find(|machine| {
                                matches!(machine.template, MachineTemplateKind::Execution)
                            })
                            .and_then(|machine| machine.metadata.get("core_execution_id"))
                            .cloned()
                            .unwrap_or(Value::Null),
                    );
                }
            }
            if spec.source_kind == MachineEventSourceKind::RiskPlane {
                payload.insert("risk_plane_approved".to_string(), Value::Bool(true));
                payload.insert(
                    "risk_plane_machine_id".to_string(),
                    Value::String(machine_id.to_string()),
                );
                payload.insert(
                    "risk_plane_decision".to_string(),
                    Value::String("approved".to_string()),
                );
            }
        }

        Value::Object(payload)
    }

    fn transition_candidate_for_machine(
        &self,
        machine: &V4MachineContract,
        event: &V4RuntimeEventEnvelope,
    ) -> Option<RuntimeTransitionCandidate> {
        let runtime_state = self.machines.get(machine.machine_id.as_str())?;
        if let Some(transition) = matching_transition(machine, runtime_state, event) {
            return Some(RuntimeTransitionCandidate {
                priority: machine.priority,
                sort_id: machine.machine_id.clone(),
                machine_id: machine.machine_id.clone(),
                transition: transition.clone(),
            });
        }

        let child_machine = machine
            .states
            .iter()
            .find(|state| state.state_id == runtime_state.state_id)
            .and_then(|state| state.child_machine.as_deref())?;
        let child_state = self.machines.get(child_machine.machine_id.as_str())?;
        let transition = matching_transition(child_machine, child_state, event)?;
        Some(RuntimeTransitionCandidate {
            priority: machine.priority.saturating_sub(1),
            sort_id: format!("{}::{}", machine.machine_id, child_machine.machine_id),
            machine_id: child_machine.machine_id.clone(),
            transition: transition.clone(),
        })
    }

    fn machine_transition_preflight(
        &self,
        machine_id: &str,
        transition: &MachineTransition,
    ) -> Option<MachineTransitionPreflight> {
        let machine = self.machine_spec(machine_id)?;
        let undeclared_memory_write = transition.action.as_ref().and_then(|action| {
            action
                .memory_writes
                .iter()
                .find(|name| {
                    !machine
                        .memory
                        .iter()
                        .any(|field| field.name.as_str() == name.as_str())
                })
                .cloned()
        });
        Some(MachineTransitionPreflight {
            is_execution_machine: matches!(machine.template, MachineTemplateKind::Execution),
            cache_return_last_then_recover: matches!(
                machine.cache_policy,
                MachineCachePolicy::ReturnLastThenRecover
            ),
            undeclared_memory_write,
        })
    }
}

#[derive(Debug)]
struct MachineTransitionPreflight {
    is_execution_machine: bool,
    cache_return_last_then_recover: bool,
    undeclared_memory_write: Option<String>,
}

fn matching_transition<'a>(
    machine: &'a V4MachineContract,
    runtime_state: &MachineRuntimeState,
    event: &V4RuntimeEventEnvelope,
) -> Option<&'a MachineTransition> {
    machine.transitions.iter().find(|transition| {
        transition.from_state == runtime_state.state_id
            && transition.event.event_type == event.event_type
            && transition_source_matches(transition.event.source.as_deref(), event)
            && transition_freshness_matches(transition.event.freshness.clone(), event)
    })
}

fn transition_source_matches(
    expected_source: Option<&str>,
    event: &V4RuntimeEventEnvelope,
) -> bool {
    expected_source
        .map(|source| source == event.source)
        .unwrap_or(true)
}

fn transition_freshness_matches(
    freshness: Option<EventFreshnessRequirement>,
    _event: &V4RuntimeEventEnvelope,
) -> bool {
    matches!(
        freshness,
        None | Some(EventFreshnessRequirement::FreshOnly)
            | Some(EventFreshnessRequirement::FreshOrStale)
            | Some(EventFreshnessRequirement::RecoveringAllowed)
    )
}
