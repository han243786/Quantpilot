# v4.16.0 runtime.mutation.ai_proposal.proposal_creation_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FF-02
> 基线: `455-runtime.mutation.ai_proposal.proposal_creation_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.mutation.ai_proposal.proposal_creation_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.proposal_creation_import_pass`
> 代码动作: no code movement
> 下一步: BE-001FF-03 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FF-02 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 抽离方案 | 方案优化 |
| 规范矩阵 | single-file import rewrite / create handler freeze / no release transition | 固定抽离边界 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.proposal_creation_import_pass` | proposal creation import pocket |
| 模块树 | `runtime.mutation.ai_proposal.proposal_creation_import_pass` | 抽离方案 |

---

## 方案结论

BE-001FF-03 只能改写 `src/runtime/mutation/ai_proposal/proposal_creation.rs` 顶部 import，禁止改函数体、可见性、capability guard、source context、static check、record assembly、自动审批、事件写入、持久化顺序、sandbox trigger、错误 payload 或 sibling owner。

```text
runtime.mutation.ai_proposal.proposal_creation_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.proposal_creation_import_pass
proposal_creation_import_pass plan_frozen
single_file_proposal_creation_import_rewrite
next_step: BE-001FF-03 extraction record
```

---

## 允许改动

唯一允许的 Rust 改动是把 `src/runtime/mutation/ai_proposal/proposal_creation.rs` 顶部 import 从 parent wildcard 改为显式输入面:

```diff
-use super::*;
+use super::{
+    approval_persistence::persist_approval,
+    event_lifecycle::{
+        ai_proposal_lifecycle_entry, build_runtime_ai_proposal_event,
+        persist_runtime_ai_proposal_transition,
+    },
+    sandbox_trigger::spawn_ai_proposal_sandbox_verification,
+    source_governance_identity::{
+        load_runtime_ai_proposal_source_context, runtime_ai_proposal_governance,
+        runtime_ai_proposal_record_id,
+    },
+    static_check::{
+        ai_proposal_static_check_result, validate_ai_model_identity, validate_hash_identity,
+    },
+};
+use crate::{
+    auth, current_time_ms, io_error, json_bad_request, json_bad_request_with_details,
+    normalize_actor_identity,
+    runtime::{
+        append_parameter_mutation_events_to_run, canonical_runtime_parameter_version,
+        governance_with_parameter_version, validate_runtime_parameter_mutation_target,
+    },
+    validate_runtime_capability_guard, AppState, CreateRuntimeAiProposalRequest,
+    RuntimeAiProposalRecord, RuntimeAiProposalSourceEvidence, RuntimeAiProposalStatus,
+    RuntimeApprovalLifecycleEntry, RuntimeApprovalLevel, RuntimeApprovalRecord,
+    RuntimeApprovalReviewState, RuntimeEvidenceSourceKind, RuntimeRollbackPlan,
+};
+use axum::{extract::State, http::StatusCode, Json};
```

允许 `cargo fmt` 对 import 分组做机械格式化。除此以外不得改动任何 Rust 语句。

---

## 不允许改动

BE-001FF-03 不得修改:

```text
create_runtime_ai_proposal body
pub(crate) visibility
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

必须保持:

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

## residual 预期

BE-001FF-03 完成后，预期 residual 从:

```text
remaining_runtime_parent_import_bridge_3
remaining_mutation_import_bridge_2
remaining_ai_proposal_import_bridge_2
```

下降为:

```text
expected_runtime_parent_import_bridge_2
expected_mutation_import_bridge_1
expected_ai_proposal_import_bridge_1
```

仍不处理:

```text
src/runtime/mod.rs
src/runtime/mutation/ai_proposal.rs
```

---

## 排除项

本批不处理:

1. 不改 Rust 代码。
2. 不执行 BE-001FF-03 实际 import rewrite。
3. 不处理 `src/runtime/mutation/ai_proposal.rs` parent facade import residual。
4. 不处理 `src/runtime/mod.rs` root parent bridge。
5. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
6. 不新增 sibling 横向连接。
7. 不启动 release transition。

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

AI 声称 BE-001FF-02 完成时，必须说明:

1. 本批只是 `no code movement` 抽离方案。
2. BE-001FF-03 只允许改写 `src/runtime/mutation/ai_proposal/proposal_creation.rs` 顶部 import。
3. 不得宣称 `proposal_creation.rs` 已完成实际抽离。
4. 不得宣称 ai proposal parent facade、mutation import、parent import bridge、backend.runtime 或 Rust 重构已经完成。

---

## 验收标准

1. `456-runtime.mutation.ai_proposal.proposal_creation_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001FF-03 的允许改动被限制为单文件 import rewrite。
3. BE-001FF-03 成为唯一下一步。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
