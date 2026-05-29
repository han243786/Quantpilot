# v4.16.0 runtime.mutation.parameter_mutation.record_query 抽离方案
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AW-02  
> 基线: `176-runtime.mutation.parameter_mutation.record_query单子叶等价基线.md`、`src/runtime/mutation/parameter_mutation.rs`、`tests/api_mutation.rs`  
> 判定: `runtime.mutation.parameter_mutation.record_query` 抽离方案已建立；当前仍为 `no code movement`，只固定 BE-001AW-03 的目标文件、父级声明、双 handler re-export、迁移清单、非目标和回退点。下一步只能进入 BE-001AW-03 实际抽离。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AW-02 record_query 抽离方案 | 方案冻结 |
| 规范矩阵 | 父子通信、双 public handler re-export、read model 边界 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.record_query` | 子叶抽离路径 |
| 模块树 | `runtime.mutation.parameter_mutation.record_query` | 白箱方案 |

---

## 目标文件与父级声明

BE-001AW-03 只允许创建一个目标文件:

```text
src/runtime/mutation/parameter_mutation/record_query.rs
```

父级 `src/runtime/mutation/parameter_mutation.rs` 只允许新增以下 child 声明:

```rust
#[path = "parameter_mutation/record_query.rs"]
mod record_query;
```

父级只允许新增以下 public handler 出口:

```rust
pub(crate) use record_query::{
    get_runtime_parameter_mutation_detail, list_runtime_parameter_mutations,
};
```

child 文件必须以父级白箱输入为唯一来源:

```rust
use super::*;
```

---

## BE-001AW-03 迁移清单

只允许迁移:

- `pub(crate) async fn list_runtime_parameter_mutations`
- `pub(crate) async fn get_runtime_parameter_mutation_detail`

`list_runtime_parameter_mutations` 签名必须保持:

```rust
pub(crate) async fn list_runtime_parameter_mutations(
    State(state): State<AppState>,
    Query(query): Query<RuntimeParameterMutationListQuery>,
) -> Result<Json<PaginatedResponse<RuntimeParameterMutationRecord>>, (StatusCode, String)>
```

`get_runtime_parameter_mutation_detail` 签名必须保持:

```rust
pub(crate) async fn get_runtime_parameter_mutation_detail(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
) -> Result<Json<RuntimeParameterMutationRecord>, (StatusCode, String)>
```

---

## list 等价约束

必须保持:

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

排序必须继续为 `created_at_ms` 倒序，随后 `proposal_id` 倒序。

---

## detail 等价约束

必须保持:

- `auth::scoped_key`
- `state.parameter_mutations`
- `cloned`
- `load_runtime_parameter_mutation_record`
- `state.mutation_store_dir.as_ref()`
- `map(Json)`

detail 必须保持 in-memory scoped lookup 优先，miss 后再 persistence fallback。

---

## 非目标

本批不得迁移:

- `create_runtime_parameter_mutation`
- `activate_runtime_parameter_mutation`
- `rollback_runtime_parameter_mutation`
- `AI proposal`
- `approval review`
- `AppState`
- `schema`
- `frontend caller`
- release transition guard

不得改变 route facade、runtime persistence owner、response mapping owner、`RuntimeParameterMutationRecord` schema、pagination schema、frontend route 或发布过渡连接。

---

## 回退点

若 BE-001AW-03 编译或等价检查失败，只允许回退本批新增的:

- `#[path = "parameter_mutation/record_query.rs"] mod record_query;`
- `pub(crate) use record_query::{...};`
- `src/runtime/mutation/parameter_mutation/record_query.rs`

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

AI 声称 BE-001AW-02 完成时，必须说明当前只是 `record_query` 抽离方案，仍为 `no code movement`；目标文件尚未创建，list/detail handler 尚未迁移。下一步只能进入 BE-001AW-03 实际抽离。不得宣称 record_query 已抽离、`runtime.mutation.parameter_mutation` 父叶已经完成、AI proposal/approval 已拆分、AppState/schema/frontend caller 已改变或发布过渡已启动。

---

## 验收标准

1. `177-runtime.mutation.parameter_mutation.record_query抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 目标文件、父级 path attribute、双 handler re-export、`use super::*`、迁移清单与非目标已冻结。
3. 本批无 Rust 代码移动。
4. BE-001AW-03 只能移动 list/detail handler。
