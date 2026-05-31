# v4.16.0 runtime.mutation.parameter_mutation_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DQ-01
> 基准: `356-runtime.mutation_import_pass父叶残余判断.md`
> 目标子叶: `runtime.mutation.parameter_mutation_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DQ-02 `runtime.mutation.parameter_mutation_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DQ-01 `runtime.mutation.parameter_mutation_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | parent import bridge、explicit import pass、minimum batch、release transition guard | 参数变更 import pocket 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation_import_pass` | parameter mutation import 白箱 |
| 模块树 | `runtime.mutation.parameter_mutation_import_pass` | 新基线 |

---

## 当前事实

BE-001DP-01 已确认 `runtime.mutation_import_pass` 父叶仍未完成:

```text
runtime.mutation_import_pass stop_split: false
old_three_leaf_pause_target_cancelled
```

当前 parent bridge 剩余:

```text
root 1
run 0
backtest 0
mutation 20
test-only 1
total 22
remaining_parent_import_bridge_22
remaining_mutation_import_bridge_20
parameter_mutation_import_pass baseline_frozen
```

本批冻结 `parameter_mutation` pocket，不改写 Rust import。

---

## 目标文件范围

本基线冻结以下 10 个文件:

```text
src/runtime/mutation/parameter_mutation.rs
src/runtime/mutation/parameter_mutation/proposal_creation.rs
src/runtime/mutation/parameter_mutation/record_query.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs
```

这些文件当前仍通过 `use super::*` 依赖父级白箱输入面。BE-001DQ-02 必须先判断是分为更小 import pocket，还是采用受控多文件 rewrite；不得跳过方案直接改写 10 个文件。

---

## 白箱 public 面

本 pocket 对外承接以下 runtime parameter mutation handler:

```text
create_runtime_parameter_mutation
list_runtime_parameter_mutations
get_runtime_parameter_mutation_detail
activate_runtime_parameter_mutation
rollback_runtime_parameter_mutation
```

这些 public 面不得在 import pass 中改变 route signature、response schema、error code、capability guard、safe window 语义、event envelope 或 persistence owner。

---

## 白箱内部 helper 面

本基线冻结以下内部 helper 和父子通信面:

```text
runtime_parameter_mutation_record_id
validate_runtime_parameter_mutation_boundary
resolve_runtime_parameter_mutation_boundary
evaluate_runtime_parameter_mutation_safe_window
auto_snapshot_on_activation
runtime_parameter_mutation_rollback_record_id
mutation_lifecycle_entry
persist_runtime_parameter_mutation_transition
```

其中:

