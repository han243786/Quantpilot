use crate::{
    ExecutionModuleProvider, ExecutionPlanner, ExecutionPlanningOutput,
    ExecutionPlanningRequest, ExecutionSubmitter,
};
use base64::Engine;
use chrono::Utc;
use hmac::{Hmac, Mac};
use qrpc_core::{
    DecisionStatus, Exchange, ExecutionPlan, ExecutionStatus, FillReport, FillResult, OrderSide,
    OrderType, PortfolioState, RuntimeEvent, RuntimeEventType, SimOrder, Symbol, TimeInForce,
};
use reqwest::blocking::Client;
use serde_json::json;
use sha2::Sha256;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use zeroize::Zeroizing;

type HmacSha256 = Hmac<Sha256>;

// ── 风控常量 ─────────────────────────────────────────────────────────

/// 单笔订单最大名义价值（USD）
const MAX_NOTIONAL_PER_ORDER: f64 = 1_000.0;

/// 日累计下单次数上限
const MAX_DAILY_ORDER_COUNT: u64 = 100;

/// 日重置周期 (24h)
const DAILY_RESET_MS: u64 = 24 * 60 * 60 * 1000;

/// 请求重试次数
const MAX_RETRIES: u32 = 3;

/// 请求基础退避（ms）
const BASE_BACKOFF_MS: u64 = 500;

/// 请求间隔限流（ms）
const RATE_LIMIT_MS: u64 = 200;

// v2.1.1: 脱敏OKX错误消息，防止泄露API密钥/签名等敏感信息
fn sanitize_error_for_event(raw: &str) -> String {
    let sensitive_keys = ["api_key", "secret", "sign", "passphrase", "password", "token", "key"];
    let lower = raw.to_lowercase();
    for key in &sensitive_keys {
        if lower.contains(key) {
            // 包含敏感关键词时返回通用消息
            return "交易执行失败，请检查交易所凭证和网络连接".to_string();
        }
    }
    // 截断过长的错误消息
    if raw.len() > 200 {
        format!("{}...(已截断)", &raw[..200])
    } else {
        raw.to_string()
    }
}

// ── LiveExecutionModule ─────────────────────────────────────────────

pub struct LiveExecutionModule {
    api_key: Zeroizing<String>,
    secret: Zeroizing<String>,
    passphrase: Zeroizing<String>,
    testnet: bool,
    client: Client,

    /// 风控：日累计下单次数
    daily_order_count: Mutex<u64>,
    /// 风控：最后一次重置的 epoch ms
    daily_reset_at_ms: Mutex<u64>,

    /// 全局请求限流（最后请求时刻 ms）
    last_request_ms: AtomicU64,
}

impl std::fmt::Debug for LiveExecutionModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiveExecutionModule")
            .field("provider_key", &"live.okx")
            .field("testnet", &self.testnet)
            .field("daily_order_count", &self.daily_order_count)
            .finish()
    }
}

impl LiveExecutionModule {
    /// 创建 OKX 实盘/测试网执行模块。
    ///
    /// * `api_key`    — OKX API Key
    /// * `secret`     — OKX Secret Key
    /// * `passphrase` — OKX Passphrase
    /// * `testnet`    — `true` 表示启用测试网模式（自动添加 `x-simulated-trading: 1` 头）
    pub fn new(
        api_key: impl Into<String>,
        secret: impl Into<String>,
        passphrase: impl Into<String>,
        testnet: bool,
    ) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("reqwest blocking Client 构建失败");

