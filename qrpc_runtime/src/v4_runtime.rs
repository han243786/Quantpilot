mod event_replay_orchestration;
mod graph_symbol_expansion;
mod machine_transition_engine;
mod runtime_constructor_mode_gate;
mod type_surface;

use self::graph_symbol_expansion::symbol_for_machine_id;
pub use self::graph_symbol_expansion::{
    expand_v4_graph_for_symbols, normalize_v4_backtest_symbols,
};
pub use self::type_surface::*;

include!("v4_runtime_types.rs");

impl V4PaperSimulatedRuntime {
    pub fn with_simulated_execution_config(
        mut self,
        config: V4SimulatedExecutionConfig,
    ) -> Result<Self> {
        validate_simulated_execution_config(&config)?;
        self.simulated_execution = V4SimulatedExecutionRuntimeState::new(config, self.sequence);
        Ok(self)
    }

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

    pub fn update_simulated_market_price(
        &mut self,
        venue_id: &str,
        symbol: &str,
        price: f64,
        now_ms: u64,
    ) -> Result<Vec<V4RuntimeEventEnvelope>> {
        if !price.is_finite() || price <= 0.0 {
            return Err(anyhow!("模拟行情价格必须是有限数且大于 0"));
        }

        let start_index = self.event_log.len();
        let outcome = self
            .simulated_execution
            .update_market_price(venue_id, symbol, price, now_ms);
        self.record_simulated_execution_events(outcome, now_ms);
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

    pub fn simulated_execution_snapshot(&self) -> V4SimulatedExecutionSnapshot {
        self.simulated_execution.snapshot()
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

    fn evaluate_risk_plane_for_execution(
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

    fn record_risk_plane_decision(&mut self, decision: V4RiskPlaneRuntimeDecision, ts_ms: u64) {
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

    fn evaluate_execution_capabilities_for_execution(
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

    fn record_execution_decision(&mut self, decision: V4ExecutionRuntimeDecision, ts_ms: u64) {
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

    fn apply_runtime_simulated_execution_for_transition(
        &mut self,
        machine_id: &str,
        event: &V4RuntimeEventEnvelope,
        ts_ms: u64,
    ) -> Result<V4SimulatedExecutionOutcome> {
        let runtime_mode_contract = default_v4_runtime_mode_contract();
        let mode_spec = runtime_mode_contract
            .mode_spec(self.runtime_mode)
            .ok_or_else(|| anyhow!("未声明 runtime mode `{:?}`", self.runtime_mode))?;

        if mode_spec.settlement_authority != RuntimeSettlementAuthority::LocalSimulated {
            let request = self.build_simulated_order_request(machine_id, event);
            return Ok(self.simulated_execution.reject_order(
                request,
                event.sequence,
                ts_ms,
                "runtime mode 不是 local_simulated；provider submission 已断开".to_string(),
            ));
        }

        if let Some(amend_order_id) =
            payload_string(&event.payload, &["amend_order_id", "replace_order_id"])
        {
            if let Err(reason) = self
                .validate_single_execution_capability(ExecutionCapabilityKind::CancelReplaceAmend)
            {
                let request = self.build_simulated_order_request(machine_id, event);
                return Ok(self.simulated_execution.reject_order(
                    request,
                    event.sequence,
                    ts_ms,
                    reason,
                ));
            }
            return Ok(self.simulated_execution.amend_order(
                &amend_order_id,
                payload_f64(
                    &event.payload,
                    &["new_reference_price", "reference_price", "price"],
                ),
                payload_f64(&event.payload, &["new_limit_price", "limit_price"]),
                payload_f64(&event.payload, &["new_trigger_price", "trigger_price"]),
                payload_f64(&event.payload, &["new_quantity", "quantity", "qty"]),
                ts_ms,
            ));
        }

        let request = self.build_simulated_order_request(machine_id, event);
        if let Err(reason) = self.validate_simulated_order_capabilities(&request) {
            return Ok(self.simulated_execution.reject_order(
                request,
                event.sequence,
                ts_ms,
                reason,
            ));
        }

        Ok(self
            .simulated_execution
            .submit_order(request, event.sequence, ts_ms))
    }

    fn build_simulated_order_request(
        &self,
        machine_id: &str,
        event: &V4RuntimeEventEnvelope,
    ) -> V4SimulatedOrderRequest {
        let config = &self.simulated_execution.config;
        let machine_metadata = self
            .machine_spec(machine_id)
            .map(|machine| &machine.metadata)
            .unwrap_or(&self.graph.metadata);
        let venue_id = payload_string(&event.payload, &["venue_id", "venue", "exchange"])
            .or_else(|| {
                self.execution
                    .capability_policy
                    .as_ref()
                    .map(|policy| policy.venue_matrix.venue_id.clone())
            })
            .or_else(|| metadata_string(machine_metadata, "core_venue_kind"))
            .unwrap_or_else(|| config.default_venue_id.clone());
        let symbol = payload_string(&event.payload, &["symbol", "instrument"])
            .or_else(|| metadata_string(&self.graph.metadata, "default_symbol"))
            .unwrap_or_else(|| config.default_symbol.clone());
        let action = payload_string(
            &event.payload,
            &["action", "position_action", "order_action", "side"],
        )
        .and_then(|raw| parse_position_action(raw.as_str()))
        .unwrap_or(V4SimulatedPositionAction::Buy);
        let order_type = payload_string(&event.payload, &["order_type", "type"])
            .and_then(|raw| parse_order_type(raw.as_str()))
            .unwrap_or(V4SimulatedOrderType::Market);
        let reference_price =
            payload_f64(&event.payload, &["reference_price", "price", "last_price"])
                .or_else(|| self.latest_market_price(&venue_id, &symbol))
                .unwrap_or(config.default_price);

        V4SimulatedOrderRequest {
            order_id: payload_string(&event.payload, &["order_id"]),
            client_order_id: payload_string(&event.payload, &["client_order_id"]),
            venue_id,
            symbol,
            action,
            order_type,
            quantity: payload_f64(&event.payload, &["quantity", "qty"])
                .unwrap_or(config.default_quantity),
            reference_price,
            limit_price: payload_f64(&event.payload, &["limit_price"]),
            trigger_price: payload_f64(&event.payload, &["trigger_price", "stop_price"]),
            take_profit_price: payload_f64(&event.payload, &["take_profit_price", "tp_price"]),
            stop_loss_price: payload_f64(&event.payload, &["stop_loss_price", "sl_price"]),
            trailing_offset_bps: payload_f64(
                &event.payload,
                &["trailing_offset_bps", "trail_offset_bps"],
            ),
            expire_at_ms: payload_u64(&event.payload, &["expire_at_ms", "expires_at_ms"]),
            time_in_force: payload_string(&event.payload, &["time_in_force", "tif"])
                .or_else(|| metadata_string(machine_metadata, "core_time_in_force"))
                .and_then(|raw| parse_time_in_force(raw.as_str())),
            post_only: payload_bool(&event.payload, &["post_only"]).unwrap_or(false),
            reduce_only: payload_bool(&event.payload, &["reduce_only"]).unwrap_or(false),
            close_only: payload_bool(&event.payload, &["close_only"]).unwrap_or(false),
            allow_partial_fill: payload_bool(&event.payload, &["allow_partial_fill"])
                .unwrap_or(config.allow_partial_fill),
            fee_bps: payload_f64(&event.payload, &["fee_bps"]).unwrap_or(config.default_fee_bps),
            slippage_bps: payload_f64(&event.payload, &["slippage_bps"])
                .unwrap_or(config.default_slippage_bps),
            max_fill_quantity: payload_f64(&event.payload, &["max_fill_quantity"])
                .or(config.max_fill_quantity),
        }
    }

    fn latest_market_price(&self, venue_id: &str, symbol: &str) -> Option<f64> {
        self.simulated_execution
            .market_prices
            .get(&(venue_id.to_string(), symbol.to_string()))
            .copied()
    }

    fn validate_simulated_order_capabilities(
        &self,
        request: &V4SimulatedOrderRequest,
    ) -> Result<(), String> {
        let Some(policy) = &self.execution.capability_policy else {
            return Err("缺少 execution capability policy".to_string());
        };
        let runtime_mode_contract = default_v4_runtime_mode_contract();
        let mut errors = Vec::new();
        for capability in simulated_order_required_capabilities(request) {
            if let Err(reason) = policy.venue_matrix.require_supported_for_mode(
                &capability,
                self.runtime_mode,
                &runtime_mode_contract,
            ) {
                errors.push(reason);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }

    fn validate_single_execution_capability(
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

    fn record_simulated_execution_events(
        &mut self,
        outcome: V4SimulatedExecutionOutcome,
        ts_ms: u64,
    ) {
        for (event_type, payload) in outcome.events {
            self.record_control_event(event_type, "runtime.execution_simulator", payload, ts_ms);
        }
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

include!("v4_simulated_execution.rs");

#[cfg(test)]
#[path = "v4_runtime_tests.rs"]
mod tests;
