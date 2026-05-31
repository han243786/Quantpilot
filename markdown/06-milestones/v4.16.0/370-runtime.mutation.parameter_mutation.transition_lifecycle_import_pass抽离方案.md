# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DV-02
> 基准: `369-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass`
> 判定: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass stop_split: false`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DW-01 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DV-02 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 抽离方案 | 方案固定 |
| 规范矩阵 | lifecycle import pass、分层 pocket、父子通信硬规则、等价风险分档 | 方案约束 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` | 下一 pocket 选择 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` | `stop_split: false` |

---

## 方案判定

本批拒绝同批改写 7 个 transition lifecycle residual 文件。

```text
runtime.mutation.parameter_mutation.transition_lifecycle_import_pass stop_split: false
reject_transition_lifecycle_bulk_rewrite_7_files
boundary_safety_import_pass_selected
single_file_boundary_safety_import_pass
old_three_leaf_pause_target_cancelled
```

理由:

1. `transition_lifecycle` 7 文件覆盖 activation、rollback、boundary safety、snapshot side effect、record id、transition persistence 多职责。
2. `activation_flow.rs` 与 `rollback_flow.rs` 是 public handler，涉及 async state、event append、metrics、record persistence 和 snapshot side effect，不应和 helper import 同批压缩风险。
3. `boundary_safety.rs` 是无 async side effect 的单文件 helper pocket，同时承载 proposal creation 通过父级白箱访问的 `validate_runtime_parameter_mutation_boundary`。
4. 先收束 `boundary_safety.rs` 可以明确 boundary helper 的显式输入面，再处理 activation / rollback flow。
5. 旧的三叶暂停目标保持取消；继续拆小 pocket 是干净递归，不是暂停目标回归。

---

## 下一 pocket

选择:

```text
BE-001DW-01
runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass
```

冻结文件:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
```

冻结 public / internal helper:

```text
validate_runtime_parameter_mutation_boundary
resolve_runtime_parameter_mutation_boundary
evaluate_runtime_parameter_mutation_safe_window
```

预期后续 BE-001DW-02 / BE-001DW-03 只允许改写:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
use super::* -> explicit imports
```

不得触碰:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs
src/runtime/mutation/parameter_mutation.rs
src/runtime/mutation/parameter_mutation/proposal_creation.rs
src/runtime/mutation/parameter_mutation/record_query.rs
src/runtime/mutation/ai_proposal/**
src/runtime/mod.rs
src/runtime/run_guard.rs
release transition
sibling horizontal link
```

---

## 预期 import 面

BE-001DW-03 的预期显式 import 面应限制为:

```rust
use crate::{
    json_bad_request, RuntimeParameterMutationBoundary,
    RuntimeParameterMutationSafeWindowSnapshot, RuntimeParameterMutationSafeWindowState,
};
use axum::http::StatusCode;
```

若编译发现还需要额外输入，只能在 `boundary_safety.rs` 自身 import 面内调整；不得通过 sibling 横向连接访问 activation / rollback flow。

---

## 等价约束

BE-001DW 后续不得改变:

1. `immediate` 拒绝语义与中文错误文案。
2. `next_cycle_start`、`manual_pause`、`sequence_cursor` 和 `sequence_cursor:<u64>` 接受规则。
3. `next_cycle_start` 解析为 `current_sequence_no + 2`。
4. `manual_pause` 解析为 `resolved_sequence_no: None`。
5. `sequence_cursor` 缺少 sequence 时的错误码和中文错误文案。
6. safe window reason code、message、retryable、retry_after_ms 与 snapshot 原样回填。
7. release transition 未启动，不新增 sibling horizontal link。

---

## 残余预期

BE-001DV-02 本身不改 Rust，残余不变化:

```text
remaining_parent_import_bridge_20
remaining_mutation_import_bridge_18
remaining_parameter_mutation_import_bridge_8
remaining_transition_lifecycle_import_bridge_7
```

BE-001DW-03 实际改写完成后，预期 residual 下降:

```text
expected_parent_import_bridge_20_to_19
expected_mutation_import_bridge_18_to_17
expected_parameter_mutation_import_bridge_8_to_7
expected_transition_lifecycle_import_bridge_7_to_6
```

---

## 下一步边界

下一步只能进入:

```text
BE-001DW-01
runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass
单子叶等价基线
```

BE-001DW-01 不得直接改 Rust；只冻结 `boundary_safety.rs` 当前输入面。

---

## 验证要求

本批提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001DV-02 完成时，必须说明:

1. 本批是 `no code movement`。
2. `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass stop_split: false`。
3. 拒绝 7 文件同批 rewrite。
4. 下一步只能进入 BE-001DW-01 `boundary_safety_import_pass` 单子叶等价基线。
5. 当前 parent bridge 仍为 total 20 / mutation 18 / parameter_mutation 8 / transition_lifecycle 7。
6. 旧三叶暂停目标保持取消。

不得宣称 transition lifecycle 已抽离、parameter mutation import 已完成、mutation import 已完成、parent import bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `370-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. transition lifecycle 父叶设置 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass stop_split: false`。
3. 下一步固定为 BE-001DW-01 `boundary_safety_import_pass` 单子叶等价基线。
4. 不恢复旧三叶暂停目标。
5. Rust / 治理 / 全量树门禁均通过。
