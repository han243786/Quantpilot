# v4.16.0 runtime.mutation.ai_proposal.parent_facade_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FH-03
> 基线: `461-runtime.mutation.ai_proposal.parent_facade_import_pass抽离方案.md`
> 目标子叶: `runtime.mutation.ai_proposal.parent_facade_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.parent_facade_import_pass`
> 真实文件: `src/runtime/mutation/ai_proposal.rs`
> 代码动作: single file import rewrite
> 下一步: BE-001FH-04 `runtime.mutation.ai_proposal.parent_facade_import_pass` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FH-03 `runtime.mutation.ai_proposal.parent_facade_import_pass` 实际抽离记录 | staged explicit import pass |
| 规范矩阵 | parent facade import rewrite / hidden input captured / no release transition | 标准档位 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.parent_facade_import_pass` | 清理 parent wildcard import |
| 模块树 | `runtime.mutation.ai_proposal.parent_facade_import_pass` | 单文件 facade import pocket |

---

## 实际变更

```text
BE-001FH-03
runtime.mutation.ai_proposal.parent_facade_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.parent_facade_import_pass
parent_facade_import_pass extraction_complete
parent_facade_import_pass_closeout_ready
single_file_ai_proposal_parent_facade_import_pass
removed_ai_proposal_parent_wildcard_import
removed_parent_private_unused_helper_imports
test_module_explicit_imports_only
hidden_parent_input_captured_RuntimeApprovalListQuery
old_three_leaf_pause_target_cancelled
progress_report_instruction_discarded
```

本批只改写 `src/runtime/mutation/ai_proposal.rs` 的 import 面:

1. 移除 parent facade 顶部 `use super::*`。
2. 删除 parent facade 顶层已转为残余的 child helper imports。
3. 保留 child module declarations 和 public re-export。
4. 将 `v4_ai_proposal_tests` 从 `use super::*` 改为显式 import。
5. 根据编译探针补入 `use super::RuntimeApprovalListQuery;`，保持 `approval_review.rs` 通过父级白箱读取 query type 的既有路径。

---

## 实际 import 面

parent facade 顶层现在只保留一个 hidden parent input:

```rust
use super::RuntimeApprovalListQuery;
```

public facade re-export 保持不变:

```rust
pub(crate) use approval_review::{
    approve_ai_proposal, claim_ai_proposal_review, get_runtime_approval_detail,
    list_runtime_approvals, reject_ai_proposal,
};
pub(crate) use proposal_creation::create_runtime_ai_proposal;
pub(crate) use record_query::{get_runtime_ai_proposal_detail, list_runtime_ai_proposals};
```

测试模块显式输入面:

```rust
use super::sandbox_trigger::ensure_ai_proposal_can_be_approved;
use crate::{
    current_time_ms, ActorIdentity, ReplayWindow, RuntimeAiModelIdentity,
    RuntimeAiProposalConfigDomainBinding, RuntimeAiProposalGovernance, RuntimeAiProposalRecord,
    RuntimeAiProposalSourceEvidence, RuntimeAiProposalStaticCheckResult,
    RuntimeAiProposalStatus, RuntimeEvidenceSourceKind, RuntimeParameterMutationTarget,
    SandboxMetrics, SandboxMetricsDiff, SandboxVerdict, SandboxVerificationReport,
    StrategyConfigProposalDomain,
};
use axum::http::StatusCode;
use serde_json::json;
```

---

## 残余变化

```text
actual_runtime_parent_import_bridge_2_to_1
actual_mutation_import_bridge_1_to_0
actual_ai_proposal_import_bridge_1_to_0
remaining_runtime_parent_import_bridge_1
remaining_mutation_import_bridge_0
remaining_ai_proposal_import_bridge_0
remaining_root_parent_import_bridge_1
```

当前 `rg --line-number "^use super::\\*;" src\\runtime -g "*.rs"` 的生产级 residual 只剩:

```text
src/runtime/mod.rs
```

测试级 / child-local `use super::*` 不归入本批生产级 parent facade residual:

```text
src/runtime/run_guard.rs
src/runtime/mutation/ai_proposal/static_check.rs
```

---

## 等价边界确认

保持不变:

```text
approval_persistence child module declaration
approval_review child module declaration
event_lifecycle child module declaration
proposal_creation child module declaration
record_query child module declaration
sandbox_trigger child module declaration
source_governance_identity child module declaration
static_check child module declaration
status_transition child module declaration
create_runtime_ai_proposal re-export
list_runtime_ai_proposals re-export
get_runtime_ai_proposal_detail re-export
list_runtime_approvals re-export
get_runtime_approval_detail re-export
approve_ai_proposal re-export
reject_ai_proposal re-export
claim_ai_proposal_review re-export
v4_ai_proposal_tests behavior
```

未发生:

```text
no_function_body_change
no_visibility_change
no_child_module_rewrite
no_reexport_rewrite
no_test_semantics_rewrite
no_approval_review_rewrite
no_proposal_creation_rewrite
no_record_query_rewrite
no_sandbox_trigger_rewrite
no_status_transition_rewrite
no_source_governance_rewrite
no_static_check_rewrite
no_event_lifecycle_rewrite
no_sibling_horizontal_link
no_release_transition
```

---

## 下一步边界

下一步只能进入 BE-001FH-04 单叶 closeout:

```text
BE-001FH-04
runtime.mutation.ai_proposal.parent_facade_import_pass
parent_facade_import_pass_closeout_ready
```

BE-001FH-03 不直接设置 `runtime.mutation.ai_proposal.parent_facade_import_pass stop_split: true`，必须先进入 BE-001FH-04 单叶 closeout。

---

## 验证要求

本批提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot v4_ai_proposal_tests::ai_proposal_approval_requires_binding_and_sandbox_report
```

---

## 幻觉检查点

AI 声称 BE-001FH-03 完成时，必须说明:

1. 本批只改写 `src/runtime/mutation/ai_proposal.rs` import 面。
2. `RuntimeApprovalListQuery` 是编译探针发现的 hidden parent input，已通过 parent facade 显式保留。
3. `runtime.mutation.ai_proposal.parent_facade_import_pass` 尚未 closeout。
4. 下一步只能进入 BE-001FH-04 单叶 closeout。
5. 当前生产级 runtime parent bridge residual 仍有 `src/runtime/mod.rs`。

不得声称 ai proposal import pass、mutation import pass、runtime parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `462-runtime.mutation.ai_proposal.parent_facade_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/mutation/ai_proposal.rs` 顶部 parent wildcard import 已移除。
3. 编译探针 hidden input `RuntimeApprovalListQuery` 已显式保留。
4. BE-001FH-04 成为唯一下一步。
5. Rust / 治理 / 全量树门禁均通过。
