# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety 抽离方案

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AH-02  
> 基准: `140-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety单子叶等价基线.md`、`139-runtime.mutation.parameter_mutation.transition_lifecycle单叶closeout.md`、`src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`、`tests/api_mutation.rs`  
> 判定: 固定 `boundary_safety` 的目标文件、父级声明、delegating validation wrapper、helper visibility 和迁移清单。当前 `no code movement`。下一步只能进入 BE-001AH-03 实际抽离。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AH-02 boundary_safety 抽离方案 | 方案 |
| 规范矩阵 | 父子通信、visibility、boundary / safe-window helper 迁移、回退点 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` | 抽离方案 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` | 决定目标文件 |

---

## 目标结构

下一批 BE-001AH-03 只允许创建一个 child:

| 项 | 决定 |
| --- | --- |
| 目标文件 | `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs` |
| 父级 owner | `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` |
| 父级声明 | `mod boundary_safety;` |
| 父级调用 | `use boundary_safety::{evaluate_runtime_parameter_mutation_safe_window, resolve_runtime_parameter_mutation_boundary};` |
| validation 出口 | 父级保留 `pub(super) fn validate_runtime_parameter_mutation_boundary(...)` delegating wrapper |
| 下一批次 | BE-001AH-03 实际抽离 |

为什么使用 delegating wrapper: `validate_runtime_parameter_mutation_boundary` 目前被上层 `src/runtime/mutation/parameter_mutation.rs` 复用。为了不扩大 child visibility，也不让上层直接依赖 `boundary_safety`，BE-001AH-03 保留一个薄 wrapper 在 `transition_lifecycle.rs`，实际逻辑迁入 child。

---

## 迁移清单

BE-001AH-03 只允许迁移以下函数逻辑:

- `validate_runtime_parameter_mutation_boundary`
- `resolve_runtime_parameter_mutation_boundary`
- `evaluate_runtime_parameter_mutation_safe_window`

父级 `transition_lifecycle.rs` 保留:

- `pub(super) fn validate_runtime_parameter_mutation_boundary(...)` wrapper
- `runtime_parameter_mutation_rollback_record_id`
- `mutation_lifecycle_entry`
- `persist_runtime_parameter_mutation_transition`
- `activate_runtime_parameter_mutation`
- `auto_snapshot_on_activation`
- `rollback_runtime_parameter_mutation`

---

## 计划代码形态

`src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`:

```rust
use super::*;

mod boundary_safety;

use boundary_safety::{
    evaluate_runtime_parameter_mutation_safe_window,
    resolve_runtime_parameter_mutation_boundary,
};

pub(super) fn validate_runtime_parameter_mutation_boundary(
    boundary: &RuntimeParameterMutationBoundary,
) -> Result<(), (StatusCode, String)> {
    boundary_safety::validate_runtime_parameter_mutation_boundary(boundary)
}
```

`src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs`:

```rust
use super::*;

pub(super) fn validate_runtime_parameter_mutation_boundary(
    boundary: &RuntimeParameterMutationBoundary,
) -> Result<(), (StatusCode, String)> {
    // existing implementation moves here
}

pub(super) fn resolve_runtime_parameter_mutation_boundary(
    boundary: &RuntimeParameterMutationBoundary,
    current_sequence_no: u64,
) -> Result<RuntimeParameterMutationBoundary, (StatusCode, String)> {
    // existing implementation moves here
}

pub(super) fn evaluate_runtime_parameter_mutation_safe_window(
    snapshot: Option<RuntimeParameterMutationSafeWindowSnapshot>,
) -> RuntimeParameterMutationSafeWindowState {
    // existing implementation moves here
}
```

BE-001AH-03 不得改变 function body 语义、error code、message、reason code、priority、retryable、retry_after_ms、`next_cycle_start` = `current_sequence_no + 2` 或 `manual_pause` 解析语义。

---

## 父子通信规则

```text
parameter_mutation.rs
  -> transition_lifecycle::validate_runtime_parameter_mutation_boundary wrapper
transition_lifecycle.rs
  -> boundary_safety::{resolve_runtime_parameter_mutation_boundary, evaluate_runtime_parameter_mutation_safe_window}
  -> boundary_safety::validate_runtime_parameter_mutation_boundary via wrapper only
```

`boundary_safety` 只能被父级 `transition_lifecycle` 调用。`parameter_mutation.rs` 不得直接 `use boundary_safety::*`；route facade、AI proposal、approval review、frontend caller 和发布过渡连接也不得直接依赖该 child。ASCII guard: `release transition guard`。

---

## 等价检查清单

| 维度 | 必须保持 |
| --- | --- |
| empty requested | 仍返回 `bad_request` |
| `immediate` | 仍返回 `parameter_mutation_boundary_violation` |
| `next_cycle_start` | 仍解析为 `current_sequence_no + 2` |
| `manual_pause` | 仍不写 resolved sequence |
| `sequence_cursor` | 仍要求 resolved sequence 或 `sequence_cursor:<u64>` |
| safe window open | 仍返回 `SAFE_WINDOW_OPEN` |
| active runtime | 仍优先返回 `SAFE_WINDOW_RUNTIME_ACTIVE` |
| open orders | 仍返回 `SAFE_WINDOW_OPEN_ORDERS` |
| risk violation | 仍返回 `SAFE_WINDOW_RISK_VIOLATION` |
| stale data | 仍返回 `SAFE_WINDOW_STALE_DATA` |
| exposure limit | 仍返回 `SAFE_WINDOW_EXPOSURE_LIMIT` |
| cooldown | 仍返回 `SAFE_WINDOW_COOLDOWN` 且带 `retry_after_ms` |

---

## 回退点

若 BE-001AH-03 出现 visibility 或路径编译问题，只回退:

- `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 中的 `mod boundary_safety`、`use boundary_safety::*` 和 validation wrapper

不得回退 BE-001AG-03 已完成的 `transition_lifecycle.rs` 抽离，也不得回退 `parameter_mutation.rs` 子模块化。

---

## 本批不做

- 不移动 Rust 代码。
- 不创建 `boundary_safety.rs`。
- 不拆 activation / rollback handler。
- 不迁移 `mutation_lifecycle_entry`、`persist_runtime_parameter_mutation_transition`、rollback id 或 snapshot helper。
- 不迁移 proposal create/list/detail、AI proposal、approval review、AppState、schema、frontend caller、route facade、测试 fixture 或发布过渡连接。

---

## 验证计划

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001AH-02 完成时，必须说明本批只完成 `boundary_safety` 抽离方案，仍为 `no code movement`，下一步只能进入 BE-001AH-03 实际抽离。不得宣称 `boundary_safety.rs` 已创建、helper 已迁移、activation/rollback 已拆分、AppState/schema/frontend caller 已改变、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `141-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案固定目标文件、父级 `mod boundary_safety`、delegating validation wrapper 和 helper visibility。
3. 方案明确 BE-001AH-03 只迁移 boundary validation、boundary resolution 和 safe-window evaluation 三个 helper。
4. 方案明确本批 `no code movement`，不迁移 activation/rollback、snapshot、AI proposal、approval、schema、state、frontend caller 或发布过渡连接。
5. 本批验证通过后，后续才能进入 BE-001AH-03 实际抽离。
