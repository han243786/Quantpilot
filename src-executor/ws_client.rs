/// v3.7.0: WebSocket 客户端 — 直连交易所实时数据
/// Binance: wss://stream.binance.com:9443/ws/{streams}
/// OKX: wss://ws.okx.com:8443/ws/v5/public
/// 自动重连: 指数退避 1s/2s/4s/.../max 30s

use crate::executor_state::KlineBar;
use std::time::Duration;
use tokio::sync::mpsc;

/// WS 数据事件
#[derive(Debug, Clone)]
pub enum WsEvent {
    /// ticker 更新: symbol, price, ts_ms
    Ticker { symbol: String, price: f64, ts_ms: u64 },
    /// K 线更新: symbol, bar
    Kline { symbol: String, bar: KlineBar },
    /// 连接状态变更
    Connected { exchange: String },
    Disconnected { exchange: String, reason: String },
}

/// WebSocket 连接池
pub struct WebSocketPool {
    /// WS 事件发送通道 (→ 执行引擎消费)
    pub event_tx: mpsc::UnboundedSender<WsEvent>,
    /// WS 事件接收通道
    pub event_rx: mpsc::UnboundedReceiver<WsEvent>,
    /// 活跃的交易所列表
    pub exchanges: Vec<String>,
    /// 重连退避 (秒)
    pub reconnect_backoff_secs: u32,
}

impl WebSocketPool {
    pub fn new() -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        Self {
            event_tx,
            event_rx,
            exchanges: Vec::new(),
            reconnect_backoff_secs: 1,
        }
    }

    /// 计算下一次重连等待时间 (指数退避, 最大 30s)
    pub fn next_backoff(&mut self) -> Duration {
        let delay = self.reconnect_backoff_secs.min(30);
        // v3.0.1 C-2: saturating_mul 防溢出
        self.reconnect_backoff_secs = self.reconnect_backoff_secs.saturating_mul(2).min(30);
        Duration::from_secs(delay as u64)
    }

    /// 重置退避计数器 (连接成功后)
    pub fn reset_backoff(&mut self) {
        self.reconnect_backoff_secs = 1;
    }
}

/// Binance WebSocket 订阅 URL 构建
pub fn binance_ws_url(symbols: &[&str], streams: &[&str]) -> String {
    // streams: ["ticker", "kline_1m", "trade"]
    let stream_names: Vec<String> = symbols.iter().flat_map(|sym| {
        let lower = sym.to_lowercase();
        streams.iter().map(move |s| format!("{}@{}", lower, s))
    }).collect();
    format!(
        "wss://stream.binance.com:9443/stream?streams={}",
        stream_names.join("/")
    )
}

/// OKX WebSocket 订阅
pub fn okx_ws_url() -> &'static str {
    "wss://ws.okx.com:8443/ws/v5/public"
}

/// v3.5.0: OKX testnet WebSocket URL (Paper testnet)
pub fn okx_testnet_ws_url() -> &'static str {
    "wss://wspap.okx.com:8443/ws/v5/public"
}

/// v3.5.0: 构建 OKX WebSocket 订阅请求 (tickers + kline)
pub fn okx_subscribe_message(symbols: &[&str], kline_interval: &str) -> String {
    let mut args = Vec::new();
    for sym in symbols {
        let inst_id = format_okx_inst_id(sym);
        args.push(serde_json::json!({"channel": "tickers", "instId": inst_id}));
        args.push(serde_json::json!({"channel": format!("candle{}", kline_interval), "instId": inst_id}));
    }
    serde_json::json!({"op": "subscribe", "args": args}).to_string()
}

fn format_okx_inst_id(symbol: &str) -> String {
    // BTCUSDT → BTC-USDT
    if symbol.ends_with("USDT") && symbol.len() > 4 {
        format!("{}-USDT", &symbol[..symbol.len()-4])
    } else {
        symbol.to_string()
    }
}

