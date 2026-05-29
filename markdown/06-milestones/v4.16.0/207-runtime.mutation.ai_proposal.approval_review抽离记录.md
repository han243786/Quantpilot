# v4.16.0 runtime.mutation.ai_proposal.approval_review 抽离记录
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BH-03  
> 基线: `205-runtime.mutation.ai_proposal.approval_review单子叶等价基线.md`、`206-runtime.mutation.ai_proposal.approval_review抽离方案.md`、`src/runtime/mutation/ai_proposal.rs`、`tests/api_ai_proposal.rs`、`tests/api_mutation.rs`  
> 判定: `runtime.mutation.ai_proposal.approval_review` 第一轮实际抽离完成。approval list/detail/approve/reject/claim 五个 route-facing handler 已迁入 child 文件；approval_persistence、sandbox_trigger、status_transition、AppState、schema owner、frontend caller、route facade、runtime persistence owner 和 release transition guard 均未迁移。下一步只能进入 BE-001BH-04 单叶 closeout。  
> 代码动作: actual extraction

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BH-03 approval_review 实际抽离 | 已落地 |
| 规范矩阵 | 父子通信、五 handler re-export、`use super::*`、helper 保留 | 执行 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.approval_review` | child 文件落地 |
| 模块树 | `runtime.mutation.ai_proposal.approval_review` | 白箱抽离完成 |

---

## 实际文件变更

新增 child 文件:

```text
src/runtime/mutation/ai_proposal/approval_review.rs
```

父级 `src/runtime/mutation/ai_proposal.rs` 新增 path-attributed child:

```rust
#[path = "ai_proposal/approval_review.rs"]
mod approval_review;
```

父级通过 handler re-export 继续向 `src/runtime/mod.rs` 和 route facade 暴露原调用面:

```rust
pub(crate) use approval_review::{
    approve_ai_proposal, claim_ai_proposal_review, get_runtime_approval_detail,
    list_runtime_approvals, reject_ai_proposal,
};
```

child 只通过父级白箱输入取依赖:

```rust
use super::*;
```

---

## 已迁移函数

| 函数 | BE-001BH-03 visibility | 调整原因 |
| --- | --- | --- |
| `list_runtime_approvals` | `pub(crate) async` | route-facing approval list handler 继续经父级 re-export 暴露 |
| `get_runtime_approval_detail` | `pub(crate) async` | route-facing approval detail handler 继续经父级 re-export 暴露 |
| `approve_ai_proposal` | `pub(crate) async` | route-facing approve action handler 继续经父级 re-export 暴露 |
| `reject_ai_proposal` | `pub(crate) async` | route-facing reject action handler 继续经父级 re-export 暴露 |
| `claim_ai_proposal_review` | `pub(crate) async` | route-facing claim action handler 继续经父级 re-export 暴露 |

BE-001BH-03 未移动单测。现有 API 回归继续通过 public handler 证明行为等价。

---

## 等价保持

已保持:

- `RuntimeApprovalListQuery`
- `ApprovalActionRequest`
- `RuntimeApprovalRecord`
- `RuntimeApprovalReviewState`
- `auth::scoped_key`
- `created_at_ms`
- `review_state`
- reviewer lifecycle
- quorum
- `approval_records -> ai_proposals`
- `APPROVAL_APPROVED`
- `APPROVAL_PARTIAL`
- `APPROVAL_REJECTED`
- `APPROVAL_CLAIMED`
- `load_runtime_ai_proposal_for_user`
- `ensure_ai_proposal_can_be_approved`
- `ai_proposal_approved_status`
- `update_ai_proposal_status`
- `persist_approval`
- `load_approval_from_disk`

list 继续使用 scoped prefix 过滤当前用户审批单，继续按 `review_state` lowercase filtering，并按 `created_at_ms` 倒序排序。detail 继续保持 memory-first scoped lookup，miss 后再 disk fallback。approve/reject/claim 继续保持 review_state guard、reviewer lifecycle、quorum 和 proposal status side effect，approve/reject 继续保持 `approval_records -> ai_proposals` 锁顺序。

---

## 未迁移边界

BE-001BH-03 未迁移或修改:

- `create_runtime_ai_proposal`
- `load_sandbox_report_for_proposal`
- `ensure_ai_proposal_can_be_approved`
- `ai_proposal_approved_status`
- `is_valid_ai_proposal_transition`
- `update_ai_proposal_status`
- `persist_approval`
- `load_approval_from_disk`
- `approval_persistence`
- `sandbox_trigger`
- `status_transition`
- `AppState`
- schema owner `src/frontend_api_types.rs`
- frontend caller
- route facade `src/backend/runtime/routes/mutation.rs`
- runtime persistence owner
- release transition guard

不得把本批解释为 approval persistence、sandbox trigger 或 status transition 已拆分。

---

## 等价验证

必须验证:

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

## 回退点

若 BE-001BH-03 后续验证失败，只允许回退本批新增/修改:

- `src/runtime/mutation/ai_proposal/approval_review.rs`
- `src/runtime/mutation/ai_proposal.rs` 中的 `#[path = "ai_proposal/approval_review.rs"] mod approval_review;`
- `src/runtime/mutation/ai_proposal.rs` 中的 `pub(crate) use approval_review::{...};`
- `src/runtime/mutation/ai_proposal.rs` 中由本批 approval review 迁移造成的删除

不得回退已 closeout 的 `static_check`、`source_governance_identity`、`event_lifecycle` 或 `record_query`，不得改 route facade、schema、AppState、frontend caller、runtime persistence owner 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001BH-03 完成时，必须说明当前只完成 `runtime.mutation.ai_proposal.approval_review` 第一轮实际抽离。不得宣称本叶已完成 closeout、approval persistence / sandbox trigger / status transition 已拆分、AppState/schema/frontend caller 已改变、发布过渡已启动或 Rust backend 重构已经完成。

---

## 验收标准

1. `207-runtime.mutation.ai_proposal.approval_review抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/mutation/ai_proposal/approval_review.rs` 存在，并承载 approval list/detail/approve/reject/claim 五个 handler。
3. 父级只通过 path-attributed child、五 handler re-export 和 `use super::*` 父子通信连接 child。
4. approval_persistence、sandbox_trigger 和 status_transition 未被宣称完成。
5. 验证通过后，后续只能进入 BE-001BH-04 单叶 closeout，判断本 child 是否值得继续细拆。