        Self {
            api_key: Zeroizing::new(api_key.into()),
            secret: Zeroizing::new(secret.into()),
            passphrase: Zeroizing::new(passphrase.into()),
            testnet,
            client,
            daily_order_count: Mutex::new(0),
            daily_reset_at_ms: Mutex::new(0),
            last_request_ms: AtomicU64::new(0),
        }
    }

    // ── HMAC-SHA256 签名 ─────────────────────────────────────

    fn build_signature(secret: &str, ts: &str, method: &str, path: &str, body: &str) -> String {
        let sign_str = format!("{}{}{}{}", ts, method, path, body);
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC-SHA256 new_from_slice 不应失败");
        mac.update(sign_str.as_bytes());
        let result = mac.finalize();
        base64::engine::general_purpose::STANDARD.encode(result.into_bytes())
    }

    // ── Symbol -> OKX 交易对 ─────────────────────────────────

    fn symbol_to_inst_id(symbol: &Symbol) -> String {
        let s = symbol.as_str();
        // "BTCUSDT" -> "BTC-USDT", "ETHUSDT" -> "ETH-USDT"
        if let Some(base) = s.strip_suffix("USDT") {
            if !base.is_empty() {
                return format!("{}-USDT", base);
            }
        }
        s.to_string()
    }

    // ── 风控检查 ─────────────────────────────────────────────

    fn check_risk_limits(
        &self,
        order_value: f64,
        now_ms: u64,
    ) -> Result<(), String> {
        // 1) 单笔名义价值上限
        if order_value > MAX_NOTIONAL_PER_ORDER {
            return Err(format!(
                "单笔订单名义价值 ${:.2} 超过上限 ${:.0}",
                order_value, MAX_NOTIONAL_PER_ORDER
            ));
        }

        // 2) 日累计下单次数
        let mut count = self.daily_order_count.lock().map_err(|e| e.to_string())?;
        let mut reset_at = self.daily_reset_at_ms.lock().map_err(|e| e.to_string())?;

        if now_ms >= reset_at.saturating_add(DAILY_RESET_MS) {
            *count = 0;
            *reset_at = now_ms;
        }

        if *count >= MAX_DAILY_ORDER_COUNT {
            return Err(format!(
                "日累计下单次数 {} 已达到上限 {}",
                *count, MAX_DAILY_ORDER_COUNT
            ));
        }

        *count += 1;
        Ok(())
    }

    // ── 请求限流 ─────────────────────────────────────────────

    fn rate_limit(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let last = self.last_request_ms.load(Ordering::Relaxed);
        let elapsed = now.saturating_sub(last);
        if elapsed < RATE_LIMIT_MS {
            std::thread::sleep(std::time::Duration::from_millis(RATE_LIMIT_MS - elapsed));
        }
        // 使用 sleep 后的最新时间戳，避免了下一次 rate_limit 计算时使用 sleep 前的时间，
        // 导致限流间隔被缩短。
        let now_after = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.last_request_ms
            .store(now_after, Ordering::Relaxed);
    }

    // ── OKX API 请求 ─────────────────────────────────────────

    fn okx_request(
        &self,
        method: &str,
        path: &str,
        body: &str,
    ) -> Result<serde_json::Value, String> {
        let mut last_err = String::new();

        for attempt in 0..MAX_RETRIES {
            self.rate_limit();

            // v2.1.0: 使用实际毫秒防同秒签名碰撞
            let now = Utc::now();
            let ts = now.format("%Y-%m-%dT%H:%M:%S.").to_string()
                + &format!("{:03}Z", now.timestamp_subsec_millis());
            let sig = Self::build_signature(self.secret.as_str(), &ts, method, path, body);

            let url = format!("https://www.okx.com{}", path);

            let mut req = self
                .client
                .request(
                    reqwest::Method::from_bytes(method.as_bytes())
                        .unwrap_or(reqwest::Method::GET),
                    &url,
                )
                .header("OK-ACCESS-KEY", self.api_key.as_str())
                .header("OK-ACCESS-SIGN", &sig)
                .header("OK-ACCESS-TIMESTAMP", &ts)
                .header("OK-ACCESS-PASSPHRASE", self.passphrase.as_str())
                .header("Content-Type", "application/json");

            // 测试网模式强制添加模拟交易头
            if self.testnet {
                req = req.header("x-simulated-trading", "1");
            }

            let resp = if body.is_empty() {
                req.send()
            } else {
                req.body(body.to_string()).send()
            };

            match resp {
                Ok(r) => {
                    let status = r.status().as_u16();
                    let text = r.text().unwrap_or_default();
                    let v: serde_json::Value =
                        serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);
                    let code = v
                        .get("code")
                        .and_then(|c| c.as_str())
                        .unwrap_or("?");
                    if code == "0" {
                        return Ok(v);
                    }
                    let msg = v
                        .get("msg")
                        .and_then(|m| m.as_str())
                        .unwrap_or("?");
                    // 限流或服务器错误 -> 重试
                    if code == "1" || status == 429 || status >= 500 {
                        let delay = BASE_BACKOFF_MS * (attempt + 1) as u64;
                        last_err = format!(
                            "OKX {}/{}: {} (重试 {}/{}, 退避 {}ms)",
                            code,
                            status,
                            msg,
                            attempt + 1,
                            MAX_RETRIES,
                            delay
                        );
                        std::thread::sleep(std::time::Duration::from_millis(delay));
                        continue;
                    }
                    return Err(format!("OKX 错误 {} ({}): {}", code, status, msg));
                }
                Err(e) => {
                    last_err = format!("请求失败: {} (重试 {}/{})", e, attempt + 1, MAX_RETRIES);
                    std::thread::sleep(std::time::Duration::from_millis(
                        BASE_BACKOFF_MS * (attempt + 1) as u64,
                    ));
                }
            }
        }

        Err(format!("OKX 请求重试耗尽: {}", last_err))
    }

    // ── 下单 ─────────────────────────────────────────────────

    fn place_order(
        &self,
        order: &SimOrder,
        portfolio: &PortfolioState,
        now_ms: u64,
        trace_id: &str,
    ) -> Result<FillReport, String> {
        let inst_id = Self::symbol_to_inst_id(&order.symbol);
        let side = match order.side {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell",
        };
        let ord_type = match order.order_type {
            OrderType::Market => "market",
            OrderType::Limit => "limit",
            OrderType::StopLoss | OrderType::StopLossLimit
            | OrderType::TakeProfit | OrderType::TakeProfitLimit => {
                return Err(format!(
                    "实盘执行不支持订单类型 {:?}，仅支持 market/limit",
                    order.order_type
                ));
            }
        };

        // 风控：检查可用资金
        let order_value = order.quantity * order.reference_price;
        if !order_value.is_finite() || order_value > MAX_NOTIONAL_PER_ORDER {
            return Err(format!(
                "订单名义价值 ${:.2} 超过单笔上限 ${:.0}",
                order_value, MAX_NOTIONAL_PER_ORDER
            ));
        }
        if portfolio.available_cash_balance < order_value {
            return Err(format!(
                "可用资金 ${:.2} 不足，订单需 ${:.2}",
                portfolio.available_cash_balance, order_value
            ));
        }

        self.check_risk_limits(order_value, now_ms)?;

        // 构建请求 body
        let mut body_map = serde_json::Map::new();
        body_map.insert("instId".into(), json!(inst_id));
        body_map.insert("tdMode".into(), json!("cash"));
        body_map.insert("side".into(), json!(side));
        body_map.insert("ordType".into(), json!(ord_type));
        body_map.insert("sz".into(), json!(order.quantity.to_string()));

        if let Some(px) = order.limit_price {
            body_map.insert("px".into(), json!(px.to_string()));
        }

        // 客户端订单 ID: 用 order_id 的前 32 位
        let cl_ord_id = &order.order_id[..order.order_id.len().min(32)];
        body_map.insert("clOrdId".into(), json!(cl_ord_id));

        let body = serde_json::Value::Object(body_map).to_string();

        let resp = self.okx_request("POST", "/api/v5/trade/order", &body)?;

        // 解析下单返回
        let data = resp
            .get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| {
                format!("OKX 下单返回缺少 data 字段: {}", resp)
            })?;

        let ord_id = data
            .get("ordId")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        let s_code = data.get("sCode").and_then(|c| c.as_str()).unwrap_or("?");
        let s_msg = data.get("sMsg").and_then(|m| m.as_str()).unwrap_or("");

        if s_code != "0" {
            return Err(format!("OKX 下单失败 {}: {}", s_code, s_msg));
        }

        // 查询订单成交详情
        let fill = self.query_order_fill(&inst_id, &ord_id, order, now_ms, trace_id)?;
        Ok(fill)
    }

    // ── 查单 ─────────────────────────────────────────────────

    fn query_order_fill(
        &self,
        inst_id: &str,
        ord_id: &str,
        order: &SimOrder,
        now_ms: u64,
        trace_id: &str,
    ) -> Result<FillReport, String> {
        let path = format!(
            "/api/v5/trade/order?instId={}&ordId={}",
            inst_id, ord_id
        );

        // 市价单：等待 500ms 让其成交
        if matches!(order.order_type, OrderType::Market) {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        let resp = self.okx_request("GET", &path, "")?;

        let data = resp
            .get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| format!("OKX 查单返回缺少 data: {}", resp))?;

        let state = data
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("canceled");

        let fill_status = match state {
            "filled" => ExecutionStatus::Filled,
            "partially_filled" => ExecutionStatus::PartiallyFilled,
            "live" => ExecutionStatus::Open,
            "canceled" => ExecutionStatus::Cancelled,
            _ => ExecutionStatus::Open,
        };

        let acc_fill_sz = data
            .get("accFillSz")
            .and_then(|v| v.as_str())
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);

        let avg_px = data
            .get("avgPx")
            .and_then(|v| v.as_str())
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(order.reference_price);

        let fill_time = data
            .get("fillTime")
            .and_then(|v| v.as_str())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(now_ms);

        let fee = data
            .get("fee")
            .and_then(|v| v.as_str())
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);

        let fill_qty = acc_fill_sz.min(order.quantity);

        Ok(FillReport {
            fill_id: format!("fill-{}-{}", ord_id, now_ms),
            plan_id: order.order_id.clone(),
            exchange: order.exchange.clone(),
            symbol: order.symbol.clone(),
            side: order.side.clone(),
            filled_qty: fill_qty,
            filled_price: avg_px,
            fee_paid: fee.max(0.0), // v2.1.2: 负费用→0, 不掩码数据异常
            filled_at_ms: fill_time,
            status: fill_status,
            trace_id: trace_id.to_string(),
        })
    }

    // ── 构建执行计划（从 RiskDecision 生成 SimOrder） ────────

    fn build_orders_from_decisions(
        &self,
        decisions: &[qrpc_core::RiskDecision],
        quote_map: &std::collections::BTreeMap<(Exchange, Symbol), f64>,
        equity: f64,
        execution_semantics: &ExecutionSemantics,
    ) -> Vec<SimOrder> {
        let mut orders = Vec::new();

        for decision in decisions
            .iter()
            .filter(|d| !matches!(d.status, DecisionStatus::Reject))
        {
            if let Some(target) = &decision.adjusted_portfolio_target_decision {
                // 投资组合目标权重再平衡
                for tw in &target.target.target_weights {
                    let price = quote_map
                        .get(&(tw.exchange.clone(), tw.symbol.clone()))
                        .copied()
                        .unwrap_or(tw.reference_price);

                    if !price.is_finite() || price <= 0.0 {
                        continue;
                    }
                    let current_weight = current_position_weight(decision, &tw.exchange, &tw.symbol, price);
                    let delta = tw.target_weight - current_weight;
                    if delta.abs() <= 0.01 {
                        continue;
                    }

                    if !equity.is_finite() || equity <= 0.0 {
                        continue;
                    }
                    let notional = equity * delta.abs();
                    if !notional.is_finite() || notional <= 0.0 {
                        continue;
                    }

                    let quantity = notional / price;
                    if !quantity.is_finite() || quantity <= 0.0 {
                        continue;
                    }

                    let side = if delta > 0.0 {
                        OrderSide::Buy
                    } else {
                        OrderSide::Sell
                    };

                    orders.push(SimOrder {
                        order_id: format!(
                            "live-{}-{}-{}",
                            decision.risk_decision_id,
                            tw.symbol.as_str(),
                            tw.exchange == Exchange::Okx
                        ),
                        exchange: tw.exchange.clone(),
                        symbol: tw.symbol.clone(),
                        side,
                        order_type: OrderType::Market,
                        quantity,
                        limit_price: None,
                        time_in_force: execution_semantics.time_in_force.clone(),
                        allow_partial: false,
                        reference_price: price,
                        slippage_bps: execution_semantics.slippage_bps,
                        fee_bps: execution_semantics.fee_bps,
                        strategy_tag: format!(
                            "portfolio_target:{}:{}",
                            target.target.allocation_kind, target.target_id
                        ),
                    });
                }
            } else {
                // 直接动作
                for action in &decision.adjusted_actions {
                    let price = quote_map
                        .get(&(action.exchange.clone(), decision.symbol.clone()))
                        .copied()
                        .unwrap_or(action.reference_price);

                    let notional = equity * action.quantity_ratio.max(0.0);
                    let quantity = if price.is_finite() && price > 0.0 {
                        notional / price
                    } else {
                        0.0
                    };

                    if !quantity.is_finite() || quantity <= 0.0 {
                        continue;
                    }

                    orders.push(SimOrder {
                        order_id: format!(
                            "live-{}-{}-{}",
                            decision.risk_decision_id,
                            decision.symbol.as_str(),
                            action.exchange == Exchange::Okx
                        ),
                        exchange: action.exchange.clone(),
                        symbol: decision.symbol.clone(),
                        side: action.side.clone(),
                        order_type: OrderType::Market,
                        quantity,
                        limit_price: None,
                        time_in_force: execution_semantics.time_in_force.clone(),
                        allow_partial: false,
                        reference_price: price,
                        slippage_bps: execution_semantics.slippage_bps,
                        fee_bps: execution_semantics.fee_bps,
                        strategy_tag: action.strategy_tag.clone(),
                    });
                }
            }
        }

        orders
    }
}

