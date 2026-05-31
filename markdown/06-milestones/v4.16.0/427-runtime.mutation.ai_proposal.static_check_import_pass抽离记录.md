# v4.16.0 runtime.mutation.ai_proposal.static_check_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001ET-03
> 基线: `426-runtime.mutation.ai_proposal.static_check_import_pass抽离方案.md`
> 目标子叶: `runtime.mutation.ai_proposal.static_check_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.static_check_import_pass`
> 代码动作: actual import rewrite
> 下一步: BE-001ET-04 `runtime.mutation.ai_proposal.static_check_import_pass` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001ET-03 `runtime.mutation.ai_proposal.static_check_import_pass` 抽离记录 | 实际抽离 |
| 规范矩阵 | explicit import pass / single-file import rewrite / no release transition | 顶部 import 显式化 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.static_check_import_pass` | static check import 输入显式化 |
| 模块树 | `runtime.mutation.ai_proposal.static_check_import_pass` | 抽离完成，待 closeout |

---

## 实际改动

本批只改写 `src/runtime/mutation/ai_proposal/static_check.rs` 顶部 import:

```text
runtime.mutation.ai_proposal.static_check_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.static_check_import_pass
static_check_import_pass extraction_done
removed use super::*
single file import rewrite
```

改动后的显式输入面:

```rust
use crate::{
    json_bad_request, CreateRuntimeAiProposalRequest, RuntimeAiModelIdentity,
    RuntimeAiProposalStaticCheckDetail, RuntimeAiProposalStaticCheckResult,
    RuntimeAiProposalStatus, RuntimeEvidenceSourceKind, RuntimeParameterMutationTarget,
    StrategyConfigProposalDomain,
};
use axum::http::StatusCode;
use serde_json::{json, Value};

#[cfg(test)]
use crate::RuntimeAiProposalConfigDomainBinding;
```

`RuntimeAiProposalConfigDomainBinding` 仅由文件内测试模块使用，因此拆为 `#[cfg(test)]` 顶部 import，避免 release/dev build 出现 unused import warning。

---

## 等价保持

以下内容未改:

```text
validate_hash_identity body
is_valid_hash_identity body
validate_ai_model_identity body
ai_proposal_static_check_result body
is_v4_ai_proposal_target body
expected_config_domain_for_target body
validate_ai_proposal_config_domain_binding body
analyze_v4_backtest_artifact_for_ai body
v4_ai_proposal_static_check_tests
pub(super) visibility
#[allow(dead_code)]
reason_code strings
detail code strings
StatusCode mapping
```

本批保持:

```text
no_static_check_rule_rewrite
no_hash_format_rewrite
no_model_identity_rewrite
no_config_domain_binding_rewrite
no_v4_source_kind_gate_rewrite
no_artifact_analysis_rewrite
no_visibility_rewrite
no_test_semantics_rewrite
no_sibling_owner_migration
old_three_leaf_pause_target_cancelled
```

---

## residual 变化

BE-001ET-03 前，`src/runtime` 内仍有 9 个 parent wildcard import residual；本批完成后，真实 residual 为:

```text
actual_runtime_parent_import_bridge_9_to_8
actual_mutation_import_bridge_8_to_7
actual_ai_proposal_import_bridge_8_to_7
remaining_runtime_parent_import_bridge_8
remaining_mutation_import_bridge_7
remaining_ai_proposal_import_bridge_7
```

仍待处理:

```text
src/runtime/mod.rs
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/approval_persistence.rs
src/runtime/mutation/ai_proposal/approval_review.rs
src/runtime/mutation/ai_proposal/event_lifecycle.rs
src/runtime/mutation/ai_proposal/proposal_creation.rs
src/runtime/mutation/ai_proposal/sandbox_trigger.rs
src/runtime/mutation/ai_proposal/status_transition.rs
```

---

## 排除项

本批未处理:

1. 未改函数体、测试、可见性或 reason code。
2. 未处理其他 ai proposal child import residual。
3. 未处理 `src/runtime/mutation/ai_proposal.rs` parent facade。
4. 未处理 `src/runtime/mod.rs` root parent bridge。
5. 未迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
6. 未新增 sibling 横向连接。
7. 未启动 release transition。

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
cargo test -p quantpilot v4_ai_proposal_static_check_tests
```

---

## 幻觉检查点

AI 声称 BE-001ET-03 完成时，必须说明:

1. 本批只完成 `static_check.rs` 顶部 import 显式化。
2. `runtime.mutation.ai_proposal.static_check_import_pass` 尚未 closeout，下一步只能进入 BE-001ET-04 单叶 closeout。
3. 不得宣称 ai proposal import、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `427-runtime.mutation.ai_proposal.static_check_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/mutation/ai_proposal/static_check.rs` 不再使用 `use super::*`。
3. 函数体、测试、可见性、reason code、status、domain map 与 sibling owner 均未改。
4. BE-001ET-04 单叶 closeout 成为唯一下一步。
5. 治理门禁、全量树覆盖和 Rust 验证均通过。
