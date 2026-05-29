# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety 抽离记录

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AH-03  
> 基线: `140-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety单子叶等价基线.md`、`141-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety抽离方案.md`、`src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`、`tests/api_mutation.rs`  
> 判定: `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` 实际抽离完成。下一步只能进入 BE-001AH-04 单叶 closeout，判断本叶是否还值得继续细拆。  
> 代码动作: actual extraction

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AH-03 boundary_safety 实际抽离 | 落地 |
| 规范矩阵 | 父子通信、可见性、path-attribute fallback、发布过渡保护 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` | 白箱节点落地 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` | 记录真实文件 |

---

## 实际抽离结果

| 项 | 结果 |
| --- | --- |
| 新增文件 | `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs` |
| 父级文件 | `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` |
| 父级声明 | `#[path = "transition_lifecycle/boundary_safety.rs"] mod boundary_safety;` |
| 父级 helper import | `use boundary_safety::{evaluate_runtime_parameter_mutation_safe_window, resolve_runtime_parameter_mutation_boundary};` |
| 父级 validation 出口 | `validate_runtime_parameter_mutation_boundary` 保留 delegating validation wrapper |
| 上层复用 | `src/runtime/mutation/parameter_mutation.rs` 仍只调用 `transition_lifecycle::validate_runtime_parameter_mutation_boundary` |
| route facade | `src/backend/runtime/routes/mutation.rs` 未改变 |

裸 `mod boundary_safety;` 在当前 `#[path = "parameter_mutation/transition_lifecycle.rs"]` 父模块结构下会被 Rust 解析到 `src/runtime/mutation/parameter_mutation/boundary_safety.rs`。因此实际抽离使用显式 path attribute 保持方案指定的嵌套目标文件。

---

## 已迁移函数

| 函数 | 目标可见性 | 说明 |
| --- | --- | --- |
| `validate_runtime_parameter_mutation_boundary` | `pub(super)` in child + parent wrapper | 供 proposal create 和 transition lifecycle 继续复用 |
| `resolve_runtime_parameter_mutation_boundary` | `pub(super)` | activation / rollback boundary resolution |
| `evaluate_runtime_parameter_mutation_safe_window` | `pub(super)` | activation / rollback safe-window evaluation |

父级 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 只保留 wrapper:

```rust
pub(super) fn validate_runtime_parameter_mutation_boundary(
    boundary: &RuntimeParameterMutationBoundary,
) -> Result<(), (StatusCode, String)> {
    boundary_safety::validate_runtime_parameter_mutation_boundary(boundary)
}
```

---

## 等价约束

本批未改变:

- `immediate` boundary rejection。
- `next_cycle_start` 解析为 `current_sequence_no + 2`。
- `manual_pause` 保持 `resolved_sequence_no: None`。
- `sequence_cursor` / `sequence_cursor:<u64>` 解析语义。
- safe-window reason code 优先级: `SAFE_WINDOW_OPEN`、`SAFE_WINDOW_RUNTIME_ACTIVE`、`SAFE_WINDOW_OPEN_ORDERS`、`SAFE_WINDOW_RISK_VIOLATION`、`SAFE_WINDOW_STALE_DATA`、`SAFE_WINDOW_EXPOSURE_LIMIT`、`SAFE_WINDOW_COOLDOWN`。
- `retryable`、`retry_after_ms`、message、error code 和 response schema。

---

## 父子通信结果

```text
parameter_mutation.rs
  -> transition_lifecycle::validate_runtime_parameter_mutation_boundary
transition_lifecycle.rs
  -> boundary_safety::validate_runtime_parameter_mutation_boundary via delegating validation wrapper
  -> boundary_safety::{evaluate_runtime_parameter_mutation_safe_window, resolve_runtime_parameter_mutation_boundary}
```

`boundary_safety` 只能被父级 `transition_lifecycle` 调用。`parameter_mutation.rs` 不直接 `use boundary_safety::*`；route facade、AI proposal、approval review、AppState、schema、frontend caller 和发布过渡连接均未改变。ASCII guard: `release transition guard`。

---

## 非目标

- 不迁移 activation / rollback handler。
- 不迁移 `runtime_parameter_mutation_rollback_record_id`、`mutation_lifecycle_entry`、`persist_runtime_parameter_mutation_transition` 或 `auto_snapshot_on_activation`。
- 不迁移 proposal create/list/detail、AI proposal、approval review、AppState、schema、frontend caller、runtime persistence owner、snapshot owner 或 route facade。
- 不启动发布过渡，不提出横向连接或性能旁路。

---

## 验证记录

| 命令 | 结果 |
| --- | --- |
| `cargo fmt` | PASS |
| `cargo fmt --check` | PASS |
| `cargo check -p quantpilot` | PASS |
| `cargo test --no-run` | PASS |
| `cargo test -p quantpilot --test api_mutation` | PASS |
| `cargo test -p quantpilot --test api_ai_proposal` | PASS |
| `cargo test -p quantpilot --test api_evidence_contract` | PASS |
| `cargo test -p quantpilot --test api_run` | PASS |
| `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1` | PASS |
| `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1` | PASS |
| `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1` | PASS |
| `git diff --check` | PASS |

---

## 下一步

下一批进入 BE-001AH-04 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` 单叶 closeout。closeout 只判断本叶是否还值得继续细拆；不得顺手迁移 activation/rollback handler、proposal record、AI proposal、approval review、schema、frontend caller 或发布过渡连接。

---

## 幻觉检查点

AI 声称 BE-001AH-03 完成时，必须说明 boundary_safety 已实际抽离到 `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs`，但尚未完成单叶 closeout。不得宣称 activation/rollback handler 已拆分、parameter_mutation 父叶完成、AI proposal/approval 已拆分、AppState/schema/frontend caller 已改变、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs` 进入模块树和全量树。
2. `142-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
3. 父级 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 保留 delegating validation wrapper，并通过 child helper import 调用 boundary resolution 与 safe-window evaluation。
4. route facade、runtime facade、AppState、schema、frontend caller 和发布过渡保护未改变。
5. 本批验证通过后，后续才能进入 BE-001AH-04 单叶 closeout。
