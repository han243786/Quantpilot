# v4.16.0 runtime.mutation.ai_proposal.sandbox_trigger_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FB-03
> 基线: `446-runtime.mutation.ai_proposal.sandbox_trigger_import_pass抽离方案.md`
> 目标子叶: `runtime.mutation.ai_proposal.sandbox_trigger_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.sandbox_trigger_import_pass`
> 代码动作: actual import rewrite
> 下一步: BE-001FB-04 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FB-03 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 抽离记录 | 实际抽离 |
| 规范矩阵 | explicit import pass / single-file import rewrite / no release transition | 顶部 import 显式化 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.sandbox_trigger_import_pass` | sandbox trigger 输入显式化 |
| 模块树 | `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` | 抽离完成，待 closeout |

---

## 实际改动

本批只改写 `src/runtime/mutation/ai_proposal/sandbox_trigger.rs` 顶部 import:

```text
runtime.mutation.ai_proposal.sandbox_trigger_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.sandbox_trigger_import_pass
sandbox_trigger_import_pass extraction_done
removed use super::*
single file import rewrite
sandbox_trigger_explicit_imports
```

改动后的显式输入面:

```rust
use super::approval_persistence::persist_approval;
use crate::{
    current_time_ms, sandbox_verification, AppState, RequestSandboxVerificationRequest,
    RuntimeAiProposalRecord, RuntimeAiProposalStatus, RuntimeApprovalLifecycleEntry,
    SandboxVerdict, SandboxVerificationReport,
};
use axum::http::StatusCode;
use futures_util::FutureExt;
use serde_json::json;
```

`safe_eprintln!` 保持 crate-local macro 调用形态；`std::panic::AssertUnwindSafe` 与 `std::time::Duration` 保持完全限定路径。

---

## 等价保持

以下内容未改:

```text
load_sandbox_report_for_proposal body
ensure_ai_proposal_can_be_approved body
spawn_ai_proposal_sandbox_verification body
pub(super) visibility
memory-first sandbox report lookup
disk fallback behavior
all StatusCode::LOCKED mappings
all json error payload keys
RequestSandboxVerificationRequest shape
retry attempt count
catch_unwind placement
sleep duration formula
success sandbox_report_url update
failure RuntimeApprovalLifecycleEntry shape
failure reason_code SANDBOX_VERIFICATION_FAILED
persist_approval call placement
outer JoinHandle monitor task
approval_records write lock block shape
```

本批保持:

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

## residual 变化

BE-001FB-03 前，`src/runtime` 内仍有 5 个 parent wildcard import residual；本批完成后，真实 residual 为:

```text
actual_runtime_parent_import_bridge_5_to_4
actual_mutation_import_bridge_4_to_3
actual_ai_proposal_import_bridge_4_to_3
remaining_runtime_parent_import_bridge_4
remaining_mutation_import_bridge_3
remaining_ai_proposal_import_bridge_3
```

仍待处理:

```text
src/runtime/mod.rs
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/approval_review.rs
src/runtime/mutation/ai_proposal/proposal_creation.rs
```

---

## 排除项

本批未处理:

1. 未改函数体、可见性、sandbox gate 判定、retry、panic 捕获、日志、lifecycle 或 persist 顺序。
2. 未处理 `approval_review.rs`、`proposal_creation.rs` 或 parent facade import residual。
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

AI 声称 BE-001FB-03 完成时，必须说明:

1. 本批只完成 `sandbox_trigger.rs` 顶部 import 显式化。
2. `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 尚未 closeout，下一步只能进入 BE-001FB-04 单叶 closeout。
3. 不得宣称 approval_review、proposal_creation、ai proposal parent facade、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `447-runtime.mutation.ai_proposal.sandbox_trigger_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/mutation/ai_proposal/sandbox_trigger.rs` 不再使用 `use super::*`。
3. 函数体、可见性、sandbox gate、retry、panic 捕获、lifecycle 与 persist 顺序均未改。
4. BE-001FB-04 单叶 closeout 成为唯一下一步。
5. 治理门禁、全量树覆盖和 Rust 验证均通过。
