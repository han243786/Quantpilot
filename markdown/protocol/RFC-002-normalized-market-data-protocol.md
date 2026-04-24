# RFC-002 Normalized Market Data Protocol

规范化市场数据协议定义系统内部统一的市场数据表达。QRPC 要求所有计算都只读取规范化数据，而不能直接消费原始交易所结构。

## 协议目标

- 固定统一时间语义
- 固定统一标的表达
- 固定统一精度与缺失值语义
- 固定统一数据源语义标识
- 为 Intent、Agent、Risk、Execution、Backtest 提供同一套上游数据对象

## 稳定对象

QRPC 只允许两个稳定的规范化对象：

- `NormalizedFactPrice`
- `NormalizedKlineSeries`

这两个对象内部都必须保留 `source_type`，以表明其语义来源。

## NormalizedFactPrice

```rust
struct NormalizedFactPrice {
    data_id: String,
    instrument: String,
    market_scope: MarketScope,
    source_type: SourceType,
    event_time_ms: u64,
    receive_time_ms: u64,
    last_price: Option<f64>,
    bid_price: Option<f64>,
    ask_price: Option<f64>,
    mid_price: Option<f64>,
    mark_price: Option<f64>,
    index_price: Option<f64>,
    volume_24h: Option<f64>,
    is_valid: bool,
    quality_score: u8,
}
```

`NormalizedFactPrice` 统一表达事实市场快照。它不是只保留 `last_price`，而是允许承载 `last / bid / ask / mid / mark / index` 等字段，但在协议层面仍然只属于 `FactPrice` 一级类型。

## NormalizedKline

```rust
struct NormalizedKline {
    instrument: String,
    market_scope: MarketScope,
    source_type: SourceType,
    timeframe: Timeframe,
    open_time_ms: u64,
    close_time_ms: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    quote_volume: Option<f64>,
    trade_count: Option<u64>,
    is_closed: bool,
}
```

## NormalizedKlineSeries

```rust
struct NormalizedKlineSeries {
    series_id: String,
    instrument: String,
    market_scope: MarketScope,
    source_type: SourceType,
    timeframe: Timeframe,
    klines: Vec<NormalizedKline>,
    generated_at_ms: u64,
}
```

## 核心约束

- 上层模块只能消费规范化对象
- 原始交易所字段只能作为适配层或调试信息存在
- `source_type` 必须跟随对象进入意图层之前的全部链路
- 规范化对象必须是跨环境稳定的，同一对象在实盘、事实测试和回测中语义一致

## 与主链的关系

`DataRequest` 定义需求，`NormalizedMarketData` 响应需求。之后意图生成器、代理、测试器和回测器只能读取规范化对象，而不能回退到原始 JSON 或交易所特定结构。
