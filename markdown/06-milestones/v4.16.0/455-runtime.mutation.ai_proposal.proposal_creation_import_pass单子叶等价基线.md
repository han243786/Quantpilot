# v4.16.0 runtime.mutation.ai_proposal.proposal_creation_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FF-01
> 上一批: `454-runtime.mutation.ai_proposal_import_pass第十轮父叶残余判断.md`
> 目标子叶: `runtime.mutation.ai_proposal.proposal_creation_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.proposal_creation_import_pass`
> 代码动作: no code movement
> 下一步: BE-001FF-02 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FF-01 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 单子叶等价基线 | 建立基线 |
| 规范矩阵 | proposal creation handler / explicit import pass / no release transition | 冻结等价边界 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.proposal_creation_import_pass` | proposal creation import pocket |
| 模块树 | `runtime.mutation.ai_proposal.proposal_creation_import_pass` | 单子叶基线 |

---

## 当前白箱边界

`src/runtime/mutation/ai_proposal/proposal_creation.rs` 当前仍通过 `use super::*` 取得父级输入面。本批只冻结真实行为与预期输入面，不移动 Rust 代码。

```text
runtime.mutation.ai_proposal.proposal_creation_import_pass baseline_frozen
runtime.mutation.ai_proposal.proposal_creation_import_pass current_parent_import_bridge: use super::*
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.proposal_creation_import_pass
old_three_leaf_pause_target_cancelled
progress_report_instruction_discarded
```

---

## 冻结的 public / parent-visible 方法

```text
create_runtime_ai_proposal
```

该 handler 属于 route-facing AI proposal creation 边界。本轮只处理 import 输入面冻结，不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或测试资产。

---

## 冻结的输入面

BE-001FF-02/03 预期只能把 parent wildcard import 收敛为以下显式输入面，不得引入新 owner 或 sibling 横向连接:

```text
super::approval_persistence::persist_approval
super::event_lifecycle::{ai_proposal_lifecycle_entry, build_runtime_ai_proposal_event, persist_runtime_ai_proposal_transition}
super::sandbox_trigger::spawn_ai_proposal_sandbox_verification
super::source_governance_identity::{load_runtime_ai_proposal_source_context, runtime_ai_proposal_governance, runtime_ai_proposal_record_id}
super::static_check::{ai_proposal_static_check_result, validate_ai_model_identity, validate_hash_identity}
crate::auth
crate::current_time_ms
crate::io_error
crate::json_bad_request
crate::json_bad_request_with_details
crate::normalize_actor_identity
crate::runtime::{append_parameter_mutation_events_to_run, canonical_runtime_parameter_version, governance_with_parameter_version, validate_runtime_parameter_mutation_target}
crate::validate_runtime_capability_guard
crate::AppState
crate::CreateRuntimeAiProposalRequest
crate::RuntimeAiProposalRecord
crate::RuntimeAiProposalSourceEvidence
crate::RuntimeAiProposalStatus
crate::RuntimeApprovalLifecycleEntry
crate::RuntimeApprovalLevel
crate::RuntimeApprovalRecord
crate::RuntimeApprovalReviewState
crate::RuntimeEvidenceSourceKind
crate::RuntimeRollbackPlan
axum::extract::State
axum::http::StatusCode
axum::Json
State / Json
```

---

## 等价冻结点

BE-001FF-02/03 必须保持以下行为不变:

```text
validate_runtime_capability_guard before request body side effects
permission_boundary ai_write_policy proposal_only
validate_runtime_parameter_mutation_target before record assembly
old_value required
new_value required
validate_ai_model_identity
validate_hash_identity prompt_hash
validate_hash_identity evidence_hash
actor required before normalize_actor_identity
load_runtime_ai_proposal_source_context before canonical versions
canonical_runtime_parameter_version old_value
canonical_runtime_parameter_version new_value
current_time_ms single creation timestamp
ai_proposal_static_check_result before record id
runtime_ai_proposal_record_id
runtime_ai_proposal_governance
RuntimeAiProposalSourceEvidence mirrors source context
RuntimeAiProposalRecord lifecycle starts empty before event push
Submitted lifecycle entry sequence_no current_sequence_no + 1
static status lifecycle entry sequence_no current_sequence_no + 2
governance_with_parameter_version old_parameter_version
RuntimeEvidenceSourceKind::Run event append path only
append_parameter_mutation_events_to_run created and static events
RuntimeAiProposalStatus::StaticCheckPassed creates approval
approval_id apr-now-seq shape
RuntimeApprovalLevel::L1SingleReviewer
RuntimeApprovalReviewState::Pending
RuntimeRollbackPlan generation_rollback
APPROVAL_CREATED lifecycle entry
persist_approval before approval_records insert
auth::scoped_key
persist_runtime_ai_proposal_transition before sandbox trigger
spawn_ai_proposal_sandbox_verification only after persisted transition
non StaticCheckPassed path persists transition without approval
Ok(Json(record))
```

---

## 等价守卫

```text
no_create_handler_body_rewrite
no_capability_guard_rewrite
no_permission_boundary_rewrite
no_source_context_rewrite
no_static_check_rewrite
no_event_lifecycle_rewrite
no_auto_approval_rewrite
no_sandbox_trigger_rewrite
no_persistence_order_rewrite
no_status_transition_rewrite
no_error_payload_rewrite
no_visibility_rewrite
no_sibling_owner_migration
```

---

## 预期 residual

BE-001FF-01 完成后 residual 不变化:

```text
remaining_runtime_parent_import_bridge_3
remaining_mutation_import_bridge_2
remaining_ai_proposal_import_bridge_2
```

BE-001FF-03 实际抽离后，才允许下降为:

```text
expected_runtime_parent_import_bridge_2
expected_mutation_import_bridge_1
expected_ai_proposal_import_bridge_1
```

---

## parent warning 归属

当前 parent facade unused imports 继续延期:

```text
parent facade unused imports remain deferred
load_approval_from_disk
load_runtime_ai_proposal_for_user
ensure_ai_proposal_can_be_approved
ai_proposal_approved_status
update_ai_proposal_status
axum::extract::Query
```

这些 warning 只有在 child import pocket 和 parent facade import pass 收束时才处理。

---

## 排除项

本批不处理:

1. 不改 Rust 代码。
2. 不改 `proposal_creation.rs` 顶部 import；这属于 BE-001FF-03。
3. 不改 `create_runtime_ai_proposal` 的函数体、可见性、状态机、自动审批、事件写入、持久化顺序、sandbox trigger 或错误 payload。
4. 不处理 `ai_proposal.rs` parent facade import residual。
5. 不处理 `src/runtime/mod.rs` root parent bridge。
6. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
7. 不新增 sibling 横向连接。
8. 不启动 release transition。

---

## 验证要求

本批为 `no code movement` 等价基线，提交前至少执行:

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

AI 声称 BE-001FF-01 完成时，必须说明:

1. 本批只是 `no code movement` 等价基线。
2. `proposal_creation.rs` 仍未实际移除 `use super::*`。
3. 下一步只能进入 BE-001FF-02 抽离方案。
4. 不得宣称 proposal_creation、ai proposal parent facade、mutation import、parent import bridge、backend.runtime 或 Rust 重构已经完成。

---

## 验收标准

1. `455-runtime.mutation.ai_proposal.proposal_creation_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `create_runtime_ai_proposal` 的 import 输入面与等价行为已冻结。
3. BE-001FF-02 成为唯一下一步。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
