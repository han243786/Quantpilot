/// v3.7.0: K线环形缓冲区
/// 每标的最近 1000 条 K 线, OHLC 聚合, 一字线生成
/// 无交易时生成一字线 (O=H=L=C=last_close)

use crate::executor_state::{KlineBar, RingBuffer};
use std::collections::HashMap;

/// K线缓冲池: symbol → RingBuffer
pub struct KlinePool {
    pub buffers: HashMap<String, RingBuffer>,
    pub capacity: usize,
    /// v3.2.2: symbol数量上限
    pub max_symbols: usize,
}

impl KlinePool {
    const DEFAULT_MAX_SYMBOLS: usize = 100;

    pub fn new(capacity: usize) -> Self {
        Self { buffers: HashMap::new(), capacity, max_symbols: Self::DEFAULT_MAX_SYMBOLS }
    }

    /// 获取或创建标的缓冲区 — v3.2.2: 超限时LRU淘汰
    pub fn get_or_create(&mut self, symbol: &str) -> &mut RingBuffer {
        if !self.buffers.contains_key(symbol) && self.buffers.len() >= self.max_symbols {
            // 淘汰最久未访问的symbol (简单策略: 移除第一个)
            if let Some(oldest) = self.buffers.keys().next().cloned() {
                self.buffers.remove(&oldest);
            }
        }
        self.buffers.entry(symbol.to_string())
            .or_insert_with(|| RingBuffer::new(self.capacity))
    }

    /// 插入新K线 (同分钟更新 OHLC)
    pub fn update_kline(&mut self, symbol: &str, bar: KlineBar) {
        // v3.6.x: 拒绝含 NaN/Inf 的K线 (全部OHLCV字段)
        if !bar.open.is_finite() || !bar.high.is_finite() || !bar.low.is_finite() || !bar.close.is_finite() || !bar.volume.is_finite() { return; }
        let buffer = self.get_or_create(symbol);
        if let Some(last) = buffer.bars.back_mut() {
            if last.open_time_ms == bar.open_time_ms {
                last.high = last.high.max(bar.high);
                last.low = last.low.min(bar.low);
                last.close = bar.close;
                last.volume += bar.volume;
                last.close_time_ms = bar.close_time_ms;
                return;
            }
        }
        buffer.push(bar);
    }

    /// Ticker 更新: 无交易时生成一字线
    pub fn update_from_ticker(&mut self, symbol: &str, price: f64, ts_ms: u64) {
        // v3.0.1 C-3: 拒绝 NaN/Inf 价格
        if !price.is_finite() { return; }
        let buffer = self.get_or_create(symbol);
        let minute_start = ts_ms / 60_000 * 60_000;
        let minute_end = minute_start + 59_999;

        if let Some(last) = buffer.bars.back_mut() {
            if last.open_time_ms == minute_start {
                // 同一分钟: 更新 OHLC
                last.high = last.high.max(price);
                last.low = last.low.min(price);
                last.close = price;
                last.close_time_ms = ts_ms;
                return;
            }
            // 跨分钟: 如有缺口, 用最后价格填充一字线
            let last_close = last.close;
            // v3.7.0: 仅填一根一字线 (长时间缺口由前端K线引擎补齐)
            let gap_start = last.close_time_ms / 60_000 * 60_000 + 60_000;
            if gap_start < minute_start {
                buffer.push(KlineBar {
                    open_time_ms: gap_start,
                    close_time_ms: gap_start + 59_999,
                    open: last_close,
                    high: last_close,
                    low: last_close,
                    close: last_close,
                    volume: 0.0,
                });
            }
        }

        // 新建当前分钟的K线
        let bar = KlineBar {
            open_time_ms: minute_start,
            close_time_ms: minute_end,
            open: price,
            high: price,
            low: price,
            close: price,
            volume: 0.0,
        };
        buffer.push(bar);
    }

    /// 获取最新的 N 条 K 线用于渲染
    pub fn recent_bars(&self, symbol: &str, count: usize) -> Vec<&KlineBar> {
        // v3.0.1 J-1: 单次正向迭代, 无需双次反转+分配
        self.buffers
            .get(symbol)
            .map(|buf| buf.bars.iter().skip(buf.bars.len().saturating_sub(count)).collect())
            .unwrap_or_default()
    }

    /// 获取最新价格
    pub fn latest_price(&self, symbol: &str) -> Option<f64> {
        self.buffers.get(symbol).and_then(|buf| buf.latest()).map(|b| b.close)
    }
}
