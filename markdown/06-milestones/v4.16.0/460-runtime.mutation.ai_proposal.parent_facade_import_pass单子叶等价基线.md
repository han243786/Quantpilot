# v4.16.0 runtime.mutation.ai_proposal.parent_facade_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FH-01
> 基线: `459-runtime.mutation.ai_proposal_import_pass第十一轮父叶残余判断.md`
> 目标子叶: `runtime.mutation.ai_proposal.parent_facade_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.parent_facade_import_pass`
> 真实文件: `src/runtime/mutation/ai_proposal.rs`
> 代码动作: no code movement
> 下一步: BE-001FH-02 `runtime.mutation.ai_proposal.parent_facade_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FH-01 `runtime.mutation.ai_proposal.parent_facade_import_pass` 单子叶等价基线 | staged explicit import pass |
| 规范矩阵 | parent facade import baseline / no code movement / explicit input freeze | 标准档位 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.parent_facade_import_pass` | 冻结 ai proposal parent facade 输入面 |
| 模块树 | `runtime.mutation.ai_proposal.parent_facade_import_pass` | 单文件 facade import pocket |

---

## 当前残余队列

```text
BE-001FH-01
BE-001FH-02
runtime.mutation.ai_proposal.parent_facade_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.parent_facade_import_pass
parent_facade_import_pass baseline_frozen
single_file_ai_proposal_parent_facade_import_pass
no code movement
remaining_runtime_parent_import_bridge_2
remaining_mutation_import_bridge_1
remaining_ai_proposal_import_bridge_1
old_three_leaf_pause_target_cancelled
progress_report_instruction_discarded
```

本轮只冻结 `src/runtime/mutation/ai_proposal.rs` 的当前输入面，不改 Rust。

---

## 白箱节点冻结

当前 parent facade 承担五类职责:

1. child module declarations: 声明 `approval_persistence`、`approval_review`、`event_lifecycle`、`proposal_creation`、`record_query`、`sandbox_trigger`、`source_governance_identity`、`static_check` 与 `status_transition`。
2. public facade re-export: 向上层暴露 AI proposal create/list/detail 与 approval list/detail/approve/reject/claim handler。
3. parent-private helper alias: 当前保留来自 child 的 helper import，供 parent facade 测试或后续显式输入探针确认。
4. test module `v4_ai_proposal_tests`: 当前通过 `use super::*` 使用 parent facade 作用域、`ensure_ai_proposal_can_be_approved` 与多组 crate-level schema / helper。
5. facade handoff: 保持上层 `runtime::mod.rs` 对 mutation AI proposal handlers 的统一 re-export 面。

当前残余 import:

```rust
use super::*;
```

当前 child module declarations:

```rust
#[path = "ai_proposal/approval_persistence.rs"]
mod approval_persistence;
#[path = "ai_proposal/approval_review.rs"]
mod approval_review;
#[path = "ai_proposal/event_lifecycle.rs"]
mod event_lifecycle;
#[path = "ai_proposal/proposal_creation.rs"]
mod proposal_creation;
#[path = "ai_proposal/record_query.rs"]
mod record_query;
#[path = "ai_proposal/sandbox_trigger.rs"]
mod sandbox_trigger;
#[path = "ai_proposal/source_governance_identity.rs"]
mod source_governance_identity;
#[path = "ai_proposal/static_check.rs"]
mod static_check;
#[path = "ai_proposal/status_transition.rs"]
mod status_transition;
```

当前 public facade re-export:

```rust
pub(crate) use approval_review::{
    approve_ai_proposal, claim_ai_proposal_review, get_runtime_approval_detail,
    list_runtime_approvals, reject_ai_proposal,
};
pub(crate) use proposal_creation::create_runtime_ai_proposal;
pub(crate) use record_query::{get_runtime_ai_proposal_detail, list_runtime_ai_proposals};
```

当前 parent-private helper imports:

