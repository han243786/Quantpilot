# v4.16.0 runtime.mutation.parameter_mutation_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DQ-02
> 基准: `357-runtime.mutation.parameter_mutation_import_pass单子叶等价基线.md`
> 目标父叶: `runtime.mutation.parameter_mutation_import_pass`
> 判定: `runtime.mutation.parameter_mutation_import_pass stop_split: false`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DR-01 `runtime.mutation.parameter_mutation.record_query_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DQ-02 `runtime.mutation.parameter_mutation_import_pass` 抽离方案 | 方案优化 |
| 规范矩阵 | parent import bridge、minimum batch、explicit import pass、release transition guard | 拒绝 10 文件整批 rewrite |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation_import_pass` | 下一 pocket 选择 |
| 模块树 | `runtime.mutation.parameter_mutation_import_pass` | `stop_split: false` |

---

## 当前事实

BE-001DQ-01 已冻结 10 个 parameter mutation residual 文件:

```text
src/runtime/mutation/parameter_mutation.rs
src/runtime/mutation/parameter_mutation/proposal_creation.rs
src/runtime/mutation/parameter_mutation/record_query.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs
```

当前 parent bridge 剩余仍为:

```text
root 1
run 0
backtest 0
mutation 20
test-only 1
total 22
remaining_parent_import_bridge_22
remaining_mutation_import_bridge_20
```

---

## 适配性校验

本父叶存在三类风险:

1. `proposal_creation` 会创建 proposal、追加 run event、写 mutation store、更新 state cache。
2. `transition_lifecycle` 会调度 activation / rollback、评估 safe window、追加 lifecycle、触发 activation snapshot 副作用。
3. parent facade `parameter_mutation.rs` 当前还承担子模块之间的桥接输入面，过早改写可能让仍未显式 import 的 child 失去父级白箱。

因此本批拒绝 10 文件整批改写:

```text
reject_parameter_mutation_bulk_rewrite_10_files
runtime.mutation.parameter_mutation_import_pass stop_split: false
old_three_leaf_pause_target_cancelled
```

---

## 候选比较

| 候选 | 文件范围 | 风险 | 判定 |
| --- | --- | --- | --- |
| `runtime.mutation.parameter_mutation.record_query_import_pass` | `record_query.rs` | 只读 list/detail，依赖面窄 | 采纳 |
| `runtime.mutation.parameter_mutation.proposal_creation_import_pass` | `proposal_creation.rs` | 写 proposal、event append、state cache | 延后 |
| `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` | `transition_lifecycle.rs` 与 6 个 child | activation / rollback / snapshot 副作用重 | 延后，后续可能继续细拆 |
| `runtime.mutation.parameter_mutation.facade_import_pass` | `parameter_mutation.rs` | parent facade 当前仍是 child import bridge | 最后处理 |

---

## 采纳方案

下一步固定为:

```text
BE-001DR-01 runtime.mutation.parameter_mutation.record_query_import_pass 单子叶等价基线
```

理由:

1. `record_query.rs` 只包含 `list_runtime_parameter_mutations` 与 `get_runtime_parameter_mutation_detail` 两个读路径 handler。
2. 它不改变 proposal 创建、activation、rollback、safe window、event append、snapshot 或 persistence 写入语义。
3. 它能在不触碰 parent facade 的情况下先消除一个 child 的 `use super::*`。
4. 它为后续处理 `proposal_creation` 与 `transition_lifecycle` 提供低风险 import 模板。

后续 BE-001DR-03 的实际改写范围必须只允许:

```text
src/runtime/mutation/parameter_mutation/record_query.rs
```

---

## 预期显式输入面

BE-001DR-01 需要冻结，BE-001DR-02 需要复核，BE-001DR-03 才允许使用类似输入面:

```rust
use crate::{
    auth, clean_optional_filter, io_error, load_runtime_parameter_mutation_record,
    list_runtime_parameter_mutation_records, paginate, AppState, PaginatedResponse,
    PaginationQuery, RuntimeParameterMutationListQuery, RuntimeParameterMutationRecord,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
```

实际实现时以 `cargo fmt` 和编译结果为准；不得为了减少 import 而恢复 wildcard import。

---

## 等价边界

后续 record query import pass 必须保持:

1. `list_runtime_parameter_mutations` 与 `get_runtime_parameter_mutation_detail` handler signature 不变。
2. list 过滤 `source_kind`、`source_id`、排序、pagination 行为不变。
3. detail 查询的 state cache 优先、store fallback、`auth::scoped_key` scope 语义不变。
4. 不改变 `RuntimeParameterMutationListQuery`、`PaginatedResponse` 或 `RuntimeParameterMutationRecord` schema。
5. 不触碰 proposal creation、activation、rollback、safe window、snapshot 或 mutation store 写入 owner。
6. 不新增 sibling horizontal link，不启动 release transition。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不直接改写 `record_query.rs` import。
- 本批不处理 `proposal_creation.rs`。
- 本批不处理 `transition_lifecycle.rs` 或其 6 个 child。
- 本批不处理 `parameter_mutation.rs` parent facade。
- 本批不处理 `ai_proposal`、`src/runtime/mod.rs` 或 test-only `src/runtime/run_guard.rs`。
- 本批不迁移 state owner、persistence owner、schema owner、frontend caller 或 test asset。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。

---

## 验证要求

本批为 `no code movement` 抽离方案，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

后续实际 import pass 至少补跑:

```powershell
cargo test -p quantpilot --test api_mutation
```

---

## 幻觉检查点

AI 声称 BE-001DQ-02 完成时，必须说明:

1. 本批次是 `no code movement` 抽离方案。
2. `runtime.mutation.parameter_mutation_import_pass stop_split: false`。
3. 本批拒绝 `reject_parameter_mutation_bulk_rewrite_10_files`。
4. 下一步只能进入 BE-001DR-01 `runtime.mutation.parameter_mutation.record_query_import_pass` 单子叶等价基线。
5. `record_query.rs` 尚未改写，当前 parent bridge 剩余仍为 total 22、mutation 20。
6. `proposal_creation`、`transition_lifecycle`、parent facade、`ai_proposal`、root bridge 与 test-only run_guard 尚未处理。
7. release transition 未启动，未新增 sibling horizontal link。
8. 旧的三叶暂停目标仍为取消状态。

不得宣称 parameter mutation import 已改写、mutation import 已完成、parent import bridge 已完全清除、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `358-runtime.mutation.parameter_mutation_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶设置 `runtime.mutation.parameter_mutation_import_pass stop_split: false`。
3. 10 文件整批 rewrite 被明确拒绝。
4. 下一步固定为 BE-001DR-01 `runtime.mutation.parameter_mutation.record_query_import_pass` 单子叶等价基线。
5. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
