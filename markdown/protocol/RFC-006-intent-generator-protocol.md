# RFC-006 Intent Generator Protocol

意图生成器协议定义每一种 Intent 如何被声明、调度和约束。它不规定算法内部如何实现，而规定生成器必须声明输入数据依赖、触发条件、参数配置和输出意图范围。

## 协议目标

- 让意图生成器成为协议核心中的一等对象
- 用 `DataRequest` 表达输入依赖，而不是直接引用交易所接口
- 固定触发语义，支持统一调度和统一测试
- 约束单机运行时间与冷却间隔

## IntentGeneratorSpec

```rust
struct IntentGeneratorSpec {
    generator_id: String,
    name: String,
    version: String,
    enabled: bool,
    input_requests: Vec<DataRequest>,
    trigger_policy: TriggerPolicy,
    parameter_schema: Vec<GeneratorParameter>,
    output_intent_type: OutputIntentType,
    cooldown_ms: u64,
    max_compute_time_ms: u32,
}
```

## 相关类型

```rust
enum TriggerPolicy {
    OnFactUpdate,
    OnKlineClose,
    FixedInterval,
    Manual,
}

struct GeneratorParameter {
    key: String,
    value_type: ParameterValueType,
    required: bool,
    default_value: Option<String>,
    description: Option<String>,
}

enum ParameterValueType {
    Bool,
    I32,
    I64,
    U32,
    U64,
    F32,
    F64,
    String,
}

enum OutputIntentType {
    Directional,
    MeanReversion,
    Breakout,
    VolatilityControl,
    Custom,
}
```

## 强约束

- `input_requests` 必须使用 `DataRequest` 语义对象
- 生成器不能直接声明交易所端点依赖
- 触发条件必须是稳定的运行事件语义
- 生成器必须受 `cooldown_ms` 与 `max_compute_time_ms` 约束
- 生成器输出的是 `Intent`，不是订单或执行请求
