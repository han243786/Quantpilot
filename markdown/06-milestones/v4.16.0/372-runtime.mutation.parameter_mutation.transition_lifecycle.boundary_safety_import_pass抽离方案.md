# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DW-02
> 基线: `371-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DW-03 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DW-02 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 抽离方案 | 方案冻结 |
| 规范矩阵 | staged explicit import pass / 单文件 import rewrite / 父子通信硬规则 | 规则约束 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` | 白箱边界固定 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` | 下一步实际抽离授权 |

---

## 方案冻结标记

```text
boundary_safety_import_pass plan_frozen
single_file_boundary_safety_import_rewrite
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
use super::* -> explicit imports
json_bad_request
RuntimeParameterMutationBoundary
RuntimeParameterMutationSafeWindowSnapshot
RuntimeParameterMutationSafeWindowState
StatusCode
validate_runtime_parameter_mutation_boundary
resolve_runtime_parameter_mutation_boundary
evaluate_runtime_parameter_mutation_safe_window
no_function_body_change
no_visibility_change
no_transition_lifecycle_facade_rewrite
no_activation_or_rollback_rewrite
no_sibling_horizontal_link
no_release_transition
old_three_leaf_pause_target_cancelled
```

当前 residual:

```text
remaining_parent_import_bridge_20
remaining_mutation_import_bridge_18
remaining_parameter_mutation_import_bridge_8
remaining_transition_lifecycle_import_bridge_7
```

BE-001DW-03 实际抽离完成后的预期 residual:

```text
expected_parent_import_bridge_20_to_19
expected_mutation_import_bridge_18_to_17
expected_parameter_mutation_import_bridge_8_to_7
expected_transition_lifecycle_import_bridge_7_to_6
```

---

## 允许动作

BE-001DW-03 只允许改写一个文件的顶部 import:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
```

当前 import:

```rust
use super::*;
```

预期替换为:

```rust
use crate::{
    json_bad_request, RuntimeParameterMutationBoundary,
    RuntimeParameterMutationSafeWindowSnapshot, RuntimeParameterMutationSafeWindowState,
};
use axum::http::StatusCode;
```

---

## 禁止动作

BE-001DW-03 不得触碰:

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

硬约束:

1. `no_function_body_change`: 不改任何函数体。
2. `no_visibility_change`: 不改 helper 可见性。
3. `no_transition_lifecycle_facade_rewrite`: 不改 `transition_lifecycle.rs` facade。
4. `no_activation_or_rollback_rewrite`: 不改 activation / rollback sibling。
5. `no_sibling_horizontal_link`: 不新增 sibling 横向连接。
6. `no_release_transition`: 不主动提出或启动发布态过渡。

---

## 等价语义

BE-001DW-03 必须保持:

1. 空 `requested` 仍返回 `bad_request` 与 `activation_boundary.requested 是必填字段`。
2. `immediate` 仍返回 `parameter_mutation_boundary_violation` 与“不支持立即激活的参数变更”。
3. `next_cycle_start` 与 `manual_pause` 仍直接通过 validation。
4. `sequence_cursor` 仍需要 `resolved_sequence_no`，并从 `sequence_cursor:<u64>` 解析。
5. `next_cycle_start` resolution 仍为 `current_sequence_no + 2`。
6. `manual_pause` resolution 仍为 `resolved_sequence_no: None`。
7. 缺少 sequence cursor 时仍返回 `parameter_mutation_boundary_violation` 与“序列游标激活边界需要 resolved_sequence_no”。
8. safe window reason code 优先级仍包含 `SAFE_WINDOW_RUNTIME_ACTIVE`、`SAFE_WINDOW_OPEN_ORDERS`、`SAFE_WINDOW_RISK_VIOLATION`、`SAFE_WINDOW_STALE_DATA`、`SAFE_WINDOW_EXPOSURE_LIMIT`、`SAFE_WINDOW_COOLDOWN`。
9. safe window `allowed` 仍只在 `SAFE_WINDOW_OPEN` 时为 true，snapshot 原样回填。
10. 仍不启动 release transition，仍不新增 sibling horizontal link。

---

## 执行步骤

1. 在 BE-001DW-03 中只替换 `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs` 顶部 `use super::*`。
2. 运行格式化与编译门禁，确认函数体没有 diff。
3. 运行 `api_mutation` 回归，确认 boundary / safe window 行为不变。
4. 统计 residual，必须从 total 20 / mutation 18 / parameter_mutation 8 / transition_lifecycle 7 下降到 total 19 / mutation 17 / parameter_mutation 7 / transition_lifecycle 6。
5. 建立 BE-001DW-03 实际抽离记录，并更新里程碑、模块树、全量树与治理门禁。

---

## 验证要求

BE-001DW-02 提交前至少执行:

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

AI 声称 BE-001DW-02 完成时，必须说明:

1. 本批是 `no code movement`。
2. 本批只冻结 BE-001DW-03 的单文件 import rewrite 方案。
3. 下一步只允许改 `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs` 顶部 import。
4. 本批未改 Rust 函数体、helper visibility、facade、activation / rollback sibling、AI proposal、root bridge 或 release transition。
5. 当前 residual 仍是 total 20 / mutation 18 / parameter_mutation 8 / transition_lifecycle 7。
6. 旧三叶暂停目标保持取消: `old_three_leaf_pause_target_cancelled`。

不得宣称 boundary safety import 已改写、transition lifecycle 已完成、parameter mutation import 已完成、mutation import 已完成、parent import bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `372-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001DW-03 的唯一 Rust 改动范围被固定为 `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs` 顶部 import。
3. 明确禁止函数体、可见性、facade、activation / rollback sibling、AI proposal、root bridge 与 release transition 变更。
4. 旧三叶暂停目标不恢复，递归队列继续保持干净。
5. Rust / 治理 / 全量树门禁均通过。
