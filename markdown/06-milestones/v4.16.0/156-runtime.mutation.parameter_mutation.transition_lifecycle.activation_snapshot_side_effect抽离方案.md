# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect 抽离方案

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AN-02  
> 基准: `155-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect单子叶等价基线.md`、`154-runtime.mutation.parameter_mutation.transition_lifecycle第三轮父叶残余判断.md`、`src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`、`src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`、`tests/api_mutation.rs`  
> 判定: 固定 `activation_snapshot_side_effect` 的目标文件、父级声明、helper visibility、迁移清单和回退点。当前 `no code movement`。下一步只能进入 BE-001AN-03 实际抽离。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AN-02 activation_snapshot_side_effect 抽离方案 | 方案 |
| 规范矩阵 | 父子通信、visibility、side effect helper 迁移、回退点 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` | 抽离方案 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` | 决定目标文件 |

---

## 目标结构

下一批 BE-001AN-03 只允许创建一个 child:

| 项 | 决定 |
| --- | --- |
| 目标文件 | `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs` |
| 父级 owner | `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` |
| 父级声明 | `#[path = "transition_lifecycle/activation_snapshot_side_effect.rs"] mod activation_snapshot_side_effect;` |
| 父级 helper 导入 | `use activation_snapshot_side_effect::auto_snapshot_on_activation;` |
| child prelude | `use super::*;` |
| helper visibility | `pub(super) async fn auto_snapshot_on_activation(...)` |
| 下一批次 | BE-001AN-03 实际抽离 |

为什么使用 path attribute: 当前 `transition_lifecycle.rs` 自身由上层 `#[path = "parameter_mutation/transition_lifecycle.rs"] mod transition_lifecycle;` 注册。若使用裸 `mod activation_snapshot_side_effect;`，Rust 可能按上层 path 解析到错误目录。BE-001AN-03 直接沿用 `boundary_safety`、`activation_flow` 和 `rollback_flow` 的 path-attributed child 模式。

---

## 迁移清单

BE-001AN-03 只允许迁移一个 helper:

- `auto_snapshot_on_activation`

父级 `transition_lifecycle.rs` 保留:

- `#[path = "transition_lifecycle/activation_flow.rs"] mod activation_flow;`
- `#[path = "transition_lifecycle/boundary_safety.rs"] mod boundary_safety;`
- `#[path = "transition_lifecycle/rollback_flow.rs"] mod rollback_flow;`
- `pub(crate) use activation_flow::activate_runtime_parameter_mutation;`
- `pub(crate) use rollback_flow::rollback_runtime_parameter_mutation;`
- `use boundary_safety::{evaluate_runtime_parameter_mutation_safe_window, resolve_runtime_parameter_mutation_boundary};`
- `pub(super) fn validate_runtime_parameter_mutation_boundary(...)` wrapper
- `runtime_parameter_mutation_rollback_record_id`
- `mutation_lifecycle_entry`
- `persist_runtime_parameter_mutation_transition`

`activation_snapshot_side_effect.rs` 可以通过 `use super::*;` 读取父级已暴露的类型、helper 和 crate imports，但不得引入 route facade、AI proposal、approval review、frontend caller、AppState owner 改造、schema owner 改造或发布过渡连接。

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

pub(crate) use activation_flow::activate_runtime_parameter_mutation;
pub(crate) use rollback_flow::rollback_runtime_parameter_mutation;

use activation_snapshot_side_effect::auto_snapshot_on_activation;
use boundary_safety::{
    evaluate_runtime_parameter_mutation_safe_window, resolve_runtime_parameter_mutation_boundary,
};
```

`src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs`:

```rust
use super::*;