1. `runtime_parameter_mutation_record_id` 与 `runtime_parameter_mutation_rollback_record_id` 负责 proposal / rollback record id 的 canonical digest。
2. `validate_runtime_parameter_mutation_boundary`、`resolve_runtime_parameter_mutation_boundary` 与 `evaluate_runtime_parameter_mutation_safe_window` 负责 activation boundary 与 safe window 决策。
3. `auto_snapshot_on_activation` 负责 activation 后的 config generation 与 deployment signature snapshot 副作用。
4. `mutation_lifecycle_entry` 与 `persist_runtime_parameter_mutation_transition` 负责 lifecycle entry 与 mutation record persistence。

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
CreateRuntimeParameterMutationRequest
RuntimeParameterMutationListQuery
ActivateRuntimeParameterMutationRequest
RollbackRuntimeParameterMutationRequest
RuntimeParameterMutationRecord
RuntimeParameterMutationTarget
RuntimeParameterMutationStatus
RuntimeParameterMutationBoundary
RuntimeParameterMutationSafeWindowSnapshot
RuntimeParameterMutationSafeWindowState
RuntimeParameterMutationActivationState
RuntimeParameterMutationLifecycleEntry
RuntimeEvidenceSourceKind
RuntimeGovernanceSnapshot
PaginatedResponse
PaginationQuery
FrontendRuntimeEvent
DeploymentSignatureSnapshot
EventSliceBounds
Value
json
canonical_json_sha256_digest
current_time_ms
normalize_actor_identity
validate_runtime_capability_guard
json_bad_request
json_bad_request_with_details
internal_error
io_error
paginate
clean_optional_filter
load_run_record_from_state
append_parameter_mutation_events_to_run
canonical_runtime_parameter_version
validate_runtime_parameter_mutation_target
build_runtime_parameter_mutation_event
runtime_parameter_mutation_governance
governance_with_parameter_version
mutation_event_contract
load_runtime_parameter_mutation_record
list_runtime_parameter_mutation_records
persist_runtime_parameter_mutation_record
qrpc_runtime::ConfigGenerationEntry
qrpc_core::canonical_json_sha256_digest
crate::runtime_persistence::atomic_write_json
safe_eprintln
std::sync::atomic::Ordering
```

该列表只是 import 输入面冻结，不代表允许迁移 owner。

---

## 等价边界

BE-001DQ-02 及后续实际抽离必须保持:

1. 不改变 parameter mutation proposal、list、detail、activate、rollback 的 handler signature。
2. 不改变 capability guard、boundary validation、safe window denial、activation schedule、rollback schedule 或 no-op rejection 语义。
3. 不改变 event type、reason code、sequence number、lifecycle entry、governance projection 或 parameter version 计算。
4. 不改变 mutation store、run record append、state cache、snapshot store 或 config generation persistence owner。
5. 不新增 sibling horizontal link，不启动 release transition。
6. 不恢复旧的三叶暂停目标；递归队列继续保持 `old_three_leaf_pause_target_cancelled`。

---

## 初步拆分候选

BE-001DQ-02 必须从以下候选中选择最小可验收实施单元:

| 候选 | 文件范围 | 备注 |
| --- | --- | --- |
| `runtime.mutation.parameter_mutation.facade_import_pass` | `parameter_mutation.rs` | 只处理 parent facade / re-export / bridge input |
| `runtime.mutation.parameter_mutation.proposal_creation_import_pass` | `proposal_creation.rs` | proposal 创建路径，依赖 shared governance 与 boundary helper |
| `runtime.mutation.parameter_mutation.record_query_import_pass` | `record_query.rs` | list/detail 查询路径，风险较窄 |
| `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` | `transition_lifecycle.rs` 与 6 个 child | activation / rollback lifecycle，状态副作用最重 |

若 BE-001DQ-02 认为 10 文件同批 rewrite 会扩大等价风险，应继续拆小 pocket；这属于递归流程内的正常选择，不需要恢复“三叶暂停”。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不直接改写 `src/runtime/mutation/parameter_mutation.rs` 或 `src/runtime/mutation/parameter_mutation/**` import。
- 本批不处理 `src/runtime/mutation/ai_proposal.rs` 或 `src/runtime/mutation/ai_proposal/**`。
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
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

后续实际 import pass 至少补跑:

```powershell
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
```

---

## 幻觉检查点

AI 声称 BE-001DQ-01 完成时，必须说明:

1. 本批次是 `no code movement` 单子叶等价基线。
2. 目标范围为 10 个 `parameter_mutation` residual 文件。
3. `use super::*` 尚未改写。
4. 当前 parent bridge 剩余仍为 root 1 / run 0 / backtest 0 / mutation 20 / test-only 1 / total 22。
5. 下一步只能进入 BE-001DQ-02 `runtime.mutation.parameter_mutation_import_pass` 抽离方案。
6. BE-001DQ-02 必须先判断是否继续拆小 pocket，不得跳过方案直接整批改写 10 文件。
7. `ai_proposal`、`src/runtime/mod.rs` 和 test-only `src/runtime/run_guard.rs` 尚未处理。
8. release transition 未启动，未新增 sibling horizontal link。
9. 旧的三叶暂停目标仍为取消状态。

不得宣称 parameter mutation import 已改写、mutation import 已完成、parent import bridge 已完全清除、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `357-runtime.mutation.parameter_mutation_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 基线冻结 10 个 parameter mutation residual 文件、5 个 public handler、8 个内部 helper 和当前隐式输入面。
3. 下一步固定为 BE-001DQ-02 `runtime.mutation.parameter_mutation_import_pass` 抽离方案。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
