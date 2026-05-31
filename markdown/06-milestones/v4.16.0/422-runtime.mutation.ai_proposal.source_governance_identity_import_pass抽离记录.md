# v4.16.0 runtime.mutation.ai_proposal.source_governance_identity_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001ER-03
> 基线: `421-runtime.mutation.ai_proposal.source_governance_identity_import_pass抽离方案.md`
> 目标子叶: `runtime.mutation.ai_proposal.source_governance_identity_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.source_governance_identity_import_pass`
> 真实文件: `src/runtime/mutation/ai_proposal/source_governance_identity.rs`
> 代码动作: actual import extraction
> 下一步: BE-001ER-04 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001ER-03 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 抽离记录 | 实际抽离 |
| 规范矩阵 | explicit import pass / single file import rewrite / no release transition | 删除父级 wildcard import |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.source_governance_identity_import_pass` | source governance import 输入显式化 |
| 模块树 | `runtime.mutation.ai_proposal.source_governance_identity_import_pass` | 抽离完成，待 closeout |

---

## 抽离事实

本批只改写:

```text
src/runtime/mutation/ai_proposal/source_governance_identity.rs
```

完成动作:

```text
BE-001ER-03
runtime.mutation.ai_proposal.source_governance_identity_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.source_governance_identity_import_pass
source_governance_identity_import_pass extraction_done
removed use super::*
single file import rewrite
old_three_leaf_pause_target_cancelled
```

`source_governance_identity.rs` 已从 parent wildcard import 收敛为显式输入面:

```rust
use crate::{
    auth, canonical_json_sha256_digest, internal_error, load_backtest_record_from_state,
    load_run_record_from_state, AppState, CreateRuntimeAiProposalRequest,
    RuntimeAiProposalGovernance, RuntimeEvidenceSourceKind, RuntimeGovernanceSnapshot,
};
use axum::http::StatusCode;
use serde_json::json;
```

---

## 等价边界

以下类型与函数只完成 import 输入显式化，函数体、可见性和字段未改:

```text
RuntimeAiProposalSourceContext
load_runtime_ai_proposal_source_context
runtime_ai_proposal_governance
runtime_ai_proposal_record_id
```

本批保持:

```text
no_source_kind_branch_rewrite
no_sequence_no_rewrite
no_governance_projection_rewrite
no_record_id_rewrite
no_sibling_owner_migration
```

具体语义:

1. Run / Backtest source kind 分支未改。
2. `current_sequence_no` last event sequence fallback 未改。
3. `RuntimeAiProposalSourceContext` 字段和可见性未改。
4. source governance 到 AI proposal governance 的字段投影未改。
5. canonical JSON digest 输入字段、`digest.value[..12]` id suffix 和 id 格式未改。

---

## 残余更新

本批完成后，生产 parent import bridge 预期下降为:

```text
remaining_parent_import_bridge_10
remaining_mutation_import_bridge_8
remaining_ai_proposal_import_bridge_8
```

`src/runtime/run_guard.rs` 的 test-only `use super::*` 不纳入生产清理目标。

---

## 排除项

本批未处理:

1. 不改 `static_check.rs`、`event_lifecycle.rs`、`approval_persistence.rs`、`approval_review.rs`、`sandbox_trigger.rs`、`proposal_creation.rs`、`status_transition.rs` 或 parent facade。
2. 不改 `src/runtime/mod.rs` root parent bridge。
3. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
4. 不新增 sibling horizontal link。
5. 不启动 release transition。

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

AI 声称 BE-001ER-03 完成时，必须说明:

1. 本批只完成 `source_governance_identity.rs` 顶部 import 显式化。
2. struct 字段、函数体、source kind 分支、sequence fallback、governance projection 与 record id 未改。
3. 当前 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 尚未 closeout，下一步只能进入 BE-001ER-04 单叶 closeout。
4. 不得宣称 ai proposal import、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `422-runtime.mutation.ai_proposal.source_governance_identity_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/mutation/ai_proposal/source_governance_identity.rs` 不再依赖 parent wildcard import。
3. BE-001ER-04 单叶 closeout 成为唯一下一步。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
