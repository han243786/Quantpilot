# RFC-004 Agent Protocol

代理协议定义系统中的代理如何接收多个 Intent、完成加权与冲突裁决，并输出订单候选。代理不是信号生成器，也不是风控器，更不是直接下单器。

## 协议目标

- 把多个 Intent 转换为订单候选
- 支持多策略组合与冲突裁决
- 限制代理资源行为，避免单机运行被复杂逻辑拖垮
- 固定 `Intent -> Agent -> Risk Gate` 的主链边界
- 固定“同层并行、层末归并、归并后过屏障”的运行方式

## AgentSpec

```rust
struct AgentSpec {
    agent_id: String,
    name: String,
    version: String,
    enabled: bool,
    accepted_intent_types: Vec<OutputIntentType>,
    accepted_instruments: Vec<String>,
    has_internal_state: bool,
    conflict_policy: ConflictPolicy,
    weighting_policy: WeightingPolicy,
    output_policy: OutputPolicy,
    evaluation_interval_ms: u64,
    max_candidates_per_cycle: u16,
}
```

## 相关类型

```rust
enum ConflictPolicy {
    HighestConfidenceWins,
    HighestStrengthWins,
    WeightedMerge,
    Netting,
    RejectOnConflict,
}

enum WeightingPolicy {
    EqualWeight,
    ConfidenceWeighted,
    StrengthWeighted,
    Custom,
}

struct OutputPolicy {
    allow_partial_target: bool,
    allow_multi_candidate: bool,
    max_total_exposure_ratio: f64,
    max_single_order_ratio: f64,
}
```

## OrderCandidate

```rust
struct OrderCandidate {
    candidate_id: String,
    agent_id: String,
    instrument: String,
    side: OrderSide,
    target_quantity: f64,
    target_price: Option<f64>,
    order_type: OrderType,
    urgency: Urgency,
    reduce_only: bool,
    derived_from_intents: Vec<String>,
    created_at_ms: u64,
}
```

```rust
enum OrderSide {
    Buy,
    Sell,
}

enum OrderType {
    Market,
    Limit,
    Stop,
    StopMarket,
    PostOnly,
}

enum Urgency {
    Low,
    Medium,
    High,
    Immediate,
}
```

## 强约束

- Agent 不得直接请求市场数据
- Agent 不得绕过风险控制直接下单
- Agent 只能消费符合声明范围的 Intent
- Agent 必须限制评估频率和单周期输出规模
- Agent 不得产生无界递归调用或无界候选输出

## 并行与归并

Agent 层允许多个代理并行评估，但这些代理必须：

- 读取同一份冻结的 Intent 集合
- 输出统一格式的代理结果或订单候选
- 不直接修改 Risk 层数据结构

在进入 Risk 层之前，所有代理输出必须先进入统一归并阶段。只有代理归并完成，系统才允许跨过同步屏障进入全局风险控制。
