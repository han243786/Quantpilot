# v4.16.0 runtime.mutation.ai_proposal.record_query_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EP-03
> 基线: `416-runtime.mutation.ai_proposal.record_query_import_pass抽离方案.md`
> 目标子叶: `runtime.mutation.ai_proposal.record_query_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.record_query_import_pass`
> 真实文件: `src/runtime/mutation/ai_proposal/record_query.rs`
> 代码动作: actual import extraction
> 下一步: BE-001EP-04 `runtime.mutation.ai_proposal.record_query_import_pass` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EP-03 `runtime.mutation.ai_proposal.record_query_import_pass` 抽离记录 | 实际抽离 |
| 规范矩阵 | explicit import pass / single file import rewrite / no release transition | 删除父级 wildcard import |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.record_query_import_pass` | record query import 输入显式化 |
| 模块树 | `runtime.mutation.ai_proposal.record_query_import_pass` | 抽离完成，待 closeout |

---

## 抽离事实

本批只改写:

```text
src/runtime/mutation/ai_proposal/record_query.rs
```

完成动作:

```text
BE-001EP-03
runtime.mutation.ai_proposal.record_query_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.record_query_import_pass
record_query_import_pass extraction_done
removed use super::*
single file import rewrite
old_three_leaf_pause_target_cancelled
```

`record_query.rs` 已从 parent wildcard import 收敛为显式输入面:

```rust
use crate::{
    auth, io_error, list_runtime_ai_proposal_records, load_runtime_ai_proposal_record,
    runtime::{clean_optional_filter, RuntimeAiProposalListQuery},
    AppState, RuntimeAiProposalRecord,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
```

---

## 等价边界

以下函数只完成 import 输入显式化，函数体、可见性和签名未改:

```text
load_runtime_ai_proposal_for_user
list_runtime_ai_proposals
get_runtime_ai_proposal_detail
```

本批保持:

```text
no_handler_signature_change
no_query_filter_rewrite
no_state_cache_rewrite
no_disk_fallback_rewrite
no_sibling_owner_migration
```

具体语义:

1. `load_runtime_ai_proposal_for_user` 仍保持 `auth::scoped_key`、state cache 优先和 disk fallback。
2. `list_runtime_ai_proposals` 仍保持 disk list、`source_kind`、`source_id`、`status` 过滤。
3. `list_runtime_ai_proposals` 仍保持 `created_at_ms` 降序和 `ai_proposal_id` tie-break 排序。
4. `get_runtime_ai_proposal_detail` 仍保持 user scoped cache lookup 和 disk fallback。
5. `RuntimeAiProposalListQuery` 与 `RuntimeAiProposalRecord` schema 未改。

---

## 残余更新

本批完成后，生产 parent import bridge 预期下降为:

```text
remaining_parent_import_bridge_11
remaining_mutation_import_bridge_9
remaining_ai_proposal_import_bridge_9
```

`src/runtime/run_guard.rs` 的 test-only `use super::*` 不纳入生产清理目标。

---

## 排除项

本批未处理:

1. 不改 `proposal_creation.rs`、`approval_review.rs`、`approval_persistence.rs`、`sandbox_trigger.rs`、`static_check.rs`、`source_governance_identity.rs`、`event_lifecycle.rs` 或 `status_transition.rs`。
2. 不改 `src/runtime/mutation/ai_proposal.rs` parent facade。
3. 不改 `src/runtime/mod.rs` root parent bridge。
4. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
5. 不新增 sibling horizontal link。
6. 不启动 release transition。

---

## 验证要求

提交前至少执行:

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

AI 声称 BE-001EP-03 完成时，必须说明:

1. 本批只完成 `record_query.rs` 顶部 import 显式化。
2. 函数体、handler signature、查询过滤、state cache、disk fallback 与 schema 未改。
3. 当前 `runtime.mutation.ai_proposal.record_query_import_pass` 尚未 closeout，下一步只能进入 BE-001EP-04 单叶 closeout。
4. 不得宣称 ai proposal import、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `417-runtime.mutation.ai_proposal.record_query_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/mutation/ai_proposal/record_query.rs` 不再依赖 parent wildcard import。
3. BE-001EP-04 单叶 closeout 成为唯一下一步。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
