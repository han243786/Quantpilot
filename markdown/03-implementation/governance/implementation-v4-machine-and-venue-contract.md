# v4 状态机与交易场所能力静态契约

> 生效目标: v4.0.0 Phase 1 | 实现锚点: `qrpc_core_ir::v4`

---

## 目标

本契约为 v4 状态机化 QuantScript 和 ExecutionMachine 能力矩阵提供第一批静态类型锚点。该阶段只定义可序列化结构和静态校验，不接入现有 v3.7.1 runtime，不改变旧策略行为。

## 状态机契约

实现入口:

- `qrpc_core_ir/src/v4.rs`
- `V4MachineContract`
- `V4MachineGraphContract`
- `QsStateMachineProfile`
- `MachineTemplateKind`
- `MachineState`
- `StateGroup`
- `MachineTransition`
- `MachineMemoryField`
- `MachineGraphEdge`
- `MachineGraphRiskPlane`
- `MachineEventCatalog`
- `MachineEventTypeSpec`
- `MachineEventPayloadField`
- `RuntimeModeContract`
- `RuntimeTradingModeSpec`

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

## 顶层 machine graph

profile 版本:

```text
quantpilot/machine-graph-contract/v1
```

v4 顶层 graph 仍然必须是有向无环图。每个顶层节点可以是状态机，但顶层连接不得形成环。

静态校验必须保证:

- `schema_version` 为 `quantpilot/machine-graph-contract/v1`。
- `graph_id` 非空。
- 至少一个 machine。
- machine id 不能重复。
- edge id 不能重复。
- edge 必须声明 `event_type`。
- edge 的 source 和 target 必须引用已声明 machine。
- edge 不能自连接。
- 顶层 machine graph 必须无环。
- 如 transition、action 或 edge 引用事件，graph 必须声明 `event_catalog`。
- 被引用的 `event_type` 必须在 `event_catalog` 中声明。

执行路径还必须满足 Risk Plane 约束:

- 含 `Execution` machine 的 graph 必须声明 `risk_plane`。
- `risk_plane.required` 必须为 true。
- `risk_plane` 至少包含一个 machine。
- `risk_plane` machine 必须使用 `Decision` 模板。
- `risk_plane` machine priority 必须不低于 `9000`。
- `Execution` machine 的顶层入边必须来自 `risk_plane`。
- `Execution` machine 必须至少有一条来自 `risk_plane` 的入边。

该约束用于保证 DecisionMachine 可以表达风控，但真实执行路径仍不能绕过 runtime 独立高优先级安全平面。

## QS 状态机 profile

实现入口:

- `QsStateMachineProfile`
- `QsStatePolicy`
- `QsActionBlockPolicy`
- `QsMemoryPolicy`
- `QsEventPolicy`
- `QsPriorityPolicy`
- `QsRiskPlanePolicy`
- `default_v4_qs_state_machine_profile()`

profile 版本:

```text
quantpilot/qs-state-machine-profile/v1
```

第一版 QS 状态机 profile 必须保持强 DSL，且只允许受控 action block。action block 可以:

- 发出事件。
- 写入本机强类型 memory。
- 写入诊断信息。

action block 禁止:

- 直接访问网络。
- 直接访问文件系统。
- 直接提交订单。
- 写入其他 machine 的 memory。
- 无界循环。
- 动态 eval。

第一版 profile 必须允许三种标准模板:

- `Observation`
- `Decision`
- `Execution`

同时必须保留以下边界:

- 允许用户自定义 priority。
- transition 必须由强类型事件驱动。
- machine memory 必须强类型。
- machine memory 必须有默认值或显式 nullable。
- DecisionMachine 可以表达风控判断。
- runtime 必须保留独立高优先级 Risk Plane。
- QS 不能绕过 runtime Risk Plane。

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

## 事件目录

profile 版本:

```text
quantpilot/machine-event-catalog/v1
```

事件目录用于把 v4 graph 内的事件从普通字符串收束为强类型契约。第一版静态校验必须保证:

