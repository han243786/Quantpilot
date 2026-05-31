# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EI-02
> 基线: `401-runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass`
> 真实文件: `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`
> 代码动作: no code movement
> 下一步: BE-001EI-03 `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EI-02 `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 抽离方案 | staged explicit import pass |
| 规范矩阵 | parent facade import plan / no code movement / single-file rewrite guard | 标准档位 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` | 固定 BE-001EI-03 改动边界 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` | 单文件 facade import pocket |

---

## 方案冻结

```text
BE-001EI-02
BE-001EI-03
runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass
parent_facade_import_pass plan_frozen
single_file_transition_lifecycle_parent_facade_import_pass
no code movement
old_three_leaf_pause_target_cancelled
```

BE-001EI-03 只允许改写一个文件:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
```

当前 residual:

```text
remaining_parent_import_bridge_14
remaining_mutation_import_bridge_12
remaining_parameter_mutation_import_bridge_2
remaining_transition_lifecycle_import_bridge_1
```

预期 BE-001EI-03 后 residual:

```text
expected_remaining_parent_import_bridge_13
expected_remaining_mutation_import_bridge_11
expected_remaining_parameter_mutation_import_bridge_1
expected_remaining_transition_lifecycle_import_bridge_0
```

---

## 实施方案

BE-001EI-03 只能把当前 parent wildcard import:

```rust
use super::*;
```

替换为显式输入面:

```rust
use super::mutation_event_contract;
use crate::RuntimeParameterMutationBoundary;
use axum::http::StatusCode;
```

允许 rustfmt 调整 import 顺序，但语义输入面不得扩大。

补充约束: `transition_record_persistence` 通过 parent facade 命名空间调用 `mutation_event_contract`，因此 BE-001EI-03 必须把它作为 parent-private helper import 显式保留，不得让 child 横向改连。

---

## 等价保护栏

BE-001EI-03 必须保持:

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

具体不得改:

1. 不改 `#[path = ...] mod ...` child module declaration。
2. 不改 `pub(crate) use activation_flow::activate_runtime_parameter_mutation`。
3. 不改 `pub(crate) use rollback_flow::rollback_runtime_parameter_mutation`。
4. 不改 `auto_snapshot_on_activation` 等 parent-private helper import。
5. 不改 `validate_runtime_parameter_mutation_boundary` 函数体和可见性。

---

## 回退点

若 BE-001EI-03 出现编译错误，唯一允许回退为:

```rust
use super::*;
```

回退后必须重新建立基线，不得把缺失类型从 sibling child 横向引入。

---

## 下一步边界

下一步只允许进入 BE-001EI-03 实际抽离记录:

```text
BE-001EI-03
parent_facade_import_pass extraction_ready
single_file_transition_lifecycle_parent_facade_import_pass
```

BE-001EI-03 不得修改方案未列出的文件；也不得宣称 `transition_lifecycle_import_pass stop_split: true`，该判断必须留到 BE-001EI-04 单叶 closeout 和后续父叶残余判断之后。

---

## 验证要求

BE-001EI-03 提交前至少执行:

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

AI 声称 BE-001EI-02 完成时，必须说明:

1. 本批是 `no code movement`。
2. 只冻结 BE-001EI-03 的单文件 import rewrite 方案。
3. 目标文件仍是 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`。
4. 实际 Rust import 尚未改写。
5. 下一步只能进入 BE-001EI-03 实际抽离记录。
6. 旧三叶暂停目标保持取消: `old_three_leaf_pause_target_cancelled`。

不得声称 parent facade import 已改写、transition_lifecycle_import_pass 已完成、parameter_mutation_import_pass 已完成、mutation_import_pass 已完成、parent_import_bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `402-runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 本批不改 Rust。
3. BE-001EI-03 的单文件 rewrite、预期输入面和回退点被固定。
4. 下一步固定为 BE-001EI-03 实际抽离记录。
5. Rust / 治理 / 全量树门禁均通过。
