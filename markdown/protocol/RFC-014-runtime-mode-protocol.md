# RFC-014 Runtime Mode Protocol

运行模式协议定义系统支持的三种运行模式，并明确这些模式只能影响数据提供器、成交语义和账户来源，而不能改变核心协议语义。

## 协议目标

- 统一定义实盘执行、事实测试和历史回测
- 保证不同模式下 `Intent -> Agent -> Risk -> Execution` 的语义一致
- 让同一策略定义可以在不同环境中复用

## RuntimeMode

```rust
enum RuntimeMode {
    LiveExecution,
    FactSimulation,
    HistoricalBacktest,
}
```

## RuntimeModeConfig

```rust
struct RuntimeModeConfig {
    mode: RuntimeMode,
    data_provider_id: String,
    account_provider_id: String,
    execution_adapter_id: String,
    fill_model: FillModel,
    enable_persistence_events: bool,
    enable_reports: bool,
}
```

## 相关类型

```rust
enum FillModel {
    RealExchange,
    SimulatedImmediate,
    SimulatedSlippage,
    HistoricalReplay,
}
```

## 约束原则

- `RuntimeMode` 不改变 `DataRequest` 语义
- `RuntimeMode` 不改变 `NormalizedMarketData` 语义
- `RuntimeMode` 不改变 `Intent`、`Agent`、`RiskDecision` 和 `ExecutionPlan` 语义
- 运行模式只改变环境提供方式，不改变核心对象定义
