# v4.16.0 runtime.mutation.ai_proposal_import_pass 第八轮父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FA-01
> 上一批: `443-runtime.mutation.ai_proposal.status_transition_import_pass单叶closeout.md`
> 目标父叶: `runtime.mutation.ai_proposal_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal_import_pass`
> 判定: `runtime.mutation.ai_proposal_import_pass stop_split: false`
> 代码动作: no code movement
> 下一步: BE-001FB-01 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FA-01 `runtime.mutation.ai_proposal_import_pass` 第八轮父叶残余判断 | 父叶重判 |
| 规范矩阵 | parent import bridge / explicit import pass / no release transition | 继续父叶拆分 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal_import_pass` | ai proposal import pass 父叶重判 |
| 模块树 | `runtime.mutation.ai_proposal_import_pass` | `stop_split: false` |

---

## 父叶重判结论

BE-001EZ-04 已完成 `status_transition.rs` import pocket closeout，但 `runtime.mutation.ai_proposal_import_pass` 父叶仍存在 4 个 ai proposal parent wildcard import residual。当前父叶不能 closeout，必须继续按单子叶方式处理。

```text
runtime.mutation.ai_proposal_import_pass eighth_parent_residual_judgment
runtime.mutation.ai_proposal_import_pass stop_split: false
sandbox_trigger_import_pass_selected
remaining_runtime_parent_import_bridge_5
remaining_mutation_import_bridge_4
remaining_ai_proposal_import_bridge_4
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
src/runtime/mutation/ai_proposal/sandbox_trigger.rs
```

`src/runtime/mod.rs` 属于 root parent bridge，`src/runtime/mutation/ai_proposal.rs` 属于 ai proposal parent facade，均不在本轮直接处理。

---

## 子叶选择表

| 候选子叶 | 文件 | 判断 | 本轮决定 |
| --- | --- | --- | --- |
| `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` | `sandbox_trigger.rs` | sandbox verification spawn 与 approval gate，承接 status transition 之后的审批前置门禁和异步副作用 | 采纳 |
| `runtime.mutation.ai_proposal.approval_review_import_pass` | `approval_review.rs` | review / approve / reject route-facing handlers，会调用 sandbox gate 与 status helper | 延后 |
| `runtime.mutation.ai_proposal.proposal_creation_import_pass` | `proposal_creation.rs` | create handler，依赖 source/governance/static check/event lifecycle/approval persistence/sandbox trigger | 延后 |
| `runtime.mutation.ai_proposal.parent_facade_import_pass` | `ai_proposal.rs` | child module declaration / re-export / parent facade | 最后处理 |

---

## sandbox_trigger 选择理由

BE-001FB-01 选择 `sandbox_trigger.rs`，原因:

1. 它是 `approval_review.rs` approve 路径的前置门禁，先显式化可降低后续 route-facing handler 的隐式输入面。
2. 它同时持有 sandbox report 读取、retry side effect、approval lifecycle 失败记录与 approval persistence 调用，属于高风险副作用边界，适合先冻结等价基线。
3. `proposal_creation.rs` 会触发 sandbox verification spawn，先固化 spawn leaf 可以降低 create handler 后续抽离复杂度。
4. 当前目标仍是 import 输入面显式化，不触碰 sandbox 判定、retry 策略、持久化顺序或异步任务语义。

---

## BE-001FB-01 冻结范围

下一批只能建立 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 单子叶等价基线，冻结以下边界:

```text
load_sandbox_report_for_proposal
ensure_ai_proposal_can_be_approved
spawn_ai_proposal_sandbox_verification
AppState
RuntimeAiProposalRecord
RuntimeAiProposalStatus::StaticCheckPassed
SandboxVerificationReport
SandboxVerdict::CandidateUnderperforms
RequestSandboxVerificationRequest
RuntimeApprovalLifecycleEntry
StatusCode::LOCKED
futures_util::FutureExt
sandbox_verification::load_sandbox_report_from_disk
sandbox_verification::run_sandbox_verification
persist_approval
current_time_ms
safe_eprintln!
tokio::spawn
tokio::time::sleep
```

必须保持:

```text
no_sandbox_gate_rule_rewrite
no_sandbox_retry_rewrite
no_async_task_shape_rewrite
no_approval_lifecycle_rewrite
no_persistence_order_rewrite
no_error_payload_rewrite
no_visibility_rewrite
no_sibling_owner_migration
```

---

## 排除项

本批不处理:

1. 不改 Rust 代码。
2. 不处理 `sandbox_trigger.rs` 顶部 import；这属于 BE-001FB-03。
3. 不处理 `approval_review.rs`、`proposal_creation.rs` 或 parent facade import residual。
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

AI 声称 BE-001FA-01 完成时，必须说明:

1. 本批只是 `no code movement` 父叶残余判断。
2. `runtime.mutation.ai_proposal_import_pass stop_split: false`。
3. 下一步只能进入 BE-001FB-01 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 单子叶等价基线。
4. 不得宣称 sandbox_trigger、approval_review、proposal_creation、ai proposal parent facade、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `444-runtime.mutation.ai_proposal_import_pass第八轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶保持 `stop_split: false`，下一颗子叶固定为 `sandbox_trigger_import_pass`。
3. BE-001FB-01 成为唯一下一步。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
