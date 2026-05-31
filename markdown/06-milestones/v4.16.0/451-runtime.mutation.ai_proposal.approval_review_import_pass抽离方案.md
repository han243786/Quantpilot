# v4.16.0 runtime.mutation.ai_proposal.approval_review_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FD-02
> 基线: `450-runtime.mutation.ai_proposal.approval_review_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.mutation.ai_proposal.approval_review_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.approval_review_import_pass`
> 代码动作: no code movement
> 下一步: BE-001FD-03 `runtime.mutation.ai_proposal.approval_review_import_pass` 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FD-02 `runtime.mutation.ai_proposal.approval_review_import_pass` 抽离方案 | 方案优化 |
| 规范矩阵 | single-file import rewrite / approval lock order freeze / status transition freeze / no release transition | 固定抽离边界 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.approval_review_import_pass` | approval review import pocket |
| 模块树 | `runtime.mutation.ai_proposal.approval_review_import_pass` | 抽离方案 |

---

## 方案结论

BE-001FD-03 只能改写 `src/runtime/mutation/ai_proposal/approval_review.rs` 顶部 import，禁止改函数体、可见性、approval list/detail/approve/reject/claim 语义、锁顺序、reviewer count、lifecycle event、status transition、persist order 或 sibling owner。

```text
runtime.mutation.ai_proposal.approval_review_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.approval_review_import_pass
approval_review_import_pass plan_frozen
single_file_approval_review_import_rewrite
next_step: BE-001FD-03 extraction record
```

---

## 允许改动

唯一允许的 Rust 改动是把 `src/runtime/mutation/ai_proposal/approval_review.rs` 顶部 import 从 parent wildcard 改为显式输入面:

```diff
-use super::*;
+use super::approval_persistence::{load_approval_from_disk, persist_approval};
+use super::record_query::load_runtime_ai_proposal_for_user;
+use super::sandbox_trigger::ensure_ai_proposal_can_be_approved;
+use super::status_transition::{ai_proposal_approved_status, update_ai_proposal_status};
+use super::RuntimeApprovalListQuery;
+use crate::{
+    auth, current_time_ms, io_error, json_bad_request, AppState, ApprovalActionRequest,
+    RuntimeAiProposalStatus, RuntimeApprovalLifecycleEntry, RuntimeApprovalRecord,
+    RuntimeApprovalReviewState,
+};
+use axum::{
+    extract::{Path, Query, State},
+    http::StatusCode,
+    Json,
+};
```

允许 `cargo fmt` 对 import 分组做机械格式化。除此以外不得改动任何 Rust 语句。

---

## 不允许改动

BE-001FD-03 不得修改:

```text
list_runtime_approvals body
get_runtime_approval_detail body
approve_ai_proposal body
reject_ai_proposal body
claim_ai_proposal_review body
pub(crate) visibility
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

必须保持:

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

## residual 预期

BE-001FD-03 完成后，预期 residual 从:

```text
remaining_runtime_parent_import_bridge_4
remaining_mutation_import_bridge_3
remaining_ai_proposal_import_bridge_3
```

下降为:

```text
expected_runtime_parent_import_bridge_3
expected_mutation_import_bridge_2
expected_ai_proposal_import_bridge_2
```

仍不处理:

```text
src/runtime/mod.rs
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/proposal_creation.rs
```

---

## 排除项

本批不处理:

1. 不改 Rust 代码。
2. 不执行 BE-001FD-03 实际 import rewrite。
3. 不处理 `proposal_creation.rs` 或 `ai_proposal.rs` parent facade import residual。
4. 不处理 `src/runtime/mod.rs` root parent bridge。
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

AI 声称 BE-001FD-02 完成时，必须说明:

1. 本批只是 `no code movement` 抽离方案。
2. BE-001FD-03 只允许改写 `src/runtime/mutation/ai_proposal/approval_review.rs` 顶部 import。
3. 不得宣称 `approval_review.rs` 已完成实际抽离。
4. 不得宣称 ai proposal import、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `451-runtime.mutation.ai_proposal.approval_review_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001FD-03 的允许改动被限制为单文件 import rewrite。
3. BE-001FD-03 成为唯一下一步。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
