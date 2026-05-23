# v4 状态机与交易场所能力静态契约

> 生效目标: v4.0.0 Phase 1-3 | 实现锚点: `qrpc_core_ir::v4`, `quantscript::v4_static_audit`

---

## 目标

本契约为 v4 状态机化 QuantScript 和 ExecutionMachine 能力矩阵提供第一批静态类型锚点，并在 Phase 2 增加编译期能力报告入口，在 Phase 3 增加 v4 QS 静态 parse/analyze/report 入口。当前阶段只定义可序列化结构、静态校验和报告生成，不接入现有 v3.7.1 runtime，不改变旧策略行为。

## 状态机契约

实现入口:

- `qrpc_core_ir/src/v4.rs`
- `quantscript/src/v4_static_audit.rs`
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
- `QsTypeSystemContract`
- `QsTypeRef`
- `V4VersionManifest`
- `PluginGovernanceContract`
- `PluginManifestSpec`
- `ReproducibilityContract`
- `ComplexityBudgetContract`
- `DeveloperLearningPipelineContract`
- `V4StaticContractBundle`
- `V4CompileTimeCapabilityRequest`
- `V4CompileTimeCapabilityReport`
- `audit_v4_quant_script_static`
- `V4QsStaticAuditReport`

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

## QS 强类型系统

profile 版本:

```text
quantpilot/qs-type-system/v1
```

第一批 scalar 类型:

```text
bool
int
decimal
time
duration
price
quantity
notional
percent
ratio
fee
slippage
leverage
symbol
venue
account
side
position_side
order_type
time_in_force
freshness
runtime_mode
order_permission
```

第一批 composite 类型:

```text
optional<T>
list<T, max=N>
map<K, V, max=N>
fresh<T>
stale<T>
```

静态校验必须保证:

- `schema_version` 为 `quantpilot/qs-type-system/v1`。
- 第一批 scalar 类型全部显式声明。
- 第一批 composite 类型全部显式声明。
- scalar 和 composite 类型不能重复。
- `list` 与 `map` 必须声明 `max_items`，且不能超过上限。
- composite 类型必须 `replay_safe`。
- 类型嵌套深度不能超过 `max_nesting_depth`。
- map key 必须是已声明 scalar 类型。

该契约先建立类型系统边界，不接入 QS parser。后续 parser/analyzer 只能引用该类型目录，不得用普通字符串绕过强类型校验。

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

## 版本演进 Manifest

profile 版本:

```text
quantpilot/version-manifest/v1
```

版本 manifest 固定静态契约阶段所依赖的版本字段:

- `qs_language_version`
- `type_schema_version`
- `machine_template_version`
- `capability_matrix_version`

静态校验必须保证:

- 类型 schema 指向 `quantpilot/qs-type-system/v1`。
- machine template 指向 `quantpilot/machine-contract/v1`。
- capability matrix 指向 `quantpilot/venue-capability-matrix/v1`。
- 新增类型兼容。
- 新增默认字段兼容。
- 收紧类型必须要求 migration。
- 删除类型必须先 deprecated。
- 语义变化必须提升 schema version。

## 插件治理契约

profile 版本:

```text
quantpilot/plugin-governance/v1
```

第一版允许三类插件:

- `Pure`
- `Runtime`
- `Venue`

插件 manifest 必须表达:

- name
- version
- input schema
- output schema
- deterministic
- side effect
- runtime permission
- network permission
- capability matrix
- test fixture

静态校验必须保证:

- QS 只声明能力，插件实现能力。
- 真实下单只能通过 venue plugin + Risk Plane。
- pure plugin 必须 deterministic。
- pure plugin 不得有 side effect、runtime permission、network permission。
- runtime plugin 不得访问 provider network。
- venue plugin 必须声明 provider network side effect。
- venue plugin 必须使用 venue adapter runtime permission。
- venue plugin 必须声明 v4 第一批能力矩阵。

## 复现证据契约

profile 版本:

```text
quantpilot/reproducibility-contract/v1
```

v4.0.0 第一版目标是关键决策路径复现。静态校验必须要求以下 evidence:

