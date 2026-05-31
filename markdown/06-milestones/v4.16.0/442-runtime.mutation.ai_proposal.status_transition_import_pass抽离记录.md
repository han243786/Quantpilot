# v4.16.0 runtime.mutation.ai_proposal.status_transition_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EZ-03
> 基线: `441-runtime.mutation.ai_proposal.status_transition_import_pass抽离方案.md`
> 目标子叶: `runtime.mutation.ai_proposal.status_transition_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.status_transition_import_pass`
> 代码动作: actual import rewrite
> 下一步: BE-001EZ-04 `runtime.mutation.ai_proposal.status_transition_import_pass` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EZ-03 `runtime.mutation.ai_proposal.status_transition_import_pass` 抽离记录 | 实际抽离 |
| 规范矩阵 | explicit import pass / single-file import rewrite / no release transition | 顶部 import 显式化 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.status_transition_import_pass` | status transition import 输入显式化 |
| 模块树 | `runtime.mutation.ai_proposal.status_transition_import_pass` | 抽离完成，待 closeout |

---

## 实际改动

本批只改写 `src/runtime/mutation/ai_proposal/status_transition.rs` 顶部 import:

```text
runtime.mutation.ai_proposal.status_transition_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.status_transition_import_pass
status_transition_import_pass extraction_done
removed use super::*
single file import rewrite
```

改动后的显式输入面:

```rust
use crate::{auth, current_time_ms, AppState, RuntimeAiProposalStatus};
```

`safe_eprintln!` 保持 crate-local macro 调用形态。

---

## 等价保持

以下内容未改:

```text
ai_proposal_approved_status body
is_valid_ai_proposal_transition body
update_ai_proposal_status body
pub(super) visibility
RuntimeAiProposalStatus::Approved mapping
Submitted -> StaticCheckPassed | StaticCheckFailed transition
StaticCheckPassed -> Approved | Denied | Expired transition
state.ai_proposals.write().await lock order
auth::scoped_key(user_id, proposal_id) call
missing record no-op behavior
safe_eprintln! invalid transition log
record.status = status assignment
record.updated_at_ms = current_time_ms() assignment
```

本批保持:

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

## residual 变化

BE-001EZ-03 前，`src/runtime` 内仍有 6 个 parent wildcard import residual；本批完成后，真实 residual 为:

```text
actual_runtime_parent_import_bridge_6_to_5
actual_mutation_import_bridge_5_to_4
actual_ai_proposal_import_bridge_5_to_4
remaining_runtime_parent_import_bridge_5
remaining_mutation_import_bridge_4
remaining_ai_proposal_import_bridge_4
```

仍待处理:

```text
src/runtime/mod.rs
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/approval_review.rs
src/runtime/mutation/ai_proposal/proposal_creation.rs
src/runtime/mutation/ai_proposal/sandbox_trigger.rs
```

---

## 排除项

本批未处理:

1. 未改函数体、可见性、状态转换表或 Approved 映射。
2. 未改 missing record 行为、invalid transition 日志、状态写入或 updated_at 写入。
3. 未处理其他 ai proposal child import residual。
4. 未处理 `src/runtime/mutation/ai_proposal.rs` parent facade。
5. 未处理 `src/runtime/mod.rs` root parent bridge。
6. 未迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
7. 未新增 sibling 横向连接。
8. 未启动 release transition。

---

## 验证要求

提交前至少执行:

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

AI 声称 BE-001EZ-03 完成时，必须说明:

1. 本批只完成 `status_transition.rs` 顶部 import 显式化。
2. `runtime.mutation.ai_proposal.status_transition_import_pass` 尚未 closeout，下一步只能进入 BE-001EZ-04 单叶 closeout。
3. 不得宣称 ai proposal import、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `442-runtime.mutation.ai_proposal.status_transition_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/mutation/ai_proposal/status_transition.rs` 不再使用 `use super::*`。
3. 函数体、可见性、状态转换表、missing record 行为、invalid transition 日志与 updated_at 写入均未改。
4. BE-001EZ-04 单叶 closeout 成为唯一下一步。
5. 治理门禁、全量树覆盖和 Rust 验证均通过。
