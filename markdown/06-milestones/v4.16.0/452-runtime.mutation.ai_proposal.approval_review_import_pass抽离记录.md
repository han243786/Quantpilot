# v4.16.0 runtime.mutation.ai_proposal.approval_review_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FD-03
> 方案: `451-runtime.mutation.ai_proposal.approval_review_import_pass抽离方案.md`
> 目标子叶: `runtime.mutation.ai_proposal.approval_review_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.approval_review_import_pass`
> 代码动作: actual import rewrite
> 下一步: BE-001FD-04 `runtime.mutation.ai_proposal.approval_review_import_pass` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FD-03 `runtime.mutation.ai_proposal.approval_review_import_pass` 抽离记录 | 实际抽离 |
| 规范矩阵 | single-file import rewrite / approval review semantics frozen / no release transition | 受控 import 收敛 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.approval_review_import_pass` | approval review import pocket |
| 模块树 | `runtime.mutation.ai_proposal.approval_review_import_pass` | 抽离记录 |

---

## 抽离结果

BE-001FD-03 已按 BE-001FD-02 方案完成单文件 import rewrite:

```text
runtime.mutation.ai_proposal.approval_review_import_pass extraction_done
removed use super::*
single file import rewrite
approval_review_explicit_imports
src/runtime/mutation/ai_proposal/approval_review.rs
```

实际改动只发生在 `src/runtime/mutation/ai_proposal/approval_review.rs` 顶部 import。五个 handler 的函数体、可见性、锁顺序、状态机、lifecycle、错误 payload 和持久化顺序均未修改。

---

## 实际显式输入面

```rust
use super::approval_persistence::{load_approval_from_disk, persist_approval};
use super::record_query::load_runtime_ai_proposal_for_user;
use super::sandbox_trigger::ensure_ai_proposal_can_be_approved;
use super::status_transition::{ai_proposal_approved_status, update_ai_proposal_status};
use super::RuntimeApprovalListQuery;
use crate::{
    auth, current_time_ms, io_error, json_bad_request, AppState, ApprovalActionRequest,
    RuntimeAiProposalStatus, RuntimeApprovalLifecycleEntry, RuntimeApprovalRecord,
    RuntimeApprovalReviewState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
```

---

## 等价边界复核

以下行为保持不变:

```text
list_runtime_approvals body unchanged
get_runtime_approval_detail body unchanged
approve_ai_proposal body unchanged
reject_ai_proposal body unchanged
claim_ai_proposal_review body unchanged
pub(crate) visibility unchanged
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

本批保持:

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

## residual 状态

本批完成后，父级 residual 下降为:

```text
actual_runtime_parent_import_bridge_4_to_3
actual_mutation_import_bridge_3_to_2
actual_ai_proposal_import_bridge_3_to_2
remaining_runtime_parent_import_bridge_3
remaining_mutation_import_bridge_2
remaining_ai_proposal_import_bridge_2
```

剩余文件:

```text
src/runtime/mod.rs
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/proposal_creation.rs
```

---

## 排除项

本批未处理:

1. 未改 handler 函数体。
2. 未处理 `proposal_creation.rs` 或 `ai_proposal.rs` parent facade import residual。
3. 未处理 `src/runtime/mod.rs` root parent bridge。
4. 未迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
5. 未新增 sibling 横向连接。
6. 未启动 release transition。

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

AI 声称 BE-001FD-03 完成时，必须说明:

1. 本批只完成 `approval_review.rs` 顶部 import rewrite。
2. `approval_review.rs` 已移除 `use super::*`，但五个 handler 函数体未改。
3. 下一步只能进入 BE-001FD-04 单叶 closeout。
4. 不得宣称 proposal_creation、ai proposal parent facade、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `452-runtime.mutation.ai_proposal.approval_review_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `approval_review.rs` 顶部 import 已显式化。
3. BE-001FD-04 成为唯一下一步。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
