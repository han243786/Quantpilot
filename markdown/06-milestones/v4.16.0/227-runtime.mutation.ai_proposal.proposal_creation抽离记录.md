# v4.16.0 runtime.mutation.ai_proposal.proposal_creation 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001BP-03
> 基线: `225-runtime.mutation.ai_proposal.proposal_creation单子叶等价基线.md`、`226-runtime.mutation.ai_proposal.proposal_creation抽离方案.md`、`src/runtime/mutation/ai_proposal.rs`
> 判定: `runtime.mutation.ai_proposal.proposal_creation` 第一轮实际抽离完成。`create_runtime_ai_proposal` 已迁入 `src/runtime/mutation/ai_proposal/proposal_creation.rs`；父级只保留 path-attributed child、handler re-export 和受控 helper 连接。下一步只能进入 BE-001BP-04 单叶 closeout。
> 代码动作: code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BP-03 AI proposal proposal_creation 实际抽离 | 物理抽离 |
| 规范矩阵 | 父子通信、public handler re-export、锁顺序、发布过渡保护 | 约束执行 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.proposal_creation` | child 文件落地 |
| 模块树 | `runtime.mutation.ai_proposal.proposal_creation` | 白箱抽离完成，待 closeout |

---

## 文件变更

新增:

```text
src/runtime/mutation/ai_proposal/proposal_creation.rs
```

父级 `src/runtime/mutation/ai_proposal.rs` 新增:

```rust
#[path = "ai_proposal/proposal_creation.rs"]
mod proposal_creation;

pub(crate) use proposal_creation::create_runtime_ai_proposal;
```

child 固定:

```rust
use super::*;
```

父级删除了原内联的 `create_runtime_ai_proposal` 实现。`src/runtime/mod.rs`、`src/backend/runtime/routes/mutation.rs`、AppState、schema owner、frontend caller、runtime persistence owner 和 release transition guard 均未改变。

---

## 实际迁移清单

已从父级迁入 child:

- `create_runtime_ai_proposal`
- `CreateRuntimeAiProposalRequest` 使用点
- `RuntimeAiProposalRecord` 构造块
- `RuntimeAiProposalSourceEvidence` 构造块
- `RuntimeApprovalRecord` 构造块
- `RuntimeApprovalLifecycleEntry` 构造块
- `APPROVAL_COUNTER`
- `StaticCheckPassed` / `StaticCheckFailed` 分支
- `state.ai_proposals` 由 `persist_runtime_ai_proposal_transition` 维持原写入语义
- `auth::scoped_key` 继续用于 scoped approval/proposal key
- `approval_records -> ai_proposals` 锁顺序注释和实现
- `persist_approval` 调用
- `persist_runtime_ai_proposal_transition` 调用
- `spawn_ai_proposal_sandbox_verification` 调用

未迁移 `record_query`、`approval_review`、`approval_persistence`、`status_transition` 或 `sandbox_trigger` 的 owner；这些 sibling 仍只能由父级连接。

---

## 受控 helper 连接

`proposal_creation` 通过父级 `use super::*` 复用以下受控 helper，未新增 sibling 横向 import:

| helper / 类型 | 当前 owner | 等价要求 |
| --- | --- | --- |
| `validate_runtime_capability_guard` | shared runtime validation / parent import | 保持 proposal_only capability gate |
| `validate_runtime_parameter_mutation_target` | shared validation / parent import | 保持 mutation target guard |
| `validate_ai_model_identity` | `static_check` via parent | 不横向 import `static_check` |
| `validate_hash_identity` | `static_check` via parent | 不横向 import `static_check` |
| `load_runtime_ai_proposal_source_context` | `source_governance_identity` via parent | 保持 source lookup owner |
| `ai_proposal_static_check_result` | `static_check` via parent | 保持 StaticCheckPassed / StaticCheckFailed 语义 |
| `runtime_ai_proposal_record_id` | `source_governance_identity` via parent | 保持 proposal id digest contract |
| `runtime_ai_proposal_governance` | `source_governance_identity` via parent | 保持 governance projection |
| `build_runtime_ai_proposal_event` | `event_lifecycle` via parent | 保持 runtime event contract |
| `ai_proposal_lifecycle_entry` | `event_lifecycle` via parent | 保持 lifecycle sequence |
| `append_parameter_mutation_events_to_run` | existing run evidence bridge | 不迁移 run evidence owner |
| `persist_approval` | `approval_persistence` via parent | 保持 approval disk write |
| `persist_runtime_ai_proposal_transition` | `event_lifecycle` via parent | 保持 proposal transition persistence |
| `spawn_ai_proposal_sandbox_verification` | `sandbox_trigger` via parent | 保持 sandbox side effect |

---

## 行为等价说明

create flow 的输入输出保持不变:

1. `auth::UserId`、`State<AppState>`、`Json<CreateRuntimeAiProposalRequest>` 仍进入同一 handler。
2. capability guard、`proposal_only` policy、mutation target、old/new value、model identity、`prompt_hash` 与 `evidence_hash` 校验顺序保持不变。
3. source context、canonical parameter version、static check、record id、governance、source evidence 和 record construction 保持不变。
4. `RuntimeEvidenceSourceKind::Run` 时仍调用 `append_parameter_mutation_events_to_run` 写入 submitted/static check events。
5. static check passed 时仍先创建 `RuntimeApprovalRecord`，再按 `approval_records -> ai_proposals` 锁顺序执行 `persist_approval`、`state.approval_records` 写入、`persist_runtime_ai_proposal_transition` 与 `spawn_ai_proposal_sandbox_verification`。
6. static check failed 时仍只持久化 proposal transition。

---

## 父子通信规则

当前通信形态固定为:

```text
src/runtime/mod.rs
  -> runtime.mutation.ai_proposal public handlers
