# v4.16.0 runtime.mutation.ai_proposal.record_query 单叶 closeout
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BF-04  
> 基线: `200-runtime.mutation.ai_proposal.record_query单子叶等价基线.md`、`201-runtime.mutation.ai_proposal.record_query抽离方案.md`、`202-runtime.mutation.ai_proposal.record_query抽离记录.md`、`src/runtime/mutation/ai_proposal/record_query.rs`  
> 判定: `runtime.mutation.ai_proposal.record_query` 单叶 closeout 完成，设置 `stop_split: true`。list/detail/read-through loader 属于同一个 AI proposal record read model；继续拆成 list/detail/loader 微文件不会产生新的稳定 owner，只会增加父子接线与 re-export 面。下一步只能进入 BE-001BG-01 `runtime.mutation.ai_proposal` 父叶残余判断。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BF-04 record_query 单叶 closeout | 收口 |
| 规范矩阵 | stop_split、父子通信、read model owner、非目标边界 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.record_query` | 白箱 closeout |
| 模块树 | `runtime.mutation.ai_proposal.record_query` | 设置 `stop_split: true` |

---

## closeout 结论

`runtime.mutation.ai_proposal.record_query` 已完成当前范围内的等价基线、抽离方案和实际抽离。

本叶设置:

```text
stop_split: true
```

原因:

- `list_runtime_ai_proposals` 与 `get_runtime_ai_proposal_detail` 都属于 AI proposal record read model。
- `load_runtime_ai_proposal_for_user` 是同一 read model 的 parent-facing loader，不是新的独立 owner。
- list/detail/loader 都依赖同一组 `RuntimeAiProposalRecord`、`AppState.ai_proposals`、`auth::scoped_key`、persistence load/list helper 和 `ai_proposal_store_dir`。
- 继续拆成 list/detail/loader 三个微文件不会形成独立状态、独立锁、独立 schema、独立 route facade 或独立验证证据。
- 继续拆分会增加 `pub(crate)` re-export、`pub(super)` helper import 和父级 wiring 面，违反当前父子通信收敛目标。

---

## 已落地文件

```text
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/record_query.rs
```

父级保留:

```rust
#[path = "ai_proposal/record_query.rs"]
mod record_query;

pub(crate) use record_query::{
    get_runtime_ai_proposal_detail, list_runtime_ai_proposals,
};
use record_query::load_runtime_ai_proposal_for_user;
```

child 保持:

```rust
use super::*;
```

---

## 等价确认

已确认:

- `list_runtime_ai_proposals` 继续读取 `list_runtime_ai_proposal_records(&state.ai_proposal_store_dir)`。
- `source_kind` filtering 不变。
- `source_id` 继续通过 `clean_optional_filter` trim 并 drop empty string。
- `status` filtering 不变。
- list sorting 继续为 `created_at_ms desc` + `ai_proposal_id desc`。
- `get_runtime_ai_proposal_detail` 继续先查 `state.ai_proposals` 的 `auth::scoped_key`。
- detail miss 后继续 fallback `load_runtime_ai_proposal_record(state.ai_proposal_store_dir.as_ref(), &ai_proposal_id)`。
- `load_runtime_ai_proposal_for_user` 继续保持 `memory-first` 与 `disk fallback`。

---

## 未迁移边界

本 closeout 不迁移:

- `create_runtime_ai_proposal`
- `approve_ai_proposal`
- `reject_ai_proposal`
- `claim_ai_proposal_review`
- `list_runtime_approvals`
- `get_runtime_approval_detail`
- `persist_approval`
- `load_approval_from_disk`
- `load_sandbox_report_for_proposal`
- `ensure_ai_proposal_can_be_approved`
- `ai_proposal_approved_status`
- `is_valid_ai_proposal_transition`
- `update_ai_proposal_status`
- `AppState`
- schema owner `src/frontend_api_types.rs`
- frontend caller
- route facade `src/backend/runtime/routes/mutation.rs`
- runtime persistence owner
- release transition guard

ASCII non-target markers: `approval_review`、`approval_persistence`、`sandbox_trigger`、`status_transition`。

---

## 验证证据

BE-001BF-03 实际抽离后已验证:

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
BE-001BG-01 runtime.mutation.ai_proposal 父叶残余判断
```

该父叶残余判断只能评估 `approval_review`、`approval_persistence`、`sandbox_trigger`、`status_transition` 等剩余稳定职责，不得回改已 closeout 的 `static_check`、`source_governance_identity`、`event_lifecycle` 或 `record_query`。

---

## 幻觉检查点

AI 声称 BE-001BF-04 完成时，必须说明 `runtime.mutation.ai_proposal.record_query` 已 closeout 并设置 `stop_split: true`，但 `runtime.mutation.ai_proposal` 父叶尚未完成。不得宣称 approval review、approval persistence、sandbox trigger、status transition、AppState/schema/frontend caller 已改变、release transition 已启动或 Rust backend 重构已完成。

---

## 验收标准

1. `203-runtime.mutation.ai_proposal.record_query单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树将 `runtime.mutation.ai_proposal.record_query` 标记为 `stop_split: true`。
3. 下一步固定为 BE-001BG-01 `runtime.mutation.ai_proposal` 父叶残余判断。
4. record_query 不再继续细拆，除非未来有新的独立状态/锁/schema/route owner 证据并重新走提案流程。
