# 运行时治理合约

此文件是运行时治理标识的活跃真实数据源合约。在更改能力合约、事件信封、运行时/回测治理快照、部署修订版或权限边界执行时使用。

v0.2.0 工作清单跟踪实现进度，但本文档在 Block 1 后拥有字段合约。

## 合约所有者

所有权按角色分配，而非按个人姓名。

| 合约领域 | 真实数据源 | 负责人角色 | 变更时必需更新 |
|---|---|---|---|
| 能力合约 | `/api/capabilities` 响应和后端能力合约构建器 | 后端能力负责人 | 后端 fixture、前端默认能力、支持矩阵测试、文档索引 |
| 事件信封 | 运行、回放、SSE、回测和工件事件上的 `FrontendRuntimeEvent.envelope` | 后端运行时负责人 | 信封验证器测试、运行/回测集成测试、回放/SSE 测试 |
| 运行时治理快照 | 运行/回测详情和回测清单上的 `governance` | 后端运行时持久化负责人 | 已保存/重载工件测试、旧版默认测试、前端读取器测试 |
| 部署修订版 | 从策略版本、编译 ID、参数版本和能力哈希推导的确定性摘要 | 后端编译/运行时负责人 | 规范哈希测试和已保存工件重载测试 |
| 权限边界 | 能力 `permission_boundary` 加运行时写入守卫 | 后端运行时负责人和前端能力门禁负责人 | 后端守卫测试、前端故障关闭操作测试、UI 措辞检查 |
| 治理诊断显示 | 诊断/详情面板中的规范化治理行 | 前端运行时负责人 | 诊断/详情 UI 测试和治理读取器测试 |

## 能力合约

后端能力合约是权威的。前端默认值、fixture 和支持矩阵规则必须遵循它。

必需字段：

- `api_version`
- `schema_version`
- `schema_hash`
- `chain_stages`
- `versioning`
- `permission_boundary`

规则：

- `schema_hash` 必须是 `sha256:<摘要>`。
- 摘要必须使用规范 JSON 并排除请求时间或仅显示噪音。
- 安全回退能力不是受信任的运行时能力边界，必须比正常能力模式更严格。
- 如果前端和后端能力事实不一致，更新前端能力规范化和测试以匹配后端真实数据。

实现参考：

- [src/capability_api.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/capability_api.rs)
- [frontend/src/capabilities/supportMatrix.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/capabilities/supportMatrix.js)
- [frontend/src/test/fixtures/capabilities/backend-capabilities-v1.json](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/test/fixtures/capabilities/backend-capabilities-v1.json)

## 运行时治理快照

每个新的运行和回测必须暴露 `governance` 快照。已保存的回测清单必须保留相同的治理标识。

必需字段：

- `schema_version`
- `governance_source`
- `capability_hash`
- `strategy_version`
- `parameter_version`
- `deployment_revision`
- `permission_boundary`

当前运行时所允许的来源值：

- `current_runtime`：由新的内存运行时/回测记录产生。
- `loaded_manifest`：从已保存或瞬态工件清单物化。
- `legacy_default`：从不包含治理数据的旧记录回填。

规则：

- 缺失的旧治理必须以安全默认值加载，不能使读取路径失败。
- 默认化的旧治理必须通过 `governance_source` 显式可见。
- 新的运行时写入路径不得依赖旧版/默认化治理来启动新运行。
- 前端代码必须通过 `frontend/src/utils/runtimeGovernance.js` 读取治理，而非在每个组件中进行临时的嵌套属性读取。

实现参考：

- [src/runtime_response_mapping.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/runtime_response_mapping.rs)
- [src/runtime_persistence.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/runtime_persistence.rs)
- [frontend/src/utils/runtimeGovernance.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/utils/runtimeGovernance.js)

## 事件信封

每个暴露给前端的运行时/回测事件必须携带信封。这适用于实时 SSE 事件、已保存的运行详情、已保存的回测详情、回测工件事件日志和回放响应。

必需字段：

- `event_id`
- `event_type`
- `stage`
- `run_id`
- `sequence_no`
- `occurred_at_ms`
- `ingested_at_ms`
- `trace_id`
- `module_key`
- `strategy_version`
- `parameter_version`
- `deployment_revision`
- `capability_hash`
- `mode`
- `severity`
- `retention_class`
- `reason_code`
- `payload_version`

规则：

- 在一个暴露的事件列表内，`sequence_no` 必须是连续的。
- `stage` 和 `retention_class` 是序列化为合约字符串的类型化后端枚举。
- 信封 `capability_hash` 和 `deployment_revision` 必须匹配封闭的运行/回测治理快照。
- 旧版事件可在暴露前在加载时修复，但暴露的事件必须满足验证器。
- `CapabilitySnapshotTaken` 和 `SecurityViolationDetected` 是 `retention_class=key` 的系统事件。

实现参考：

- [src/runtime_event_projection.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/runtime_event_projection.rs)
- [frontend/src/utils/runtimeDiagnosticsProjection.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/utils/runtimeDiagnosticsProjection.js)

## 权限边界

权限边界定义运行时安全行为和前端操作门禁。

必需字段：

- `model_version`
- `execution_owner_module`
- `live_execution_allowed`
- `ai_write_policy`
- `plugin_network_default`
- `non_execution_order_access`

当前安全默认值：

- `live_execution_allowed=false`
- `ai_write_policy=disabled`
- `plugin_network_default=deny`
- `non_execution_order_access=deny`

规则：

- 缺失或格式错误的运行时能力上下文必须在创建运行/回测记录之前拒绝运行时写入请求。
- 缺失的前端权限边界必须阻止编译/运行/回测/扫掠操作。
- 未知的前端权限值必须规范化为限制性默认值。
- 面向用户的 UI 和文档不得声称实盘执行或 AI 写入支持，除非能力权限策略允许。

实现参考：

- [src/runtime_validation.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/runtime_validation.rs)
- [frontend/src/capabilities/supportMatrix.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/capabilities/supportMatrix.js)
- [frontend/src/store/graphStoreRuntimeSessionActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeSessionActions.js)

## 诊断面

运行时诊断和回测详情视图必须暴露足够的治理标识，以便用户回答是哪个能力边界、策略版本、参数版本、部署修订版和权限模型产生了事件/结果。

规则：

- 长哈希可在视觉上缩短。
- 完整值必须通过工具提示或复制元数据保持可用。
- UI 必须显示规范化的治理行，而非原始 JSON。
- 当时间线块开始时，诊断应使用 `event.envelope.stage` 和 `event.envelope.retention_class` 作为时间线分组输入。

实现参考：

- [frontend/src/components/RuntimeDiagnosticsPanel.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/components/RuntimeDiagnosticsPanel.jsx)
- [frontend/src/pages/BacktestDetailPage.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/BacktestDetailPage.jsx)

## 变更检查清单

对此合约的任何更改必须在同一批次中更新所有受影响的层：

- 后端能力或运行时构建器
- 持久化的运行/回测工件形态
- 前端规范化器和能力门禁
- 当用户可见的治理事实更改时的诊断/详情显示
- 针对当前、已保存、流式、回放和旧版/默认化记录的测试
- 本文档和文档索引

在工作清单项目完成之前，活跃合约文档和实现进度必须一致。
