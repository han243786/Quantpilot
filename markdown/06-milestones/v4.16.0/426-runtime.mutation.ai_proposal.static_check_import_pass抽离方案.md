# v4.16.0 runtime.mutation.ai_proposal.static_check_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001ET-02
> 基线: `425-runtime.mutation.ai_proposal.static_check_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.mutation.ai_proposal.static_check_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.static_check_import_pass`
> 代码动作: no code movement
> 下一步: BE-001ET-03 `runtime.mutation.ai_proposal.static_check_import_pass` 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001ET-02 `runtime.mutation.ai_proposal.static_check_import_pass` 抽离方案 | 方案优化 |
| 规范矩阵 | single-file import rewrite / static check rule freeze / no release transition | 固定抽离边界 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.static_check_import_pass` | static check import pocket |
| 模块树 | `runtime.mutation.ai_proposal.static_check_import_pass` | 抽离方案 |

---

## 方案结论

BE-001ET-03 只能改写 `src/runtime/mutation/ai_proposal/static_check.rs` 顶部 import，禁止改函数体、测试、可见性、reason code、status、domain map 或 sibling owner。

```text
runtime.mutation.ai_proposal.static_check_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.static_check_import_pass
static_check_import_pass plan_frozen
single_file_static_check_import_rewrite
next_step: BE-001ET-03 extraction record
```

---

## 允许改动

唯一允许的 Rust 改动:

```diff
-use super::*;
+use crate::{
+    json_bad_request, CreateRuntimeAiProposalRequest, RuntimeAiModelIdentity,
+    RuntimeAiProposalConfigDomainBinding, RuntimeAiProposalStaticCheckDetail,
+    RuntimeAiProposalStaticCheckResult, RuntimeAiProposalStatus, RuntimeEvidenceSourceKind,
+    RuntimeParameterMutationTarget, StrategyConfigProposalDomain,
+};
+use axum::http::StatusCode;
+use serde_json::{json, Value};
```

允许 cargo fmt 对 import 分组做机械格式化。

---

## 不允许改动

BE-001ET-03 不得修改:

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

---

## 等价守卫

必须保持:

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

## 残余预期

BE-001ET-03 完成后，预期 residual 从:

```text
remaining_parent_import_bridge_10
remaining_mutation_import_bridge_8
remaining_ai_proposal_import_bridge_8
```

降为:

```text
remaining_parent_import_bridge_9
remaining_mutation_import_bridge_7
remaining_ai_proposal_import_bridge_7
```

`src/runtime/mod.rs` 与 `src/runtime/mutation/ai_proposal.rs` 仍不在 BE-001ET-03 范围内。

---

## 排除项

本批不处理:

1. 不改 Rust 代码。
2. 不执行 BE-001ET-03 实际 import rewrite。
3. 不处理其他 ai proposal child import residual。
4. 不处理 parent facade 或 root parent bridge。
5. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
6. 不新增 sibling 横向连接。
7. 不启动 release transition。

---

## 验证要求

本批为 `no code movement` 抽离方案，提交前至少执行:

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

AI 声称 BE-001ET-02 完成时，必须说明:

1. 本批只是 `no code movement` 抽离方案。
2. BE-001ET-03 只能改写 `src/runtime/mutation/ai_proposal/static_check.rs` 顶部 import。
3. 不得宣称 `static_check.rs` 已完成实际抽离。
4. 不得宣称 ai proposal import、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `426-runtime.mutation.ai_proposal.static_check_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001ET-03 的允许改动被限制为单文件 import rewrite。
3. BE-001ET-03 成为唯一下一步。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
