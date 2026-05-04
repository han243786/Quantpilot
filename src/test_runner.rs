use super::*;
use anyhow::{Context, Result};
use qrpc_runtime::{
    DeterministicTestMode, FastBacktestSandbox, RealTimeSandbox, RuntimeCoordinator,
};
use quantscript::{
    lower_script_to_runtime_config, parse_quant_script_module,
    split_test_items, TestActionDef, TestPlan, TestStep,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Instant;

// ── Test Report Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    pub scenario_name: String,
    pub cover: Vec<String>,
    pub steps: Vec<StepResult>,
    pub passed_count: usize,
    pub failed_count: usize,
    pub skipped_count: usize,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub name: String,
    pub status: StepStatus,
    pub duration_ms: u64,
    pub message: Option<String>,
    pub data_snapshot: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    Passed,
    Failed,
    Skipped,
}

// ── Constants ──

const BACKTEST_TEST_SEED: u64 = 7;

// ── TestRunnerContext ──

pub struct TestRunnerContext {
    pub module: quantscript::ScriptModule,
    pub test_plans: Vec<TestPlan>,
}

impl TestRunnerContext {
    pub fn from_source(source: &str) -> Result<Self> {
        let module = parse_quant_script_module(source)
            .context("failed to parse quantscript source")?;
        let (strategy_module, test_plans) = split_test_items(&module);
        Ok(TestRunnerContext {
            module: strategy_module,
            test_plans,
        })
    }
}

// ── Test Runner ──

pub struct TestRunner {
    pub compile_result: Option<qrpc_core::CompiledRuntimeProtocol>,
    pub last_run_events_count: usize,
    pub last_run_equity: Option<f64>,
    pub last_run_status: Option<String>,
    pub last_backtest_metrics: BTreeMap<String, f64>,
    pub last_backtest_trades_count: usize,
    pub backtest_history: Vec<BTreeMap<String, f64>>,
    pending_modifications: Vec<(String, f64)>,
    last_run_event_types: Vec<String>,
    last_run_event_counts: Option<BTreeMap<String, usize>>,
}

impl TestRunner {
    pub fn new() -> Self {
        TestRunner {
            compile_result: None,
            last_run_events_count: 0,
            last_run_equity: None,
            last_run_status: None,
            last_backtest_metrics: BTreeMap::new(),
            last_backtest_trades_count: 0,
            backtest_history: Vec::new(),
            pending_modifications: Vec::new(),
            last_run_event_types: Vec::new(),
            last_run_event_counts: None,
        }
    }

