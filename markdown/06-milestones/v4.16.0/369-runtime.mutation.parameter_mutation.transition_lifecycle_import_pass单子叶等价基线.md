# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DV-01
> 基准: `368-runtime.mutation.parameter_mutation_import_pass第二轮父叶残余判断.md`
> 目标子叶: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DV-02 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DV-01 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | lifecycle equivalence、parent bridge residual accounting、父子通信硬规则、public handler 白箱 | 输入面冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` | transition lifecycle 白箱登记 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` | 建立单子叶基线 |

---

## 基线冻结

```text
runtime.mutation.parameter_mutation.transition_lifecycle_import_pass
transition_lifecycle_import_pass baseline_frozen
frozen_transition_lifecycle_residual_7_files
remaining_parent_import_bridge_20
remaining_mutation_import_bridge_18
remaining_parameter_mutation_import_bridge_8
old_three_leaf_pause_target_cancelled
```

冻结文件:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs
```

当前 parent bridge residual:

```text
use super::*
transition_lifecycle_facade_parent_wildcard_import
activation_flow_parent_wildcard_import
activation_snapshot_side_effect_parent_wildcard_import
boundary_safety_parent_wildcard_import
rollback_flow_parent_wildcard_import
rollback_record_identity_parent_wildcard_import
transition_record_persistence_parent_wildcard_import
```

---

## 白箱输入输出

public 输出:

| 输出 | 文件 | 当前可见性 | 上游调用 |
| --- | --- | --- | --- |
| `activate_runtime_parameter_mutation` | `activation_flow.rs` | `pub(crate)` re-export | `parameter_mutation.rs` parent facade、runtime routes |
| `rollback_runtime_parameter_mutation` | `rollback_flow.rs` | `pub(crate)` re-export | `parameter_mutation.rs` parent facade、runtime routes |

父级白箱 helper:

| helper | 文件 | 当前可见性 | 上游调用 |
| --- | --- | --- | --- |
| `validate_runtime_parameter_mutation_boundary` | `transition_lifecycle.rs` -> `boundary_safety.rs` | facade `pub(super)` | `proposal_creation.rs` 通过 parent white-box path |

内部 helper:

| helper | 文件 | 作用 |
| --- | --- | --- |
| `resolve_runtime_parameter_mutation_boundary` | `boundary_safety.rs` | activation / rollback 显式边界解析 |
| `evaluate_runtime_parameter_mutation_safe_window` | `boundary_safety.rs` | activation / rollback safe window 判定 |
| `auto_snapshot_on_activation` | `activation_snapshot_side_effect.rs` | activation 后自动签名快照与 generation side effect |
| `runtime_parameter_mutation_rollback_record_id` | `rollback_record_identity.rs` | rollback record deterministic id |
| `mutation_lifecycle_entry` | `transition_record_persistence.rs` | lifecycle entry 事件映射 |
| `persist_runtime_parameter_mutation_transition` | `transition_record_persistence.rs` | mutation record persistence 与 in-memory cache 同步 |

---

## 等价语义

必须保持不变:

1. `activate_runtime_parameter_mutation` capability guard、proposal status gate、source run loading、boundary resolution、safe window denial、schedule event、activation event、failure event、metrics、record persistence 和 auto snapshot 顺序。
2. `rollback_runtime_parameter_mutation` capability guard、activated-only gate、ledger rollback value resolution、noop rejection、boundary resolution、safe window denial、schedule event、rollback event、failure event、metrics 和 record persistence 顺序。
3. `validate_runtime_parameter_mutation_boundary` 对 `immediate` 的拒绝、`next_cycle_start` / `manual_pause` / `sequence_cursor` 的接受规则和中文错误文案。
4. `resolve_runtime_parameter_mutation_boundary` 对 `next_cycle_start` 的 `current_sequence_no + 2`、`manual_pause` 的 `None` 和 `sequence_cursor` 的 resolved sequence 行为。
5. `evaluate_runtime_parameter_mutation_safe_window` 的 reason code、message、retryable、retry_after_ms 与 snapshot 回填。
6. `auto_snapshot_on_activation` 的 generation 增量、历史截断、snapshot id、signature fallback、atomic write 和 `auth::scoped_key` cache key。
7. `mutation_lifecycle_entry` 仍通过 `mutation_event_contract` 派生 reason code。
8. `persist_runtime_parameter_mutation_transition` 仍先写 store，再写 `state.parameter_mutations` cache。
9. release transition 未启动，未新增 sibling horizontal link。

---

## 风险与方案前置判断

BE-001DV-02 必须先决定:

```text
same_batch_transition_lifecycle_7_files
or
split_transition_lifecycle_micro_pockets
```

当前不直接整批改写 Rust 的原因:

1. 7 文件覆盖 activation、rollback、boundary、snapshot side effect、record id、persistence 多职责。
2. `activation_flow.rs` 与 `rollback_flow.rs` 是 public handler 子叶，涉及 async state、event append、metrics 和持久化。
3. `boundary_safety.rs` 同时为 proposal creation 提供父级白箱 helper，import 路径必须先固定。
4. 如果 BE-001DV-02 发现同批 rewrite 会扩大等价风险，应继续拆为更小 pocket；这属于干净递归，不需要恢复三叶暂停目标。

---

## 下一步边界

下一步只能进入:

```text
BE-001DV-02
runtime.mutation.parameter_mutation.transition_lifecycle_import_pass
抽离方案
```

BE-001DV-02 不得直接改 Rust；必须先明确:

1. 是否同批处理 7 个 transition lifecycle residual 文件。
2. 是否需要先拆 `boundary_safety_import_pass`、`transition_record_persistence_import_pass`、`rollback_record_identity_import_pass` 等微 pocket。
3. 是否需要调整 helper visibility；若需要，必须说明父子通信边界，禁止 sibling horizontal link。
4. 排除 `parameter_mutation.rs` parent facade、`proposal_creation.rs`、`record_query.rs`、`ai_proposal`、root bridge、test-only `run_guard` 和 release transition。

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

AI 声称 BE-001DV-01 完成时，必须说明:

1. 本批是 `no code movement`。
2. 冻结 7 个 transition lifecycle residual 文件。
3. public handler 是 `activate_runtime_parameter_mutation` 与 `rollback_runtime_parameter_mutation`。
4. 父级白箱 helper 是 `validate_runtime_parameter_mutation_boundary`。
5. 当前 parent bridge 仍为 total 20 / mutation 18 / parameter_mutation 8。
6. 下一步只能进入 BE-001DV-02 抽离方案。
7. 旧三叶暂停目标保持取消。

不得宣称 transition lifecycle 已抽离、parameter mutation import 已完成、mutation import 已完成、parent import bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `369-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 冻结 7 文件 transition lifecycle 当前输入面。
3. 下一步固定为 BE-001DV-02 抽离方案。
4. 不恢复旧三叶暂停目标。
5. Rust / 治理 / 全量树门禁均通过。
