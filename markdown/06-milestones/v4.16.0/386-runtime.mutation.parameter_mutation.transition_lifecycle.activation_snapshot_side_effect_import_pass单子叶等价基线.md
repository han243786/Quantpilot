# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EC-01
> 基线: `385-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass第三轮父叶残余判断.md`
> 目标子叶: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass`
> 代码动作: no code movement
> 下一步: BE-001EC-02 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EC-01 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | staged explicit import pass / parent white-box helper / activation side effect contract | 输入面冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` | activation snapshot 白箱登记 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` | 建立单子叶基线 |

---

## 基线冻结

```text
runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass
activation_snapshot_side_effect_import_pass baseline_frozen
single_file_activation_snapshot_side_effect_import_pass
remaining_parent_import_bridge_17
remaining_mutation_import_bridge_15
remaining_parameter_mutation_import_bridge_5
remaining_transition_lifecycle_import_bridge_4
old_three_leaf_pause_target_cancelled
```

冻结文件:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs
```

当前 residual:

```rust
use super::*;
```

本批不移动代码、不改函数体、不改可见性、不改 activation flow 调用点、不改 snapshot persistence。

---

## 白箱输入输出

目标 helper:

| helper | 当前可见性 | 调用方 | 约束 |
| --- | --- | --- | --- |
| `auto_snapshot_on_activation` | `pub(super)` | `activation_flow.rs` 通过 `transition_lifecycle.rs` parent facade | 不改 activation 后自动快照 side effect |

函数签名必须保持:

```rust
pub(super) async fn auto_snapshot_on_activation(
    state: &AppState,
    user_id: &auth::UserId,
    mutation: &RuntimeParameterMutationRecord,
)
```

显式输入面候选:

```text
auth
current_time_ms
AppState
RuntimeParameterMutationRecord
DeploymentSignatureSnapshot
EventSliceBounds
qrpc_runtime::ConfigGenerationEntry
qrpc_core::canonical_json_sha256_digest
serde_json::json
crate::runtime_persistence::atomic_write_json
safe_eprintln!
std::sync::atomic::Ordering
state.config_generation_history.lock().await
state.snapshots
```

预期 BE-001EC-03 import:

```rust
use crate::{
    auth, current_time_ms, AppState, DeploymentSignatureSnapshot, EventSliceBounds,
    RuntimeParameterMutationRecord,
};
```

`qrpc_runtime::ConfigGenerationEntry`、`qrpc_core::canonical_json_sha256_digest`、`serde_json::json`、`crate::runtime_persistence::atomic_write_json` 与 `std::sync::atomic::Ordering` 当前保持完全限定调用；`safe_eprintln!` 保持 crate 内宏调用，不新增发布面。

---

## 等价语义

必须保持不变:

1. `now_ms` 仍来自 `current_time_ms()`。
2. config generation 仍通过 `state.config_generation.fetch_add(1, Ordering::SeqCst)` 递增。
3. generation history 仍追加 `qrpc_runtime::ConfigGenerationEntry`。
4. generation history 上限仍为 `MAX_GENERATION_HISTORY: usize = 100`。
5. overflow 仍通过 `history.drain(0..overflow)` 清理。
6. pre-activation metrics baseline 仍只读取 `_pre_activation_risk_reject` 与 `_pre_activation_rollback`。
7. observation deadline 仍为 `now_ms.saturating_add(60_000)`。
8. `snapshot_id` 仍为 `snap-auto-{now_ms}`。
9. `DeploymentSignatureSnapshot` 字段映射不变。
10. signature 仍使用 `qrpc_core::canonical_json_sha256_digest(&serde_json::json!(...))`，失败 fallback 仍为 `signature-unavailable`。
11. snapshot path 仍为 `state.snapshot_store_dir.join(format!("{}.json", snapshot_id))`。
12. 持久化仍调用 `crate::runtime_persistence::atomic_write_json(&path, &snapshot).await`。
13. 写入失败仍只通过 `safe_eprintln!` 记录，不改变返回类型。
14. generation history lock 仍通过 `state.config_generation_history.lock().await` 取得。
15. memory snapshot map 仍写入 `state.snapshots`、`auth::scoped_key(user_id, &snapshot_id)` 和 `snapshot`。
16. 不启动发布过渡，不引入 sibling horizontal link。

ASCII guard:

```text
no_code_movement
no_function_body_change
no_visibility_change
no_parent_facade_rewrite
no_activation_flow_rewrite
no_rollback_flow_rewrite
no_snapshot_persistence_rewrite
no_sibling_horizontal_link
no_release_transition
old_three_leaf_pause_target_cancelled
```

---

## 影响边界

BE-001EC-01 只冻结 `activation_snapshot_side_effect.rs` 的 import 输入面。
不得触碰:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs
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
BE-001EC-02
runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass
抽离方案
```

BE-001EC-02 必须固定 BE-001EC-03 的单文件 import rewrite 边界，不得直接改 Rust。

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

AI 声称 BE-001EC-01 完成时，必须说明:

1. 本批是 `no code movement`。
2. 冻结文件是 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs`。
3. 当前 residual 是 `use super::*`。
4. helper 是 `auto_snapshot_on_activation`。
5. 当前 residual 仍为 total 17 / mutation 15 / parameter_mutation 5 / transition_lifecycle 4。
6. 下一步只能进入 BE-001EC-02 抽离方案。
7. 旧三叶暂停目标保持取消，递归流继续干净推进。

不得宣称 activation_snapshot_side_effect import 已改写、transition_lifecycle_import_pass 已完成、parameter_mutation_import_pass 已完成、mutation_import_pass 已完成、parent_import_bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `386-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 冻结 `activation_snapshot_side_effect.rs` 当前输入面与等价语义。
3. 下一步固定为 BE-001EC-02 抽离方案。
4. 不恢复旧三叶暂停目标。
5. Rust / 治理 / 全量树门禁均通过。
