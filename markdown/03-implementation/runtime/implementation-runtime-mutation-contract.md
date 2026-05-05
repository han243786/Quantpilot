# 运行时变异合约

这是受控的运行时参数变异的活跃合约真实数据源。规划工作清单跟踪交付状态；本文档拥有稳定的字段合约、安全规则、证据行为和 Block 4 交接约束。

## 范围

运行时变异涵盖操作员发起的参数提案、安全窗口检查、显式边界的激活、回滚到台账支持的参数版本、受治理的证据事件、前端读取器/显示行为、合约快照和健康指标。

运行时变异本身不授予实盘执行权限。权限边界和能力上下文检查在提案、激活或回滚之前仍然是强制性的。

## 字段所有权

| 面 | 负责人 | 真实数据源 | 更新规则 |
|---|---|---|---|
| 变异请求/记录形态 | 后端 | `src/frontend_api_types.rs` | 首先以类型化结构体/枚举添加字段，然后更新快照、读取器、测试和文档。 |
| 变异台账持久化 | 后端 | `src/runtime_persistence.rs` | 台账记录是仅追加的。绝不为表达状态而删除或重写激活/回滚历史。 |
| 变异 API 行为 | 后端 | `src/runtime_api.rs` | 提案、激活、安全窗口拒绝和回滚必须在变异状态变更之前验证能力上下文。 |
| 事件信封分类 | 后端 | `src/runtime_event_projection.rs` | 每个 `ParameterMutation*` 事件是 `system` + `key`，必须通过受治理的信封验证。 |
| 时间线/报告投影 | 后端 + 前端 | `src/runtime_response_mapping.rs`、`frontend/src/utils/runtimeTimeline.js`、`frontend/src/utils/runtimeEvidenceSummary.js` | 变异生命周期证据必须在时间线、回放、紧凑证据和报告部分中保持可见。 |
| 前端读取器合约 | 前端 | `frontend/src/utils/runtimeMutation.js` | UI 仅消费规范化状态；原始 JSON 不是主要用户面。 |
| 前端显示 | 前端 | `frontend/src/components/RuntimeMutationPanel.jsx` | 显示当前、待定、已拒绝、活跃、已回滚和失败状态，并禁用不安全的操作。 |
| 合约快照 | 测试 | `tests/fixtures/runtime/mutation_contract_snapshot.json` | 仅在有意的合约更新和匹配的文档更改时允许快照漂移。 |
| 健康指标 | 后端 | `/api/runtime/evidence/health` | 指标是观察性的。它们不得改变用户可见的变异行为。 |

## 稳定的状态值

- `proposed`（已提议）
- `rejected`（已拒绝）
- `activation_scheduled`（激活已调度）
- `activated`（已激活）
- `activation_failed`（激活失败）
- `safe_window_denied`（安全窗口拒绝）
- `rollback_scheduled`（回滚已调度）
- `rolled_back`（已回滚）
- `rollback_failed`（回滚失败）

## 安全规则

- `immediate` 激活是被禁止的。支持的边界是 `next_cycle_start`、`manual_pause` 和 `sequence_cursor`。
- 安全窗口评估由后端拥有。前端显示可解释 `safe_window_state`，但不能授权激活或回滚。
- 不安全窗口返回 `parameter_mutation_safe_window_denied`，持久化拒绝状态，并发出 `ParameterMutationSafeWindowDenied` 作为关键证据。
- 回滚目标必须已存在于同一来源和目标对的变异台账中。
- 回滚创建带有 `rollback_of` 和 `rollback_target_parameter_version` 的新反向变异记录。它不得删除或重写原始激活记录。
- 历史事件信封必须保留其原始 `parameter_version`。激活或回滚到较新版本不得将有效先前的信封修复为新的活跃版本。

## 受治理的事件

所有变异事件是保留的关键证据：

- `ParameterMutationProposed`
- `ParameterMutationRejected`
- `ParameterMutationActivationScheduled`
- `ParameterMutationActivated`
- `ParameterMutationActivationFailed`
- `ParameterMutationSafeWindowDenied`
- `ParameterMutationRollbackScheduled`
- `ParameterMutationRolledBack`
- `ParameterMutationRollbackFailed`

每个事件负载必须包含提案标识、源标识、目标参数、旧/提议参数版本、参与者、原因、治理信息，以及任何可用的激活、安全窗口或回滚状态。

## 健康指标

`/api/runtime/evidence/health` 暴露变异计数器以及证据指标：

- 已创建/拒绝的提案
- 已调度/已应用/失败的激活
- 激活延迟总计和平均
- 安全窗口拒绝
- 回滚尝试/已调度/已应用/失败

这些指标仅用于运维可见性和告警。

## 更新检查清单

更改变异行为时：

- 首先更新类型化后端合约
- 更新任何新事件类型的事件信封分类
- 更新前端读取器规范化和 UI 状态标签
- 更新 API/集成测试，覆盖提案、激活、安全窗口拒绝、回滚、时间线/回放/报告证据和健康指标
- 更新 `mutation_contract_snapshot.json`
- 更新本文档和 v0.2.0 工作清单
- 验证所有更改后的 Markdown 文件可解码为 UTF-8

## Block 4 交接

Block 4 AI 批准必须建立在此合约之上。AI 可产生提案或批准建议，但不得绕过能力上下文验证、权限边界检查、安全窗口评估、显式激活边界、仅追加的台账历史或受治理的证据发出。
