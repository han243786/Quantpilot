# v4.16.0 runtime.mutation.ai_proposal.source_governance_identity_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001ER-02
> 基线: `420-runtime.mutation.ai_proposal.source_governance_identity_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.mutation.ai_proposal.source_governance_identity_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.source_governance_identity_import_pass`
> 真实文件: `src/runtime/mutation/ai_proposal/source_governance_identity.rs`
> 代码动作: no code movement
> 下一步: BE-001ER-03 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001ER-02 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 抽离方案 | 方案优化 |
| 规范矩阵 | staged explicit import pass / single file import rewrite / no release transition | 固定最小实施单元 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.source_governance_identity_import_pass` | source governance import pocket |
| 模块树 | `runtime.mutation.ai_proposal.source_governance_identity_import_pass` | 抽离方案 |

---

## 当前事实

BE-001ER-01 已冻结 `src/runtime/mutation/ai_proposal/source_governance_identity.rs` 的等价基线。本批只把后续实际改写范围落成方案，不改 Rust。

```text
BE-001ER-02
BE-001ER-03
runtime.mutation.ai_proposal.source_governance_identity_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.source_governance_identity_import_pass
source_governance_identity_import_pass plan_frozen
single file import rewrite
old_three_leaf_pause_target_cancelled
```

当前目标文件顶部仍为:

```rust
use super::*;
```

---

## 采纳方案

BE-001ER-03 只能改写以下单文件顶部 import:

```text
src/runtime/mutation/ai_proposal/source_governance_identity.rs
```

目标是把 `use super::*` 收敛为显式输入面。不得改写 struct 字段、函数体、可见性、source kind 分支、sequence number fallback、governance projection 或 record id digest。

预期显式输入面:

```rust
use crate::{
    auth, canonical_json_sha256_digest, internal_error, load_backtest_record_from_state,
    load_run_record_from_state, AppState, CreateRuntimeAiProposalRequest,
    RuntimeAiProposalGovernance, RuntimeEvidenceSourceKind, RuntimeGovernanceSnapshot,
};
use axum::http::StatusCode;
use serde_json::json;
```

实际以 `cargo fmt --check`、`cargo check -p quantpilot` 和 `cargo test -p quantpilot --test api_ai_proposal` 为准；如编译提示缺口，只允许补充显式 import，不得恢复 wildcard import。

---

## 等价边界

BE-001ER-03 必须保持以下类型与函数行为不变:

```text
RuntimeAiProposalSourceContext
load_runtime_ai_proposal_source_context
runtime_ai_proposal_governance
runtime_ai_proposal_record_id
```

必须保持:

1. Run / Backtest source kind 分支不变。
2. `current_sequence_no` 的 last event sequence fallback 不变。
3. `RuntimeAiProposalSourceContext` 字段与可见性不变。
4. source governance 到 AI proposal governance 的字段投影不变。
5. canonical JSON digest 输入字段、顺序语义与 `digest.value[..12]` id suffix 不变。

```text
no_source_kind_branch_rewrite
no_sequence_no_rewrite
no_governance_projection_rewrite
no_record_id_rewrite
no_sibling_owner_migration
```

---

## 预期残余变化

BE-001ER-03 完成后，预期生产 import residual 从:

```text
remaining_parent_import_bridge_11
remaining_mutation_import_bridge_9
remaining_ai_proposal_import_bridge_9
```

下降为:

```text
expected_remaining_parent_import_bridge_10
expected_remaining_mutation_import_bridge_8
expected_remaining_ai_proposal_import_bridge_8
```

test-only `src/runtime/run_guard.rs` 不纳入本批生产 residual 统计。

---

## 排除项

本批不处理:

1. 不修改 Rust 代码。
2. 不改写 `source_governance_identity.rs` 的函数体或 struct 字段。
3. 不处理 `static_check.rs`、`event_lifecycle.rs`、`approval_persistence.rs`、`approval_review.rs`、`sandbox_trigger.rs`、`proposal_creation.rs`、`status_transition.rs` 或 parent facade。
4. 不处理 `src/runtime/mod.rs` root parent bridge。
5. 不处理 test-only `src/runtime/run_guard.rs`。
6. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
7. 不新增 sibling horizontal link。
8. 不启动 release transition。

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

AI 声称 BE-001ER-02 完成时，必须说明:

1. 本批只是 `no code movement` 抽离方案。
2. BE-001ER-03 只能改写 `src/runtime/mutation/ai_proposal/source_governance_identity.rs` 顶部 import。
3. `use super::*` 尚未改写。
4. 预期实际抽离后 residual 降为 total 10、mutation 8、ai proposal 8。
5. 不得宣称 source governance identity import 已完成、ai proposal import 已完成、mutation import 已完成、parent import bridge 已完全清除、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `421-runtime.mutation.ai_proposal.source_governance_identity_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001ER-03 的目标文件、显式 import 输入面、等价边界和排除项被固定。
3. 下一步固定为 BE-001ER-03 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 实际抽离记录。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
