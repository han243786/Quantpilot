# v4.16.0 runtime.mutation.ai_proposal.approval_review_import_pass 单叶 closeout

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FD-04
> 基线: `452-runtime.mutation.ai_proposal.approval_review_import_pass抽离记录.md`
> 目标子叶: `runtime.mutation.ai_proposal.approval_review_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.approval_review_import_pass`
> 判定: `runtime.mutation.ai_proposal.approval_review_import_pass stop_split: true`
> 代码动作: no code movement
> 下一步: BE-001FE-01 `runtime.mutation.ai_proposal_import_pass` 父叶残余判断

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FD-04 `runtime.mutation.ai_proposal.approval_review_import_pass` 单叶 closeout | 单叶收口 |
| 规范矩阵 | stop split / explicit import pass / approval review semantics freeze / no release transition | 禁止继续细拆 approval review import pocket |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.approval_review_import_pass` | 白箱节点收口 |
| 模块树 | `runtime.mutation.ai_proposal.approval_review_import_pass` | `stop_split: true` |

---

## 收口判定

BE-001FD-03 已完成 `src/runtime/mutation/ai_proposal/approval_review.rs` 的 parent wildcard import 删除:

```text
runtime.mutation.ai_proposal.approval_review_import_pass closeout_done
runtime.mutation.ai_proposal.approval_review_import_pass stop_split: true
removed use super::*
single file import rewrite
approval_review_explicit_imports
```

本叶不继续拆分为 approval list filter、detail fallback、approve lock order、reject flow、claim flow、reviewer counter、lifecycle event 或 persist order 微叶。原因:

1. 当前治理目标是 import 输入面显式化，五个 handler 函数体没有发生迁移。
2. approval review 的锁顺序、状态机、lifecycle 与 persist order 已由 BE-001FD-01/02/03 冻结并验证。
3. 继续拆微叶只会扩大文档成本，不会进一步降低当前 parent wildcard residual 风险。
4. 父叶 `runtime.mutation.ai_proposal_import_pass` 仍有更高价值 residual。

---

## 等价边界复核

以下内容保持不变:

```text
list_runtime_approvals
get_runtime_approval_detail
approve_ai_proposal
reject_ai_proposal
claim_ai_proposal_review
list_runtime_approvals scoped prefix lookup
review_state optional case-insensitive filter
created_at_ms descending sort
get_runtime_approval_detail memory-first lookup
load_approval_from_disk fallback
approve_ai_proposal loads proposal before approval write lock
ensure_ai_proposal_can_be_approved gate remains before approval write lock
approval write lock existing shape
Pending | UnderReview approval states for approve/reject
Pending only for claim
reviewers_approved no duplicate and no rejected actor
reviewers_assigned no duplicate
reviewers_required threshold
APPROVAL_APPROVED
APPROVAL_PARTIAL
APPROVAL_REJECTED
APPROVAL_CLAIMED
RuntimeAiProposalStatus::Denied
ai_proposal_approved_status
persist_approval before scoped insert
auth::scoped_key
```

本批保持:

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

## residual 状态

本叶 closeout 后，父级 residual 继续为:

```text
remaining_runtime_parent_import_bridge_3
remaining_mutation_import_bridge_2
remaining_ai_proposal_import_bridge_2
```

剩余文件:

```text
src/runtime/mod.rs
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/proposal_creation.rs
```

下一步只能回到父叶:

```text
BE-001FE-01 runtime.mutation.ai_proposal_import_pass 父叶残余判断
```

---

## 排除项

本批不处理:

1. 不修改 Rust 代码。
2. 不处理 `proposal_creation.rs` 或 `ai_proposal.rs` parent facade import residual。
3. 不删除 `src/runtime/mutation/ai_proposal.rs` 中因本叶显式化而出现的 unused import；该残余属于后续父叶处理。
4. 不处理 `src/runtime/mod.rs` root parent bridge。
5. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
6. 不新增 sibling 横向连接。
7. 不启动 release transition。

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

AI 声称 BE-001FD-04 完成时，必须说明:

1. 本批只是 `no code movement` 单叶 closeout。
2. `runtime.mutation.ai_proposal.approval_review_import_pass stop_split: true`。
3. 下一步只能进入 BE-001FE-01 `runtime.mutation.ai_proposal_import_pass` 父叶残余判断。
4. 不得宣称 proposal_creation、ai proposal parent facade、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `453-runtime.mutation.ai_proposal.approval_review_import_pass单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 本叶设置 `stop_split: true`，不继续拆 approval review import pocket 微叶。
3. 下一步固定为 BE-001FE-01 父叶残余判断。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