```rust
use approval_persistence::{load_approval_from_disk, persist_approval};
use event_lifecycle::{
    ai_proposal_lifecycle_entry, build_runtime_ai_proposal_event,
    persist_runtime_ai_proposal_transition,
};
use record_query::load_runtime_ai_proposal_for_user;
use sandbox_trigger::{ensure_ai_proposal_can_be_approved, spawn_ai_proposal_sandbox_verification};
use source_governance_identity::{
    load_runtime_ai_proposal_source_context, runtime_ai_proposal_governance,
    runtime_ai_proposal_record_id,
};
use static_check::{
    ai_proposal_static_check_result, validate_ai_model_identity, validate_hash_identity,
};
use status_transition::{ai_proposal_approved_status, update_ai_proposal_status};
```

---

## 预期输入探针

BE-001FH-02 只能围绕下列输入面建立方案:

```text
initial_parent_facade_import_surface_candidate
hidden_parent_input_probe_required
test_module_explicit_import_probe_required
test module v4_ai_proposal_tests
parent_private_helper_alias_probe_required
```

预期 BE-001FH-03 先尝试移除 `use super::*`，并把 parent facade 需要的输入面显式化。如果 `cargo check -p quantpilot` 或 `cargo test -p quantpilot v4_ai_proposal_tests::ai_proposal_approval_requires_binding_and_sandbox_report` 发现隐藏父级输入，必须回到方案记录显式 import，不得扩大到 child rewrite。

等价约束:

```text
no_function_body_change
no_visibility_change
no_child_module_rewrite
no_reexport_rewrite
no_private_helper_alias_rewrite
no_test_semantics_rewrite
no_approval_review_rewrite
no_proposal_creation_rewrite
no_record_query_rewrite
no_sandbox_trigger_rewrite
no_status_transition_rewrite
no_source_governance_rewrite
no_static_check_rewrite
no_event_lifecycle_rewrite
no_sibling_horizontal_link
no_release_transition
```

---

## 不进入范围

本轮不处理:

1. 不修改 `src/runtime/mutation/ai_proposal.rs`。
2. 不改任何 child file。
3. 不改 public handler re-export。
4. 不改 `v4_ai_proposal_tests` 测试语义。
5. 不继续拆 approval / proposal / query / sandbox / status / governance / static / event child。
6. 不宣称 `runtime.mutation.ai_proposal_import_pass stop_split: true`。
7. 不处理 `src/runtime/mod.rs` root parent bridge。
8. 不启动发布过渡连接。

---

## 下一步边界

下一步只允许进入 BE-001FH-02 抽离方案:

```text
BE-001FH-02
runtime.mutation.ai_proposal.parent_facade_import_pass
single_file_ai_proposal_parent_facade_import_pass
```

BE-001FH-02 仍不得直接修改 Rust；实际 import rewrite 只能在 BE-001FH-03 发生。

---

## 验证要求

本批提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot v4_ai_proposal_tests::ai_proposal_approval_requires_binding_and_sandbox_report
```

---

## 幻觉检查点

AI 声称 BE-001FH-01 完成时，必须说明:

1. 本批是 `no code movement`。
2. 只建立 `parent_facade_import_pass baseline_frozen`。
3. 当前真实文件仍是 `src/runtime/mutation/ai_proposal.rs`。
4. `use super::*` 仍未改写。
5. 下一步只能进入 BE-001FH-02 抽离方案。
6. 旧三叶暂停目标保持取消: `old_three_leaf_pause_target_cancelled`。

不得声称 parent facade import 已改写、ai proposal import pass 已完成、mutation import pass 已完成、parent import bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `460-runtime.mutation.ai_proposal.parent_facade_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001FH-01 只冻结 baseline，不改 Rust。
3. `hidden_parent_input_probe_required`、`test_module_explicit_import_probe_required` 与 `parent_private_helper_alias_probe_required` 被记录。
4. 下一步固定为 BE-001FH-02 抽离方案。
5. Rust / 治理 / 全量树门禁均通过。
