# v4.16.0 runtime.mutation.ai_proposal.record_query 抽离记录
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BF-03  
> 基线: `200-runtime.mutation.ai_proposal.record_query单子叶等价基线.md`、`201-runtime.mutation.ai_proposal.record_query抽离方案.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime_persistence.rs`、`tests/api_ai_proposal.rs`、`tests/api_mutation.rs`  
> 判定: `runtime.mutation.ai_proposal.record_query` 第一轮实际抽离完成。proposal list/detail/read-through loader 已迁入 child 文件；approval_review、approval_persistence、sandbox_trigger、status_transition、AppState、schema owner、frontend caller、route facade、runtime persistence owner 和 release transition guard 均未迁移。下一步只能进入 BE-001BF-04 单叶 closeout。  
> 代码动作: actual extraction

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BF-03 record_query 实际抽离 | 已落地 |
| 规范矩阵 | 父子通信、双 public handler re-export、`pub(super)` loader、read model 边界 | 执行 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.record_query` | child 文件落地 |
| 模块树 | `runtime.mutation.ai_proposal.record_query` | 白箱抽离完成 |

---

## 实际文件变更

新增 child 文件:

```text
src/runtime/mutation/ai_proposal/record_query.rs
```

父级 `src/runtime/mutation/ai_proposal.rs` 新增 path-attributed child:

```rust
#[path = "ai_proposal/record_query.rs"]
mod record_query;
```

父级通过 public handler re-export 继续向 `src/runtime/mod.rs` 暴露 route-facing handler:

```rust
pub(crate) use record_query::{
    get_runtime_ai_proposal_detail, list_runtime_ai_proposals,
};
```

父级通过受控 helper import 继续复用 read-through loader:

```rust
use record_query::load_runtime_ai_proposal_for_user;
```

child 只通过父级白箱输入取依赖:

```rust
use super::*;
```

---

## 已迁移函数

| 函数 | BE-001BF-03 visibility | 调整原因 |
| --- | --- | --- |
| `load_runtime_ai_proposal_for_user` | `pub(super) async` | 父级 approval/status/sandbox flow 仍需复用 scoped read-through loader |
| `list_runtime_ai_proposals` | `pub(crate) async` | route-facing handler 继续经父级 re-export 暴露 |
| `get_runtime_ai_proposal_detail` | `pub(crate) async` | route-facing handler 继续经父级 re-export 暴露 |

BE-001BF-03 未移动单测。现有 API 回归继续通过 public handler 证明行为等价。

---

## 等价保持

已保持:

- `list_runtime_ai_proposal_records`
- `state.ai_proposal_store_dir`
- `io_error`
- `RuntimeAiProposalListQuery`
- `source_kind`
- `source_id`
- `status`
- `clean_optional_filter`
- `created_at_ms`
- `ai_proposal_id`
- `auth::scoped_key`
- `state.ai_proposals`
- `load_runtime_ai_proposal_record`
- `memory-first`
- `disk fallback`

list sorting 继续为 `created_at_ms` 倒序，随后 `ai_proposal_id` 倒序。detail 与 loader 继续保持 scoped in-memory lookup 优先，miss 后再 fallback 到 persistence load helper。

---

## 未迁移边界

BE-001BF-03 未迁移或修改:

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

不得把本批解释为 approval review、approval persistence、sandbox trigger 或 status transition 已拆分。

---

## 等价验证

已验证:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
git diff --check
```

文档补齐后继续验证:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
```

---

## 回退点

若 BE-001BF-03 后续验证失败，只允许回退本批新增/修改:

- `src/runtime/mutation/ai_proposal/record_query.rs`
- `src/runtime/mutation/ai_proposal.rs` 中的 `#[path = "ai_proposal/record_query.rs"] mod record_query;`
- `src/runtime/mutation/ai_proposal.rs` 中的 `pub(crate) use record_query::{...};`
- `src/runtime/mutation/ai_proposal.rs` 中的 `use record_query::load_runtime_ai_proposal_for_user;`
- `src/runtime/mutation/ai_proposal.rs` 中由本批 record query 迁移造成的删除

不得回改已 closeout 的 `static_check`、`source_governance_identity` 或 `event_lifecycle`，不得改 route facade、schema、AppState、frontend caller、runtime persistence owner 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001BF-03 完成时，必须说明当前只完成 `runtime.mutation.ai_proposal.record_query` 第一轮实际抽离。不得宣称本叶已完成 closeout、approval review / approval persistence / sandbox trigger / status transition 已拆分、AppState/schema/frontend caller 已改变、发布过渡已启动或 Rust backend 重构已经完成。

---

## 验收标准

1. `202-runtime.mutation.ai_proposal.record_query抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/mutation/ai_proposal/record_query.rs` 存在，并承载 proposal list/detail/read-through loader。
3. 父级只通过 path-attributed child、public handler re-export 和 `pub(super)` loader import 调用 child。
4. approval_review、approval_persistence、sandbox_trigger 和 status_transition 未被宣称完成。
5. 验证通过后，后续只能进入 BE-001BF-04 单叶 closeout，判断本 child 是否值得继续细拆。
