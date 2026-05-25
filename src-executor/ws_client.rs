/// v3.7.0/v4.8.0: WebSocket 客户端 — OKX 公共实时行情
/// OKX: wss://ws.okx.com:8443/ws/v5/public
/// OKX candles: wss://ws.okx.com:8443/ws/v5/business
/// 自动重连: 指数退避 1s/2s/4s/.../max 30s
use crate::executor_state::KlineBar;
use anyhow::{bail, Context, Result};
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
#[cfg(test)]
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{client_async_tls_with_config, MaybeTlsStream, WebSocketStream};

const OKX_PUBLIC_WS_URL: &str = "wss://ws.okx.com:8443/ws/v5/public";
const OKX_BUSINESS_WS_URL: &str = "wss://ws.okx.com:8443/ws/v5/business";
#[cfg(test)]
const OKX_DEMO_WS_URL: &str = "wss://wspap.okx.com:8443/ws/v5/public";
const OKX_WS_PROXY_ENV: &str = "QUANTPILOT_OKX_WS_PROXY";
const OKX_WS_SYMBOLS_ENV: &str = "QUANTPILOT_OKX_PUBLIC_SYMBOLS";
const OKX_WS_CONNECT_TIMEOUT_SECS: u64 = 12;

/// WS 数据事件
#[derive(Debug, Clone)]
pub enum WsEvent {
    /// ticker 更新: symbol, price, ts_ms
    Ticker {
        symbol: String,
        price: f64,
        ts_ms: u64,
    },
    /// K 线更新: symbol, bar
    Kline { symbol: String, bar: KlineBar },
    /// 连接状态变更
    Connected { exchange: String },
    #[cfg(test)]
    Disconnected { exchange: String, reason: String },
}

/// WebSocket 连接池
#[cfg(test)]
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

#[cfg(test)]
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
#[cfg(test)]
pub fn binance_ws_url(symbols: &[&str], streams: &[&str]) -> String {
    // streams: ["ticker", "kline_1m", "trade"]
    let stream_names: Vec<String> = symbols
        .iter()
        .flat_map(|sym| {
            let lower = sym.to_lowercase();
            streams.iter().map(move |s| format!("{}@{}", lower, s))
        })
        .collect();
    format!(
        "wss://stream.binance.com:9443/stream?streams={}",
        stream_names.join("/")
    )
}

/// OKX WebSocket 订阅
#[cfg(test)]
pub fn okx_ws_url() -> &'static str {
    OKX_PUBLIC_WS_URL
}

#[cfg(test)]
pub fn okx_business_ws_url() -> &'static str {
    OKX_BUSINESS_WS_URL
}

/// v3.5.0: OKX demo WebSocket URL.
/// 仅用于 provider 连通性观察; 策略测试行情以真实公共行情或归档回放数据为准。
#[cfg(test)]
pub fn okx_testnet_ws_url() -> &'static str {
    OKX_DEMO_WS_URL
}

/// v3.5.0/v4.8.0: 构建 OKX ticker 订阅请求。
pub fn okx_tickers_subscribe_message(symbols: &[&str]) -> String {
    let mut args = Vec::new();
    for sym in symbols {
        let inst_id = format_okx_inst_id(sym);
        args.push(serde_json::json!({"channel": "tickers", "instId": inst_id}));
    }
    serde_json::json!({"op": "subscribe", "args": args}).to_string()
}

/// v3.5.0/v4.8.0: 构建 OKX candle 订阅请求。
pub fn okx_candles_subscribe_message(symbols: &[&str], kline_interval: &str) -> String {
    let mut args = Vec::new();
    for sym in symbols {
        let inst_id = format_okx_inst_id(sym);
        args.push(
            serde_json::json!({"channel": format!("candle{}", kline_interval), "instId": inst_id}),
        );
    }
    serde_json::json!({"op": "subscribe", "args": args}).to_string()
}

fn format_okx_inst_id(symbol: &str) -> String {
    let symbol = symbol.trim().to_ascii_uppercase();
    if symbol.contains('-') {
        symbol
    } else if symbol.ends_with("USDT") && symbol.len() > 4 {
        format!("{}-USDT", &symbol[..symbol.len() - 4])
    } else {
        symbol
    }
}

