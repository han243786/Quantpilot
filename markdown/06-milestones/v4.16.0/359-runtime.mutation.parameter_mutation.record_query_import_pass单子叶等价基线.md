# v4.16.0 runtime.mutation.parameter_mutation.record_query_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DR-01
> 基准: `358-runtime.mutation.parameter_mutation_import_pass抽离方案.md`
> 目标子叶: `runtime.mutation.parameter_mutation.record_query_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.record_query_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DR-02 `runtime.mutation.parameter_mutation.record_query_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DR-01 `runtime.mutation.parameter_mutation.record_query_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | parent import bridge、single-file import pass、read-only handler boundary | record query 输入面冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.record_query_import_pass` | parameter mutation record query 白箱 |
| 模块树 | `runtime.mutation.parameter_mutation.record_query_import_pass` | 新基线 |

---

## 当前事实

目标文件:

```text
src/runtime/mutation/parameter_mutation/record_query.rs
```

文件当前顶部仍存在:

```rust
use super::*;
```

当前 parent bridge 剩余:

```text
root 1
run 0
backtest 0
mutation 20
test-only 1
total 22
remaining_parent_import_bridge_22
remaining_mutation_import_bridge_20
record_query_import_pass baseline_frozen
old_three_leaf_pause_target_cancelled
```

本基线不改写 Rust import，只冻结下一步单文件方案的输入面。

---

## 白箱 public 面

本子叶只冻结两个读路径 handler:

```text
list_runtime_parameter_mutations
get_runtime_parameter_mutation_detail
```

其中:

1. `list_runtime_parameter_mutations` 从 mutation store 读取记录，按 `source_kind` 与 `source_id` 过滤，按 `created_at_ms` / `proposal_id` 倒序排序，再分页返回。
2. `get_runtime_parameter_mutation_detail` 使用 `auth::scoped_key` 优先读取内存 cache，未命中时回退到 mutation store。

---

## 当前隐式输入面

BE-001DR-02 需要复核以下显式输入面，BE-001DR-03 才允许实际改写:

```text
auth
clean_optional_filter
io_error
load_runtime_parameter_mutation_record
list_runtime_parameter_mutation_records
paginate
AppState
PaginatedResponse
PaginationQuery
RuntimeParameterMutationListQuery
RuntimeParameterMutationRecord
Path
Query
State
StatusCode
Json
String
```

预期显式 import 形状:

```rust
use crate::{
    auth, clean_optional_filter, io_error, load_runtime_parameter_mutation_record,
    list_runtime_parameter_mutation_records, paginate, AppState, PaginatedResponse,
    PaginationQuery, RuntimeParameterMutationListQuery, RuntimeParameterMutationRecord,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
```

实际写法以编译和 `cargo fmt` 为准；不得恢复 wildcard import。

---

## 等价边界

后续实际 import pass 必须保持:

1. 两个 handler 的签名、可见性和返回类型不变。
2. list 的 store 读取、`io_error` mapping、`source_kind` / `source_id` filter、排序和 pagination 不变。
3. detail 的 scoped memory cache 优先和 store fallback 不变。
4. 不改变 `RuntimeParameterMutationListQuery`、`PaginatedResponse` 或 `RuntimeParameterMutationRecord` schema。
5. 不改变 auth scope、mutation store 路径、state lock、persistence owner 或 error code。
6. 不触碰 proposal creation、activation、rollback、safe window、snapshot、parent facade 或 AI proposal。
7. 不新增 sibling horizontal link，不启动 release transition。

---

## 预期收敛

若 BE-001DR-03 成功:

```text
expected_parent_import_bridge_22_to_21
expected_mutation_import_bridge_20_to_19
expected_record_query_import_bridge_1_to_0
```

本基线不直接实现该收敛。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不直接改写 `src/runtime/mutation/parameter_mutation/record_query.rs` import。
- 本批不处理 `proposal_creation.rs`。
- 本批不处理 `transition_lifecycle.rs` 或其 6 个 child。
- 本批不处理 `parameter_mutation.rs` parent facade。
- 本批不处理 `ai_proposal`、`src/runtime/mod.rs` 或 test-only `src/runtime/run_guard.rs`。
- 本批不迁移 state owner、persistence owner、schema owner、frontend caller 或 test asset。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。

---

## 验证要求

本批为 `no code movement` 基线，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

后续实际 import pass 至少补跑:

```powershell
cargo test -p quantpilot --test api_mutation
```

---

## 幻觉检查点

AI 声称 BE-001DR-01 完成时，必须说明:

1. 本批次是 `no code movement` 单子叶等价基线。
2. 目标文件只有 `src/runtime/mutation/parameter_mutation/record_query.rs`。
3. `use super::*` 尚未改写。
4. 当前 parent bridge 剩余仍为 root 1 / run 0 / backtest 0 / mutation 20 / test-only 1 / total 22。
5. 下一步只能进入 BE-001DR-02 `runtime.mutation.parameter_mutation.record_query_import_pass` 抽离方案。
6. `proposal_creation`、`transition_lifecycle`、parent facade、`ai_proposal`、root bridge 与 test-only run_guard 尚未处理。
7. release transition 未启动，未新增 sibling horizontal link。
8. 旧的三叶暂停目标仍为取消状态。

不得宣称 record_query import 已改写、parameter mutation import 已完成、mutation import 已完成、parent import bridge 已完全清除、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `359-runtime.mutation.parameter_mutation.record_query_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 基线冻结 `record_query.rs` 两个读路径 handler、当前 `use super::*` 和预期显式输入面。
3. 下一步固定为 BE-001DR-02 `runtime.mutation.parameter_mutation.record_query_import_pass` 抽离方案。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
