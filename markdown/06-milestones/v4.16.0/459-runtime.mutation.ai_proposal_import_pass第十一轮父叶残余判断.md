# v4.16.0 runtime.mutation.ai_proposal_import_pass 第十一轮父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FG-01
> 上一批: `458-runtime.mutation.ai_proposal.proposal_creation_import_pass单叶closeout.md`
> 目标父叶: `runtime.mutation.ai_proposal_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal_import_pass`
> 判定: `runtime.mutation.ai_proposal_import_pass stop_split: false`
> 代码动作: no code movement
> 下一步: BE-001FH-01 `runtime.mutation.ai_proposal.parent_facade_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FG-01 `runtime.mutation.ai_proposal_import_pass` 第十一轮父叶残余判断 | 父叶重判 |
| 规范矩阵 | parent import bridge / parent facade import pass / no release transition | 保持父叶继续拆分 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal_import_pass` | ai proposal import pass 父叶重判 |
| 模块树 | `runtime.mutation.ai_proposal_import_pass` | `stop_split: false` |

---

## 父叶重判结论

BE-001FF-04 已完成 `proposal_creation.rs` import pocket closeout，`runtime.mutation.ai_proposal_import_pass` 的 child import pockets 已全部完成。但父叶仍存在 `src/runtime/mutation/ai_proposal.rs` 的 parent facade residual，因此父叶不能 closeout，必须继续按单子叶方式处理 parent facade。

```text
runtime.mutation.ai_proposal_import_pass eleventh_parent_residual_judgment
runtime.mutation.ai_proposal_import_pass stop_split: false
parent_facade_import_pass_selected
remaining_runtime_parent_import_bridge_2
remaining_mutation_import_bridge_1
remaining_ai_proposal_import_bridge_1
old_three_leaf_pause_target_cancelled
progress_report_instruction_discarded
```

本批不改 Rust 代码，只做父叶残余判断和下一颗子叶选择。

---

## 当前 residual 清单

```text
src/runtime/mod.rs
src/runtime/mutation/ai_proposal.rs
```

`src/runtime/mutation/ai_proposal.rs` 属于 ai proposal parent facade import residual，应先于 root bridge 处理。`src/runtime/mod.rs` 属于 root parent bridge，不在本轮直接处理。

---

## 子叶选择表

| 候选子叶 | 文件 | 判断 | 本轮决定 |
| --- | --- | --- | --- |
| `runtime.mutation.ai_proposal.parent_facade_import_pass` | `ai_proposal.rs` | child module declaration / public re-export / parent facade glue / tests | 采纳 |
| `runtime.parent_import_bridge` | `runtime/mod.rs` | root runtime bridge | 延后到 ai proposal parent facade 后处理 |

---

## parent facade 选择理由

BE-001FH-01 选择 `src/runtime/mutation/ai_proposal.rs`，原因:

1. ai proposal 的 `record_query`、`source_governance_identity`、`static_check`、`event_lifecycle`、`approval_persistence`、`status_transition`、`sandbox_trigger`、`approval_review` 和 `proposal_creation` child import pockets 已全部显式化并 closeout。
2. `ai_proposal.rs` 当前仍保留 `use super::*`，并暴露出 child 显式化后的 parent facade unused imports。
3. parent facade 是 ai proposal import pass 内部最后一个 residual；不处理它就不能声明 `runtime.mutation.ai_proposal_import_pass` 完成。
4. root bridge `src/runtime/mod.rs` 必须等 ai proposal parent facade 收敛后再判断，避免越级处理。

---

## BE-001FH-01 冻结范围

下一批只能建立 `runtime.mutation.ai_proposal.parent_facade_import_pass` 单子叶等价基线，冻结以下边界:

```text
src/runtime/mutation/ai_proposal.rs
child module declarations
pub(crate) use approval_review handlers
pub(crate) use proposal_creation::create_runtime_ai_proposal
pub(crate) use record_query::{get_runtime_ai_proposal_detail, list_runtime_ai_proposals}
test module v4_ai_proposal_tests
RuntimeApprovalListQuery
parent facade unused imports
```

必须保持:

```text
no_child_module_declaration_rewrite
no_re_export_rewrite
no_test_semantics_rewrite
no_handler_body_rewrite
no_route_facade_rewrite
no_state_owner_migration
no_schema_owner_migration
no_sibling_owner_migration
no_release_transition
```

---

## parent warning 归属

当前 cargo warning 中的 ai proposal parent facade unused imports 归属后续 parent facade import pass，不在 BE-001FG-01 主动清理:

```text
parent facade unused imports remain deferred
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
axum::extract::Query
```

这些 warning 是 child import 显式化后的父级残余信号，不是本批失败条件。

---

## 排除项

本批不处理:

1. 不改 Rust 代码。
2. 不处理 `ai_proposal.rs` 顶部 import；这属于 BE-001FH 后续批次。
3. 不处理 `src/runtime/mod.rs` root parent bridge。
4. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
5. 不新增 sibling 横向连接。
6. 不启动 release transition。

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

AI 声称 BE-001FG-01 完成时，必须说明:

1. 本批只是 `no code movement` 父叶残余判断。
2. `runtime.mutation.ai_proposal_import_pass stop_split: false`。
3. 下一步只能进入 BE-001FH-01 `runtime.mutation.ai_proposal.parent_facade_import_pass` 单子叶等价基线。
4. 不得宣称 ai proposal parent facade、mutation import、parent import bridge、backend.runtime 或 Rust 重构已经完成。

---

## 验收标准

1. `459-runtime.mutation.ai_proposal_import_pass第十一轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶保持 `stop_split: false`，下一颗子叶固定为 `parent_facade_import_pass`。
3. BE-001FH-01 成为唯一下一步。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
