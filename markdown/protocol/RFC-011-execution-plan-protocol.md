# RFC-011 Execution Plan Protocol

执行计划协议定义风控之后的最终执行表达。它不是交易所原始订单结构，而是系统内部已经获批的执行对象。

## 协议目标

- 把风控批准结果收敛为稳定执行表达
- 让执行层拥有场地映射空间，但不破坏上层协议稳定性
- 保持 ExecutionPlan 与交易所 API 解耦

## ExecutionPlan

```rust
struct ExecutionPlan {
    plan_id: String,
    source_decision_id: String,
    instrument: String,
    side: OrderSide,
    approved_quantity: f64,
    price_constraint: Option<PriceConstraint>,
    order_type: OrderType,
    urgency: Urgency,
    time_in_force: TimeInForce,
    reduce_only: bool,
    route_hint: Option<RouteHint>,
    valid_until_ms: u64,
    created_at_ms: u64,
}
```

## 相关类型

```rust
struct PriceConstraint {
    limit_price: Option<f64>,
    max_slippage_bps: Option<u32>,
}

enum TimeInForce {
    Gtc,
    Ioc,
    Fok,
    Day,
}

struct RouteHint {
    preferred_venue: Option<String>,
    account_slot: Option<String>,
}
```

## 边界说明

- `ExecutionPlan` 只描述系统内部最终批准的执行意图
- 只有执行引擎可以把它转换为交易所原始订单
- 上层模块不应依赖场地接口字段
