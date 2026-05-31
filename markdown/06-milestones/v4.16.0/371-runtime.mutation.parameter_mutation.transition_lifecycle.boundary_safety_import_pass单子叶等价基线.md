# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DW-01
> 基准: `370-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass抽离方案.md`
> 目标子叶: `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DW-02 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DW-01 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | boundary equivalence、safe window reason code、parent white-box helper、explicit import pass | 输入面冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` | boundary safety 白箱登记 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` | 建立单子叶基线 |

---

## 基线冻结

```text
runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass
boundary_safety_import_pass baseline_frozen
single_file_boundary_safety_import_pass
remaining_parent_import_bridge_20
remaining_mutation_import_bridge_18
remaining_parameter_mutation_import_bridge_8
remaining_transition_lifecycle_import_bridge_7
old_three_leaf_pause_target_cancelled
```

冻结文件:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
```

当前 residual:

```rust
use super::*;
```

---

## 白箱输入输出

目标 helper:

| helper | 当前可见性 | 调用方 | 约束 |
| --- | --- | --- | --- |
| `validate_runtime_parameter_mutation_boundary` | `pub(super)` | `transition_lifecycle.rs` facade；proposal creation 通过 parent white-box path | 不改变 accepted boundary 与错误文案 |
| `resolve_runtime_parameter_mutation_boundary` | `pub(super)` | activation / rollback flow | 不改变 sequence resolution |
| `evaluate_runtime_parameter_mutation_safe_window` | `pub(super)` | activation / rollback flow | 不改变 reason code 优先级 |

显式输入面应限制为:

```text
json_bad_request
RuntimeParameterMutationBoundary
RuntimeParameterMutationSafeWindowSnapshot
RuntimeParameterMutationSafeWindowState
StatusCode
```

预期 BE-001DW-03 import:

```rust
use crate::{
    json_bad_request, RuntimeParameterMutationBoundary,
    RuntimeParameterMutationSafeWindowSnapshot, RuntimeParameterMutationSafeWindowState,
};
use axum::http::StatusCode;
```

---

## 等价语义

必须保持不变:

1. 空 `requested` 返回 `bad_request` 与 `activation_boundary.requested 是必填字段`。
2. `immediate` 返回 `parameter_mutation_boundary_violation` 与“不支持立即激活的参数变更”文案。
3. `next_cycle_start` 与 `manual_pause` 直接通过 validation。
4. `sequence_cursor` 需要 `resolved_sequence_no`，`sequence_cursor:<u64>` 可从 requested 中解析。
5. `next_cycle_start` resolution 仍为 `current_sequence_no + 2`。
6. `manual_pause` resolution 仍为 `resolved_sequence_no: None`。
7. 缺少 sequence cursor 时仍返回 `parameter_mutation_boundary_violation` 与“序列游标激活边界需要 resolved_sequence_no”文案。
8. safe window reason code 优先级保持: `SAFE_WINDOW_RUNTIME_ACTIVE`、`SAFE_WINDOW_OPEN_ORDERS`、`SAFE_WINDOW_RISK_VIOLATION`、`SAFE_WINDOW_STALE_DATA`、`SAFE_WINDOW_EXPOSURE_LIMIT`、`SAFE_WINDOW_COOLDOWN`。
9. safe window `allowed` 仅在 `SAFE_WINDOW_OPEN` 时为 true，snapshot 原样回填。
10. release transition 未启动，未新增 sibling horizontal link。

---

## 影响边界

BE-001DW-01 只冻结 `boundary_safety.rs` 的 import 输入面。

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

## 下一步边界

下一步只能进入:

```text
BE-001DW-02
runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass
抽离方案
```

BE-001DW-02 必须固定 BE-001DW-03 的单文件 import rewrite 边界，不能直接改 Rust。

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

AI 声称 BE-001DW-01 完成时，必须说明:

1. 本批是 `no code movement`。
2. 冻结文件是 `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs`。
3. 当前 residual 是 `use super::*`。
4. helper 为 `validate_runtime_parameter_mutation_boundary`、`resolve_runtime_parameter_mutation_boundary`、`evaluate_runtime_parameter_mutation_safe_window`。
5. 当前 parent bridge 仍为 total 20 / mutation 18 / parameter_mutation 8 / transition_lifecycle 7。
6. 下一步只能进入 BE-001DW-02 抽离方案。
7. 旧三叶暂停目标保持取消。

不得宣称 boundary safety import 已改写、transition lifecycle 已抽离、parameter mutation import 已完成、mutation import 已完成、parent import bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `371-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 冻结 `boundary_safety.rs` 当前输入面与等价语义。
3. 下一步固定为 BE-001DW-02 抽离方案。
4. 不恢复旧三叶暂停目标。
5. Rust / 治理 / 全量树门禁均通过。
