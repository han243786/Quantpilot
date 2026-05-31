# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EA-01
> 基线: `380-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass父叶残余判断.md`
> 目标子叶: `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass`
> 代码动作: no code movement
> 下一步: BE-001EA-02 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EA-01 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | staged explicit import pass / parent white-box helper / lifecycle persistence contract | 输入面冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` | transition persistence 白箱登记 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` | 建立单子叶基线 |

---

## 基线冻结

```text
runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass
transition_record_persistence_import_pass baseline_frozen
single_file_transition_record_persistence_import_pass
remaining_parent_import_bridge_18
remaining_mutation_import_bridge_16
remaining_parameter_mutation_import_bridge_6
remaining_transition_lifecycle_import_bridge_5
old_three_leaf_pause_target_cancelled
```

冻结文件:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs
```

当前 residual:

```rust
use super::*;
```

本批不移动代码、不改函数体、不改可见性、不改父级 facade。

---

## 白箱输入输出

目标 helper:

| helper | 当前可见性 | 调用方 | 约束 |
| --- | --- | --- | --- |
| `mutation_lifecycle_entry` | `pub(super)` | activation / rollback flow 经 `transition_lifecycle.rs` parent facade | 不改 lifecycle entry 映射 |
| `persist_runtime_parameter_mutation_transition` | `pub(super)` | activation / rollback flow 经 `transition_lifecycle.rs` parent facade | 不改 record 持久化与 in-memory ledger 写入 |

函数签名必须保持:

```rust
pub(super) fn mutation_lifecycle_entry(
    status: RuntimeParameterMutationStatus,
    event: &FrontendRuntimeEvent,
    sequence_no: u64,
    message: impl Into<String>,
) -> RuntimeParameterMutationLifecycleEntry

pub(super) async fn persist_runtime_parameter_mutation_transition(
    state: &AppState,
    user_id: &auth::UserId,
    record: &RuntimeParameterMutationRecord,
) -> Result<(), (StatusCode, String)>
```

显式输入面候选:

```text
auth
io_error
mutation_event_contract
persist_runtime_parameter_mutation_record
AppState
FrontendRuntimeEvent
RuntimeParameterMutationLifecycleEntry
RuntimeParameterMutationRecord
RuntimeParameterMutationStatus
StatusCode
```

预期 BE-001EA-03 import:

```rust
use crate::{
    auth, io_error, mutation_event_contract, persist_runtime_parameter_mutation_record, AppState,
    FrontendRuntimeEvent, RuntimeParameterMutationLifecycleEntry, RuntimeParameterMutationRecord,
    RuntimeParameterMutationStatus,
};
use axum::http::StatusCode;
```

---

## 等价语义

必须保持不变:

1. `mutation_lifecycle_entry` 仍通过 `mutation_event_contract(status)` 取得 reason code。
2. lifecycle entry 的 `status`、`event_id`、`sequence_no`、`occurred_at_ms`、`reason_code`、`message` 映射不变。
3. `event_id` 仍来自 `event.event_id.clone()`。
4. `occurred_at_ms` 仍来自 `event.event_time_ms`。
5. `message` 仍通过 `message.into()` 写入。
6. `persist_runtime_parameter_mutation_transition` 仍先调用 `persist_runtime_parameter_mutation_record(state.mutation_store_dir.as_ref(), record).await.map_err(io_error)?`。
7. in-memory ledger 仍写入 `state.parameter_mutations.write().await.insert(auth::scoped_key(user_id, &record.proposal_id), record.clone())`。
8. 返回类型仍为 `Result<(), (StatusCode, String)>`。
9. activation / rollback flow 的调用面不改。
10. release transition 未启动，未新增 sibling horizontal link。

ASCII guard:

```text
no_code_movement
no_function_body_change
no_visibility_change
no_parent_facade_rewrite
no_activation_flow_rewrite
no_rollback_flow_rewrite
no_sibling_horizontal_link
no_release_transition
old_three_leaf_pause_target_cancelled
```

---

## 影响边界

BE-001EA-01 只冻结 `transition_record_persistence.rs` 的 import 输入面。
不得触碰:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs
src/runtime/mutation/parameter_mutation.rs
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
BE-001EA-02
runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass
抽离方案
```

BE-001EA-02 必须固定 BE-001EA-03 的单文件 import rewrite 边界，不得直接改 Rust。

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

AI 声称 BE-001EA-01 完成时，必须说明:

1. 本批是 `no code movement`。
2. 冻结文件是 `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs`。
3. 当前 residual 是 `use super::*`。
4. helper 是 `mutation_lifecycle_entry` 与 `persist_runtime_parameter_mutation_transition`。
5. 当前 residual 仍为 total 18 / mutation 16 / parameter_mutation 6 / transition_lifecycle 5。
6. 下一步只能进入 BE-001EA-02 抽离方案。
7. 旧三叶暂停目标保持取消，递归流继续干净推进。

不得宣称 transition_record_persistence import 已改写、transition_lifecycle_import_pass 已完成、parameter_mutation_import_pass 已完成、mutation_import_pass 已完成、parent_import_bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `381-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 冻结 `transition_record_persistence.rs` 当前输入面与等价语义。
3. 下一步固定为 BE-001EA-02 抽离方案。
4. 不恢复旧三叶暂停目标。
5. Rust / 治理 / 全量树门禁均通过。
