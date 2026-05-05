# RFC-005 Intent Protocol

Intent 协议定义系统如何表达中间交易主张。Intent 是 QRPC 中最重要的中间层对象之一，它既不是信号，也不是订单，而是建立在规范化市场数据之上的可执行前判断。

## 协议目标

- 统一表达跨环境可复用的交易主张
- 保持策略层、代理层、风险层和执行层解耦
- 记录意图生成所依赖的数据语义组合
- 支持实盘、事实测试和回测统一追踪

## Intent

```rust
struct Intent {
    intent_id: String,
    generator_id: String,
    instrument: String,
    direction_bias: DirectionBias,
    strength: f64,
    confidence: f64,
    suggested_weight: f64,
    valid_from_ms: u64,
    valid_until_ms: u64,
    data_dependencies: Vec<IntentDataDependency>,
    thesis: Option<String>,
    metadata: IntentMetadata,
}
```

## 相关类型

```rust
enum DirectionBias {
    Long,
    Short,
    Neutral,
    ReduceLong,
    ReduceShort,
    Exit,
}

struct IntentDataDependency {
    primary_data_type: PrimaryDataType,
    source_type: SourceType,
    timeframe: Option<Timeframe>,
    lookback_count: Option<u32>,
}

struct IntentMetadata {
    regime_tag: Option<String>,
    model_version: Option<String>,
    feature_hash: Option<String>,
    created_at_ms: u64,
}
```

## 解释原则

- `Intent` 只表达交易主张，不表达交易所订单字段
- `data_dependencies` 必须显式记录 `primary_data_type + source_type` 组合
- 同一 Intent 在实盘、事实测试和回测中必须保持同一语义
- Intent 可以被多个代理复用，但不能携带场地级执行字段

## 边界说明

Intent 回答的是：

- 当前偏向哪个方向
- 判断强度与置信度如何
- 建议在多长时间内有效
- 依赖了哪些数据语义对象

Intent 不回答的是：

- 去哪个交易所下单
- 下哪种场地特定订单
- 如何拆单
- 如何绕过风险直接执行