- `strategy_run_id`
- `event_sequence`
- `input_snapshot_id`
- `memory_change_log`
- `capability_hash`
- `deployment_revision`
- `order_capability_source`
- `risk_decision_evidence`

事件 envelope 必须携带:

- `event_id`
- `event_type`
- `event_time`
- `source`
- `payload`
- `freshness`
- `sequence`
- `replayable`

逐 tick 完全复现第一版仍是非目标，不能提前作为必备能力声明。

## 复杂度预算契约

profile 版本:

```text
quantpilot/complexity-budget/v1
```

静态阶段必须能表达并校验:

- `state_count`
- `transition_count`
- `memory_field_count`
- `plugin_call_count`
- `mode_count`
- `stale_dependency_count`
- `estimated_order_paths`
- `event_rate_estimate`

预算为 0 或实际指标超过预算时静态校验失败。未来若启用嵌套状态机，必须在该预算层增加嵌套深度和回放成本警告。

## 学习流水线静态契约

profile 版本:

```text
quantpilot/learning-pipeline/v1
```

静态校验必须保证:

- 核心学习流水线在仓库中。
- 本地个人学习目录为 `markdown/learning/`。
- `markdown/learning/` 必须 gitignored。
- 写入个人学习记录必须有用户明确指令。
- 学习流水线不进入常规强制门禁。
- MAJOR closeout 必须询问 owner 必学机制。
- 第一版保持 owner-first，不提前泛化给所有开发者。

## 静态契约总包

profile 版本:

```text
quantpilot/static-contract-bundle/v1
```

`V4StaticContractBundle` 是 Phase 1 静态契约收口入口。它聚合:

- `V4VersionManifest`
- `QsStateMachineProfile`
- `QsTypeSystemContract`
- `RuntimeModeContract`
- `PluginGovernanceContract`
- `ReproducibilityContract`
- `ComplexityBudgetContract`
- `DeveloperLearningPipelineContract`
- `V4MachineGraphContract`
- `VenueCapabilityMatrix`
- `PluginManifestSpec`

静态总包必须至少包含一个 machine graph 和一个 venue matrix，并逐项调用子契约校验。该总包只证明 v4 语义边界完整，不接 parser、lowering、runtime 或 UI。

## 编译期能力报告

profile 版本:

```text
quantpilot/compile-time-capability-request/v1
quantpilot/compile-time-capability-report/v1
```

`V4CompileTimeCapabilityReport` 是 v4 Phase 2 的验收入口。它只生成静态报告，不接真实 runtime，不提交订单，也不改变旧策略行为。

报告输入:

- `graph_id`
- `venue_id`
- `runtime_mode`
- `required_execution_capabilities`
- `required_type_refs`
- `required_plugin_ids`

报告输出必须包含:

- graph、venue、runtime mode 是否可解析。
- 复杂度指标。
- 强类型引用检查结果。
- v4 第一批执行能力在目标 venue 与目标 runtime mode 下的来源、支持状态和拒绝原因。
- 插件 manifest 检查结果。
- 结构化诊断。
- `Accepted` / `Rejected` verdict。

Phase 2 拒绝规则:

- 请求 schema version 不匹配时拒绝。
- graph、venue 或 runtime mode 无法解析时拒绝。
- `required_type_refs` 不能通过 QS type system 时拒绝。
- `required_execution_capabilities` 为 `unsupported` 时拒绝。
- `provider_actual` 模式请求非 `provider_native` 能力时拒绝。
- `local_simulated` 模式请求非 `runtime_simulated` 能力时拒绝。
- 请求的能力未在 venue matrix 中显式声明时拒绝。
- 请求的插件不存在或 manifest 不通过治理契约时拒绝。
- 报告若附带 execution submission，必须拒绝。

该阶段只证明“编译前可以生成能力报告并拒绝不支持路径”。Phase 2 不执行新订单能力，不接 QS parser，不接 Core IR lowering，不接 runtime。

## v4 QS 静态审计

profile 版本:

```text
quantpilot/qs-v4-static-audit-report/v1
```

`audit_v4_quant_script_static(...)` 是 v4 Phase 3 的验收入口。它接收 v4 QS 源码和 `V4StaticContractBundle`，完成:

