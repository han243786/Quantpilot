# v4.16.0 runtime.mutation.ai_proposal.proposal_creation_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FF-03
> 方案: `456-runtime.mutation.ai_proposal.proposal_creation_import_pass抽离方案.md`
> 目标子叶: `runtime.mutation.ai_proposal.proposal_creation_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.proposal_creation_import_pass`
> 代码动作: actual import rewrite
> 下一步: BE-001FF-04 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 单叶 closeout

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FF-03 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 抽离记录 | 实际抽离 |
| 规范矩阵 | single-file import rewrite / create handler semantics frozen / no release transition | 受控 import 收敛 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.proposal_creation_import_pass` | proposal creation import pocket |
| 模块树 | `runtime.mutation.ai_proposal.proposal_creation_import_pass` | 抽离记录 |

---

## 抽离结果

BE-001FF-03 已按 BE-001FF-02 方案完成单文件 import rewrite:

```text
runtime.mutation.ai_proposal.proposal_creation_import_pass extraction_done
removed use super::*
single file import rewrite
proposal_creation_explicit_imports
src/runtime/mutation/ai_proposal/proposal_creation.rs
```

实际改动只发生在 `src/runtime/mutation/ai_proposal/proposal_creation.rs` 顶部 import。`create_runtime_ai_proposal` 函数体、可见性、capability guard、permission boundary、source context、static check、record assembly、自动审批、事件写入、持久化顺序、sandbox trigger、错误 payload 和 sibling owner 均未修改。

---

## 实际显式输入面
```rust
use super::{
    approval_persistence::persist_approval,
    event_lifecycle::{
        ai_proposal_lifecycle_entry, build_runtime_ai_proposal_event,
        persist_runtime_ai_proposal_transition,
    },
    sandbox_trigger::spawn_ai_proposal_sandbox_verification,
    source_governance_identity::{
        load_runtime_ai_proposal_source_context, runtime_ai_proposal_governance,
        runtime_ai_proposal_record_id,
    },
    static_check::{
        ai_proposal_static_check_result, validate_ai_model_identity, validate_hash_identity,
    },
};
use crate::{
    auth, current_time_ms, io_error, json_bad_request, json_bad_request_with_details,
    normalize_actor_identity,
    runtime::{
        append_parameter_mutation_events_to_run, canonical_runtime_parameter_version,
        governance_with_parameter_version, validate_runtime_parameter_mutation_target,
    },
    validate_runtime_capability_guard, AppState, CreateRuntimeAiProposalRequest,
    RuntimeAiProposalRecord, RuntimeAiProposalSourceEvidence, RuntimeAiProposalStatus,
    RuntimeApprovalLevel, RuntimeApprovalLifecycleEntry, RuntimeApprovalRecord,
    RuntimeApprovalReviewState, RuntimeEvidenceSourceKind, RuntimeRollbackPlan,
};
use axum::{extract::State, http::StatusCode, Json};
```

---

## 等价边界复核

以下行为保持不变:

```text
create_runtime_ai_proposal body unchanged
pub(crate) visibility unchanged
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

本批保持:

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

## residual 状态
本批完成后，父级 residual 下降为:

```text
actual_runtime_parent_import_bridge_3_to_2
actual_mutation_import_bridge_2_to_1
actual_ai_proposal_import_bridge_2_to_1
remaining_runtime_parent_import_bridge_2
remaining_mutation_import_bridge_1
remaining_ai_proposal_import_bridge_1
```

剩余文件:

```text
src/runtime/mod.rs
src/runtime/mutation/ai_proposal.rs
```

父层 unused imports 因本叶脱离 parent wildcard 而扩大暴露，继续延期到 parent facade import pass:

```text
parent facade unused imports remain deferred
```

---

## 排除项
本批未处理:

1. 未改 handler 函数体。
2. 未处理 `src/runtime/mutation/ai_proposal.rs` parent facade import residual。
3. 未处理 `src/runtime/mod.rs` root parent bridge。
4. 未迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
5. 未新增 sibling 横向连接。
6. 未启动 release transition。

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

AI 声称 BE-001FF-03 完成时，必须说明:

1. 本批只完成 `proposal_creation.rs` 顶部 import rewrite。
2. `proposal_creation.rs` 已移除 `use super::*`，但 `create_runtime_ai_proposal` 函数体未改。
3. 下一步只能进入 BE-001FF-04 单叶 closeout。
4. 不得宣称 ai proposal parent facade、mutation import、parent import bridge、backend.runtime 或 Rust 重构已经完成。

---

## 验收标准

1. `457-runtime.mutation.ai_proposal.proposal_creation_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `proposal_creation.rs` 顶部 import 已显式化。
3. BE-001FF-04 成为唯一下一步。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
