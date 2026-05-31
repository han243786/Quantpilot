# v4.16.0 runtime.mutation.ai_proposal.static_check_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001ET-01
> 父叶判定: `424-runtime.mutation.ai_proposal_import_pass第四轮父叶残余判断.md`
> 目标子叶: `runtime.mutation.ai_proposal.static_check_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.static_check_import_pass`
> 代码动作: no code movement
> 下一步: BE-001ET-02 `runtime.mutation.ai_proposal.static_check_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001ET-01 `runtime.mutation.ai_proposal.static_check_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | explicit import pass / static check rule freeze / no release transition | 冻结静态校验输入面 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.static_check_import_pass` | static check 白箱 |
| 模块树 | `runtime.mutation.ai_proposal.static_check_import_pass` | 新基线 |

---

## 基线结论

本批只冻结 `src/runtime/mutation/ai_proposal/static_check.rs` 的当前等价边界，不改 Rust 代码。

```text
runtime.mutation.ai_proposal.static_check_import_pass baseline_frozen
runtime.mutation.ai_proposal.static_check_import_pass no_code_movement
src/runtime/mutation/ai_proposal/static_check.rs
current_parent_import_bridge: use super::*
next_step: BE-001ET-02 extraction plan
```

`static_check.rs` 是 ai proposal write path 的静态校验叶子，负责在 create handler 写入 record 前输出 deterministic static check result。它不是 route facade，也不是 persistence owner。

---

## 白箱节点

| 项 | 当前边界 |
| --- | --- |
| 输入 | `CreateRuntimeAiProposalRequest`、old/proposed parameter version、source event count、checked timestamp、AI model identity、target module/path、config domain binding |
| 输出 | `Result<(), (StatusCode, String)>`、`RuntimeAiProposalStaticCheckResult`、`RuntimeAiProposalStaticCheckDetail`、v4 artifact analysis `Value` |
| 处理者 | `validate_hash_identity`、`validate_ai_model_identity`、`ai_proposal_static_check_result`、`validate_ai_proposal_config_domain_binding` |
| 调用方 | `src/runtime/mutation/ai_proposal.rs` parent facade 与 proposal creation path |
| 禁止事项 | 不改校验规则、不改 reason code、不改 status、不改 domain map、不新增 sibling 横向连接 |

---

## 当前 public / 可见入口

本子叶对父模块暴露 3 个 `pub(super)` helper:

```text
validate_hash_identity
validate_ai_model_identity
ai_proposal_static_check_result
```

文件内私有 helper:

```text
is_valid_hash_identity
is_v4_ai_proposal_target
expected_config_domain_for_target
validate_ai_proposal_config_domain_binding
analyze_v4_backtest_artifact_for_ai
```

测试模块:

```text
v4_ai_proposal_static_check_tests
v4_ai_proposal_static_check_requires_backtest_source
ai_proposal_static_check_requires_config_domain_binding
ai_proposal_static_check_accepts_matching_config_domain_binding
v4_artifact_analysis_summarizes_trajectory_and_fill_rate
```

---

## 当前隐式输入面

当前文件顶部仍为:

```rust
use super::*;
```

BE-001ET-03 预期只把该 parent wildcard import 收敛为显式输入面。预期输入面包括:

```rust
use crate::{
    json_bad_request, CreateRuntimeAiProposalRequest, RuntimeAiModelIdentity,
    RuntimeAiProposalConfigDomainBinding, RuntimeAiProposalStaticCheckDetail,
    RuntimeAiProposalStaticCheckResult, RuntimeAiProposalStatus, RuntimeEvidenceSourceKind,
    RuntimeParameterMutationTarget, StrategyConfigProposalDomain,
};
use axum::http::StatusCode;
use serde_json::{json, Value};
```

该预期仅作为输入面基线，真正代码改写必须等 BE-001ET-03。

---

## 等价边界

### Hash identity

必须保持:

```text
sha256:<64 lower hex>
trim before strip_prefix
error target/label passthrough
bad_request status mapping
```

不得改变 `validate_hash_identity` 和 `is_valid_hash_identity` 的返回语义。

### Model identity

必须保持:

```text
model.provider non-empty
model.model non-empty
model.model_version non-empty
bad_request on missing identity field
```

不得改变错误信息、字段顺序或 `Ok(())` 条件。

### Static check result

必须保持以下 detail 生成规则:

```text
missing_source_evidence
noop_parameter_version
missing_reason
strategy_config_ai_binding_required
strategy_config_ai_binding_domain_mismatch
strategy_config_ai_binding_before_digest_invalid
strategy_config_ai_binding_after_digest_invalid
strategy_config_ai_binding_before_digest_mismatch
strategy_config_ai_binding_after_digest_mismatch
strategy_config_ai_binding_evidence_required
v4_proposal_requires_backtest_artifact
non_v4_proposal_requires_run_source
```

必须保持 pass/fail 输出:

```text
RuntimeAiProposalStatus::StaticCheckPassed
RuntimeAiProposalStatus::StaticCheckFailed
AI_PROPOSAL_STATIC_CHECK_PASSED
AI_PROPOSAL_STATIC_CHECK_FAILED
checked_at_ms passthrough
```

### Config domain binding

必须保持 `expected_config_domain_for_target` 映射:

```text
builtin.data.kline -> Market
builtin.data.quote -> Market
builtin.risk.global -> Risk
builtin.execution.paper -> Execution
builtin.runtime.control -> Execution
v4.machine.param -> StateMachine
v4.transition.guard -> StateMachine
builtin.intent.* -> Observation
builtin.agent.* -> Observation
default -> AiGovernance
```

不得改变 before/after digest 与 old/proposed parameter version 的对比逻辑。

### v4 artifact analysis

`analyze_v4_backtest_artifact_for_ai` 当前为 `#[allow(dead_code)]` helper，仍需保持:

```text
analysis_version: quantpilot/v4-ai-trajectory-analysis/v1
machine_trajectory aggregation
risk_plane_decisions aggregation
microstructure fill_rate passthrough/default
```

---

## 不变量

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

## 排除项

本批不处理:

1. 不改 Rust 代码。
2. 不删除 `use super::*`。
3. 不改函数体、测试、可见性或 reason code。
4. 不处理其他 ai proposal child import residual。
5. 不处理 `src/runtime/mutation/ai_proposal.rs` parent facade。
6. 不处理 `src/runtime/mod.rs` root parent bridge。
7. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
8. 不新增 sibling 横向连接。
9. 不启动 release transition。

---

## 验证要求

本批为 `no code movement` 基线，提交前至少执行:

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

AI 声称 BE-001ET-01 完成时，必须说明:

1. 本批只是 `no code movement` 单子叶等价基线。
2. `static_check.rs` 仍未实际删除 `use super::*`。
3. 下一步只能进入 BE-001ET-02 `runtime.mutation.ai_proposal.static_check_import_pass` 抽离方案。
4. 不得宣称 static_check import、ai proposal import、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `425-runtime.mutation.ai_proposal.static_check_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `static_check.rs` 白箱输入、输出、处理者、调用方和禁止事项已冻结。
3. 下一步固定为 BE-001ET-02 抽离方案。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