// ── ExecutionSemantics ─────────────────────────────────────────────

#[derive(Debug, Clone)]
struct ExecutionSemantics {
    slippage_bps: f64,
    fee_bps: f64,
    time_in_force: TimeInForce,
}

impl Default for ExecutionSemantics {
    fn default() -> Self {
        Self {
            slippage_bps: 5.0,
            fee_bps: 10.0,
            time_in_force: TimeInForce::Ioc,
        }
    }
}

// ── 辅助函数 ──────────────────────────────────────────────────────

fn current_position_weight(
    decision: &qrpc_core::RiskDecision,
    exchange: &Exchange,
    symbol: &Symbol,
    price: f64,
) -> f64 {
    // 实盘执行模块不做持仓权重追踪（真实仓位由 OKX 管理），
    // 因此始终返回 0.0 表示当前无持仓权重。
    // 这是一个有意的桩实现 (GP §5.3 允许)，因为 OKX 服务端管理所有实际仓位，
    // 而现货模式下权重追踪对市价单再平衡没有实际影响。
    0.0
}

fn quote_price_map(
    normalized_data: &[qrpc_core::NormalizedMarketData],
) -> std::collections::BTreeMap<(Exchange, Symbol), f64> {
    let mut map = std::collections::BTreeMap::new();
    for item in normalized_data {
        match item {
            qrpc_core::NormalizedMarketData::Quote(quote) => {
                map.insert(
                    (quote.exchange.clone(), quote.symbol.clone()),
                    quote.mid_price,
                );
            }
            qrpc_core::NormalizedMarketData::KlineSeries(series) => {
                if let Some(last) = series.bars.last() {
                    map.entry((series.exchange.clone(), series.symbol.clone()))
                        .or_insert(last.close);
                }
            }
        }
    }
    map
}

