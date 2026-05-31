# v4.16.0 runtime.mutation.ai_proposal_import_pass 第五轮父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EU-01
> 上一批: `428-runtime.mutation.ai_proposal.static_check_import_pass单叶closeout.md`
> 目标父叶: `runtime.mutation.ai_proposal_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal_import_pass`
> 判定: `runtime.mutation.ai_proposal_import_pass stop_split: false`
> 代码动作: no code movement
> 下一步: BE-001EV-01 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EU-01 `runtime.mutation.ai_proposal_import_pass` 第五轮父叶残余判断 | 父叶重判 |
| 规范矩阵 | parent import bridge / explicit import pass / no release transition | 保持父叶继续拆分 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal_import_pass` | ai proposal import pass 父叶重判 |
| 模块树 | `runtime.mutation.ai_proposal_import_pass` | `stop_split: false` |

---

## 父叶重判结论

BE-001ET-04 已完成 `static_check.rs` import pocket closeout，但 `runtime.mutation.ai_proposal_import_pass` 父叶仍存在 7 个 production parent wildcard import residual。当前父叶不能 closeout，必须继续按单子叶方式处理。

```text
runtime.mutation.ai_proposal_import_pass fifth_parent_residual_judgment
runtime.mutation.ai_proposal_import_pass stop_split: false
event_lifecycle_import_pass_selected
remaining_runtime_parent_import_bridge_8
remaining_mutation_import_bridge_7
remaining_ai_proposal_import_bridge_7
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
src/runtime/mutation/ai_proposal/status_transition.rs
```

`src/runtime/mod.rs` 属于 root parent bridge，`src/runtime/mutation/ai_proposal.rs` 属于 ai proposal parent facade，均不在本轮直接处理。

---

## 子叶选择表

| 候选子叶 | 文件 | 判断 | 本轮决定 |
| --- | --- | --- | --- |
| `runtime.mutation.ai_proposal.event_lifecycle_import_pass` | `event_lifecycle.rs` | lifecycle event contract、event payload、entry builder 与 transition persistence；位于 create / review / status 流程之间，先显式化能降低后续 handler 的隐式输入面 | 采纳 |
| `runtime.mutation.ai_proposal.approval_persistence_import_pass` | `approval_persistence.rs` | approval disk persistence，范围较窄但依赖审批语义 | 延后 |
| `runtime.mutation.ai_proposal.approval_review_import_pass` | `approval_review.rs` | review / approve / reject route-facing handlers，依赖 event lifecycle、sandbox 与 persistence | 延后 |
| `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` | `sandbox_trigger.rs` | sandbox verification spawn 与 approval gate，涉及异步任务边界 | 延后 |
| `runtime.mutation.ai_proposal.status_transition_import_pass` | `status_transition.rs` | status transition helper，依赖 lifecycle 语义 | 延后 |
| `runtime.mutation.ai_proposal.proposal_creation_import_pass` | `proposal_creation.rs` | create handler，依赖 source/governance/static check/event lifecycle | 延后 |
| `runtime.mutation.ai_proposal.parent_facade_import_pass` | `ai_proposal.rs` | child module declaration / re-export / parent facade | 最后处理 |

---

## event_lifecycle 选择理由

BE-001EV-01 选择 `event_lifecycle.rs`，原因:

1. 它是 AI proposal create、review、status transition 之间共享的事件白箱，先显式化 import 能减少后续 handler 的隐式耦合。
2. 当前目标仍是 import 输入面显式化，不触碰 event contract、payload schema、lifecycle sequence、状态语义或持久化锁顺序。
3. 文件体量适中，且 public-in-parent 方法边界清晰，适合作为下一颗单子叶。
4. 它会调用 record persistence 与 in-memory index 更新，等价基线必须先冻结这些副作用边界。

---

## BE-001EV-01 冻结范围

下一批只能建立 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 单子叶等价基线，冻结以下边界:

```text
ai_proposal_event_contract
build_runtime_ai_proposal_event
ai_proposal_lifecycle_entry
persist_runtime_ai_proposal_transition
RuntimeEventEnvelope::default
persist_runtime_ai_proposal_record
auth::scoped_key
io_error
```

必须保持:

```text
no_event_contract_rewrite
no_event_payload_rewrite
no_lifecycle_sequence_rewrite
no_persistence_order_rewrite
no_status_semantics_rewrite
no_visibility_rewrite
no_sibling_owner_migration
```

---

## 排除项

本批不处理:

1. 不改 Rust 代码。
2. 不处理 `event_lifecycle.rs` 顶部 import；这属于 BE-001EV-03。
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

AI 声称 BE-001EU-01 完成时，必须说明:

1. 本批只是 `no code movement` 父叶残余判断。
2. `runtime.mutation.ai_proposal_import_pass stop_split: false`。
3. 下一步只能进入 BE-001EV-01 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 单子叶等价基线。
4. 不得宣称 ai proposal import、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `429-runtime.mutation.ai_proposal_import_pass第五轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶保持 `stop_split: false`，下一颗子叶固定为 `event_lifecycle_import_pass`。
3. BE-001EV-01 成为唯一下一步。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
