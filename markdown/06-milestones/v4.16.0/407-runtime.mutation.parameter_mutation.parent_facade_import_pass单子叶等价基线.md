# v4.16.0 runtime.mutation.parameter_mutation.parent_facade_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EL-01
> 基线: `406-runtime.mutation.parameter_mutation_import_pass第三轮父叶残余判断.md`
> 目标子叶: `runtime.mutation.parameter_mutation.parent_facade_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.parent_facade_import_pass`
> 真实文件: `src/runtime/mutation/parameter_mutation.rs`
> 代码动作: no code movement
> 下一步: BE-001EL-02 `runtime.mutation.parameter_mutation.parent_facade_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EL-01 `runtime.mutation.parameter_mutation.parent_facade_import_pass` 单子叶等价基线 | staged explicit import pass |
| 规范矩阵 | parent facade import baseline / no code movement / explicit input freeze | 标准档位 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.parent_facade_import_pass` | 冻结 parent facade 输入面 |
| 模块树 | `runtime.mutation.parameter_mutation.parent_facade_import_pass` | 单文件 facade import pocket |

---

## 当前残余队列

```text
BE-001EL-01
BE-001EL-02
runtime.mutation.parameter_mutation.parent_facade_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.parent_facade_import_pass
parent_facade_import_pass baseline_frozen
single_file_parameter_mutation_parent_facade_import_pass
no code movement
remaining_parent_import_bridge_13
remaining_mutation_import_bridge_11
remaining_parameter_mutation_import_bridge_1
remaining_transition_lifecycle_import_bridge_0
old_three_leaf_pause_target_cancelled
```

本轮只冻结 `src/runtime/mutation/parameter_mutation.rs` 的当前输入面，不改 Rust。

---

## 白箱节点冻结

当前 parent facade 承担四类职责:

1. child module declaration: 声明 `proposal_creation`、`record_query`、`transition_lifecycle`。
2. public facade re-export: `create_runtime_parameter_mutation`、`get_runtime_parameter_mutation_detail`、`list_runtime_parameter_mutations`、`activate_runtime_parameter_mutation`、`rollback_runtime_parameter_mutation`。
3. parent-private helper alias: `validate_runtime_parameter_mutation_boundary` 从 `transition_lifecycle` 引入，供 child `proposal_creation` 通过父级白箱调用。
4. facade handoff: 保持上层 `runtime::mod.rs` 对 mutation parameter handlers 的统一 re-export 面。

当前残余 import:

```rust
use super::*;
```

当前 child module declaration:

```rust
#[path = "parameter_mutation/proposal_creation.rs"]
mod proposal_creation;
#[path = "parameter_mutation/record_query.rs"]
mod record_query;
#[path = "parameter_mutation/transition_lifecycle.rs"]
mod transition_lifecycle;
```

当前 public facade re-export:

```rust
pub(crate) use proposal_creation::create_runtime_parameter_mutation;
pub(crate) use record_query::{
    get_runtime_parameter_mutation_detail, list_runtime_parameter_mutations,
};
pub(crate) use transition_lifecycle::{
    activate_runtime_parameter_mutation, rollback_runtime_parameter_mutation,
};
```

当前 parent-private helper alias:

```rust
use transition_lifecycle::validate_runtime_parameter_mutation_boundary;
```

---

## 预期显式输入面

BE-001EL-02 只能围绕下列输入面建立方案:

```text
empty_explicit_parent_import_surface
no_replacement_import_expected
```

预期 BE-001EL-03 只允许删除 `use super::*`。如果编译发现隐藏输入，必须回到方案文档补充显式 import，不得扩大到 child rewrite。

等价约束:

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

## 不进入范围

本轮不处理:

1. 不修改 `src/runtime/mutation/parameter_mutation.rs`。
2. 不改任何 child file。
3. 不改 public handler re-export。
4. 不改 `validate_runtime_parameter_mutation_boundary` 的父级白箱 alias。
5. 不继续拆 `proposal_creation_import_pass`、`record_query_import_pass` 或 `transition_lifecycle_import_pass`。
6. 不宣称 `parameter_mutation_import_pass stop_split: true`。
7. 不启动发布过渡连接。

---

## 下一步边界

下一步只允许进入 BE-001EL-02 抽离方案:

```text
BE-001EL-02
runtime.mutation.parameter_mutation.parent_facade_import_pass
single_file_parameter_mutation_parent_facade_import_pass
```

BE-001EL-02 仍不得直接修改 Rust；实际 import rewrite 只能在 BE-001EL-03 发生。

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

AI 声称 BE-001EL-01 完成时，必须说明:

1. 本批是 `no code movement`。
2. 只建立 `parent_facade_import_pass baseline_frozen`。
3. 当前真实文件仍是 `src/runtime/mutation/parameter_mutation.rs`。
4. `use super::*` 仍未改写。
5. 下一步只能进入 BE-001EL-02 抽离方案。
6. 旧三叶暂停目标保持取消: `old_three_leaf_pause_target_cancelled`。

不得声称 parent facade import 已改写、parameter_mutation_import_pass 已完成、mutation_import_pass 已完成、parent_import_bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `407-runtime.mutation.parameter_mutation.parent_facade_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001EL-01 只冻结 baseline，不改 Rust。
3. `empty_explicit_parent_import_surface` 与 `no_replacement_import_expected` 被记录。
4. 下一步固定为 BE-001EL-02 抽离方案。
5. Rust / 治理 / 全量树门禁均通过。