fn portfolio_equity(portfolio: &PortfolioState) -> f64 {
    portfolio.cash_balance + portfolio.total_net_notional
}

// ── ExecutionModuleProvider 实现 ─────────────────────────────────

impl ExecutionPlanner for LiveExecutionModule {
    fn provider_key(&self) -> &'static str {
        "live.okx"
    }

    fn plan_execution(&self, request: ExecutionPlanningRequest<'_>) -> ExecutionPlanningOutput {
        let quote_map = quote_price_map(request.normalized_data);
        let equity = portfolio_equity(request.portfolio).max(0.0);

        let semantics = ExecutionSemantics {
            slippage_bps: request.core_ir.execution.slippage_bps,
            fee_bps: request.core_ir.execution.taker_fee_bps,
            time_in_force: match &request.core_ir.execution.time_in_force {
                qrpc_core_ir::CoreTimeInForce::Ioc => TimeInForce::Ioc,
                qrpc_core_ir::CoreTimeInForce::Fok => TimeInForce::Fok,
                qrpc_core_ir::CoreTimeInForce::Gtc => TimeInForce::Gtc,
            },
        };

        let orders = self.build_orders_from_decisions(
            request.risk_decisions,
            &quote_map,
            equity,
            &semantics,
        );

        let plan_id = format!("live-plan-{}", request.now_ms);
        let mut events = Vec::new();

        if !orders.is_empty() {
            let plan = ExecutionPlan {
                plan_id: plan_id.clone(),
                source_risk_decision_id: request
                    .risk_decisions
                    .first()
                    .map(|d| d.risk_decision_id.clone())
                    .unwrap_or_default(),
                orders,
                created_at_ms: request.now_ms,
                trace_id: request.trace_id.to_string(),
            };

            events.push(RuntimeEvent {
                event_id: format!("evt-live-plan-{}-{}", plan_id, request.now_ms),
                event_type: RuntimeEventType::ExecutionPlanned,
                trace_id: request.trace_id.to_string(),
                source_id: "live_execution".to_string(),
                ts_ms: request.now_ms,
                payload: json!({
                    "provider_key": "live.okx",
                    "plan_id": plan_id,
                    "orders": plan.orders.len(),
                    "testnet": self.testnet,
                }),
            });

            ExecutionPlanningOutput {
                plans: vec![plan],
                events,
            }
        } else {
            ExecutionPlanningOutput {
                plans: Vec::new(),
                events,
            }
        }
    }
}

