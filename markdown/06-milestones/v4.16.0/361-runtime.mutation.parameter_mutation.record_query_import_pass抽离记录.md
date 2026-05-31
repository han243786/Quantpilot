# v4.16.0 runtime.mutation.parameter_mutation.record_query_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DR-03
> 基准: `360-runtime.mutation.parameter_mutation.record_query_import_pass抽离方案.md`
> 目标子叶: `runtime.mutation.parameter_mutation.record_query_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.record_query_import_pass`
> 代码动作: actual Rust import rewrite
> 下一步: BE-001DR-04 `runtime.mutation.parameter_mutation.record_query_import_pass` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DR-03 `runtime.mutation.parameter_mutation.record_query_import_pass` 实际抽离 | 单文件实施 |
| 规范矩阵 | explicit import pass、read-only handler equivalence、parent bridge residual accounting | parent wildcard 清理 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.record_query_import_pass` | record query 白箱实际收敛 |
| 模块树 | `runtime.mutation.parameter_mutation.record_query_import_pass` | 实际抽离记录 |

---

## 实际改动

本批只改写:

```text
src/runtime/mutation/parameter_mutation/record_query.rs
single_file_record_query_import_rewrite
```

删除:

```rust
use super::*;
```

新增显式 import:

```rust
use crate::{
    auth, io_error, list_runtime_parameter_mutation_records,
    load_runtime_parameter_mutation_record, paginate,
    runtime::{clean_optional_filter, RuntimeParameterMutationListQuery},
    AppState, PaginatedResponse, PaginationQuery, RuntimeParameterMutationRecord,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
```

函数体、handler signature、visibility、排序、filter、pagination、state cache fallback、store fallback 和 error mapping 均未改动。

```text
function_bodies_unchanged
actual_parent_import_bridge_22_to_21
actual_mutation_import_bridge_20_to_19
actual_record_query_import_bridge_1_to_0
old_three_leaf_pause_target_cancelled
```

---

## 实施校正

BE-001DR-02 的预期 import 草案把 `clean_optional_filter` 与 `RuntimeParameterMutationListQuery` 当作 crate root 输入。实际编译确认它们是 runtime 父级白箱输入，因此本批采用:

```rust
runtime::{clean_optional_filter, RuntimeParameterMutationListQuery}
```

这仍是显式 import，不恢复 `super::*`，也没有新增 sibling horizontal link。该经验应在后续 `parameter_mutation` child import pass 中复用: 如果输入属于 runtime parent 白箱，优先用 `crate::runtime::{...}` 明确指向父级白箱，不要回退到 wildcard。

---

## 等价核查

保持不变:

1. `list_runtime_parameter_mutations` handler signature 与返回类型。
2. `get_runtime_parameter_mutation_detail` handler signature 与返回类型。
3. list 的 mutation store 读取、`source_kind` / `source_id` filter、倒序排序和 pagination。
4. detail 的 `auth::scoped_key` cache-first lookup 与 store fallback。
5. `RuntimeParameterMutationListQuery`、`PaginatedResponse` 和 `RuntimeParameterMutationRecord` schema。
6. proposal creation、activation、rollback、safe window、snapshot、parent facade 与 AI proposal 均未触碰。
7. release transition 未启动，未新增 sibling horizontal link。

---

## 当前残余

本批后 parent bridge 剩余:

```text
root 1
run 0
backtest 0
mutation 19
test-only 1
total 21
remaining_parent_import_bridge_21
remaining_mutation_import_bridge_19
```

剩余 `parameter_mutation` pocket:

```text
src/runtime/mutation/parameter_mutation.rs
src/runtime/mutation/parameter_mutation/proposal_creation.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs
```

---

## 验证要求

本批提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_mutation
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001DR-03 完成时，必须说明:

1. 本批只改写 `src/runtime/mutation/parameter_mutation/record_query.rs` 顶部 import。
2. `record_query.rs` 已无 `use super::*` / `super::` residual。
3. parent bridge 剩余已从 total 22 / mutation 20 降到 total 21 / mutation 19。
4. 函数体和 handler signature 未改。
5. 下一步只能进入 BE-001DR-04 单叶 closeout。
6. `proposal_creation`、`transition_lifecycle`、parent facade、`ai_proposal`、root bridge 与 test-only run_guard 尚未处理。
7. release transition 未启动，未新增 sibling horizontal link。
8. 旧的三叶暂停目标仍为取消状态。

不得宣称 parameter mutation import 已完成、mutation import 已完成、parent import bridge 已完全清除、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `src/runtime/mutation/parameter_mutation/record_query.rs` 已删除 parent wildcard import，并改为显式 import。
2. `361-runtime.mutation.parameter_mutation.record_query_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
3. parent bridge residual 计数更新为 total 21 / mutation 19。
4. 下一步固定为 BE-001DR-04 单叶 closeout。
5. Rust / 治理 / 全量树门禁均通过。
