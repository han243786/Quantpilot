# v4.16.0 runtime.mutation.parameter_mutation.parent_facade_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EL-02
> 基线: `407-runtime.mutation.parameter_mutation.parent_facade_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.mutation.parameter_mutation.parent_facade_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.parent_facade_import_pass`
> 真实文件: `src/runtime/mutation/parameter_mutation.rs`
> 代码动作: no code movement
> 下一步: BE-001EL-03 `runtime.mutation.parameter_mutation.parent_facade_import_pass` 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EL-02 `runtime.mutation.parameter_mutation.parent_facade_import_pass` 抽离方案 | staged explicit import pass |
| 规范矩阵 | parent facade import plan / single file rewrite / no release transition | 标准档位 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.parent_facade_import_pass` | 固定 BE-001EL-03 改动边界 |
| 模块树 | `runtime.mutation.parameter_mutation.parent_facade_import_pass` | 单文件 facade import pocket |

---

## 方案锁定

```text
BE-001EL-02
BE-001EL-03
runtime.mutation.parameter_mutation.parent_facade_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.parent_facade_import_pass
parent_facade_import_pass plan_frozen
parent_facade_import_pass extraction_ready
single_file_parameter_mutation_parent_facade_import_pass
delete_parameter_mutation_parent_wildcard_import_only
empty_explicit_parent_import_surface
no_replacement_import_expected
no code movement
old_three_leaf_pause_target_cancelled
```

BE-001EL-03 只允许改写一个文件:

```text
src/runtime/mutation/parameter_mutation.rs
```

唯一允许的 Rust 改动:

```diff
-use super::*;
```

不新增替代 import:

```text
empty_explicit_parent_import_surface
no_replacement_import_expected
```

---

## 等价边界

必须保持不变:

```text
proposal_creation child module declaration
record_query child module declaration
transition_lifecycle child module declaration
create_runtime_parameter_mutation re-export
get_runtime_parameter_mutation_detail re-export
list_runtime_parameter_mutations re-export
validate_runtime_parameter_mutation_boundary private helper alias
activate_runtime_parameter_mutation re-export
rollback_runtime_parameter_mutation re-export
```

不得改写:

```text
no_function_body_change
no_visibility_change
no_child_module_rewrite
no_reexport_rewrite
no_private_helper_alias_rewrite
no_proposal_creation_rewrite
no_record_query_rewrite
no_transition_lifecycle_rewrite
no_sibling_horizontal_link
no_release_transition
```

---

## 预期残余变化

BE-001EL-03 通过后，预期 parent bridge residual:

```text
expected_remaining_parent_import_bridge_12
expected_remaining_mutation_import_bridge_10
expected_remaining_parameter_mutation_import_bridge_0
expected_remaining_transition_lifecycle_import_bridge_0
```

当前父叶仍不在本方案中 closeout；BE-001EL-03 完成后必须先进入 BE-001EL-04 单叶 closeout。

---

## 禁止项

本方案明确禁止:

1. 不修改 `proposal_creation.rs`、`record_query.rs` 或 `transition_lifecycle.rs`。
2. 不修改 `src/runtime/mod.rs` 的 re-export 面。
3. 不把 `validate_runtime_parameter_mutation_boundary` 改成 public re-export。
4. 不删除或重命名 child module。
5. 不移动 handler、state owner、schema、storage 或 frontend caller。
6. 不启动发布过渡连接。

---

## 下一步边界

下一步只允许进入 BE-001EL-03 实际抽离记录:

```text
BE-001EL-03
runtime.mutation.parameter_mutation.parent_facade_import_pass
single_file_parameter_mutation_parent_facade_import_pass
delete_parameter_mutation_parent_wildcard_import_only
```

BE-001EL-03 完成后必须先验证 Rust 编译与 `api_mutation`，再进入 BE-001EL-04 单叶 closeout。

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

AI 声称 BE-001EL-02 完成时，必须说明:

1. 本批是 `no code movement`。
2. 只建立 `parent_facade_import_pass plan_frozen`。
3. BE-001EL-03 只能删除 `src/runtime/mutation/parameter_mutation.rs` 的 `use super::*`。
4. 不新增替代 import。
5. 下一步只能进入 BE-001EL-03 实际抽离记录。
6. 旧三叶暂停目标保持取消: `old_three_leaf_pause_target_cancelled`。

不得声称 parent facade import 已改写、parameter_mutation_import_pass 已完成、mutation_import_pass 已完成、parent_import_bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `408-runtime.mutation.parameter_mutation.parent_facade_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001EL-02 只冻结方案，不改 Rust。
3. BE-001EL-03 改动边界固定为单文件删除 parent wildcard import。
4. 下一步固定为 BE-001EL-03 实际抽离记录。
5. Rust / 治理 / 全量树门禁均通过。
