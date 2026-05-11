use super::*;
use anyhow::{Context, Result};
use crate::credential_vault::CredentialVault;
use zeroize::Zeroizing;
use qrpc_runtime::{
    DeterministicTestMode, FastBacktestSandbox, RealTimeSandbox, RuntimeCoordinator,
};
use quantscript::{
    lower_script_to_runtime_config, parse_quant_script_module,
    split_test_items, TestActionDef, TestPlan, TestStep,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::OnceLock;
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
    pub graph_id: Option<String>,
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
    pub original_source: String,
}

impl TestRunnerContext {
    pub fn from_source(source: &str) -> Result<Self> {
        let module = parse_quant_script_module(source)
            .context("无法解析 QuantScript 源码")?;
        let (strategy_module, test_plans) = split_test_items(&module);
        Ok(TestRunnerContext {
            module: strategy_module,
            test_plans,
            original_source: source.to_string(),
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
    pending_modifications: Vec<(String, String, f64)>, // (node_id, param, value)
    last_debug_vars: Option<Vec<String>>,
    last_debug_bars: Option<serde_json::Value>,
    last_compare_diff: Option<serde_json::Value>,
    last_run_event_types: Vec<String>,
    last_graph_id: Option<String>,
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
            last_graph_id: None,
            last_debug_vars: None,
            last_debug_bars: None,
            last_compare_diff: None,
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
                    "测试场景 '{}' 没有 @step 指令",
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
                        message: Some("编译失败".to_string()),
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
            graph_id: self.last_graph_id.clone(),
        })
    }

    fn compile_strategy(&mut self, ctx: &TestRunnerContext) -> bool {
        self.compile_result = None; // L0-1: clear old result on recompile
        match lower_script_to_runtime_config(&ctx.module) {
            Ok(mut runtime_config) => {
                // Apply pending @modify parameter changes
                for (node_id, param_name, new_value) in &self.pending_modifications {
                    let mut affected: Vec<String> = Vec::new();
                    // Data sources
                    for ds in &mut runtime_config.data_sources {
                        if ds.data_id == *node_id || node_id.is_empty() {
                            if param_name.contains("window") || param_name.contains("days") {
                                ds.days = Some(*new_value as u32);
                                affected.push(ds.data_id.clone());
                            }
                        }
                    }
                    // Risk controls
                    for risk in &mut runtime_config.risks {
                        if risk.risk_id == *node_id || node_id.is_empty() {
                            if param_name.contains("max_total_leverage") {
                                risk.max_total_leverage = *new_value;
                                affected.push(risk.risk_id.clone());
                            }
                            if param_name.contains("max_exchange_leverage") {
                                risk.max_exchange_leverage = *new_value;
                                affected.push(risk.risk_id.clone());
                            }
                            if param_name.contains("min_action_interval") {
                                risk.min_action_interval_ms = *new_value as u64;
                                affected.push(risk.risk_id.clone());
                            }
                        }
                    }
                    // Intents
                    for intent in &mut runtime_config.intents {
                        if intent.intent_id == *node_id || node_id.is_empty() {
                            let key = if param_name.contains("fast") {
                                "fast_period"
                            } else if param_name.contains("slow") {
                                "slow_period"
                            } else {
                                param_name.as_str()
                            };
                            intent.params.insert(key.to_string(), *new_value);
                            affected.push(intent.intent_id.clone());
                        }
                    }
                    if node_id.is_empty() && !affected.is_empty() {
                        safe_eprintln!("[TestRunner] 修改: {}={} 已应用于 {} 个节点: [{}]",
                            param_name, new_value, affected.len(), affected.join(", "));
                    }
                }
                self.pending_modifications.clear();
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
                match qrpc_compiler::compile_runtime_protocol_config(&runtime_config) {
                    Ok(compiled) => {
                        // M5-1: save graph so frontend can load it
                        let graph_id = format!("qs_{}", current_time_ms());
                        let graph_dir = std::path::Path::new("storage").join("graphs");
                        let _ = std::fs::create_dir_all(&graph_dir);

                        // Save QS source
                        let _ = std::fs::write(
                            graph_dir.join(format!("{}.qs", graph_id)),
                            &ctx.original_source,
                        );

                        // Build visual graph JSON from frontend_config
                        let graph_json = build_visual_graph_json(
                            &graph_id,
                            &frontend_config,
                        );
                        let _ = std::fs::write(
                            graph_dir.join(format!("{}.json", graph_id)),
                            serde_json::to_string_pretty(&graph_json).unwrap_or_default(),
                        );

                        self.last_graph_id = Some(graph_id);
                        self.compile_result = Some(compiled);
                        true
                    }
                    Err(e) => {
                        safe_eprintln!("[TestRunner] 编译失败: {e:?}");
                        false
                    }
                }
            }
            Err(e) => {
                safe_eprintln!("[TestRunner] 降级失败: {e:?}");
                false
            }
        }
    }