/// v3.5.0: 解析 OKX ticker 消息
pub fn parse_okx_ticker(data: &serde_json::Value) -> Option<WsEvent> {
    let result = (|| {
        let arg = data.get("arg")?;
        if arg.get("channel")?.as_str()? != "tickers" { return None; }
        let inst_id = arg.get("instId")?.as_str()?;
        let symbol = inst_id.replace("-", "");
        let ticker_data = data.get("data")?.as_array()?.first()?;
        let price = ticker_data.get("last")?.as_str()?.parse::<f64>().ok()?;
        if !price.is_finite() { return None; }
        let ts_ms = ticker_data.get("ts").and_then(|v| v.as_str()).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        Some(WsEvent::Ticker { symbol, price, ts_ms })
    })();
    if result.is_none() {
        eprintln!("[ws] 解析失败: {}", data);
    }
    result
}

/// v3.5.0: 解析 OKX K 线消息
pub fn parse_okx_kline(data: &serde_json::Value) -> Option<WsEvent> {
    let result = (|| {
        let arg = data.get("arg")?;
        let channel = arg.get("channel")?.as_str()?;
        if !channel.starts_with("candle") { return None; }
        let inst_id = arg.get("instId")?.as_str()?;
        let symbol = inst_id.replace("-", "");
        let candle_data = data.get("data")?.as_array()?.first()?.as_array()?;
        if candle_data.len() < 6 { return None; }
        let open = candle_data[1].as_str()?.parse::<f64>().ok()?;
        let high = candle_data[2].as_str()?.parse::<f64>().ok()?;
        let low = candle_data[3].as_str()?.parse::<f64>().ok()?;
        let close = candle_data[4].as_str()?.parse::<f64>().ok()?;
        let volume = candle_data[5].as_str()?.parse::<f64>().ok()?;
        if !open.is_finite() || !high.is_finite() || !low.is_finite() || !close.is_finite() || !volume.is_finite() {
            return None;
        }
        let bar = KlineBar {
            open_time_ms: candle_data[0].as_str()?.parse::<u64>().ok()?,
            open, high, low, close, volume,
            close_time_ms: candle_data[0].as_str()?.parse::<u64>().ok()?,
        };
        Some(WsEvent::Kline { symbol, bar })
    })();
    if result.is_none() {
        eprintln!("[ws] 解析失败: {}", data);
    }
    result
}

/// 解析 Binance ticker 消息
pub fn parse_binance_ticker(data: &serde_json::Value) -> Option<WsEvent> {
    let result = (|| {
        let symbol = data.get("s")?.as_str()?.to_string();
        let price = data.get("c")?.as_str()?.parse::<f64>().ok()?;
        let ts_ms = data.get("E").and_then(|v| v.as_u64()).unwrap_or(0);
        Some(WsEvent::Ticker { symbol, price, ts_ms })
    })();
    if result.is_none() {
        eprintln!("[ws] 解析失败: {}", data);
    }
    result
}

/// 解析 Binance K 线消息
pub fn parse_binance_kline(data: &serde_json::Value) -> Option<WsEvent> {
    let result = (|| {
        let kline = data.get("k")?;
        let symbol = data.get("s")?.as_str()?.to_string();
        let bar = KlineBar {
            open_time_ms: kline.get("t").and_then(|v| v.as_u64())?,
            close_time_ms: kline.get("T").and_then(|v| v.as_u64())?,
            open: kline.get("o")?.as_str()?.parse().ok()?,
            high: kline.get("h")?.as_str()?.parse().ok()?,
            low: kline.get("l")?.as_str()?.parse().ok()?,
            close: kline.get("c")?.as_str()?.parse().ok()?,
            volume: kline.get("v")?.as_str()?.parse().ok()?,
        };
        Some(WsEvent::Kline { symbol, bar })
    })();
    if result.is_none() {
        eprintln!("[ws] 解析失败: {}", data);
    }
    result
}
