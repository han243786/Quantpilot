# v4.16.0 runtime.mutation.ai_proposal_import_pass 第四轮父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001ES-01
> 上一批: `423-runtime.mutation.ai_proposal.source_governance_identity_import_pass单叶closeout.md`
> 目标父叶: `runtime.mutation.ai_proposal_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal_import_pass`
> 判定: `runtime.mutation.ai_proposal_import_pass stop_split: false`
> 代码动作: no code movement
> 下一步: BE-001ET-01 `runtime.mutation.ai_proposal.static_check_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001ES-01 `runtime.mutation.ai_proposal_import_pass` 第四轮父叶残余判断 | 父叶重判 |
| 规范矩阵 | parent import bridge / explicit import pass / no release transition | 保持父叶继续拆分 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal_import_pass` | ai proposal import pass 父叶重判 |
| 模块树 | `runtime.mutation.ai_proposal_import_pass` | `stop_split: false` |

---

## 父叶重判结论

BE-001ER-04 已完成 `source_governance_identity.rs` 的 import pocket closeout，但 `runtime.mutation.ai_proposal_import_pass` 父叶仍存在 8 个 production parent wildcard import residual。当前父叶不能 closeout，必须继续按单子叶方式处理。

```text
runtime.mutation.ai_proposal_import_pass fourth_parent_residual_judgment
runtime.mutation.ai_proposal_import_pass stop_split: false
static_check_import_pass_selected
remaining_parent_import_bridge_10
remaining_mutation_import_bridge_8
remaining_ai_proposal_import_bridge_8
old_three_leaf_pause_target_cancelled
```

本批不改 Rust 代码，只做父叶残余判断和下一颗子叶选择。

---

## 当前 residual 清单

```text
src/runtime/mod.rs
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/approval_persistence.rs
src/runtime/mutation/ai_proposal/approval_review.rs
src/runtime/mutation/ai_proposal/event_lifecycle.rs
src/runtime/mutation/ai_proposal/proposal_creation.rs
src/runtime/mutation/ai_proposal/sandbox_trigger.rs
src/runtime/mutation/ai_proposal/static_check.rs
src/runtime/mutation/ai_proposal/status_transition.rs
```

`src/runtime/mod.rs` 属于 root parent bridge；`src/runtime/mutation/ai_proposal.rs` 属于 ai proposal parent facade，均不在本轮直接处理。

---

## 子叶选择表

| 候选子叶 | 文件 | 判断 | 本轮决定 |
| --- | --- | --- | --- |
| `runtime.mutation.ai_proposal.static_check_import_pass` | `src/runtime/mutation/ai_proposal/static_check.rs` | 纯校验与 domain binding helper，IO/async 依赖少，适合作为下一颗单子叶 | 采纳 |
| `runtime.mutation.ai_proposal.event_lifecycle_import_pass` | `event_lifecycle.rs` | lifecycle entry / event persistence，依赖 status 与 record 语义 | 延后 |
| `runtime.mutation.ai_proposal.approval_persistence_import_pass` | `approval_persistence.rs` | approval disk persistence，涉及 record load/persist | 延后 |
| `runtime.mutation.ai_proposal.approval_review_import_pass` | `approval_review.rs` | review / approve / reject route-facing handlers，依赖 sandbox 与 persistence | 延后 |
| `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` | `sandbox_trigger.rs` | sandbox verification spawn 与 approval gate | 延后 |
| `runtime.mutation.ai_proposal.status_transition_import_pass` | `status_transition.rs` | status helper 与 transition side effect | 延后 |
| `runtime.mutation.ai_proposal.proposal_creation_import_pass` | `proposal_creation.rs` | create handler，依赖 source/governance/static check/event lifecycle | 延后 |
| `runtime.mutation.ai_proposal.parent_facade_import_pass` | `ai_proposal.rs` | child module declaration / re-export / private helper import | 最后处理 |

---

## static_check 选择理由

BE-001ET-01 选择 `static_check.rs`，原因:

1. 它主要负责 hash identity、AI model identity、static check result、config domain binding 和 v4 source-kind gate。
2. 当前改动目标仍是显式 import 输入面，不触碰函数体、校验规则、测试语义或 release transition。
3. 它是 `proposal_creation` 的上游校验 helper，先收敛它能降低后续 create handler 的隐式输入面。
4. 文件内已有局部测试模块，适合作为单文件 import rewrite 的等价基线。

---

## BE-001ET-01 冻结范围

下一批只能建立 `runtime.mutation.ai_proposal.static_check_import_pass` 单子叶等价基线，冻结以下边界:

```text
validate_hash_identity
is_valid_hash_identity
validate_ai_model_identity
ai_proposal_static_check_result
is_v4_ai_proposal_target
expected_config_domain_for_target
validate_ai_proposal_config_domain_binding
analyze_v4_backtest_artifact_for_ai
v4_ai_proposal_static_check_tests
```

必须保持:

```text
no_static_check_rule_rewrite
no_hash_format_rewrite
no_config_domain_binding_rewrite
no_v4_source_kind_gate_rewrite
no_artifact_analysis_rewrite
no_visibility_rewrite
no_sibling_owner_migration
```

---

## 排除项

本批不处理:

1. 不改 Rust 代码。
2. 不处理 `static_check.rs` 顶部 import；这属于 BE-001ET-03。
3. 不处理其他 ai proposal child import residual。
4. 不处理 `src/runtime/mutation/ai_proposal.rs` parent facade。
5. 不处理 `src/runtime/mod.rs` root parent bridge。
6. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
7. 不新增 sibling 横向连接。
8. 不启动 release transition。

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

AI 声称 BE-001ES-01 完成时，必须说明:

1. 本批只是 `no code movement` 父叶残余判断。
2. `runtime.mutation.ai_proposal_import_pass stop_split: false`。
3. 下一步只能进入 BE-001ET-01 `runtime.mutation.ai_proposal.static_check_import_pass` 单子叶等价基线。
4. 不得宣称 ai proposal import、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `424-runtime.mutation.ai_proposal_import_pass第四轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶保持 `stop_split: false`，下一颗子叶固定为 `static_check_import_pass`。
3. BE-001ET-01 成为唯一下一步。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
