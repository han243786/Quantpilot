# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DY-01
> 基线: `375-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass父叶残余判断.md`
> 目标子叶: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DY-02 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DY-01 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | staged explicit import pass / parent white-box helper / deterministic id contract | 输入面冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` | rollback identity 白箱登记 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` | 建立单子叶基线 |

---

## 基线冻结

```text
runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass
rollback_record_identity_import_pass baseline_frozen
single_file_rollback_record_identity_import_pass
remaining_parent_import_bridge_19
remaining_mutation_import_bridge_17
remaining_parameter_mutation_import_bridge_7
remaining_transition_lifecycle_import_bridge_6
old_three_leaf_pause_target_cancelled
```

冻结文件:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs
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
| `runtime_parameter_mutation_rollback_record_id` | `pub(super)` | `transition_lifecycle.rs` parent facade / rollback flow 经父级白箱调用 | 不改 rollback proposal id 生成语义 |

函数签名必须保持:

```rust
pub(super) fn runtime_parameter_mutation_rollback_record_id(
    source_id: &str,
    rollback_of: &str,
    target: &RuntimeParameterMutationTarget,
    created_at_ms: u64,
    source_event_count: usize,
    proposed_parameter_version: &str,
) -> Result<String, (StatusCode, String)>
```

显式输入面候选:

```text
canonical_json_sha256_digest
internal_error
RuntimeParameterMutationTarget
StatusCode
json!
anyhow::anyhow
```

预期 BE-001DY-03 import:

```rust
use crate::{canonical_json_sha256_digest, internal_error, RuntimeParameterMutationTarget};
use axum::http::StatusCode;
use serde_json::json;
```

`anyhow::anyhow!` 保持完全限定调用，不单独引入 `use anyhow`。

---

## 等价语义

必须保持不变:

1. digest 仍通过 `canonical_json_sha256_digest` 生成。
2. digest input 仍由 `json!` 构造。
3. digest input key 仍覆盖 `created_at_ms`、`rollback_of`、`source_event_count`、`source_id`、`target`、`proposed_parameter_version`。
4. digest error 仍映射为 `internal_error(anyhow::anyhow!(error))`。
5. 输出 prefix 仍为 `parameter_rollback_`。
6. 输出格式仍为 `parameter_rollback_{created_at_ms}_{digest.value[..12]}`。
7. 返回类型仍为 `Result<String, (StatusCode, String)>`。
8. `RuntimeParameterMutationTarget` 序列化语义不改。
9. 不改变 activation / rollback transition record 的持久化、事件、状态写入或校验。

ASCII guard:

```text
no_code_movement
no_function_body_change
no_visibility_change
no_parent_facade_rewrite
no_rollback_flow_rewrite
no_sibling_horizontal_link
no_release_transition
old_three_leaf_pause_target_cancelled
```

---

## 影响边界

BE-001DY-01 只冻结 `rollback_record_identity.rs` 的 import 输入面。
不得触碰:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
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
BE-001DY-02
runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass
抽离方案
```

BE-001DY-02 必须固定 BE-001DY-03 的单文件 import rewrite 边界，不得直接改 Rust。

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

AI 声称 BE-001DY-01 完成时，必须说明:

1. 本批是 `no code movement`。
2. 冻结文件是 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs`。
3. 当前 residual 是 `use super::*`。
4. helper 是 `runtime_parameter_mutation_rollback_record_id`。
5. 当前 residual 仍为 total 19 / mutation 17 / parameter_mutation 7 / transition_lifecycle 6。
6. 下一步只能进入 BE-001DY-02 抽离方案。
7. 旧三叶暂停目标保持取消，递归流继续干净推进。

不得宣称 rollback_record_identity import 已改写、transition_lifecycle_import_pass 已完成、parameter_mutation_import_pass 已完成、mutation_import_pass 已完成、parent_import_bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `376-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 冻结 `rollback_record_identity.rs` 当前输入面与等价语义。
3. 下一步固定为 BE-001DY-02 抽离方案。
4. 不恢复旧三叶暂停目标。
5. Rust / 治理 / 全量树门禁均通过。
