use super::*;

impl V4PaperSimulatedRuntime {
    pub fn run_backtest_bars(&mut self, bars: &[V4BacktestBarInput]) -> Result<V4BacktestArtifact> {
        let started_at_ms = bars.first().map(|bar| bar.ts_ms).unwrap_or(0);
        let mut ended_at_ms = started_at_ms;
        let mut trajectory = Vec::new();
        let mut risk_plane_decisions = Vec::new();
        let mut execution_capability_sources = Vec::new();
        let mut seen_risk_decisions = BTreeSet::new();
        let mut seen_execution_entries = BTreeSet::new();
        let mut symbols = BTreeSet::new();
        let mut sorted_bars = bars.to_vec();
        sorted_bars.sort_by(|left, right| {
            left.ts_ms
                .cmp(&right.ts_ms)
                .then_with(|| left.symbol.cmp(&right.symbol))
                .then_with(|| left.venue_id.cmp(&right.venue_id))
        });

        for bar in &sorted_bars {
            symbols.insert(bar.symbol.clone());
            ended_at_ms = ended_at_ms.max(bar.ts_ms);
            self.submit_market_bar_closed(
                &bar.venue_id,
                &bar.symbol,
                bar.close,
                bar.ts_ms,
                &bar.event_type,
            )?;
            self.advance_time(bar.ts_ms);
            let snapshot = self.memory_snapshot(bar.ts_ms);

            for machine in flatten_machine_snapshots(&snapshot.machines) {
                trajectory.push(V4BacktestMachineTrajectoryPoint {
                    ts_ms: snapshot.ts_ms,
                    event_sequence: snapshot.event_sequence,
                    machine_id: machine.machine_id.clone(),
                    template: machine.template.clone(),
                    state_id: machine.state_id.clone(),
                    status: v4_machine_status_label(machine.status).to_string(),
                    symbol: symbol_for_machine_id(&machine.machine_id),
                });
            }

            if let Some(decision) = &snapshot.risk_plane.last_decision {
                if seen_risk_decisions.insert(decision.decision_id.clone()) {
                    risk_plane_decisions.push(V4BacktestRiskPlaneDecisionRecord {
                        decision_id: decision.decision_id.clone(),
                        target_machine_id: decision.target_machine_id.clone(),
                        source_machine_id: decision.source_machine_id.clone(),
                        event_type: decision.event_type.clone(),
                        approved: decision.approved,
                        reason: decision.reason.clone(),
                        ts_ms: decision.ts_ms,
                        sequence: decision.sequence,
                        symbol: symbol_for_machine_id(&decision.target_machine_id),
                    });
                }
            }

            if let Some(decision) = &snapshot.execution.last_decision {
                for entry in &decision.entries {
                    let key = format!(
                        "{}:{}:{:?}:{:?}",
                        decision.decision_id,
                        decision.target_machine_id,
                        entry.capability,
                        entry.status
                    );
                    if seen_execution_entries.insert(key) {
                        execution_capability_sources.push(
                            V4BacktestExecutionCapabilitySourceRecord {
                                decision_id: decision.decision_id.clone(),
                                target_machine_id: decision.target_machine_id.clone(),
                                venue_id: decision.venue_id.clone(),
                                runtime_mode: decision.runtime_mode,
                                accepted: decision.accepted,
                                reason: decision.reason.clone(),
                                capability: entry.capability,
                                source: entry.source,
                                status: v4_execution_capability_status_label(entry.status)
                                    .to_string(),
                                ts_ms: decision.ts_ms,
                                sequence: decision.sequence,
                                symbol: symbol_for_machine_id(&decision.target_machine_id),
                            },
                        );
                    }
                }
            }
        }

        let final_snapshot = serde_json::to_value(self.memory_snapshot(ended_at_ms))
            .map_err(|error| anyhow!("序列化 v4 回测最终快照失败: {error}"))?;

        Ok(V4BacktestArtifact {
            schema_version: V4_BACKTEST_ARTIFACT_VERSION.to_string(),
            graph_id: self.graph.graph_id.clone(),
            started_at_ms,
            ended_at_ms,
            replay_mode: "deterministic_bar_replay".to_string(),
            input_bar_count: sorted_bars.len(),
            input_tick_count: None,
            symbols: symbols.into_iter().collect(),
            machine_trajectory: trajectory,
            risk_plane_decisions,
            execution_capability_sources,
            microstructure_metrics: Some(self.simulated_execution.microstructure_metrics()),
            final_snapshot: Some(final_snapshot),
        })
    }

