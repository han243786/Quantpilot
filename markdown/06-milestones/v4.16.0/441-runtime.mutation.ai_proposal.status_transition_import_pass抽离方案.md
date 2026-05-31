# v4.16.0 runtime.mutation.ai_proposal.status_transition_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EZ-02
> 基线: `440-runtime.mutation.ai_proposal.status_transition_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.mutation.ai_proposal.status_transition_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.status_transition_import_pass`
> 代码动作: no code movement
> 下一步: BE-001EZ-03 `runtime.mutation.ai_proposal.status_transition_import_pass` 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EZ-02 `runtime.mutation.ai_proposal.status_transition_import_pass` 抽离方案 | 方案优化 |
| 规范矩阵 | single-file import rewrite / status transition freeze / no release transition | 固定抽离边界 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.status_transition_import_pass` | status transition import pocket |
| 模块树 | `runtime.mutation.ai_proposal.status_transition_import_pass` | 抽离方案 |

---

## 方案结论

BE-001EZ-03 只能改写 `src/runtime/mutation/ai_proposal/status_transition.rs` 顶部 import，禁止改函数体、可见性、状态转换表、missing record 行为、invalid transition 日志、updated_at 写入或 sibling owner。

```text
runtime.mutation.ai_proposal.status_transition_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.status_transition_import_pass
status_transition_import_pass plan_frozen
single_file_status_transition_import_rewrite
next_step: BE-001EZ-03 extraction record
```

---

## 允许改动

唯一允许的 Rust 改动:

```diff
-use super::*;
+use crate::{auth, current_time_ms, AppState, RuntimeAiProposalStatus};
```

允许 cargo fmt 对 import 分组做机械格式化。`safe_eprintln!` 保持 crate-local macro 调用形态，不在本批迁移。

---

## 不允许改动

BE-001EZ-03 不得修改:

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

---

## 等价守卫

必须保持:

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

## 残余预期

BE-001EZ-03 完成后，预期 residual 从:

```text
remaining_runtime_parent_import_bridge_6
remaining_mutation_import_bridge_5
remaining_ai_proposal_import_bridge_5
```

降为:

```text
remaining_runtime_parent_import_bridge_5
remaining_mutation_import_bridge_4
remaining_ai_proposal_import_bridge_4
```

`src/runtime/mod.rs` 与 `src/runtime/mutation/ai_proposal.rs` 仍不在 BE-001EZ-03 范围内。

---

## 排除项

本批不处理:

1. 不改 Rust 代码。
2. 不执行 BE-001EZ-03 实际 import rewrite。
3. 不处理其他 ai proposal child import residual。
4. 不处理 parent facade 或 root parent bridge。
5. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
6. 不新增 sibling 横向连接。
7. 不启动 release transition。

---

## 验证要求

本批为 `no code movement` 抽离方案，提交前至少执行:

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

AI 声称 BE-001EZ-02 完成时，必须说明:

1. 本批只是 `no code movement` 抽离方案。
2. BE-001EZ-03 只能改写 `src/runtime/mutation/ai_proposal/status_transition.rs` 顶部 import。
3. 不得宣称 `status_transition.rs` 已完成实际抽离。
4. 不得宣称 ai proposal import、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `441-runtime.mutation.ai_proposal.status_transition_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001EZ-03 的允许改动被限制为单文件 import rewrite。
3. BE-001EZ-03 成为唯一下一步。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
