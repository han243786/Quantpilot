# v4.16.0 runtime.mutation.ai_proposal.approval_review_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FD-01
> 上一批: `449-runtime.mutation.ai_proposal_import_pass第九轮父叶残余判断.md`
> 目标子叶: `runtime.mutation.ai_proposal.approval_review_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.approval_review_import_pass`
> 代码动作: no code movement
> 下一步: BE-001FD-02 `runtime.mutation.ai_proposal.approval_review_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FD-01 `runtime.mutation.ai_proposal.approval_review_import_pass` 单子叶等价基线 | 建立基线 |
| 规范矩阵 | approval review handler / explicit import pass / no release transition | 冻结等价边界 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.approval_review_import_pass` | approval review import pocket |
| 模块树 | `runtime.mutation.ai_proposal.approval_review_import_pass` | 单子叶基线 |

---

## 当前白箱边界

`src/runtime/mutation/ai_proposal/approval_review.rs` 当前仍通过 `use super::*` 取得父级输入面。本批只冻结真实行为与预期输入面，不移动 Rust 代码。

```text
runtime.mutation.ai_proposal.approval_review_import_pass baseline_frozen
runtime.mutation.ai_proposal.approval_review_import_pass current_parent_import_bridge: use super::*
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.approval_review_import_pass
old_three_leaf_pause_target_cancelled
progress_report_instruction_discarded
```

---

## 冻结的 public / parent-visible 方法

```text
list_runtime_approvals
get_runtime_approval_detail
approve_ai_proposal
reject_ai_proposal
claim_ai_proposal_review
```

这些 handler 属于 route-facing approval review 边界；本轮只处理 import 输入面冻结，不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或测试资产。

---

## 冻结的输入面

BE-001FD-02/03 预期只能把 parent wildcard import 收敛为以下显式输入面，不得引入新 owner 或 sibling 横向连接:

```text
super::approval_persistence::{load_approval_from_disk, persist_approval}
super::record_query::load_runtime_ai_proposal_for_user
super::sandbox_trigger::ensure_ai_proposal_can_be_approved
super::status_transition::{ai_proposal_approved_status, update_ai_proposal_status}
crate::auth
crate::current_time_ms
crate::io_error
crate::json_bad_request
crate::AppState
crate::ApprovalActionRequest
crate::RuntimeAiProposalStatus
crate::RuntimeApprovalLifecycleEntry
crate::RuntimeApprovalListQuery
crate::RuntimeApprovalRecord
crate::RuntimeApprovalReviewState
axum::extract::{Path, Query, State}
axum::http::StatusCode
axum::Json
State / Query / Path / Json
```

---

## 等价冻结点

BE-001FD-02/03 必须保持以下行为不变:

```text
list_runtime_approvals scoped prefix lookup
review_state optional case-insensitive filter
created_at_ms descending sort
get_runtime_approval_detail memory-first lookup
load_approval_from_disk fallback
approve_ai_proposal loads proposal before approval write lock
ensure_ai_proposal_can_be_approved gate remains before approval write lock
approval write lock existing shape
Pending | UnderReview approval states for approve/reject
Pending only for claim
reviewers_approved no duplicate and no rejected actor
reviewers_assigned no duplicate
reviewers_required threshold
APPROVAL_APPROVED
APPROVAL_PARTIAL
APPROVAL_REJECTED
APPROVAL_CLAIMED
RuntimeAiProposalStatus::Denied
ai_proposal_approved_status
persist_approval before scoped insert
auth::scoped_key
```

---

## 等价守卫

```text
no_approval_filter_rewrite
no_approval_lock_order_rewrite
no_reviewer_count_rewrite
no_lifecycle_event_rewrite
no_status_transition_rewrite
no_persistence_order_rewrite
no_error_payload_rewrite
no_visibility_rewrite
no_sibling_owner_migration
```

---

## 预期 residual

BE-001FD-01 完成后 residual 不变化:

```text
remaining_runtime_parent_import_bridge_4
remaining_mutation_import_bridge_3
remaining_ai_proposal_import_bridge_3
```

BE-001FD-03 实际抽离后，才允许下降为:

```text
expected_runtime_parent_import_bridge_3
expected_mutation_import_bridge_2
expected_ai_proposal_import_bridge_2
```

---

## 排除项

本批不处理:

1. 不改 Rust 代码。
2. 不改 `approval_review.rs` 顶部 import；这属于 BE-001FD-03。
3. 不改 approval list/detail/approve/reject/claim 的函数体、锁顺序、状态机、lifecycle、错误 payload 或持久化顺序。
4. 不处理 `proposal_creation.rs` 或 `ai_proposal.rs` parent facade import residual。
5. 不处理 `src/runtime/mod.rs` root parent bridge。
6. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
7. 不新增 sibling 横向连接。
8. 不启动 release transition。

---

## 验证要求

本批为 `no code movement` 等价基线，提交前至少执行:

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

AI 声称 BE-001FD-01 完成时，必须说明:

1. 本批只是 `no code movement` 等价基线。
2. `approval_review.rs` 仍未实际移除 `use super::*`。
3. 下一步只能进入 BE-001FD-02 抽离方案。
4. 不得宣称 approval_review、proposal_creation、ai proposal parent facade、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `450-runtime.mutation.ai_proposal.approval_review_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. list/detail/approve/reject/claim 五个 handler 的 import 输入面与等价行为已冻结。
3. BE-001FD-02 成为唯一下一步。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
