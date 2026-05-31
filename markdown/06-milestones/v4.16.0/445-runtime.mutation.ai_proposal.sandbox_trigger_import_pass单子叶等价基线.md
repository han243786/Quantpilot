# v4.16.0 runtime.mutation.ai_proposal.sandbox_trigger_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FB-01
> 上一批: `444-runtime.mutation.ai_proposal_import_pass第八轮父叶残余判断.md`
> 目标子叶: `runtime.mutation.ai_proposal.sandbox_trigger_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.sandbox_trigger_import_pass`
> 代码动作: no code movement
> 下一步: BE-001FB-02 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FB-01 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 单子叶等价基线 | 建立基线 |
| 规范矩阵 | sandbox gate / async retry side effect / explicit import pass / no release transition | 冻结等价边界 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.sandbox_trigger_import_pass` | sandbox trigger import pocket |
| 模块树 | `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` | 单子叶基线 |

---

## 当前白箱边界

`src/runtime/mutation/ai_proposal/sandbox_trigger.rs` 当前仍通过 `use super::*` 取得父级输入面。本轮只冻结真实行为，不改 Rust 代码。

```text
runtime.mutation.ai_proposal.sandbox_trigger_import_pass baseline_frozen
runtime.mutation.ai_proposal.sandbox_trigger_import_pass current_parent_import_bridge: use super::*
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.sandbox_trigger_import_pass
old_three_leaf_pause_target_cancelled
progress_report_instruction_discarded
```

---

## 冻结的 public / parent-visible 方法

```text
ensure_ai_proposal_can_be_approved
spawn_ai_proposal_sandbox_verification
```

`load_sandbox_report_for_proposal` 是本文件私有 helper，不作为 public 方法，但必须纳入等价基线，因为它决定内存 sandbox report 与磁盘 sandbox report 的读取顺序。

---

## 冻结的输入面

当前 import rewrite 预期显式化以下输入，BE-001FB-02/03 不得引入新 owner 或 sibling 横向连接:

```text
super::approval_persistence::persist_approval
crate::sandbox_verification
crate::AppState
crate::RuntimeAiProposalRecord
crate::RuntimeAiProposalStatus
crate::RuntimeApprovalLifecycleEntry
crate::SandboxVerificationReport
crate::SandboxVerdict
crate::RequestSandboxVerificationRequest
crate::current_time_ms
axum::http::StatusCode
futures_util::FutureExt
serde_json::json
std::panic::AssertUnwindSafe
std::time::Duration
tokio::spawn
tokio::time::sleep
safe_eprintln!
```

`safe_eprintln!` 保持 crate-local macro 调用形态，不在本轮迁移。

---

## 等价冻结点

BE-001FB-02/03 必须保持以下行为不变:

```text
load_sandbox_report_for_proposal memory-first lookup
sandbox_verification::load_sandbox_report_from_disk fallback
missing sandbox report maps to ai_proposal_sandbox_required
missing config binding maps to strategy_config_ai_binding_required
non StaticCheckPassed maps to ai_proposal_static_check_required
SandboxVerdict::CandidateUnderperforms maps to ai_proposal_sandbox_failed
StatusCode::LOCKED for all approval gate denials
RequestSandboxVerificationRequest { backtest_id: None, proposal_id: pid.clone() }
three attempts exactly
catch_unwind around run_sandbox_verification
sleep duration 500ms * (attempt + 1)
success updates sandbox_report_url
success persists updated approval
failure pushes RuntimeApprovalLifecycleEntry
failure reason_code SANDBOX_VERIFICATION_FAILED
failure persists updated approval
outer JoinHandle monitor task remains
```

---

## 锁顺序与副作用

必须保持:

```text
approval_records write lock only inside local update block
approval clone collected before persist_approval
persist_approval happens after approval_records write lock block
no await while holding approval_records write lock outside existing shape
state_clone approval_store_dir remains persistence target
```

本轮不得改变审批记录 owner、sandbox report owner、runtime persistence owner 或 async task owner。

---

## 等价守卫

```text
no_sandbox_gate_rule_rewrite
no_sandbox_retry_rewrite
no_async_task_shape_rewrite
no_approval_lifecycle_rewrite
no_persistence_order_rewrite
no_error_payload_rewrite
no_status_code_rewrite
no_visibility_rewrite
no_sibling_owner_migration
```

---

## 预期 residual

BE-001FB-01 完成后 residual 不变化:

```text
remaining_runtime_parent_import_bridge_5
remaining_mutation_import_bridge_4
remaining_ai_proposal_import_bridge_4
```

BE-001FB-03 实际抽离后，才允许下降为:

```text
expected_runtime_parent_import_bridge_4
expected_mutation_import_bridge_3
expected_ai_proposal_import_bridge_3
```

---

## 排除项

本批不处理:

1. 不改 Rust 代码。
2. 不改 `sandbox_trigger.rs` 顶部 import；这属于 BE-001FB-03。
3. 不改 sandbox gate 判定、retry 次数、sleep 时长、panic 捕获、日志、lifecycle 或持久化顺序。
4. 不处理 `approval_review.rs`、`proposal_creation.rs` 或 parent facade import residual。
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

AI 声称 BE-001FB-01 完成时，必须说明:

1. 本批只是 `no code movement` 等价基线。
2. `sandbox_trigger.rs` 仍未实际移除 `use super::*`。
3. 下一步只能进入 BE-001FB-02 抽离方案。
4. 不得宣称 sandbox_trigger、approval_review、proposal_creation、ai proposal parent facade、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `445-runtime.mutation.ai_proposal.sandbox_trigger_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. sandbox gate、async retry side effect、approval lifecycle 和 persistence order 均被等价冻结。
3. BE-001FB-02 成为唯一下一步。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
