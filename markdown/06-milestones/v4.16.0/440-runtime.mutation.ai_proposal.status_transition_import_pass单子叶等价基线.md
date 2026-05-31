# v4.16.0 runtime.mutation.ai_proposal.status_transition_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EZ-01
> 父叶判定: `439-runtime.mutation.ai_proposal_import_pass第七轮父叶残余判断.md`
> 目标子叶: `runtime.mutation.ai_proposal.status_transition_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.status_transition_import_pass`
> 代码动作: no code movement
> 下一步: BE-001EZ-02 `runtime.mutation.ai_proposal.status_transition_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EZ-01 `runtime.mutation.ai_proposal.status_transition_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | explicit import pass / status transition freeze / no release transition | 冻结状态转换输入面 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.status_transition_import_pass` | status transition 白箱 |
| 模块树 | `runtime.mutation.ai_proposal.status_transition_import_pass` | 新基线 |

---

## 基线结论

本批只冻结 `src/runtime/mutation/ai_proposal/status_transition.rs` 的当前等价边界，不改 Rust 代码。

```text
runtime.mutation.ai_proposal.status_transition_import_pass baseline_frozen
runtime.mutation.ai_proposal.status_transition_import_pass no_code_movement
src/runtime/mutation/ai_proposal/status_transition.rs
current_parent_import_bridge: use super::*
next_step: BE-001EZ-02 extraction plan
```

`status_transition.rs` 是 ai proposal approval path 的状态转换白箱，负责 Approved 状态映射与合法状态转换写入。它不是 route facade，也不是 persistence owner。

---

## 白箱节点

| 项 | 当前边界 |
| --- | --- |
| 输入 | `AppState`、`auth::UserId`、proposal id、目标 `RuntimeAiProposalStatus` |
| 输出 | `RuntimeAiProposalStatus`、in-memory proposal status / updated_at side effect |
| 处理者 | `ai_proposal_approved_status`、`is_valid_ai_proposal_transition`、`update_ai_proposal_status` |
| 调用方 | `approval_review.rs`、ai proposal parent facade |
| 禁止事项 | 不改状态转换表、不改 Approved 映射、不改锁顺序、不改 invalid transition 日志、不新增 sibling 横向连接 |

---

## 当前 public / 可见入口

本子叶对父模块暴露 2 个 `pub(super)` helper:

```text
ai_proposal_approved_status
update_ai_proposal_status
```

文件内私有 helper:

```text
is_valid_ai_proposal_transition
```

---

## 当前隐式输入面

当前文件顶部仍为:

```rust
use super::*;
```

BE-001EZ-03 预期只把该 parent wildcard import 收敛为显式输入面。预期输入面包括:

```rust
use crate::{auth, current_time_ms, AppState, RuntimeAiProposalStatus};
```

`safe_eprintln!` 是 crate-local macro 调用，保持原调用形式，不在本子叶改写为函数或迁移 owner。该预期仅作为输入面基线，真正代码改写必须等 BE-001EZ-03。

---

## 等价边界

### Approved status

必须保持:

```text
ai_proposal_approved_status -> RuntimeAiProposalStatus::Approved
```

不得回退为 `StaticCheckPassed` 或新增其他状态映射。

### Transition rule

必须保持 `is_valid_ai_proposal_transition` 当前转换表:

```text
Submitted -> StaticCheckPassed | StaticCheckFailed
StaticCheckPassed -> Approved | Denied | Expired
```

不得放宽 Draft、Denied、Expired、Approved 等状态的后续转换。

### Status write side effect

必须保持 `update_ai_proposal_status` 的副作用顺序:

```text
state.ai_proposals.write().await
auth::scoped_key(user_id, proposal_id)
if missing record -> no-op
if invalid transition -> safe_eprintln! and return
record.status = status
record.updated_at_ms = current_time_ms()
```

不得改锁顺序、missing record 行为、invalid transition 行为、日志内容或 updated_at 赋值时机。

---

## 不变量

```text
no_status_transition_rule_rewrite
no_approved_status_rewrite
no_state_lock_order_rewrite
no_missing_record_behavior_rewrite
no_updated_at_rewrite
no_invalid_transition_log_rewrite
no_visibility_rewrite
no_sibling_owner_migration
old_three_leaf_pause_target_cancelled
```

---

## 排除项

本批不处理:

1. 不改 Rust 代码。
2. 不删除 `use super::*`。
3. 不改函数体、可见性、状态转换表、日志或时间戳写入。
4. 不处理其他 ai proposal child import residual。
5. 不处理 `src/runtime/mutation/ai_proposal.rs` parent facade。
6. 不处理 `src/runtime/mod.rs` root parent bridge。
7. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
8. 不新增 sibling 横向连接。
9. 不启动 release transition。

---

## 验证要求

本批为 `no code movement` 基线，提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_ai_proposal
```

---

## 幻觉检查点

AI 声称 BE-001EZ-01 完成时，必须说明:

1. 本批只是 `no code movement` 单子叶等价基线。
2. `status_transition.rs` 仍未实际删除 `use super::*`。
3. 下一步只能进入 BE-001EZ-02 `runtime.mutation.ai_proposal.status_transition_import_pass` 抽离方案。
4. 不得宣称 status_transition import、ai proposal import、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `440-runtime.mutation.ai_proposal.status_transition_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `status_transition.rs` 白箱输入、输出、处理者、调用方和禁止事项已冻结。
3. 下一步固定为 BE-001EZ-02 抽离方案。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
