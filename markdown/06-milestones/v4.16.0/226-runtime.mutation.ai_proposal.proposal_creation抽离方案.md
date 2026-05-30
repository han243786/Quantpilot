# v4.16.0 runtime.mutation.ai_proposal.proposal_creation 抽离方案

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BP-02  
> 基线: `225-runtime.mutation.ai_proposal.proposal_creation单子叶等价基线.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime/mutation/ai_proposal/static_check.rs`、`src/runtime/mutation/ai_proposal/source_governance_identity.rs`、`src/runtime/mutation/ai_proposal/event_lifecycle.rs`、`src/runtime/mutation/ai_proposal/approval_persistence.rs`、`src/runtime/mutation/ai_proposal/sandbox_trigger.rs`、`src/runtime/mutation/ai_proposal/status_transition.rs`  
> 判定: `runtime.mutation.ai_proposal.proposal_creation` 抽离方案已建立。当前 `no code movement`，只固定 BE-001BP-03 的目标文件、父级 child 声明、handler re-export、`use super::*`、迁移清单、非目标和回退点。下一步只能进入 BE-001BP-03 实际抽离。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BP-02 AI proposal proposal_creation 抽离方案 | 方案冻结 |
| 规范矩阵 | 父子通信、public handler re-export、closed child 不回改、发布过渡保护 | 约束固化 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.proposal_creation` | 实际抽离路径 |
| 模块树 | `runtime.mutation.ai_proposal.proposal_creation` | 待物理抽离 |

---

## 目标文件

BE-001BP-03 只允许创建:

```text
src/runtime/mutation/ai_proposal/proposal_creation.rs
```

父级只允许新增:

```rust
#[path = "ai_proposal/proposal_creation.rs"]
mod proposal_creation;
pub(crate) use proposal_creation::create_runtime_ai_proposal;
```

child 固定:

```rust
use super::*;
```

父级不得直接 import proposal_creation sibling 内部 helper；route facade、`src/runtime/mod.rs` 和 `src/backend/runtime/routes/mutation.rs` 调用面保持不变。

---

## 允许迁移清单

BE-001BP-03 只允许迁移:

- `create_runtime_ai_proposal`

允许随 handler 一起移动且不改变语义的局部内容:

- `CreateRuntimeAiProposalRequest` 使用点。
- `RuntimeAiProposalRecord` 构造块。
- `RuntimeAiProposalSourceEvidence` 构造块。
- `RuntimeApprovalRecord` 构造块。
- `RuntimeApprovalLifecycleEntry` 构造块。
- `APPROVAL_COUNTER` static。
- StaticCheckPassed / StaticCheckFailed 分支。
- `approval_records -> ai_proposals` 锁顺序注释和实现。
- `persist_approval` 调用。
- `persist_runtime_ai_proposal_transition` 调用。
- `spawn_ai_proposal_sandbox_verification` 调用。

不允许把上述内容继续拆成新 helper；本批次只做最小物理抽离。

---

## 必须保留的父级受控 helper 连接

抽离后 child 只能经 `use super::*` 复用父级已受控连接的 helper:

| helper / 类型 | 当前 owner | BE-001BP-03 要求 |
| --- | --- | --- |
| `validate_runtime_capability_guard` | parent imports / shared runtime validation | 保持原调用，不迁移 owner |
| `validate_runtime_parameter_mutation_target` | parent imports / shared validation | 保持原调用，不迁移 owner |
| `validate_ai_model_identity` | `static_check` child via parent | 不横向 import `static_check` |
| `validate_hash_identity` | `static_check` child via parent | 不横向 import `static_check` |
| `load_runtime_ai_proposal_source_context` | `source_governance_identity` child via parent | 不横向 import sibling |
| `canonical_runtime_parameter_version` | existing shared helper | 不迁移 owner |
| `ai_proposal_static_check_result` | `static_check` child via parent | 不改变 StaticCheckPassed / StaticCheckFailed |
| `runtime_ai_proposal_record_id` | `source_governance_identity` child via parent | 不改变 digest contract |
| `runtime_ai_proposal_governance` | `source_governance_identity` child via parent | 不改变 governance evidence |
| `build_runtime_ai_proposal_event` | `event_lifecycle` child via parent | 不改变 event contract |
| `ai_proposal_lifecycle_entry` | `event_lifecycle` child via parent | 不改变 lifecycle sequence |
| `append_parameter_mutation_events_to_run` | existing run evidence bridge | 不迁移 run evidence owner |
| `persist_approval` | `approval_persistence` child via parent | 不横向 import sibling |
| `persist_runtime_ai_proposal_transition` | `event_lifecycle` child via parent | 不改变 transition persistence |
| `spawn_ai_proposal_sandbox_verification` | `sandbox_trigger` child via parent | 不改变 sandbox retry/failure side effect |

---

## 父子通信规则

BE-001BP-03 完成后，通信形态必须为:

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
- `static_check`、`source_governance_identity`、`event_lifecycle`、`approval_persistence`、`sandbox_trigger` 或 `status_transition` 回头 import `proposal_creation`。
- route facade 直接 import child。
- `src/runtime/mod.rs` 直接 import child。
- 任何 AppState/schema/frontend caller/runtime persistence owner 迁移。
- 启动 release transition guard 之外的发布过渡动作。

---

## 非目标

BE-001BP-02 不移动代码，不创建目标文件。BE-001BP-03 也不得:

- 迁移 `list_runtime_ai_proposals`、`get_runtime_ai_proposal_detail` 或 `load_runtime_ai_proposal_for_user`。
- 迁移 `approve_ai_proposal`、`reject_ai_proposal`、`claim_ai_proposal_review`、`list_runtime_approvals` 或 `get_runtime_approval_detail`。
- 迁移 `persist_approval` 或 `load_approval_from_disk`。
- 迁移 `ai_proposal_approved_status`、`is_valid_ai_proposal_transition` 或 `update_ai_proposal_status`。
- 迁移 `load_sandbox_report_for_proposal`、`ensure_ai_proposal_can_be_approved` 或 `spawn_ai_proposal_sandbox_verification`。
- 拆分 approval record construction、lifecycle event append、sandbox trigger call 或 transition persistence。
- 改变 `state.approval_records`、`state.ai_proposals` 或 `auth::scoped_key` 语义。
- 改变 `AppState`、schema owner、frontend caller、route facade 或 runtime persistence owner。
- 启动发布过渡或提出横向连接。

---

## 回退点

如果 BE-001BP-03 实际抽离后出现编译或测试失败，回退只允许按反向最小改动:

1. 删除父级 `#[path = "ai_proposal/proposal_creation.rs"] mod proposal_creation;`。
2. 删除父级 `pub(crate) use proposal_creation::create_runtime_ai_proposal;`。
3. 将 `create_runtime_ai_proposal` 原样移回 `src/runtime/mutation/ai_proposal.rs`。
4. 删除新建 child 文件。
5. 不回改任何已 closeout child。

---

## 回归保护

本方案批次只跑治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

BE-001BP-03 实际抽离必须补跑:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
```

---

## 下一步

下一步只能进入:

```text
BE-001BP-03 runtime.mutation.ai_proposal.proposal_creation 实际抽离
```

该步骤只能创建计划 child 并迁移 `create_runtime_ai_proposal`，不得混入整理、重构、进一步细拆或发布过渡。

---

## 幻觉检查点

AI 声称 BE-001BP-02 完成时，必须说明当前只是 `runtime.mutation.ai_proposal.proposal_creation` 抽离方案，仍为 `no code movement`；目标文件尚未创建，`create_runtime_ai_proposal` 尚未迁移。不得宣称 proposal_creation 已抽离、已 closeout、AppState/schema/frontend caller 已改变、route facade 已改变、runtime persistence owner 已迁移、release transition 已启动或 Rust backend 重构已完成。

---

## 验收标准

1. `226-runtime.mutation.ai_proposal.proposal_creation抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 目标文件、父级 child 声明、handler re-export 与 `use super::*` 已固定。
3. 迁移清单只包含 `create_runtime_ai_proposal` 及其局部构造块。
4. 非目标、回退点和验证门禁已冻结。
5. 本批次无代码移动。
