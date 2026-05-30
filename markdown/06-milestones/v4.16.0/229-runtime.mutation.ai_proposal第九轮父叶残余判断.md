# v4.16.0 runtime.mutation.ai_proposal 第九轮父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001BQ-01
> 基线: `184-runtime.mutation.ai_proposal单叶closeout.md`、`188-runtime.mutation.ai_proposal.static_check单叶closeout.md`、`193-runtime.mutation.ai_proposal.source_governance_identity单叶closeout.md`、`198-runtime.mutation.ai_proposal.event_lifecycle单叶closeout.md`、`203-runtime.mutation.ai_proposal.record_query单叶closeout.md`、`208-runtime.mutation.ai_proposal.approval_review单叶closeout.md`、`213-runtime.mutation.ai_proposal.approval_persistence单叶closeout.md`、`218-runtime.mutation.ai_proposal.sandbox_trigger单叶closeout.md`、`223-runtime.mutation.ai_proposal.status_transition单叶closeout.md`、`228-runtime.mutation.ai_proposal.proposal_creation单叶closeout.md`
> 判定: `runtime.mutation.ai_proposal` 父叶残余判断完成。九个生产子叶均已 closeout 并设置 `stop_split: true`；父叶生产代码只剩 path-attributed child、受控 helper import、public handler re-export 与 test guard，因此父叶设置 `stop_split: true`。下一步只能进入 BE-001BR-01 `backend.runtime.routes` 父叶残余判断。
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BQ-01 AI proposal 父叶残余判断 | 父叶收口 |
| 规范矩阵 | 父子通信、closed child 不回改、发布过渡保护 | `stop_split: true` |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal` | 父叶 closeout |
| 模块树 | `runtime.mutation.ai_proposal` | 所有子叶关闭，父叶关闭 |

---

## 子叶状态矩阵

| 子叶 | 文件 | closeout |
| --- | --- | --- |
| `static_check` | `src/runtime/mutation/ai_proposal/static_check.rs` | `stop_split: true` |
| `source_governance_identity` | `src/runtime/mutation/ai_proposal/source_governance_identity.rs` | `stop_split: true` |
| `event_lifecycle` | `src/runtime/mutation/ai_proposal/event_lifecycle.rs` | `stop_split: true` |
| `record_query` | `src/runtime/mutation/ai_proposal/record_query.rs` | `stop_split: true` |
| `approval_review` | `src/runtime/mutation/ai_proposal/approval_review.rs` | `stop_split: true` |
| `approval_persistence` | `src/runtime/mutation/ai_proposal/approval_persistence.rs` | `stop_split: true` |
| `sandbox_trigger` | `src/runtime/mutation/ai_proposal/sandbox_trigger.rs` | `stop_split: true` |
| `status_transition` | `src/runtime/mutation/ai_proposal/status_transition.rs` | `stop_split: true` |
| `proposal_creation` | `src/runtime/mutation/ai_proposal/proposal_creation.rs` | `stop_split: true` |

---

## 父叶当前生产形态

`src/runtime/mutation/ai_proposal.rs` 当前只保留:

```rust
#[path = "ai_proposal/approval_persistence.rs"]
mod approval_persistence;
#[path = "ai_proposal/approval_review.rs"]
mod approval_review;
#[path = "ai_proposal/event_lifecycle.rs"]
mod event_lifecycle;
#[path = "ai_proposal/proposal_creation.rs"]
mod proposal_creation;
#[path = "ai_proposal/record_query.rs"]
mod record_query;
#[path = "ai_proposal/sandbox_trigger.rs"]
mod sandbox_trigger;
#[path = "ai_proposal/source_governance_identity.rs"]
mod source_governance_identity;
#[path = "ai_proposal/static_check.rs"]
mod static_check;
#[path = "ai_proposal/status_transition.rs"]
mod status_transition;
```

父级 public handler 出口保持:

```rust
pub(crate) use approval_review::{
    approve_ai_proposal, claim_ai_proposal_review, get_runtime_approval_detail,
    list_runtime_approvals, reject_ai_proposal,
};
pub(crate) use proposal_creation::create_runtime_ai_proposal;
pub(crate) use record_query::{get_runtime_ai_proposal_detail, list_runtime_ai_proposals};
```

父级受控 helper import 仅服务子叶父子通信:

```rust
use approval_persistence::{load_approval_from_disk, persist_approval};
use event_lifecycle::{
    ai_proposal_lifecycle_entry, build_runtime_ai_proposal_event,
    persist_runtime_ai_proposal_transition,
};
use record_query::load_runtime_ai_proposal_for_user;
use sandbox_trigger::{ensure_ai_proposal_can_be_approved, spawn_ai_proposal_sandbox_verification};
use source_governance_identity::{
    load_runtime_ai_proposal_source_context, runtime_ai_proposal_governance,
    runtime_ai_proposal_record_id,
};
use static_check::{
    ai_proposal_static_check_result, validate_ai_model_identity, validate_hash_identity,
};
use status_transition::{ai_proposal_approved_status, update_ai_proposal_status};
```

所有 child 固定通过 `use super::*` 复用父级白箱输入，不横向 import sibling。

---

## 残余判断

父叶仍包含 `v4_ai_proposal_tests`，但它是 test guard，不是生产 owner。该测试覆盖 sandbox binding / approval gate 的父级受控 helper 接线，保留在父叶能同时观察 `ensure_ai_proposal_can_be_approved` 与 shared test fixture，不形成新的生产叶子。

生产残余只剩:

1. path-attributed child declaration。
2. public handler re-export。
3. parent-mediated helper import。
4. test-only guard。

这些职责共同构成父级白箱网络接线层，不再值得继续拆分。继续拆会把 `approval_persistence`、`approval_review`、`event_lifecycle`、`proposal_creation`、`record_query`、`sandbox_trigger`、`source_governance_identity`、`static_check` 与 `status_transition` 的父子通信再次外溢，增加横向连接幻觉风险。因此父叶设置 `stop_split: true`。

---

## 非目标边界

BE-001BQ-01 不迁移、不修改:

- `AppState`
- schema owner
- frontend caller
- route facade `src/backend/runtime/routes/mutation.rs`
- runtime persistence owner
- `approval_records -> ai_proposals` 锁顺序
- `auth::scoped_key`
- `state.ai_proposals`
- `state.approval_records`
- release transition guard

---

## 回归保护

本父叶判断为治理收口批次，只运行治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只能进入:

```text
BE-001BR-01 backend.runtime.routes 父叶残余判断
```

该父叶判断只允许确认 run / event_stream / backtest / mutation handler 域当前递归状态，并选择下一个 route sibling 候选；不得跳过适配性校验直接迁移 report、evidence、experiment、ops、schema owner、state/persistence owner、frontend caller 或发布过渡连接。

---

## 幻觉检查点

AI 声称 BE-001BQ-01 完成时，必须说明 `runtime.mutation.ai_proposal` 父叶已 closeout 并设置 `stop_split: true`，但 `backend.runtime.routes` 父叶尚未完成残余判断。不得宣称 AppState/schema/frontend caller、route facade、runtime persistence owner 或 release transition guard 已改变，也不得宣称 Rust backend 重构已完成。

---

## 验收标准

1. `runtime.mutation.ai_proposal` 在模块树中设置 `stop_split: true`。
2. `229-runtime.mutation.ai_proposal第九轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
3. 文档明确九个子叶均已 closeout 且不继续细拆。
4. 下一步固定为 BE-001BR-01 `backend.runtime.routes` 父叶残余判断。
5. 本批次保持 `no code movement`。