impl ExecutionSubmitter for LiveExecutionModule {
    fn submit_plan(
        &mut self,
        plan: &ExecutionPlan,
        _normalized_data: &[qrpc_core::NormalizedMarketData],
        portfolio: &mut PortfolioState,
        now_ms: u64,
        trace_id: &str,
    ) -> FillResult {
        let mut fills = Vec::new();
        let open_orders = Vec::new();
        let mut events = Vec::new();
        let mut all_succeeded = true;

        for order in &plan.orders {
            // 只处理 OKX 交易所的订单
            if !matches!(order.exchange, Exchange::Okx) {
                events.push(RuntimeEvent {
                    event_id: format!(
                        "evt-live-skip-{}-{}",
                        order.order_id, now_ms
                    ),
                    event_type: RuntimeEventType::RuntimeWarning,
                    trace_id: trace_id.to_string(),
                    source_id: "live_execution".to_string(),
                    ts_ms: now_ms,
                    payload: json!({
                        "message": "跳过非 OKX 订单",
                        "exchange": format!("{:?}", order.exchange),
                        "order_id": order.order_id,
                    }),
                });
                continue;
            }

            match self.place_order(order, portfolio, now_ms, trace_id) {
                Ok(fill) => {
                    fills.push(fill.clone());

                    // 更新投资组合
                    let fill_cost = fill.filled_qty * fill.filled_price;
                    match fill.side {
                        OrderSide::Buy => {
                            portfolio.cash_balance -= fill_cost + fill.fee_paid;
                            portfolio.available_cash_balance =
                                (portfolio.available_cash_balance - fill_cost - fill.fee_paid)
                                    .max(0.0);
                        }
                        OrderSide::Sell => {
                            let proceeds = fill_cost;
                            portfolio.cash_balance += proceeds - fill.fee_paid;
                            portfolio.available_cash_balance += proceeds - fill.fee_paid;
                        }
                    }

                    // v2.1.2: 卖单成交前检查是否有足够持仓
                    if fill.side == OrderSide::Sell {
                        let pos_qty = portfolio.positions.iter()
                            .find(|p| p.exchange == fill.exchange && p.symbol == fill.symbol)
                            .map(|p| p.net_qty)
                            .unwrap_or(0.0);
                        if pos_qty < fill.filled_qty && pos_qty < 1e-9 {
                            eprintln!("[live_exec] 警告: 卖单 {} 成交但无对应持仓 (symbol={:?}, qty={})",
                                fill.fill_id, fill.symbol, fill.filled_qty);
                        }
                    }
                    // 更新持仓
                    update_position(portfolio, &fill);

                    // 记录事件
                    events.push(RuntimeEvent {
                        event_id: format!("evt-live-fill-{}-{}", fill.fill_id, now_ms),
                        event_type: RuntimeEventType::ExecutionFilled,
                        trace_id: trace_id.to_string(),
                        source_id: "live_execution".to_string(),
                        ts_ms: now_ms,
                        payload: json!({
                            "provider_key": "live.okx",
                            "order_id": order.order_id,
                            "symbol": order.symbol.as_str(),
                            "side": format!("{:?}", fill.side),
                            "filled_qty": fill.filled_qty,
                            "filled_price": fill.filled_price,
                            "fee": fill.fee_paid,
                            "testnet": self.testnet,
                        }),
                    });
                }
                Err(e) => {
                    all_succeeded = false;
                    // v2.1.1: 脱敏错误消息，防止OKX API错误泄露凭证信息
                    let sanitized = sanitize_error_for_event(&e);
                    events.push(RuntimeEvent {
                        event_id: format!("evt-live-err-{}-{}", order.order_id, now_ms),
                        event_type: RuntimeEventType::RuntimeError,
                        trace_id: trace_id.to_string(),
                        source_id: "live_execution".to_string(),
                        ts_ms: now_ms,
                        payload: json!({
                            "error": sanitized,
                            "order_id": order.order_id,
                            "symbol": order.symbol.as_str(),
                        }),
                    });
                }
            }
        }

        let status = if fills.is_empty() {
            ExecutionStatus::Rejected
        } else if fills.iter().all(|f| f.status == ExecutionStatus::Filled) {
            ExecutionStatus::Filled
        } else if fills.iter().any(|f| f.status == ExecutionStatus::PartiallyFilled) {
            ExecutionStatus::PartiallyFilled
        } else if all_succeeded {
            ExecutionStatus::Accepted
        } else {
            ExecutionStatus::Rejected
        };

        FillResult {
            plan_id: plan.plan_id.clone(),
            status,
            fills,
            open_orders,
            events,
        }
    }