    pub fn run_backtest_ticks(
        &mut self,
        ticks: &[V4BacktestTickInput],
    ) -> Result<V4BacktestArtifact> {
        let started_at_ms = ticks.first().map(|tick| tick.ts_ms).unwrap_or(0);
        let mut ended_at_ms = started_at_ms;
        let mut trajectory = Vec::new();
        let mut risk_plane_decisions = Vec::new();
        let mut execution_capability_sources = Vec::new();
        let mut seen_risk_decisions = BTreeSet::new();
        let mut seen_execution_entries = BTreeSet::new();
        let mut symbols = BTreeSet::new();
        let mut sorted_ticks = ticks.to_vec();
        crate::sandbox::replay::sort_v4_replay_ticks_deterministically(&mut sorted_ticks);

        for tick in &sorted_ticks {
            if !tick.price.is_finite() || tick.price <= 0.0 {
                return Err(anyhow!("v4 tick replay requires finite positive prices"));
            }
            if !tick.size.is_finite() || tick.size < 0.0 {
                return Err(anyhow!("v4 tick replay requires finite non-negative sizes"));
            }
            symbols.insert(tick.symbol.clone());
            ended_at_ms = ended_at_ms.max(tick.ts_ms);
            self.submit_market_price_tick(
                &tick.venue_id,
                &tick.symbol,
                tick.price,
                tick.ts_ms,
                &tick.event_type,
            )?;
            self.advance_time(tick.ts_ms);
            let snapshot = self.memory_snapshot(tick.ts_ms);

            for machine in flatten_machine_snapshots(&snapshot.machines) {
                trajectory.push(V4BacktestMachineTrajectoryPoint {
                    ts_ms: snapshot.ts_ms,
                    event_sequence: snapshot.event_sequence,
                    machine_id: machine.machine_id.clone(),
                    template: machine.template.clone(),
                    state_id: machine.state_id.clone(),
                    status: v4_machine_status_label(machine.status).to_string(),
                    symbol: symbol_for_machine_id(&machine.machine_id),
                });
            }

            if let Some(decision) = &snapshot.risk_plane.last_decision {
                if seen_risk_decisions.insert(decision.decision_id.clone()) {
                    risk_plane_decisions.push(V4BacktestRiskPlaneDecisionRecord {
                        decision_id: decision.decision_id.clone(),
                        target_machine_id: decision.target_machine_id.clone(),
                        source_machine_id: decision.source_machine_id.clone(),
                        event_type: decision.event_type.clone(),
                        approved: decision.approved,
                        reason: decision.reason.clone(),
                        ts_ms: decision.ts_ms,
                        sequence: decision.sequence,
                        symbol: symbol_for_machine_id(&decision.target_machine_id),
                    });
                }
            }

            if let Some(decision) = &snapshot.execution.last_decision {
                for entry in &decision.entries {
                    let key = format!(
                        "{}:{}:{:?}:{:?}",
                        decision.decision_id,
                        decision.target_machine_id,
                        entry.capability,
                        entry.status
                    );
                    if seen_execution_entries.insert(key) {
                        execution_capability_sources.push(
                            V4BacktestExecutionCapabilitySourceRecord {
                                decision_id: decision.decision_id.clone(),
                                target_machine_id: decision.target_machine_id.clone(),
                                venue_id: decision.venue_id.clone(),
                                runtime_mode: decision.runtime_mode,
                                accepted: decision.accepted,
                                reason: decision.reason.clone(),
                                capability: entry.capability,
                                source: entry.source,
                                status: v4_execution_capability_status_label(entry.status)
                                    .to_string(),
                                ts_ms: decision.ts_ms,
                                sequence: decision.sequence,
                                symbol: symbol_for_machine_id(&decision.target_machine_id),
                            },
                        );
                    }
                }
            }
        }

        let final_snapshot = serde_json::to_value(self.memory_snapshot(ended_at_ms))
            .map_err(|error| anyhow!("序列化 v4 tick 回测最终快照失败: {error}"))?;

        Ok(V4BacktestArtifact {
            schema_version: V4_BACKTEST_ARTIFACT_VERSION.to_string(),
            graph_id: self.graph.graph_id.clone(),
            started_at_ms,
            ended_at_ms,
            replay_mode: "tick_replay".to_string(),
            input_bar_count: 0,
            input_tick_count: Some(sorted_ticks.len()),
            symbols: symbols.into_iter().collect(),
            machine_trajectory: trajectory,
            risk_plane_decisions,
            execution_capability_sources,
            microstructure_metrics: Some(self.simulated_execution.microstructure_metrics()),
            final_snapshot: Some(final_snapshot),
        })
    }

    pub fn submit_event(
        &mut self,
        event: V4RuntimeInputEvent,
    ) -> Result<V4PaperSimulatedRunOutput> {
        let start_index = self.event_log.len();
        self.enqueue_graph_event(
            event.event_type,
            event.source,
            event.payload,
            event.ts_ms,
            true,
            V4RuntimeEventOrigin::ExternalInput,
        );
        self.run_until_idle()?;
        Ok(self.output_since(start_index, event.ts_ms))
    }

