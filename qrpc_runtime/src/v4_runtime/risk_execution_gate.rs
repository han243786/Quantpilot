use super::*;

impl V4PaperSimulatedRuntime {
    pub(super) fn evaluate_risk_plane_for_execution(
        &self,
        target_machine_id: &str,
        event: &V4RuntimeEventEnvelope,
    ) -> V4RiskPlaneRuntimeDecision {
        let reject = |reason: String| V4RiskPlaneRuntimeDecision {
            decision_id: format!("risk-decision-{}", self.sequence + 1),
            target_machine_id: target_machine_id.to_string(),
            source_machine_id: event.source.clone(),
            event_type: event.event_type.clone(),
            approved: false,
            reason,
            ts_ms: event.ts_ms,
            sequence: self.sequence + 1,
        };

        if !self.risk_plane.required {
            return reject(
                "执行转换必须启用运行时 Risk Plane，但当前未要求 Risk Plane".to_string(),
            );
        }
        if !self.risk_plane.machine_ids.contains(event.source.as_str()) {
            return reject(format!(
                "执行事件来源 `{}` 不是运行时 Risk Plane 机器",
                event.source
            ));
        }
        if event.origin != V4RuntimeEventOrigin::MachineEmit {
            return reject("执行事件必须由 Risk Plane 机器转换发出".to_string());
        }
        if self.event_source_kind(&event.event_type) != Some(MachineEventSourceKind::RiskPlane) {
            return reject(format!(
                "执行事件 `{}` 未声明为 Risk Plane 事件",
                event.event_type
            ));
        }
        if event.payload.get("risk_plane_approved") != Some(&Value::Bool(true)) {
            return reject("Risk Plane 事件负载缺少显式批准标记".to_string());
        }

        let Some(source_machine) = self.machine_spec(event.source.as_str()) else {
            return reject(format!(
                "运行时 Risk Plane 来源 `{}` 不是已声明机器",
                event.source
            ));
        };
        if !matches!(source_machine.template, MachineTemplateKind::Decision) {
            return reject(format!(
                "运行时 Risk Plane 来源 `{}` 不是 Decision 机器",
                event.source
            ));
        }
        if source_machine.priority < self.risk_plane.min_priority {
            return reject(format!(
                "运行时 Risk Plane 来源 `{}` 优先级 {} 低于 min_priority {}",
                event.source, source_machine.priority, self.risk_plane.min_priority
            ));
        }
        match self.machines.get(event.source.as_str()) {
            Some(state) if state.status == V4MachineRuntimeStatus::Active => {}
            Some(state) => {
                return reject(format!(
                    "运行时 Risk Plane 来源 `{}` 当前不是 Active 状态: {:?}",
                    event.source, state.status
                ));
            }
            None => {
                return reject(format!(
                    "运行时 Risk Plane 来源 `{}` 缺少运行时状态",
                    event.source
                ));
            }
        }

        V4RiskPlaneRuntimeDecision {
            decision_id: format!("risk-decision-{}", self.sequence + 1),
            target_machine_id: target_machine_id.to_string(),
            source_machine_id: event.source.clone(),
            event_type: event.event_type.clone(),
            approved: true,
            reason: "Risk Plane 已批准执行转换".to_string(),
            ts_ms: event.ts_ms,
            sequence: self.sequence + 1,
        }
    }

    pub(super) fn record_risk_plane_decision(
        &mut self,
        decision: V4RiskPlaneRuntimeDecision,
        ts_ms: u64,
    ) {
        if decision.approved {
            self.risk_plane.approved_event_count += 1;
        } else {
            self.risk_plane.rejected_event_count += 1;
        }
        self.risk_plane.last_decision = Some(decision.clone());

        self.record_control_event(
            if decision.approved {
                EVENT_RISK_PLANE_APPROVED
            } else {
                EVENT_RISK_PLANE_REJECTED
            },
            "runtime.risk_plane",
            json!({ "decision": decision }),
            ts_ms,
        );
    }

