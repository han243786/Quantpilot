# 运行时 AI 批准合约

这是 AI 辅助的运行时提案和批准链的活跃合约真实数据源。v0.2.0 工作清单跟踪交付状态；本文档拥有稳定的安全边界和已存在的字段。

## 范围

当前实现范围是 Block 4 P0：

- AI 提案候选接收
- 源证据标识
- 模型、提示和证据哈希
- 能力和权限边界验证
- 静态验证通过/失败
- 仅追加的 AI 提案台账
- 提案读取 API
- 受治理的 AI 提案事件
- 提案/静态检查状态的前端规范化

沙箱回放、人工批准、转换为 Block 3 变异提案以及批准链报告尚未实现。它们仍然是 P1/P2 工作。

## 安全边界

- AI 仅在能力上下文为当前且 `ai_write_policy=proposal_only` 时可以创建候选。
- AI 提案创建从不写入 Block 3 变异台账。
- AI 提案创建从不调度激活、回滚或活跃的参数版本变更。
- 静态检查失败是可审计的，但不能进入沙箱回放或批准，因为那些路由在 P0 中不可用。
- 任何未来的批准或转换路径仍必须通过运行时变异合约的安全窗口和激活边界规则。

## 字段所有权

| 面 | 负责人 | 真实数据源 | 更新规则 |
|---|---|---|---|
| AI 提案请求/记录形态 | 后端 | `src/frontend_api_types.rs` | 首先以类型化结构体/枚举添加字段，然后更新测试、前端读取器和文档。 |
| AI 提案台账持久化 | 后端 | `src/runtime_persistence.rs` | AI 提案记录与 Block 3 变异记录保持分离。 |
| AI 提案 API 行为 | 后端 | `src/runtime_api.rs` | 候选接收必须验证能力上下文、AI 写入策略、目标、参与者、模型标识、提示哈希和证据哈希。 |
| AI 提案事件分类 | 后端 | `src/runtime_event_projection.rs` | 每个 `AIProposal*` P0 事件是 `system` + `key`，必须通过受治理的信封验证。 |
| 时间线/回放投影 | 后端 | `src/runtime_response_mapping.rs` | AI 提案关键事件必须在运行详情和回放证据中保持可见。 |
| 前端读取器合约 | 前端 | `frontend/src/utils/runtimeAiProposal.js` | UI 消费规范化的提案/静态检查状态和禁用原因。 |
| 合约测试 | 后端 + 前端 | `tests/api_ai_proposal.rs`、`frontend/src/utils/runtimeAiProposal.test.js` | 测试必须覆盖允许的接收、拒绝、静态检查失败、事件信封、回放可见性和读取器回退行为。 |

## 稳定的状态值

- `draft`（草稿）
- `submitted`（已提交）
- `static_check_failed`（静态检查失败）
- `static_check_passed`（静态检查通过）
- `denied`（已拒绝）
- `expired`（已过期）

P0 创建请求返回 `static_check_passed` 或 `static_check_failed`。`draft`、`submitted`、`denied` 和 `expired` 是为生命周期和未来批准工作保留的合约状态。

## 受治理的事件

P0 认可以下事件为保留的关键证据：

- `AIProposalCreated`
- `AIProposalDenied`
- `AIProposalStaticCheckPassed`
- `AIProposalStaticCheckFailed`

每个事件负载必须包含 AI 提案标识、源标识、源证据、目标参数、旧/提议参数版本、模型标识、提示哈希、证据哈希、参与者、原因、静态检查状态和治理信息。

## 更新检查清单

更改 AI 提案或批准行为时：

- 首先更新类型化后端合约
- 更新任何新事件类型的事件信封分类
- 保持 AI 提案台账与变异台账分离，除非经批准的转换显式创建 Block 3 变异提案
- 更新前端读取器规范化和禁用/可操作状态
- 更新 API 和前端测试，覆盖拒绝、静态检查失败、时间线和回放可见性
- 更新本文档和 v0.2.0 工作清单
- 验证更改后的 Markdown 文件可解码为 UTF-8
