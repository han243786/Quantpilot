# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow 抽离方案

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AJ-02  
> 基准: `145-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow单子叶等价基线.md`、`144-runtime.mutation.parameter_mutation.transition_lifecycle父叶残余判断.md`、`src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`、`src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs`、`tests/api_mutation.rs`  
> 判定: 固定 `activation_flow` 的目标文件、父级声明、handler re-export、迁移清单和回退点。当前 `no code movement`。下一步只能进入 BE-001AJ-03 实际抽离。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AJ-02 activation_flow 抽离方案 | 方案 |
| 规范矩阵 | 父子通信、visibility、activation handler 迁移、回退点 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` | 抽离方案 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` | 决定目标文件 |

---

## 目标结构

下一批 BE-001AJ-03 只允许创建一个 child:

| 项 | 决定 |
| --- | --- |
| 目标文件 | `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs` |
| 父级 owner | `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` |
| 父级声明 | `#[path = "transition_lifecycle/activation_flow.rs"] mod activation_flow;` |
| 父级出口 | `pub(crate) use activation_flow::activate_runtime_parameter_mutation;` |
| child prelude | `use super::*;` |
| 下一批次 | BE-001AJ-03 实际抽离 |

为什么使用 path attribute: 当前 `transition_lifecycle.rs` 自身由上层 `#[path = "parameter_mutation/transition_lifecycle.rs"] mod transition_lifecycle;` 注册。若使用裸 `mod activation_flow;`，Rust 可能按上层 path 解析到错误目录。BE-001AJ-03 直接沿用 `boundary_safety` 的 path-attributed child 模式。

---

## 迁移清单

BE-001AJ-03 只允许迁移一个 public handler:

- `activate_runtime_parameter_mutation`

父级 `transition_lifecycle.rs` 保留:

- `#[path = "transition_lifecycle/boundary_safety.rs"] mod boundary_safety;`
- `use boundary_safety::{evaluate_runtime_parameter_mutation_safe_window, resolve_runtime_parameter_mutation_boundary};`
- `pub(super) fn validate_runtime_parameter_mutation_boundary(...)` wrapper
- `runtime_parameter_mutation_rollback_record_id`
- `mutation_lifecycle_entry`
- `persist_runtime_parameter_mutation_transition`
- `auto_snapshot_on_activation`
- `rollback_runtime_parameter_mutation`

`activation_flow.rs` 可以通过 `use super::*;` 调用父级保留的 helper，但不得引入 route facade、AI proposal、approval review、frontend caller 或发布过渡连接。

---

## 计划代码形态

`src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`:

```rust
use super::*;

#[path = "transition_lifecycle/activation_flow.rs"]
mod activation_flow;
#[path = "transition_lifecycle/boundary_safety.rs"]
mod boundary_safety;

pub(crate) use activation_flow::activate_runtime_parameter_mutation;

use boundary_safety::{
    evaluate_runtime_parameter_mutation_safe_window, resolve_runtime_parameter_mutation_boundary,
};
```

`src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`:

```rust
use super::*;

pub(crate) async fn activate_runtime_parameter_mutation(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
    Json(request): Json<ActivateRuntimeParameterMutationRequest>,
) -> Result<Json<RuntimeParameterMutationRecord>, (StatusCode, String)> {
    // existing implementation moves here unchanged
}
```

BE-001AJ-03 不得改变 activation body 语义、error code、status transition、event sequence、metrics、transition persistence 顺序、`auto_snapshot_on_activation` 调用时机或 response schema。

关键 helper 调用必须保持原样，尤其是 `append_parameter_mutation_events_to_run`、`persist_runtime_parameter_mutation_transition` 和 `auto_snapshot_on_activation` 的相对顺序。

---

## 等价检查清单

| 维度 | 必须保持 |
| --- | --- |
| capability denied | 仍返回 `parameter_mutation_boundary_violation`，不写 mutation record |
| invalid status | 仍只允许 `Proposed` / `SafeWindowDenied` 进入 activation |
| safe-window denied | 仍写 `SafeWindowDenied`、denied lifecycle/event、denied metric，并返回 `parameter_mutation_safe_window_denied` |
| scheduled | 仍写 `ActivationScheduled`、schedule lifecycle/event、scheduled metric |
| next cycle activation | 仍写 `Activated`、active parameter version、activation lifecycle/event 和 applied metric |
| failed boundary | 仍写 `ActivationFailed`、failure reason、failed lifecycle/event 和 failed metric |
| append order | 仍先 append run events，再 persist transition，再触发 `auto_snapshot_on_activation` |
| sibling boundary_safety | 仍通过父级受控 helper 调用，不直接绕过父级 |

---

## 父子通信规则

```text
parameter_mutation.rs
  -> transition_lifecycle::activate_runtime_parameter_mutation
transition_lifecycle.rs
  -> activation_flow::activate_runtime_parameter_mutation
activation_flow.rs
  -> parent-owned helpers via use super::*
```

`activation_flow` 只能被父级 `transition_lifecycle` 调用或 re-export。`parameter_mutation.rs` 不得直接声明或导入 `activation_flow`；route facade、AI proposal、approval review、frontend caller 和发布过渡连接也不得直接依赖该 child。ASCII guard: `release transition guard`。

---

## 回退点

若 BE-001AJ-03 出现 visibility、path attribute 或 borrow checker 问题，只回退:

- `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 中的 `mod activation_flow` 与 `pub(crate) use activation_flow::activate_runtime_parameter_mutation`

不得回退 BE-001AH 已完成的 `boundary_safety` 抽离，不得回退 BE-001AG 已完成的 `transition_lifecycle.rs` 抽离，也不得回退 `parameter_mutation.rs` 子模块化。

---

## 本批不做

- 不移动 Rust 代码。
- 不创建 `activation_flow.rs`。
- 不迁移 `rollback_runtime_parameter_mutation`。
- 不迁移 `auto_snapshot_on_activation` helper body。
- 不迁移 `boundary_safety` helper。
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

AI 声称 BE-001AJ-02 完成时，必须说明本批只完成 `activation_flow` 抽离方案，仍为 `no code movement`，下一步只能进入 BE-001AJ-03 实际抽离。不得宣称 `activation_flow.rs` 已创建、activation handler 已迁移、rollback_flow 已拆分、snapshot helper body 已迁移、AppState/schema/frontend caller 已改变、发布过渡已启动或整理/重构已经完成。

---

## 验收标准

1. `146-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案固定目标文件、父级 path attribute、handler re-export 和 helper 保留边界。
3. 方案明确 BE-001AJ-03 只迁移 `activate_runtime_parameter_mutation`。
4. 方案明确本批 `no code movement`，不迁移 rollback、snapshot helper body、AI proposal、approval、schema、state、frontend caller 或发布过渡连接。
5. 本批验证通过后，后续才能进入 BE-001AJ-03。