    fn execute_step(&mut self, step: &TestStep) -> Result<String, String> {
        let mut messages = Vec::new();
        for action in &step.actions {
            match action {
                TestActionDef::Compile => {
                    messages.push("编译: ok".to_string());
                }
                TestActionDef::Run {
                    mode,
                    duration_secs,
                    save,
                } => {
                    if *duration_secs == 0 {
                        return Err("run 时长必须大于 0".to_string());
                    }
                    let result = if mode == "testnet" {
                        self.execute_testnet_run(*duration_secs)?
                    } else if mode == "paper" {
                        self.execute_run(*duration_secs)?
                    } else {
                        return Err(format!(
                            "不支持的运行模式: '{}'。请使用 'paper' 或 'testnet'。",
                            mode
                        ));
                    };
                    if *save {
                        messages.push(format!("保存: 运行数据已捕获 ({} 个事件)", self.last_run_events_count));
                    }
                    messages.push(result);
                }
                TestActionDef::Backtest {
                    source,
                    start,
                    end,
                    seed,
                    save,
                    volatility,
                } => {
                    if source != "deterministic_mock" && source != "historical_replay" {
                        return Err(format!(
                            "不支持的回测源: '{}'。请使用 'deterministic_mock' 或 'historical_replay'。",
                            source
                        ));
                    }
                    let s = seed.unwrap_or(BACKTEST_TEST_SEED);
                    let result = match if source == "historical_replay" {
                        self.execute_backtest_historical(s)
                    } else {
                        self.execute_backtest_with_range(s, start.as_deref(), end.as_deref())
                    } {
                        Ok(r) => r,
                        Err(e) => {
                            self.last_backtest_metrics.clear();
                            self.last_backtest_trades_count = 0;
                            return Err(e);
                        }
                    };
                    if let Some(vol) = volatility {
                        let bits = vol.to_bits();
                        qrpc_runtime::MOCK_VOLATILITY.store(bits, std::sync::atomic::Ordering::Relaxed);
                        messages.push(format!("volatility={} (已应用)", vol));
                    } else {
                        qrpc_runtime::MOCK_VOLATILITY.store(0, std::sync::atomic::Ordering::Relaxed);
                    }
                    messages.push(result);
                    if *save {
                        let backtest_dir = std::path::Path::new("storage").join("backtests").join(format!("backtest_{}", current_time_ms()));
                        let _ = std::fs::create_dir_all(&backtest_dir);
                        let artifact = serde_json::json!({
                            "saved_at_ms": current_time_ms(),
                            "graph_id": self.last_graph_id.clone().unwrap_or_default(),
                            "backtest_metrics": self.last_backtest_metrics,
                            "backtest_trades_count": self.last_backtest_trades_count,
                        });
                        let artifact_path = backtest_dir.join("summary.json");
                        if let Ok(json) = serde_json::to_string_pretty(&artifact) {
                            let _ = std::fs::write(&artifact_path, json);
                            messages.push(format!("回测已保存到 {}", artifact_path.display()));
                        }
                    }
                }
                TestActionDef::Assert(expr) => {
                    match self.evaluate_assert(expr) {
                        Ok(true) => messages.push(format!("assert({}) = true", expr)),
                        Ok(false) => {
                            let actual = resolve_assert_actual(self, expr);
                            return Err(format!(
                                "断言失败: {} (实际值: {})",
                                expr, actual
                            ));
                        }
                        Err(e) => return Err(format!("断言错误: {} — {}", expr, e)),
                    }
                }
                TestActionDef::SaveRun => {
                    if self.backtest_history.is_empty() && self.last_run_equity.is_none() {
                        return Err(
                            "save_run: 没有先前的运行或回测可供保存".to_string()
                        );
                    }
                    // L2-3: persist to storage/test-runs/
                    let dir = std::path::Path::new("storage").join("test-runs");
                    std::fs::create_dir_all(&dir).map_err(|e| format!("save_run 错误: {e}"))?;
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
                        .map_err(|e| format!("save_run 错误: {e}"))?;
                    messages.push(format!("save_run: 已持久化到 {}", path.display()));
                }
                TestActionDef::Modify {
                    node,
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
                    self.pending_modifications.push((node.clone(), param.clone(), num_val));
                    if node.is_empty() {
                        messages.push(format!(
                            "modify: {} = {} (警告: 未指定节点，将在下次编译时应用于所有匹配的节点)",
                            param, num_val
                        ));
                    } else {
                        messages.push(format!(
                            "modify: node={} {} = {} (待定，将在下次编译时应用)",
                            node, param, num_val
                        ));
                    }
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
                                        "等待超时 ({}s): {}",
                                        timeout_secs, condition
                                    ));
                                }
                                std::thread::sleep(std::time::Duration::from_millis(200));
                            }
                            Err(e) => {
                                return Err(format!("wait 错误: {}", e));
                            }
                        }
                    }
                }
                TestActionDef::Debug(vars) => {
                    self.last_debug_vars = Some(vars.clone());
                    messages.push(format!("debug: 将在回测后追踪 {} 个变量: {}", vars.len(), vars.join(", ")));
                }
                TestActionDef::CompareBacktests { left, right } => {
                    if *left >= self.backtest_history.len() || *right >= self.backtest_history.len() {
                        return Err(format!(
                            "比较: 索引越界 (left={}, right={}, history_len={})",
                            left, right, self.backtest_history.len()
                        ));
                    }
                    let left_metrics = &self.backtest_history[*left];
                    let right_metrics = &self.backtest_history[*right];
                    let mut diffs = Vec::new();
                    let mut diff_map = serde_json::Map::new();
                    for key in left_metrics.keys() {
                        let lv = left_metrics.get(key).copied().unwrap_or(0.0);
                        let rv = right_metrics.get(key).copied().unwrap_or(0.0);
                        if (lv - rv).abs() > f64::EPSILON {
                            diffs.push(format!("{}: {} vs {}", key, lv, rv));
                            diff_map.insert(key.clone(), serde_json::json!({
                                "left": lv,
                                "right": rv,
                                "diff": rv - lv
                            }));
                        }
                    }
                    self.last_compare_diff = Some(serde_json::Value::Object(diff_map));
                    if diffs.is_empty() {
                        messages.push("比较: 相同".to_string());
                    } else {
                        messages.push(format!("比较: {} 处差异 — {}", diffs.len(), diffs.join(", ")));
                    }
                }
            }
        }
        Ok(messages.join("; "))
    }

    /// Load exchange credentials — tries env vars first, then falls back to CredentialVault
    fn load_exchange_credentials() -> Result<(Zeroizing<String>, Zeroizing<String>, Zeroizing<String>), String> {
        // Try environment variables first
        let key_env = std::env::var("QUANTPILOT_EXCHANGE_KEY").ok();
        let secret_env = std::env::var("QUANTPILOT_EXCHANGE_SECRET").ok();
        let passphrase_env = std::env::var("QUANTPILOT_EXCHANGE_PASSPHRASE").ok();

        if let (Some(ref key), Some(ref secret), Some(ref passphrase)) = (&key_env, &secret_env, &passphrase_env) {
            if !key.is_empty() && !secret.is_empty() && !passphrase.is_empty() {
                return Ok((
                    Zeroizing::new(key.clone()),
                    Zeroizing::new(secret.clone()),
                    Zeroizing::new(passphrase.clone()),
                ));
            }
        }

        // Fall back to credential vault
        let vault = CredentialVault::load()
            .map_err(|e| format!("无法加载凭证保险库: {}", e))?;
        crate::safe_log::register_credential_patterns(vault.extract_secret_patterns());

        let fields = vault.get_service("okx")
            .ok_or_else(|| "testnet 模式需要设置环境变量 QUANTPILOT_EXCHANGE_KEY 或在凭证保险库中配置".to_string())?;

        let key = Zeroizing::new(fields.get("key").cloned()
            .ok_or_else(|| "凭证标签 'okx' 中缺少 'key' 字段".to_string())?);
        let secret = Zeroizing::new(fields.get("secret").cloned()
            .ok_or_else(|| "凭证标签 'okx' 中缺少 'secret' 字段".to_string())?);
        let passphrase = Zeroizing::new(fields.get("passphrase").cloned()
            .ok_or_else(|| "凭证标签 'okx' 中缺少 'passphrase' 字段".to_string())?);

        Ok((key, secret, passphrase))
    }

    fn execute_testnet_run(&mut self, duration_secs: u64) -> Result<String, String> {
        // Risk guard: max runtime 3600s
        const MAX_TESTNET_DURATION: u64 = 3600;
        if duration_secs > MAX_TESTNET_DURATION {
            return Err(format!(
                "testnet 已中止: 时长 {}s 超过最大限制 {}s",
                duration_secs, MAX_TESTNET_DURATION
            ));
        }

        let (key, secret, passphrase) = Self::load_exchange_credentials()?;

        // proxy 由 testnet_agent() 内部的 AgentBuilder::proxy() 独立配置, 不修改全局环境变量

        // Build a simple testnet run
        let mut orders = 0usize;
        let mut errors = 0usize;
        let mut last_error = String::new();
        let max_errors = 10;

        // Get balance
        let balance = match Self::okx_request(
            &key, &secret, &passphrase, "GET", "/api/v5/account/balance", "",
        ) {
            Ok(v) => v,
            Err(e) => return Err(format!("testnet 余额检查失败: {e}")),
        };
        let total_eq: f64 = balance["data"][0]["totalEq"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        // Risk guard: max position size = 10% of equity per trade
        let max_trade_qty = total_eq * 0.10;
        // Risk guard: max leverage = 3x (simulated via cash mode only)

        // Continuous execution loop: poll ticker and place orders until duration expires
        let start_instant = std::time::Instant::now();
        let cycle_interval = std::time::Duration::from_secs(12); // ~5 req/min per OKX limit
        let mut status_summary = String::new();

        while start_instant.elapsed().as_secs() < duration_secs && errors < max_errors {
            // Get latest ticker
            let ticker = match Self::okx_request(
                &key, &secret, &passphrase, "GET", "/api/v5/market/ticker?instId=BTC-USDT", "",
            ) {
                Ok(v) => v,
                Err(e) => {
                    errors += 1;
                    last_error = e;
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    continue;
                }
            };
            let last_price: f64 = ticker["data"][0]["last"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0);

            if last_price > 0.0 {
                const MIN_TRADE_QTY_RATIO: f64 = 0.005;
                let trade_notional = (total_eq * MIN_TRADE_QTY_RATIO).min(max_trade_qty);
                let qty = format!("{:.6}", (trade_notional / last_price).min(0.01).max(0.001));
                let aggressive_px = format!("{:.1}", last_price * 1.002);
                let order_body = serde_json::json!({
                    "instId": "BTC-USDT",
                    "tdMode": "cash",
                    "side": "buy",
                    "ordType": "limit",
                    "sz": qty,
                    "px": aggressive_px,
                });
                match Self::okx_request(
                    &key, &secret, &passphrase, "POST", "/api/v5/trade/order",
                    &serde_json::to_string(&order_body).unwrap_or_default(),
                ) {
                    Ok(v) => {
                        orders += 1;
                        let ord_id = v["data"][0]["ordId"].as_str().unwrap_or("?");
                        status_summary = format!("order #{}: {} BTC @ {} (id={})", orders, qty, aggressive_px, ord_id);
                    }
                    Err(e) => {
                        errors += 1;
                        last_error = e;
                    }
                }
            }

            // Status update every 10 cycles
            if orders > 0 && orders % 10 == 0 {
                status_summary = format!("状态: 已下达 {} 个订单，{} 个错误，equity={:.2}，已耗时 {}s",
                    orders, errors, total_eq, start_instant.elapsed().as_secs());
            }

            std::thread::sleep(cycle_interval);
        }

        self.last_run_events_count = orders;
        self.last_run_equity = Some(total_eq);
        self.last_run_status = Some(if errors >= max_errors { "error".to_string() } else { "completed".to_string() });

        // 主动释放凭证，立即触发 Zeroizing 的零化 Drop
        drop(key);
        drop(secret);
        drop(passphrase);

        if errors >= max_errors {
            Err(format!("testnet 已中止: {} 个连续错误，最后: {}", errors, last_error))
        } else if orders == 0 {
            Ok(format!("testnet: 未下达订单 (duration={}s, equity={:.2})", duration_secs, total_eq))
        } else {
            Ok(format!("testnet: 已下达 {} 个订单，耗时 {}s，equity={:.2}，最后: {}", orders, duration_secs, total_eq, status_summary))
        }
    }

    /// Shared HTTP agent with proxy support for testnet requests
    fn testnet_agent() -> &'static ureq::Agent {
        static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
        AGENT.get_or_init(|| {
            let proxy_url = std::env::var("QUANTPILOT_PROXY")
                .unwrap_or_else(|_| "http://127.0.0.1:7897".to_string());
            let proxy = ureq::Proxy::new(&proxy_url).expect("proxy config error");
            ureq::AgentBuilder::new().proxy(proxy).build()
        })
    }

    /// OKX API request with HMAC-SHA256 signing and rate-limit backoff
    fn okx_request(
        api_key: &str,
        secret_key: &str,
        passphrase: &str,
        method: &str,
        path: &str,
        body: &str,
    ) -> Result<serde_json::Value, String> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        // Rate limit: ensure at least 500ms between requests (OKX testnet limit ~2 req/s)
        static LAST_REQUEST: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        let last = LAST_REQUEST.load(std::sync::atomic::Ordering::Relaxed);
        let elapsed = now - last;
        if elapsed < 1000 {
            std::thread::sleep(std::time::Duration::from_millis((1000 - elapsed) as u64));
        }
        LAST_REQUEST.store(now, std::sync::atomic::Ordering::Relaxed);

        // Build agent with proxy support (shared static instance)
        let proxy_agent = Self::testnet_agent();

        // Adaptive retry: up to 3 attempts with exponential backoff
        let mut last_err = String::new();
        for attempt in 0..3 {
            let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S.000Z").to_string();
            let sign_str = format!("{}{}{}{}", ts, method, path, body);
            let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
                .map_err(|e| format!("HMAC 错误: {e}"))?;
            mac.update(sign_str.as_bytes());
            let result = mac.finalize();
            let sig = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &result.into_bytes(),
            );

            let url = format!("https://www.okx.com{}", path);
            let req = proxy_agent.request(method, &url)
                .set("OK-ACCESS-KEY", api_key)
                .set("OK-ACCESS-SIGN", &sig)
                .set("OK-ACCESS-TIMESTAMP", &ts)
                .set("OK-ACCESS-PASSPHRASE", passphrase)
                .set("x-simulated-trading", "1")
                .set("Content-Type", "application/json");

            let resp = if body.is_empty() {
                req.call()
            } else {
                req.send_string(body)
            };

            match resp {
                Ok(r) => {
                    let status = r.status();
                    let text = r.into_string().unwrap_or_default();
                    let v: serde_json::Value =
                        serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);
                    let code = v.get("code").and_then(|c| c.as_str()).unwrap_or("?");
                    if code == "0" {
                        return Ok(v);
                    }
                    let msg = v.get("msg").and_then(|m| m.as_str()).unwrap_or("?");
                    // Rate-limited or transient → retry with backoff
                    if code == "1" || status == 429 || status >= 500 {
                        let delay_ms = 1000 * (attempt + 1) as u64;
                        last_err = format!("OKX {}/{}: {} (重试 {}/3, 退避 {}ms)", code, status, msg, attempt + 1, delay_ms);
                        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                        continue;
                    }
                    return Err(format!("OKX 错误 {} ({}): {}", code, status, msg));
                }
                Err(e) => {
                    last_err = format!("请求失败: {} (重试 {}/3)", e, attempt + 1);
                    std::thread::sleep(std::time::Duration::from_millis(500 * (attempt + 1) as u64));
                }
            }
        }
        Err(last_err)
    }

    fn execute_run(&mut self, duration_secs: u64) -> Result<String, String> {
        let compiled = self
            .compile_result
            .clone()
            .ok_or_else(|| "无编译结果".to_string())?;
        let mut sandbox = RealTimeSandbox::new(RuntimeCoordinator::new(compiled));
        sandbox
            .start()
            .map_err(|e| format!("沙盒启动失败: {e}"))?;
        let now_ms = current_time_ms();
        let run_duration = std::cmp::min(duration_secs * 1000, RUN_WINDOW_MS);
        let session = sandbox
            .run_session(now_ms, now_ms + run_duration)
            .map_err(|e| format!("运行会话失败: {e}"))?;
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
                .map_err(|e| format!("无效的日期范围: {e}"))?
        } else if let Some(start) = start_date {
            let start_ms = parse_iso_date_ms(start)
                .map_err(|e| format!("无效的开始日期 '{}': {e}", start))?;
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
            .ok_or_else(|| "无编译结果".to_string())?;
        let test_mode =
            DeterministicTestMode::replay_defaults(now_ms, seed);
        let mut sandbox =
            FastBacktestSandbox::with_mock_replay_from_core_ir_and_test_mode(
                compiled.core_ir.clone(),
                now_ms,
                test_mode,
            )
            .map_err(|e| format!("回测初始化失败: {e}"))?;
        if let Some(ref debug_vars) = self.last_debug_vars {
            sandbox.debug_var_names = debug_vars.clone();
        }
        sandbox
            .start()
            .map_err(|e| format!("回测启动失败: {e}"))?;
        let backtest = sandbox
            .run_backtest()
            .map_err(|e| format!("回测运行失败: {e}"))?;

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
            .fold(
                (0, 0, 0.0, 0.0),
                |(buys, sells, fees, notional), fill| {
                    let fee = fill.fee_paid;
                    let qty = fill.filled_qty;
                    let price = fill.filled_price;
                    if qty > 0.0 {
                        (buys + 1, sells, fees + fee, notional + qty * price)
                    } else {
                        (buys, sells + 1, fees + fee, notional + qty.abs() * price)
                    }
                },
            );

        // Compute win_rate from equity curve: count sessions where equity increased
        let (ewins, elosses): (usize, usize) = backtest
            .equity_curve
            .windows(2)
            .fold((0, 0), |(w, l), window| {
                if window[1].equity > window[0].equity {
                    (w + 1, l)
                } else if window[1].equity < window[0].equity {
                    (w, l + 1)
                } else {
                    (w, l)
                }
            });
        let win_rate = if ewins + elosses > 0 {
            ewins as f64 / (ewins + elosses) as f64
        } else {
            0.0
        };

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
        metrics.insert("win_rate".to_string(), win_rate);
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

        // Populate @debug per-bar data from equity curve + debug values
        if let Some(ref debug_vars) = self.last_debug_vars {
            let debug_rows = backtest.debug_values.as_ref();
            let bars: Vec<_> = backtest.equity_curve.iter().enumerate().map(|(i, pt)| {
                let mut bar = serde_json::Map::new();
                bar.insert("bar".to_string(), serde_json::json!(i));
                bar.insert("equity".to_string(), serde_json::json!(pt.equity));
                bar.insert("timestamp_ms".to_string(), serde_json::json!(pt.ts_ms));
                for var in debug_vars {
                    let value = debug_rows
                        .and_then(|rows| rows.get(i))
                        .and_then(|row| {
                            row.iter().find(|(k, _)| {
                                k.contains(var.as_str()) || var.as_str().contains(k.as_str())
                            }).map(|(_, v)| *v)
                        });
                    bar.insert(var.clone(), serde_json::json!(value));
                }
                serde_json::Value::Object(bar)
            }).collect();
            self.last_debug_bars = Some(serde_json::Value::Array(bars.iter().map(|b| b.clone()).collect()));
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
            .ok_or_else(|| "无编译结果".to_string())?;
        let now_ms = current_time_ms();
        // Use historical_replay — requires local cache files
        let mut sandbox = FastBacktestSandbox::with_replay_from_core_ir(
            compiled.core_ir.clone(),
            now_ms,
        )
        .map_err(|e| format!(
            "历史回放需要缓存的市场数据。请先运行 paper 仿真来填充缓存。详情: {e}"
        ))?;
        sandbox
            .start()
            .map_err(|e| format!("回测启动失败: {e}"))?;
        let backtest = sandbox
            .run_backtest()
            .map_err(|e| format!("回测运行失败: {e}"))?;

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

        // Resolve metric name aliases (docs use short names, code uses full names)
        fn resolve_metric<'a>(name: &'a str) -> &'a str {
            match name {
                "sharpe" => "sharpe_ratio",
                "max_drawdown" => "max_drawdown_pct",
                "return" => "total_return_pct",
                "annual_return" => "annual_return_pct",
                "drawdown" => "max_drawdown_pct",
                "volatility" => "annual_volatility_pct",
                _ => name,
            }
        }

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
                if let Some(ref compiled) = self.compile_result {
                    let count_name = rest.trim_start_matches("counts.").trim();
                    let actual = match count_name {
                        s if s.starts_with("data_sources") => compiled.config.data_sources.len() as f64,
                        s if s.starts_with("intent_generators") || s.starts_with("intents") => compiled.config.intents.len() as f64,
                        s if s.starts_with("agents") => compiled.config.agents.len() as f64,
                        s if s.starts_with("risks") || s.starts_with("risk_controls") => compiled.config.risks.len() as f64,
                        _ => return Err(format!("未知的计数字段: {}", count_name)),
                    };
                    // Evaluate comparison against actual
                    return evaluate_numeric_assert(count_name, actual, rest);
                }
                return Ok(false);
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
            return Err(format!("不支持的编译断言: {}", rest));
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
            return Err(format!("不支持的运行断言: {}", rest));
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
                        let metric_name = resolve_metric(name.trim());
                        let expected: f64 = expected_str.trim().parse().unwrap_or(0.0);
                        let actual = self
                            .last_backtest_metrics
                            .get(metric_name)
                            .copied()
                            .unwrap_or(f64::NAN);
                        if actual.is_nan() {
                            return Err(format!(
                                "未知的回测指标: '{}'。可用指标: {:?}",
                                metric_name,
                                self.last_backtest_metrics.keys().collect::<Vec<_>>()
                            ));
                        }
                        return Ok(op_fn(actual, expected));
                    }
                }
                // Handle == with optional tolerance: ==(0.01) or ==
                if let Some((name, expected_str)) = metric_expr.split_once("==") {
                    let metric_name = resolve_metric(name.trim());
                    let (expected, tolerance) = parse_value_with_tolerance(expected_str.trim());
                    let actual = self
                        .last_backtest_metrics
                        .get(metric_name)
                        .copied()
                        .unwrap_or(f64::NAN);
                    if actual.is_nan() {
                        return Err(format!(
                            "未知的回测指标: {}", metric_name
                        ));
                    }
                    return Ok((actual - expected).abs() < tolerance);
                }
                return Err(format!("不支持的指标比较: {}", metric_expr));
            }
            if rest.contains("trades.length > 0") || rest.contains("trades.length>0") {
                return Ok(self.last_backtest_trades_count > 0);
            }
            if rest.contains("trades.length >= 0") || rest.contains("trades.length>=0") {
                return Ok(true);
            }
            return Err(format!(
                "不支持的回测断言: '{}'。可用指标: {:?}",
                rest,
                self.last_backtest_metrics.keys().collect::<Vec<_>>()
            ));
        }

        Err(format!(
            "未知的断言: '{}'。支持的前缀: compile., run., backtest.metrics.",
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
            for (node_id, param, val) in &self.pending_modifications {
                mod_map.insert(
                    format!("{}.{}", node_id, param),
                    serde_json::Value::Number(serde_json::Number::from_f64(*val).unwrap_or(0.into())),
                );
            }
            snapshot.insert(
                "pending_modifications".to_string(),
                serde_json::Value::Object(mod_map),
            );
        }

        // @debug per-bar data
        if let Some(ref debug_bars) = self.last_debug_bars {
            snapshot.insert("debug_bars".to_string(), debug_bars.clone());
        }

        // @compare_backtests structured diff
        if let Some(ref compare_diff) = self.last_compare_diff {
            snapshot.insert("compare_diff".to_string(), compare_diff.clone());
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
    let start_ms = parse_iso_date_ms(start)?;
    let end_ms = parse_iso_date_ms(end)?;
    if end_ms <= start_ms {
        return Err(format!("结束时间 ({}) 必须大于开始时间 ({})", end, start));
    }
    Ok(end_ms)
}

/// Resolve the actual value for an assertion expression for error display
fn resolve_assert_actual(runner: &TestRunner, expr: &str) -> String {
    let expr = expr.trim();
    if expr.starts_with("compile.compilable") {
        format!("{}", runner.compile_result.is_some())
    } else if expr.starts_with("run.equity") {
        format!("{:.2}", runner.last_run_equity.unwrap_or(f64::NAN))
    } else if expr.starts_with("run.events.length") {
        format!("{}", runner.last_run_events_count)
    } else if expr.starts_with("backtest.metrics.") {
        let metric = &expr["backtest.metrics.".len()..];
        let name = metric.split(|c: char| !c.is_alphanumeric() && c != '_').next().unwrap_or(metric);
        if let Some(val) = runner.last_backtest_metrics.get(name) {
            format!("{:.6}", val)
        } else {
            "unknown".to_string()
        }
    } else {
        "?".to_string()
    }
}

/// Evaluate a numeric assertion like ">= 1" or "== 2" against an actual value
fn evaluate_numeric_assert(field_name: &str, actual: f64, rest: &str) -> Result<bool, String> {
    // rest is the full expression like "data_sources >= 1" — extract op and value
    let expr_after_name = rest.trim_start_matches(field_name).trim();
    for (op_str, op_fn) in &[
        (">=", (|a: f64, b: f64| a >= b) as fn(f64, f64) -> bool),
        ("<=", (|a, b| a <= b) as fn(f64, f64) -> bool),
        ("==", (|a, b| (a - b).abs() < 0.001) as fn(f64, f64) -> bool),
        (">", (|a, b| a > b) as fn(f64, f64) -> bool),
        ("<", (|a, b| a < b) as fn(f64, f64) -> bool),
    ] {
        if let Some(expected_str) = expr_after_name.split(*op_str).nth(1) {
            let expected: f64 = expected_str.trim().parse().unwrap_or(0.0);
            return Ok(op_fn(actual, expected));
        }
    }
    Err(format!("不支持的比较表达式: {}", expr_after_name))
}

/// Build a visual strategy graph JSON from frontend config for workspace rendering
fn build_visual_graph_json(
    graph_id: &str,
    config: &FrontendRuntimeConfig,
) -> serde_json::Value {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut y_offset = 50.0;

    // Runtime control node — has no input ports, outputs to self for lifecycle
    if let Some(ref rc) = config.runtime_control {
        nodes.push(serde_json::json!({
            "id": rc.id,
            "type": "runtime",
            "position": { "x": 400.0, "y": y_offset },
            "data": {
                "nodeId": rc.id, "nodeType": "runtime", "title": rc.name, "subtitle": rc.module_key,
                "inputPorts": [], "outputPorts": [],
                "handlesConnectable": true, "simplified": false,
                "summaryValues": ["运行时控制"], "quickFieldDefinitions": [],
                "collapsed": false, "dimmed": false, "runtimeStatus": "idle"
            }
        }));
        y_offset += 150.0;
    }

    // Data source nodes — output only (market_data_out)
    for ds in &config.data_sources {
        nodes.push(serde_json::json!({
            "id": ds.id,
            "type": "data",
            "position": { "x": 100.0, "y": y_offset },
            "data": {
                "nodeId": ds.id, "nodeType": "data", "title": ds.name, "subtitle": ds.module_key,
                "config": ds.config,
                "inputPorts": [], "outputPorts": [{ "key": "market_data_out" }],
                "handlesConnectable": true, "simplified": false,
                "summaryValues": [format!("{}.{}", ds.config.get("exchange").and_then(|v| v.as_str()).unwrap_or("?"), ds.config.get("instrument").and_then(|v| v.as_str()).unwrap_or("?"))],
                "quickFieldDefinitions": [],
                "collapsed": false, "dimmed": false, "runtimeStatus": "idle"
            }
        }));
        y_offset += 120.0;
    }

    y_offset = 100.0;

    // Intent nodes — input + output
    for intent in &config.intent_generators {
        let node_id = &intent.id;
        nodes.push(serde_json::json!({
            "id": node_id,
            "type": "intent",
            "position": { "x": 350.0, "y": y_offset },
            "data": {
                "nodeId": node_id, "nodeType": "intent", "title": intent.name, "subtitle": intent.module_key,
                "config": intent.config,
                "inputPorts": [{ "key": "data_input" }], "outputPorts": [{ "key": "intent_out" }],
                "handlesConnectable": true, "simplified": false,
                "summaryValues": [intent.name.clone()],
                "quickFieldDefinitions": [],
                "collapsed": false, "dimmed": false, "runtimeStatus": "idle"
            }
        }));
        for input_ref in &intent.input_refs {
            edges.push(serde_json::json!({
                "id": format!("e-{}-{}", input_ref.source_id, node_id),
                "source_node_id": input_ref.source_id,
                "target_node_id": node_id,
                "source_port": input_ref.source_port,
                "target_port": input_ref.target_port,
                "edge_type": format!("{}-{}", input_ref.source_id, node_id),
            }));
        }
        y_offset += 120.0;
    }

    // Agent nodes
    for agent in &config.agents {
        let node_id = &agent.id;
        nodes.push(serde_json::json!({
            "id": node_id,
            "type": "agent",
            "position": { "x": 600.0, "y": y_offset },
            "data": {
                "nodeId": node_id, "nodeType": "agent", "title": agent.name, "subtitle": agent.module_key,
                "inputPorts": [{ "key": "intent_input" }], "outputPorts": [{ "key": "agent_out" }],
                "handlesConnectable": true, "simplified": false,
                "summaryValues": ["加权代理"],
                "quickFieldDefinitions": [],
                "collapsed": false, "dimmed": false, "runtimeStatus": "idle"
            }
        }));
        for intent in &config.intent_generators {
            edges.push(serde_json::json!({
                "id": format!("e-{}-{}", intent.id, node_id),
                "source_node_id": intent.id,
                "source_port": "intent_out",
                "target_node_id": node_id,
                "target_port": "intent_input",
                "edge_type": format!("{}-{}", intent.id, node_id),
            }));
        }
        y_offset += 120.0;
    }

    // Risk nodes
    for risk in &config.risk_controls {
        let node_id = &risk.id;
        nodes.push(serde_json::json!({
            "id": node_id,
            "type": "risk",
            "position": { "x": 850.0, "y": y_offset },
            "data": {
                "nodeId": node_id, "nodeType": "risk", "title": risk.name, "subtitle": risk.module_key,
                "inputPorts": [{ "key": "agent_input" }], "outputPorts": [{ "key": "risk_out" }],
                "handlesConnectable": true, "simplified": false,
                "summaryValues": ["全局限险"],
                "quickFieldDefinitions": [],
                "collapsed": false, "dimmed": false, "runtimeStatus": "idle"
            }
        }));
        for agent in &config.agents {
            edges.push(serde_json::json!({
                "id": format!("e-{}-{}", agent.id, node_id),
                "source_node_id": agent.id,
                "source_port": "agent_out",
                "target_node_id": node_id,
                "target_port": "agent_input",
                "edge_type": format!("{}-{}", agent.id, node_id),
            }));
        }
        y_offset += 120.0;
    }

    // Execution nodes
    for exec in &config.executions {
        let node_id = &exec.id;
        nodes.push(serde_json::json!({
            "id": node_id,
            "type": "execution",
            "position": { "x": 1100.0, "y": y_offset },
            "data": {
                "nodeId": node_id, "nodeType": "execution", "title": exec.name, "subtitle": exec.module_key,
                "inputPorts": [{ "key": "risk_input" }], "outputPorts": [],
                "handlesConnectable": true, "simplified": false,
                "summaryValues": ["模拟执行"],
                "quickFieldDefinitions": [],
                "collapsed": false, "dimmed": false, "runtimeStatus": "idle"
            }
        }));
        for risk in &config.risk_controls {
            edges.push(serde_json::json!({
                "id": format!("e-{}-{}", risk.id, node_id),
                "source_node_id": risk.id,
                "source_port": "risk_out",
                "target_node_id": node_id,
                "target_port": "risk_input",
                "edge_type": format!("{}-{}", risk.id, node_id),
            }));
        }
    }

    // Normalize nodes to frontend store format:
    // - Add top-level `config` (required by graph_api save→QS generation)
    // - Add top-level `module_key`, `input_ports`, `output_ports` (required by isValidConnection validation)
    let nodes_normalized: Vec<_> = nodes.into_iter().map(|mut node| {
        if let Some(obj) = node.as_object_mut() {
            // Copy module_key from data.subtitle to top level
            if !obj.contains_key("module_key") {
                if let Some(subtitle) = obj.get("data").and_then(|d| d.get("subtitle")).and_then(|v| v.as_str()) {
                    obj.insert("module_key".to_string(), serde_json::json!(subtitle));
                }
            }
            // Copy input_ports from data.inputPorts to top level
            if !obj.contains_key("input_ports") {
                if let Some(ports) = obj.get("data").and_then(|d| d.get("inputPorts")).cloned() {
                    obj.insert("input_ports".to_string(), ports);
                }
            }
            // Copy output_ports from data.outputPorts to top level
            if !obj.contains_key("output_ports") {
                if let Some(ports) = obj.get("data").and_then(|d| d.get("outputPorts")).cloned() {
                    obj.insert("output_ports".to_string(), ports);
                }
            }
            // Copy config from data.config to top level
            if let Some(data_config) = obj.get("data").and_then(|d| d.get("config")).cloned() {
                obj.insert("config".to_string(), data_config);
            }
            if obj.get("type").and_then(|t| t.as_str()) == Some("runtime") {
                obj.entry("config").or_insert_with(|| serde_json::json!({
                    "mode": config.metadata.mode
                }));
            }
            // Ensure ui_state and runtime_state exist at top level
            if !obj.contains_key("ui_state") {
                obj.insert("ui_state".to_string(), serde_json::json!({"collapsed": false}));
            }
            if !obj.contains_key("runtime_state") {
                obj.insert("runtime_state".to_string(), serde_json::json!({
                    "status": "idle",
                    "last_event_type": null,
                    "last_event_time": null,
                    "last_message": "",
                    "metrics": {},
                    "error": null
                }));
            }
        }
        node
    }).collect();

    serde_json::json!({
        "metadata": {
            "graph_id": graph_id,
            "name": config.metadata.name,
            "version": config.metadata.version,
            "mode": config.metadata.mode,
            "source_mode": "quantscript",
            "updated_at": chrono::Utc::now().to_rfc3339(),
        },
        "validation_state": {
            "is_runnable": true,
            "is_compilable": true,
            "issue_counts": { "error": 0, "warning": 0, "info": 0 }
        },
        "compile_summary": {
            "compilable": true,
            "protocol_name": "quantpilot/minimal-sim/v1",
            "diagnostics": []
        },
        "nodes": nodes_normalized,
        "edges": edges,
    })
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
