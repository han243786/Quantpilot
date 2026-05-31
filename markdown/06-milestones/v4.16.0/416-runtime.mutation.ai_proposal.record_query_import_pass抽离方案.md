# v4.16.0 runtime.mutation.ai_proposal.record_query_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EP-02
> 基线: `415-runtime.mutation.ai_proposal.record_query_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.mutation.ai_proposal.record_query_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.record_query_import_pass`
> 真实文件: `src/runtime/mutation/ai_proposal/record_query.rs`
> 代码动作: no code movement
> 下一步: BE-001EP-03 `runtime.mutation.ai_proposal.record_query_import_pass` 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EP-02 `runtime.mutation.ai_proposal.record_query_import_pass` 抽离方案 | 方案优化 |
| 规范矩阵 | staged explicit import pass / single file import rewrite / no release transition | 固定最小实施单元 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.record_query_import_pass` | record query import pocket |
| 模块树 | `runtime.mutation.ai_proposal.record_query_import_pass` | 抽离方案 |

---

## 当前事实

BE-001EP-01 已冻结 `src/runtime/mutation/ai_proposal/record_query.rs` 的等价基线。本批只把后续实际改写范围落成方案，不改 Rust。

```text
BE-001EP-02
BE-001EP-03
runtime.mutation.ai_proposal.record_query_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.record_query_import_pass
record_query_import_pass plan_frozen
single file import rewrite
old_three_leaf_pause_target_cancelled
```

当前目标文件顶部仍为:

```rust
use super::*;
```

---

## 采纳方案

BE-001EP-03 只能改写以下单文件顶部 import:

```text
src/runtime/mutation/ai_proposal/record_query.rs
```

目标是把 `use super::*` 收敛为显式输入面。不得改写函数体、handler signature、返回类型、错误映射、查询过滤、排序、state cache 优先级或 disk fallback。

预期显式输入面:

```rust
use crate::{
    auth, clean_optional_filter, io_error, load_runtime_ai_proposal_record,
    list_runtime_ai_proposal_records, AppState, RuntimeAiProposalListQuery,
    RuntimeAiProposalRecord,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
```

实际以 `cargo fmt --check`、`cargo check -p quantpilot` 和 `cargo test -p quantpilot --test api_ai_proposal` 为准；如编译提示缺口，只允许补充显式 import，不得恢复 wildcard import。

---

## 等价边界

BE-001EP-03 必须保持以下函数行为不变:

```text
load_runtime_ai_proposal_for_user
list_runtime_ai_proposals
get_runtime_ai_proposal_detail
```

必须保持:

1. `load_runtime_ai_proposal_for_user` 的 `auth::scoped_key`、state cache 优先和 disk fallback 不变。
2. `list_runtime_ai_proposals` 的 disk list、`source_kind`、`source_id`、`status` 过滤不变。
3. `list_runtime_ai_proposals` 的 `created_at_ms` 降序与 `ai_proposal_id` tie-break 排序不变。
4. `get_runtime_ai_proposal_detail` 的 user scoped cache lookup 和 disk fallback 不变。
5. `RuntimeAiProposalListQuery` 与 `RuntimeAiProposalRecord` schema 不变。
6. route-facing handler signature 与 response schema 不变。

```text
no_handler_signature_change
no_query_filter_rewrite
no_state_cache_rewrite
no_disk_fallback_rewrite
no_sibling_owner_migration
```

---

## 预期残余变化

BE-001EP-03 完成后，预期生产 import residual 从:

```text
remaining_parent_import_bridge_12
remaining_mutation_import_bridge_10
remaining_ai_proposal_import_bridge_10
```

下降为:

```text
expected_remaining_parent_import_bridge_11
expected_remaining_mutation_import_bridge_9
expected_remaining_ai_proposal_import_bridge_9
```

test-only `src/runtime/run_guard.rs` 不纳入本批生产 residual 统计。

---

## 排除项

本批不处理:

1. 不修改 Rust 代码。
2. 不改写 `src/runtime/mutation/ai_proposal/record_query.rs` 的函数体。
3. 不处理 `proposal_creation.rs`、`approval_review.rs`、`approval_persistence.rs`、`sandbox_trigger.rs`、`static_check.rs`、`source_governance_identity.rs`、`event_lifecycle.rs` 或 `status_transition.rs`。
4. 不处理 `src/runtime/mutation/ai_proposal.rs` parent facade。
5. 不处理 `src/runtime/mod.rs` root parent bridge。
6. 不处理 test-only `src/runtime/run_guard.rs`。
7. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
8. 不新增 sibling horizontal link。
9. 不启动 release transition。

---

## 验证要求

本批为 `no code movement` 抽离方案，提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_ai_proposal
```

---

## 幻觉检查点

AI 声称 BE-001EP-02 完成时，必须说明:

1. 本批只是 `no code movement` 抽离方案。
2. BE-001EP-03 只能改写 `src/runtime/mutation/ai_proposal/record_query.rs` 顶部 import。
3. `use super::*` 尚未改写。
4. 预期实际抽离后 residual 降为 total 11、mutation 9、ai proposal 9。
5. 不得宣称 record query import 已完成、ai proposal import 已完成、mutation import 已完成、parent import bridge 已完全清除、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `416-runtime.mutation.ai_proposal.record_query_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001EP-03 的目标文件、显式 import 输入面、等价边界和排除项被固定。
3. 下一步固定为 BE-001EP-03 `runtime.mutation.ai_proposal.record_query_import_pass` 实际抽离记录。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
