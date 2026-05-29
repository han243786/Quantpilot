# v4.16.0 runtime.mutation.ai_proposal.approval_review 单叶 closeout
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BH-04  
> 基线: `205-runtime.mutation.ai_proposal.approval_review单子叶等价基线.md`、`206-runtime.mutation.ai_proposal.approval_review抽离方案.md`、`207-runtime.mutation.ai_proposal.approval_review抽离记录.md`、`src/runtime/mutation/ai_proposal/approval_review.rs`  
> 判定: `runtime.mutation.ai_proposal.approval_review` 单叶 closeout 完成，设置 `stop_split: true`。approval list/detail/approve/reject/claim 五个 handler 共同构成同一审批 review route-facing owner；继续拆成 query/action 微叶不会产生新的稳定状态 owner、schema owner、route facade 或锁 owner，只会增加父子接线和 re-export 面。下一步只能进入 BE-001BI-01 `runtime.mutation.ai_proposal` 第五轮父叶残余判断。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BH-04 approval_review 单叶 closeout | 收口 |
| 规范矩阵 | stop_split、父子通信、approval review owner、非目标边界 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.approval_review` | 白箱 closeout |
| 模块树 | `runtime.mutation.ai_proposal.approval_review` | 设置 `stop_split: true` |

---

## closeout 结论

`runtime.mutation.ai_proposal.approval_review` 已完成当前范围内的等价基线、抽离方案和实际抽离。

本叶设置:

```text
stop_split: true
```

原因:

- `list_runtime_approvals` 与 `get_runtime_approval_detail` 是 approval review 的只读入口，但不足以形成独立 owner。
- `approve_ai_proposal`、`reject_ai_proposal` 与 `claim_ai_proposal_review` 共享同一 `RuntimeApprovalRecord`、`RuntimeApprovalReviewState`、reviewer lifecycle、quorum 和 approval lifecycle。
- 三个 action handler 都依赖同一 `state.approval_records` 写锁、`auth::scoped_key`、`persist_approval` 与 proposal status side effect。
- approve/reject 继续保持 `approval_records -> ai_proposals` 锁顺序；这一锁序属于 approval review 的整体契约，不应被微拆到多个 sibling。
- 继续拆成 query/action 或 approve/reject/claim 微文件不会形成新的独立状态、独立锁、独立 schema、独立 route facade 或独立验证证据。
- 继续拆分会增加 `pub(crate)` re-export、父级 wiring 面和治理挂载面，违反当前父子通信收敛目标。

---

## 已落地文件

```text
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/approval_review.rs
```

父级保留:

```rust
#[path = "ai_proposal/approval_review.rs"]
mod approval_review;

pub(crate) use approval_review::{
    approve_ai_proposal, claim_ai_proposal_review, get_runtime_approval_detail,
    list_runtime_approvals, reject_ai_proposal,
};
```

child 保持:

```rust
use super::*;
```

---

## 等价确认

已确认:

- `list_runtime_approvals` 继续用 `auth::scoped_key(&user_id, "")` 做 scoped prefix。
- list 继续读取 `state.approval_records` memory，不触碰 disk list。
- `RuntimeApprovalListQuery.review_state` 继续 lowercase filtering。
- `ApprovalActionRequest` 继续作为 approve/reject/claim 的 action input，不改变 actor_id 或 comment 字段语义。
- list sorting 继续按 `created_at_ms` 倒序。
- `get_runtime_approval_detail` 继续 memory-first scoped lookup，miss 后调用 `load_approval_from_disk` 做 disk fallback。
- `approve_ai_proposal` 继续先经过 `load_runtime_ai_proposal_for_user` 与 `ensure_ai_proposal_can_be_approved`。
- approve 继续保持 reviewer 去重、quorum、`APPROVAL_APPROVED` / `APPROVAL_PARTIAL` lifecycle 和 `ai_proposal_approved_status` side effect。
- `reject_ai_proposal` 继续保持 Pending / UnderReview guard、`APPROVAL_REJECTED` lifecycle、comment fallback 和 `RuntimeAiProposalStatus::Denied` side effect。
- `claim_ai_proposal_review` 继续保持 Pending-only guard、assigned 去重、UnderReview 状态和 `APPROVAL_CLAIMED` lifecycle。
- approve/reject 继续保持 `approval_records -> ai_proposals` lock order。

---

## 未迁移边界

本 closeout 不迁移:

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

---

## 验证证据

BE-001BH-03 实际抽离后已验证:

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

本 closeout 批次为 `no code movement`，提交前继续执行治理门禁。

---

## 下一步

下一步只能进入:

```text
BE-001BI-01 runtime.mutation.ai_proposal 第五轮父叶残余判断
```

该父叶残余判断只能评估 `approval_persistence`、`sandbox_trigger`、`status_transition` 等剩余稳定职责，不得回改已 closeout 的 `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query` 或 `approval_review`。

---

## 幻觉检查点

AI 声称 BE-001BH-04 完成时，必须说明 `runtime.mutation.ai_proposal.approval_review` 已 closeout 并设置 `stop_split: true`，但 `runtime.mutation.ai_proposal` 父叶尚未完成。不得宣称 approval_persistence、sandbox_trigger、status_transition、AppState/schema/frontend caller 已改变、release transition 已启动或 Rust backend 重构已完成。

---

## 验收标准

1. `208-runtime.mutation.ai_proposal.approval_review单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树将 `runtime.mutation.ai_proposal.approval_review` 标记为 `stop_split: true`。
3. 下一步固定为 BE-001BI-01 `runtime.mutation.ai_proposal` 第五轮父叶残余判断。
4. approval_review 不再继续细拆，除非未来有新的独立状态/锁/schema/route owner 证据并重新走提案流程。