    pub fn submit_market_price_tick(
        &mut self,
        venue_id: &str,
        symbol: &str,
        price: f64,
        ts_ms: u64,
        event_type: &str,
    ) -> Result<V4PaperSimulatedRunOutput> {
        if !price.is_finite() || price <= 0.0 {
            return Err(anyhow!(
                "v4 market price_tick requires a finite positive price"
            ));
        }
        let start_index = self.event_log.len();
        let outcome = self
            .simulated_execution
            .update_market_price(venue_id, symbol, price, ts_ms);
        self.record_simulated_execution_events(outcome, ts_ms);
        self.enqueue_graph_event(
            event_type,
            V4_DEFAULT_MARKET_DATA_SOURCE,
            json!({
                "venue_id": venue_id,
                "symbol": symbol,
                "price": price,
                "last_price": price,
                "ts_ms": ts_ms,
            }),
            ts_ms,
            true,
            V4RuntimeEventOrigin::ExternalInput,
        );
        self.run_until_idle()?;
        Ok(self.output_since(start_index, ts_ms))
    }

    pub fn submit_market_bar_closed(
        &mut self,
        venue_id: &str,
        symbol: &str,
        close: f64,
        ts_ms: u64,
        event_type: &str,
    ) -> Result<V4PaperSimulatedRunOutput> {
        if !close.is_finite() || close <= 0.0 {
            return Err(anyhow!(
                "v4 market bar_closed requires a finite positive close price"
            ));
        }
        let start_index = self.event_log.len();
        let outcome = self
            .simulated_execution
            .update_market_price(venue_id, symbol, close, ts_ms);
        self.record_simulated_execution_events(outcome, ts_ms);
        self.enqueue_graph_event(
            event_type,
            V4_DEFAULT_MARKET_DATA_SOURCE,
            json!({
                "venue_id": venue_id,
                "symbol": symbol,
                "close": close,
                "price": close,
                "ts_ms": ts_ms,
            }),
            ts_ms,
            true,
            V4RuntimeEventOrigin::ExternalInput,
        );
        self.run_until_idle()?;
        Ok(self.output_since(start_index, ts_ms))
    }

    pub fn advance_time(&mut self, now_ms: u64) -> Vec<V4RuntimeEventEnvelope> {
        let start_index = self.event_log.len();
        let machine_ids = self
            .graph
            .machines
            .iter()
            .map(|machine| machine.machine_id.clone())
            .collect::<Vec<_>>();

        for machine_id in machine_ids {
            let Some(machine) = self.machine_spec(&machine_id) else {
                continue;
            };
            let MachineSilencePolicy::SoftDormantAfter { ttl_ms } = machine.silence_policy else {
                continue;
            };
            let Some(state) = self.machines.get_mut(&machine_id) else {
                continue;
            };
            if state.status != V4MachineRuntimeStatus::Active {
                continue;
            }
            let last_observed = state
                .last_pulled_at_ms
                .or(state.last_event_at_ms)
                .unwrap_or(state.initialized_at_ms);
            if now_ms.saturating_sub(last_observed) >= ttl_ms {
                state.status = V4MachineRuntimeStatus::SoftSilent;
                self.record_control_event(
                    EVENT_SILENCE_ENTERED,
                    "runtime",
                    json!({
                        "machine_id": machine_id,
                        "ttl_ms": ttl_ms,
                        "last_observed_at_ms": last_observed
                    }),
                    now_ms,
                );
            }
        }

        let expiration_outcome = self.simulated_execution.expire_orders(now_ms);
        self.record_simulated_execution_events(expiration_outcome, now_ms);

        self.events_since_and_trim(start_index)
    }

    fn output_since(&mut self, start_index: usize, now_ms: u64) -> V4PaperSimulatedRunOutput {
        let events = self.events_since_and_trim(start_index);
        V4PaperSimulatedRunOutput {
            runtime_mode: self.runtime_mode,
            events,
            memory_snapshot: self.memory_snapshot(now_ms),
            provider_order_submission_attached: self.provider_order_submission_attached,
        }
    }

    pub(super) fn events_since_and_trim(
        &mut self,
        start_index: usize,
    ) -> Vec<V4RuntimeEventEnvelope> {
        let events = self
            .event_log
            .get(start_index..)
            .map(|slice| slice.to_vec())
            .unwrap_or_default();
        self.trim_event_log();
        events
    }

    pub(super) fn trim_event_log(&mut self) {
        if self.event_log.len() > V4_RUNTIME_MAX_EVENT_LOG_ENTRIES {
            let overflow = self.event_log.len() - V4_RUNTIME_MAX_EVENT_LOG_ENTRIES;
            self.event_log.drain(0..overflow);
        }
    }

    fn run_until_idle(&mut self) -> Result<()> {
        let mut steps = 0usize;
        while let Some(event) = self.event_queue.pop_front() {
            steps += 1;
            if steps > V4_RUNTIME_MAX_EVENT_STEPS {
                return Err(anyhow!(
                    "v4 runtime 超过最大事件步数 {}",
                    V4_RUNTIME_MAX_EVENT_STEPS
                ));
            }
            self.process_event(event)?;
        }
        Ok(())
    }
}
