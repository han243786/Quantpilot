# v4.16.0 runtime.mutation.ai_proposal.proposal_creation_import_pass 单叶 closeout

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FF-04
> 基线: `457-runtime.mutation.ai_proposal.proposal_creation_import_pass抽离记录.md`
> 目标子叶: `runtime.mutation.ai_proposal.proposal_creation_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.proposal_creation_import_pass`
> 判定: `runtime.mutation.ai_proposal.proposal_creation_import_pass stop_split: true`
> 代码动作: no code movement
> 下一步: BE-001FG-01 `runtime.mutation.ai_proposal_import_pass` 父叶残余判断

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FF-04 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 单叶 closeout | 单叶收口 |
| 规范矩阵 | stop split / explicit import pass / create handler semantics freeze / no release transition | 禁止继续细拆 create handler import pocket |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.proposal_creation_import_pass` | 白箱节点收口 |
| 模块树 | `runtime.mutation.ai_proposal.proposal_creation_import_pass` | `stop_split: true` |

---

## 收口判定

BE-001FF-03 已完成 `src/runtime/mutation/ai_proposal/proposal_creation.rs` 的 parent wildcard import 删除:

```text
runtime.mutation.ai_proposal.proposal_creation_import_pass closeout_done
runtime.mutation.ai_proposal.proposal_creation_import_pass stop_split: true
removed use super::*
single file import rewrite
proposal_creation_explicit_imports
```

本叶不继续拆分为 capability guard、source context、static check、record assembly、approval creation、event append、persist order、sandbox trigger 或 error payload 微叶。原因:

1. 当前治理目标是 import 输入面显式化，`create_runtime_ai_proposal` 函数体没有发生迁移。
2. create handler 内部状态机、自动审批、事件写入、持久化顺序和 sandbox trigger 已由 BE-001FF-01/02/03 冻结并验证。
3. 继续细拆会进入 handler 语义重构，不属于本轮 import pass 的最小等价边界。
4. 父叶 `runtime.mutation.ai_proposal_import_pass` 仍有更高价值 residual: parent facade import bridge。

---

## 等价边界复核

以下内容保持不变:

```text
create_runtime_ai_proposal
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
persist_approval before approval_records insert
auth::scoped_key
persist_runtime_ai_proposal_transition before sandbox trigger
spawn_ai_proposal_sandbox_verification only after persisted transition
non StaticCheckPassed path persists transition without approval
Ok(Json(record))
```

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
本叶 closeout 后，父级 residual 继续为:

```text
remaining_runtime_parent_import_bridge_2
remaining_mutation_import_bridge_1
remaining_ai_proposal_import_bridge_1
```

剩余文件:

```text
src/runtime/mod.rs
src/runtime/mutation/ai_proposal.rs
```

下一步只能回到父叶:

```text
BE-001FG-01 runtime.mutation.ai_proposal_import_pass 父叶残余判断
```

---

## 排除项
本批不处理:

1. 不修改 Rust 代码。
2. 不清理 `src/runtime/mutation/ai_proposal.rs` parent facade unused imports。
3. 不处理 `src/runtime/mod.rs` root parent bridge。
4. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
5. 不新增 sibling 横向连接。
6. 不启动 release transition。

---

## 验证要求

本批为 `no code movement` closeout，提交前至少执行:

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

AI 声称 BE-001FF-04 完成时，必须说明:

1. 本批只是 `no code movement` 单叶 closeout。
2. `runtime.mutation.ai_proposal.proposal_creation_import_pass stop_split: true`。
3. 下一步只能进入 BE-001FG-01 `runtime.mutation.ai_proposal_import_pass` 父叶残余判断。
4. 不得宣称 ai proposal parent facade、mutation import、parent import bridge、backend.runtime 或 Rust 重构已经完成。

---

## 验收标准

1. `458-runtime.mutation.ai_proposal.proposal_creation_import_pass单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 本叶设置 `stop_split: true`，不继续拆 create handler import pocket 微叶。
3. 下一步固定为 BE-001FG-01 父叶残余判断。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
