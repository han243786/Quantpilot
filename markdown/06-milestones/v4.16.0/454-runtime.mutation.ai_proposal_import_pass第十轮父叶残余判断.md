# v4.16.0 runtime.mutation.ai_proposal_import_pass 第十轮父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FE-01
> 上一批: `453-runtime.mutation.ai_proposal.approval_review_import_pass单叶closeout.md`
> 目标父叶: `runtime.mutation.ai_proposal_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal_import_pass`
> 判定: `runtime.mutation.ai_proposal_import_pass stop_split: false`
> 代码动作: no code movement
> 下一步: BE-001FF-01 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FE-01 `runtime.mutation.ai_proposal_import_pass` 第十轮父叶残余判断 | 父叶重判 |
| 规范矩阵 | parent import bridge / explicit import pass / no release transition | 保持父叶继续拆分 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal_import_pass` | ai proposal import pass 父叶重判 |
| 模块树 | `runtime.mutation.ai_proposal_import_pass` | `stop_split: false` |

---

## 父叶重判结论

BE-001FD-04 已完成 `approval_review.rs` import pocket closeout，但 `runtime.mutation.ai_proposal_import_pass` 父叶仍存在 2 个 ai proposal parent wildcard import residual。当前父叶不能 closeout，必须继续按单子叶方式处理。

```text
runtime.mutation.ai_proposal_import_pass tenth_parent_residual_judgment
runtime.mutation.ai_proposal_import_pass stop_split: false
proposal_creation_import_pass_selected
remaining_runtime_parent_import_bridge_3
remaining_mutation_import_bridge_2
remaining_ai_proposal_import_bridge_2
old_three_leaf_pause_target_cancelled
progress_report_instruction_discarded
```

本批不改 Rust 代码，只做父叶残余判断和下一颗子叶选择。

---

## 当前 residual 清单

```text
src/runtime/mod.rs
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/proposal_creation.rs
```

`src/runtime/mod.rs` 属于 root parent bridge；`src/runtime/mutation/ai_proposal.rs` 属于 ai proposal parent facade。两者不在本轮直接处理。

---

## 子叶选择表

| 候选子叶 | 文件 | 判断 | 本轮决定 |
| --- | --- | --- | --- |
| `runtime.mutation.ai_proposal.proposal_creation_import_pass` | `proposal_creation.rs` | create handler，依赖 source governance、static check、event lifecycle、approval persistence、sandbox trigger 与 shared mutation governance；是最后一个 child import pocket | 采纳 |
| `runtime.mutation.ai_proposal.parent_facade_import_pass` | `ai_proposal.rs` | child module declaration / re-export / parent facade | 最后处理 |

---

## proposal_creation 选择理由

BE-001FF-01 选择 `proposal_creation.rs`，原因:

1. `approval_review.rs` 已完成 import 显式化，当前唯一 remaining child import pocket 是 `proposal_creation.rs`。
2. proposal creation 是 AI proposal 创建路径的 route-facing handler，仍使用 parent wildcard import，输入面应在 parent facade 之前显式化。
3. 它依赖的 source governance、static check、event lifecycle、approval persistence、sandbox trigger、record query 与 status transition import pocket 已完成显式化，具备冻结 create flow 输入面的条件。
4. parent facade import 必须最后处理，以便先稳定 child 输入面，再判断 `ai_proposal.rs` 是否只剩 child declaration、re-export 与受控 parent glue。

---

## BE-001FF-01 冻结范围

下一批只能建立 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 单子叶等价基线，冻结以下边界:

```text
create_runtime_ai_proposal
validate_runtime_capability_guard
permission_boundary ai_write_policy proposal_only
validate_runtime_parameter_mutation_target
old_value and new_value required
validate_ai_model_identity
validate_hash_identity
normalize_actor_identity
load_runtime_ai_proposal_source_context
canonical_runtime_parameter_version
ai_proposal_static_check_result
runtime_ai_proposal_record_id
runtime_ai_proposal_governance
RuntimeAiProposalSourceEvidence
RuntimeAiProposalRecord
build_runtime_ai_proposal_event
ai_proposal_lifecycle_entry
governance_with_parameter_version
append_parameter_mutation_events_to_run
append_parameter_mutation_events_to_backtest
persist_runtime_ai_proposal_transition
persist_approval
spawn_ai_proposal_sandbox_verification
auth::scoped_key
current_time_ms
json_bad_request
json_bad_request_with_details
```

必须保持:

```text
no_create_handler_body_rewrite
no_capability_guard_rewrite
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

## parent warning 归属

当前 cargo warning 中的 parent facade unused imports 继续延期到后续 parent facade import pass，不在 BE-001FE-01 或 BE-001FF-01 基线前主动清理:

```text
parent facade unused imports remain deferred
load_approval_from_disk
load_runtime_ai_proposal_for_user
ensure_ai_proposal_can_be_approved
ai_proposal_approved_status
update_ai_proposal_status
axum::extract::Query
```

这些 warning 是 child import 显式化后的父级残余信号，不是本批失败条件。

---

## 排除项

本批不处理:

1. 不改 Rust 代码。
2. 不处理 `proposal_creation.rs` 顶部 import；这属于 BE-001FF-03。
3. 不处理 `ai_proposal.rs` parent facade import residual。
4. 不处理 `src/runtime/mod.rs` root parent bridge。
5. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
6. 不新增 sibling 横向连接。
7. 不启动 release transition。

---

## 验证要求

本批为 `no code movement` 父叶重判，提交前至少执行:

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

AI 声称 BE-001FE-01 完成时，必须说明:

1. 本批只是 `no code movement` 父叶残余判断。
2. `runtime.mutation.ai_proposal_import_pass stop_split: false`。
3. 下一步只能进入 BE-001FF-01 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 单子叶等价基线。
4. 不得宣称 proposal_creation、ai proposal parent facade、mutation import、parent import bridge、backend.runtime 或 Rust 重构已经完成。

---

## 验收标准

1. `454-runtime.mutation.ai_proposal_import_pass第十轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶保持 `stop_split: false`，下一颗子叶固定为 `proposal_creation_import_pass`。
3. BE-001FF-01 成为唯一下一步。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