- parse: 解析 `v4_strategy`、`machine`、`state`、`state_group`、`memory`、`on event`、`edge`、`risk_plane`、`require capability/type/plugin`。
- analyze: 构造 `V4MachineGraphContract`，派生 `MachineEventCatalog`，执行 graph 静态校验。
- report: 生成 `V4CompileTimeCapabilityRequest`，调用 Phase 2 编译期能力报告，并返回 `V4QsStaticAuditReport`。

Phase 3 硬边界:

- 不调用 QS v1 lowering。
- 不调用 Core IR lowering。
- 不创建 `RuntimeCoordinator`。
- 不运行 PaperSimulated。
- 不提交订单。
- `V4QsStaticAuditReport.runtime_attached` 必须为 false。
- `V4QsStaticAuditReport.lowering_attached` 必须为 false。

第一版 v4 QS 静态语法只支持扁平状态机:

```text
v4_strategy <graph_id> {
  venue <venue_id>
  mode <paper_actual|paper_simulated|live_actual|live_simulated>
  require capability <capability>
  require type <type-ref>
  require plugin <plugin_id>

  machine <machine_id> <observation|decision|execution> priority <n> {
    state <state_id> [initial] [terminal]
    state_group <group_id> <state_id...>
    memory <name>: <type> [nullable]
    on <event_type> from <state_id> to <state_id> [emit <event...>] [write <memory...>]
  }

  edge <source_machine_id> -> <target_machine_id> on <event_type>
  risk_plane <machine_id...> priority <n>
}
```

Phase 3 拒绝规则:

- 顶层不是 `v4_strategy <graph_id> {` 时拒绝。
- 缺少 `venue` 或 `mode` 时拒绝。
- 未知 runtime mode、execution capability 或 QS type ref 时拒绝。
- machine header、state、state_group、memory、transition、edge、risk_plane 语法不符合静态语法时拒绝。
- machine 内再次声明 `machine` 时拒绝，嵌套状态机仍为 reserved。
- 不以 `on <event>` 表达的 transition 语法拒绝。
- graph 静态契约失败时拒绝。
- Phase 2 编译期能力报告 rejected 时拒绝，包括 required capability 为 `unsupported`、运行模式与能力来源不匹配、required plugin 缺失等。

## 当前非目标

- 不把 v4 QS 静态 parser 接入现有编译 API 或 runtime lowering。
- 不接入 Core IR lowering。
- 不接入 RuntimeCoordinator。
- 不接入真实 VenueAdapter。
- 不改变现有 `OrderType`、`TimeInForce`、`ExecutionPlan` 行为。

## 验证

针对性验证:

```powershell
cargo test -p qrpc-core-ir v4
cargo test -p quantscript v4_static
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
- 默认 QS 强类型系统可通过。
- QS 强类型系统缺少第一批 scalar 类型会失败。
- QS 强类型系统 composite 类型重复会失败。
- `list` / `map` 缺少或超出 `max_items` 会失败。
- 类型嵌套超过预算会失败。
- 静态总包可整体验证通过。
- 版本 manifest 缺少语义变更升 schema 要求会失败。
- pure plugin 申请 provider network 会失败。
- 复现契约缺少 risk decision evidence 会失败。
- 复杂度指标超预算会失败。
- 学习记录目录未 gitignored 会失败。
- 编译期能力报告可接受受支持的 Phase 2 请求。
- 编译期能力报告会拒绝 required 但 unsupported 的执行能力。
- 编译期能力报告会拒绝 local_simulated 模式下误用 provider_native 能力。
- 编译期能力报告会拒绝无效强类型引用。
- 编译期能力报告会拒绝缺失的 required plugin。
- v4 QS 静态审计可接受受支持的状态机脚本，且不接 runtime / lowering。
- v4 QS 静态审计会拒绝 unsupported required capability。
- v4 QS 静态审计会拒绝嵌套 machine block。
- v4 QS 静态审计会拒绝非 `on event` transition 语法。
- v4 QS 静态审计会拒绝 runtime mode 与 capability source 不匹配。
- Venue capability 重复声明会失败。
- 缺失能力不会被当作 supported。
- v4 第一批能力必须显式标记来源。
- 默认 unsupported 矩阵不会假装支持任何订单能力。
