# RFC-003 Runtime State Protocol

运行态协议定义单机热存储中的合法运行对象集合。QuantPilot 采用热存储优先设计，因此必须对运行态对象做明确约束，避免系统在个人电脑环境下失控膨胀。

## 协议目标

- 定义单机内存中的合法对象边界
- 保证没有数据库时系统运行状态仍可控
- 让实盘、事实测试和历史回测共享同一套运行态语义

## 合法对象集合

运行态协议只允许存在以下七类热态对象：

- 规范化事实价格缓存
- 规范化 K 线窗口缓存
- 当前有效 Intent 集
- Agent 内部状态
- 风险控制状态
- 订单状态
- 持仓状态

## RuntimeStateSnapshot

```rust
struct RuntimeStateSnapshot {
    snapshot_id: String,
    mode: RuntimeMode,
    fact_price_cache_size: u32,
    kline_series_cache_size: u32,
    active_intent_count: u32,
    active_agent_state_count: u32,
    open_order_count: u32,
    open_position_count: u32,
    captured_at_ms: u64,
}
```

## 缓存与状态对象

```rust
struct FactPriceCacheEntry {
    instrument: String,
    source_type: SourceType,
    latest: NormalizedFactPrice,
}

struct KlineCacheEntry {
    instrument: String,
    source_type: SourceType,
    timeframe: Timeframe,
    series: NormalizedKlineSeries,
}

struct AgentState {
    agent_id: String,
    last_evaluated_ms: u64,
    last_output_candidate_id: Option<String>,
    internal_state_blob: Option<String>,
}

struct OrderState {
    order_id: String,
    instrument: String,
    side: OrderSide,
    quantity: f64,
    filled_quantity: f64,
    avg_fill_price: Option<f64>,
    status: OrderStatus,
    created_at_ms: u64,
    updated_at_ms: u64,
}

struct PositionState {
    instrument: String,
    long_quantity: f64,
    short_quantity: f64,
    avg_long_price: Option<f64>,
    avg_short_price: Option<f64>,
    unrealized_pnl: f64,
    realized_pnl: f64,
    updated_at_ms: u64,
}
```

```rust
enum OrderStatus {
    Pending,
    Submitted,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
}
```

## 约束原则

- 运行态对象必须使用协议结构表达
- 不允许出现无界内存增长的隐式对象
- 不允许用自由结构替代协议对象长期驻留热态
- 状态快照的语义不能依赖数据库是否存在