/// Block 5 P1-6 + P3-2: 激活时自动生成签名快照 + 递增代际
pub(super) async fn auto_snapshot_on_activation(
    state: &AppState,
    user_id: &auth::UserId,
    mutation: &RuntimeParameterMutationRecord,
) {
    // existing implementation moves here unchanged
}
```

BE-001AN-03 不得改变 config generation 递增、history truncation、snapshot id、`DeploymentSignatureSnapshot` payload、`canonical_json_sha256_digest` signature、`atomic_write_json`、`safe_eprintln!` fallback、`state.snapshots` insert 或 activation response 语义。

---

## 等价检查清单

| 维度 | 必须保持 |
| --- | --- |
| generation | 仍使用 `state.config_generation.fetch_add(1, SeqCst)` |
| history truncation | 仍使用 `MAX_GENERATION_HISTORY = 100`，overflow 从头部 drain |
| metric baseline reads | 仍只读取 proposal rejected 与 rollback attempt 计数，不改变 metric |
| observation window | 仍只计算 `_observation_deadline_ms = now_ms.saturating_add(60_000)` |
| snapshot id | 仍为 `snap-auto-{now_ms}` |
| snapshot payload | 仍构造 `DeploymentSignatureSnapshot`，保持 deployment / capability / strategy / parameter / empty event slice |
| snapshot signature | 仍使用 `canonical_json_sha256_digest`，失败 fallback `signature-unavailable` |
| atomic write | 仍调用 `crate::runtime_persistence::atomic_write_json(&path, &snapshot).await` |
| write failure | 仍只 `safe_eprintln!`，不影响 activation handler response |
| in-memory insert | 仍写入 `state.snapshots`，key 仍为 `auth::scoped_key(user_id, &snapshot_id)` |
| parent call surface | `activation_flow` 仍通过父级 `transition_lifecycle` 的 helper 名称调用 |

---

## 父子通信规则

```text
activation_flow.rs
  -> transition_lifecycle::auto_snapshot_on_activation
transition_lifecycle.rs
  -> activation_snapshot_side_effect::auto_snapshot_on_activation
activation_snapshot_side_effect.rs
  -> parent-owned imports / helpers via use super::*
```

`activation_snapshot_side_effect` 只能被父级 `transition_lifecycle` 管理，`activation_flow` 也只能经父级 helper 名称调用。`parameter_mutation.rs`、route facade、AI proposal、approval review、frontend caller 和发布过渡连接不得直接依赖该 child。ASCII guard: `release transition guard`。

---

## 回退点

若 BE-001AN-03 出现 visibility、path attribute、borrow checker 或 import 问题，只回退:

- `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 中的 `mod activation_snapshot_side_effect` 与 `use activation_snapshot_side_effect::auto_snapshot_on_activation`

不得回退 BE-001AH 已完成的 `boundary_safety` 抽离，不得回退 BE-001AJ 已完成的 `activation_flow` 抽离，不得回退 BE-001AL 已完成的 `rollback_flow` 抽离，也不得回退 BE-001AG 已完成的 `transition_lifecycle.rs` 抽离。

---

## 本批不做

- 不移动 Rust 代码。
- 不创建 `activation_snapshot_side_effect.rs`。
- 不迁移 `activate_runtime_parameter_mutation`。
- 不迁移 `rollback_runtime_parameter_mutation`。
- 不迁移 `mutation_lifecycle_entry`。
- 不迁移 `persist_runtime_parameter_mutation_transition`。
- 不迁移 `runtime_parameter_mutation_rollback_record_id`。
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

AI 声称 BE-001AN-02 完成时，必须说明本批只完成 `activation_snapshot_side_effect` 抽离方案，仍为 `no code movement`，下一步只能进入 BE-001AN-03 实际抽离。不得宣称 child 文件已创建、snapshot helper 已迁移、shared helper 已拆分、AppState/schema/frontend caller 已改变、发布过渡已启动或整理/重构已经完成。

---

## 验收标准

1. `156-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案固定目标文件、父级 path attribute、helper import、helper visibility 和回退点。
3. 方案明确 BE-001AN-03 只迁移 `auto_snapshot_on_activation`。
4. 方案明确本批 `no code movement`，不迁移 activation、rollback、shared lifecycle/persistence helper、AI proposal、approval、schema、state、frontend caller 或发布过渡连接。
5. 本批验证通过后，后续才能进入 BE-001AN-03。
