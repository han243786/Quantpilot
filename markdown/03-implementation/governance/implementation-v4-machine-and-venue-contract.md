# v4 状态机与交易场所能力静态契约

> 生效目标: v4.0.0 Phase 1 | 实现锚点: `qrpc_core_ir::v4`

---

## 目标

本契约为 v4 状态机化 QuantScript 和 ExecutionMachine 能力矩阵提供第一批静态类型锚点。该阶段只定义可序列化结构和静态校验，不接入现有 v3.7.1 runtime，不改变旧策略行为。

## 状态机契约

实现入口:

- `qrpc_core_ir/src/v4.rs`
- `V4MachineContract`
- `MachineTemplateKind`
- `MachineState`
- `StateGroup`
- `MachineTransition`
- `MachineMemoryField`

第一版模板:

| 模板 | 含义 |
|------|------|
| `Observation` | 数据、指标、特征、证据 |
| `Decision` | 意图、代理、风控判断 |
| `Execution` | 执行计划、订单、成交、资产事件 |

静态校验必须保证:

- `schema_version` 为 `quantpilot/machine-contract/v1`。
- `machine_id` 非空。
- 至少一个 state。
- 恰好一个 initial state。
- `state_group` 只引用已声明 state。
- state 只引用已声明 group。
- transition 的 `from_state` 和 `to_state` 必须存在。
- transition 必须声明 `event_type`。
- memory 字段必须声明 `type_name`。
- 非 nullable memory 字段必须有默认值。

## 扁平状态与 state group

v4.0.0 Phase 1 只支持扁平 state 和 state group。`state_group` 只是语义分组，不是嵌套状态机。

禁止:

- 任意层级嵌套状态机。
- 父子状态路径。
- 跨层 transition 优先级。

未来若支持嵌套状态机，必须增加性能和可审计性警告。

## 事件驱动边界

transition 必须绑定事件:

```text
MachineTransition.event.event_type
```

没有 `event_type` 的 transition 在静态校验阶段失败。

## 缓存、静默与恢复

第一批静态枚举:

- `MachineCachePolicy`
- `MachineSilencePolicy`
- `MachineRecoveryPolicy`

`Pinned` machine 不应使用 `return_last_then_recover` 语义。该约束已经进入静态校验。

## 交易场所能力矩阵

实现入口:

- `VenueCapabilityMatrix`
- `VenueCapability`
- `ExecutionCapabilityKind`
- `CapabilitySupportSource`
- `RuntimeTradingMode`

能力来源:

| 来源 | 含义 |
|------|------|
| `ProviderNative` | 交易所或券商原生支持 |
| `RuntimeSimulated` | QuantPilot 本地模拟或合成 |
| `Unsupported` | 不支持且未模拟 |

静态校验必须保证:

- `schema_version` 为 `quantpilot/venue-capability-matrix/v1`。
- `venue_id` 非空。
- 单个 venue 中同一执行能力不能重复声明。
- 非 `Unsupported` 能力必须声明至少一个支持的运行模式。
- v4 第一批执行能力必须全部显式声明来源，即使当前来源是 `Unsupported`。

缺失能力默认视为 `Unsupported`，不得静默降级。

默认构造:

- `unsupported_v4_first_wave_matrix(...)` 会为 v4 第一批执行能力全部生成 `Unsupported` 条目。
- 后续 VenueAdapter 只能逐项把能力提升为 `ProviderNative` 或 `RuntimeSimulated`。
- 不允许通过省略能力来表达“稍后再说”；省略只能触发静态校验失败。

## 当前非目标

- 不接入 QuantScript parser。
- 不接入 Core IR lowering。
- 不接入 RuntimeCoordinator。
- 不接入真实 VenueAdapter。
- 不改变现有 `OrderType`、`TimeInForce`、`ExecutionPlan` 行为。

## 验证

针对性验证:

```powershell
cargo test -p qrpc-core-ir v4
```

该测试覆盖:

- 扁平 state group 可通过。
- transition 缺少事件会失败。
- transition 指向未知 state 会失败。
- Venue capability 重复声明会失败。
- 缺失能力不会被当作 supported。
- v4 第一批能力必须显式标记来源。
- 默认 unsupported 矩阵不会假装支持任何订单能力。