- `schema_version` 为 `quantpilot/machine-event-catalog/v1`。
- 至少声明一个 event。
- `event_type` 非空且不能重复。
- payload 字段名非空且不能重复。
- payload 字段必须声明 `type_name`。
- payload 字段不能同时 `required=true` 且 `nullable=true`。
- transition 消费的事件必须在目录中。
- action emit 的事件必须在目录中。
- graph edge 使用的事件必须在目录中。
- 如果事件声明了 `allowed_emitters`，transition source、action emitter、edge source 必须命中。
- 如果事件声明了 `allowed_consumers`，transition 所属 machine、edge target 必须命中。

这让事件模型可以先在静态层证明“谁能发、谁能收、payload 应是什么类型”，再进入 parser、lowering 和 runtime。

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
- `RuntimeModeContract`

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

运行模式与能力来源必须匹配:

- `provider_actual` 模式只接受 `provider_native` 能力。
- `local_simulated` 模式只接受 `runtime_simulated` 能力。
- 能力必须显式包含当前 `RuntimeTradingMode`，否则运行前拒绝。

默认构造:

- `unsupported_v4_first_wave_matrix(...)` 会为 v4 第一批执行能力全部生成 `Unsupported` 条目。
- 后续 VenueAdapter 只能逐项把能力提升为 `ProviderNative` 或 `RuntimeSimulated`。
- 不允许通过省略能力来表达“稍后再说”；省略只能触发静态校验失败。

## 四种运行模式契约

profile 版本:

```text
quantpilot/runtime-mode-contract/v1
```

第一版必须显式声明四种模式:

| 模式 | Account Domain | Settlement Authority | 事件来源 | 真实下单 |
|------|----------------|----------------------|----------|----------|
| `PaperActual` | `paper` | `provider_actual` | `provider_actual` | 允许 |
| `PaperSimulated` | `paper` | `local_simulated` | `local_simulated` | 禁止 |
| `LiveActual` | `live` | `provider_actual` | `provider_actual` | 允许 |
| `LiveSimulated` | `live` | `local_simulated` | `local_simulated` | 禁止 |

四种模式必须共享执行事件:

- `order_acknowledged`
- `order_rejected`
- `order_partially_filled`
- `order_filled`
- `fee_charged`
- `portfolio_changed`

静态校验必须保证:

- 四种模式全部声明，不能重复。
- `PaperActual` 和 `LiveActual` 必须使用 provider 实际成交与 provider 事件来源。
- `PaperSimulated` 和 `LiveSimulated` 必须使用本地成交引擎、本地账本与本地模拟事件来源。
- `LiveSimulated` 必须读取真实账户上下文，但禁止真实下单。
- 所有模式都必须要求 runtime Risk Plane。
- 每个模式都必须声明完整执行事件集合。

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
- 顶层 machine graph 可通过 DAG 校验。
- 顶层 machine graph 出现环会失败。
- 顶层 edge 指向未知 machine 会失败。
- 含 Execution machine 的 graph 缺少 Risk Plane 会失败。
- Execution machine 被非 Risk Plane 入边直连会失败。
- Risk Plane machine priority 过低会失败。
- 事件目录强类型 payload 可通过。
- 事件目录 payload 缺少类型会失败。
- graph 引用事件但缺少事件目录会失败。
- transition 引用未声明事件会失败。
- 未授权 emitter 发出事件会失败。
- 默认 QS 状态机 profile 可通过。
- QS profile 必须允许三种标准模板。
- QS action block 不能直接提交订单。
- 嵌套状态机在第一版保持 reserved。
- runtime 必须保留独立高优先级 Risk Plane。
- 默认四模式契约可通过。
- 四模式契约缺少任一模式会失败。
- `LiveSimulated` 允许真实下单会失败。
- 四模式契约缺少执行事件会失败。
- `provider_actual` 模式不能使用 `runtime_simulated` 能力。
- `local_simulated` 模式必须使用 `runtime_simulated` 能力。
- Venue capability 重复声明会失败。
- 缺失能力不会被当作 supported。
- v4 第一批能力必须显式标记来源。
- 默认 unsupported 矩阵不会假装支持任何订单能力。
