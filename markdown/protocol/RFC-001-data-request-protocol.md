# RFC-001 Data Request Protocol

数据请求协议定义系统如何表达当前需要什么数据语义对象，而不是表达去哪个交易所调哪个接口。它是 QRPC 主链的起点，必须稳定地服务实盘执行、事实测试和历史回测。

## 协议目标

- 统一表达数据需求，而不是端点调用细节
- 把数据请求收敛为稳定的数据语义对象
- 让上层只依赖数据类型，不依赖交易所字段或适配器能力名
- 为单机环境下的数据调度、缓存和按需加载提供统一入口

## 核心修订

QRPC 中，数据类型只有两个一级类型：

- `FactPrice`
- `KlineRange`

`source_type` 必须被视为数据类型体系内部的子维度，用于标记语义来源，例如现货、永续、期货、指数、标记价格或聚合价格。它不是交易所接口来源信息，也不是适配器层实现细节。

## DataRequest

```rust
struct DataRequest {
    request_id: String,
    instrument: String,
    market_scope: MarketScope,
    primary_data_type: PrimaryDataType,
    source_type: SourceType,
    timeframe: Option<Timeframe>,
    lookback_count: Option<u32>,
    time_range: Option<TimeRange>,
    precision_policy: PrecisionPolicy,
    usage_tag: UsageTag,
    priority: u8,
    is_realtime: bool,
    requested_at_ms: u64,
}
```

## 字段约束

- `instrument`: 统一标的标识，例如 `BTCUSDT`
- `market_scope`: 市场域，用于表达当前请求所处的市场上下文
- `primary_data_type`: 只允许 `FactPrice` 或 `KlineRange`
- `source_type`: 一级数据类型下的语义子维度
- `timeframe`: 事实价格通常可为空，K 线通常必填
- `lookback_count`: 回看窗口大小
- `time_range`: 历史装载或回测常用的显式时间范围
- `precision_policy`: 用于控制数值精度与舍入规则
- `usage_tag`: 用于区分实时执行、意图计算、事实测试或回测用途
- `priority`: 用于单机资源有限时的调度优先级

## 相关枚举与子结构

```rust
enum MarketScope {
    Spot,
    Margin,
    Perpetual,
    Futures,
    Options,
    Index,
    Composite,
}

enum PrimaryDataType {
    FactPrice,
    KlineRange,
}

enum SourceType {
    SpotTrade,
    SpotTicker,
    PerpetualTrade,
    PerpetualMark,
    PerpetualIndex,
    FuturesTrade,
    FuturesMark,
    FuturesIndex,
    IndexPrice,
    Aggregated,
}

enum Timeframe {
    Tick,
    Ms100,
    Sec1,
    Sec5,
    Min1,
    Min3,
    Min5,
    Min15,
    Min30,
    Hour1,
    Hour4,
    Day1,
    Week1,
}

struct TimeRange {
    start_ms: u64,
    end_ms: u64,
}

struct PrecisionPolicy {
    price_scale: u8,
    quantity_scale: u8,
    rounding_mode: RoundingMode,
}

enum RoundingMode {
    Floor,
    Ceil,
    Round,
    Truncate,
}

enum UsageTag {
    LiveExecution,
    IntentComputation,
    FactSimulation,
    HistoricalBacktest,
    Diagnostics,
}
```

## 解释原则

- `FactPrice` 不是单一 `last price`，而是事实市场快照类别
- `KlineRange` 也必须带 `source_type`，用于区分现货 K 线、合约 K 线、指数 K 线或标记价格 K 线
- 同一个 `instrument` 可以对应多条不同 `source_type` 的请求
- 请求对象只表达需要什么数据语义，不表达最终走哪家交易所和哪条接口

## 与适配层的关系

`DataRequest` 是语义层协议，`Source Adapter` 是实现层协议。`source_type` 不等于适配器来源，也不等于交易所名字。适配层负责把 `DataRequest` 翻译成具体调用，但不能修改 `DataRequest` 的核心语义。
