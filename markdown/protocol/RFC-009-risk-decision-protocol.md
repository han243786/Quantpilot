# RFC-009 Risk Decision Protocol

风险决策协议定义全局风险控制器对订单候选给出的正式裁决结果。它是 Agent 与 ExecutionPlan 之间的唯一稳定约束出口。

## 协议目标

- 把风险裁决统一为可执行的稳定结果类型
- 让执行层在不理解全部风险细节的情况下仍能正确工作
- 保留足够原因信息以支持审计、回放和解释

## RiskDecision

```rust
struct RiskDecision {
    decision_id: String,
    candidate_id: String,
    action: RiskAction,
    approved_quantity: Option<f64>,
    risk_score: f64,
    reason_code: RiskReasonCode,
    reason_text: Option<String>,
    decided_at_ms: u64,
}
```

## 相关类型

```rust
enum RiskAction {
    Approve,
    Reduce,
    Delay,
    Reject,
}

enum RiskReasonCode {
    WithinLimit,
    ExceedTotalExposure,
    ExceedInstrumentExposure,
    ExceedOrderNotional,
    ExceedDailyLoss,
    ExceedDrawdown,
    RateLimited,
    LongDisabled,
    ShortDisabled,
    InvalidState,
    Custom,
}
```

## 解释原则

- `Approve`: 允许执行
- `Reduce`: 降低规模或强度后执行
- `Delay`: 当前不适合执行，但允许下一周期重评估
- `Reject`: 彻底阻断该候选

风险决策必须由统一风险模块产生，不能由代理或执行层私自模拟。

`RiskDecision` 还承担风险层同步屏障之后的唯一输出职责。也就是说，只有在全部风险检查完成并归并之后，系统才允许生成正式的 `RiskDecision` 对象。
