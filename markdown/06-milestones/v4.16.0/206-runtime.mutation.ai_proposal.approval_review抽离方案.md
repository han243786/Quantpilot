# v4.16.0 runtime.mutation.ai_proposal.approval_review 抽离方案
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BH-02  
> 基线: `205-runtime.mutation.ai_proposal.approval_review单子叶等价基线.md`、`src/runtime/mutation/ai_proposal.rs`  
> 判定: 固定 BE-001BH-03 的实际抽离方案。下一步只允许创建 `src/runtime/mutation/ai_proposal/approval_review.rs`，迁移 approval list/detail/approve/reject/claim 五个 route-facing handler，并通过父级 path-attributed child 与 handler re-export 保持外部调用面不变。当前 `no code movement`。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BH-02 approval_review 抽离方案 | 方案固化 |
| 规范矩阵 | 父子通信、handler re-export、helper 保留、回退点 | 约束冻结 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.approval_review` | 抽离路径 |
| 模块树 | `runtime.mutation.ai_proposal.approval_review` | 计划物理抽离 |

---

## 目标文件与父级接线

BE-001BH-03 允许创建:

```text
src/runtime/mutation/ai_proposal/approval_review.rs
```

父级 `src/runtime/mutation/ai_proposal.rs` 允许新增:

```rust
#[path = "ai_proposal/approval_review.rs"]
mod approval_review;

pub(crate) use approval_review::{
    approve_ai_proposal, claim_ai_proposal_review, get_runtime_approval_detail,
    list_runtime_approvals, reject_ai_proposal,
};
```

child 文件固定使用:

```rust
use super::*;
```

不允许改变 `src/backend/runtime/routes/mutation.rs` route facade，不允许改变 `src/runtime/mod.rs` 对外导出清单，除非 rustfmt 只调整排序。

---

## 允许迁移清单

BE-001BH-03 只允许把下列函数从父级移动到 child:

- `list_runtime_approvals`
- `get_runtime_approval_detail`
- `approve_ai_proposal`
- `reject_ai_proposal`
- `claim_ai_proposal_review`

这些函数迁移后必须继续保持 `pub(crate)` handler 可见性，并由父级 re-export 给 `src/runtime/mod.rs` 与 route facade。

---

## 类型契约标记

BE-001BH-03 不允许改变以下 schema / query / action 类型:

- `RuntimeApprovalListQuery`
- `ApprovalActionRequest`
- `RuntimeApprovalRecord`
- `RuntimeApprovalReviewState`
- reviewer lifecycle

---

## 必须留在父级的 helper

下列 helper 本轮不得迁移:

- `create_runtime_ai_proposal`
- `load_sandbox_report_for_proposal`
- `ensure_ai_proposal_can_be_approved`
- `ai_proposal_approved_status`
- `is_valid_ai_proposal_transition`
- `update_ai_proposal_status`
- `persist_approval`
- `load_approval_from_disk`

其中:

- `ensure_ai_proposal_can_be_approved` 与 `load_sandbox_report_for_proposal` 仍归后续 `sandbox_trigger` 残余。
- `ai_proposal_approved_status`、`is_valid_ai_proposal_transition` 与 `update_ai_proposal_status` 仍归后续 `status_transition` 残余。
- `persist_approval` 与 `load_approval_from_disk` 仍归后续 `approval_persistence` 残余。
- `load_runtime_ai_proposal_for_user` 已由 closed child `record_query` 提供，approval_review 只能通过父级受控导入使用，不得直接横向连接 record_query。

---

## 行为不变要求

BE-001BH-03 必须保持以下行为逐字等价:

1. `list_runtime_approvals` 继续使用 `auth::scoped_key(&user_id, "")` 做 prefix，继续只读取 `state.approval_records` memory，继续按 `review_state` lowercase filtering，继续按 `created_at_ms` 倒序排序。
2. `get_runtime_approval_detail` 继续 memory-first scoped lookup，miss 后继续调用 `load_approval_from_disk(&state.approval_store_dir, &approval_id)`。
3. `approve_ai_proposal` 继续先调用 `load_runtime_ai_proposal_for_user` 与 `ensure_ai_proposal_can_be_approved`，再持有 `state.approval_records.write()` 完成读改写。
4. `approve_ai_proposal` 继续保持 reviewer 去重、quorum 判断、`APPROVAL_APPROVED` / `APPROVAL_PARTIAL` lifecycle 与 Approved side effect。
5. `reject_ai_proposal` 继续只允许 Pending / UnderReview，继续写入 `APPROVAL_REJECTED` lifecycle，comment 缺省为 `"审批拒绝"`，并把 proposal status 写为 Denied。
6. `claim_ai_proposal_review` 继续只允许 Pending，继续保持 assigned 去重、UnderReview 状态和 `APPROVAL_CLAIMED` lifecycle。
7. approve/reject 的状态副作用继续保持 `approval_records -> ai_proposals` 锁顺序。

---

## 测试策略

BE-001BH-03 实际抽离必须运行:

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

如果实际抽离触碰 lifecycle 拼装、quorum、claim/reject guard 或锁顺序，允许在同一实际抽离批次补 focused approval API equivalence tests；测试只能覆盖现有行为，不得扩大审批语义。

---

## 回退点

如果 BE-001BH-03 编译或测试失败，回退方式固定为:

1. 删除 `src/runtime/mutation/ai_proposal/approval_review.rs`。
2. 从 child 恢复五个 handler 到 `src/runtime/mutation/ai_proposal.rs` 原位置。
3. 删除父级 `mod approval_review` 与 `pub(crate) use approval_review::{...};`。
4. 不回退 closed child: `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query`。
5. 不改变 AppState、schema owner、frontend caller、route facade、runtime persistence owner 或 release transition guard。

---

## 非目标边界

BE-001BH-02 不移动代码。BE-001BH-03 也不得迁移或修改:

- `create_runtime_ai_proposal`
- `load_sandbox_report_for_proposal`
- `ensure_ai_proposal_can_be_approved`
- `ai_proposal_approved_status`
- `is_valid_ai_proposal_transition`
- `update_ai_proposal_status`
- `persist_approval`
- `load_approval_from_disk`
- `AppState`
- schema owner `src/frontend_api_types.rs`
- frontend caller
- route facade `src/backend/runtime/routes/mutation.rs`
- runtime persistence owner
- release transition guard

不得回收或重拆 `static_check`、`source_governance_identity`、`event_lifecycle` 或 `record_query` 已 closeout 子叶。

---

## 下一步

下一步只能进入:

```text
BE-001BH-03 runtime.mutation.ai_proposal.approval_review 实际抽离
```

该步骤才允许创建目标文件并迁移允许清单内五个 handler。

---

## 验证计划

本批 `no code movement`，只需要治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001BH-02 完成时，必须说明当前只是 `runtime.mutation.ai_proposal.approval_review` 抽离方案，尚未创建目标文件也尚未迁移 handler。不得宣称 approval_review 已抽离、approval_persistence 已拆分、sandbox_trigger 已迁移、status_transition 已迁移、AppState/schema/frontend caller、route facade、runtime persistence owner 或 release transition 已改变。

---

## 验收标准

1. `206-runtime.mutation.ai_proposal.approval_review抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001BH-03 的目标文件、父级声明、handler re-export、迁移清单、非目标和回退点已固定。
3. 下一步固定为 BE-001BH-03 实际抽离。
4. 本批不产生代码变更，不回收 closed child，不启动 release transition。