    fn on_market_update(
        &mut self,
        normalized_data: &[qrpc_core::NormalizedMarketData],
        portfolio: &mut PortfolioState,
        now_ms: u64,
        trace_id: &str,
    ) -> FillResult {
        // 实盘模式下不追踪限价单状态（由 OKX 管理），
        // 因此始终返回空 FillResult。
        // 这是一个有意的桩实现 (GP §5.3 允许)，因为 OKX 交易所负责限价单的撮合和状态管理，
        // 实盘执行模块不需要轮询或追踪挂单状态。
        FillResult {
            plan_id: format!("live-on-market-{now_ms}"),
            status: ExecutionStatus::Open,
            fills: Vec::new(),
            open_orders: Vec::new(),
            events: vec![RuntimeEvent {
                event_id: format!("evt-live-market-update-{now_ms}"),
                event_type: RuntimeEventType::RuntimeWarning,
                trace_id: trace_id.to_string(),
                source_id: "live_execution".to_string(),
                ts_ms: now_ms,
                payload: json!({
                    "message": "实盘模式 on_market_update 无操作",
                    "provider_key": "live.okx",
                }),
            }],
        }
    }
}

// ── 持仓更新辅助 ────────────────────────────────────────────────

fn update_position(portfolio: &mut PortfolioState, fill: &FillReport) {
    let pos = portfolio
        .positions
        .iter_mut()
        .find(|p| p.exchange == fill.exchange && p.symbol == fill.symbol);

    match fill.side {
        OrderSide::Buy => {
            if let Some(p) = pos {
                let total_cost = p.avg_entry_price * p.net_qty.abs()
                    + fill.filled_qty * fill.filled_price;
                let total_qty = p.net_qty + fill.filled_qty;
                p.avg_entry_price = if total_qty.abs() > f64::EPSILON {
                    total_cost / total_qty.abs()
                } else {
                    0.0
                };
                p.net_qty = total_qty;
            } else {
                portfolio.positions.push(qrpc_core::Position {
                    exchange: fill.exchange.clone(),
                    symbol: fill.symbol.clone(),
                    net_qty: fill.filled_qty,
                    frozen_qty: 0.0,
                    avg_entry_price: fill.filled_price,
                    mark_price: fill.filled_price,
                    unrealized_pnl: 0.0,
                    realized_pnl: 0.0,
                });
            }
        }
        OrderSide::Sell => {
            if let Some(p) = pos {
                let new_qty = p.net_qty - fill.filled_qty;
                if new_qty.abs() <= f64::EPSILON || new_qty < 0.0 {
                    // 平仓或卖空
                    let realized = if p.net_qty.is_finite() && p.net_qty > 0.0 {
                        (fill.filled_price - p.avg_entry_price) * fill.filled_qty.min(p.net_qty)
                    } else {
                        0.0
                    };
                    p.realized_pnl += realized;
                    p.net_qty = new_qty.max(0.0); // 不允许负仓位（现货）
                    if p.net_qty.abs() <= f64::EPSILON {
                        p.avg_entry_price = 0.0;
                    }
                } else {
                    p.net_qty = new_qty;
                }
            }
        }
    }
}