    pub(super) fn evaluate_execution_capabilities_for_execution(
        &self,
        target_machine_id: &str,
        ts_ms: u64,
    ) -> V4ExecutionRuntimeDecision {
        let decision_id = format!("execution-capability-decision-{}", self.sequence + 1);

        let Some(policy) = &self.execution.capability_policy else {
            return V4ExecutionRuntimeDecision {
                decision_id,
                target_machine_id: target_machine_id.to_string(),
                venue_id: "<missing>".to_string(),
                runtime_mode: self.runtime_mode,
                accepted: false,
                reason: "缺少 execution capability policy".to_string(),
                entries: vec![V4ExecutionCapabilityRuntimeEntry {
                    capability: ExecutionCapabilityKind::Market,
                    source: CapabilitySupportSource::Unsupported,
                    status: V4ExecutionCapabilityRuntimeStatus::PolicyMissing,
                    reason: "缺少 execution capability policy".to_string(),
                }],
                provider_order_submission_attached: self.provider_order_submission_attached,
                ts_ms,
                sequence: self.sequence + 1,
            };
        };

        if policy.required_capabilities.is_empty() {
            return V4ExecutionRuntimeDecision {
                decision_id,
                target_machine_id: target_machine_id.to_string(),
                venue_id: policy.venue_matrix.venue_id.clone(),
                runtime_mode: self.runtime_mode,
                accepted: false,
                reason: "ExecutionMachine 至少需要声明一个 execution capability".to_string(),
                entries: Vec::new(),
                provider_order_submission_attached: self.provider_order_submission_attached,
                ts_ms,
                sequence: self.sequence + 1,
            };
        }

        let runtime_mode_contract = default_v4_runtime_mode_contract();
        let mut entries = Vec::new();
        let mut errors = Vec::new();

        for capability in &policy.required_capabilities {
            let entry = match policy.venue_matrix.capability_entry(capability) {
                Some(entry) => entry,
                None => {
                    let reason = format!(
                        "execution capability `{:?}` 未在 venue `{}` 中声明",
                        capability, policy.venue_matrix.venue_id
                    );
                    errors.push(reason.clone());
                    entries.push(V4ExecutionCapabilityRuntimeEntry {
                        capability: *capability,
                        source: CapabilitySupportSource::Unsupported,
                        status: V4ExecutionCapabilityRuntimeStatus::NotDeclared,
                        reason,
                    });
                    continue;
                }
            };

            if matches!(entry.source, CapabilitySupportSource::Unsupported) {
                let reason = format!(
                    "execution capability `{:?}` 在 venue `{}` 中不受支持",
                    capability, policy.venue_matrix.venue_id
                );
                errors.push(reason.clone());
                entries.push(V4ExecutionCapabilityRuntimeEntry {
                    capability: *capability,
                    source: entry.source,
                    status: V4ExecutionCapabilityRuntimeStatus::Unsupported,
                    reason,
                });
                continue;
            }

            match policy.venue_matrix.require_supported_for_mode(
                capability,
                self.runtime_mode,
                &runtime_mode_contract,
            ) {
                Ok(source) => entries.push(V4ExecutionCapabilityRuntimeEntry {
                    capability: *capability,
                    source,
                    status: V4ExecutionCapabilityRuntimeStatus::Accepted,
                    reason: format!(
                        "execution capability `{:?}` 在 runtime mode `{:?}` 下以 `{:?}` 来源通过",
                        capability, self.runtime_mode, source
                    ),
                }),
                Err(reason) => {
                    errors.push(reason.clone());
                    entries.push(V4ExecutionCapabilityRuntimeEntry {
                        capability: *capability,
                        source: entry.source,
                        status: V4ExecutionCapabilityRuntimeStatus::ModeRejected,
                        reason,
                    });
                }
            }
        }

        V4ExecutionRuntimeDecision {
            decision_id,
            target_machine_id: target_machine_id.to_string(),
            venue_id: policy.venue_matrix.venue_id.clone(),
            runtime_mode: self.runtime_mode,
            accepted: errors.is_empty(),
            reason: if errors.is_empty() {
                "execution capabilities 已通过当前 runtime mode 校验".to_string()
            } else {
                errors.join("; ")
            },
            entries,
            provider_order_submission_attached: self.provider_order_submission_attached,
            ts_ms,
            sequence: self.sequence + 1,
        }
    }

    pub(super) fn record_execution_decision(
        &mut self,
        decision: V4ExecutionRuntimeDecision,
        ts_ms: u64,
    ) {
        if decision.accepted {
            self.execution.accepted_count += 1;
        } else {
            self.execution.rejected_count += 1;
        }
        self.execution.last_decision = Some(decision.clone());

        self.record_control_event(
            if decision.accepted {
                EVENT_EXECUTION_CAPABILITY_ACCEPTED
            } else {
                EVENT_EXECUTION_CAPABILITY_REJECTED
            },
            "runtime.execution_capability",
            json!({ "decision": decision }),
            ts_ms,
        );
    }

    pub(super) fn validate_single_execution_capability(
        &self,
        capability: ExecutionCapabilityKind,
    ) -> Result<(), String> {
        let Some(policy) = &self.execution.capability_policy else {
            return Err("缺少 execution capability policy".to_string());
        };
        let runtime_mode_contract = default_v4_runtime_mode_contract();
        policy
            .venue_matrix
            .require_supported_for_mode(&capability, self.runtime_mode, &runtime_mode_contract)
            .map(|_| ())
    }
}