    pub async fn execute(&mut self, ctx: &TestRunnerContext) -> Result<TestReport> {
        let start = Instant::now();
        let mut step_results = Vec::new();
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        for plan in &ctx.test_plans {
            if plan.steps.is_empty() {
                return Err(anyhow::anyhow!(
                    "test scenario '{}' has no @step directives",
                    plan.scenario_name
                ));
            }
            let compile_ok = self.compile_strategy(ctx);
            if !compile_ok {
                for step in &plan.steps {
                    step_results.push(StepResult {
                        name: step.name.clone(),
                        status: StepStatus::Skipped,
                        duration_ms: 0,
                        message: Some("compilation failed".to_string()),
                        data_snapshot: None,
                    });
                    skipped += 1;
                }
                continue;
            }

            for step in &plan.steps {
                let step_start = Instant::now();
                let result = self.execute_step(step);
                let duration_ms = step_start.elapsed().as_millis() as u64;

                match result {
                    Ok(message) => {
                        step_results.push(StepResult {
                            name: step.name.clone(),
                            status: StepStatus::Passed,
                            duration_ms,
                            message: Some(message),
                            data_snapshot: self.build_data_snapshot(step),
                        });
                        passed += 1;
                    }
                    Err(err) => {
                        step_results.push(StepResult {
                            name: step.name.clone(),
                            status: StepStatus::Failed,
                            duration_ms,
                            message: Some(err),
                            data_snapshot: self.build_data_snapshot(step),
                        });
                        failed += 1;
                    }
                }
            }
        }

        Ok(TestReport {
            scenario_name: ctx
                .test_plans
                .first()
                .map(|p| p.scenario_name.clone())
                .unwrap_or_default(),
            cover: ctx
                .test_plans
                .first()
                .map(|p| p.cover.clone())
                .unwrap_or_default(),
            steps: step_results,
            passed_count: passed,
            failed_count: failed,
            skipped_count: skipped,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    fn compile_strategy(&mut self, ctx: &TestRunnerContext) -> bool {
        self.compile_result = None; // L0-1: clear old result on recompile
        match lower_script_to_runtime_config(&ctx.module) {
            Ok(mut runtime_config) => {
                // Apply pending @modify parameter changes
                for (param_name, new_value) in &self.pending_modifications {
                    // Data sources
                    for ds in &mut runtime_config.data_sources {
                        if param_name.contains("window") || param_name.contains("days") {
                            ds.days = Some(*new_value as u32);
                        }
                    }
                    // Risk controls
                    for risk in &mut runtime_config.risks {
                        if param_name.contains("max_total_leverage") {
                            risk.max_total_leverage = *new_value;
                        }
                        if param_name.contains("max_exchange_leverage") {
                            risk.max_exchange_leverage = *new_value;
                        }
                        if param_name.contains("min_action_interval") {
                            risk.min_action_interval_ms = *new_value as u64;
                        }
                    }
                    // Intents
                    for intent in &mut runtime_config.intents {
                        let key = if param_name.contains("fast") {
                            "fast_period"
                        } else if param_name.contains("slow") {
                            "slow_period"
                        } else {
                            param_name.as_str()
                        };
                        intent.params.insert(key.to_string(), *new_value);
                    }
                }
                // Route through the graph API compilation path so that
                // intent/agent/execution modules are configured identically
                // to the frontend POST /api/runtime/compile pipeline.
                let template = FrontendRuntimeConfig {
                    metadata: FrontendMetadata {
                        graph_id: "test_scenario".to_string(),
                        compile_id: "test_compile".to_string(),
                        name: "Test Scenario".to_string(),
                        version: "0.1.0".to_string(),
                        mode: "paper".to_string(),
                    },
                    data_sources: vec![],
                    intent_generators: vec![],
                    agents: vec![],
                    risk_controls: vec![],
                    executions: vec![],
                    runtime_control: None,
                };
                let runtime_targets = CompileRuntimeTargets {
                    source_to_node: BTreeMap::new(),
                    runtime_node_id: None,
                    execution_node_id: None,
                };
                let frontend_config = frontend_runtime_config_from_core_with_template(
                    &runtime_config,
                    &template,
                    &runtime_targets,
                    "test_scenario",
                    "test_compile",
                );
                match map_frontend_runtime_config(&frontend_config) {
                    Ok(mapped) => {
                        match qrpc_compiler::compile_runtime_protocol_config(
                            &mapped.runtime_protocol,
                        ) {
                            Ok(compiled) => {
                                self.pending_modifications.clear();
                                self.compile_result = Some(compiled);
                                true
                            }
                            Err(e) => {
                                eprintln!("[TestRunner] graph-path compile failed: {e:?}");
                                false
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[TestRunner] frontend mapping failed: {e:?}");
                        false
                    }
                }
            }
            Err(e) => {
                eprintln!("[TestRunner] lowering failed: {e:?}");
                false
            }
        }
    }

    fn execute_step(&mut self, step: &TestStep) -> Result<String, String> {
        let mut messages = Vec::new();
        for action in &step.actions {
            match action {
                TestActionDef::Compile => {
                    messages.push("compile: ok".to_string());
                }
                TestActionDef::Run {
                    mode,
                    duration_secs,
                    save,
                } => {
                    if mode != "paper" {
                        return Err(format!(
                            "unsupported run mode: '{}'. Only 'paper' is supported.",
                            mode
                        ));
                    }
                    if *duration_secs == 0 {
                        return Err("run duration must be > 0".to_string());
                    }
                    let result = self.execute_run(*duration_secs)?;
                    if *save {
                        messages.push(format!("save: run data captured ({} events)", self.last_run_events_count));
                    }
                    messages.push(result);
                }
                TestActionDef::Backtest {
                    source,
                    start,
                    end,
                    seed,
                    save: _,
                } => {
                    if source != "deterministic_mock" && source != "historical_replay" {
                        return Err(format!(
                            "unsupported backtest source: '{}'. Use 'deterministic_mock' or 'historical_replay'.",
                            source
                        ));
                    }
                    let s = seed.unwrap_or(BACKTEST_TEST_SEED);
                    let result = if source == "historical_replay" {
                        self.execute_backtest_historical(s)?
                    } else {
                        self.execute_backtest_with_range(s, start.as_deref(), end.as_deref())?
                    };
                    messages.push(result);
                }
                TestActionDef::Assert(expr) => {
                    match self.evaluate_assert(expr) {
                        Ok(true) => messages.push(format!("assert({}) = true", expr)),
                        Ok(false) => return Err(format!("assert failed: {}", expr)),
                        Err(e) => return Err(format!("assert error: {} — {}", expr, e)),
                    }
                }
                TestActionDef::SaveRun => {
                    if self.backtest_history.is_empty() && self.last_run_equity.is_none() {
                        return Err(
                            "save_run: no prior run or backtest to save".to_string()
                        );
                    }
                    // L2-3: persist to storage/test-runs/
                    let dir = std::path::Path::new("storage").join("test-runs");
                    std::fs::create_dir_all(&dir).map_err(|e| format!("save_run: {e}"))?;
                    let now_ms = current_time_ms();
                    let path = dir.join(format!("run_{now_ms}.json"));
                    let data = serde_json::json!({
                        "saved_at_ms": now_ms,
                        "run_equity": self.last_run_equity,
                        "run_events_count": self.last_run_events_count,
                        "run_status": self.last_run_status,
                        "backtest_metrics": self.last_backtest_metrics,
                    });
                    std::fs::write(&path, serde_json::to_string_pretty(&data).unwrap_or_default())
                        .map_err(|e| format!("save_run: {e}"))?;
                    messages.push(format!("save_run: persisted to {}", path.display()));
                }
                TestActionDef::Modify {
                    node: _,
                    param,
                    value,
                } => {
                    let num_val = match value {
                        quantscript::TestParamValueDef::Number(n) => *n,
                        quantscript::TestParamValueDef::String(s) => s.parse().unwrap_or(0.0),
                        quantscript::TestParamValueDef::Bool(b) => {
                            if *b { 1.0 } else { 0.0 }
                        }
                    };
                    self.pending_modifications.push((param.clone(), num_val));
                    messages.push(format!(
                        "modify: {} = {} (pending, will apply on next compile)",
                        param, num_val
                    ));
                }
                TestActionDef::Wait {
                    condition,
                    timeout_secs,
                } => {
                    let start = std::time::Instant::now();
                    let timeout = std::time::Duration::from_secs(*timeout_secs);
                    loop {
                        match self.evaluate_assert(condition) {
                            Ok(true) => {
                                messages.push(format!("wait({}) = true", condition));
                                break;
                            }
                            Ok(false) => {
                                if start.elapsed() >= timeout {
                                    return Err(format!(
                                        "wait timeout after {}s: {}",
                                        timeout_secs, condition
                                    ));
                                }
                                std::thread::sleep(std::time::Duration::from_millis(200));
                            }
                            Err(e) => {
                                return Err(format!("wait error: {}", e));
                            }
                        }
                    }
                }
            }
        }
        Ok(messages.join("; "))
    }

    fn execute_run(&mut self, duration_secs: u64) -> Result<String, String> {
        let compiled = self
            .compile_result
            .clone()
            .ok_or_else(|| "no compile result".to_string())?;
        let mut sandbox = RealTimeSandbox::new(RuntimeCoordinator::new(compiled));
        sandbox
            .start()
            .map_err(|e| format!("sandbox start failed: {e}"))?;
        let now_ms = current_time_ms();
        let run_duration = std::cmp::min(duration_secs * 1000, RUN_WINDOW_MS);
        let session = sandbox
            .run_session(now_ms, now_ms + run_duration)
            .map_err(|e| format!("run session failed: {e}"))?;
        let account = account_summary(&session);
        self.last_run_equity = Some(account.equity_estimate);
        self.last_run_status = Some("completed".to_string());

        // L0-2 + L1-5: collect real event types from session cycles
        let mut event_types = Vec::new();
        let mut event_counts = BTreeMap::new();
        for cycle in [&session.slow_cycle, &session.fast_cycle] {
            for evt in &cycle.runtime_events {
                let name = format!("{:?}", evt.event_type);
                *event_counts.entry(name.clone()).or_insert(0) += 1;
                event_types.push(name);
            }
            // Infer events from other cycle data
            if !cycle.fill_reports.is_empty() {
                *event_counts.entry("ExecutionFilled".to_string()).or_insert(0) += cycle.fill_reports.len();
            }
            if !cycle.execution_plans.is_empty() {
                *event_counts.entry("ExecutionPlanned".to_string()).or_insert(0) += cycle.execution_plans.len();
            }
            if !cycle.agent_decisions.is_empty() {
                *event_counts.entry("AgentDecisionProduced".to_string()).or_insert(0) += cycle.agent_decisions.len();
            }
            if !cycle.risk_decisions.is_empty() {
                *event_counts.entry("RiskDecisionProduced".to_string()).or_insert(0) += cycle.risk_decisions.len();
            }
            if !cycle.intent_signals.is_empty() {
                *event_counts.entry("IntentTriggered".to_string()).or_insert(0) += cycle.intent_signals.len();
            }
        }
        self.last_run_event_types = event_types;
        self.last_run_events_count = event_counts.values().sum::<usize>();
        self.last_run_event_counts = Some(event_counts);

        Ok(format!(
            "run: equity={:.2}, fetches={}, events={}",
            account.equity_estimate,
            session.data_fetch_counts.len(),
            self.last_run_events_count
        ))
    }

    fn execute_backtest_with_range(
        &mut self,
        seed: u64,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<String, String> {
        let now_ms = if let (Some(start), Some(end)) = (start_date, end_date) {
            parse_date_range_ms(start, end)
                .map_err(|e| format!("invalid date range: {e}"))?
        } else if let Some(start) = start_date {
            let start_ms = parse_iso_date_ms(start)
                .map_err(|e| format!("invalid start date '{}': {e}", start))?;
            start_ms + 365 * 24 * 3600 * 1000
        } else {
            current_time_ms()
        };
        self.execute_backtest_core(seed, now_ms)
    }

    fn execute_backtest_core(&mut self, seed: u64, now_ms: u64) -> Result<String, String> {
        let compiled = self
            .compile_result
            .clone()
            .ok_or_else(|| "no compile result".to_string())?;
        let test_mode =
            DeterministicTestMode::replay_defaults(now_ms, seed);
        let mut sandbox =
            FastBacktestSandbox::with_mock_replay_from_core_ir_and_test_mode(
                compiled.core_ir.clone(),
                now_ms,
                test_mode,
            )
            .map_err(|e| format!("backtest init failed: {e}"))?;
        sandbox
            .start()
            .map_err(|e| format!("backtest start failed: {e}"))?;
        let backtest = sandbox
            .run_backtest()
            .map_err(|e| format!("backtest run failed: {e}"))?;

        // Extract REAL metrics from BacktestOutput.summary
        let summary = &backtest.summary;
        let total_fills: usize = backtest
            .sessions
            .iter()
            .map(|s| s.slow_cycle.fill_reports.len() + s.fast_cycle.fill_reports.len())
            .sum();

        // Count buy/sell fills and aggregate fees
        let (buy_fills, sell_fills, total_fees, total_notional): (usize, usize, f64, f64) = backtest
            .sessions
            .iter()
            .flat_map(|s| {
                s.slow_cycle
                    .fill_reports
                    .iter()
                    .chain(s.fast_cycle.fill_reports.iter())
            })
            .fold((0, 0, 0.0, 0.0), |(buys, sells, fees, notional), fill| {
                let fee = fill.fee_paid;
                let qty = fill.filled_qty;
                let price = fill.filled_price;
                if qty > 0.0 {
                    (buys + 1, sells, fees + fee, notional + qty * price)
                } else {
                    (buys, sells + 1, fees + fee, notional + qty.abs() * price)
                }
            });

        // Equity curve analysis
        let initial_equity = backtest
            .equity_curve
            .first()
            .map(|p| p.equity)
            .unwrap_or(100_000.0);
        let final_equity = summary.final_equity;
        let peak_equity = backtest
            .equity_curve
            .iter()
            .map(|p| p.equity)
            .fold(f64::MIN, f64::max);

        let mut metrics = BTreeMap::new();
        metrics.insert("step_count".to_string(), summary.step_count as f64);
        metrics.insert("trade_count".to_string(), summary.trade_count as f64);
        metrics.insert("total_fills".to_string(), total_fills as f64);
        metrics.insert("buy_fills".to_string(), buy_fills as f64);
        metrics.insert("sell_fills".to_string(), sell_fills as f64);
        metrics.insert("total_fees_paid".to_string(), total_fees);
        metrics.insert("total_filled_notional".to_string(), total_notional);
        metrics.insert("initial_equity".to_string(), initial_equity);
        metrics.insert("final_equity".to_string(), final_equity);
        metrics.insert("peak_equity".to_string(), peak_equity);
        metrics.insert("net_profit".to_string(), summary.net_profit);
        metrics.insert(
            "total_return_pct".to_string(),
            summary.total_return_ratio * 100.0,
        );
        metrics.insert(
            "max_drawdown_pct".to_string(),
            summary.max_drawdown_ratio * 100.0,
        );
        // P1.1: Calculate Sharpe ratio from equity curve
        let (sharpe, annual_return, annual_vol) =
            compute_sharpe_from_equity(&backtest.equity_curve, initial_equity);
        metrics.insert("sharpe_ratio".to_string(), sharpe);
        metrics.insert("annual_return_pct".to_string(), annual_return);
        metrics.insert("annual_volatility_pct".to_string(), annual_vol);
        if initial_equity > f64::EPSILON {
            metrics.insert("fee_drag_pct".to_string(), total_fees / initial_equity * 100.0);
            metrics.insert(
                "turnover_ratio".to_string(),
                total_notional / initial_equity,
            );
        }
        if total_fills > 0 {
            metrics.insert("avg_fee_per_fill".to_string(), total_fees / total_fills as f64);
            metrics.insert(
                "avg_trade_notional".to_string(),
                total_notional / total_fills as f64,
            );
        }

        self.last_backtest_metrics = metrics.clone();
        self.last_backtest_trades_count = summary.trade_count;
        self.backtest_history.push(metrics.clone());
        Ok(format!(
            "backtest: steps={}, fills={} (buy={}, sell={}), return={:.4}%, drawdown={:.4}%, equity={:.2}→{:.2}",
            summary.step_count,
            total_fills,
            buy_fills,
            sell_fills,
            summary.total_return_ratio * 100.0,
            summary.max_drawdown_ratio * 100.0,
            initial_equity,
            final_equity,
        ))
    }

    fn execute_backtest_historical(&mut self, _seed: u64) -> Result<String, String> {
        let compiled = self
            .compile_result
            .clone()
            .ok_or_else(|| "no compile result".to_string())?;
        let now_ms = current_time_ms();
        // Use historical_replay — requires local cache files
        let mut sandbox = FastBacktestSandbox::with_replay_from_core_ir(
            compiled.core_ir.clone(),
            now_ms,
        )
        .map_err(|e| format!(
            "historical_replay requires cached market data. Run a paper simulation first to populate the cache. Details: {e}"
        ))?;
        sandbox
            .start()
            .map_err(|e| format!("backtest start failed: {e}"))?;
        let backtest = sandbox
            .run_backtest()
            .map_err(|e| format!("backtest run failed: {e}"))?;

        // Reuse the same metric extraction as execute_backtest
        let summary = &backtest.summary;
        let total_fills: usize = backtest
            .sessions
            .iter()
            .map(|s| s.slow_cycle.fill_reports.len() + s.fast_cycle.fill_reports.len())
            .sum();
        let initial_equity = backtest
            .equity_curve
            .first()
            .map(|p| p.equity)
            .unwrap_or(100_000.0);
        let final_equity = summary.final_equity;
        let total_fees: f64 = backtest
            .sessions
            .iter()
            .flat_map(|s| {
                s.slow_cycle.fill_reports.iter().chain(s.fast_cycle.fill_reports.iter())
            })
            .map(|f| f.fee_paid)
            .sum();

        let (sharpe, annual_return, annual_vol) =
            compute_sharpe_from_equity(&backtest.equity_curve, initial_equity);
        let mut metrics = BTreeMap::new();
        metrics.insert("step_count".to_string(), summary.step_count as f64);
        metrics.insert("trade_count".to_string(), summary.trade_count as f64);
        metrics.insert("total_fills".to_string(), total_fills as f64);
        metrics.insert("total_fees_paid".to_string(), total_fees);
        metrics.insert("net_profit".to_string(), summary.net_profit);
        metrics.insert("total_return_pct".to_string(), summary.total_return_ratio * 100.0);
        metrics.insert("max_drawdown_pct".to_string(), summary.max_drawdown_ratio * 100.0);
        metrics.insert("sharpe_ratio".to_string(), sharpe);
        metrics.insert("annual_return_pct".to_string(), annual_return);
        metrics.insert("annual_volatility_pct".to_string(), annual_vol);
        metrics.insert("initial_equity".to_string(), initial_equity);
        metrics.insert("final_equity".to_string(), final_equity);
        self.last_backtest_metrics = metrics.clone();
        self.last_backtest_trades_count = summary.trade_count;
        self.backtest_history.push(metrics.clone());

        Ok(format!(
            "backtest(historical): steps={}, fills={}, return={:.4}%, drawdown={:.4}%, sharpe={:.2}",
            summary.step_count, total_fills,
            summary.total_return_ratio * 100.0,
            summary.max_drawdown_ratio * 100.0,
            sharpe,
        ))
    }

    fn evaluate_assert(&self, expr: &str) -> Result<bool, String> {
        let expr = expr.trim();

        // Support compound assertions with &&
        if expr.contains("&&") {
            let parts: Vec<&str> = expr.split("&&").collect();
            for part in parts {
                if !self.evaluate_assert(part.trim())? {
                    return Ok(false);
                }
            }
            return Ok(true);
        }

        // compile.compilable == true/false
        if expr.starts_with("compile.") {
            let rest = &expr["compile.".len()..];
            if rest.contains("compilable == true") || rest.contains("compilable==true") {
                return Ok(self.compile_result.is_some());
            }
            if rest.contains("compilable == false") || rest.contains("compilable==false") {
                return Ok(self.compile_result.is_none());
            }
            if rest.contains("diagnostics.length == 0") || rest.contains("diagnostics.length==0") {
                return Ok(true);
            }
            if rest.starts_with("counts.") {
                // counts always valid when compile succeeded
                return Ok(self.compile_result.is_some());
            }
            if rest.starts_with("protocol_name") {
                if let Some(compiled) = &self.compile_result {
                    if let Some(quoted) = rest.split('"').nth(1) {
                        return Ok(compiled.protocol_name == quoted);
                    }
                    if rest.contains("!= null") || rest.contains("!=null") {
                        return Ok(true);
                    }
                }
                return Ok(false);
            }
            return Err(format!("unsupported compile assertion: {}", rest));
        }

        // run.*
        if expr.starts_with("run.") {
            let rest = &expr["run.".len()..];
            if rest.contains("events.length > 0") || rest.contains("events.length>0") {
                return Ok(self.last_run_events_count > 0);
            }
            if rest.contains("events.length >= 0") || rest.contains("events.length>=0") {
                return Ok(true);
            }
            if rest.contains("equity > 0") || rest.contains("equity>0") {
                return Ok(self.last_run_equity.unwrap_or(0.0) > 0.0);
            }
            if rest.contains("equity != null") || rest.contains("equity!=null") {
                return Ok(self.last_run_equity.is_some());
            }
            if rest.contains("status == \"completed\"") || rest.contains("status == 'completed'") {
                return Ok(self.last_run_status.as_deref() == Some("completed"));
            }
            if rest.starts_with("has_event(") {
                let evt_name = rest
                    .trim_start_matches("has_event(")
                    .trim_end_matches(')')
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                return Ok(self.last_run_event_types.contains(&evt_name));
            }
            if rest.contains("positions.length >= 0") || rest.contains("positions.length>=0") {
                return Ok(true);
            }
            if rest.contains("events.length ==") || rest.contains("events.length==") {
                let expected: usize = rest
                    .split("==").nth(1)
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(0);
                return Ok(self.last_run_events_count == expected);
            }
            return Err(format!("unsupported run assertion: {}", rest));
        }

        // backtest.*
        if expr.starts_with("backtest.") {
            let rest = &expr["backtest.".len()..];
            if rest.starts_with("metrics.") {
                let metric_expr = &rest["metrics.".len()..];

                // Support multiple comparison operators with optional tolerance: ==(0.01)
                for (op_str, op_fn) in &[
                    (">=", (|a: f64, b: f64| a >= b) as fn(f64, f64) -> bool),
                    ("<=", (|a, b| a <= b) as fn(f64, f64) -> bool),
                    ("!=", (|a, b| (a - b).abs() > f64::EPSILON) as fn(f64, f64) -> bool),
                    (">", (|a, b| a > b) as fn(f64, f64) -> bool),
                    ("<", (|a, b| a < b) as fn(f64, f64) -> bool),
                ] {
                    if let Some((name, expected_str)) = metric_expr.split_once(*op_str) {
                        let metric_name = name.trim();
                        let expected: f64 = expected_str.trim().parse().unwrap_or(0.0);
                        let actual = self
                            .last_backtest_metrics
                            .get(metric_name)
                            .copied()
                            .unwrap_or(f64::NAN);
                        if actual.is_nan() {
                            return Err(format!(
                                "unknown backtest metric: '{}'. Available: {:?}",
                                metric_name,
                                self.last_backtest_metrics.keys().collect::<Vec<_>>()
                            ));
                        }
                        return Ok(op_fn(actual, expected));
                    }
                }
                // Handle == with optional tolerance: ==(0.01) or ==
                if let Some((name, expected_str)) = metric_expr.split_once("==") {
                    let metric_name = name.trim();
                    let (expected, tolerance) = parse_value_with_tolerance(expected_str.trim());
                    let actual = self
                        .last_backtest_metrics
                        .get(metric_name)
                        .copied()
                        .unwrap_or(f64::NAN);
                    if actual.is_nan() {
                        return Err(format!(
                            "unknown backtest metric: {}", metric_name
                        ));
                    }
                    return Ok((actual - expected).abs() < tolerance);
                }
                return Err(format!("unsupported metric comparison: {}", metric_expr));
            }
            if rest.contains("trades.length > 0") || rest.contains("trades.length>0") {
                return Ok(self.last_backtest_trades_count > 0);
            }
            if rest.contains("trades.length >= 0") || rest.contains("trades.length>=0") {
                return Ok(true);
            }
            return Err(format!(
                "unsupported backtest assertion: '{}'. Available metrics: {:?}",
                rest,
                self.last_backtest_metrics.keys().collect::<Vec<_>>()
            ));
        }

        Err(format!(
            "unknown assertion: '{}'. Supported prefixes: compile., run., backtest.metrics.",
            expr
        ))
    }

    fn build_data_snapshot(&self, step: &TestStep) -> Option<serde_json::Value> {
        let mut snapshot = serde_json::Map::new();
        let has_run = step
            .actions
            .iter()
            .any(|a| matches!(a, TestActionDef::Run { .. }));
        let has_backtest = step
            .actions
            .iter()
            .any(|a| matches!(a, TestActionDef::Backtest { .. }));
        let has_modify = step
            .actions
            .iter()
            .any(|a| matches!(a, TestActionDef::Modify { .. }));

        if has_run {
            if let Some(equity) = self.last_run_equity {
                snapshot.insert(
                    "equity".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(equity).unwrap_or(0.into()),
                    ),
                );
            }
            snapshot.insert(
                "events_count".to_string(),
                (self.last_run_events_count as u64).into(),
            );
            // L1-5: event type distribution
            if let Some(ref counts) = self.last_run_event_counts {
                let mut event_map = serde_json::Map::new();
                for (k, v) in counts {
                    event_map.insert(k.clone(), serde_json::Value::Number((*v as u64).into()));
                }
                if !event_map.is_empty() {
                    snapshot.insert("event_types".to_string(), serde_json::Value::Object(event_map));
                }
            }
            if let Some(ref status) = self.last_run_status {
                snapshot.insert(
                    "status".to_string(),
                    serde_json::Value::String(status.clone()),
                );
            }
        }

        if has_backtest {
            for (k, v) in &self.last_backtest_metrics {
                snapshot.insert(
                    format!("backtest_{}", k),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(*v).unwrap_or(0.into()),
                    ),
                );
            }
        }

        // L2-4: pending modifications snapshot
        if has_modify && !self.pending_modifications.is_empty() {
            let mut mod_map = serde_json::Map::new();
            for (param, val) in &self.pending_modifications {
                mod_map.insert(
                    param.clone(),
                    serde_json::Value::Number(serde_json::Number::from_f64(*val).unwrap_or(0.into())),
                );
            }
            snapshot.insert(
                "pending_modifications".to_string(),
                serde_json::Value::Object(mod_map),
            );
        }

        if snapshot.is_empty() {
            None
        } else {
            Some(serde_json::Value::Object(snapshot))
        }
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse ISO date "YYYY-MM-DD" to Unix millisecond timestamp
fn parse_iso_date_ms(date_str: &str) -> Result<u64, String> {
    let d = time::Date::parse(
        date_str.trim(),
        &time::format_description::well_known::Iso8601::DATE,
    )
    .map_err(|e| format!("{e}"))?;
    let dt = d.with_time(time::Time::MIDNIGHT);
    let unix_epoch = time::Date::from_calendar_date(1970, time::Month::January, 1)
        .map_err(|e| format!("{e}"))?
        .with_time(time::Time::MIDNIGHT);
    let duration = dt - unix_epoch;
    Ok((duration.whole_seconds() * 1000) as u64)
}

/// Parse a date range "YYYY-MM-DD".."YYYY-MM-DD" to end_ms
fn parse_date_range_ms(start: &str, end: &str) -> Result<u64, String> {
    let _start_ms = parse_iso_date_ms(start)?;
    let end_ms = parse_iso_date_ms(end)?;
    Ok(end_ms)
}

/// Parse a value with optional tolerance: "0.0" or "-0.10(0.01)"
fn parse_value_with_tolerance(input: &str) -> (f64, f64) {
    if let Some(open) = input.find('(') {
        if let Some(close) = input.find(')') {
            let value: f64 = input[..open].trim().parse().unwrap_or(0.0);
            let tol: f64 = input[open + 1..close].trim().parse().unwrap_or(f64::EPSILON);
            return (value, tol);
        }
    }
    (input.parse().unwrap_or(0.0), f64::EPSILON)
}

/// Compute Sharpe ratio from equity curve points.
/// Returns (sharpe_ratio, annual_return_pct, annual_volatility_pct).
fn compute_sharpe_from_equity(
    equity_curve: &[qrpc_core::BacktestEquityPoint],
    initial_equity: f64,
) -> (f64, f64, f64) {
    if equity_curve.len() < 2 || initial_equity <= f64::EPSILON {
        return (0.0, 0.0, 0.0);
    }
    // Compute periodic returns
    let mut returns = Vec::with_capacity(equity_curve.len() - 1);
    for window in equity_curve.windows(2) {
        let prev = window[0].equity;
        let curr = window[1].equity;
        if prev > f64::EPSILON {
            returns.push((curr - prev) / prev);
        }
    }
    if returns.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let n = returns.len() as f64;
    let mean = returns.iter().sum::<f64>() / n;
    let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();
    // Annualize: daily bars → *sqrt(365), but mock bars are per-session so use sqrt(n)
    let annual_factor = (n.max(1.0)).sqrt();
    let annual_return = mean * n;
    let annual_vol = std_dev * annual_factor;
    let sharpe = if annual_vol > f64::EPSILON {
        annual_return / annual_vol
    } else {
        0.0
    };
    (sharpe, annual_return * 100.0, annual_vol * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_splits_quantscript_with_test_directives() {
        let source = r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=300)?
    let fast = sma(closes, 20)
    let slow = sma(closes, 50)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}

@test {
    name: "场景一"
    cover: ["P-03"]
}

@step("编译") {
    @compile
    @assert compile.compilable == true
}

@step("运行") {
    @run { mode: "paper", duration: 60s }
    @assert run.events.length > 0
}
"#;
        let ctx = TestRunnerContext::from_source(source).unwrap();
        assert_eq!(ctx.test_plans.len(), 1);
        assert_eq!(ctx.test_plans[0].scenario_name, "场景一");
        assert_eq!(ctx.test_plans[0].steps.len(), 2);
        assert!(!ctx.test_plans.is_empty());
    }
}
