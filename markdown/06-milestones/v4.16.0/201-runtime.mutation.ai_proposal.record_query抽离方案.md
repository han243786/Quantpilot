# v4.16.0 runtime.mutation.ai_proposal.record_query 抽离方案
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BF-02  
> 基线: `200-runtime.mutation.ai_proposal.record_query单子叶等价基线.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime_persistence.rs`、`src/runtime/mod.rs`、`tests/api_ai_proposal.rs`、`tests/api_mutation.rs`  
> 判定: `runtime.mutation.ai_proposal.record_query` 抽离方案已建立；当前仍为 `no code movement`，只固定 BE-001BF-03 的目标文件、父级 child 声明、双 handler re-export、loader helper import、`use super::*`、迁移清单、非目标和回退点。下一步只能进入 BE-001BF-03 实际抽离。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BF-02 record_query 抽离方案 | 方案冻结 |
| 规范矩阵 | 父子通信、双 public handler re-export、`pub(super)` loader、read model 边界 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.record_query` | 子叶抽离路径 |
| 模块树 | `runtime.mutation.ai_proposal.record_query` | 白箱方案 |

---

## 目标文件与父级声明

BE-001BF-03 只允许创建一个目标文件:

```text
src/runtime/mutation/ai_proposal/record_query.rs
```

父级 `src/runtime/mutation/ai_proposal.rs` 只允许新增以下 child 声明:

```rust
#[path = "ai_proposal/record_query.rs"]
mod record_query;
```

父级只允许新增以下 public handler 出口:

```rust
pub(crate) use record_query::{
    get_runtime_ai_proposal_detail, list_runtime_ai_proposals,
};
```

父级只允许新增以下受控 helper import:

```rust
use record_query::load_runtime_ai_proposal_for_user;
```

child 文件必须以父级白箱输入为唯一来源:

```rust
use super::*;
```

---

## BE-001BF-03 迁移清单

只允许迁移:

- `async fn load_runtime_ai_proposal_for_user`
- `pub(crate) async fn list_runtime_ai_proposals`
- `pub(crate) async fn get_runtime_ai_proposal_detail`

`load_runtime_ai_proposal_for_user` 迁移后必须改为父级可见:

```rust
pub(super) async fn load_runtime_ai_proposal_for_user(
    state: &AppState,
    user_id: &auth::UserId,
    proposal_id: &str,
) -> Result<RuntimeAiProposalRecord, (StatusCode, String)>
```

`list_runtime_ai_proposals` 签名必须保持:

```rust
pub(crate) async fn list_runtime_ai_proposals(
    State(state): State<AppState>,
    Query(query): Query<RuntimeAiProposalListQuery>,
) -> Result<Json<Vec<RuntimeAiProposalRecord>>, (StatusCode, String)>
```

`get_runtime_ai_proposal_detail` 签名必须保持:

```rust
pub(crate) async fn get_runtime_ai_proposal_detail(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(ai_proposal_id): Path<String>,
) -> Result<Json<RuntimeAiProposalRecord>, (StatusCode, String)>
```

---

## list 等价约束

必须保持:

- `list_runtime_ai_proposal_records`
- `state.ai_proposal_store_dir`
- `io_error`
- `source_kind`
- `clean_optional_filter`
- `source_id`
- `status`
- `created_at_ms`
- `ai_proposal_id`

排序必须继续为 `created_at_ms` 倒序，随后 `ai_proposal_id` 倒序。不得引入 pagination、limit、offset 或 response shape 变化。

---

## detail / loader 等价约束

必须保持:

- `auth::scoped_key`
- `state.ai_proposals`
- `cloned`
- `load_runtime_ai_proposal_record`
- `state.ai_proposal_store_dir.as_ref()`
- `map(Json)`
- `memory-first`
- `disk fallback`

detail 必须保持 scoped in-memory lookup 优先，miss 后再 persistence fallback。`load_runtime_ai_proposal_for_user` 必须保持同一 read-through 行为，供 parent approval/status/sandbox flow 复用。

---

## 非目标

本批不得迁移:

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

不得回收或重拆 `static_check`、`source_governance_identity` 或 `event_lifecycle` 已 closeout 子叶。

---

## 回退点

若 BE-001BF-03 编译或等价检查失败，只允许回退本批新增的:

- `#[path = "ai_proposal/record_query.rs"] mod record_query;`
- `pub(crate) use record_query::{...};`
- `use record_query::load_runtime_ai_proposal_for_user;`
- `src/runtime/mutation/ai_proposal/record_query.rs`
- `src/runtime/mutation/ai_proposal.rs` 中由本批 record_query 迁移造成的删除

不得回改已 closeout 的 `static_check`、`source_governance_identity` 或 `event_lifecycle`，不得改 route facade、schema、AppState、frontend caller、runtime persistence owner 或 release transition guard。

---

## 验证计划

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

## 幻觉检查点

AI 声称 BE-001BF-02 完成时，必须说明当前只是 `runtime.mutation.ai_proposal.record_query` 抽离方案，仍为 `no code movement`；目标文件尚未创建，list/detail/loader 尚未迁移。下一步只能进入 BE-001BF-03 实际抽离。不得宣称 record_query 已抽离、approval review 已拆分、AppState/schema/frontend caller 已改变、release transition 已启动或 Rust backend 重构已完成。

---

## 验收标准

1. `201-runtime.mutation.ai_proposal.record_query抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 目标文件、父级 child 声明、双 handler re-export、loader helper import、`use super::*`、迁移清单与非目标已冻结。
3. 本批无 Rust 代码移动。
4. BE-001BF-03 只能移动 list/detail/read-through loader。