src/runtime/mutation/ai_proposal.rs
  -> proposal_creation::create_runtime_ai_proposal
src/runtime/mutation/ai_proposal/proposal_creation.rs
  -> parent-owned imports / helpers via use super::*
```

禁止:

- `proposal_creation` 横向 import `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query`、`approval_review`、`approval_persistence`、`sandbox_trigger` 或 `status_transition` sibling。
- sibling 回头 import `proposal_creation`。
- route facade 直接 import child。
- `src/runtime/mod.rs` 直接 import child。
- 迁移 AppState、schema owner、frontend caller、runtime persistence owner 或 route facade。
- 在 release transition guard 之外提出横向连接或性能旁路。

---

## 验证记录

本实际抽离批次必须运行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只能进入:

```text
BE-001BP-04 runtime.mutation.ai_proposal.proposal_creation 单叶 closeout
```

该 closeout 只能判断本叶是否停止细分，不得继续拆 approval record construction、lifecycle append、sandbox trigger call、transition persistence、AppState/schema/frontend caller、route facade、runtime persistence owner 或发布过渡连接。

---

## 幻觉检查点

AI 声称 BE-001BP-03 完成时，必须说明当前只是 `runtime.mutation.ai_proposal.proposal_creation` 第一轮实际抽离，尚未完成单叶 closeout。不得宣称 AppState/schema/frontend caller、route facade、runtime persistence owner 或 release transition guard 已改变，也不得宣称 Rust backend 重构已完成。

---

## 验收标准

1. `src/runtime/mutation/ai_proposal/proposal_creation.rs` 存在，并承接 `create_runtime_ai_proposal`。
2. 父级 `src/runtime/mutation/ai_proposal.rs` 只通过 `#[path = "ai_proposal/proposal_creation.rs"] mod proposal_creation;` 与 `pub(crate) use proposal_creation::create_runtime_ai_proposal;` 保持 route-facing 调用面。
3. child 固定 `use super::*`，未出现 sibling 横向 import。
4. `227-runtime.mutation.ai_proposal.proposal_creation抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
5. 下一步固定为 BE-001BP-04 单叶 closeout。
