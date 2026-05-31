# v4.16.0 runtime.mutation_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DN-01
> 基准: `349-runtime.parent_import_bridge父叶残余判断.md`
> 目标子叶: `runtime.mutation_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DN-02 `runtime.mutation_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DN-01 `runtime.mutation_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | parent import bridge、explicit import pass、minimum batch、release transition guard | mutation import 基线 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation_import_pass` | mutation import 白箱 |
| 模块树 | `runtime.mutation_import_pass` | 新基线 |

---

## 当前事实

`runtime.mutation_import_pass` 是 parent import bridge 的 staged explicit import pass，不是新增业务 owner。当前 `src/runtime/mutation/**` 中仍有 21 个文件存在 `use super::*` 或 `super::` 依赖。

当前 parent bridge 总分布:

```text
root 1
run 0
backtest 0
mutation 21
test-only 1
total 23
```

---

## 目标文件清单

```text
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/approval_persistence.rs
src/runtime/mutation/ai_proposal/approval_review.rs
src/runtime/mutation/ai_proposal/event_lifecycle.rs
src/runtime/mutation/ai_proposal/proposal_creation.rs
src/runtime/mutation/ai_proposal/record_query.rs
src/runtime/mutation/ai_proposal/sandbox_trigger.rs
src/runtime/mutation/ai_proposal/source_governance_identity.rs
src/runtime/mutation/ai_proposal/static_check.rs
src/runtime/mutation/ai_proposal/status_transition.rs
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
src/runtime/mutation/shared_governance.rs
```

其余 parent bridge 排队区:

```text
src/runtime/mod.rs
src/runtime/run_guard.rs
```

---

## 白箱输入面

### `runtime.mutation.ai_proposal`

父级对外 handler / re-export:

```text
create_runtime_ai_proposal
list_runtime_ai_proposals
get_runtime_ai_proposal_detail
list_runtime_approvals
get_runtime_approval_detail
approve_ai_proposal
reject_ai_proposal
claim_ai_proposal_review
```

内部 handoff / helper:

```text
load_runtime_ai_proposal_for_user
persist_approval
load_approval_from_disk
ai_proposal_lifecycle_entry
build_runtime_ai_proposal_event
persist_runtime_ai_proposal_transition
ensure_ai_proposal_can_be_approved
spawn_ai_proposal_sandbox_verification
load_runtime_ai_proposal_source_context
runtime_ai_proposal_governance
runtime_ai_proposal_record_id
validate_hash_identity
validate_ai_model_identity
ai_proposal_static_check_result
ai_proposal_approved_status
update_ai_proposal_status
```

### `runtime.mutation.parameter_mutation`

父级对外 handler / re-export:

```text
create_runtime_parameter_mutation
list_runtime_parameter_mutations
get_runtime_parameter_mutation_detail
activate_runtime_parameter_mutation
rollback_runtime_parameter_mutation
```

内部 handoff / helper:

```text
runtime_parameter_mutation_record_id
validate_runtime_parameter_mutation_boundary
auto_snapshot_on_activation
resolve_runtime_parameter_mutation_boundary
evaluate_runtime_parameter_mutation_safe_window
runtime_parameter_mutation_rollback_record_id
mutation_lifecycle_entry
persist_runtime_parameter_mutation_transition
```

### `runtime.mutation.shared_governance`

共享 helper:

```text
canonical_runtime_parameter_version
validate_runtime_parameter_mutation_target
runtime_mode_from_events
status_contract_value
mutation_event_contract
build_runtime_parameter_mutation_event
append_parameter_mutation_events_to_run
runtime_parameter_mutation_governance
governance_with_parameter_version
```

---

## 等价边界

本基线只冻结事实，不进行 Rust import 改写。后续方案必须保持:

1. route facade、handler owner、state owner、persistence owner、schema owner、frontend caller 和 test asset 不迁移。
2. `create_runtime_ai_proposal`、approval lifecycle、AI sandbox gate、static check、config-domain binding、parameter mutation create/list/detail、activation/rollback lifecycle、shared governance event append 行为不变。
3. 子模块共享 helper 继续经父级受控 surface 或当前父子层级访问，不新增 sibling horizontal link。
4. 未进入 release transition 前，不得为了性能让 sibling child 横向直连。
5. `src/runtime/mod.rs` root parent bridge 与 test-only `src/runtime/run_guard.rs` 不属于本基线目标。

---

## 拆分判断输入

BE-001DN-02 必须先判断 21 文件是否继续拆分。当前基线倾向不允许一次性改写 21 文件，因为存在至少三个自然 pocket:

```text
runtime.mutation.ai_proposal_import_pass
runtime.mutation.parameter_mutation_import_pass
runtime.mutation.shared_governance_import_pass
```

判断依据:

1. `ai_proposal` 涉及 proposal creation、approval review、approval persistence、event lifecycle、sandbox trigger、source/governance/id、static check、status transition。
2. `parameter_mutation` 涉及 proposal creation、record query、transition lifecycle 及 activation/rollback 多层 pocket。
3. `shared_governance` 为 mutation 两侧共享 helper，风险比单文件 import rewrite 高，但可以独立验证。
4. mutation 子树已经是最后一个完整业务子树，不能为了加速破坏父子通信规则。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不直接改写 `src/runtime/mutation/**` import。
- 本批不处理 `src/runtime/mod.rs` root parent bridge。
- 本批不处理 test-only `src/runtime/run_guard.rs`。
- 本批不迁移 route facade、handler owner、state owner、persistence owner、schema owner、frontend caller 或 test asset。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。
- 本批不恢复旧的三叶暂停目标；递归队列继续保持 `old_three_leaf_pause_target_cancelled`。

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

AI 声称 BE-001DN-01 完成时，必须说明:

1. 本批次是 `no code movement` 单子叶等价基线。
2. `runtime.mutation_import_pass` 当前仍有 21 个 parent bridge 依赖文件。
3. 当前总分布为 root 1 / run 0 / backtest 0 / mutation 21 / test-only 1 / total 23。
4. 下一步只能进入 BE-001DN-02 `runtime.mutation_import_pass` 抽离方案。
5. BE-001DN-02 必须判断是否拆成 `runtime.mutation.ai_proposal_import_pass`、`runtime.mutation.parameter_mutation_import_pass` 与 `runtime.mutation.shared_governance_import_pass` 等 pocket。
6. 不得直接 21 文件一次性改写。
7. `src/runtime/mod.rs` 和 test-only `src/runtime/run_guard.rs` 尚未处理。
8. release transition 未启动，未新增 sibling horizontal link。
9. 旧的三叶暂停目标仍为取消状态。

不得宣称 mutation import 已完成、parent import bridge 已完全清除、backend.runtime 已完成、Rust 重构已完成或 root bridge 已处理。

---

## 验收标准

1. `350-runtime.mutation_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 基线冻结 21 个 mutation parent bridge 文件、public handler / re-export、内部 helper 和拆分候选。
3. 下一步固定为 BE-001DN-02 `runtime.mutation_import_pass` 抽离方案。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
