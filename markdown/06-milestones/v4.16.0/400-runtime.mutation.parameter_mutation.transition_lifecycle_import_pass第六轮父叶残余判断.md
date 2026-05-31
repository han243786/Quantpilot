# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle_import_pass 第六轮父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EH-01
> 基线: `399-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass单叶closeout.md`
> 目标父叶: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle_import_pass`
> 代码动作: no code movement
> 下一步: BE-001EI-01 `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EH-01 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第六轮父叶残余判断 | 父叶递归分派 |
| 规范矩阵 | recursive residual judgment / staged explicit import pass | 继续细拆 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` | 选择 parent facade 子叶 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` | 选择下一子叶 |

---

## 当前残余队列

```text
runtime.mutation.parameter_mutation.transition_lifecycle_import_pass
transition_lifecycle_import_pass sixth_parent_residual_judgment
runtime.mutation.parameter_mutation.transition_lifecycle_import_pass stop_split: false
transition_lifecycle_parent_facade_import_pass_selected
single_file_transition_lifecycle_parent_facade_import_pass
remaining_parent_import_bridge_14
remaining_mutation_import_bridge_12
remaining_parameter_mutation_import_bridge_2
remaining_transition_lifecycle_import_bridge_1
old_three_leaf_pause_target_cancelled
```

当前 transition_lifecycle residual 为 1 文件:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
```

---

## 选择判断

本轮选择:

```text
runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass
```

选择理由:

1. `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 是 transition lifecycle 子树最后一个 parent wildcard residual。
2. activation flow、rollback flow、boundary safety、rollback record identity、transition record persistence 与 activation snapshot side effect 均已完成各自 import pocket closeout。
3. parent facade 当前只承接 child module declaration、`pub(crate) use` re-export、内部 helper import 与 `validate_runtime_parameter_mutation_boundary` wrapper，适合单独建立 facade import pass。
4. 直接宣称 `transition_lifecycle_import_pass` 完成会跳过 parent facade 的等价基线与实际 import rewrite，不符合递归流程。
5. 本轮不改 Rust，只选择下一 staged explicit import pass。

---

## 不选择项

暂不选择父叶 closeout:

```text
runtime.mutation.parameter_mutation.transition_lifecycle_import_pass stop_split: true
```

原因: parent facade 仍有 `use super::*`，未建立 facade 输入面基线，也未证明 `RuntimeParameterMutationBoundary` / `StatusCode` 的显式输入面等价。

---

## 下一步边界

下一步只允许建立 BE-001EI-01 等价基线:

```text
BE-001EI-01
runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass
```

BE-001EI-01 不得直接改 Rust。

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

AI 声称 BE-001EH-01 完成时，必须说明:

1. 本批是 `no code movement`。
2. 父叶仍保持 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass stop_split: false`。
3. 当前 transition_lifecycle residual 为 1 文件。
4. 下一步只能进入 BE-001EI-01 `parent_facade_import_pass` 单子叶等价基线。
5. 旧三叶暂停目标保持取消: `old_three_leaf_pause_target_cancelled`。

不得宣称 parent facade import 已改写、transition_lifecycle_import_pass 已完成、parameter_mutation_import_pass 已完成、mutation_import_pass 已完成、parent_import_bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `400-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass第六轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶保持 `stop_split: false`。
3. 下一步固定为 BE-001EI-01 单子叶等价基线。
4. 不恢复旧三叶暂停目标。
5. Rust / 治理 / 全量树门禁均通过。
