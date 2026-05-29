# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity 抽离方案

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AR-02  
> 基准: `165-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity单子叶等价基线.md`、`164-runtime.mutation.parameter_mutation.transition_lifecycle第五轮父叶残余判断.md`、`src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`、`src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`、`tests/api_mutation.rs`  
> 判定: 固定 `rollback_record_identity` 的目标文件、父级声明、helper visibility、迁移清单和回退点。当前 `no code movement`。下一步只能进入 BE-001AR-03 实际抽离。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AR-02 rollback_record_identity 抽离方案 | 方案 |
| 规范矩阵 | 父子通信、visibility、rollback id helper 迁移、回退点 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` | 抽离方案 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` | 决定目标文件 |

---

## 目标结构

下一批 BE-001AR-03 只允许创建一个 child:

| 项 | 决定 |
| --- | --- |
| 目标文件 | `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs` |
| 父级 owner | `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` |
| 父级声明 | `#[path = "transition_lifecycle/rollback_record_identity.rs"] mod rollback_record_identity;` |
| 父级 helper 导入 | `use rollback_record_identity::runtime_parameter_mutation_rollback_record_id;` |
| child prelude | `use super::*;` |
| helper visibility | `pub(super) fn runtime_parameter_mutation_rollback_record_id(...)` |
| 下一批次 | BE-001AR-03 实际抽离 |

为什么使用 path attribute: 当前 `transition_lifecycle.rs` 自身由上层 `#[path = "parameter_mutation/transition_lifecycle.rs"] mod transition_lifecycle;` 注册。BE-001AR-03 直接沿用 sibling child 的 path-attributed child 模式，避免 Rust 按上层 path 解析到错误目录。

---

## 迁移清单

BE-001AR-03 只允许迁移一个 helper:

- `runtime_parameter_mutation_rollback_record_id`

父级 `transition_lifecycle.rs` 保留:

- `#[path = "transition_lifecycle/activation_flow.rs"] mod activation_flow;`
- `#[path = "transition_lifecycle/activation_snapshot_side_effect.rs"] mod activation_snapshot_side_effect;`
- `#[path = "transition_lifecycle/boundary_safety.rs"] mod boundary_safety;`
- `#[path = "transition_lifecycle/rollback_flow.rs"] mod rollback_flow;`
- `#[path = "transition_lifecycle/transition_record_persistence.rs"] mod transition_record_persistence;`
- `pub(crate) use activation_flow::activate_runtime_parameter_mutation;`
- `pub(crate) use rollback_flow::rollback_runtime_parameter_mutation;`
- `use activation_snapshot_side_effect::auto_snapshot_on_activation;`
- `use boundary_safety::{evaluate_runtime_parameter_mutation_safe_window, resolve_runtime_parameter_mutation_boundary};`
- `use transition_record_persistence::{mutation_lifecycle_entry, persist_runtime_parameter_mutation_transition};`
- `pub(super) fn validate_runtime_parameter_mutation_boundary(...)` wrapper

`rollback_record_identity.rs` 可以通过 `use super::*;` 读取父级已暴露的 type/import/helper，但不得引入 route facade、AI proposal、approval review、frontend caller、AppState owner 改造、schema owner 改造或发布过渡连接。

---

## 计划代码形态

`src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`:

```rust
#[path = "transition_lifecycle/rollback_record_identity.rs"]
mod rollback_record_identity;

use rollback_record_identity::runtime_parameter_mutation_rollback_record_id;
```

`src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs`:

```rust
use super::*;

pub(super) fn runtime_parameter_mutation_rollback_record_id(
    source_id: &str,
    rollback_of: &str,
    target: &RuntimeParameterMutationTarget,
    created_at_ms: u64,
    source_event_count: usize,
    proposed_parameter_version: &str,
) -> Result<String, (StatusCode, String)> {
    // existing implementation moves here unchanged
}
```

BE-001AR-03 不得改变 digest input order/names, `canonical_json_sha256_digest`, `json!`, `internal_error`, `parameter_rollback_` prefix, `created_at_ms` output segment or `digest[..12]` segment.

---

## 父子通信规则

```text
rollback_flow.rs
  -> transition_lifecycle::runtime_parameter_mutation_rollback_record_id
transition_lifecycle.rs
  -> rollback_record_identity::runtime_parameter_mutation_rollback_record_id
rollback_record_identity.rs
  -> parent-owned imports / helpers via use super::*
```

`rollback_record_identity` 只能被父级 `transition_lifecycle` 管理。`rollback_flow` 也只能经父级 helper 名称调用。`parameter_mutation.rs`、route facade、AI proposal、approval review、frontend caller 和发布过渡连接不得直接依赖该 child。ASCII guard: `release transition guard`。

---

## 回退点

若 BE-001AR-03 出现 visibility、path attribute、borrow checker 或 import 问题，只回退:

- `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 中的 `mod rollback_record_identity` 与 `use rollback_record_identity::runtime_parameter_mutation_rollback_record_id`

不得回退已完成的 `boundary_safety`、`activation_flow`、`rollback_flow`、`activation_snapshot_side_effect` 或 `transition_record_persistence` 抽离，也不得回退 BE-001AG 已完成的 `transition_lifecycle.rs` 抽离。

---

## 本批不做

- 不移动 Rust 代码。
- 不创建 `rollback_record_identity.rs`。
- 不回改已 closeout 的 `rollback_flow`。
- 不迁移 activation handler、rollback handler、boundary helper、snapshot helper、transition record persistence helper、proposal create/list/detail、AI proposal、approval review、AppState、schema、frontend caller、route facade、测试 fixture 或发布过渡连接。
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

AI 声称 BE-001AR-02 完成时，必须说明本批只完成 `rollback_record_identity` 抽离方案，仍为 `no code movement`，下一步只能进入 BE-001AR-03 实际抽离。不得宣称 child 文件已创建、rollback id helper 已迁移、rollback_flow 已回改、AppState/schema/frontend caller 已改变、发布过渡已启动或整理/重构已经完成。

---

## 验收标准

1. `166-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案固定目标文件、父级 path attribute、helper import、helper visibility 和回退点。
3. 方案明确 BE-001AR-03 只迁移 `runtime_parameter_mutation_rollback_record_id`。
4. 方案明确本批 `no code movement`，不回改 `rollback_flow`、不迁移 AI proposal、approval、schema、state、frontend caller 或发布过渡连接。
5. 本批验证通过后，后续才能进入 BE-001AR-03。
