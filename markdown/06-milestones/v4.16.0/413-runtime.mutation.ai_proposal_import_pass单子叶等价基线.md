# v4.16.0 runtime.mutation.ai_proposal_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EO-01
> 基线: `412-runtime.mutation_import_pass第二轮父叶残余判断.md`
> 目标子叶: `runtime.mutation.ai_proposal_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal_import_pass`
> 代码动作: no code movement
> 下一步: BE-001EO-02 `runtime.mutation.ai_proposal_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EO-01 `runtime.mutation.ai_proposal_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | parent import bridge / explicit import pass / minimum batch / release transition guard | AI proposal import pocket 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal_import_pass` | ai proposal import 白箱 |
| 模块树 | `runtime.mutation.ai_proposal_import_pass` | 新基线 |

---

## 当前事实

BE-001EN-01 已确认 `runtime.mutation_import_pass` 父叶仍未完成:

```text
runtime.mutation_import_pass stop_split: false
runtime.mutation.ai_proposal_import_pass baseline_frozen
old_three_leaf_pause_target_cancelled
```

当前 parent bridge 剩余:

```text
root 1
run 0
backtest 0
mutation 10
test-only 1
total 12
remaining_parent_import_bridge_12
remaining_mutation_import_bridge_10
remaining_parameter_mutation_import_bridge_0
remaining_ai_proposal_import_bridge_10
```

本批冻结 `ai_proposal` pocket，不改写 Rust import。

---

## 目标文件范围

本基线冻结以下 10 个文件:

```text
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/approval_persistence.rs
src/runtime/mutation/ai_proposal/approval_review.rs
src/runtime/mutation/ai_proposal/event_lifecycle.rs
src/runtime/mutation/ai_proposal/proposal_creation.rs
src/runtime/mutation/ai_proposal/record_query.rs
src/runtime/mutation/ai_proposal/sandbox_trigger.rs
src/runtime/mutation/ai_proposal/source_governance_identity.rs
src/runtime/mutation/ai_proposal/static_check.rs
src/runtime/mutation/ai_proposal/status_transition.rs
```

这些文件当前仍通过 `use super::*` 依赖父级白箱输入面。BE-001EO-02 必须先判断是继续拆成更小 import pocket，还是采用受控多文件 rewrite；不得跳过方案直接改写 10 个文件。

---

## 白箱 public 面

本 pocket 对外承接以下 route-facing handler:

```text
create_runtime_ai_proposal
list_runtime_ai_proposals
get_runtime_ai_proposal_detail
list_runtime_approvals
get_runtime_approval_detail
approve_ai_proposal
reject_ai_proposal
claim_ai_proposal_review
```

这些 public 面不得在 import pass 中改变 route signature、response schema、error code、capability guard、approval lock order、sandbox gate、static-check result、event envelope 或 persistence owner。

当前 route facade:

```text
src/backend/runtime/routes/mutation.rs
/api/runtime/ai-proposals
/api/runtime/ai-proposals/:ai_proposal_id
/api/v1/ai/approvals
/api/v1/ai/approvals/:approval_id
/api/v1/ai/proposals/:proposal_id/approve
/api/v1/ai/proposals/:proposal_id/reject
/api/v1/ai/proposals/:proposal_id/claim
```

---

## 白箱内部 helper 面

本基线冻结以下内部 helper 和父子通信面:

```text
persist_approval
load_approval_from_disk
load_runtime_ai_proposal_for_user
ai_proposal_lifecycle_entry
build_runtime_ai_proposal_event
persist_runtime_ai_proposal_transition
load_sandbox_report_for_proposal
ensure_ai_proposal_can_be_approved
spawn_ai_proposal_sandbox_verification
RuntimeAiProposalSourceContext
load_runtime_ai_proposal_source_context
runtime_ai_proposal_governance
runtime_ai_proposal_record_id
validate_hash_identity
validate_ai_model_identity
ai_proposal_static_check_result
ai_proposal_approved_status
update_ai_proposal_status
```

其中:

1. `persist_approval` 与 `load_approval_from_disk` 负责 approval store 读写。
2. `load_runtime_ai_proposal_for_user` 负责 state cache 与 disk record fallback。
3. `build_runtime_ai_proposal_event`、`ai_proposal_lifecycle_entry` 与 `persist_runtime_ai_proposal_transition` 负责 event / lifecycle / persistence 同步。
4. `ensure_ai_proposal_can_be_approved` 与 `spawn_ai_proposal_sandbox_verification` 负责 approval 通过前的 binding、static check 与 sandbox gate。
5. `RuntimeAiProposalSourceContext`、`load_runtime_ai_proposal_source_context`、`runtime_ai_proposal_governance` 与 `runtime_ai_proposal_record_id` 负责 source evidence、governance projection 与 canonical id。
6. `validate_hash_identity`、`validate_ai_model_identity` 与 `ai_proposal_static_check_result` 负责 static-check contract。
7. `ai_proposal_approved_status` 与 `update_ai_proposal_status` 负责受控状态迁移。

---

## 当前隐式输入面

后续显式 import 改写必须从 `use super::*` 中拆出所需输入，至少覆盖:

```text
auth::UserId
State
Path
Query
Json
StatusCode
AppState
ApprovalActionRequest
CreateRuntimeAiProposalRequest
RuntimeAiProposalListQuery
RuntimeApprovalListQuery
RuntimeAiProposalRecord
RuntimeApprovalRecord
RuntimeApprovalLifecycleEntry
RuntimeApprovalLevel
RuntimeApprovalReviewState
RuntimeRollbackPlan
RuntimeAiProposalStatus
RuntimeAiProposalLifecycleEntry
RuntimeAiProposalSourceEvidence
RuntimeAiProposalGovernance
RuntimeAiProposalStaticCheckResult
RuntimeAiProposalStaticCheckDetail
RuntimeAiModelIdentity
RuntimeParameterMutationTarget
RuntimeEvidenceSourceKind
RuntimeGovernanceSnapshot
RuntimeEventEnvelope
FrontendRuntimeEvent
RequestSandboxVerificationRequest
SandboxVerificationReport
SandboxVerdict
StrategyConfigProposalDomain
FsPath
Value
json
anyhow
fs
current_time_ms
normalize_actor_identity
validate_runtime_capability_guard
validate_runtime_parameter_mutation_target
canonical_runtime_parameter_version
governance_with_parameter_version
append_parameter_mutation_events_to_run
load_run_record_from_state
load_backtest_record_from_state
load_runtime_ai_proposal_record
list_runtime_ai_proposal_records
persist_runtime_ai_proposal_record
json_bad_request
json_bad_request_with_details
internal_error
io_error
clean_optional_filter
canonical_json_sha256_digest
sandbox_verification
safe_eprintln
tokio
futures_util::FutureExt
std::sync::atomic::AtomicU64
std::sync::atomic::Ordering
std::time::Duration
qrpc_core_ir::v4::V4BacktestArtifact
```

该列表只是 import 输入面冻结，不代表允许迁移 owner。

---

## 等价边界

BE-001EO-02 及后续实际抽离必须保持:

1. 不改变 ai proposal create / list / detail 与 approval list / detail / approve / reject / claim 的 handler signature。
2. 不改变 capability guard、AI write policy、model identity、hash identity、config-domain binding 或 v4 source-kind static-check 语义。
3. 不改变 approval lock order、review state、reviewer vectors、lifecycle event id、reason code 或 approval persistence owner。
4. 不改变 sandbox verification trigger、retry、report URL、failure lifecycle 或 background task monitoring。
5. 不改变 source evidence、governance projection、parameter version、proposal id digest 或 event envelope。
6. 不新增 sibling horizontal link，不启动 release transition。
7. 不恢复旧的三叶暂停目标；递归队列继续保持 `old_three_leaf_pause_target_cancelled`。

---

## 初步拆分候选

BE-001EO-02 必须从以下候选中选择最小可验收实施单元:

| 候选 | 文件范围 | 备注 |
| --- | --- | --- |
| `runtime.mutation.ai_proposal.record_query_import_pass` | `record_query.rs` | list/detail 查询路径，风险较窄 |
| `runtime.mutation.ai_proposal.source_governance_identity_import_pass` | `source_governance_identity.rs` | source evidence / governance / record id |
| `runtime.mutation.ai_proposal.static_check_import_pass` | `static_check.rs` | static-check contract 与 v4 domain binding |
| `runtime.mutation.ai_proposal.event_lifecycle_import_pass` | `event_lifecycle.rs` | event / lifecycle / persistence transition |
| `runtime.mutation.ai_proposal.approval_persistence_import_pass` | `approval_persistence.rs` | approval disk read/write |
| `runtime.mutation.ai_proposal.approval_review_import_pass` | `approval_review.rs` | approval route handlers 与 lock order |
| `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` | `sandbox_trigger.rs` | sandbox gate 与 background retry |
| `runtime.mutation.ai_proposal.proposal_creation_import_pass` | `proposal_creation.rs` | proposal create route，依赖多个 sibling helper |
| `runtime.mutation.ai_proposal.status_transition_import_pass` | `status_transition.rs` | proposal status transition guard |
| `runtime.mutation.ai_proposal.parent_facade_import_pass` | `ai_proposal.rs` | parent facade / re-export / bridge input |

若 BE-001EO-02 认为 10 文件同批 rewrite 会扩大等价风险，应继续拆小 pocket。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不直接改写 `src/runtime/mutation/ai_proposal.rs` 或 `src/runtime/mutation/ai_proposal/**` import。
- 本批不处理 `src/runtime/mod.rs` root parent bridge。
- 本批不处理 test-only `src/runtime/run_guard.rs`。
- 本批不迁移 route facade、handler owner、state owner、persistence owner、schema owner、frontend caller 或 test asset。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。

---

## 验证要求

本批为 `no code movement` 基线，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_ai_proposal
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001EO-01 完成时，必须说明:

1. 本批次是 `no code movement` 单子叶等价基线。
2. 目标范围为 10 个 `ai_proposal` residual 文件。
3. `use super::*` 尚未改写。
4. 当前 parent bridge 剩余仍为 root 1 / run 0 / backtest 0 / mutation 10 / test-only 1 / total 12。
5. 下一步只能进入 BE-001EO-02 `runtime.mutation.ai_proposal_import_pass` 抽离方案。
6. BE-001EO-02 必须先判断是否继续拆小 pocket，不得跳过方案直接整批改写 10 文件。
7. `src/runtime/mod.rs` 和 test-only `src/runtime/run_guard.rs` 尚未处理。
8. release transition 未启动，未新增 sibling horizontal link。
9. 旧的三叶暂停目标仍为取消状态。

不得宣称 ai proposal import 已改写、mutation import 已完成、parent import bridge 已完全清除、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `413-runtime.mutation.ai_proposal_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 基线冻结 10 个 ai proposal residual 文件、8 个 route-facing handler、18 个内部 helper 和当前隐式输入面。
3. 下一步固定为 BE-001EO-02 `runtime.mutation.ai_proposal_import_pass` 抽离方案。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
