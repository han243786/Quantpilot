# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EC-02
> 基线: `386-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass`
> 代码动作: no code movement
> 下一步: BE-001EC-03 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EC-02 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 抽离方案 | 实施边界冻结 |
| 规范矩阵 | staged explicit import pass / single-file rewrite / parent-child communication | 改写约束冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` | activation snapshot import rewrite 指令 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` | 固定下一步实际抽离范围 |

---

## 方案冻结

```text
runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass
activation_snapshot_side_effect_import_pass plan_frozen
single_file_activation_snapshot_side_effect_import_rewrite
be_001ec_03_only_rewrite_activation_snapshot_side_effect_imports
remaining_parent_import_bridge_17
remaining_mutation_import_bridge_15
remaining_parameter_mutation_import_bridge_5
remaining_transition_lifecycle_import_bridge_4
old_three_leaf_pause_target_cancelled
```

BE-001EC-03 只允许改写一个文件的顶部 import:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs
```

当前 import:

```rust
use super::*;
```

预期 import:

```rust
use crate::{
    auth, current_time_ms, AppState, DeploymentSignatureSnapshot, EventSliceBounds,
    RuntimeParameterMutationRecord,
};
```

`qrpc_runtime::ConfigGenerationEntry`、`qrpc_core::canonical_json_sha256_digest`、`serde_json::json`、`crate::runtime_persistence::atomic_write_json` 与 `std::sync::atomic::Ordering` 继续保持完全限定调用；`safe_eprintln!` 继续作为 crate 内宏调用。

---

## 禁止改写范围

BE-001EC-03 不得触碰:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
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

ASCII guard:

```text
no_function_body_change
no_visibility_change
no_parent_facade_rewrite
no_activation_flow_rewrite
no_rollback_flow_rewrite
no_transition_lifecycle_facade_rewrite
no_snapshot_persistence_rewrite
no_sibling_horizontal_link
no_release_transition
old_three_leaf_pause_target_cancelled
```

---

## 等价清单

BE-001EC-03 必须保持:

1. `auto_snapshot_on_activation` 名称、签名、`pub(super)` 可见性不变。
2. `now_ms` 仍来自 `current_time_ms()`。
3. config generation 仍通过 `state.config_generation.fetch_add(1, std::sync::atomic::Ordering::SeqCst)` 递增。
4. generation history lock 仍通过 `state.config_generation_history.lock().await` 取得。
5. generation history 仍追加 `qrpc_runtime::ConfigGenerationEntry`。
6. generation history 上限仍为 `MAX_GENERATION_HISTORY: usize = 100`。
7. overflow 仍通过 `history.drain(0..overflow)` 清理。
8. pre-activation metrics baseline 仍只读取 `_pre_activation_risk_reject` 与 `_pre_activation_rollback`。
9. observation deadline 仍为 `now_ms.saturating_add(60_000)`。
10. `snapshot_id` 仍为 `snap-auto-{now_ms}`。
11. `DeploymentSignatureSnapshot` 字段映射不变。
12. `EventSliceBounds` 字段映射不变。
13. signature 仍使用 `qrpc_core::canonical_json_sha256_digest(&serde_json::json!(...))`，失败 fallback 仍为 `signature-unavailable`。
14. snapshot path 仍为 `state.snapshot_store_dir.join(format!("{}.json", snapshot_id))`。
15. 持久化仍调用 `crate::runtime_persistence::atomic_write_json(&path, &snapshot).await`。
16. 写入失败仍只通过 `safe_eprintln!` 记录，不改变返回类型。
17. memory snapshot map 仍写入 `state.snapshots`、`auth::scoped_key(user_id, &snapshot_id)` 和 `snapshot`。
18. `transition_lifecycle.rs` parent facade 与 activation flow 调用面不变。
19. 不启动发布过渡，不引入 sibling horizontal link。

---

## 预期残余变化

BE-001EC-03 完成后预期:

```text
actual_parent_import_bridge_17_to_16
actual_mutation_import_bridge_15_to_14
actual_parameter_mutation_import_bridge_5_to_4
actual_transition_lifecycle_import_bridge_4_to_3
remaining_parent_import_bridge_16
remaining_mutation_import_bridge_14
remaining_parameter_mutation_import_bridge_4
remaining_transition_lifecycle_import_bridge_3
```

如果实际统计与预期不一致，BE-001EC-03 必须停在记录阶段说明差异，不得顺手扩大改写范围。

---

## 下一步边界

下一步只能进入:

```text
BE-001EC-03
runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass
实际抽离记录
```

BE-001EC-03 完成后必须回到单叶 closeout，判断本 import pocket 是否值得继续细拆。

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

AI 声称 BE-001EC-02 完成时，必须说明:

1. 本批是 `no code movement`。
2. 下一步只允许改写 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs` 顶部 import。
3. 不得改函数体、可见性、parent facade、activation flow、rollback flow、snapshot persistence 或 sibling。
4. 当前 residual 仍为 total 17 / mutation 15 / parameter_mutation 5 / transition_lifecycle 4。
5. 旧三叶暂停目标保持取消: `old_three_leaf_pause_target_cancelled`。

不得宣称 activation_snapshot_side_effect import 已改写、transition_lifecycle_import_pass 已完成、parameter_mutation_import_pass 已完成、mutation_import_pass 已完成、parent_import_bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `387-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001EC-03 的单文件 import rewrite 边界被固定。
3. 不恢复旧三叶暂停目标。
4. Rust / 治理 / 全量树门禁均通过。