/// v3.5.0: 解析 OKX ticker 消息
pub fn parse_okx_ticker(data: &serde_json::Value) -> Option<WsEvent> {
    let result = (|| {
        let arg = data.get("arg")?;
        if arg.get("channel")?.as_str()? != "tickers" {
            return None;
        }
        let inst_id = arg.get("instId")?.as_str()?;
        let symbol = inst_id.replace("-", "");
        let ticker_data = data.get("data")?.as_array()?.first()?;
        let price = ticker_data.get("last")?.as_str()?.parse::<f64>().ok()?;
        if !price.is_finite() {
            return None;
        }
        let ts_ms = ticker_data
            .get("ts")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        Some(WsEvent::Ticker {
            symbol,
            price,
            ts_ms,
        })
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
        if !channel.starts_with("candle") {
            return None;
        }
        let inst_id = arg.get("instId")?.as_str()?;
        let symbol = inst_id.replace("-", "");
        let candle_data = data.get("data")?.as_array()?.first()?.as_array()?;
        if candle_data.len() < 6 {
            return None;
        }
        let open = candle_data[1].as_str()?.parse::<f64>().ok()?;
        let high = candle_data[2].as_str()?.parse::<f64>().ok()?;
        let low = candle_data[3].as_str()?.parse::<f64>().ok()?;
        let close = candle_data[4].as_str()?.parse::<f64>().ok()?;
        let volume = candle_data[5].as_str()?.parse::<f64>().ok()?;
        if !open.is_finite()
            || !high.is_finite()
            || !low.is_finite()
            || !close.is_finite()
            || !volume.is_finite()
        {
            return None;
        }
        let open_time_ms = candle_data[0].as_str()?.parse::<u64>().ok()?;
        let close_time_ms = open_time_ms
            + okx_candle_interval_ms(channel)
                .unwrap_or(60_000)
                .saturating_sub(1);
        let bar = KlineBar {
            open_time_ms,
            open,
            high,
            low,
            close,
            volume,
            close_time_ms,
        };
        Some(WsEvent::Kline { symbol, bar })
    })();
    if result.is_none() {
        eprintln!("[ws] 解析失败: {}", data);
    }
    result
}

fn okx_candle_interval_ms(channel: &str) -> Option<u64> {
    match channel.strip_prefix("candle")? {
        "1m" => Some(60_000),
        "3m" => Some(3 * 60_000),
        "5m" => Some(5 * 60_000),
        "15m" => Some(15 * 60_000),
        "30m" => Some(30 * 60_000),
        "1H" | "1h" => Some(60 * 60_000),
        "2H" | "2h" => Some(2 * 60 * 60_000),
        "4H" | "4h" => Some(4 * 60 * 60_000),
        "1D" | "1d" => Some(24 * 60 * 60_000),
        _ => None,
    }
}

