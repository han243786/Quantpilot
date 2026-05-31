# v4.16.0 runtime.mutation.parameter_mutation.parent_facade_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EL-03
> 基线: `408-runtime.mutation.parameter_mutation.parent_facade_import_pass抽离方案.md`
> 目标子叶: `runtime.mutation.parameter_mutation.parent_facade_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.parent_facade_import_pass`
> 真实文件: `src/runtime/mutation/parameter_mutation.rs`
> 代码动作: single file import rewrite
> 下一步: BE-001EL-04 `runtime.mutation.parameter_mutation.parent_facade_import_pass` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EL-03 `runtime.mutation.parameter_mutation.parent_facade_import_pass` 实际抽离记录 | staged explicit import pass |
| 规范矩阵 | hidden parent input detection / explicit parent import / no child rewrite | 标准档位 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.parent_facade_import_pass` | 清理 parent wildcard import |
| 模块树 | `runtime.mutation.parameter_mutation.parent_facade_import_pass` | 单文件 facade import pocket |

---

## 实际改动

```text
BE-001EL-03
BE-001EL-04
runtime.mutation.parameter_mutation.parent_facade_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.parent_facade_import_pass
parent_facade_import_pass extraction_complete
parent_facade_import_pass_closeout_ready
single_file_parameter_mutation_parent_facade_import_pass
delete_parameter_mutation_parent_wildcard_import_complete
hidden_parent_input_detected_by_cargo_check
mutation_event_contract_explicit_parent_import
old_three_leaf_pause_target_cancelled
```

`cargo check -p quantpilot` 在删除 `use super::*` 后抓到了隐藏父级输入:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
use super::mutation_event_contract;
```

因此 BE-001EL-03 最终将 parent wildcard import 收敛为显式父级输入:

```diff
-use super::*;
+use super::mutation_event_contract;
```

---

## 等价保持

本批只改写 `src/runtime/mutation/parameter_mutation.rs` 顶部 import。保持不变:

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

未发生:

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

## 残余变化

当前 `parameter_mutation` import pass residual 已清零:

```text
remaining_parent_import_bridge_12
remaining_mutation_import_bridge_10
remaining_parameter_mutation_import_bridge_0
remaining_transition_lifecycle_import_bridge_0
```

BE-001EL-03 不直接设置 `runtime.mutation.parameter_mutation.parent_facade_import_pass stop_split: true`，必须先进入 BE-001EL-04 单叶 closeout。

---

## 下一步边界

下一步只允许进入 BE-001EL-04 单叶 closeout:

```text
BE-001EL-04
runtime.mutation.parameter_mutation.parent_facade_import_pass
single_file_parameter_mutation_parent_facade_import_pass
```

BE-001EL-04 只能判断本 import pocket 是否继续细拆 module declaration / re-export / private helper alias，不得改 Rust。

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

AI 声称 BE-001EL-03 完成时，必须说明:

1. 本批只改写 `src/runtime/mutation/parameter_mutation.rs` 顶部 import。
2. `use super::*` 已被移除。
3. `mutation_event_contract` 是编译探针发现的显式父级输入。
4. child module declaration、public re-export、private helper alias 与 child files 均未改。
5. 下一步只能进入 BE-001EL-04 单叶 closeout。
6. 旧三叶暂停目标保持取消: `old_three_leaf_pause_target_cancelled`。

不得声称 parent facade import 已 closeout、parameter_mutation_import_pass 已完成、mutation_import_pass 已完成、parent_import_bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `409-runtime.mutation.parameter_mutation.parent_facade_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/mutation/parameter_mutation.rs` 不再包含 `use super::*`。
3. `use super::mutation_event_contract;` 被记录为显式父级输入。
4. 下一步固定为 BE-001EL-04 单叶 closeout。
5. Rust / 治理 / 全量树门禁均通过。
