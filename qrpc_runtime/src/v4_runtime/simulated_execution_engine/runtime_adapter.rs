use super::*;

impl V4PaperSimulatedRuntime {
    pub fn with_simulated_execution_config(
        mut self,
        config: V4SimulatedExecutionConfig,
    ) -> Result<Self> {
        validate_simulated_execution_config(&config)?;
        self.simulated_execution = V4SimulatedExecutionRuntimeState::new(config, self.sequence);
        Ok(self)
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

    pub fn simulated_execution_snapshot(&self) -> V4SimulatedExecutionSnapshot {
        self.simulated_execution.snapshot()
    }

    pub(in crate::v4_runtime) fn apply_runtime_simulated_execution_for_transition(
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

    pub(in crate::v4_runtime) fn validate_simulated_order_capabilities(
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

    pub(in crate::v4_runtime) fn record_simulated_execution_events(
        &mut self,
        outcome: V4SimulatedExecutionOutcome,
        ts_ms: u64,
    ) {
        for (event_type, payload) in outcome.events {
            self.record_control_event(event_type, "runtime.execution_simulator", payload, ts_ms);
        }
    }
}
