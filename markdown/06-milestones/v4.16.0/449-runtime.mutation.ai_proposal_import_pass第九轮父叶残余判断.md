# v4.16.0 runtime.mutation.ai_proposal_import_pass 第九轮父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FC-01
> 上一批: `448-runtime.mutation.ai_proposal.sandbox_trigger_import_pass单叶closeout.md`
> 目标父叶: `runtime.mutation.ai_proposal_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal_import_pass`
> 判定: `runtime.mutation.ai_proposal_import_pass stop_split: false`
> 代码动作: no code movement
> 下一步: BE-001FD-01 `runtime.mutation.ai_proposal.approval_review_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FC-01 `runtime.mutation.ai_proposal_import_pass` 第九轮父叶残余判断 | 父叶重判 |
| 规范矩阵 | parent import bridge / explicit import pass / no release transition | 保持父叶继续拆分 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal_import_pass` | ai proposal import pass 父叶重判 |
| 模块树 | `runtime.mutation.ai_proposal_import_pass` | `stop_split: false` |

---

## 父叶重判结论

BE-001FB-04 已完成 `sandbox_trigger.rs` import pocket closeout，但 `runtime.mutation.ai_proposal_import_pass` 父叶仍存在 3 个 ai proposal parent wildcard import residual。当前父叶不能 closeout，必须继续按单子叶方式处理。

```text
runtime.mutation.ai_proposal_import_pass ninth_parent_residual_judgment
runtime.mutation.ai_proposal_import_pass stop_split: false
approval_review_import_pass_selected
remaining_runtime_parent_import_bridge_4
remaining_mutation_import_bridge_3
remaining_ai_proposal_import_bridge_3
old_three_leaf_pause_target_cancelled
progress_report_instruction_discarded
```

本批不改 Rust 代码，只做父叶残余判断和下一颗子叶选择。

---

## 当前 residual 清单

```text
src/runtime/mod.rs
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/approval_review.rs
src/runtime/mutation/ai_proposal/proposal_creation.rs
```

`src/runtime/mod.rs` 属于 root parent bridge，`src/runtime/mutation/ai_proposal.rs` 属于 ai proposal parent facade，均不在本轮直接处理。

---

## 子叶选择表

| 候选子叶 | 文件 | 判断 | 本轮决定 |
| --- | --- | --- | --- |
| `runtime.mutation.ai_proposal.approval_review_import_pass` | `approval_review.rs` | route-facing approval handlers，依赖 approval persistence、record query、sandbox gate 与 status transition；这些依赖已完成显式化 | 采纳 |
| `runtime.mutation.ai_proposal.proposal_creation_import_pass` | `proposal_creation.rs` | create handler，依赖 source/governance/static check/event lifecycle/approval persistence/sandbox trigger，体量更大 | 延后 |
| `runtime.mutation.ai_proposal.parent_facade_import_pass` | `ai_proposal.rs` | child module declaration / re-export / parent facade | 最后处理 |

---

## approval_review 选择理由

BE-001FD-01 选择 `approval_review.rs`，原因:

1. 它是 list/detail/approve/reject/claim 的 route-facing handler 集合，父级调用面已经明确。
2. 它依赖的 `approval_persistence`、`record_query`、`sandbox_trigger` 和 `status_transition` import pocket 均已显式化，适合现在冻结 handler 输入面。
3. `proposal_creation.rs` 仍持有更大的创建流程和自动审批单/沙箱触发链路，等 approval review 固化后再处理更稳。
4. 当前目标仍是 import 输入面显式化，不触碰审批状态机、审批锁顺序、approval lifecycle、proposal status 写入或 handler 返回语义。

---

## BE-001FD-01 冻结范围

下一批只能建立 `runtime.mutation.ai_proposal.approval_review_import_pass` 单子叶等价基线，冻结以下边界:

```text
list_runtime_approvals
get_runtime_approval_detail
approve_ai_proposal
reject_ai_proposal
claim_ai_proposal_review
auth::UserId
AppState
RuntimeApprovalListQuery
RuntimeApprovalRecord
RuntimeApprovalReviewState
RuntimeApprovalLifecycleEntry
RuntimeAiProposalStatus::Denied
ApprovalActionRequest
load_approval_from_disk
load_runtime_ai_proposal_for_user
ensure_ai_proposal_can_be_approved
ai_proposal_approved_status
update_ai_proposal_status
persist_approval
json_bad_request
io_error
current_time_ms
State / Query / Path / Json
StatusCode
```

必须保持:

```text
no_approval_filter_rewrite
no_approval_lock_order_rewrite
no_reviewer_count_rewrite
no_lifecycle_event_rewrite
no_status_transition_rewrite
no_persistence_order_rewrite
no_error_payload_rewrite
no_visibility_rewrite
no_sibling_owner_migration
```

---

## 排除项

本批不处理:

1. 不改 Rust 代码。
2. 不处理 `approval_review.rs` 顶部 import；这属于 BE-001FD-03。
3. 不处理 `proposal_creation.rs` 或 parent facade import residual。
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

AI 声称 BE-001FC-01 完成时，必须说明:

1. 本批只是 `no code movement` 父叶残余判断。
2. `runtime.mutation.ai_proposal_import_pass stop_split: false`。
3. 下一步只能进入 BE-001FD-01 `runtime.mutation.ai_proposal.approval_review_import_pass` 单子叶等价基线。
4. 不得宣称 approval_review、proposal_creation、ai proposal parent facade、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `449-runtime.mutation.ai_proposal_import_pass第九轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶保持 `stop_split: false`，下一颗子叶固定为 `approval_review_import_pass`。
3. BE-001FD-01 成为唯一下一步。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
