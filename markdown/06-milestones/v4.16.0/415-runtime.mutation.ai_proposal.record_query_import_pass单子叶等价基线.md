# v4.16.0 runtime.mutation.ai_proposal.record_query_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EP-01
> 基线: `414-runtime.mutation.ai_proposal_import_pass抽离方案.md`
> 目标子叶: `runtime.mutation.ai_proposal.record_query_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.record_query_import_pass`
> 真实文件: `src/runtime/mutation/ai_proposal/record_query.rs`
> 代码动作: no code movement
> 下一步: BE-001EP-02 `runtime.mutation.ai_proposal.record_query_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EP-01 `runtime.mutation.ai_proposal.record_query_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | staged explicit import pass / minimum batch / no release transition | 单文件 import pocket |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.record_query_import_pass` | record query 白箱 |
| 模块树 | `runtime.mutation.ai_proposal.record_query_import_pass` | 新基线 |

---

## 当前事实

BE-001EO-02 已拒绝 10 文件整批 rewrite，并选择 record query 作为第一枚 ai proposal child pocket:

```text
runtime.mutation.ai_proposal_import_pass stop_split: false
reject_ai_proposal_bulk_rewrite_10_files
runtime.mutation.ai_proposal.record_query_import_pass baseline_frozen
old_three_leaf_pause_target_cancelled
```

当前 parent bridge 剩余:

```text
remaining_parent_import_bridge_12
remaining_mutation_import_bridge_10
remaining_ai_proposal_import_bridge_10
```

本批冻结 `record_query.rs`，不改写 Rust import。

---

## 目标文件范围

```text
src/runtime/mutation/ai_proposal/record_query.rs
```

当前文件顶部仍为:

```rust
use super::*;
```

---

## 白箱 public / helper 面

本基线冻结以下函数:

```text
load_runtime_ai_proposal_for_user
list_runtime_ai_proposals
get_runtime_ai_proposal_detail
```

语义边界:

1. `load_runtime_ai_proposal_for_user` 必须保持 state cache 优先、disk fallback 与 `auth::scoped_key` scope 语义。
2. `list_runtime_ai_proposals` 必须保持 disk list、`source_kind`、`source_id`、`status` 过滤与 `created_at_ms` / `ai_proposal_id` 排序。
3. `get_runtime_ai_proposal_detail` 必须保持 user scoped cache lookup 与 disk fallback。
4. route-facing handler signature、response schema 与错误映射不变。

---

## 当前隐式输入面

BE-001EP-02 需要复核，BE-001EP-03 才允许把 `use super::*` 收敛为显式 import。预期输入面至少包括:

```text
auth::UserId
State
Path
Query
Json
StatusCode
AppState
RuntimeAiProposalListQuery
RuntimeAiProposalRecord
RuntimeEvidenceSourceKind
auth::scoped_key
clean_optional_filter
io_error
load_runtime_ai_proposal_record
list_runtime_ai_proposal_records
```

实际实现以 `cargo check -p quantpilot` 为准，不得恢复 wildcard import。

---

## 不进入范围

本批不处理:

1. 不修改 `src/runtime/mutation/ai_proposal/record_query.rs`。
2. 不处理 `proposal_creation.rs`、`approval_review.rs`、`approval_persistence.rs`、`sandbox_trigger.rs`、`static_check.rs`、`source_governance_identity.rs`、`event_lifecycle.rs` 或 `status_transition.rs`。
3. 不处理 `ai_proposal.rs` parent facade。
4. 不处理 `src/runtime/mod.rs` root parent bridge。
5. 不处理 test-only `src/runtime/run_guard.rs`。
6. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
7. 不新增 sibling horizontal link。
8. 不启动 release transition。

---

## 验证要求

本批为 `no code movement` 基线，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_ai_proposal
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001EP-01 完成时，必须说明:

1. 本批次是 `no code movement` 单子叶等价基线。
2. 目标文件为 `src/runtime/mutation/ai_proposal/record_query.rs`。
3. `use super::*` 尚未改写。
4. 下一步只能进入 BE-001EP-02 抽离方案。
5. 当前 parent bridge 剩余仍为 total 12、mutation 10、ai proposal 10。
6. approval、sandbox、static-check、source governance、event lifecycle、proposal creation、status transition、parent facade、root bridge 与 test-only run_guard 尚未处理。
7. release transition 未启动，未新增 sibling horizontal link。
8. 旧的三叶暂停目标仍为取消状态。

不得宣称 record query import 已改写、ai proposal import 已完成、mutation import 已完成、parent import bridge 已完全清除、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `415-runtime.mutation.ai_proposal.record_query_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 基线冻结 `record_query.rs` 的 3 个函数与当前隐式输入面。
3. 下一步固定为 BE-001EP-02 `runtime.mutation.ai_proposal.record_query_import_pass` 抽离方案。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