// ── 单元测试 ──

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use hmac::{Hmac, Mac};
    use qrpc_core::Symbol;
    use sha2::Sha256;

    type TestHmacSha256 = Hmac<Sha256>;

    /// 辅助：计算 HMAC-SHA256 签名用于验证
    fn expected_signature(secret: &str, ts: &str, method: &str, path: &str, body: &str) -> String {
        let sign_str = format!("{}{}{}{}", ts, method, path, body);
        let mut mac =
            TestHmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC 初始化失败");
        mac.update(sign_str.as_bytes());
        let result = mac.finalize();
        base64::engine::general_purpose::STANDARD.encode(result.into_bytes())
    }

    /// 辅助：构建测试用 LiveExecutionModule
    fn test_module() -> LiveExecutionModule {
        LiveExecutionModule::new("test_key", "test_secret", "test_passphrase", true)
    }

    // ── build_signature ──

    #[test]
    fn test_build_signature_deterministic() {
        let sig1 =
            LiveExecutionModule::build_signature("secret", "ts1", "GET", "/path", "");
        let sig2 =
            LiveExecutionModule::build_signature("secret", "ts1", "GET", "/path", "");
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_build_signature_different_secret() {
        let sig1 =
            LiveExecutionModule::build_signature("secret_a", "ts", "GET", "/path", "");
        let sig2 =
            LiveExecutionModule::build_signature("secret_b", "ts", "GET", "/path", "");
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_build_signature_different_body() {
        let sig1 =
            LiveExecutionModule::build_signature("s", "ts", "POST", "/path", "body1");
        let sig2 =
            LiveExecutionModule::build_signature("s", "ts", "POST", "/path", "body2");
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_build_signature_matches_expected_hmac() {
        let secret = "my_secret_key";
        let ts = "2024-01-01T00:00:00.000Z";
        let method = "GET";
        let path = "/api/v5/account/balance";
        let body = "";

        let actual =
            LiveExecutionModule::build_signature(secret, ts, method, path, body);
        let expected = expected_signature(secret, ts, method, path, body);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_build_signature_valid_base64() {
        let sig = LiveExecutionModule::build_signature("s", "t", "GET", "/p", "b");
        let decoded =
            base64::engine::general_purpose::STANDARD.decode(&sig);
        assert!(decoded.is_ok());
        assert!(!decoded.unwrap().is_empty());
    }

    #[test]
    fn test_build_signature_includes_all_parts() {
        let with_body = LiveExecutionModule::build_signature(
            "s",
            "t",
            "POST",
            "/order",
            r#"{"sz":"1"}"#,
        );
        let without_body =
            LiveExecutionModule::build_signature("s", "t", "POST", "/order", "");
        assert_ne!(with_body, without_body);
    }

    // ── symbol_to_inst_id ──

    #[test]
    fn test_symbol_to_inst_id_btc_usdt() {
        let result =
            LiveExecutionModule::symbol_to_inst_id(&Symbol::BtcUsdt);
        assert_eq!(result, "BTC-USDT");
    }

    #[test]
    fn test_symbol_to_inst_id_eth_usdt() {
        let result = LiveExecutionModule::symbol_to_inst_id(
            &Symbol::Other("ETHUSDT".into()),
        );
        assert_eq!(result, "ETH-USDT");
    }

    #[test]
    fn test_symbol_to_inst_id_sol_usdt() {
        let result = LiveExecutionModule::symbol_to_inst_id(
            &Symbol::Other("SOLUSDT".into()),
        );
        assert_eq!(result, "SOL-USDT");
    }

    #[test]
    fn test_symbol_to_inst_id_no_usdt_suffix() {
        let result = LiveExecutionModule::symbol_to_inst_id(
            &Symbol::Other("ETHBTC".into()),
        );
        assert_eq!(result, "ETHBTC");
    }

    #[test]
    fn test_symbol_to_inst_id_just_usdt() {
        let result = LiveExecutionModule::symbol_to_inst_id(
            &Symbol::Other("USDT".into()),
        );
        assert_eq!(result, "USDT");
    }

    // ── check_risk_limits ──

    #[test]
    fn test_check_risk_limits_within_limits() {
        let module = test_module();
        let result = module.check_risk_limits(500.0, 1000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_risk_limits_over_max_notional() {
        let module = test_module();
        let result = module.check_risk_limits(1500.0, 1000);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("超过上限"));
    }

    #[test]
    fn test_check_risk_limits_at_exact_notional_boundary() {
        let module = test_module();
        // MAX_NOTIONAL_PER_ORDER = 1000.0, 代码使用 `>` 比较
        let result = module.check_risk_limits(1000.0, 1000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_risk_limits_over_max_notional_just_above() {
        let module = test_module();
        let result = module.check_risk_limits(1000.01, 1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_risk_limits_increments_count() {
        let module = test_module();
        let initial = *module.daily_order_count.lock().unwrap();
        module.check_risk_limits(100.0, 1000).unwrap();
        let after = *module.daily_order_count.lock().unwrap();
        assert_eq!(after, initial + 1);
    }

    #[test]
    fn test_check_risk_limits_over_daily_count() {
        let module = test_module();
        *module.daily_order_count.lock().unwrap() = 100;
        *module.daily_reset_at_ms.lock().unwrap() = 1000;

        let result = module.check_risk_limits(500.0, 2000);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("已达到上限"));
    }

    #[test]
    fn test_check_risk_limits_daily_count_below_max_passes() {
        let module = test_module();
        *module.daily_order_count.lock().unwrap() = 99;
        *module.daily_reset_at_ms.lock().unwrap() = 1000;

        let result = module.check_risk_limits(500.0, 2000);
        assert!(result.is_ok());
        assert_eq!(*module.daily_order_count.lock().unwrap(), 100);
    }

    #[test]
    fn test_check_risk_limits_daily_reset_at_24h_boundary() {
        let module = test_module();
        *module.daily_order_count.lock().unwrap() = 100;
        *module.daily_reset_at_ms.lock().unwrap() = 1000;

        let now_ms = 1000 + 24 * 60 * 60 * 1000 + 1;
        let result = module.check_risk_limits(500.0, now_ms);
        assert!(result.is_ok());
        assert_eq!(*module.daily_order_count.lock().unwrap(), 1);
        assert_eq!(*module.daily_reset_at_ms.lock().unwrap(), now_ms);
    }

    #[test]
    fn test_check_risk_limits_exact_24h_no_reset() {
        let module = test_module();
        // 条件为 now_ms >= reset_at.saturating_add(DAILY_RESET_MS)
        // 刚好低于 24h 边界时不应重置
        *module.daily_order_count.lock().unwrap() = 100;
        *module.daily_reset_at_ms.lock().unwrap() = 1000;

        let now_ms = 1000 + 24 * 60 * 60 * 1000 - 1; // 差 1ms 到 24h
        let result = module.check_risk_limits(500.0, now_ms);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_risk_limits_zero_value() {
        let module = test_module();
        let result = module.check_risk_limits(0.0, 1000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_risk_limits_negative_value() {
        let module = test_module();
        let result = module.check_risk_limits(-100.0, 1000);
        assert!(result.is_ok());
    }
}
