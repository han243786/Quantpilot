# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EI-03
> 基线: `402-runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass抽离方案.md`
> 目标子叶: `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass`
> 真实文件: `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`
> 代码动作: single-file import rewrite
> 下一步: BE-001EI-04 `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EI-03 `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 实际抽离记录 | staged explicit import pass |
| 规范矩阵 | parent facade import extraction / single-file rewrite / equivalence gate | 标准档位 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` | 清理 parent facade wildcard import |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` | 单文件 facade import pocket |

---

## 实际改动

```text
BE-001EI-03
runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass
parent_facade_import_pass extraction_complete
single_file_transition_lifecycle_parent_facade_import_pass
old_three_leaf_pause_target_cancelled
```

本批只改写一个 Rust 文件:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
```

移除:

```rust
use super::*;
```

替换为:

```rust
use super::mutation_event_contract;
use crate::RuntimeParameterMutationBoundary;
use axum::http::StatusCode;
```

`mutation_event_contract` 是 `transition_record_persistence` 通过 parent facade 命名空间使用的 parent-private helper import。本批将其显式保留，避免 child 横向改连。

---

## 等价保持

本批未改:

```text
no_function_body_change
no_visibility_change
no_child_module_rewrite
no_reexport_rewrite
no_helper_import_rewrite
no_activation_flow_rewrite
no_rollback_flow_rewrite
no_boundary_safety_rewrite
no_transition_record_persistence_rewrite
no_rollback_record_identity_rewrite
no_sibling_horizontal_link
no_release_transition
```

保持不变的白箱节点:

1. child module declaration: `activation_flow`、`activation_snapshot_side_effect`、`boundary_safety`、`rollback_flow`、`rollback_record_identity`、`transition_record_persistence`。
2. public re-export: `pub(crate) use activation_flow::activate_runtime_parameter_mutation`。
3. public re-export: `pub(crate) use rollback_flow::rollback_runtime_parameter_mutation`。
4. parent-private helper import: `auto_snapshot_on_activation`、`evaluate_runtime_parameter_mutation_safe_window`、`resolve_runtime_parameter_mutation_boundary`、`runtime_parameter_mutation_rollback_record_id`、`mutation_lifecycle_entry`、`persist_runtime_parameter_mutation_transition`。
5. boundary wrapper: `validate_runtime_parameter_mutation_boundary` 函数体与可见性未改。

---

## residual 更新

本批前:

```text
remaining_parent_import_bridge_14
remaining_mutation_import_bridge_12
remaining_parameter_mutation_import_bridge_2
remaining_transition_lifecycle_import_bridge_1
```

本批后:

```text
remaining_parent_import_bridge_13
remaining_mutation_import_bridge_11
remaining_parameter_mutation_import_bridge_1
remaining_transition_lifecycle_import_bridge_0
```

---

## 下一步边界

下一步只允许进入 BE-001EI-04 单叶 closeout:

```text
BE-001EI-04
runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass
parent_facade_import_pass_closeout_ready
```

BE-001EI-04 只判断本 import pocket 是否继续细拆，不得直接宣称 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass stop_split: true`；父叶 stop_split 必须在后续父叶残余判断中处理。

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

AI 声称 BE-001EI-03 完成时，必须说明:

1. 本批只改 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 顶部 import。
2. `use super::*` 已从该文件移除。
3. 函数体、可见性、child module declaration、re-export、helper import 和 sibling 均未改。
4. transition_lifecycle residual 已降为 0。
5. 下一步只能进入 BE-001EI-04 单叶 closeout。
6. 旧三叶暂停目标保持取消: `old_three_leaf_pause_target_cancelled`。

不得声称 transition_lifecycle_import_pass 已完成、parameter_mutation_import_pass 已完成、mutation_import_pass 已完成、parent_import_bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `403-runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 已使用显式输入面。
3. 本批只改一个 Rust 文件的顶部 import。
4. 下一步固定为 BE-001EI-04 单叶 closeout。
5. Rust / 治理 / 全量树门禁均通过。
