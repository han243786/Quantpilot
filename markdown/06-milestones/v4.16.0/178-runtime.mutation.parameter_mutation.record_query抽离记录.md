# v4.16.0 runtime.mutation.parameter_mutation.record_query 抽离记录
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AW-03  
> 方案: `177-runtime.mutation.parameter_mutation.record_query抽离方案.md`  
> 判定: `runtime.mutation.parameter_mutation.record_query` 第一轮实际抽离已完成；`list_runtime_parameter_mutations` 与 `get_runtime_parameter_mutation_detail` 已迁入 `src/runtime/mutation/parameter_mutation/record_query.rs`，父级通过 path-attributed child 和双 handler re-export 保持调用面。下一步只能进入 BE-001AW-04 单叶 closeout。  
> 代码动作: moved list/detail handlers

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AW-03 record_query 实际抽离 | 代码抽离 |
| 规范矩阵 | 父子通信、read model 等价、public handler re-export | 执行 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.record_query` | 子叶落位 |
| 模块树 | `runtime.mutation.parameter_mutation.record_query` | 白箱更新 |

---

## 实际文件变更

新增:

```text
src/runtime/mutation/parameter_mutation/record_query.rs
```

父级新增 child 声明:

```rust
#[path = "parameter_mutation/record_query.rs"]
mod record_query;
```

父级新增兼容出口:

```rust
pub(crate) use record_query::{
    get_runtime_parameter_mutation_detail, list_runtime_parameter_mutations,
};
```

child 只通过父级白箱输入取依赖:

```rust
use super::*;
```

---

## 已迁移清单

已从 `src/runtime/mutation/parameter_mutation.rs` 迁入 child:

- `pub(crate) async fn list_runtime_parameter_mutations`
- `pub(crate) async fn get_runtime_parameter_mutation_detail`

---

## list 等价保持

已保持:

- `State<AppState>`
- `Query<RuntimeParameterMutationListQuery>`
- `Result<Json<PaginatedResponse<RuntimeParameterMutationRecord>>, (StatusCode, String)>`
- `list_runtime_parameter_mutation_records`
- `state.mutation_store_dir`
- `io_error`
- `source_kind`
- `clean_optional_filter`
- `source_id`
- `created_at_ms`
- `proposal_id`
- `PaginationQuery`
- `paginate`

排序仍为 `created_at_ms` 倒序，随后 `proposal_id` 倒序。

---

## detail 等价保持

已保持:

- `auth::UserId`
- `State<AppState>`
- `Path<String>`
- `Result<Json<RuntimeParameterMutationRecord>, (StatusCode, String)>`
- `auth::scoped_key`
- `state.parameter_mutations`
- `cloned`
- `load_runtime_parameter_mutation_record`
- `state.mutation_store_dir.as_ref()`
- `map(Json)`

detail 仍先读 in-memory scoped lookup，miss 后再走 persistence fallback。

---

## 未迁移边界

本批未迁移:

- `create_runtime_parameter_mutation`
- `activate_runtime_parameter_mutation`
- `rollback_runtime_parameter_mutation`
- `AI proposal`
- `approval review`
- `AppState`
- `schema`
- `frontend caller`
- release transition guard

route facade、runtime persistence owner、response mapping owner、`RuntimeParameterMutationRecord` schema、pagination schema、frontend route 和发布过渡连接均未改变。

---

## 回退点

若 BE-001AW-04 closeout 或后续验证失败，只允许回退本批新增的:

- `src/runtime/mutation/parameter_mutation/record_query.rs`
- `#[path = "parameter_mutation/record_query.rs"] mod record_query;`
- `pub(crate) use record_query::{...};`

不得回改已 closeout 的 `proposal_creation` 或 `transition_lifecycle`，不得改 route facade、schema、AppState、frontend caller 或 persistence owner。

---

## 验证计划

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001AW-03 完成时，必须说明当前只完成 `record_query` 第一轮实际抽离；list/detail handler 已迁入 child，但 `runtime.mutation.parameter_mutation.record_query` 尚未做单叶 closeout，`runtime.mutation.parameter_mutation` 父叶也尚未完成。不得宣称 AI proposal/approval 已拆分、AppState/schema/frontend caller 已改变或发布过渡已启动。

---

## 验收标准

1. `src/runtime/mutation/parameter_mutation/record_query.rs` 已创建并承接 list/detail handler。
2. 父级 `src/runtime/mutation/parameter_mutation.rs` 只保留 child declaration、handler re-export 和 sibling imports。
3. list/detail read model 行为保持等价。
4. 下一步只能进入 BE-001AW-04 单叶 closeout。
