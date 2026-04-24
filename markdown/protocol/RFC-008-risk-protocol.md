# RFC-008 Global Risk Control Protocol

全局风险控制协议定义系统中的统一风险主闸门。每个策略运行链只能存在一个全局风险控制模块，它接收代理输出的订单候选和当前运行态信息，并给出统一裁决。

## 协议目标

- 让所有代理最终受同一风险模块约束
- 固定风险参数边界与资源限制
- 保证系统在多代理并行时仍可预测、可解释、可回放
- 固定风险层“并行检查、统一归并、唯一裁决”的结构

## GlobalRiskControllerSpec

```rust
struct GlobalRiskControllerSpec {
    risk_id: String,
    name: String,
    version: String,
    max_total_position_ratio: f64,
    max_single_instrument_ratio: f64,
    max_order_notional: f64,
    max_daily_loss: f64,
    max_drawdown_ratio: f64,
    max_orders_per_minute: u32,
    allow_new_long: bool,
    allow_new_short: bool,
    enabled: bool,
}
```

## 约束原则

- 风控器只接收订单候选与当前运行态信息
- 风控器不直接读取原始 Intent
- 风控器不直接读取原始市场数据
- 风控器必须是系统中的统一主闸门，而不是局部补丁

## 并行与归并

风险层允许多个约束模块并行检查同一份代理决策，例如仓位、杠杆、回撤、流动性、波动率、相关性和熔断规则。但这些检查结果不能各自直接推动执行。

风险层必须在层末完成统一归并，并收敛为唯一全局风险决策。归并后才允许生成 `RiskDecision` 并进入 `ExecutionPlan` 阶段。