pub fn okx_public_feed_symbols_from_env() -> Vec<String> {
    std::env::var(OKX_WS_SYMBOLS_ENV)
        .ok()
        .map(|raw| {
            raw.split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| vec!["BTCUSDT".to_string()])
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OkxWsEndpoint {
    host: String,
    port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Socks5ProxyEndpoint {
    host: String,
    port: u16,
}

fn parse_okx_ws_endpoint(url: &str) -> Result<OkxWsEndpoint> {
    let rest = url
        .strip_prefix("wss://")
        .ok_or_else(|| anyhow::anyhow!("OKX public WS URL 必须以 wss:// 开头"))?;
    let authority = rest.split('/').next().unwrap_or_default();
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (
            host.to_string(),
            port.parse::<u16>()
                .with_context(|| format!("OKX public WS port 无效: {port}"))?,
        ),
        None => (authority.to_string(), 443),
    };
    if host.trim().is_empty() {
        bail!("OKX public WS host 不能为空");
    }
    Ok(OkxWsEndpoint { host, port })
}

fn okx_ws_proxy_url() -> Option<String> {
    [
        OKX_WS_PROXY_ENV,
        "ALL_PROXY",
        "all_proxy",
        "HTTPS_PROXY",
        "https_proxy",
        "HTTP_PROXY",
        "http_proxy",
    ]
    .into_iter()
    .find_map(|name| {
        std::env::var(name)
            .ok()
            .map(|value| value.trim().trim_end_matches('/').to_string())
            .filter(|value| !value.is_empty())
    })
}

fn parse_socks5_proxy_endpoint(raw: &str) -> Result<Socks5ProxyEndpoint> {
    let rest = raw
        .strip_prefix("socks5h://")
        .or_else(|| raw.strip_prefix("socks5://"))
        .ok_or_else(|| anyhow::anyhow!("OKX WS proxy 仅支持 socks5h:// 或 socks5://"))?;
    if rest.contains('@') {
        bail!("OKX WS proxy 当前不接受带用户名/密码的代理 URL");
    }
    let (host, port) = rest
        .rsplit_once(':')
        .ok_or_else(|| anyhow::anyhow!("OKX WS proxy 必须包含 host:port"))?;
    let port = port
        .parse::<u16>()
        .with_context(|| format!("OKX WS proxy port 无效: {port}"))?;
    if host.trim().is_empty() {
        bail!("OKX WS proxy host 不能为空");
    }
    Ok(Socks5ProxyEndpoint {
        host: host.to_string(),
        port,
    })
}

async fn connect_socks5_to_target(
    proxy: &Socks5ProxyEndpoint,
    target: &OkxWsEndpoint,
) -> Result<TcpStream> {
    let mut stream = tokio::time::timeout(
        Duration::from_secs(OKX_WS_CONNECT_TIMEOUT_SECS),
        TcpStream::connect((proxy.host.as_str(), proxy.port)),
    )
    .await
    .context("OKX WS SOCKS5 proxy connect timed out")??;
    stream.write_all(&[0x05, 0x01, 0x00]).await?;
    let mut auth = [0u8; 2];
    stream.read_exact(&mut auth).await?;
    if auth != [0x05, 0x00] {
        bail!(
            "OKX WS SOCKS5 proxy refused no-auth handshake: {:02x?}",
            auth
        );
    }

    let host = target.host.as_bytes();
    if host.len() > u8::MAX as usize {
        bail!("OKX WS target host 过长");
    }
    let mut req = Vec::with_capacity(7 + host.len());
    req.extend_from_slice(&[0x05, 0x01, 0x00, 0x03, host.len() as u8]);
    req.extend_from_slice(host);
    req.extend_from_slice(&target.port.to_be_bytes());
    stream.write_all(&req).await?;

    let mut head = [0u8; 4];
    stream.read_exact(&mut head).await?;
    if head[0] != 0x05 || head[1] != 0x00 {
        bail!("OKX WS SOCKS5 proxy connect failed: {:02x?}", head);
    }
    match head[3] {
        0x01 => {
            let mut rest = [0u8; 6];
            stream.read_exact(&mut rest).await?;
        }
        0x03 => {
            let mut len = [0u8; 1];
            stream.read_exact(&mut len).await?;
            let mut rest = vec![0u8; len[0] as usize + 2];
            stream.read_exact(&mut rest).await?;
        }
        0x04 => {
            let mut rest = [0u8; 18];
            stream.read_exact(&mut rest).await?;
        }
        other => bail!("OKX WS SOCKS5 proxy returned unknown address type: {other}"),
    }
    Ok(stream)
}

async fn connect_okx_ws(
    ws_url: &'static str,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    let endpoint = parse_okx_ws_endpoint(ws_url)?;
    let tcp_stream = if let Some(proxy_url) = okx_ws_proxy_url() {
        let proxy = parse_socks5_proxy_endpoint(&proxy_url)
            .with_context(|| format!("OKX WS proxy 配置无效: {proxy_url}"))?;
        connect_socks5_to_target(&proxy, &endpoint).await?
    } else {
        tokio::time::timeout(
            Duration::from_secs(OKX_WS_CONNECT_TIMEOUT_SECS),
            TcpStream::connect((endpoint.host.as_str(), endpoint.port)),
        )
        .await
        .with_context(|| format!("OKX WS connect timed out: {ws_url}"))??
    };
    let (ws_stream, _) = client_async_tls_with_config(ws_url, tcp_stream, None, None).await?;
    Ok(ws_stream)
}

fn dispatch_okx_market_value(
    source: &str,
    value: &serde_json::Value,
    tx: &tokio::sync::mpsc::UnboundedSender<WsEvent>,
) {
    if value.get("event").is_some() {
        if value.get("event").and_then(|v| v.as_str()) == Some("error") {
            eprintln!("[ws] {source} event error: {value}");
        }
        return;
    }

    let channel = value
        .get("arg")
        .and_then(|arg| arg.get("channel"))
        .and_then(|channel| channel.as_str());
    let event = match channel {
        Some("tickers") => parse_okx_ticker(value),
        Some(channel) if channel.starts_with("candle") => parse_okx_kline(value),
        _ => None,
    };
    if let Some(event) = event {
        let _ = tx.send(event);
    }
}

async fn run_okx_market_channel(
    tx: tokio::sync::mpsc::UnboundedSender<WsEvent>,
    symbols: Vec<String>,
    ws_url: &'static str,
    source: &'static str,
    subscribe_message: String,
    exchange: &'static str,
) {
    let mut backoff_secs = 1u64;

    loop {
        match connect_okx_ws(ws_url).await {
            Ok(mut socket) => {
                eprintln!("[ws] {source} connected; symbols={}", symbols.join(","));
                let _ = tx.send(WsEvent::Connected {
                    exchange: exchange.to_string(),
                });
                if let Err(error) = socket.send(Message::Text(subscribe_message.clone())).await {
                    eprintln!("[ws] {source} subscribe failed: {error}");
                } else {
                    backoff_secs = 1;
                }

                while let Some(message) = socket.next().await {
                    match message {
                        Ok(Message::Text(text)) if text == "ping" => {
                            let _ = socket.send(Message::Text("pong".to_string())).await;
                        }
                        Ok(Message::Ping(payload)) => {
                            let _ = socket.send(Message::Pong(payload)).await;
                        }
                        Ok(Message::Text(text)) => {
                            match serde_json::from_str::<serde_json::Value>(&text) {
                                Ok(value) => dispatch_okx_market_value(source, &value, &tx),
                                Err(error) => {
                                    eprintln!("[ws] {source} message parse failed: {error}")
                                }
                            }
                        }
                        Ok(Message::Binary(payload)) => {
                            match serde_json::from_slice::<serde_json::Value>(&payload) {
                                Ok(value) => dispatch_okx_market_value(source, &value, &tx),
                                Err(error) => {
                                    eprintln!("[ws] {source} binary parse failed: {error}")
                                }
                            }
                        }
                        Ok(Message::Close(frame)) => {
                            eprintln!("[ws] {source} closed: {:?}", frame);
                            break;
                        }
                        Ok(_) => {}
                        Err(error) => {
                            eprintln!("[ws] {source} stream error: {error}");
                            break;
                        }
                    }
                }
            }
            Err(error) => eprintln!("[ws] {source} connect failed: {error:#}"),
        }

        tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
        backoff_secs = backoff_secs.saturating_mul(2).min(30);
    }
}

pub async fn run_okx_public_market_feed(
    tx: tokio::sync::mpsc::UnboundedSender<WsEvent>,
    symbols: Vec<String>,
) {
    let symbols = if symbols.is_empty() {
        vec!["BTCUSDT".to_string()]
    } else {
        symbols
    };
    let symbol_refs = symbols.iter().map(String::as_str).collect::<Vec<_>>();
    let ticker_subscribe_message = okx_tickers_subscribe_message(&symbol_refs);
    let candle_subscribe_message = okx_candles_subscribe_message(&symbol_refs, "1m");
    tokio::join!(
        run_okx_market_channel(
            tx.clone(),
            symbols.clone(),
            OKX_PUBLIC_WS_URL,
            "OKX public tickers",
            ticker_subscribe_message,
            "okx_public"
        ),
        run_okx_market_channel(
            tx,
            symbols,
            OKX_BUSINESS_WS_URL,
            "OKX business candles",
            candle_subscribe_message,
            "okx_business"
        )
    );
}

/// 解析 Binance ticker 消息
#[cfg(test)]
pub fn parse_binance_ticker(data: &serde_json::Value) -> Option<WsEvent> {
    let result = (|| {
        let symbol = data.get("s")?.as_str()?.to_string();
        let price = data.get("c")?.as_str()?.parse::<f64>().ok()?;
        let ts_ms = data.get("E").and_then(|v| v.as_u64()).unwrap_or(0);
        Some(WsEvent::Ticker {
            symbol,
            price,
            ts_ms,
        })
    })();
    if result.is_none() {
        eprintln!("[ws] 解析失败: {}", data);
    }
    result
}

/// 解析 Binance K 线消息
#[cfg(test)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn websocket_pool_backoff_caps_and_resets() {
        let mut pool = WebSocketPool::new();
        pool.exchanges.push("okx".to_string());
        assert_eq!(pool.next_backoff(), Duration::from_secs(1));
        assert_eq!(pool.next_backoff(), Duration::from_secs(2));
        pool.reconnect_backoff_secs = 64;
        assert_eq!(pool.next_backoff(), Duration::from_secs(30));
        pool.reset_backoff();
        assert_eq!(pool.reconnect_backoff_secs, 1);
        assert!(pool
            .event_tx
            .send(WsEvent::Connected {
                exchange: "okx".to_string()
            })
            .is_ok());
        assert!(matches!(
            pool.event_rx.try_recv(),
            Ok(WsEvent::Connected { .. })
        ));
        assert_eq!(pool.exchanges, vec!["okx".to_string()]);
    }

    #[test]
    fn websocket_urls_and_okx_subscription_are_stable() {
        assert_eq!(
            binance_ws_url(&["BTCUSDT"], &["ticker", "kline_1m"]),
            "wss://stream.binance.com:9443/stream?streams=btcusdt@ticker/btcusdt@kline_1m"
        );
        assert_eq!(okx_ws_url(), "wss://ws.okx.com:8443/ws/v5/public");
        assert_eq!(
            okx_business_ws_url(),
            "wss://ws.okx.com:8443/ws/v5/business"
        );
        assert_eq!(
            okx_testnet_ws_url(),
            "wss://wspap.okx.com:8443/ws/v5/public"
        );
        let ticker_message: serde_json::Value =
            serde_json::from_str(&okx_tickers_subscribe_message(&["BTCUSDT"])).unwrap();
        assert_eq!(ticker_message["op"], "subscribe");
        assert_eq!(ticker_message["args"][0]["channel"], "tickers");
        assert_eq!(ticker_message["args"][0]["instId"], "BTC-USDT");
        let candle_message: serde_json::Value =
            serde_json::from_str(&okx_candles_subscribe_message(&["BTCUSDT"], "1m")).unwrap();
        assert_eq!(candle_message["args"][0]["channel"], "candle1m");
        assert_eq!(candle_message["args"][0]["instId"], "BTC-USDT");
        let hyphenated: serde_json::Value =
            serde_json::from_str(&okx_tickers_subscribe_message(&["BTC-USDT"])).unwrap();
        assert_eq!(hyphenated["args"][0]["instId"], "BTC-USDT");
        let proxy = parse_socks5_proxy_endpoint("socks5h://127.0.0.1:7897").unwrap();
        assert_eq!(proxy.host, "127.0.0.1");
        assert_eq!(proxy.port, 7897);
    }

    #[test]
    fn parses_exchange_market_data_messages() {
        let okx_ticker = serde_json::json!({
            "arg": {"channel": "tickers", "instId": "BTC-USDT"},
            "data": [{"last": "70000.5", "ts": "1710000000000"}]
        });
        assert!(matches!(
            parse_okx_ticker(&okx_ticker),
            Some(WsEvent::Ticker { .. })
        ));

        let okx_kline = serde_json::json!({
            "arg": {"channel": "candle1m", "instId": "BTC-USDT"},
            "data": [["1710000000000", "1", "2", "0.5", "1.5", "12"]]
        });
        match parse_okx_kline(&okx_kline) {
            Some(WsEvent::Kline { bar, .. }) => {
                assert_eq!(bar.open_time_ms, 1_710_000_000_000);
                assert_eq!(bar.close_time_ms, 1_710_000_059_999);
            }
            other => panic!("unexpected OKX kline parse result: {:?}", other),
        }

        let binance_ticker =
            serde_json::json!({"s": "BTCUSDT", "c": "70000.5", "E": 1710000000000u64});
        assert!(matches!(
            parse_binance_ticker(&binance_ticker),
            Some(WsEvent::Ticker { .. })
        ));

        let binance_kline = serde_json::json!({
            "s": "BTCUSDT",
            "k": {"t": 1u64, "T": 2u64, "o": "1", "h": "2", "l": "0.5", "c": "1.5", "v": "12"}
        });
        assert!(matches!(
            parse_binance_kline(&binance_kline),
            Some(WsEvent::Kline { .. })
        ));

        let disconnected = WsEvent::Disconnected {
            exchange: "okx".to_string(),
            reason: "test".to_string(),
        };
        assert!(matches!(disconnected, WsEvent::Disconnected { .. }));
    }
}
