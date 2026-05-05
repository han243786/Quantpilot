# RFC-015 Runtime Event Protocol

运行事件协议定义系统如何记录关键决策链事件。QuantPilot 不保存全部原始市场数据，但必须保存高价值链路事件，以支持回放、排错、分析和治理。

## 协议目标

- 记录关键过程，而不是记录所有原始流水
- 让事件具备稳定结构，支持单机轻量存储
- 为回放、报表、错误分析和治理提供统一证据链

## RuntimeEvent

```rust
struct RuntimeEvent {
    event_id: String,
    event_type: RuntimeEventType,
    source_id: String,
    instrument: Option<String>,
    event_time_ms: u64,
    summary: String,
    detail_ref: Option<String>,
    severity: EventSeverity,
}
```

## 相关枚举

```rust
enum RuntimeEventType {
    DataUpdated,
    IntentGenerated,
    AgentEvaluated,
    RiskEvaluated,
    ExecutionPlanned,
    OrderUpdated,
    PositionUpdated,
    ErrorOccurred,
    ReportGenerated,
}

enum EventSeverity {
    Trace,
    Info,
    Warn,
    Error,
    Fatal,
}
```

## 专用事件结构

```rust
struct DataUpdatedEvent {
    request_id: String,
    data_id: String,
    primary_data_type: PrimaryDataType,
    source_type: SourceType,
    updated_at_ms: u64,
}

struct IntentGeneratedEvent {
    intent_id: String,
    generator_id: String,
    instrument: String,
    strength: f64,
    confidence: f64,
    generated_at_ms: u64,
}

struct RiskEvaluatedEvent {
    candidate_id: String,
    decision_id: String,
    action: RiskAction,
    risk_score: f64,
    evaluated_at_ms: u64,
}

struct ErrorEvent {
    error_code: String,
    message: String,
    component: String,
    retryable: bool,
    occurred_at_ms: u64,
}
```

## 约束原则

- 事件只保存高价值字段和必要上下文摘要
- 事件必须与主链对象可关联
- 事件协议不能退化为自由日志文本
