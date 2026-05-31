# v4.16.0 runtime.mutation.ai_proposal.parent_facade_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FH-02
> 基线: `460-runtime.mutation.ai_proposal.parent_facade_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.mutation.ai_proposal.parent_facade_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.parent_facade_import_pass`
> 真实文件: `src/runtime/mutation/ai_proposal.rs`
> 代码动作: no code movement
> 下一步: BE-001FH-03 `runtime.mutation.ai_proposal.parent_facade_import_pass` 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FH-02 `runtime.mutation.ai_proposal.parent_facade_import_pass` 抽离方案 | staged explicit import pass |
| 规范矩阵 | parent facade import plan / single file rewrite / no release transition | 标准档位 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.parent_facade_import_pass` | 固定 BE-001FH-03 改动边界 |
| 模块树 | `runtime.mutation.ai_proposal.parent_facade_import_pass` | 单文件 facade import pocket |

---

## 方案锁定

```text
BE-001FH-02
BE-001FH-03
runtime.mutation.ai_proposal.parent_facade_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.parent_facade_import_pass
parent_facade_import_pass plan_frozen
parent_facade_import_pass extraction_ready
single_file_ai_proposal_parent_facade_import_pass
remove_ai_proposal_parent_wildcard_import
remove_parent_private_unused_helper_imports
test_module_explicit_imports_only
hidden_parent_input_probe_required
test_module_explicit_import_probe_required
parent_private_helper_alias_probe_required
no code movement
old_three_leaf_pause_target_cancelled
progress_report_instruction_discarded
```

BE-001FH-03 只允许改写一个文件:

```text
src/runtime/mutation/ai_proposal.rs
```

允许的 Rust 改动仅限 import 面:

```diff
-use super::*;
```

并且允许删除 parent facade 顶层已转为残余的 helper imports:

```text
load_approval_from_disk
persist_approval
ai_proposal_lifecycle_entry
build_runtime_ai_proposal_event
persist_runtime_ai_proposal_transition
load_runtime_ai_proposal_for_user
ensure_ai_proposal_can_be_approved
spawn_ai_proposal_sandbox_verification
load_runtime_ai_proposal_source_context
runtime_ai_proposal_governance
runtime_ai_proposal_record_id
ai_proposal_static_check_result
validate_ai_model_identity
validate_hash_identity
ai_proposal_approved_status
update_ai_proposal_status
```

测试模块只能把 `use super::*` 改为显式 import，不能改测试断言、数据构造、状态机检查或 sandbox report 写入。

---

## 预期显式输入面

BE-001FH-03 预期 parent facade 顶层只保留 child module declarations 与 public re-export，不新增替代 parent wildcard import。`v4_ai_proposal_tests` 需要显式引入测试使用的 schema、helper 和宏:

```rust
use super::sandbox_trigger::ensure_ai_proposal_can_be_approved;
use crate::{
    current_time_ms, ActorIdentity, ReplayWindow, RuntimeAiModelIdentity,
    RuntimeAiProposalConfigDomainBinding, RuntimeAiProposalGovernance, RuntimeAiProposalRecord,
    RuntimeAiProposalSourceEvidence, RuntimeAiProposalStaticCheckResult,
    RuntimeAiProposalStatus, RuntimeEvidenceSourceKind, RuntimeParameterMutationTarget,
    SandboxMetrics, SandboxMetricsDiff, SandboxVerificationReport, SandboxVerdict,
    StrategyConfigProposalDomain,
};
use axum::http::StatusCode;
use serde_json::json;
```

如果编译发现隐藏父级输入，BE-001FH-03 只能补充该测试模块或 parent facade import 面，不得改 child 文件或 handler 逻辑。

---

## 等价边界

必须保持不变:

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

不得改写:

```text
no_function_body_change
no_visibility_change
no_child_module_rewrite
no_reexport_rewrite
no_private_helper_alias_rewrite
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

## 预期残余变化

BE-001FH-03 通过后，预期 runtime parent bridge residual:

```text
expected_remaining_runtime_parent_import_bridge_1
expected_remaining_mutation_import_bridge_0
expected_remaining_ai_proposal_import_bridge_0
```

当前父叶仍不在本方案中 closeout；BE-001FH-03 完成后必须先进入 BE-001FH-04 单叶 closeout。

---

## 禁止项

本方案明确禁止:

1. 不修改任何 `src/runtime/mutation/ai_proposal/*.rs` child 文件。
2. 不修改 `src/runtime/mod.rs` 的 re-export 面或 root parent bridge。
3. 不删除或重命名 child module。
4. 不改 public handler re-export。
5. 不修改 handler 函数体、状态机、锁顺序、持久化顺序、事件 payload 或错误 payload。
6. 不移动 state owner、schema、storage、frontend caller 或 test asset。
7. 不启动发布过渡连接。

---

## 下一步边界

下一步只允许进入 BE-001FH-03 实际抽离记录:

```text
BE-001FH-03
runtime.mutation.ai_proposal.parent_facade_import_pass
single_file_ai_proposal_parent_facade_import_pass
remove_ai_proposal_parent_wildcard_import
test_module_explicit_imports_only
```

BE-001FH-03 完成后必须先验证 Rust 编译、`api_ai_proposal` 与内部 `v4_ai_proposal_tests`，再进入 BE-001FH-04 单叶 closeout。

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

AI 声称 BE-001FH-02 完成时，必须说明:

1. 本批是 `no code movement`。
2. 只建立 `parent_facade_import_pass plan_frozen`。
3. BE-001FH-03 只能改写 `src/runtime/mutation/ai_proposal.rs` 的 import 面。
4. 测试模块只能从 `use super::*` 改成显式 import，不能改测试语义。
5. 下一步只能进入 BE-001FH-03 实际抽离记录。
6. 旧三叶暂停目标保持取消: `old_three_leaf_pause_target_cancelled`。

不得声称 parent facade import 已改写、ai proposal import pass 已完成、mutation import pass 已完成、parent import bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `461-runtime.mutation.ai_proposal.parent_facade_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001FH-02 只冻结方案，不改 Rust。
3. BE-001FH-03 改动边界固定为单文件 import rewrite。
4. 下一步固定为 BE-001FH-03 实际抽离记录。
5. Rust / 治理 / 全量树门禁均通过。
