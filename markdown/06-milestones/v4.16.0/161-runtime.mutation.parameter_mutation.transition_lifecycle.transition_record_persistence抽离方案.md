# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence 抽离方案

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AP-02  
> 基准: `160-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence单子叶等价基线.md`、`159-runtime.mutation.parameter_mutation.transition_lifecycle第四轮父叶残余判断.md`、`src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`、`src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`、`src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`、`tests/api_mutation.rs`  
> 判定: 固定 `transition_record_persistence` 的目标文件、父级声明、helper visibility、迁移清单和回退点。当前 `no code movement`。下一步只能进入 BE-001AP-03 实际抽离。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AP-02 transition_record_persistence 抽离方案 | 方案 |
| 规范矩阵 | 父子通信、visibility、shared helper 迁移、回退点 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` | 抽离方案 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` | 决定目标文件 |

---

## 目标结构

下一批 BE-001AP-03 只允许创建一个 child:

| 项 | 决定 |
| --- | --- |
| 目标文件 | `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs` |
| 父级 owner | `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` |
| 父级声明 | `#[path = "transition_lifecycle/transition_record_persistence.rs"] mod transition_record_persistence;` |
| 父级 helper 导入 | `use transition_record_persistence::{mutation_lifecycle_entry, persist_runtime_parameter_mutation_transition};` |
| child prelude | `use super::*;` |
| helper visibility | `pub(super) fn mutation_lifecycle_entry(...)`、`pub(super) async fn persist_runtime_parameter_mutation_transition(...)` |
| 下一批次 | BE-001AP-03 实际抽离 |

为什么使用 path attribute: 当前 `transition_lifecycle.rs` 自身由上层 `#[path = "parameter_mutation/transition_lifecycle.rs"] mod transition_lifecycle;` 注册。若使用裸 `mod transition_record_persistence;`，Rust 可能按上层 path 解析到错误目录。BE-001AP-03 直接沿用 `boundary_safety`、`activation_flow`、`rollback_flow` 和 `activation_snapshot_side_effect` 的 path-attributed child 模式。

---

## 迁移清单

BE-001AP-03 只允许迁移两个 helper:

- `mutation_lifecycle_entry`
- `persist_runtime_parameter_mutation_transition`

父级 `transition_lifecycle.rs` 保留:

- `#[path = "transition_lifecycle/activation_flow.rs"] mod activation_flow;`
- `#[path = "transition_lifecycle/activation_snapshot_side_effect.rs"] mod activation_snapshot_side_effect;`
- `#[path = "transition_lifecycle/boundary_safety.rs"] mod boundary_safety;`
- `#[path = "transition_lifecycle/rollback_flow.rs"] mod rollback_flow;`
- `pub(crate) use activation_flow::activate_runtime_parameter_mutation;`
- `pub(crate) use rollback_flow::rollback_runtime_parameter_mutation;`
- `use activation_snapshot_side_effect::auto_snapshot_on_activation;`
- `use boundary_safety::{evaluate_runtime_parameter_mutation_safe_window, resolve_runtime_parameter_mutation_boundary};`
- `pub(super) fn validate_runtime_parameter_mutation_boundary(...)` wrapper
- `runtime_parameter_mutation_rollback_record_id`

`transition_record_persistence.rs` 可以通过 `use super::*;` 读取父级已暴露的类型、helper 和 crate imports，但不得引入 route facade、AI proposal、approval review、frontend caller、AppState owner 改造、schema owner 改造或发布过渡连接。

---

## 计划代码形态

`src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`:

```rust
use super::*;

#[path = "transition_lifecycle/activation_flow.rs"]
mod activation_flow;
#[path = "transition_lifecycle/activation_snapshot_side_effect.rs"]
mod activation_snapshot_side_effect;
#[path = "transition_lifecycle/boundary_safety.rs"]
mod boundary_safety;
#[path = "transition_lifecycle/rollback_flow.rs"]
mod rollback_flow;
#[path = "transition_lifecycle/transition_record_persistence.rs"]
mod transition_record_persistence;

pub(crate) use activation_flow::activate_runtime_parameter_mutation;
pub(crate) use rollback_flow::rollback_runtime_parameter_mutation;

use activation_snapshot_side_effect::auto_snapshot_on_activation;
use boundary_safety::{
    evaluate_runtime_parameter_mutation_safe_window, resolve_runtime_parameter_mutation_boundary,
};
use transition_record_persistence::{
    mutation_lifecycle_entry, persist_runtime_parameter_mutation_transition,
};
```

`src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs`:

```rust
use super::*;

pub(super) fn mutation_lifecycle_entry(
    status: RuntimeParameterMutationStatus,
    event: &FrontendRuntimeEvent,
    sequence_no: u64,
    message: impl Into<String>,
) -> RuntimeParameterMutationLifecycleEntry {
    // existing implementation moves here unchanged
}

pub(super) async fn persist_runtime_parameter_mutation_transition(
    state: &AppState,
    user_id: &auth::UserId,
    record: &RuntimeParameterMutationRecord,
) -> Result<(), (StatusCode, String)> {
    // existing implementation moves here unchanged
}
```

BE-001AP-03 不得改变 lifecycle field source、`mutation_event_contract(status)` reason code、`persist_runtime_parameter_mutation_record` call order、`io_error` mapping、`state.parameter_mutations` write or `auth::scoped_key` key semantics.

---

## 等价检查清单

| 维度 | 必须保持 |
| --- | --- |
| lifecycle status | 与传入 `RuntimeParameterMutationStatus` 完全一致 |
| lifecycle reason_code | 仍来自 `mutation_event_contract(status)` 的第二返回值 |
| lifecycle event id | 仍为 `event.event_id.clone()` |
| lifecycle sequence no | 仍使用 caller 传入的 `sequence_no` |
| lifecycle occurred_at_ms | 仍为 `event.event_time_ms` |
| lifecycle message | 仍为 `message.into()` |
| persistence order | 仍先 `persist_runtime_parameter_mutation_record`，再写 `state.parameter_mutations` |
| persistence error | 仍 `map_err(io_error)`，不吞错、不改 status code |
| in-memory key | 仍为 `auth::scoped_key(user_id, &record.proposal_id)` |
| parent call surface | `activation_flow` / `rollback_flow` 仍通过父级 `transition_lifecycle` helper 名称调用 |

---

## 父子通信规则

```text
activation_flow.rs / rollback_flow.rs
  -> transition_lifecycle::{mutation_lifecycle_entry, persist_runtime_parameter_mutation_transition}
transition_lifecycle.rs
  -> transition_record_persistence::{mutation_lifecycle_entry, persist_runtime_parameter_mutation_transition}
transition_record_persistence.rs
  -> parent-owned imports / helpers via use super::*
```

`transition_record_persistence` 只能被父级 `transition_lifecycle` 管理。`activation_flow` 与 `rollback_flow` 也只能经父级 helper 名称调用。`parameter_mutation.rs`、route facade、AI proposal、approval review、frontend caller 和发布过渡连接不得直接依赖该 child。ASCII guard: `release transition guard`。

---

## 回退点

若 BE-001AP-03 出现 visibility、path attribute、borrow checker 或 import 问题，只回退:

- `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 中的 `mod transition_record_persistence` 与 `use transition_record_persistence::{...}`

不得回退 BE-001AH 已完成的 `boundary_safety` 抽离，不得回退 BE-001AJ 已完成的 `activation_flow` 抽离，不得回退 BE-001AL 已完成的 `rollback_flow` 抽离，不得回退 BE-001AN 已完成的 `activation_snapshot_side_effect` 抽离，也不得回退 BE-001AG 已完成的 `transition_lifecycle.rs` 抽离。

---

## 本批不做

- 不移动 Rust 代码。
- 不创建 `transition_record_persistence.rs`。
- 不迁移 `runtime_parameter_mutation_rollback_record_id`。
- 不迁移 `activate_runtime_parameter_mutation`。
- 不迁移 `rollback_runtime_parameter_mutation`。
- 不迁移 boundary helper。
- 不迁移 `auto_snapshot_on_activation`。
- 不迁移 proposal create/list/detail、AI proposal、approval review、AppState、schema、frontend caller、route facade、测试 fixture 或发布过渡连接。
- 不主动提出发布版本过渡或横向性能连接。

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

AI 声称 BE-001AP-02 完成时，必须说明本批只完成 `transition_record_persistence` 抽离方案，仍为 `no code movement`，下一步只能进入 BE-001AP-03 实际抽离。不得宣称 child 文件已创建、transition persistence helper 已迁移、rollback id 已拆分、AppState/schema/frontend caller 已改变、发布过渡已启动或整理/重构已经完成。

---

## 验收标准

1. `161-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案固定目标文件、父级 path attribute、helper import、helper visibility 和回退点。
3. 方案明确 BE-001AP-03 只迁移 `mutation_lifecycle_entry` 与 `persist_runtime_parameter_mutation_transition`。
4. 方案明确本批 `no code movement`，不迁移 rollback id、activation、rollback、boundary、snapshot、AI proposal、approval、schema、state、frontend caller 或发布过渡连接。
5. 本批验证通过后，后续才能进入 BE-001AP-03。
